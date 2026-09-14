//! The rehearsal harmony as a tap consumer (Plan v3 §5.2: a display
//! computation, not a wire): joins the `multi_f0` voices with the `satb`
//! labels into Coral's `VoiceIn`, converts the detection spectrum to dB
//! with Coral's floor estimate (the median of every eighth bin), and runs
//! the card analyser. `now_ms` is the caller's clock — the hop's end
//! sample in ms for the harness, wall time on the phone.

use crate::choir::cards::{HarmonyAnalyzer, HarmonyResult};
use crate::choir::harmony::{HarmonyConfig, VoiceIn};
use crate::config::consts::DB_PER_DECADE_AMPLITUDE;
use crate::config::{ChoirHarmonyConfig, ChoirStftConfig};
use crate::pipeline::types::{NoteSet, SectionLabels, Spectrum};

pub struct ChoirHarmony {
    pub analyzer: HarmonyAnalyzer,
    hc: ChoirHarmonyConfig,
    eps: f32,
    db: Vec<f32>,
    sample: Vec<f32>,
    input: Vec<VoiceIn>,
}

impl ChoirHarmony {
    pub fn new(cfg: HarmonyConfig, hc: ChoirHarmonyConfig, stft: ChoirStftConfig) -> Self {
        Self {
            analyzer: HarmonyAnalyzer::new(cfg, hc),
            hc,
            eps: stft.magnitude_epsilon,
            db: Vec::new(),
            sample: Vec::new(),
            input: Vec::with_capacity(16),
        }
    }

