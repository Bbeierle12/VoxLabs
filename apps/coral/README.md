# Coral

Choral spectrogram & harmonic analyzer — now also the home of the **Choral Harmony Analyzer** rehearsal view (v0.2.0 merged the standalone PWA into this codebase; the analyzer repo is retired).

Two views share one audio pipeline. **Spectrogram** is Coral's original waterfall with note detection and section labels. **Rehearsal** shows four fixed SATB cards (pitch, cents, in-tune band, section scatter), the harmony card (chord ID, consonance index, drift since start, pair table with pure ratios and beat rates, register balance) and a snapshot log for rehearsal notes. Targets are anchored on the lowest sounding chord tone and come from a selectable profile (Pure / Ensemble / Equal — the Ensemble profile is what measured a cappella groups actually produce; see docs/).

Live mic capture is the primary input; decoded files (WAV, MP3, FLAC, OGG) are a secondary input that feed the same waterfall view at real-time pace. The DSP and render paths are shared across both modes, so dB readings, note detection, and section labels are calibrated identically whether the source is live audio or a file.

## Getting started

```bash
npm install
npm run dev
```

Open the URL Vite prints. The left rail toggles between **Mic** and **File** input. In Mic mode, click Start mic and grant permission. In File mode, drop an audio file or click the drop zone to pick one. File playback is **analysis-only** — nothing is routed to the speakers; you watch the file analyze at real-time pace.

AudioWorklet requires a secure context: `localhost` works, raw LAN IPs don't.

## Scripts

| script | what it does |
| --- | --- |
| `npm run dev` | Vite dev server with HMR |
| `npm run build` | Type-check (`tsc -b`, app + config) and build to `dist/` |
| `npm run preview` | Serve the built bundle locally |
| `npm test` | Run Vitest once |
| `npm run test:watch` | Vitest in watch mode |
| `npm run tauri dev` | Run as a native desktop app (dev) |
| `npm run tauri build` | Build the desktop app |

## Desktop & Android

Coral also packages as a native **desktop** app and an **Android** app (for
Google Play) via Tauri v2, from this same frontend — see
[`docs/PACKAGING.md`](docs/PACKAGING.md). The web build is unaffected. Live mic
capture works in both: the Linux WebKitGTK permission quirk is handled in
`src-tauri/src/lib.rs`, and Android declares `RECORD_AUDIO` in its manifest.

## Architecture

```
src/
  audio/
    window.ts              # Periodic Hann window (librosa/scipy fftbins=True convention)
    streaming-stft.ts      # Stateful STFT — sliding buffer, one magnitude frame per hop
    spectrogram.ts         # Batch STFT (bit-identical calibration; kept for PNG export flow)
    dsp-core.ts            # SpectrogramDsp: display STFT + decoupled 8192-pt detection STFT
                           #   + NoteDetector + QIFFT single-F0 + u8 quantization + level meter
    dsp-worker.ts          # Web Worker shell around SpectrogramDsp
    dsp-protocol.ts        # Typed messages for the main↔worker hop
    mic-pipeline.ts        # getUserMedia → AudioWorklet → DSP worker → frame callback
    file-playback.ts       # AudioBuffer → DSP worker at real-time pace via rAF
    decoder.ts             # File → AudioBuffer (resampled to device rate!); mono downmix; hash
    note-detector.ts       # Multi-F0: whitened salience + iterative harmonic cancellation
    section-labeler.ts     # Detected notes → S/A/T/B via pitch-range DP (A/T? abstention class)
    harmony.ts             # Rehearsal maths: card assignment, chord ID, JI/ensemble targets, pairs,
                           #   consonance, drift, section scatter (ported from the analyzer)
    qifft.ts               # Quadratic-interpolated FFT peak → precise single F0
    intonation.ts          # F0 → nearest ET note + signed cents; rolling summary
    cqt.ts                 # Constant-Q front end (tested; NOT wired — STFT@8192 won, see tests)
    choir-synth.ts         # Seeded synthetic SATB chords — ground truth for detection tests
    choir-bench.ts         # Parametric detection benchmark (full report = skipped test, run on demand)
    spectrogram-lut.ts     # Quantized byte → RGB LUT for the render hot path
  components/
    LiveSpectrogramView.tsx # Vertical waterfall: X = log freq, Y = time scrolling down;
                            #   bottom piano-keyboard band w/ section tints; intonation readout;
                            #   freeze + PNG export
    Sidebar.tsx             # Input toggle, device picker, level meter, STFT/dB/freq params
    FileDrop.tsx            # Drag-and-drop + file picker
    SpectrogramView.tsx     # Batch one-shot render (unwired; kept for the PNG-export flow)
  config/
    fft.ts                  # Default display framing (2048/512)
    detector.ts             # DETECTOR_CONFIG: detection fftSize 8192, salience threshold,
                            #   whitening (Hz-specified), merge radius — the calibration surface
    intonation.ts           # QIFFT range/voicing + cents color scale
    spectrogram-encoding.ts # Wire dB range for u8 frames (floor −140, ceil +12 headroom)
    playback.ts             # File-playback pacing (per-tick advance cap)
    render.ts               # Canvas geometry (batch + waterfall), default dB/freq windows
  utils/
    colormap.ts             # Viridis
    log.ts                  # Structured JSON logger (lifecycle-only; never at frame rate)
    save-png.ts             # PNG export: browser download, or OS save sheet + file write in the Tauri shell
  App.tsx                   # Pipeline wiring, input-mode state, error/warning surface
public/
  mic-worklet.js            # AudioWorklet processor shipping 128-sample chunks (see MicWorkletMsg)
docs/
  TOLERANCES.md             # Numerical tolerance tiers + window-convention rationale
  vocal-acoustics-research.md
```

