# Fach Lab — the voice-part study harness

`voxlab` runs a dataset of voice recordings through the **same
`FrameAnalyzer` the phone runs** (`src/frame.rs`) and writes the per-frame,
per-note and per-file measurements as CSV. Nothing here is a second DSP: a
number in these tables is the number the app would have shown. The study plan
(hypotheses, dataset spec, statistics) lives in the "Fach Lab" brief; this
page is the mechanics.

Desktop only (it links the audio decoders). Build with the rest of the crate:

```bash
cargo build --release --bin voxlab
```

---

## 1. Dataset layout

```
dataset/
  metadata.csv
  <part>/<singer_id>/<file>.wav|flac|mp3|m4a|aac|ogg
```

`metadata.csv` columns (any order; extra columns are carried through):

```
file, singer_id, part, weight, sex, material, vowel, nominal_pitch, key, source, notes
```

`file` is the path relative to the dataset root (forward slashes). `material`
is one of `vowel / glide / scale / repertoire / speech`; `source` one of
`studio / commercial / home / phone`. Files with no metadata row are still
analyzed — their metadata columns come out empty.

Any container/codec symphonia decodes: WAV (PCM/float), FLAC, MP3, AAC in
MP4/M4A, Vorbis/Ogg. Stereo is downmixed to mono; every file is resampled to
the engine rate (48 kHz by default, `--sr` to change) before analysis, so the
tables are comparable across sources.

---

## 2. Commands

```bash
# Whole dataset → three CSVs (default out dir: <dataset>/results)
voxlab analyze dataset/ [--out DIR] [--sr 48000] [--vtl-f0-max 300]

# One file, human-readable summary
voxlab file path/to/recording.wav

# Synthetic fixtures with known answers (bass & tenor vowels, a glide)
voxlab synth fixtures/
```

`--vtl-f0-max` sets the f0 ceiling for the *relaxed* VTL pool (default 300 Hz;
the identity-grade pool always uses the app's own 200 Hz gate).

---

## 3. Output tables

**`frames.csv`** — one row per 2048-sample frame (42.7 ms at 48 kHz):
`t_s, voiced, f0_hz, note, rms_db, snr_db, noisy, hnr_db, h1h2_db, cpp_db,
jitter_pct, shimmer_db, centroid_hz, f1..f3, formants_f0, grade, identity_ok,
vtl_cm, a2a1_db, tilt_db_oct, dominant_h, fhe_m, fhe_f, band_centroid_m,
h1..h16`.

**`notes.csv`** — one row per sustained note (≥ 1 s within ±1 semitone of the
run's median): medians of the frame measures, plus the note name.

**`files.csv`** — one row per file: duration, native rate, QC (peak dBFS,
clipped samples, P5 floor), voiced and identity fractions, tessitura
percentiles (P10/25/50/75/90, P2/P98 range, P50 note), FHE in the male
(2.0–3.6 kHz) and female (2.3–4.5 kHz) bands, singer's-formant cluster (peak,
prominence, −3 dB width — measured on the LTAS smoothed over one harmonic
spacing), identity-grade and relaxed VTL, F3, vibrato rate/extent,
dominant-harmonic shares (H1/H2/H3), turnover events (`401.1↑H1` = crossing
at 401 Hz on a rising pitch, H1 taking over), register events, the
third-octave LTAS (100 Hz – 8 kHz, mean-normalized dB), and the metadata
columns.

---

## 4. Phone captures

Every capture the app records is also written out as raw audio — the exact
frames the analyzer consumed — as **32-bit float mono WAV** at the input rate,
named `capture-YYYYMMDD-HHMMSS.wav`. The result card shows the file name,
length, peak and clipping, and an `AUDIO EXPORT: ON/OFF` chip. Saving the
session writes a sidecar `capture-YYYYMMDD-HHMMSS.json` beside the WAV with
the session's own measurements (f0, match %, HNR, H1–H2, CPP, jitter,
shimmer, centroid, harmonic profile, formants, coverage), and records the
`capture_file` name in `archive.json` — so the phone's numbers and the
harness's numbers join on the file name.

| Platform | Directory |
|---|---|
| Android | `/sdcard/Android/data/<package>/files/captures/` (app-specific external storage — no permission needed, readable with a plain `adb pull`) |
| Desktop | `$XDG_DATA_HOME/VoxLabs/captures/` (`~/.local/share/VoxLabs/captures/`) |
| Web | no export (no filesystem) |

Pull and analyze (dev package id shown, from the Cargo.toml patch step in
`docs/STATUS.md` §5; an unpatched build is `org.voxlabs.core`):

```bash
adb pull /sdcard/Android/data/org.voxlabs.core.dev/files/captures ./captures
voxlab analyze ./captures --out ./captures/results
```

Add the captures to the dataset by moving the files under
`dataset/<part>/<singer_id>/` and listing them in `metadata.csv` with
`source=phone`. (`archive.json` itself lives in app-private internal storage;
the dev APK is release-signed, so `run-as` cannot read it — the sidecars are
the export.)

A capture that is discarded in the app keeps its WAV (no sidecar) —
discarding drops the session, not the evidence. Delete files from the
directory if needed; the app never does.

---

## 5. Analyzing files in the app

The app itself analyzes audio files, through the same pipeline as a live
capture — the result card, the voiceprint match, the session archive, and
the sidecar export are all the live ones. Two ways in:

**Share sheet.** In Drive, Files, a voice recorder, or anything else that
can share an audio file, choose *Share → VoxLabs (dev)* (or *Open with*).
The app starts, copies the file into its import folder, and analyzes it
immediately: the Capture screen shows *Analyzing <file> · 43 %*, the
tract card reads FILE instead of LIVE, and the result card appears when
the file ends. (A share opens a fresh instance of the app; the previous
one, if it was open, is retired — the same relaunch path an activity
restart takes.)

**Import folder.** Put files in

| Platform | Import folder |
|---|---|
| Android | `/sdcard/Android/data/org.voxlabs.core.dev/files/import/` (`adb push song.wav /sdcard/Android/data/org.voxlabs.core.dev/files/import/`) |
| Desktop | `~/.local/share/VoxLabs/import/` |

and they appear in the **Files** card at the top of the Sessions screen,
newest first, each with an ANALYZE chip. The record button cancels a
running analysis. Formats: WAV, FLAC, MP3, M4A/AAC, OGG; stereo is
downmixed; any sample rate is resampled to the engine's.

A session saved from a file records `capture_file = import/<name>` (a
live capture records `capture-….wav`), and its sidecar JSON is written
beside the file in the import folder. While a file runs, the microphone
is ignored by the capture accumulators; the spectrogram and scope still
show the room.

---

## 6. Known answers (M0 acceptance)

`voxlab synth` writes three fixtures; `voxlab analyze` on them must report:

| Fixture | Known | Measured (2026-09-04) |
|---|---|---|
| `bass/synth_b/vowel_a_110.wav` — 110 Hz, cluster resonance 2400 Hz | FHE ≈ 2400, note A2 | FHE(m) 2412, cluster peak 2414, A2 |
| `tenor/synth_t/vowel_a_180.wav` — 180 Hz, cluster 2800 Hz | FHE ≈ 2800, note F♯3 | FHE(m) 2849, cluster peak 2859, F♯3 |
| `tenor/synth_t/glide_a.wav` — 200→600 Hz glide, F1 = 600 Hz | one ascending turnover at 2·F1/3 = 400 Hz, H1 taking over | 401 Hz ↑ H1 |

These are also unit tests (`cargo test --bin voxlab`).
