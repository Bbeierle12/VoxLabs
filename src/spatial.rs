//! TV-path spatial calibration and live path-consistency scoring.
//!
//! The simultaneous-source research (see the "Live or Loudspeaker" brief,
//! section 07) established that the one thing about the TV that survives
//! every program is its *spatial transfer path*: for a single dominant
//! source, the ratio of what two microphones hear, Y1(f)/Y2(f) =
//! H1(f)/H2(f), cancels the program entirely and leaves only the fixed
//! physics of driver → room → phone. The device probe measured (25 Aug,
//! Pixel) that this phone hands the app two genuinely independent channels
//! (corr 0.351 between mics ~14 cm apart), so that ratio is learnable here.
//!
//! Calibration (TV playing, user silent, phone in its usual spot)
//! accumulates the 2×2 spatial covariance R(f) per STFT bin; its dominant
//! eigenvector is the normalized relative-transfer vector ĥ(f), and the
//! magnitude-squared coherence γ²(f) says how much that bin was actually
//! excited by one coherent source (diffuse noise and silence give γ² ≈ 0).
//! Live scoring projects each frame's per-bin channel vector ŷ(f) onto
//! ĥ(f): c_T(f) = |ĥᴴŷ|² ∈ [0, 1] by Cauchy–Schwarz — near 1 when the
//! energy arrives through the calibrated TV path — and aggregates over the
//! band with weights = bin energy × calibration coherence.
//!
//! HONESTY BOUNDARY (mirror this in every UI string):
//!   * Two microphones make this *evidence, not proof*. Any source standing
//!     at (or near) the TV matches the path; a second TV driver or a moved
//!     phone breaks it. M = 2 supports one spatial constraint — the
//!     research's M ≥ D+1 rule — so a stereo soundbar is only partially
//!     modeled by the dominant direction.
//!   * A low score means "not the calibrated path", never "live human".
//!   * The calibration is per-session and dies the moment the phone moves;
//!     the eigen-decomposition can't tell "two sources during calibration"
//!     from "one source" better than the rank ratio λ2/λ1 it reports — so
//!     that number is surfaced, not swallowed.

use rustfft::num_complex::Complex;
use std::sync::Mutex;

/// STFT frame length (samples). 2048 @ 48 kHz ≈ 43 ms — matches the analysis
/// engine's frame scale; 23.4 Hz bins.
pub const FFT_N: usize = 2048;
/// STFT hop (samples). 50% overlap.
pub const HOP: usize = 1024;
/// Frames of TV-only audio a calibration accumulates (512 hops ≈ 11 s).
pub const CALIB_FRAMES: usize = 512;
/// Band used for calibration and scoring. Below ~200 Hz the phone's port
/// roll-off and processed capture make the path unreliable; above ~8 kHz
/// TV content is sparse (codec ceiling) and bins go uncalibrated anyway.
pub const BAND_LO_HZ: f32 = 200.0;
pub const BAND_HI_HZ: f32 = 8000.0;
/// Frame RMS (dBFS, both channels) below which no score is computed —
/// scoring silence would manufacture a number from noise.
pub const QUIET_DBFS: f32 = -65.0;
/// EMA coefficient for the displayed score (per scored frame, ~21 ms).
pub const SCORE_EMA_ALPHA: f32 = 0.1;
/// A bin's calibration weight is its coherence γ²; below this it counts as
/// uncovered for the band-coverage statistic.
pub const COVERED_COHERENCE: f32 = 0.5;
/// Energy-weighted mean λ2/λ1 above which the calibration warns that more
/// than one source was audible while it learned. Calibrated against the
/// unit tests' geometry: a clean single path measures ≈ 0.01, two
/// equal-power sources at TV-like vs mouth-like paths measure ≈ 0.26 —
/// 0.2 splits them with margin, and the raw ratio is always displayed so
/// a borderline pass is visible rather than silently blessed.
pub const RANK_WARN: f32 = 0.2;
/// During calibration, quiet frames don't count; if fewer than
/// [`CALIB_FRAMES`] usable frames arrive in this many wall frames, the
/// calibration fails loudly instead of finishing on junk.
pub const CALIB_MAX_WALL_FACTOR: usize = 3;
/// UI display thresholds for the live score — visible in copy, not hidden.
pub const SCORE_TV: f32 = 0.85;
pub const SCORE_NOT_TV: f32 = 0.40;