## How the pipeline works

Both input modes converge on the same DSP worker and the same renderer; the only difference is provenance.

1. **Capture.** Mic mode: `MicPipeline.start` calls `getUserMedia` with echoCancellation / noiseSuppression / autoGainControl explicitly disabled, verifies via `track.getSettings()` that the browser actually honored that (warning banner if not), and wires the stream into an `AudioWorkletNode` (`mic-worklet.js`) that posts 128-sample chunks. File mode: `decodeAudioFile` decodes to an AudioBuffer (note: resampled to the *device* rate by Web Audio), and `FilePlayback` paces mono samples by wall clock under a per-tick cap — a backgrounded tab pauses rather than bursting the missed span.
2. **DSP (off-thread).** Chunks are forwarded (buffer-transferred) to a dedicated worker running `SpectrogramDsp`: a display `StreamingStft` at the sidebar framing, plus a decoupled 8192-point detection STFT feeding the calibrated `NoteDetector` (multi-F0 via whitened salience + iterative harmonic cancellation), plus QIFFT single-F0 for the intonation trail. Display magnitudes are quantized to one u8 per bin over the wire range in `config/spectrogram-encoding.ts`.
3. **Render (main thread).** `LiveSpectrogramView` is a vertical waterfall — X = log frequency, Y = time with the newest row on top, scrolling down. Frames are max-pooled into a pending row and at most one row advances per ~33 ms, so scroll speed is decoupled from the ~90 fps producer. Color is a 256-entry byte→RGB LUT keyed by the dB-clamp window. The bottom band is a piano keyboard that tints active notes by SATB section (B blue / T teal / A green / S amber / A-or-T gray) with the intonation readout beside it. Freeze stops the scroll while capture continues; PNG export saves the canvas.

### Magnitude calibration

Both STFT paths apply the **periodic Hann** window (the librosa/scipy `fftbins=True` convention) and divide magnitudes by `sum(window) / 2`, so a 1.0-amplitude sine at a bin center peaks at 0 dB — enforced bit-identically across the batch and streaming paths by `stft-parity.test.ts`. The wire encoding carries **+12 dB of headroom above full scale** because coherent voices sharing a bin legitimately exceed 0 dBFS (two in-phase unisons sum to +6 dB). Tolerance tiers and the float32 precision floor are documented in `docs/TOLERANCES.md` — read it before generating oracle fixtures or loosening a test.

## Tuning

