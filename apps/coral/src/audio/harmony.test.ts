import { describe, it, expect } from "vitest";
import {
  HarmonyAnalyzer,
  identifyChord,
  midiToHz,
  noteInfo,
  type VoiceIn,
} from "./harmony";
import { synthesizeChoirChord } from "./choir-synth";
import { SpectrogramDsp } from "./dsp-core";
import type { Section } from "./section-labeler";

const A4 = 440;
const P = (n: number, oct: number, c = 0) =>
  midiToHz(12 * (oct + 1) + n, A4) * Math.pow(2, c / 1200);
const NOTE: Record<string, number> = {
  C: 0,
  D: 2,
  E: 4,
  F: 5,
  G: 7,
  A: 9,
  B: 11,
};
const v = (section: Section, name: string, oct: number, c = 0): VoiceIn => {
  const f0Hz = P(NOTE[name] as number, oct, c);
  return {
    midi: noteInfo(f0Hz, A4).midi,
    f0Hz,
    salience: 2,
    section,
    label: section,
    sectionConf: 0.8,
  };
};
/** Feed the same voices for `frames` detection frames 47 ms apart, return the last result. */
function run(h: HarmonyAnalyzer, voices: VoiceIn[], frames = 12, t0 = 1000) {
  let r = h.analyze(voices, null, 44100 / 8192, -120, t0);
  for (let i = 1; i < frames; i++)
    r = h.analyze(voices, null, 44100 / 8192, -120, t0 + i * 47);
  return r;
}
const cmajPure = [
  v("B", "C", 3),
  v("T", "G", 3, 2.0),
  v("A", "E", 4, -13.7),
  v("S", "C", 5),
];

describe("harmony: targets, profiles, anchoring", () => {
  it("a pure C major reads 0 error under the Pure profile and −7.7 c on the third under Ensemble", () => {
    const h = new HarmonyAnalyzer();
    h.setConfig({ profile: "pure" });
    const r = run(h, cmajPure);
    expect(r.id?.chord.name).toBe("Major");
    expect(r.settled).toBe(true);
    for (const t of r.tg) expect(Math.abs(t?.err ?? 99)).toBeLessThan(0.5);
    h.setConfig({ profile: "ensemble" });
    const r2 = run(h, cmajPure);
    const alto = r2.voices.findIndex((x) => x.section === "A");
    expect(r2.tg[alto]?.err).toBeCloseTo(-7.7, 0);
  });
  it("anchors on the bass: detuning the bass +10 c leaves the upper voices at 0 relative error", () => {
    const h = new HarmonyAnalyzer();
    h.setConfig({ profile: "pure" });
    const det = cmajPure.map((x, i) =>
      i === 0 ? { ...x, f0Hz: x.f0Hz * Math.pow(2, 10 / 1200) } : x,
    );
    const r = run(h, det);
    expect(r.tg[0]?.anchor).toBe(true);
    expect(Math.abs(r.voices[0]!.cents - 10)).toBeLessThan(0.1);
    for (let i = 1; i < 4; i++)
      expect(Math.abs(r.tg[i]!.err - -10)).toBeLessThan(0.5);
  });
  it("identifies a dominant seventh and sizes the 5th→7th pair as 7:6", () => {
    const r = run(new HarmonyAnalyzer(), [
      v("B", "G", 2),
      v("T", "D", 4, 2),
      v("A", "F", 4, -31.2),
      v("S", "B", 4, -13.7),
    ]);
    expect(r.id?.chord.name).toBe("Dom 7");
    const p = r.pairs.find((x) => x.a.name === "D" && x.b.name === "F");
    expect(p?.ratio).toBe("7:6");
  });
  it("does not settle before 300 ms and reports drift after 20 flat updates", () => {
    const h = new HarmonyAnalyzer();
    expect(run(h, cmajPure, 4).settled).toBe(false);
    run(h, cmajPure, 12, 5000); // in-tune start sets r0
    const flat = cmajPure.map((x) => ({
      ...x,
      f0Hz: x.f0Hz * Math.pow(2, -30 / 1200),
    }));
    let r = h.analyze(flat, null, 44100 / 8192, -120, 20000);
    for (let i = 1; i < 22; i++)
      r = h.analyze(flat, null, 44100 / 8192, -120, 20000 + i * 2100);
    expect(r.drift).toBeCloseTo(-30 * (1 - Math.pow(0.85, 20)), 0);
  });
  it("drops voices whose section is switched off", () => {
    const h = new HarmonyAnalyzer();
    h.setConfig({ sections: { A: false, S: false } });
    const r = run(h, cmajPure);
    expect(r.voices.map((x) => x.section)).toEqual(["B", "T"]);
  });
  it("identifyChord: cluster for a non-tertian set", () => {
    expect(identifyChord([{ pc: 0 }, { pc: 1 }, { pc: 6 }])?.cluster).toBe(
      true,
    );
  });
});

