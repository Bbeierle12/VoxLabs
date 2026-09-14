import { runBench, BENCH_CASES, formatReport } from '../src/audio/choir-bench';
import { SpectrogramDsp } from '../src/audio/dsp-core';
import { StreamingStft } from '../src/audio/streaming-stft';
// @ts-ignore
import * as CHA from './cha-analysis.mjs';
const SR = 44100;
function coral(samples: Float32Array): Set<number> {
  const dsp = new SpectrogramDsp({ fftSize: 2048, hopSize: 512, sampleRate: SR, minFreqHz: 50, maxFreqHz: 8000 });
  let last: number[] = [];
  for (let off = 0; off < samples.length; off += 128) { const r = dsp.pushSamples(samples.subarray(off, off + 128)); if (r.frames.length) last = r.frames[r.frames.length - 1]!.activeMidi; }
  return new Set(last);
}
function cha(samples: Float32Array): Set<number> {
  const stft = new StreamingStft(8192, 2048); let lastMags: Float32Array | null = null;
  for (let off = 0; off < samples.length; off += 128) { const fr = stft.pushSamples(samples.subarray(off, off + 128)); if (fr.length) lastMags = fr[fr.length - 1]!; }
  if (!lastMags) return new Set();
  const spec = new Float32Array(lastMags.length); for (let b = 0; b < spec.length; b++) spec[b] = Math.max(-140, 20 * Math.log10(lastMags[b]! + 1e-12));
  CHA.cfg.thr = -82; CHA.cfg.sections = { B: true, T: true, A: true, S: true };
  const d = CHA.detectVoices(spec, SR / 8192);
  return new Set(d.voices.map((v: any) => Math.round(69 + 12 * Math.log2(v.f / 440))));
}
for (const [name, fn] of [['Coral NoteDetector', coral], ['CHA detectVoices', cha]] as const) {
  const rep = runBench(BENCH_CASES, fn as any);
  console.log(`\n=== ${name} ===\n` + formatReport(rep));
}
