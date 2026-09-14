/**
 * Message protocol between the main thread and the DSP worker
 * (dsp-worker.ts). Types only — no runtime, no DSP imports — so both
 * the worker and the main-thread pipelines can import it without pulling
 * the FFT dependency into each other's bundle.
 */

export interface DspInitMsg {
  type: "init";
  fftSize: number;
  hopSize: number;
  sampleRate: number;
  minFreqHz: number;
  maxFreqHz: number;
}

/** Raw audio samples to analyze. `samples.buffer` is transferred. */
export interface DspSamplesMsg {
  type: "samples";
  samples: Float32Array;
}

export interface DspReframeMsg {
  type: "reframe";
  fftSize: number;
  hopSize: number;
}

export interface DspFreqRangeMsg {
  type: "setFreqRange";
  minFreqHz: number;
  maxFreqHz: number;
}

export interface DspResetMsg {
  type: "reset";
}

/** Rehearsal-view settings that live in the worker (see harmony.ts / DETECTOR_CONFIG). */
export interface DspSetHarmonyMsg {
  type: "setHarmony";
  a4?: number;
  profile?: "pure" | "ensemble" | "equal";
  vibratoHeavy?: boolean;
  sections?: Partial<Record<"B" | "T" | "A" | "S", boolean>>;
  /** NoteDetector familyMarginDb (0 = off; 8 = solo / sectional rehearsal). Rebuilds the detector. */
  familyMarginDb?: number;
  /** Reset the drift reference (r0) without touching anything else. */
  resetDrift?: boolean;
}

export type DspInbound =
  | DspInitMsg
  | DspSamplesMsg
  | DspReframeMsg
  | DspFreqRangeMsg
  | DspResetMsg
  | DspSetHarmonyMsg;

/** One analyzed magnitude column. `bytes.buffer` is transferred. */
export interface DspFrameMsg {
  type: "frame";
  bytes: Uint8Array;
  numBins: number;
  activeMidi: number[];
  /** Single best fundamental (QIFFT) for the intonation trail, or null. */
  f0Hz: number | null;
  /** 0..1 voicing confidence of f0Hz. */
  f0Confidence: number;
}

/** One detected voice: an accepted note with its sub-bin pitch and section label. */
export interface DspVoice {
  midi: number;
  /** Parabolic-interpolated fundamental on the detection STFT, Hz. */
  f0Hz: number;
  /** Detector confidence (salience / threshold; ≥ 1 active). */
  salience: number;
  section: "B" | "T" | "A" | "S";
  label: "B" | "T" | "A" | "S" | "A/T?";
  sectionConf: number;
}

/**
 * Rehearsal analysis, one per DETECTION frame (~21/s). Carries the raw
 * voices and the HarmonyAnalyzer result (chord, targets, pairs, drift, …).
 * The result is plain data (structured-clone safe).
 */
export interface DspHarmonyMsg {
  type: "harmony";
  voices: DspVoice[];
  result: import("./harmony").HarmonyResult;
}

export interface DspLevelMsg {
  type: "level";
  peakDb: number;
  rmsDb: number;
}

export type DspOutbound = DspFrameMsg | DspLevelMsg | DspHarmonyMsg;
