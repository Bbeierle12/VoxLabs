import { log } from "../utils/log";
import type {
  DspOutbound,
  DspHarmonyMsg,
  DspSetHarmonyMsg,
} from "./dsp-protocol";

/**
 * One analyzed magnitude frame produced by the DSP worker, plus the
 * context the renderer needs to interpret it.
 *
 * Magnitudes are quantized to one u8 per FFT bin over the wire dB range
 * (see config/spectrogram-encoding.ts); the renderer decodes them
 * through a colormap LUT. `activeMidi` is the set of notes the worker's
 * detector found in this frame.
 */
export interface MicFrame {
  bytes: Uint8Array;
  numBins: number;
  sampleRate: number;
  activeMidi: number[];
  /** Single best fundamental (QIFFT) for the intonation trail, or null. */
  f0Hz: number | null;
  /** 0..1 voicing confidence of f0Hz. */
  f0Confidence: number;
  timestampMs: number;
}

/**
 * What `public/mic-worklet.js` posts per 128-sample block: a fresh
 * Float32Array copy of the input channel, transferred. The worklet is a
 * plain JS file outside the typed worker protocol (dsp-protocol.ts), so
 * this alias is the contract for the worklet→main hop — see
 * mic-worklet.test.ts for the string-level check on the worklet source.
 */
export type MicWorkletMsg = Float32Array;

export interface MicPipelineCallbacks {
  onFrame?: (frame: MicFrame) => void;
  /** Rehearsal analysis, one per detection frame (~21/s). */
  onHarmony?: (msg: DspHarmonyMsg) => void;
  onLevel?: (peakDb: number, rmsDb: number) => void;
  onError?: (err: Error) => void;
  /**
   * Non-fatal analysis-validity warning — e.g. the browser ignored our
   * request to disable its mic DSP. The pipeline keeps running; the UI
   * should surface the caveat.
   */
  onWarning?: (message: string) => void;
}

export interface MicStartOptions {
  deviceId?: string;
  fftSize: number;
  hopSize: number;
  minFreqHz: number;
  maxFreqHz: number;
}

/**
 * Owns the live mic pipeline: getUserMedia → AudioContext → AudioWorkletNode
 * → DSP worker → onFrame callback.
 *
 * The capture AudioWorklet posts raw 128-sample chunks to this object on
 * the main thread; we immediately forward them (transferring the buffer)
 * to a dedicated DSP worker that runs the STFT, note detection, and
 * quantization. The main thread therefore does no per-sample DSP — only
 * a near-free postMessage relay — which keeps the worklet queue from
 * backing up and the waterfall from drifting behind live audio.
 *
 * The AudioContext sample rate is whatever the device reports (usually
 * 44.1 or 48 kHz). Each frame carries `sampleRate` so the renderer can
 * map FFT bins to frequencies correctly without assuming a value.
 *
 * Mic constraints explicitly disable echoCancellation / noiseSuppression /
 * autoGainControl. Browser DSP would non-deterministically mangle the
 * signal before our STFT sees it, which would silently invalidate any
 * spectral analysis we draw conclusions from.
 */
export class MicPipeline {
  private callbacks: MicPipelineCallbacks;
  private ctx: AudioContext | null = null;
  private stream: MediaStream | null = null;
  private source: MediaStreamAudioSourceNode | null = null;
  private worklet: AudioWorkletNode | null = null;
  private worker: Worker | null = null;

  private running = false;

  constructor(callbacks: MicPipelineCallbacks = {}) {
    this.callbacks = callbacks;
  }

  setCallbacks(callbacks: MicPipelineCallbacks): void {
    this.callbacks = callbacks;
  }

  get isRunning(): boolean {
    return this.running;
  }

  get sampleRate(): number | null {
    return this.ctx?.sampleRate ?? null;
  }

  /**
   * Enumerate audio input devices. Labels are populated only after
   * getUserMedia has been granted at least once in this origin's lifetime
   * — Chrome and Firefox both gate the labels behind a permission grant
   * to prevent fingerprinting.
   */
  static async listAudioInputs(): Promise<MediaDeviceInfo[]> {
    if (!navigator.mediaDevices?.enumerateDevices) {
      throw new Error(
        "MicPipeline.listAudioInputs: enumerateDevices unsupported",
      );
    }
    const all = await navigator.mediaDevices.enumerateDevices();
    return all.filter((d) => d.kind === "audioinput");
  }

