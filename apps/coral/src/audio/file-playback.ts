import { toMonoSamples } from "./decoder";
import { PLAYBACK_CONFIG } from "../config/playback";
import { log } from "../utils/log";
import type { MicFrame } from "./mic-pipeline";
import type {
  DspOutbound,
  DspHarmonyMsg,
  DspSetHarmonyMsg,
} from "./dsp-protocol";

export interface FilePlaybackCallbacks {
  onFrame?: (frame: MicFrame) => void;
  /** Rehearsal analysis, one per detection frame (~21/s). */
  onHarmony?: (msg: DspHarmonyMsg) => void;
  onComplete?: () => void;
  onError?: (err: Error) => void;
}

export interface FilePlaybackStartOptions {
  buffer: AudioBuffer;
  fftSize: number;
  hopSize: number;
  minFreqHz: number;
  maxFreqHz: number;
}

/**
 * Plays a decoded AudioBuffer through the DSP worker at real-time pace,
 * emitting magnitude frames in the same shape as MicPipeline. Renderer
 * code (LiveSpectrogramView) treats mic frames and file frames
 * identically — the only difference is provenance.
 *
 * Pacing is driven by requestAnimationFrame and an elapsed-time cursor,
 * not setTimeout, so playback stays in lockstep with the renderer's
 * paint cadence. Each tick slices the samples elapsed since the last
 * tick and ships them to the worker; the FFT itself runs off the main
 * thread. rAF suspends in a backgrounded tab while performance.now()
 * keeps running, so the first tick after restore re-anchors the clock
 * instead of shipping the missed span — backgrounding pauses playback
 * rather than bursting it (see PLAYBACK_CONFIG.maxTickAdvanceSec).
 *
 * One playback session per instance — calling start() while running
 * stops the previous session and begins a new one.
 */
export class FilePlayback {
  private callbacks: FilePlaybackCallbacks;
  private rafId: number | null = null;
  private startTimeMs = 0;
  private samples: Float32Array | null = null;
  private sampleRateHz = 0;
  private worker: Worker | null = null;
  private cursor = 0;
  private running = false;

  constructor(callbacks: FilePlaybackCallbacks = {}) {
    this.callbacks = callbacks;
  }

  setCallbacks(callbacks: FilePlaybackCallbacks): void {
    this.callbacks = callbacks;
  }

  get isRunning(): boolean {
    return this.running;
  }

  get sampleRate(): number | null {
    return this.running ? this.sampleRateHz : null;
  }

  start(opts: FilePlaybackStartOptions): void {
    this.stop();
    this.samples = toMonoSamples(opts.buffer);
    this.sampleRateHz = opts.buffer.sampleRate;
    this.worker = this.spawnWorker();
    this.worker.postMessage({
      type: "init",
      fftSize: opts.fftSize,
      hopSize: opts.hopSize,
      sampleRate: this.sampleRateHz,
      minFreqHz: opts.minFreqHz,
      maxFreqHz: opts.maxFreqHz,
    });
    this.cursor = 0;
    this.startTimeMs = performance.now();
    this.running = true;
    log.info("file:start", {
      durationSec: opts.buffer.duration,
      sampleRate: this.sampleRateHz,
      fftSize: opts.fftSize,
      hopSize: opts.hopSize,
    });
    this.tick();
  }

  stop(): void {
    if (this.rafId !== null) cancelAnimationFrame(this.rafId);
    this.rafId = null;
    if (this.running) log.info("file:stop", { cursor: this.cursor });
    this.worker?.terminate();
    this.worker = null;
    this.running = false;
    this.samples = null;
    this.cursor = 0;
  }

  /**
   * Swap STFT framing mid-playback (sidebar param change). The worker
   * drops any partial-frame samples — same semantics as MicPipeline.
   * Cursor is preserved so playback continues from the same position.
   */
  setFraming(fftSize: number, hopSize: number): void {
    this.worker?.postMessage({ type: "reframe", fftSize, hopSize });
  }

  /** Update the note-detection frequency range without restarting. */
  /** Push rehearsal-view settings to the worker (no-op when not running). */
  setHarmony(settings: Omit<DspSetHarmonyMsg, "type">): void {
    this.worker?.postMessage({ type: "setHarmony", ...settings });
  }

  setFreqRange(minFreqHz: number, maxFreqHz: number): void {
    this.worker?.postMessage({ type: "setFreqRange", minFreqHz, maxFreqHz });
  }

  private tick = (): void => {
    if (!this.running || !this.samples || !this.worker) return;

    const elapsedMs = performance.now() - this.startTimeMs;
    let targetSample = Math.min(
      this.samples.length,
      Math.floor((elapsedMs / 1000) * this.sampleRateHz),
    );

    // A gap beyond the per-tick cap means rAF was suspended (backgrounded
    // tab) while the wall clock ran. Re-anchor so the cursor's position
    // corresponds to "now" and ship nothing this tick — playback resumes
    // from where it paused instead of collapsing the missed span into a
    // single burst the renderer would fold into one row.
    const maxAdvance = Math.floor(
      PLAYBACK_CONFIG.maxTickAdvanceSec * this.sampleRateHz,
    );
    if (targetSample - this.cursor > maxAdvance) {
      this.startTimeMs =
        performance.now() - (this.cursor / this.sampleRateHz) * 1000;
      targetSample = this.cursor;
      log.info("file:reanchor", { cursor: this.cursor });
    }

    if (targetSample > this.cursor) {
      // slice() copies, so the transfer below can't detach our backing
      // sample buffer (subarray would share it).
      const chunk = this.samples.slice(this.cursor, targetSample);
      this.worker.postMessage({ type: "samples", samples: chunk }, [
        chunk.buffer,
      ]);
      this.cursor = targetSample;
    }

    if (this.cursor >= this.samples.length) {
      this.running = false;
      this.rafId = null;
      log.info("file:complete", {});
      this.callbacks.onComplete?.();
      return;
    }

    this.rafId = requestAnimationFrame(this.tick);
  };

  private spawnWorker(): Worker {
    const worker = new Worker(new URL("./dsp-worker.ts", import.meta.url), {
      type: "module",
    });
    worker.onmessage = (ev: MessageEvent<DspOutbound>) => {
      const msg = ev.data;
      if (msg.type === "harmony") {
        this.callbacks.onHarmony?.(msg);
        return;
      }
      // File playback has no level meter; ignore level messages.
      if (msg.type !== "frame") return;
      this.callbacks.onFrame?.({
        bytes: msg.bytes,
        numBins: msg.numBins,
        sampleRate: this.sampleRateHz,
        activeMidi: msg.activeMidi,
        f0Hz: msg.f0Hz,
        f0Confidence: msg.f0Confidence,
        timestampMs: performance.now(),
      });
    };
    worker.onerror = (ev) => {
      this.callbacks.onError?.(new Error(`dsp-worker: ${ev.message}`));
    };
    return worker;
  }
}