/// One learned TV path: per-bin normalized relative-transfer vectors plus
/// the quality numbers a user needs to judge it.
#[derive(Clone, Debug)]
pub struct PathCalibration {
    // Provenance fields: not read by live scoring (the processor keeps its
    // own band), but they are what future persistence / validation of a
    // stored calibration will check against.
    #[allow(dead_code)]
    pub sample_rate: f32,
    /// First and last (inclusive) FFT bin in the calibrated band.
    #[allow(dead_code)]
    pub bin_lo: usize,
    #[allow(dead_code)]
    pub bin_hi: usize,
    /// Normalized dominant eigenvector per band bin, `[L, R]` convention.
    pub h: Vec<[Complex<f32>; 2]>,
    /// Per-bin weight: magnitude-squared coherence γ² ∈ [0, 1].
    pub w: Vec<f32>,
    /// Fraction of band bins with γ² ≥ [`COVERED_COHERENCE`].
    pub band_coverage: f32,
    /// Energy-weighted mean γ² over the band — overall "how well did the
    /// TV's audio actually pin down a path".
    pub quality: f32,
    /// Energy-weighted mean λ2/λ1 — 0 for one clean source; grows when a
    /// second source (or heavy diffuse noise) shared the calibration.
    pub rank_ratio: f32,
    pub two_source_warning: bool,
}

/// What one processed hop produced.
#[derive(Clone, Debug, PartialEq)]
pub enum FrameOutcome {
    /// Calibration consumed the frame; this many usable frames remain.
    CalibrationProgress { frames_left: usize },
    /// Calibration finished on this frame.
    CalibrationDone,
    /// Calibration gave up: not enough non-quiet frames.
    CalibrationFailed(String),
    /// Frame below the quiet floor — no score.
    Quiet,
    /// Scored against the current calibration.
    Scored { score: f32, ema: f32 },
}

/// Streaming two-channel STFT processor: accumulates calibration, then
/// scores. Pure Rust, cross-target, unit-tested — the Android capture
/// thread merely feeds it samples.
pub struct SpatialProcessor {
    sample_rate: f32,
    bin_lo: usize,
    bin_hi: usize,
    window: Vec<f32>,
    acc_l: Vec<f32>,
    acc_r: Vec<f32>,
    // Calibration accumulators (band bins only).
    calibrating: bool,
    calib_used: usize,
    calib_wall: usize,
    c11: Vec<f64>,
    c22: Vec<f64>,
    c12: Vec<Complex<f64>>,
    calibration: Option<PathCalibration>,
    score_ema: Option<f32>,
}

impl SpatialProcessor {
    pub fn new(sample_rate: f32) -> Self {
        let hz_per_bin = sample_rate / FFT_N as f32;
        let bin_lo = (BAND_LO_HZ / hz_per_bin).ceil() as usize;
        let bin_hi = ((BAND_HI_HZ / hz_per_bin).floor() as usize).min(FFT_N / 2);
        let n_bins = bin_hi - bin_lo + 1;
        let window: Vec<f32> = (0..FFT_N)
            .map(|i| {
                let t = i as f32 / FFT_N as f32;
                0.5 - 0.5 * (2.0 * std::f32::consts::PI * t).cos()
            })
            .collect();
        Self {
            sample_rate,
            bin_lo,
            bin_hi,
            window,
            acc_l: Vec::with_capacity(FFT_N * 2),
            acc_r: Vec::with_capacity(FFT_N * 2),
            calibrating: false,
            calib_used: 0,
            calib_wall: 0,
            c11: vec![0.0; n_bins],
            c22: vec![0.0; n_bins],
            c12: vec![Complex::new(0.0, 0.0); n_bins],
            calibration: None,
            score_ema: None,
        }
    }

    pub fn calibration(&self) -> Option<&PathCalibration> {
        self.calibration.as_ref()
    }

    /// Begin (or restart) learning the TV path. Any previous calibration
    /// stays in effect until the new one completes — a failed re-learn must
    /// not silently destroy a working one.
    pub fn begin_calibration(&mut self) {
        self.calibrating = true;
        self.calib_used = 0;
        self.calib_wall = 0;
        self.c11.iter_mut().for_each(|v| *v = 0.0);
        self.c22.iter_mut().for_each(|v| *v = 0.0);
        self.c12
            .iter_mut()
            .for_each(|v| *v = Complex::new(0.0, 0.0));
    }

    /// Feed equal-length channel slices; returns one outcome per completed
    /// hop (possibly none, possibly several).
    pub fn push(&mut self, l: &[f32], r: &[f32]) -> Vec<FrameOutcome> {
        let n = l.len().min(r.len());
        self.acc_l.extend_from_slice(&l[..n]);
        self.acc_r.extend_from_slice(&r[..n]);
        let mut out = Vec::new();
        while self.acc_l.len() >= FFT_N {
            let outcome = self.process_hop();
            out.push(outcome);
            self.acc_l.drain(..HOP);
            self.acc_r.drain(..HOP);
        }
        out
    }