  async start(opts: MicStartOptions): Promise<void> {
    if (this.running) {
      throw new Error("MicPipeline.start: already running — call stop() first");
    }

    log.info("mic:start:request", {
      deviceId: opts.deviceId ?? "default",
      fftSize: opts.fftSize,
      hopSize: opts.hopSize,
    });

    let stream: MediaStream;
    try {
      stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          deviceId: opts.deviceId ? { exact: opts.deviceId } : undefined,
          echoCancellation: false,
          noiseSuppression: false,
          autoGainControl: false,
        },
      });
    } catch (err) {
      throw new Error(
        `MicPipeline.start: getUserMedia failed — ${(err as Error).message}`,
      );
    }

    // Constraints are REQUESTS — browsers may ignore them per device.
    // If any browser DSP survived, every dB reading downstream is
    // suspect, so say so loudly rather than analyze a mangled signal.
    const settings = stream.getAudioTracks()[0]?.getSettings() ?? {};
    const survived = (
      ["echoCancellation", "noiseSuppression", "autoGainControl"] as const
    ).filter((k) => settings[k] === true);
    if (survived.length > 0) {
      log.warn("mic:constraints:overridden", { survived });
      this.callbacks.onWarning?.(
        `Browser kept ${survived.join(", ")} enabled on this mic — ` +
          `spectral readings may be altered by browser DSP.`,
      );
    }

    const ctx = new AudioContext();

    try {
      await ctx.audioWorklet.addModule("/mic-worklet.js");
    } catch (err) {
      stream.getTracks().forEach((t) => t.stop());
      void ctx.close();
      throw new Error(
        `MicPipeline.start: failed to load audio worklet — ${(err as Error).message}`,
      );
    }

    const source = ctx.createMediaStreamSource(stream);
    const worklet = new AudioWorkletNode(ctx, "mic-capture");
    const worker = this.spawnWorker(ctx.sampleRate);
    worker.postMessage({
      type: "init",
      fftSize: opts.fftSize,
      hopSize: opts.hopSize,
      sampleRate: ctx.sampleRate,
      minFreqHz: opts.minFreqHz,
      maxFreqHz: opts.maxFreqHz,
    });

    // Forward raw capture chunks to the worker, transferring the buffer
    // (the worklet hands us a fresh copy each block, so we don't need it).
    worklet.port.onmessage = (ev) => {
      if (!this.worker) return;
      const chunk = ev.data as MicWorkletMsg;
      this.worker.postMessage({ type: "samples", samples: chunk }, [
        chunk.buffer,
      ]);
    };

    source.connect(worklet);
    // We do NOT connect worklet to destination — that would feed mic
    // back through speakers and create a howl loop.

    this.ctx = ctx;
    this.stream = stream;
    this.source = source;
    this.worklet = worklet;
    this.worker = worker;
    this.running = true;

    log.info("mic:start:ok", { sampleRate: ctx.sampleRate });
  }

  /**
   * Swap STFT framing — used when the user changes fftSize or hopSize
   * from the sidebar without restarting the mic. The worker drops any
   * partial-frame samples; the next frame lands `fftSize` samples on.
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

  stop(): void {
    if (!this.running) return;

    try {
      this.worklet?.port.close();
    } catch {
      // ignore
    }
    try {
      this.source?.disconnect();
    } catch {
      // ignore
    }
    try {
      this.worklet?.disconnect();
    } catch {
      // ignore
    }
    this.worker?.terminate();
    this.stream?.getTracks().forEach((t) => t.stop());
    void this.ctx?.close();

    this.ctx = null;
    this.stream = null;
    this.source = null;
    this.worklet = null;
    this.worker = null;
    this.running = false;
    log.info("mic:stop", {});
  }

  // sampleRate is passed in because this runs BEFORE this.ctx is
  // assigned (and an AudioContext's rate never changes anyway).
  private spawnWorker(sampleRateHz: number): Worker {
    const worker = new Worker(new URL("./dsp-worker.ts", import.meta.url), {
      type: "module",
    });
    worker.onmessage = (ev: MessageEvent<DspOutbound>) => {
      const msg = ev.data;
      if (msg.type === "harmony") {
        this.callbacks.onHarmony?.(msg);
        return;
      }
      if (msg.type === "frame") {
        this.callbacks.onFrame?.({
          bytes: msg.bytes,
          numBins: msg.numBins,
          sampleRate: sampleRateHz,
          activeMidi: msg.activeMidi,
          f0Hz: msg.f0Hz,
          f0Confidence: msg.f0Confidence,
          timestampMs: performance.now(),
        });
      } else {
        this.callbacks.onLevel?.(msg.peakDb, msg.rmsDb);
      }
    };
    worker.onerror = (ev) => {
      this.callbacks.onError?.(new Error(`dsp-worker: ${ev.message}`));
    };
    return worker;
  }
}
