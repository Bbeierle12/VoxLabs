# Phase 1 gate — running the walking skeleton on the Pixel 10 Pro XL

Plan v3 §5 Phase 1, D18: the latency gate is measured on the phone, not on a
desktop. This is the procedure and what to read off it. Host-side proof of the
mechanism (builder, adapter-chain errors, runner re-framing, taps, timing
stats, contract tests) is `cargo test pipeline::`.

## What runs on the phone

`android_main` spawns the pipeline runner (`src/pipeline/runner.rs`) with
`pipelines/live_model.toml`: YIN → LPC/Levinson → grid inverse → Story tract,
hop 1024 at the device's capture rate, four taps. The not-yet-wrapped per-frame
work (SNR/hum gates, harmonics, voice metrics, spectrogram, room calibration)
runs as a hop observer every second hop — the same 2048-sample frames as
before Phase 1, so every UI readout is unchanged. YIN and LPC therefore run
twice per such frame until Phase 2 wraps the rest; the hop timing below
includes that.

The Room screen's **PIPELINE** card shows, live: each tap's latest value with
its stage's mean/max microseconds, a tube drawn from the `tract` tap, the last
and worst hop against the hop budget and the gate fraction, deadline misses,
analysis and mic-to-render latency against the configured ceiling, backlog,
tap drops, and stage errors. All thresholds come from `live_model.toml`
(`[runner]`).

## Build and install

```bash
export ANDROID_HOME="$HOME/android-sdk" ANDROID_SDK_ROOT="$HOME/android-sdk"
export ANDROID_NDK_ROOT="$HOME/android-sdk/ndk/26.3.11579264" ANDROID_NDK_HOME="$ANDROID_NDK_ROOT"
export RUSTFLAGS="-C link-arg=-Wl,-z,max-page-size=16384"   # Pixel 8+ 16 KB pages
cargo apk build --lib --release        # optimized: the timing gate needs this
adb install -r target/release/apk/vox-core.apk
adb shell pm grant org.voxlabs.core android.permission.RECORD_AUDIO
```

Use the release build for the numbers. The debug build runs the same
pipeline but unoptimized, and its hop times are not the gate's. For the
`org.voxlabs.core.dev` / "VoxLabs (dev)" variant D18 names, patch `package`
and `label` in `Cargo.toml` before building and revert after, as
`docs/STATUS.md` §5 describes; the `pm grant` line then takes the `.dev` id.

## The ten-minute run

1. Launch, open **Room**, confirm the PIPELINE card reads RUNNING and every
   tap shows a value while you sing (a sustained vowel; the `inverse` tap
   reads VALID and the tube moves between vowels).
2. Sing and speak for ten minutes with the app in the foreground. Keep the
   Room screen up so the render leg of the latency is measured every paint.
3. Read the numbers from the card at the end, and from logcat throughout:

```bash
adb logcat -s vox_core::pipeline::runner
```

Every `report_every_hops` hops (512 ≈ 11 s) the runner logs one line:

```
pipeline `live_model` hop 27648: worst hop 6210 us of 21333 us budget, misses 0, over-gate 0,
analysis latency max 7440 us, render latency max 31200 us, backlog max 1024 samples,
tap drops 0, stage errors 0; yin mean 1180 us max 2100 us; lpc mean 640 us max 1350 us;
inverse mean 45 us max 120 us; tract mean 6 us max 20 us
```

(The values above are the line's shape, not measurements.)

## Pass criteria (from `live_model.toml`)

| Quantity | Where | Gate |
|---|---|---|
| Deadline misses over 10 min | `misses` | 0 |
| Worst hop | `worst hop … of … budget` | < `hop_budget_fraction_max` (0.5) × budget (21.3 ms at 48 kHz) |
| Mic-to-render latency | `render latency max` / card | < `mic_to_render_max_ms` (100 ms), measured from ring-buffer arrival to the paint that consumed the `tract` tap; the device's own capture latency is not included |
| Per-stage timing | `yin … lpc … inverse … tract` | reported, no threshold in Phase 1 |
| Tap drops, stage errors | same line | 0 |
| Every tap live | Room card | yes |
| Tube moves | Room card / hero card | yes |

A wrong wiring cannot reach the phone: the builder refuses it and logs the
adapter chain (`cargo test pipeline::tests::wiring_a_wrong_type…` shows the
text). A stage panic on the phone stops the worker, logs it, and the card and
the engine banner read FAILED / "Analysis pipeline stopped".

## Contract tests on both targets

Desktop: `cargo test` (the stage wrappers' contract tests are
`pipeline::stages::*::tests`, the acceptance tests `pipeline::tests`). Android:
the library builds for `aarch64-linux-android`; the same tests run on the
phone with `cargo apk`'s test support or a `cargo ndk` test runner — not set
up in this repository yet, so the Android column of the contract-test report
is "builds; tests not executed on device" until it is.