    fn process_hop(&mut self) -> FrameOutcome {
        // Quiet gate on the raw frame, before any spectral work.
        let mut energy = 0.0f64;
        for i in 0..FFT_N {
            let (a, b) = (self.acc_l[i] as f64, self.acc_r[i] as f64);
            energy += a * a + b * b;
        }
        let rms_db = 10.0 * (energy / (2 * FFT_N) as f64).max(1e-18).log10() as f32;
        let quiet = rms_db < QUIET_DBFS;

        if self.calibrating {
            self.calib_wall += 1;
            if quiet {
                if self.calib_wall >= CALIB_FRAMES * CALIB_MAX_WALL_FACTOR {
                    self.calibrating = false;
                    return FrameOutcome::CalibrationFailed(
                        "too little TV energy — turn the TV up and re-learn".into(),
                    );
                }
                return FrameOutcome::CalibrationProgress {
                    frames_left: CALIB_FRAMES - self.calib_used,
                };
            }
            let (sl, sr) = self.spectra();
            for (i, bin) in (self.bin_lo..=self.bin_hi).enumerate() {
                let a = sl[bin];
                let b = sr[bin];
                let a64 = Complex::new(a.re as f64, a.im as f64);
                let b64 = Complex::new(b.re as f64, b.im as f64);
                self.c11[i] += a64.norm_sqr();
                self.c22[i] += b64.norm_sqr();
                self.c12[i] += a64 * b64.conj();
            }
            self.calib_used += 1;
            if self.calib_used >= CALIB_FRAMES {
                self.finalize_calibration();
                return FrameOutcome::CalibrationDone;
            }
            if self.calib_wall >= CALIB_FRAMES * CALIB_MAX_WALL_FACTOR {
                self.calibrating = false;
                return FrameOutcome::CalibrationFailed(
                    "too little TV energy — turn the TV up and re-learn".into(),
                );
            }
            return FrameOutcome::CalibrationProgress {
                frames_left: CALIB_FRAMES - self.calib_used,
            };
        }

        if quiet {
            return FrameOutcome::Quiet;
        }
        let Some(calib) = &self.calibration else {
            return FrameOutcome::Quiet; // nothing to score against yet
        };
        let (sl, sr) = self.spectra();
        let mut num = 0.0f64;
        let mut den = 0.0f64;
        for (i, bin) in (self.bin_lo..=self.bin_hi).enumerate() {
            let w = calib.w[i] as f64;
            if w <= 1e-3 {
                continue;
            }
            let y0 = sl[bin];
            let y1 = sr[bin];
            let e = (y0.norm_sqr() + y1.norm_sqr()) as f64;
            if e <= 0.0 {
                continue;
            }
            let h = &calib.h[i];
            let proj = h[0].conj() * y0 + h[1].conj() * y1;
            let c = (proj.norm_sqr() as f64 / e).clamp(0.0, 1.0);
            num += w * e * c;
            den += w * e;
        }
        if den <= 1e-12 {
            return FrameOutcome::Quiet;
        }
        let score = (num / den) as f32;
        let ema = match self.score_ema {
            Some(prev) => prev + SCORE_EMA_ALPHA * (score - prev),
            None => score,
        };
        self.score_ema = Some(ema);
        FrameOutcome::Scored { score, ema }
    }

    /// Windowed FFT of the oldest FFT_N samples of each channel.
    fn spectra(&self) -> (Vec<Complex<f32>>, Vec<Complex<f32>>) {
        let fft = crate::math::fft_forward(FFT_N);
        let mut bl: Vec<Complex<f32>> = self
            .acc_l
            .iter()
            .take(FFT_N)
            .zip(&self.window)
            .map(|(x, w)| Complex::new(x * w, 0.0))
            .collect();
        let mut br: Vec<Complex<f32>> = self
            .acc_r
            .iter()
            .take(FFT_N)
            .zip(&self.window)
            .map(|(x, w)| Complex::new(x * w, 0.0))
            .collect();
        fft.process(&mut bl);
        fft.process(&mut br);
        (bl, br)
    }

