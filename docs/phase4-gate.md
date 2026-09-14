# Phase 4 gate — Coral import; the choir branch in Rust

Plan v3 §5 Phase 4 (`docs/PLAN_v3_completion.md` §5). Coral's DSP is
ported to `vox-core::choir` with Coral's own tests and librosa oracle
fixtures as the contracts, wrapped as four stages, and run headless by
the harness as `pipelines/choir.toml`. The shell work the plan put in
this phase (Tauri/React desktop, Coral's views as tap renderers, the
TypeScript worker's deletion, Coral's capture → cpal) moves to Phase 5a
under the Pixel-only directive; see "Deviations" and `DECISIONS.md`.

## Import

`apps/coral/` is Coral at commit `382290e` (v0.2.0), a working-tree
copy with the revision recorded in `apps/coral/IMPORT.md` — the
session's clone was shallow and `git subtree add` refuses shallow roots.
`node_modules`, `dist`, `builds` and `src-tauri/target` are not imported.

## What was ported

| Coral (`apps/coral/src/audio/`) | Rust | Contract (Coral's test) |
|---|---|---|
| `window.ts`, `streaming-stft.ts` | `choir::stft::CoralStft` (periodic Hann, `2/Σw`, N/2 bins + the Nyquist bin) | 6 librosa 0.11.0 oracle fixtures within rtol 1e-4 / atol 1e-7, bin 0 excluded (`oracle-fixtures.test.ts`); 0 dB calibration canary |
| `qifft.ts` | `choir::qifft` | `qifft.test.ts`: parabola properties, ±15 c across the vocal range, silence, sub-octave check |
| `note-detector.ts` (legacy and harmonic paths, family test, merge) | `choir::note_detector::NoteDetector` | `note-detector.test.ts`: bands, −50 dB threshold, C7–E7–G7, 0.92 decay (55–70 frames), reset, harmonic stack → one fundamental, flat spectrum rejected, harmonic bins, confidences, invalid options |
| `section-labeler.ts` | `choir::labeler` | `section-labeler.test.ts`: order labels, `A/T?` abstention, SPR tie-break, dedupe, confidence scaling, and the end-to-end P3 floor (≥ 55 % correct, ≤ 15 % wrong) |
| `harmony.ts` (pure half) | `choir::harmony` | chord id, note info, ratio text, Hann σ |
| `harmony.ts` (`HarmonyAnalyzer`) | `choir::cards::HarmonyAnalyzer` | `harmony.test.ts`: pure C major 0 error / ensemble −7.7 c on the third, bass anchoring, Dom 7 with the 7:6 pair, no settle before 300 ms and the −30 c drift after 20 flat updates, sections off, cluster |
| `dsp-core.ts` (`analyzeHarmony`) | `choir::pipeline::ChoirHarmony` (tap consumer) | `harmony.test.ts` "through the real pipeline": four voices within 3 c and a Major chord; a 4-per-part C major holds one identity for 6 s; C → F change identified within 1 s |
| `choir-synth.ts` | `choir::synth` | mulberry32 sequence bit-exact with the TypeScript; normalization; determinism |
| `dsp-core.ts` (detection path) | stages below | `choir-detection.test.ts`: silence, one note exactly, close triad exactly, wide chord's lower voices, battery recall ≥ 0.65 / precision ≥ 0.8, octave doubling's lower note |

Stages and wires:

| Stage | Backend | In → Out |
|---|---|---|
| `stft` | `coral_hann` | AudioFrame → Spectrum (Coral's scaling, Coral's epsilon in dB) |
| `pitch` | `qifft` | Spectrum → F0Track |
| `multi_f0` | `harmonic_cancellation` | Spectrum → `NoteSet` (display-range `active_midi`; rehearsal `voices` ≤ 1150 Hz with parabola-refined f0 and salience) |
| `satb` | `labeler` | NoteSet → `SectionLabels` |

`NoteSet` and `SectionLabels` are the §2 table's types, now with
payloads; the comparer covers them (`[tolerance]` `f0_hz`, `confidence`;
labels by equality). `pipelines/choir.toml` frames at Coral's detection
size (8192 / 2048); Coral's 2048 / 512 display STFT is not a second
stage — the waterfall can read the 8192 tap, and the mode file records
Coral's display framing in `[choir_stft]` for the renderer.

Config: `[choir_stft]`, `[choir_detector]`, `[qifft]`, `[choir_labeler]`,
`[choir_harmony]` — every literal Coral's worker used, including the
`NoteDetector` locals (`thresholdDb` −50, `cancelHarmonicCount` 16,
`cancelHeadroom` 1, `fundamentalFloor` 1) and the harmony card constants.
The tuning profiles, JI table, chord templates and SATB ranges are
Coral's published tables and stay as consts (`choir::harmony`,
`[choir_labeler]` ranges are fields).

A note Coral's own `detector.ts` carries: the `salienceThreshold` literal
is 3.0 while its comment argues for 3.5 and `PROGRESS.md` records 3.2 as
the tuned value. The port keeps the literal (3.0) — the shipped
behaviour — and the drift is recorded here, not resolved.

## Gate

| Check | Result |
|---|---|
| Coral 0.2.0 fixtures reproduce within tolerance from the Rust branch | pass: 6 librosa fixtures inside the cross-implementation tier; every Coral DSP test above passes on the port |
| `choir.toml` runs headless in `vox-harness` | pass: `voxlab run choir <wav>` records Spectrum, F0Track, NoteSet, SectionLabels taps; `voxlab validate` round-trips them within the bands (`vox-validation/tests/round_trip.rs::choir_mode_round_trips_note_sets_and_labels`) |
| On device | Engineering Console → Run self-test: `contract.choir_stft_matches_librosa` (the 48 kHz fixture), `contract.choir_detects_a_close_triad`, `contract.choir_harmony_hears_c_major` — to be read off the Pixel |
| The TypeScript worker is deleted | **not done in this phase** (below) |

`cargo test --workspace`: 226 library, 6 harness, 4 validation tests.
Clippy clean; Android and wasm library type-checks pass.

## Deviations, recorded

- **Shell and renderers (plan §5.3) → Phase 5a.** The plan stood up
  Tauri 2 + React on desktop here. The Pixel is the only product (D18
  amendment); a desktop-only shell is not on the schedule. Coral's
  Spectrogram and Rehearsal views become tap renderers in whatever shell
  5a chooses for the phone.
- **The TypeScript worker stays** in `apps/coral/src/audio/` until that
  shell consumes the Rust taps. Deleting it now would leave the imported
  app unable to run at all, with nothing yet in its place on the phone.
  The plan's "not kept just in case" is honoured by this record: the
  deletion is 5a's exit condition, not optional.
- **Coral's capture** (browser `getUserMedia` → AudioWorklet in the
  WebView) → cpal is likewise the shell's work (completion-plan call 8
  named Phase 4; it is bound to the shell, so it moves with it).
- **JI harmony is a consumer**, not a stage, as §5.2 preferred; the
  fixtures did not need it to be a wire.
- **No `git subtree`** (above).

## Not done

- Coral's benchmark battery (`choir-bench.ts`, 32 tagged cases) is not
  ported; the six-case `choir-detection.test.ts` contract is.
- Coral's CQT front end (evaluated and not adopted in Coral) is not
  ported.
- `intonation.ts`'s vibrato-rate detector duplicates `vox-core`'s own
  contour vibrato; not ported.
