# VoxLabs — project status (14 September 2026)

Handoff note for whoever (or whatever) picks the repository up next. It says
what exists, what has been verified and how, what is only built and not yet
field-tested, and what comes next. `docs/fach-lab.md` covers the study
harness; `docs/android-build.md` the APK toolchain.

**Repository state.** `main` == `review-fixes/phase-f-g-h` == `b041c7a`.
72 commits. Working tree clean. `cargo test`: 141 library tests + 6 harness
tests pass; `cargo clippy --all-targets -- -D warnings` clean; `cargo check`
passes for `aarch64-linux-android` (lib) and `wasm32-unknown-unknown` (lib).
~17k lines of Rust across 27 modules.

---

## 1. What the app is

A real-time voice-acoustics instrument for singers. One Rust crate builds
three ways: desktop (eframe/egui on wgpu), Android (NativeActivity cdylib
via cargo-apk), web (wasm). Audio in through cpal (AAudio on Android),
analysis on a dedicated thread in 2048-sample frames, results to the UI via
triple buffers.

Per-frame pipeline (`src/frame.rs`, `FrameAnalyzer` — shared by the Android
loop, the file-import path, and the `voxlab` harness; the desktop GPU path
mirrors it with a wgpu YIN stage): YIN pitch with confidence gate → SNR
voicing gate against a learned ambient floor + calibrated-interferer (room
hum) gate → LPC formants on a decimated frame, tagged with the f0 they were
measured at → harmonic amplitudes at k·f0 → HNR, H1–H2, CPP, jitter, shimmer,
centroid, tilt, vibrato/steadiness.

On top: a Story two-mode vocal-tract inversion (`tract.rs`, `tract_data.rs`)
giving an area function, vowel coordinate (q1,q2) and vocal-tract length; a
classical voiceprint (median identity-grade formants, VTL, centroid, tilt,
16-harmonic profile) with enrollment, similarity scoring and a *coverage*
statistic; room calibration and interferer fingerprinting; a JNI device
capability probe; a two-microphone spatial stage (per-bin relative transfer
functions for the TV's path and the singer's path, live path-consistency
score); voice-part measurements (`fach.rs`); raw-capture export
(`capture_log.rs`); in-app file import (`import.rs`, `audio_file.rs`,
`share_intent.rs`); the `voxlab` study harness (`src/bin/voxlab.rs`).

UI screens: Overview, Capture (hero tract card, harmonics ladder, turnover
gauge, result card), Sessions (FILES import card + archive), Detail, Room
(levels, calibration, DEVICE probe card, TV PATH card).

---

## 2. Timeline of this branch (what each phase did)

| Phase | Commits | Outcome |
|---|---|---|
| F/G/H review fixes | early branch | No panics on the analysis path, loops report their death, cpal/wasm errors surfaced, android_main re-entry guard, per-frame cost telemetry, dependency audit posture (two quick-xml advisories accepted, documented in Cargo.toml) |
| Android hardening | — | 16 KB page alignment (`RUSTFLAGS="-C link-arg=-Wl,-z,max-page-size=16384"`), mic-unavailable banner, activity relaunch survival |
| DSP honesty | — | Formant estimates gated by the f0 they were measured at (`FormantGrade` Identity / DisplayOnly / Reject); turnover gauge on Bozeman's A2/A1 observable |
| Vocal tract | — | Story basis extracted, forward model + inversion + VTL, live model on the Capture hero card |
| Room | — | Noise floor + SNR gate, room calibrator (ambient RMS + steady interferer), Room tab |
| Liveness | 8bf18be…c440bd6 | Device probe (UNPROCESSED absent on the Pixel; VOICE_RECOGNITION + channel-index mask gives two independent mics), spatial RTF calibration, two-path contrast, recorder-contention fix |
| Fach Lab M0 | a471cb6…b041c7a | Shared FrameAnalyzer, `fach.rs` measurements, `voxlab` harness, raw WAV export + session sidecar, in-app file import + share sheet |

---

## 3. Verified vs. built-but-unverified