    /// Closed-form 2×2 Hermitian eigen-decomposition per bin; dominant
    /// eigenvector becomes ĥ, coherence becomes the weight, λ2/λ1 the
    /// honesty number.
    fn finalize_calibration(&mut self) {
        self.calibrating = false;
        let n_bins = self.c11.len();
        let mut h = Vec::with_capacity(n_bins);
        let mut w = Vec::with_capacity(n_bins);
        let mut covered = 0usize;
        let mut e_sum = 0.0f64;
        let mut q_sum = 0.0f64;
        let mut r_sum = 0.0f64;
        for i in 0..n_bins {
            let a = self.c11[i];
            let b = self.c22[i];
            let c = self.c12[i];
            let e = a + b;
            let coh = (c.norm_sqr() / (a * b).max(1e-30)).clamp(0.0, 1.0) as f32;
            let disc = ((a - b) * (a - b) + 4.0 * c.norm_sqr()).max(0.0).sqrt();
            let l1 = 0.5 * (a + b + disc);
            let l2 = (0.5 * (a + b - disc)).max(0.0);
            // Eigenvector of [[a, c], [c*, b]] for λ1: (a−λ1)v0 + c·v1 = 0
            // → v = [c, λ1−a]; degenerate c ≈ 0 falls back to an axis.
            let v = if c.norm_sqr() > 1e-30 {
                [c, Complex::new(l1 - a, 0.0)]
            } else if a >= b {
                [Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)]
            } else {
                [Complex::new(0.0, 0.0), Complex::new(1.0, 0.0)]
            };
            let norm = (v[0].norm_sqr() + v[1].norm_sqr()).sqrt().max(1e-30);
            h.push([
                Complex::new((v[0].re / norm) as f32, (v[0].im / norm) as f32),
                Complex::new((v[1].re / norm) as f32, (v[1].im / norm) as f32),
            ]);
            w.push(coh);
            if coh >= COVERED_COHERENCE {
                covered += 1;
            }
            e_sum += e;
            q_sum += e * coh as f64;
            r_sum += e * if l1 > 1e-30 { l2 / l1 } else { 0.0 };
        }
        let quality = if e_sum > 0.0 {
            (q_sum / e_sum) as f32
        } else {
            0.0
        };
        let rank_ratio = if e_sum > 0.0 {
            (r_sum / e_sum) as f32
        } else {
            0.0
        };
        self.score_ema = None;
        self.calibration = Some(PathCalibration {
            sample_rate: self.sample_rate,
            bin_lo: self.bin_lo,
            bin_hi: self.bin_hi,
            h,
            w,
            band_coverage: covered as f32 / n_bins as f32,
            quality,
            rank_ratio,
            two_source_warning: rank_ratio > RANK_WARN,
        });
    }
}

// ─── Shared UI-visible state ────────────────────────────────────────────────

/// Capture-side facts for the card's health line.
#[derive(Clone, Debug)]
pub struct CaptureHealth {
    pub source: &'static str,
    pub mask: &'static str,
    pub sample_rate: u32,
    /// Chunks read so far — logged, not displayed.
    #[allow(dead_code)]
    pub chunks: u64,
}

/// Live-score snapshot for the card.
#[derive(Clone, Debug, Default)]
pub struct LiveScore {
    pub ema: Option<f32>,
    pub last: Option<f32>,
    /// True when recent frames were under the quiet floor.
    pub quiet: bool,
    pub scored_frames: u64,
}

/// TV-path lifecycle, snapshotted by the UI each frame.
#[derive(Clone, Debug)]
pub enum SpatialStatus {
    Off,
    /// Capture starting up (recorder being built).
    Starting,
    Calibrating {
        seconds_left: f32,
        health: CaptureHealth,
    },
    Ready {
        quality: f32,
        band_coverage: f32,
        rank_ratio: f32,
        two_source_warning: bool,
        live: LiveScore,
        health: CaptureHealth,
    },
    Failed(String),
}

static STATE: Mutex<SpatialStatus> = Mutex::new(SpatialStatus::Off);

pub fn status() -> SpatialStatus {
    match STATE.lock() {
        Ok(s) => s.clone(),
        Err(p) => p.into_inner().clone(),
    }
}

fn set_status(s: SpatialStatus) {
    match STATE.lock() {
        Ok(mut slot) => *slot = s,
        Err(p) => *p.into_inner() = s,
    }
}

// ─── Android capture thread ─────────────────────────────────────────────────

#[cfg(target_os = "android")]
pub use capture::{request_shutdown, start_learning};

#[cfg(target_os = "android")]
mod capture {
    use super::*;
    use jni::objects::{JObject, JString, JValue};
    use jni::{Env, jni_sig, jni_str};
    use std::sync::atomic::{AtomicBool, Ordering};

    // Kept in sync with device_probe::android — same frozen public API
    // constants, same validated configuration ladder.
    const SOURCE_VOICE_RECOGNITION: i32 = 6;
    const SOURCE_UNPROCESSED: i32 = 9;
    const ENCODING_PCM_16BIT: i32 = 2;
    const CHANNEL_IN_STEREO: i32 = 0xC;
    const STATE_INITIALIZED: i32 = 1;
    const RECORDSTATE_RECORDING: i32 = 3;
    /// 0.1 s of interleaved stereo i16 at 48 kHz.
    const CHUNK: usize = 9600;
    const SAMPLE_RATE: u32 = 48_000;

    static RUNNING: AtomicBool = AtomicBool::new(false);
    static SHUTDOWN: AtomicBool = AtomicBool::new(false);
    static RELEARN: AtomicBool = AtomicBool::new(false);

    struct JErr(String);
    impl From<jni::errors::Error> for JErr {
        fn from(e: jni::errors::Error) -> Self {
            JErr(format!("{e}"))
        }
    }

    /// Stop the capture thread (activity relaunch). Idempotent.
    pub fn request_shutdown() {
        SHUTDOWN.store(true, Ordering::Relaxed);
    }