    /// Coral's `dsp-core.ts::analyzeHarmony`, second half.
    pub fn feed(
        &mut self,
        spectrum: &Spectrum,
        notes: &NoteSet,
        labels: &SectionLabels,
        now_ms: f32,
    ) -> HarmonyResult {
        self.input.clear();
        for lab in &labels.labels {
            if let Some(v) = notes.voices.iter().find(|v| v.midi == lab.midi) {
                self.input.push(VoiceIn {
                    midi: v.midi,
                    f0_hz: v.f0_hz,
                    salience: v.salience,
                    section: lab.guess,
                    label: lab.label,
                    section_conf: lab.confidence,
                });
            }
        }
        let n = spectrum.fft_size / 2;
        if self.db.len() != n {
            self.db = vec![0.0; n];
        }
        self.sample.clear();
        for b in 0..n {
            self.db[b] = DB_PER_DECADE_AMPLITUDE * (spectrum.magnitude[b] + self.eps).log10();
            if b % self.hc.floor_sample_stride == 0 {
                self.sample.push(self.db[b]);
            }
        }
        self.sample.sort_by(f32::total_cmp);
        let floor_db = self
            .sample
            .get(self.sample.len() / 2)
            .copied()
            .unwrap_or(self.hc.floor_fallback_db);
        let bin_hz = spectrum.bin_hz;
        self.analyzer
            .analyze(&self.input, Some(&self.db), bin_hz, floor_db, now_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::choir::labeler::{Label, Section};
    use crate::choir::synth::{ChordSpec, VoiceSpec, synthesize_choir_chord};
    use crate::config::PipelineParams;
    use crate::pipeline::PipelineDefinition;
    use crate::pipeline::offline::run_offline;
    use crate::pipeline::types::Wire;

    struct Frame {
        notes: NoteSet,
        labels: SectionLabels,
        harmony: HarmonyResult,
        now_ms: f32,
    }

    /// The choir mode over `samples`, one entry per detection frame.
    fn run_choir(samples: &[f32], sr: f32) -> Vec<Frame> {
        let def = PipelineDefinition::by_name_or_path("choir").unwrap();
        let p = PipelineParams::DEFAULT;
        let mut harmony =
            ChoirHarmony::new(HarmonyConfig::default(), p.choir_harmony, p.choir_stft);
        let hop = def.format.hop;
        let frame = def.format.frame_samples;
        let mut out = Vec::new();
        run_offline(&def, sr, samples, |h| {
            let (Some(Wire::Spectrum(s)), Some(Wire::NoteSet(n)), Some(Wire::SectionLabels(l))) =
                (h.wires.get(1), h.wires.get(3), h.wires.get(4))
            else {
                panic!("choir wires");
            };
            let now_ms = (h.hop as usize * hop + frame) as f32 / sr * 1000.0;
            let r = harmony.feed(s, n, l, now_ms);
            out.push(Frame {
                notes: n.clone(),
                labels: l.clone(),
                harmony: r,
                now_ms,
            });
        })
        .unwrap();
        out
    }

    fn chord(sr: f32, secs: f64, seed: u32, voices: &[(Section, i32)]) -> Vec<f32> {
        let mut spec = ChordSpec::new(
            f64::from(sr),
            secs,
            voices.iter().map(|&(s, m)| VoiceSpec::new(s, m)).collect(),
        );
        spec.seed = seed;
        synthesize_choir_chord(&spec).unwrap().samples
    }

    fn last_active(samples: &[f32]) -> Vec<i32> {
        run_choir(samples, 44_100.0)
            .last()
            .unwrap()
            .notes
            .active_midi
            .clone()
    }

    /// Coral's `choir-detection.test.ts`: the calibrated multi-F0 contract.
    #[test]
    fn satb_detection_reproduces_corals_calibrated_battery() {
        let sr = 44_100.0;
        assert!(last_active(&vec![0.0; (sr * 0.4) as usize]).is_empty());
        let single = last_active(&chord(sr, 0.4, 7, &[(Section::A, 64)]));
        assert_eq!(single, vec![64], "{single:?}");
        let triad = last_active(&chord(
            sr,
            0.4,
            7,
            &[(Section::B, 60), (Section::T, 64), (Section::A, 67)],
        ));
        assert_eq!(triad, vec![60, 64, 67]);
        let wide = last_active(&chord(
            sr,
            0.4,
            7,
            &[
                (Section::B, 48),
                (Section::T, 55),
                (Section::A, 64),
                (Section::S, 72),
            ],
        ));
        for m in [48, 55, 64] {
            assert!(wide.contains(&m), "{wide:?}");
        }
        type Case = (&'static [i32], &'static [(Section, i32)]);
        let battery: [Case; 3] = [
            (
                &[48, 55, 64, 72],
                &[
                    (Section::B, 48),
                    (Section::T, 55),
                    (Section::A, 64),
                    (Section::S, 72),
                ],
            ),
            (
                &[60, 64, 67],
                &[(Section::B, 60), (Section::T, 64), (Section::A, 67)],
            ),
            (
                &[55, 59, 62, 67],
                &[
                    (Section::B, 55),
                    (Section::T, 59),
                    (Section::A, 62),
                    (Section::S, 67),
                ],
            ),
        ];
        let (mut tp, mut detected, mut gt_total) = (0usize, 0usize, 0usize);
        for (gt, voices) in battery {
            let active = last_active(&chord(sr, 0.4, 7, voices));
            tp += gt.iter().filter(|m| active.contains(m)).count();
            detected += active.len();
            gt_total += gt.len();
        }
        let recall = tp as f32 / gt_total as f32;
        let precision = tp as f32 / detected as f32;
        assert!(recall >= 0.65, "recall {recall}");
        assert!(precision >= 0.8, "precision {precision}");
        let octave = last_active(&chord(sr, 0.4, 7, &[(Section::B, 48), (Section::S, 60)]));
        assert!(octave.contains(&48), "{octave:?}");
    }

    /// Coral's `section-labeler.test.ts`, the end-to-end half: the P3
    /// label-accuracy floor on labelled SATB chords (no SPR).
    #[test]
    fn labels_hit_the_p3_accuracy_floor_end_to_end() {
        let sr = 44_100.0;
        let battery: [&[(Section, i32)]; 4] = [
            &[
                (Section::B, 48),
                (Section::T, 55),
                (Section::A, 64),
                (Section::S, 72),
            ],
            &[
                (Section::B, 55),
                (Section::T, 59),
                (Section::A, 62),
                (Section::S, 67),
            ],
            &[
                (Section::B, 50),
                (Section::T, 57),
                (Section::A, 62),
                (Section::S, 69),
            ],
            &[
                (Section::B, 52),
                (Section::T, 59),
                (Section::A, 67),
                (Section::S, 76),
            ],
        ];
        let (mut correct, mut wrong, mut total) = (0, 0, 0);
        for voices in battery {
            let frames = run_choir(&chord(sr, 0.5, 7, voices), sr);
            let last = frames.last().unwrap();
            for l in &last.labels.labels {
                let Some(&(truth, _)) = voices.iter().find(|(_, m)| *m == l.midi) else {
                    continue;
                };
                total += 1;
                match l.label {
                    Label::Section(s) if s == truth => correct += 1,
                    Label::Section(_) => wrong += 1,
                    Label::AmbiguousAT => {}
                }
            }
        }
        assert!(total > 8, "{total}");
        assert!(correct as f32 / total as f32 >= 0.55, "{correct}/{total}");
        assert!(wrong as f32 / total as f32 <= 0.15, "{wrong}/{total}");
    }

    /// Coral's `harmony.test.ts`, "through the real pipeline".
    #[test]
    fn a_synthesized_c_major_yields_four_voices_within_3_cents_and_a_major_chord() {
        let sr = 44_100.0;
        let mut spec = ChordSpec::new(
            f64::from(sr),
            1.5,
            [
                (Section::B, 48),
                (Section::T, 55),
                (Section::A, 64),
                (Section::S, 72),
            ]
            .iter()
            .map(|&(s, m)| {
                let mut v = VoiceSpec::new(s, m);
                v.singers = 1;
                v.detune_cents = 0.0;
                v.vibrato_cents = 0.0;
                v
            })
            .collect(),
        );
        spec.seed = 3;
        let samples = synthesize_choir_chord(&spec).unwrap().samples;
        let frames = run_choir(&samples, sr);
        let res = &frames.last().unwrap().harmony;
        assert_eq!(
            res.voices.iter().map(|x| x.info.midi).collect::<Vec<_>>(),
            [48, 55, 64, 72]
        );
        assert_eq!(
            res.voices.iter().map(|x| x.section).collect::<Vec<_>>(),
            [Section::B, Section::T, Section::A, Section::S]
        );
        for x in &res.voices {
            assert!(x.info.cents.abs() < 3.0, "{:?}", x.info);
        }
        assert_eq!(res.id.unwrap().chord.name, "Major");
        assert!(res.settled);
    }

    fn four_by_four(midis: [i32; 4], secs: f64, seed: u32) -> Vec<f32> {
        let mut spec = ChordSpec::new(
            48_000.0,
            secs,
            [Section::B, Section::T, Section::A, Section::S]
                .iter()
                .zip(midis)
                .map(|(&s, m)| {
                    let mut v = VoiceSpec::new(s, m);
                    v.singers = 4;
                    v.detune_cents = 6.0;
                    v.vibrato_cents = 15.0;
                    v
                })
                .collect(),
        );
        spec.seed = seed;
        synthesize_choir_chord(&spec).unwrap().samples
    }

    #[test]
    fn a_four_per_part_c_major_holds_one_chord_identity_for_six_seconds() {
        let frames = run_choir(&four_by_four([48, 55, 64, 72], 6.0, 11), 48_000.0);
        let mut keys = std::collections::BTreeSet::new();
        let mut max_held = 0.0f32;
        for f in &frames {
            if f.now_ms < 1000.0 {
                continue;
            }
            let id = f.harmony.id.unwrap();
            keys.insert(format!(
                "{}:{}:{}",
                id.root,
                id.chord.name,
                f.harmony.voices.len()
            ));
            max_held = max_held.max(f.harmony.chord_held_ms);
        }
        assert_eq!(keys.into_iter().collect::<Vec<_>>(), ["0:Major:4"]);
        assert!(max_held > 4500.0, "{max_held}");
    }

    #[test]
    fn follows_a_real_chord_change_from_c_major_to_f_major() {
        let mut samples = four_by_four([48, 55, 64, 72], 2.0, 5);
        samples.extend(four_by_four([53, 57, 60, 69], 3.5, 6));
        let frames = run_choir(&samples, 48_000.0);
        let first_f = frames
            .iter()
            .find(|f| {
                f.harmony
                    .id
                    .is_some_and(|id| id.root == 5 && id.chord.name == "Major")
                    && f.harmony.settled
            })
            .map(|f| f.now_ms)
            .expect("F major settled");
        assert!(first_f - 2000.0 < 1000.0, "{first_f}");
        let last = &frames.last().unwrap().harmony;
        assert_eq!(
            last.voices.iter().map(|x| x.info.midi).collect::<Vec<_>>(),
            [53, 57, 60, 69]
        );
    }
}