describe("harmony: through the real pipeline", () => {
  it("a synthesised C major (one singer per part) yields four voices within 3 c of the synth and a Major chord", () => {
    const SR = 44100;
    const { samples } = synthesizeChoirChord({
      sampleRate: SR,
      durationSec: 1.5,
      seed: 3,
      voices: [
        { section: "B", midi: 48, singers: 1, detuneCents: 0, vibratoCents: 0 },
        { section: "T", midi: 55, singers: 1, detuneCents: 0, vibratoCents: 0 },
        { section: "A", midi: 64, singers: 1, detuneCents: 0, vibratoCents: 0 },
        { section: "S", midi: 72, singers: 1, detuneCents: 0, vibratoCents: 0 },
      ],
    });
    const dsp = new SpectrogramDsp({
      fftSize: 2048,
      hopSize: 512,
      sampleRate: SR,
      minFreqHz: 50,
      maxFreqHz: 8000,
    });
    let t = 0;
    dsp.now = () => t;
    let last = null as
      ReturnType<SpectrogramDsp["pushSamples"]>["harmony"][number] | null;
    for (let o = 0; o < samples.length; o += 128) {
      t += (128 / SR) * 1000;
      const r = dsp.pushSamples(samples.subarray(o, o + 128));
      if (r.harmony.length) last = r.harmony[r.harmony.length - 1]!;
    }
    expect(last).not.toBeNull();
    const res = last!.result;
    expect(res.voices.map((x) => x.midi)).toEqual([48, 55, 64, 72]);
    expect(res.voices.map((x) => x.section)).toEqual(["B", "T", "A", "S"]);
    for (const x of res.voices) expect(Math.abs(x.cents)).toBeLessThan(3);
    expect(res.id?.chord.name).toBe("Major");
    expect(res.settled).toBe(true);
  });

  const FOUR_BY_FOUR = (
    midis: [number, number, number, number],
    durationSec: number,
    seed: number,
  ) =>
    synthesizeChoirChord({
      sampleRate: 48000,
      durationSec,
      seed,
      voices: (["B", "T", "A", "S"] as const).map((section, i) => ({
        section,
        midi: midis[i]!,
        singers: 4,
        detuneCents: 6,
        vibratoCents: 15,
      })),
    });

  it("a 4-singer-per-part C major holds one chord identity for 6 s (no hold resets from octave phantoms)", () => {
    const SR = 48000;
    const { samples } = FOUR_BY_FOUR([48, 55, 64, 72], 6, 11);
    const dsp = new SpectrogramDsp({
      fftSize: 2048,
      hopSize: 512,
      sampleRate: SR,
      minFreqHz: 50,
      maxFreqHz: 8000,
    });
    let t = 0;
    dsp.now = () => t;
    const keys = new Set<string>();
    let maxHeld = 0;
    for (let o = 0; o < samples.length; o += 128) {
      t += (128 / SR) * 1000;
      for (const h of dsp.pushSamples(samples.subarray(o, o + 128)).harmony) {
        if (t < 1000) continue; // onset + lock-in escape
        keys.add(
          `${h.result.id?.root}:${h.result.id?.chord.name}:${h.result.voices.length}`,
        );
        maxHeld = Math.max(maxHeld, h.result.chordHeldMs);
      }
    }
    expect([...keys]).toEqual(["0:Major:4"]);
    expect(maxHeld).toBeGreaterThan(4500);
  });

  it("still follows a real chord change: C major → F major identifies within a second and lands all four voices within ~2.5 s", () => {
    const SR = 48000;
    const a = FOUR_BY_FOUR([48, 55, 64, 72], 2, 5).samples;
    const b = FOUR_BY_FOUR([53, 57, 60, 69], 3.5, 6).samples;
    const samples = new Float32Array(a.length + b.length);
    samples.set(a);
    samples.set(b, a.length);
    const dsp = new SpectrogramDsp({
      fftSize: 2048,
      hopSize: 512,
      sampleRate: SR,
      minFreqHz: 50,
      maxFreqHz: 8000,
    });
    let t = 0;
    dsp.now = () => t;
    let firstF: number | null = null;
    let last = null as
      ReturnType<SpectrogramDsp["pushSamples"]>["harmony"][number] | null;
    for (let o = 0; o < samples.length; o += 128) {
      t += (128 / SR) * 1000;
      for (const h of dsp.pushSamples(samples.subarray(o, o + 128)).harmony) {
        last = h;
        if (
          firstF == null &&
          h.result.id?.root === 5 &&
          h.result.id.chord.name === "Major" &&
          h.result.settled
        )
          firstF = t;
      }
    }
    expect(firstF).not.toBeNull();
    expect(firstF! - 2000).toBeLessThan(1000);
    expect(last!.result.voices.map((x) => x.midi)).toEqual([53, 57, 60, 69]);
  });
});