    /// User tapped LEARN / RE-LEARN: start the capture thread if it isn't
    /// running, and queue a (re-)calibration either way.
    pub fn start_learning() {
        RELEARN.store(true, Ordering::Relaxed);
        if RUNNING.swap(true, Ordering::Relaxed) {
            return; // thread already up; it will notice the flag
        }
        SHUTDOWN.store(false, Ordering::Relaxed);
        set_status(SpatialStatus::Starting);
        std::thread::spawn(|| {
            let outcome = run_capture();
            RUNNING.store(false, Ordering::Relaxed);
            match outcome {
                Ok(()) => {
                    // Deliberate shutdown (relaunch): back to Off quietly.
                    set_status(SpatialStatus::Off);
                }
                Err(JErr(e)) => {
                    log::error!("spatial capture failed: {e}");
                    set_status(SpatialStatus::Failed(e));
                }
            }
        });
    }

    fn clear_exception(env: &mut Env) {
        if env.exception_check() {
            env.exception_clear();
        }
    }

    fn run_capture() -> Result<(), JErr> {
        let ctx = ndk_context::android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) };
        let activity_ptr = ctx.context();
        vm.attach_current_thread(|env| {
            let activity = unsafe { JObject::from_raw(env, activity_ptr as jni::sys::jobject) };
            capture_loop(env, &activity)
        })
    }

    fn unprocessed_supported(env: &mut Env, activity: &JObject) -> bool {
        let r = (|| -> Result<bool, JErr> {
            let name = env.new_string("audio")?;
            let am = env
                .call_method(
                    activity,
                    jni_str!("getSystemService"),
                    jni_sig!("(Ljava/lang/String;)Ljava/lang/Object;"),
                    &[JValue::Object(&name)],
                )?
                .l()?;
            if am.is_null() {
                return Ok(false);
            }
            let key = env.new_string("android.media.property.SUPPORT_AUDIO_SOURCE_UNPROCESSED")?;
            let v = env
                .call_method(
                    &am,
                    jni_str!("getProperty"),
                    jni_sig!("(Ljava/lang/String;)Ljava/lang/String;"),
                    &[JValue::Object(&key)],
                )?
                .l()?;
            if v.is_null() {
                return Ok(false);
            }
            let js = unsafe { JString::from_raw(env, v.into_raw()) };
            Ok(js.try_to_string(env).map(|s| s == "true").unwrap_or(false))
        })();
        match r {
            Ok(b) => b,
            Err(_) => {
                clear_exception(env);
                false
            }
        }
    }

    /// Build a recorder from the same ladder the probe validated; returns
    /// the recorder plus which (source, mask) succeeded.
    fn build_recorder<'l>(
        env: &mut Env<'l>,
        activity: &JObject,
    ) -> Result<(JObject<'l>, &'static str, &'static str), JErr> {
        let mut ladder: Vec<(&'static str, i32, &'static str, bool)> = Vec::new();
        if unprocessed_supported(env, activity) {
            ladder.push(("UNPROCESSED", SOURCE_UNPROCESSED, "index mask 0|1", true));
            ladder.push(("UNPROCESSED", SOURCE_UNPROCESSED, "position stereo", false));
        }
        ladder.push((
            "VOICE_RECOGNITION",
            SOURCE_VOICE_RECOGNITION,
            "index mask 0|1",
            true,
        ));
        ladder.push((
            "VOICE_RECOGNITION",
            SOURCE_VOICE_RECOGNITION,
            "position stereo",
            false,
        ));
        let mut last = String::from("no configuration attempted");
        for (source_name, source, mask_name, index_mask) in ladder {
            match try_build(env, source, index_mask) {
                Ok(rec) => return Ok((rec, source_name, mask_name)),
                Err(JErr(e)) => {
                    clear_exception(env);
                    last = format!("{source_name} + {mask_name}: {e}");
                }
            }
        }
        Err(JErr(format!(
            "all recorder configurations failed; last: {last}"
        )))
    }

    fn try_build<'l>(
        env: &mut Env<'l>,
        source: i32,
        index_mask: bool,
    ) -> Result<JObject<'l>, JErr> {
        let fmt_builder = env.new_object(
            jni_str!("android/media/AudioFormat$Builder"),
            jni_sig!("()V"),
            &[],
        )?;
        env.call_method(
            &fmt_builder,
            jni_str!("setSampleRate"),
            jni_sig!("(I)Landroid/media/AudioFormat$Builder;"),
            &[JValue::Int(SAMPLE_RATE as i32)],
        )?;
        env.call_method(
            &fmt_builder,
            jni_str!("setEncoding"),
            jni_sig!("(I)Landroid/media/AudioFormat$Builder;"),
            &[JValue::Int(ENCODING_PCM_16BIT)],
        )?;
        if index_mask {
            env.call_method(
                &fmt_builder,
                jni_str!("setChannelIndexMask"),
                jni_sig!("(I)Landroid/media/AudioFormat$Builder;"),
                &[JValue::Int(0b11)],
            )?;
        } else {
            env.call_method(
                &fmt_builder,
                jni_str!("setChannelMask"),
                jni_sig!("(I)Landroid/media/AudioFormat$Builder;"),
                &[JValue::Int(CHANNEL_IN_STEREO)],
            )?;
        }
        let format = env
            .call_method(
                &fmt_builder,
                jni_str!("build"),
                jni_sig!("()Landroid/media/AudioFormat;"),
                &[],
            )?
            .l()?;
        let rec_builder = env.new_object(
            jni_str!("android/media/AudioRecord$Builder"),
            jni_sig!("()V"),
            &[],
        )?;
        env.call_method(
            &rec_builder,
            jni_str!("setAudioSource"),
            jni_sig!("(I)Landroid/media/AudioRecord$Builder;"),
            &[JValue::Int(source)],
        )?;
        env.call_method(
            &rec_builder,
            jni_str!("setAudioFormat"),
            jni_sig!("(Landroid/media/AudioFormat;)Landroid/media/AudioRecord$Builder;"),
            &[JValue::Object(&format)],
        )?;
        // 2 s of headroom so a UI stall can't overrun the HAL buffer.
        env.call_method(
            &rec_builder,
            jni_str!("setBufferSizeInBytes"),
            jni_sig!("(I)Landroid/media/AudioRecord$Builder;"),
            &[JValue::Int((SAMPLE_RATE * 2 * 2 * 2) as i32)],
        )?;
        let rec = env
            .call_method(
                &rec_builder,
                jni_str!("build"),
                jni_sig!("()Landroid/media/AudioRecord;"),
                &[],
            )?
            .l()?;
        let state = env
            .call_method(&rec, jni_str!("getState"), jni_sig!("()I"), &[])?
            .i()?;
        if state != STATE_INITIALIZED {
            let _ = env.call_method(&rec, jni_str!("release"), jni_sig!("()V"), &[]);
            return Err(JErr(format!("recorder state {state}")));
        }
        Ok(rec)
    }

    fn capture_loop(env: &mut Env, activity: &JObject) -> Result<(), JErr> {
        let (rec, source, mask) = build_recorder(env, activity)?;
        env.call_method(&rec, jni_str!("startRecording"), jni_sig!("()V"), &[])?;
        let rec_state = env
            .call_method(&rec, jni_str!("getRecordingState"), jni_sig!("()I"), &[])?
            .i()?;
        if rec_state != RECORDSTATE_RECORDING {
            let _ = env.call_method(&rec, jni_str!("release"), jni_sig!("()V"), &[]);
            return Err(JErr(format!("recording state {rec_state}")));
        }
        log::info!("spatial capture running: {source} · {mask} · {SAMPLE_RATE} Hz");

        let mut processor = SpatialProcessor::new(SAMPLE_RATE as f32);
        let chunk_arr = env.new_short_array(CHUNK)?;
        let mut buf = vec![0i16; CHUNK];
        let mut left = vec![0f32; CHUNK / 2];
        let mut right = vec![0f32; CHUNK / 2];
        let mut chunks: u64 = 0;
        let mut live = LiveScore::default();
        let mut zero_reads = 0u32;

        while !SHUTDOWN.load(Ordering::Relaxed) {
            if RELEARN.swap(false, Ordering::Relaxed) {
                processor.begin_calibration();
                live = LiveScore::default();
            }
            let n = env
                .call_method(
                    &rec,
                    jni_str!("read"),
                    jni_sig!("([SII)I"),
                    &[
                        JValue::Object(&chunk_arr),
                        JValue::Int(0),
                        JValue::Int(CHUNK as i32),
                    ],
                )?
                .i()?;
            if n <= 0 {
                zero_reads += 1;
                if zero_reads > 20 {
                    let _ = env.call_method(&rec, jni_str!("stop"), jni_sig!("()V"), &[]);
                    let _ = env.call_method(&rec, jni_str!("release"), jni_sig!("()V"), &[]);
                    return Err(JErr(format!("read() kept failing (last {n})")));
                }
                continue;
            }
            zero_reads = 0;
            chunk_arr.get_region(env, 0, &mut buf[..n as usize])?;
            let frames = n as usize / 2;
            for i in 0..frames {
                left[i] = buf[2 * i] as f32 / 32768.0;
                right[i] = buf[2 * i + 1] as f32 / 32768.0;
            }
            chunks += 1;
            let outcomes = processor.push(&left[..frames], &right[..frames]);
            let health = CaptureHealth {
                source,
                mask,
                sample_rate: SAMPLE_RATE,
                chunks,
            };
            let mut failed: Option<String> = None;
            for o in outcomes {
                match o {
                    FrameOutcome::Scored { score, ema } => {
                        live.last = Some(score);
                        live.ema = Some(ema);
                        live.quiet = false;
                        live.scored_frames += 1;
                    }
                    FrameOutcome::Quiet => live.quiet = true,
                    FrameOutcome::CalibrationDone => {
                        if let Some(c) = processor.calibration() {
                            log::info!(
                                "TV path learned: quality {:.2} · coverage {:.2} · rank ratio {:.2}",
                                c.quality,
                                c.band_coverage,
                                c.rank_ratio
                            );
                        }
                    }
                    FrameOutcome::CalibrationFailed(e) => failed = Some(e),
                    FrameOutcome::CalibrationProgress { .. } => {}
                }
            }
            if let Some(e) = failed {
                set_status(SpatialStatus::Failed(e));
            } else if processor.is_calibrating() {
                set_status(SpatialStatus::Calibrating {
                    seconds_left: processor.calib_frames_left() as f32 * HOP as f32
                        / SAMPLE_RATE as f32,
                    health,
                });
            } else if let Some(c) = processor.calibration() {
                set_status(SpatialStatus::Ready {
                    quality: c.quality,
                    band_coverage: c.band_coverage,
                    rank_ratio: c.rank_ratio,
                    two_source_warning: c.two_source_warning,
                    live: live.clone(),
                    health,
                });
            }
        }
        let _ = env.call_method(&rec, jni_str!("stop"), jni_sig!("()V"), &[]);
        let _ = env.call_method(&rec, jni_str!("release"), jni_sig!("()V"), &[]);
        log::info!("spatial capture stopped");
        Ok(())
    }
}

