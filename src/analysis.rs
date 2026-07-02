use crate::types::{Formant, VocalProfile};
use triple_buffer::Input;

/// Number of mono samples per analysis frame. Sized so the YIN difference
/// function has room for its full lag range: window (`YIN_WINDOW`) plus the
/// maximum search lag must fit inside one frame. 2048 @ 44.1 kHz ≈ 46 ms.
pub const ANALYSIS_FRAME: usize = 2048;

/// Length of the GPU difference-function / prefix-sum buffers, in lags.
///
/// Must be a power of two so the single-workgroup scan covers it exactly
/// (`yin_scan.wgsl`'s `SCAN_LEN` must match this), ≤ `ANALYSIS_FRAME -
/// YIN_WINDOW` so every `x[j+tau]` stays in-bounds, and ≥ `sample_rate /
/// F0_MIN` (~883 @ 44.1 kHz / 50 Hz) so it covers the whole pitch search range.
/// 1024 satisfies all three.
pub const DIFF_LEN: usize = 1024;

/// Fallback spectral envelope used before the first voiced frame is analyzed,
/// and held through unvoiced gaps so the synthesis envelope never collapses.
const DEFAULT_FORMANTS: [Formant; 3] = [
    Formant {
        frequency: 500.0,
        bandwidth: 80.0,
    },
    Formant {
        frequency: 1500.0,
        bandwidth: 120.0,
    },
    Formant {
        frequency: 2500.0,
        bandwidth: 160.0,
    },
];

/// GPU side of the YIN pitch detector. Two compute passes per frame:
///   1. `yin_diff.wgsl`  — difference function `d(tau)` (the O(window × lags)
///      heavy part, one invocation per lag).
///   2. `yin_scan.wgsl`  — inclusive parallel prefix-sum of `d` into a separate
///      buffer, giving the CMND denominator `sum_{j=1..tau} d(j)`.
///
/// Both `d` and its prefix sum are read back; `math::yin_f0_from_diff_cumsum`
/// then forms `d'(tau)` and does the (cheap, sequential) lag search on the CPU.
/// The search core is shared with the CPU reference path, so GPU and CPU f0
/// agree by construction.
pub struct GpuYin {
    device: wgpu::Device,
    queue: wgpu::Queue,
    diff_pipeline: wgpu::ComputePipeline,
    scan_pipeline: wgpu::ComputePipeline,
    audio_buf: wgpu::Buffer,
    diff_buf: wgpu::Buffer,
    cumsum_buf: wgpu::Buffer,
    diff_readback: wgpu::Buffer,
    cumsum_readback: wgpu::Buffer,
    diff_bind_group: wgpu::BindGroup,
    scan_bind_group: wgpu::BindGroup,
}

impl GpuYin {
    pub async fn new() -> anyhow::Result<Self> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await?;

