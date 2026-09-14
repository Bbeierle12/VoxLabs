//! Coral's synthetic SATB chord generator (`choir-synth.ts`): the ground
//! truth its detection and harmony tests measure against. Deterministic
//! by seed (mulberry32 + Box–Muller, as in the TypeScript), singer-major
//! synthesis with per-singer detune, vibrato (FM) and tremolo (AM), a
//! per-section singer's formant, 1/k tilt, Nyquist-limited harmonics,
//! peak-normalized to 0.9.

use std::f64::consts::TAU;

use crate::choir::labeler::Section;

/// The TypeScript `mulberry32`, bit for bit (u32 arithmetic, `Math.imul`).
pub struct Mulberry32(pub u32);

impl Mulberry32 {
    pub fn next_f64(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6d2b_79f5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(1 | t);
        t = (t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t))) ^ t;
        ((t ^ (t >> 14)) as f64) / 4_294_967_296.0
    }

    /// Standard normal via Box–Muller, as the TypeScript does.
    pub fn gaussian(&mut self) -> f64 {
        let mut u = 0.0;
        let mut v = 0.0;
        while u == 0.0 {
            u = self.next_f64();
        }
        while v == 0.0 {
            v = self.next_f64();
        }
        (-2.0 * u.ln()).sqrt() * (TAU * v).cos()
    }
}

pub fn midi_to_freq(midi: f64) -> f64 {
    440.0 * 2f64.powf((midi - 69.0) / 12.0)
}

#[derive(Clone, Debug)]
pub struct VoiceSpec {
    pub midi: i32,
    pub section: Section,
    pub singers: usize,
    pub detune_cents: f64,
    pub vibrato_hz: f64,
    pub vibrato_cents: f64,
    pub gain: f64,
    pub formant_hz: Option<f64>,
    pub harmonic_db: Option<Vec<f64>>,
}

