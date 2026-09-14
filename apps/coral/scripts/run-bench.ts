import { runBench, BENCH_CASES, formatReport } from '../src/audio/choir-bench';
import { SpectrogramDsp } from '../src/audio/dsp-core';

const SR = 44100;

function detectViaPipeline(samples: Float32Array): Set<number> {
  const dsp = new SpectrogramDsp({
    fftSize: 2048,
    hopSize: 512,
    sampleRate: SR,
    minFreqHz: 50,
    maxFreqHz: 8000,
  });
  let last: number[] = [];
  const CHUNK = 128;
  for (let off = 0; off < samples.length; off += CHUNK) {
    const r = dsp.pushSamples(samples.subarray(off, off + CHUNK));
    if (r.frames.length) {
      last = r.frames[r.frames.length - 1]!.activeMidi;
    }
  }
  return new Set(last);
}

console.log('Running note-detector benchmark battery...');
const start = Date.now();
const report = runBench(BENCH_CASES, detectViaPipeline);
const duration = (Date.now() - start) / 1000;

console.log('\n' + formatReport(report));
console.log(`\nBenchmark completed in ${duration.toFixed(2)}s.`);

// Delimiter to easily parse the final metrics block
console.log('\nMETRICS_JSON_START');
console.log(JSON.stringify({
  overall: report.overall,
  byTag: report.byTag,
}, null, 2));
console.log('METRICS_JSON_END');
