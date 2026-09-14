import { describe, it, expect } from "vitest";
import { synthesizeChoirChord } from "./choir-synth";
import { StreamingStft } from "./streaming-stft";
import { NoteDetector } from "./note-detector";
import { DETECTOR_CONFIG } from "../config/detector";

const SR = 44100;
function detect(samples: Float32Array, familyMarginDb: number): number[] {
  const stft = new StreamingStft(8192, 2048);
  const det = new NoteDetector({
    numBins: 4096,
    sampleRate: SR,
    minFreqHz: 50,
    maxFreqHz: 8000,
    harmonic: true,
    harmonicCount: DETECTOR_CONFIG.harmonicCount,
    salienceThreshold: DETECTOR_CONFIG.salienceThreshold,
    decayPerFrame: DETECTOR_CONFIG.decayPerFrame,
    familyMarginDb,
  });
  let last = new Set<number>();
  for (let o = 0; o < samples.length; o += 128)
    for (const m of stft.pushSamples(samples.subarray(o, o + 128)))
      last = det.analyze(m);
  return [...last].sort((a, b) => a - b);
}
const chest = (midi: number) =>
  synthesizeChoirChord({
    sampleRate: SR,
    durationSec: 0.4,
    seed: 7,
    voices: [
      {
        section: "B",
        midi,
        harmonicDb: [-14, -6, -12, -10, -18, -24, -28, -32],
        singers: 1,
        detuneCents: 0,
      },
    ],
  }).samples;

describe("harmonic-family test (Analyzer merge)", () => {
  it("with the margin on, one chest-voice singer (H2 above H1) yields exactly one note", () => {
    expect(detect(chest(50), 8)).toEqual([50]);
    expect(detect(chest(55), 8)).toEqual([55]);
  });
  it("with the margin off, the same singer sprouts octave phantoms (documents the known limit)", () => {
    expect(detect(chest(50), 0).length).toBeGreaterThan(1);
  });
  it("a soprano an octave+ above a smooth bass survives the family test", () => {
    const { samples } = synthesizeChoirChord({
      sampleRate: SR,
      durationSec: 0.4,
      seed: 7,
      voices: [
        {
          section: "B",
          midi: 48,
          harmonicDb: [-10, -14, -18, -24, -28, -32, -36, -40],
          singers: 1,
          detuneCents: 0,
        },
        {
          section: "S",
          midi: 72,
          harmonicDb: [-10, -19, -28, -37],
          singers: 1,
          detuneCents: 0,
        },
      ],
    });
    expect(detect(samples, 8)).toEqual([48, 72]);
  });
});
