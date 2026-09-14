#!/usr/bin/env python3
"""Generate librosa oracle fixtures for Coral's STFT.

Each fixture stores BOTH the input samples and librosa's expected
magnitudes, so the TypeScript side never re-synthesizes the signal —
the comparison is purely STFT vs STFT on bit-identical float32 input.

Conventions matched to Coral (src/audio/streaming-stft.ts):
  - periodic Hann window (fftbins=True)
  - no padding / no centering: frame f covers [f*hop, f*hop + n_fft)
  - bins 0 .. n_fft/2 - 1 (Nyquist dropped)
  - single-sided amplitude scaling: |X| * 2 / sum(window)

Bin 0 is stored at librosa's true value but EXCLUDED from the TS
assertion: Coral applies the 2x one-sided factor to DC where it
shouldn't (documented wart in spectrogram.ts).

Output: src/audio/__fixtures__/oracle/<name>.json
Regeneration: see scripts/requirements-fixtures.txt. Fixtures are
committed; CI never runs this.
"""

import json
import pathlib

import librosa
import numpy as np

OUT_DIR = pathlib.Path(__file__).resolve().parent.parent / "src" / "audio" / "__fixtures__" / "oracle"


def synth_sine(freq_hz: float, n: int, sr: int, amp: float = 1.0) -> np.ndarray:
    t = np.arange(n, dtype=np.float64) / sr
    return amp * np.sin(2 * np.pi * freq_hz * t)


def synth_two_tone_noise(n: int, sr: int, seed: int = 0x12345678) -> np.ndarray:
    """Two non-bin-aligned tones + seeded uniform noise — broadband content."""
    rng = np.random.default_rng(seed)
    t = np.arange(n, dtype=np.float64) / sr
    return (
        0.5 * np.sin(2 * np.pi * 440.7 * t)
        + 0.3 * np.sin(2 * np.pi * 1337.2 * t)
        + (rng.random(n) - 0.5) * 0.1
    )


def synth_chirp(f0: float, f1: float, n: int, sr: int) -> np.ndarray:
    """Linear chirp via the phase integral (no scipy dependency)."""
    t = np.arange(n, dtype=np.float64) / sr
    dur = n / sr
    phase = 2 * np.pi * (f0 * t + (f1 - f0) / (2 * dur) * t * t)
    return 0.8 * np.sin(phase)


def oracle_magnitudes(y32: np.ndarray, n_fft: int, hop: int) -> np.ndarray:
    """librosa STFT magnitudes with Coral's scaling, bins [0, n_fft/2)."""
    S = librosa.stft(
        y32.astype(np.float64),  # explicit: oracle math in float64
        n_fft=n_fft,
        hop_length=hop,
        window="hann",  # periodic (fftbins=True) — librosa's default
        center=False,
    )
    window = librosa.filters.get_window("hann", n_fft, fftbins=True)
    mags = np.abs(S) * (2.0 / np.sum(window))
    return mags[: n_fft // 2].T  # rows = frames, cols = bins


def write_fixture(name: str, y: np.ndarray, sr: int, n_fft: int, hop: int) -> None:
    y32 = y.astype(np.float32)  # cast FIRST — both sides see these exact values
    mags = oracle_magnitudes(y32, n_fft, hop)
    fixture = {
        "name": name,
        "sampleRate": sr,
        "fftSize": n_fft,
        "hopSize": hop,
        "numBins": n_fft // 2,
        "frames": mags.shape[0],
        "librosaVersion": librosa.__version__,
        "samples": [float(v) for v in y32],
        # rows = frames; bin 0 stored but not asserted (Coral's DC wart)
        "expectedMagnitudes": [[float(v) for v in row] for row in mags],
    }
    path = OUT_DIR / f"{name}.json"
    path.write_text(json.dumps(fixture, separators=(",", ":")) + "\n")
    print(f"{name}: {mags.shape[0]} frames x {mags.shape[1]} bins -> {path.stat().st_size / 1024:.0f} KB")


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    def n_for_frames(n_fft: int, hop: int, frames: int) -> int:
        return n_fft + (frames - 1) * hop

    # 2048/512 cases, 8 frames each.
    n = n_for_frames(2048, 512, 8)
    sr = 44100
    write_fixture("sine-bin100-44k1-2048", synth_sine(100 * sr / 2048, n, sr), sr, 2048, 512)
    write_fixture("sine-440p7-44k1-2048", synth_sine(440.7, n, sr), sr, 2048, 512)
    write_fixture("twotone-noise-44k1-2048", synth_two_tone_noise(n, sr), sr, 2048, 512)
    write_fixture("chirp-100-4k-44k1-2048", synth_chirp(100, 4000, n, sr), sr, 2048, 512)

    sr48 = 48000
    write_fixture("sine-bin100-48k-2048", synth_sine(100 * sr48 / 2048, n, sr48), sr48, 2048, 512)

    # 4096/1024, 4 frames — nearer the detection-STFT regime.
    n4 = n_for_frames(4096, 1024, 4)
    write_fixture("sine-440p7-44k1-4096", synth_sine(440.7, n4, sr), sr, 4096, 1024)


if __name__ == "__main__":
    main()