impl VoiceSpec {
    pub fn new(section: Section, midi: i32) -> Self {
        Self {
            midi,
            section,
            singers: 8,
            detune_cents: 25.0,
            vibrato_hz: 5.5,
            vibrato_cents: 22.0,
            gain: 1.0,
            formant_hz: None,
            harmonic_db: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChordSpec {
    pub sample_rate: f64,
    pub duration_sec: f64,
    pub voices: Vec<VoiceSpec>,
    pub harmonics: usize,
    pub rolloff_hz: f64,
    pub formant_hz: Option<f64>,
    pub formant_gain_db: f64,
    pub formant_bw_hz: f64,
    pub am_depth: f64,
    pub noise_floor: f64,
    pub seed: u32,
}

impl ChordSpec {
    pub fn new(sample_rate: f64, duration_sec: f64, voices: Vec<VoiceSpec>) -> Self {
        Self {
            sample_rate,
            duration_sec,
            voices,
            harmonics: 16,
            rolloff_hz: 4000.0,
            formant_hz: None,
            formant_gain_db: 6.0,
            formant_bw_hz: 1500.0,
            am_depth: 0.04,
            noise_floor: 0.0,
            seed: 1,
        }
    }
}

/// Singer's-formant centre by section, Hz (Sundberg 2001; alto interpolated).
pub fn section_formant_hz(s: Section) -> f64 {
    match s {
        Section::B => 2384.0,
        Section::T => 2705.0,
        Section::A => 2900.0,
        Section::S => 3092.0,
    }
}

pub struct ChordResult {
    pub samples: Vec<f32>,
    pub sample_rate: f64,
    /// Distinct sounding MIDI notes, ascending.
    pub ground_truth_midi: Vec<i32>,
}

pub fn synthesize_choir_chord(spec: &ChordSpec) -> Result<ChordResult, String> {
    if !(spec.sample_rate > 0.0 && spec.duration_sec > 0.0) || spec.voices.is_empty() {
        return Err("synthesize_choir_chord: sample rate, duration and voices required".into());
    }
    let len = ((spec.duration_sec * spec.sample_rate).round() as usize).max(1);
    let nyquist = spec.sample_rate / 2.0;
    let formant_lin = 10f64.powf(spec.formant_gain_db / 20.0) - 1.0;
    let mut rng = Mulberry32(spec.seed);
    let mut samples = vec![0.0f32; len];
    let inv_sr = 1.0 / spec.sample_rate;
    for voice in &spec.voices {
        let f0base = midi_to_freq(voice.midi as f64);
        let vib_depth_oct = voice.vibrato_cents / 1200.0;
        let per_singer_gain = voice.gain / (voice.singers as f64).sqrt();
        let voice_formant = voice
            .formant_hz
            .or(spec.formant_hz)
            .unwrap_or_else(|| section_formant_hz(voice.section));
        for _ in 0..voice.singers {
            let f0 = f0base * 2f64.powf(rng.gaussian() * voice.detune_cents / 1200.0);
            let vib_rate = voice.vibrato_hz * (1.0 + 0.1 * rng.gaussian());
            let vib_phase = rng.next_f64() * TAU;
            let am_rate = 3.0 + rng.next_f64() * 3.0;
            let am_phase = rng.next_f64() * TAU;
            let mut phi = rng.next_f64() * TAU;
            let mut amps = vec![0.0f64; spec.harmonics + 1];
            let mut k_max = 0;
            for k in 1..=spec.harmonics {
                let fk = k as f64 * f0;
                if fk >= nyquist {
                    break;
                }
                if let Some(explicit) = &voice.harmonic_db {
                    if k > explicit.len() {
                        break;
                    }
                    amps[k] = 10f64.powf(explicit[k - 1] / 20.0);
                    k_max = k;
                    continue;
                }
                let mut a = 1.0 / k as f64;
                if voice_formant > 0.0 && formant_lin > 0.0 {
                    let r = (fk - voice_formant) / spec.formant_bw_hz;
                    a *= 1.0 + formant_lin * (-r * r).exp();
                }
                a *= (-fk / (2.0 * spec.rolloff_hz)).exp();
                amps[k] = a;
                k_max = k;
            }
            let w_vib = TAU * vib_rate;
            let w_am = TAU * am_rate;
            for (n, s) in samples.iter_mut().enumerate() {
                let t = n as f64 * inv_sr;
                let vib = 2f64.powf(vib_depth_oct * (w_vib * t + vib_phase).sin());
                phi += TAU * f0 * vib * inv_sr;
                let am = 1.0 + spec.am_depth * (w_am * t + am_phase).sin();
                let mut acc = 0.0;
                for (k, &a) in amps.iter().enumerate().take(k_max + 1).skip(1) {
                    acc += a * (k as f64 * phi).sin();
                }
                *s = (*s as f64 + per_singer_gain * am * acc) as f32;
            }
        }
    }
    let peak = samples.iter().fold(0.0f32, |p, s| p.max(s.abs()));
    if peak > 0.0 {
        let g = 0.9 / peak;
        for s in &mut samples {
            *s *= g;
        }
    }
    if spec.noise_floor > 0.0 {
        for s in &mut samples {
            *s = (*s as f64 + rng.gaussian() * spec.noise_floor) as f32;
        }
    }
    let mut gt: Vec<i32> = spec.voices.iter().map(|v| v.midi).collect();
    gt.sort_unstable();
    gt.dedup();
    Ok(ChordResult {
        samples,
        sample_rate: spec.sample_rate,
        ground_truth_midi: gt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The PRNG matches JavaScript's mulberry32 for seed 1 (first values
    /// computed by hand from the TypeScript: the sequence is deterministic
    /// and the first output for seed 1 is 0.6270739405881613).
    #[test]
    fn mulberry32_matches_the_typescript_sequence() {
        let mut r = Mulberry32(1);
        let a = r.next_f64();
        assert!((a - 0.627_073_940_588_161_3).abs() < 1e-12, "{a}");
    }

    #[test]
    fn a_chord_is_normalized_and_deterministic() {
        let spec = ChordSpec::new(
            44_100.0,
            0.2,
            vec![
                VoiceSpec::new(Section::B, 48),
                VoiceSpec::new(Section::S, 72),
            ],
        );
        let a = synthesize_choir_chord(&spec).unwrap();
        let b = synthesize_choir_chord(&spec).unwrap();
        assert_eq!(a.samples, b.samples);
        let peak = a.samples.iter().fold(0.0f32, |p, s| p.max(s.abs()));
        assert!((peak - 0.9).abs() < 1e-5);
        assert_eq!(a.ground_truth_midi, vec![48, 72]);
    }
}