impl SpatialProcessor {
    /// True while a calibration pass is accumulating.
    pub fn is_calibrating(&self) -> bool {
        self.calibrating
    }
    /// Usable frames still needed by the running calibration.
    pub fn calib_frames_left(&self) -> usize {
        CALIB_FRAMES.saturating_sub(self.calib_used)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic LCG noise, same recipe as device_probe's tests.
    struct Lcg(u64);
    impl Lcg {
        fn next_f32(&mut self) -> f32 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((self.0 >> 40) as f32 / (1u32 << 23) as f32) - 1.0
        }
    }
    fn noise(seed: u64, n: usize, amp: f32) -> Vec<f32> {
        let mut g = Lcg(seed);
        (0..n).map(|_| amp * g.next_f32()).collect()
    }

    const SR: f32 = 48_000.0;

    /// A "source at a fixed position": channel L gets the signal delayed by
    /// `d_l` with gain `g_l`, channel R likewise. Pure delay + gain is the
    /// simplest physically-shaped relative transfer function.
    fn two_ch(sig: &[f32], d_l: usize, g_l: f32, d_r: usize, g_r: f32) -> (Vec<f32>, Vec<f32>) {
        let n = sig.len() - d_l.max(d_r);
        let l: Vec<f32> = (0..n).map(|i| g_l * sig[i + d_l]).collect();
        let r: Vec<f32> = (0..n).map(|i| g_r * sig[i + d_r]).collect();
        (l, r)
    }