**Verified in the field (Brandon's Pixel):** capture, tract card, Room
calibration, device probe (two independent channels, corr 0.35–0.89 across
runs, lag +0.17–0.29 ms), single-path TV calibration (quality 36 %, band
coverage 1 %, consistency 0.91 — i.e. non-discriminative in a reverberant
room, which motivated the contrast build), recorder-contention fix, LIVE
indicator.

**Verified by tests only:** everything in `fach.rs` and `voxlab`
(synthetic fixtures with known answers — see `docs/fach-lab.md` §6),
`capture_log`, `audio_file`, `import` (a synthetic WAV streams every frame
voiced at 150 Hz; junk input errors instead of hanging), `share_intent`
name/MIME sanitizing.

**Built, not yet exercised on a phone:**
- Two-path contrast (`LEARN YOUR PATH`): the separation % it reports is the
  number to look at — if separation stays low the two RTFs are too similar
  for a 14 cm mic pair below ~1.2 kHz and the approach needs the reference
  (played-file) route instead.
- Raw-capture export + sidecar JSON (e94fa18): check that
  `/sdcard/Android/data/com.voiceharmonic.engine.dev/files/captures/` fills
  and `adb pull` works.
- File import via share sheet (2cb860b): share an MP3 from Drive → app
  restarts → "Analyzing … %" → result card. The import-folder path and the
  worker are unit-tested; the JNI intent read is not.

**Known limitations, deliberate:**
- A share opens a fresh activity instance and retires the previous one (the
  relaunch path); NativeActivity gives no `onNewIntent`.
- While a file is being analyzed the capture accumulators ignore the mic
  but the spectrogram/scope still show the room.
- The current voiceprint fingerprints the vowel and effort more than the
  singer (harmonic profile is not pitch-invariant; population SDs
  uncalibrated; single frozen reference). Fingerprint v2 is planned from
  the study, not by argument — see `fach-lab.md` and the Fach Lab brief.
- Voice-part classification has no norms yet; the app cannot honestly say
  bass from tenor until M2 produces class distributions.
- winit 0.30 stubs `MainEvent::Destroy`; real teardown on Android waits
  for the eframe/winit upgrade.
- `UNPROCESSED` capture is a firmware fact on this device; closed.

---

## 4. Next steps (the Fach Lab plan, M1–M5)

- **M1 — ingest and look** (starts when the dataset lands in Google Drive;
  layout in `fach-lab.md` §1): QC every file, `voxlab analyze`, dashboard of
  LTAS per part, FHE and tessitura distributions, harmonic-tuning maps,
  passaggio scans.
- **M2 — the numbers:** Cohen's d / AUC per feature per adjacent pair;
  leave-one-singer-out classification (accuracy reported against number of
  singers); verdicts on the four hypotheses vs. published expectations
  (FHE bass 2384 · baritone 2454 · tenor 2705 · soprano 3092 Hz).
- **M3 — innovation pass:** features as slopes against f0 per singer;
  vowel normalization through the Story (q1,q2) coordinate; intra- vs
  inter-singer distances to pick fingerprint-v2 features.
- **M4 — into the app:** voice-part evidence card (distributions with the
  overlap drawn, never a bare label), fingerprint v2 with personal norms and
  coverage-aware verdicts, tessitura task, turnover logging.
- **M5 — the channel:** replay a dataset subset through the TV, recapture
  on the Pixel, tabulate per-feature shifts → phone-domain norms; first
  reference-based TV-cancellation test; four-state live/TV surface.

Smaller items: field-verify the three "built, not exercised" features
above; feed file frames to the spectrogram during import; `voxlab archive`
(sidecar/archive → CSV join) if the study wants it.

---

## 5. Conventions and gotchas

- **Dev APK:** patch `package` → `com.voiceharmonic.engine.dev` and `label`
  → `VoxLabs (dev)` in Cargo.toml, `cargo apk build --release --lib` with the
  16 KB `RUSTFLAGS`, then `git checkout Cargo.toml`. Verify page alignment
  with `llvm-readelf -l` (every LOAD at 0x4000). Build outputs were named
  `voxlabs-dev-<feature>-<sha>.apk`.
- **Manifest:** intent filters are declared in Cargo.toml; once any filter
  is declared the MAIN/LAUNCHER one must be spelled out too (cargo-apk only
  adds it when none exist).
- **jni 0.22:** typed signatures (`jni_str!`, `jni_sig!`), `Env`,
  `attach_current_thread(|env| …)`, `JavaVM::from_raw` returns the VM
  directly, `JString::from_raw(env, obj.into_raw()).try_to_string(env)`.
  Patterns in `device_probe.rs` / `spatial.rs` / `share_intent.rs`.
- **Android recorders:** the engine (AAudio), the spatial thread and the
  probe each hold an AudioRecord; the HAL refuses a third. The probe skips
  its 2-ch test while spatial runs; spatial waits for the probe.
- **Desktop dead-code:** modules whose types are only constructed on
  Android carry `#[cfg_attr(not(target_os = "android"), allow(dead_code))]`.
- **Doctests:** an indented equation in a doc comment must be fenced as
  ```` ```text ````.
- **Formatting edits:** run `cargo fmt` before scripted text replacements —
  it reflows code and a stale anchor silently no-ops.
- **Commit trailers used on this branch:**
  `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>` and a
  `Claude-Session:` line.
- **Data locations:** archive `internal_data_path()/archive.json`; captures
  and import folder under `external_data_path()/{captures,import}`;
  desktop `~/.local/share/VoxLabs/{archive.json,captures,import}`.

## 6. Where the thinking lives

Three published briefs (HTML sources are in the delivered archive, not in
the repo): *Live or Loudspeaker* (replay/liveness research, the
simultaneous-source case, four-state design), *Fach, Measured* (what
separates voice parts acoustically, the classification ceiling), *Fach Lab*
(the study plan this status tracks). The fingerprint review's conclusions
are summarized in §3 above.