        let diff_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("YIN Diff Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("yin_diff.wgsl").into()),
        });
        let diff_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("YIN Diff Pipeline"),
            layout: None,
            module: &diff_shader,
            entry_point: Some("compute_diff"),
            compilation_options: Default::default(),
            cache: None,
        });

        let scan_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("YIN Scan Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("yin_scan.wgsl").into()),
        });
        let scan_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("YIN Scan Pipeline"),
            layout: None,
            module: &scan_shader,
            entry_point: Some("compute_scan"),
            compilation_options: Default::default(),
            cache: None,
        });

        let audio_bytes = (ANALYSIS_FRAME * std::mem::size_of::<f32>()) as u64;
        let diff_bytes = (DIFF_LEN * std::mem::size_of::<f32>()) as u64;

        let audio_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("yin-audio-in"),
            size: audio_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let diff_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("yin-diff"),
            size: diff_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let cumsum_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("yin-cumsum"),
            size: diff_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let diff_readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("yin-diff-readback"),
            size: diff_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let cumsum_readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("yin-cumsum-readback"),
            size: diff_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Diff pass: binding 0 = audio_in (read), 1 = diff_out (read_write).
        let diff_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("yin-diff-bind"),
            layout: &diff_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: audio_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: diff_buf.as_entire_binding(),
                },
            ],
        });
        // Scan pass: binding 0 = diff (read), 1 = cumsum (read_write).
        let scan_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("yin-scan-bind"),
            layout: &scan_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: diff_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: cumsum_buf.as_entire_binding(),
                },
            ],
        });

        Ok(Self {
            device,
            queue,
            diff_pipeline,
            scan_pipeline,
            audio_buf,
            diff_buf,
            cumsum_buf,
            diff_readback,
            cumsum_readback,
            diff_bind_group,
            scan_bind_group,
        })
    }

    /// Run both compute passes for one frame and read back `(d(tau), cumsum(tau))`
    /// for `tau in 0..DIFF_LEN`. Blocks on the GPU (this runs on the non-real-time
    /// analysis thread, so blocking is fine).
    pub fn analyze(&self, audio_in: &[f32]) -> anyhow::Result<(Vec<f32>, Vec<f32>)> {
        if audio_in.len() < ANALYSIS_FRAME {
            anyhow::bail!(
                "frame too short for GPU YIN: {} < {ANALYSIS_FRAME}",
                audio_in.len()
            );
        }
        let frame = &audio_in[..ANALYSIS_FRAME];
        self.queue
            .write_buffer(&self.audio_buf, 0, bytemuck::cast_slice(frame));

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("yin-encoder"),
            });

        // Pass 1: difference function. Separate pass from the scan so wgpu
        // inserts the read-after-write barrier on diff_buf between them.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.diff_pipeline);
            pass.set_bind_group(0, &self.diff_bind_group, &[]);
            pass.dispatch_workgroups((DIFF_LEN as u32).div_ceil(64), 1, 1);
        }
        // Pass 2: inclusive prefix-sum, single workgroup.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.scan_pipeline);
            pass.set_bind_group(0, &self.scan_bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }

        let diff_bytes = (DIFF_LEN * std::mem::size_of::<f32>()) as u64;
        encoder.copy_buffer_to_buffer(&self.diff_buf, 0, &self.diff_readback, 0, diff_bytes);
        encoder.copy_buffer_to_buffer(&self.cumsum_buf, 0, &self.cumsum_readback, 0, diff_bytes);
        self.queue.submit(Some(encoder.finish()));

        // Map both readback buffers; a single blocking poll drives both callbacks.
        let diff_slice = self.diff_readback.slice(..);
        let cumsum_slice = self.cumsum_readback.slice(..);
        let (dtx, drx) = std::sync::mpsc::channel();
        let (ctx, crx) = std::sync::mpsc::channel();
        diff_slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = dtx.send(r);
        });
        cumsum_slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = ctx.send(r);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| anyhow::anyhow!("device poll failed: {e:?}"))?;
        drx.recv()
            .map_err(|e| anyhow::anyhow!("diff map channel closed: {e:?}"))?
            .map_err(|e| anyhow::anyhow!("diff buffer map failed: {e:?}"))?;
        crx.recv()
            .map_err(|e| anyhow::anyhow!("cumsum map channel closed: {e:?}"))?
            .map_err(|e| anyhow::anyhow!("cumsum buffer map failed: {e:?}"))?;

        let diff: Vec<f32> = {
            let view = diff_slice.get_mapped_range();
            bytemuck::cast_slice::<u8, f32>(&view).to_vec()
        };
        let cumsum: Vec<f32> = {
            let view = cumsum_slice.get_mapped_range();
            bytemuck::cast_slice::<u8, f32>(&view).to_vec()
        };
        self.diff_readback.unmap();
        self.cumsum_readback.unmap();

        Ok((diff, cumsum))
    }
}

pub struct AnalysisEngine {
    gpu: GpuYin,
    profile_tx: Input<VocalProfile>,
    ui_profile_tx: Input<VocalProfile>,
    /// Last successfully measured formants, held across unvoiced frames.
    last_formants: [Formant; 3],
}

impl AnalysisEngine {
    pub async fn new(
        profile_tx: Input<VocalProfile>,
        ui_profile_tx: Input<VocalProfile>,
    ) -> anyhow::Result<Self> {
        let gpu = GpuYin::new().await?;
        Ok(Self {
            gpu,
            profile_tx,
            ui_profile_tx,
            last_formants: DEFAULT_FORMANTS,
        })
    }