    fn calibrate_on(proc_: &mut SpatialProcessor, l: &[f32], r: &[f32]) {
        proc_.begin_calibration();
        let mut done = false;
        // Feed repeatedly until the calibration completes.
        while !done {
            for o in proc_.push(l, r) {
                match o {
                    FrameOutcome::CalibrationDone => done = true,
                    FrameOutcome::CalibrationFailed(e) => panic!("calibration failed: {e}"),
                    _ => {}
                }
            }
        }
    }

    fn mean_score(proc_: &mut SpatialProcessor, l: &[f32], r: &[f32]) -> f32 {
        let mut sum = 0.0;
        let mut n = 0u32;
        for o in proc_.push(l, r) {
            if let FrameOutcome::Scored { score, .. } = o {
                sum += score;
                n += 1;
            }
        }
        assert!(n > 0, "no frames scored");
        sum / n as f32
    }

    #[test]
    fn tv_path_scores_high_on_tv_frames() {
        let mut p = SpatialProcessor::new(SR);
        let tv = noise(1, 120_000, 0.1);
        let (l, r) = two_ch(&tv, 0, 1.0, 5, 0.7);
        calibrate_on(&mut p, &l, &r);
        let c = p.calibration().unwrap();
        assert!(c.quality > 0.9, "quality {}", c.quality);
        assert!(c.band_coverage > 0.9, "coverage {}", c.band_coverage);
        assert!(!c.two_source_warning, "rank ratio {}", c.rank_ratio);

        let tv2 = noise(2, 120_000, 0.1); // different program, same path
        let (l2, r2) = two_ch(&tv2, 0, 1.0, 5, 0.7);
        let s = mean_score(&mut p, &l2, &r2);
        assert!(s > 0.9, "same-path score {s}");
    }

