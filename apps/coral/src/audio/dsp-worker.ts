/**
 * DSP worker. Owns one SpectrogramDsp and runs all per-frame analysis
 * (STFT, note detection, dB→u8 quantization, level metering) off the
 * main thread. The main-thread pipelines (mic-pipeline, file-playback)
 * feed it sample chunks and relay its frame/level output to the UI.
 *
 * Vite bundles this as a module worker via
 *   new Worker(new URL('./dsp-worker.ts', import.meta.url), { type: 'module' })
 */
import { SpectrogramDsp } from "./dsp-core";
import type { DspInbound, DspOutbound } from "./dsp-protocol";

// The DOM lib (no WebWorker lib in this tsconfig) types `self` as a
// Window, whose postMessage signature differs from a worker's. Narrow to
// just what we use to keep transfer-list calls well-typed.
interface DspWorkerScope {
  postMessage(message: DspOutbound, transfer?: Transferable[]): void;
  onmessage: ((ev: MessageEvent<DspInbound>) => void) | null;
}
const ctx = self as unknown as DspWorkerScope;

let dsp: SpectrogramDsp | null = null;

ctx.onmessage = (ev: MessageEvent<DspInbound>) => {
  const msg = ev.data;
  switch (msg.type) {
    case "init":
      dsp = new SpectrogramDsp({
        fftSize: msg.fftSize,
        hopSize: msg.hopSize,
        sampleRate: msg.sampleRate,
        minFreqHz: msg.minFreqHz,
        maxFreqHz: msg.maxFreqHz,
      });
      break;

    case "samples": {
      if (!dsp) break;
      const { frames, level, harmony } = dsp.pushSamples(msg.samples);
      if (level) {
        ctx.postMessage({
          type: "level",
          peakDb: level.peakDb,
          rmsDb: level.rmsDb,
        });
      }
      for (const h of harmony) {
        ctx.postMessage({
          type: "harmony",
          voices: h.voices,
          result: h.result,
        });
      }
      const numBins = dsp.numBins;
      for (const frame of frames) {
        ctx.postMessage(
          {
            type: "frame",
            bytes: frame.bytes,
            numBins,
            activeMidi: frame.activeMidi,
            f0Hz: frame.f0Hz,
            f0Confidence: frame.f0Confidence,
          },
          [frame.bytes.buffer],
        );
      }
      break;
    }

    case "reframe":
      dsp?.setFraming(msg.fftSize, msg.hopSize);
      break;

    case "setFreqRange":
      dsp?.setFreqRange(msg.minFreqHz, msg.maxFreqHz);
      break;

    case "reset":
      dsp?.reset();
      break;

    case "setHarmony": {
      const { type: _t, ...rest } = msg;
      void _t;
      dsp?.setHarmony(rest);
      break;
    }
  }
};