    pub fn process_frame(&mut self, audio_in: &[f32], sample_rate: f32) -> anyhow::Result<()> {
        // --- Pitch (f0): GPU YIN (difference + prefix-sum) → shared CPU search ---
        // The GPU computes the difference function AND its inclusive prefix-sum
        // (the CMND denominator). The CPU forms d'(tau) and does the lag search
        // with the same code the CPU reference uses, so f0 agrees by construction.
        // Any GPU failure falls back to the pure-CPU YIN.
        let pitch = match self.gpu.analyze(audio_in) {
            Ok((diff, cumsum)) => crate::math::yin_f0_from_diff_cumsum(&diff, &cumsum, sample_rate),
            Err(e) => {
                eprintln!("GPU YIN dispatch failed ({e:?}); using CPU fallback");
                crate::math::yin_pitch(audio_in, sample_rate)
            }
        };
        let (f0, voiced) = match pitch {
            Some(p) if p.confidence > 0.4 && (50.0..=1000.0).contains(&p.f0) => (p.f0, true),
            _ => (0.0, false),
        };

        // --- Formants via LPC on a formant-band-decimated signal ---
        // Only analyzed on voiced frames; the last good set is held otherwise
        // (formants are ill-defined for silence/noise). Pipeline:
        //   decimate to ~11 kHz → pre-emphasis+Hamming → autocorrelation →
        //   Levinson-Durbin → Aberth root-finding → pole→(freq,bandwidth).
        if voiced {
            // Decimate so LPC spends its poles on the 0..~5.5 kHz formant band
            // rather than the full 22 kHz, where a low order can't resolve them.
            let m = ((sample_rate / 11_025.0).round() as usize).max(1);
            let fs_dec = sample_rate / m as f32;
            let order = (2 + (fs_dec / 1000.0) as usize).clamp(8, 20);

            let decimated = crate::math::decimate(audio_in, m);
            let lpc = crate::math::lpc_coefficients(&decimated, order, 0.97);
            let measured = crate::math::formants_from_lpc(&lpc, fs_dec);

            // Accept only if we actually resolved at least F1.
            if measured[0].frequency > 0.0 {
                self.last_formants = measured;
            }
        }
        let formants = self.last_formants;

        // Harmonic series at k·f0 (drives the musician-facing ladder);
        // silent when unvoiced rather than holding stale bars.
        let partial_amplitudes = if voiced {
            crate::math::harmonic_amplitudes(audio_in, sample_rate, f0)
        } else {
            [0.0; crate::types::MAX_PARTIALS]
        };

        let profile = VocalProfile {
            f0,
            formants,
            partial_amplitudes,
            valid: voiced,
        };

        // Publish to synthesis + UI (VocalProfile is Copy).
        self.profile_tx.write(profile);
        self.ui_profile_tx.write(profile);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the GPU difference function AND the GPU inclusive prefix-sum
    /// match the CPU reference, and therefore that GPU and CPU f0 agree. Skips
    /// cleanly if no GPU adapter is available in the environment.
    #[test]
    fn gpu_yin_matches_cpu() {
        let gpu = match pollster::block_on(GpuYin::new()) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("skipping GPU parity test — no adapter: {e:?}");
                return;
            }
        };

        let sr = 44_100.0;
        let buf: Vec<f32> = (0..ANALYSIS_FRAME)
            .map(|i| (2.0 * std::f32::consts::PI * 220.0 * i as f32 / sr).sin())
            .collect();

        let (gpu_diff, gpu_cumsum) = gpu.analyze(&buf).expect("gpu analyze");
        let cpu_diff =
            crate::math::yin_difference(&buf, crate::math::YIN_WINDOW, gpu_diff.len() - 1);

        // CPU inclusive prefix-sum of d (the reference for the GPU scan).
        let mut cpu_cumsum = vec![0.0f32; cpu_diff.len()];
        let mut running = 0.0f32;
        for tau in 1..cpu_diff.len() {
            running += cpu_diff[tau];
            cpu_cumsum[tau] = running;
        }

        assert_eq!(gpu_diff.len(), DIFF_LEN);
        assert_eq!(gpu_cumsum.len(), DIFF_LEN);
        assert!(gpu_diff[0].abs() < 1e-3, "d(0) = {}", gpu_diff[0]);
        assert!(gpu_cumsum[0].abs() < 1e-3, "cumsum(0) = {}", gpu_cumsum[0]);

        // Difference function parity (f32 both sides, same summation order).
        for tau in 1..gpu_diff.len() {
            let (g, c) = (gpu_diff[tau], cpu_diff[tau]);
            assert!(
                (g - c).abs() <= 1e-2 * (c.abs() + 1.0),
                "diff mismatch at tau={tau}: gpu={g} cpu={c}"
            );
        }

        // Prefix-sum parity. Looser relative tolerance: the GPU tree-sum and the
        // CPU sequential sum accumulate f32 rounding in different orders.
        for tau in 1..gpu_cumsum.len() {
            let (g, c) = (gpu_cumsum[tau], cpu_cumsum[tau]);
            assert!(
                (g - c).abs() <= 5e-3 * (c.abs() + 1.0),
                "cumsum mismatch at tau={tau}: gpu={g} cpu={c}"
            );
        }

        // And therefore f0 agrees (via the GPU cumsum path) and is correct.
        let gpu_f0 = crate::math::yin_f0_from_diff_cumsum(&gpu_diff, &gpu_cumsum, sr)
            .expect("gpu f0")
            .f0;
        let cpu_f0 = crate::math::yin_f0_from_diff(&cpu_diff, sr)
            .expect("cpu f0")
            .f0;
        assert!(
            (gpu_f0 - cpu_f0).abs() < 2.0,
            "gpu {gpu_f0} vs cpu {cpu_f0}"
        );
        assert!((gpu_f0 - 220.0).abs() < 5.0, "gpu f0 {gpu_f0} ~ 220 Hz");
    }
}