    #[test]
    fn other_position_scores_lower() {
        let mut p = SpatialProcessor::new(SR);
        let tv = noise(3, 120_000, 0.1);
        let (l, r) = two_ch(&tv, 0, 1.0, 5, 0.7);
        calibrate_on(&mut p, &l, &r);

        // A different source position: opposite delay ordering, different
        // gains — a mouth at arm's length instead of the TV across the room.
        let voice = noise(4, 120_000, 0.1);
        let (l2, r2) = two_ch(&voice, 8, 0.8, 0, 1.0);
        let s_other = mean_score(&mut p, &l2, &r2);

        let tv2 = noise(5, 120_000, 0.1);
        let (l3, r3) = two_ch(&tv2, 0, 1.0, 5, 0.7);
        let s_tv = mean_score(&mut p, &l3, &r3);

        assert!(
            s_tv > s_other + 0.2,
            "tv {s_tv} vs other {s_other} — paths not separated"
        );
        assert!(s_other < 0.75, "other-path score too high: {s_other}");
    }

    #[test]
    fn quiet_frames_produce_no_score() {
        let mut p = SpatialProcessor::new(SR);
        let tv = noise(6, 120_000, 0.1);
        let (l, r) = two_ch(&tv, 0, 1.0, 5, 0.7);
        calibrate_on(&mut p, &l, &r);
        let silence = vec![0.0f32; 40_000];
        let outcomes = p.push(&silence, &silence);
        // The STFT accumulator legitimately carries up to FFT_N−1 samples of
        // the loud calibration feed into the first hops — those may score.
        // Everything after the carry-over must be Quiet.
        let carry_hops = FFT_N / HOP; // 2
        assert!(outcomes.len() > carry_hops + 3);
        for o in &outcomes[carry_hops..] {
            assert_eq!(*o, FrameOutcome::Quiet);
        }
    }

    #[test]
    fn two_sources_during_calibration_are_flagged() {
        let mut p = SpatialProcessor::new(SR);
        let a = noise(7, 200_000, 0.1);
        let b = noise(8, 200_000, 0.1);
        let (la, ra) = two_ch(&a, 0, 1.0, 5, 0.7);
        let (lb, rb) = two_ch(&b, 8, 0.8, 0, 1.0);
        let n = la.len().min(lb.len());
        let l: Vec<f32> = (0..n).map(|i| la[i] + lb[i]).collect();
        let r: Vec<f32> = (0..n).map(|i| ra[i] + rb[i]).collect();
        calibrate_on(&mut p, &l, &r);
        let c = p.calibration().unwrap();
        assert!(
            c.two_source_warning,
            "two equal sources not flagged (rank ratio {})",
            c.rank_ratio
        );
    }

    #[test]
    fn single_source_rank_ratio_is_low() {
        let mut p = SpatialProcessor::new(SR);
        let tv = noise(9, 120_000, 0.1);
        let (l, r) = two_ch(&tv, 0, 1.0, 5, 0.7);
        calibrate_on(&mut p, &l, &r);
        let c = p.calibration().unwrap();
        assert!(c.rank_ratio < 0.1, "rank ratio {}", c.rank_ratio);
    }

    #[test]
    fn incoherent_channels_give_low_quality() {
        let mut p = SpatialProcessor::new(SR);
        // No shared source at all: nothing for a path to be learned from.
        let l = noise(10, 120_000, 0.1);
        let r = noise(11, 120_000, 0.1);
        calibrate_on(&mut p, &l, &r);
        let c = p.calibration().unwrap();
        assert!(c.quality < 0.3, "quality {} on incoherent input", c.quality);
        assert!(
            c.band_coverage < 0.3,
            "coverage {} on incoherent input",
            c.band_coverage
        );
    }

    #[test]
    fn calibration_fails_loudly_on_silence() {
        let mut p = SpatialProcessor::new(SR);
        p.begin_calibration();
        let silence = vec![0.0f32; FFT_N * CALIB_FRAMES * CALIB_MAX_WALL_FACTOR];
        let outcomes = p.push(&silence, &silence);
        assert!(
            outcomes
                .iter()
                .any(|o| matches!(o, FrameOutcome::CalibrationFailed(_))),
            "silent calibration did not fail"
        );
        assert!(p.calibration().is_none());
    }

    #[test]
    fn relearn_keeps_old_calibration_until_done() {
        let mut p = SpatialProcessor::new(SR);
        let tv = noise(12, 120_000, 0.1);
        let (l, r) = two_ch(&tv, 0, 1.0, 5, 0.7);
        calibrate_on(&mut p, &l, &r);
        assert!(p.calibration().is_some());
        p.begin_calibration();
        // Old calibration must survive an in-progress relearn.
        assert!(p.calibration().is_some());
        assert!(p.is_calibrating());
    }
}