`src/config/fft.ts` holds the default display framing (2048 / 512 at 44.1 kHz ≈ 21.5 Hz bin width, 11.6 ms hop, 75% overlap); the sidebar swaps among 512–4096 live. Detection framing is independent (`config/detector.ts`, 8192-pt) so bass semitones resolve regardless of the display setting. dB clamp defaults to [−100, 0] — quiet material wants `minDb` toward −120, brick-walled masters toward −60. Frequency range defaults to 50 Hz – 8 kHz (≈G1 up through the singer's-formant region).

## Rehearsal view (harmony)

`src/audio/harmony.ts` runs inside the DSP worker on every detection frame, fed by a second `NoteDetector` capped at 1150 Hz (soprano C6) so high partials never compete for cards. Card assignment is deliberately conservative: a card holding a note follows it (nearest voice within 80 c, whatever the labeler called it that frame); a candidate that sits on partial 2–4 of a held note is a *suspect* — it may take a free card after ~0.3 s (a real octave doubling) but never displaces a held note; a real voice that nobody claims (typically because a card locked onto a phantom at onset) displaces the mis-set card after ~0.6 s. Measured on synthetic 4-singer-per-part chords this holds one chord identity for the full 6 s (previously the hold reset every 1–2 s from octave phantoms) and re-identifies a chord change within a second; a voice that is itself an octave doubling can take ~2 s to land on the right card after a change.

Chord/target rules: a chord must hold 300 ms before targets appear; ±10 c in tune / 20 c out (flat side widened for vibrato-heavy ensembles; ±20/30 for notes under 250 ms); the anchor never gets a "move" instruction; drift is a leaky-memory (μ 0.85) chord offset. Section scatter is the RMS width of the highest isolated partial ≥ 500 Hz with the window width removed in quadrature; > 30 c reads "scattered" instead of sharp/flat.

**Solo / sectional mode** (sidebar) sets `familyMarginDb` to 8: a lone chest voice's H2/H3 then stay in its own card instead of lighting tenor/alto. It is off by default because the same test rejects real equal-level octave partners — the measured trade-off is in `src/config/detector.ts`. Known gap either way: a very low bass (around E2) with a strong second harmonic can still leak one extra card; the temporal harmonicity test (cents deviation of the partial from k·f0 over ~0.5 s) is the next algorithmic step.

## What this does NOT do (yet)

- Audio output during file playback (analysis-only by design; a transport — scrub/pause/seek — is a v1 candidate)
- devicePixelRatio-aware canvas (text softens on 2× displays rendered large; deferred to the WebGL2 renderer)
- Retroactive recolor on dB-clamp change (history rows keep their baked color; needs the byte-history + paint-time LUT shape, also the WebGL2 milestone)
- Multi-channel separation (mono downmix only)
- Individual-singer separation (research north star, not a milestone — see docs/vocal-acoustics-research.md for why a single ambient mic makes this near-frontier)

## Testing

`npm test` runs the Vitest suite (node environment — the suite is DSP math; component tests opt into jsdom per-file). The load-bearing groups:

- **Calibration canaries** — bin-aligned unit sine peaks at 0 dB ±0.3 (44.1 **and** 48 kHz); peak lands on the expected bin. If these fail, every downstream dB reading lies.
- **Batch ↔ streaming parity** (`stft-parity.test.ts`) — bit-identical magnitudes at equal framing, including worklet-sized chunk delivery.
- **librosa oracle** (`oracle-fixtures.test.ts`) — committed fixtures (float32 samples + librosa complex128 magnitudes, `scripts/generate_fixtures.py`) asserted at the cross-implementation tier; measured margin ~60× (see `docs/TOLERANCES.md`). The independent-implementation check self-calibration can't provide.
- **Wire encoding round trip** (`spectrogram-lut.test.ts`) — quantize → LUT decode lands within half a quantization step; endpoint clamps; +6 dBFS headroom canary.
- **Detection contract** (`choir-detection.test.ts`, `choir-bench.ts`) — synthetic SATB chords through the real pipeline; calibrated accuracy floor pinned, physically-hard cases (octave doublings, semitone clusters) documented as honest limits. Un-skip the bench describe for the full parametric report.
- **Section labeling** (`section-labeler.test.ts`) — P3 accuracy floor with the first-class A/T? abstention.
- **Intonation** (`qifft.test.ts`, `intonation.test.ts`) — cents accuracy of the QIFFT F0 path.
- **Pipeline semantics** (`dsp-core.test.ts`, `file-playback.test.ts`) — reframe drops partial frames and detection survives; playback re-anchors instead of bursting after a background gap (mocked clock).
- **Worklet contract** (`mic-worklet.test.ts`) — string-level pin on `public/mic-worklet.js`: processor name, fresh-copy transfer.

## Practices

- **Indexed-access casts**: `noUncheckedIndexedAccess` is on. `(arr[i] as number)` is permitted **only** where the bound is provable within the enclosing function (loop bounds, length checks in the same scope). If you can't point at the proof, handle `undefined`.
- **No allocation or logging in frame-rate paths.** Hot loops pre-allocate; `log` calls are lifecycle-only.
- **Config over literals**: thresholds, framings, ranges, and pacing constants live in `src/config/` — calibration changes should be one-line diffs.
