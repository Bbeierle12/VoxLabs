import { StreamingStft } from "./streaming-stft";
import { NoteDetector } from "./note-detector";
import { estimateF0, parabolicPeak } from "./qifft";
import { labelSections } from "./section-labeler";
import {
  HarmonyAnalyzer,
  type HarmonyConfig,
  type HarmonyResult,
  type VoiceIn,
} from "./harmony";
import type { DspVoice } from "./dsp-protocol";
import { DETECTOR_CONFIG } from "../config/detector";
import { INTONATION_CONFIG } from "../config/intonation";
import {
  SPEC_SRC_FLOOR_DB,
  SPEC_SRC_CEIL_DB,
  SPEC_MAGNITUDE_EPSILON as EPS,
} from "../config/spectrogram-encoding";

/**
 * All per-frame DSP for the live spectrogram, in one place so it can run
 * off the main thread inside a Worker (see dsp-worker.ts) and be
 * unit-tested without one.
 *
 * Previously the STFT, note detection, and dB/colormap conversion all
 * ran synchronously on the UI thread at audio-callback cadence (~86 fps),
 * which backed up the worklet message queue and made the waterfall lag
 * behind live audio. This class owns that work; the renderer is left
 * with only a byte→RGB LUT lookup and a blit.
 *
 * Output is compact: each frame is a u8 magnitude column (one byte per
 * FFT bin, quantized over [SPEC_SRC_FLOOR_DB, SPEC_SRC_CEIL_DB]) plus the
 * set of active MIDI notes. The full-precision Float32 magnitudes are
 * used for note detection before they are quantized, so detection
 * accuracy is unaffected by the wire encoding.
 */

export interface DspFrame {
  /** One quantized magnitude byte per FFT bin (length === numBins). */
  bytes: Uint8Array;
  /** MIDI note numbers sounding in this frame. */
  activeMidi: number[];
  /** Single best fundamental (QIFFT) for the intonation trail, or null if unvoiced. */
  f0Hz: number | null;
  /** 0..1 voicing confidence of f0Hz. */
  f0Confidence: number;
}

export interface DspLevel {
  peakDb: number;
  rmsDb: number;
}

export interface DspHarmony {
  voices: DspVoice[];
  result: HarmonyResult;
}

export interface DspResult {
  frames: DspFrame[];
  /** Non-null at most once per ~levelIntervalMs of accumulated samples. */
  level: DspLevel | null;
  /** One per detection frame that landed in this chunk (usually 0 or 1). */
  harmony: DspHarmony[];
}

export interface SpectrogramDspOptions {
  fftSize: number;
  hopSize: number;
  sampleRate: number;
  minFreqHz: number;
  maxFreqHz: number;
  /** Level-meter cadence in ms. Default 50. */
  levelIntervalMs?: number;
}

const SRC_SPAN = SPEC_SRC_CEIL_DB - SPEC_SRC_FLOOR_DB;
/** Upper bound of sung fundamentals for the rehearsal view (just above C6 = 1046.5 Hz). */
const HARMONY_MAX_HZ = 1150;

export class SpectrogramDsp {
  readonly sampleRate: number;

  private stft: StreamingStft;
  /** Higher-resolution STFT that feeds the detector, decoupled from display. */
  private detStft: StreamingStft;
  private detector: NoteDetector;
  /**
   * Second detector for the rehearsal view, bound to the sung-fundamental
   * range (≤ C6). Harmonic scoring still reaches Nyquist; only the CANDIDATE
   * notes are limited, so the iterative canceller spends its polyphony budget
   * on fundamentals instead of the needle-sharp upper partials a very steady
   * voice produces (which the display detector, ranging to 8 kHz, reports as
   * high notes).
   */
  private harmonyDetector: NoteDetector;
  /** Latest active-note set from the detector, stamped onto display frames. */
  private lastActiveMidi: number[] = [];
  private minFreqHz: number;
  private maxFreqHz: number;

  /** Rehearsal analysis (chord, targets, drift, cards). */
  readonly harmony = new HarmonyAnalyzer();
  private familyMarginDb: number = DETECTOR_CONFIG.familyMarginDb;
  /** Reused dB copy of the latest detection frame (scatter measure). */
  private detDb: Float32Array = new Float32Array(0);
  /** Wall-clock source for card hold / drift timing (injectable for tests). */
  now: () => number = () =>
    typeof performance !== "undefined" ? performance.now() : Date.now();

  private levelPeak = 0;
  private levelSumSquares = 0;
  private levelSampleCount = 0;
  private readonly samplesPerLevelUpdate: number;

  constructor(opts: SpectrogramDspOptions) {
    this.sampleRate = opts.sampleRate;
    this.minFreqHz = opts.minFreqHz;
    this.maxFreqHz = opts.maxFreqHz;
    this.stft = new StreamingStft(opts.fftSize, opts.hopSize);
    // Detection runs on its own, higher-resolution STFT (decoupled from the
    // display STFT) so bass semitones resolve — see DETECTOR_CONFIG.fftSize.
    this.detStft = new StreamingStft(
      DETECTOR_CONFIG.fftSize,
      DETECTOR_CONFIG.fftSize / 4,
    );
    this.detector = this.buildDetector();
    this.harmonyDetector = this.buildDetector(HARMONY_MAX_HZ);
    this.samplesPerLevelUpdate = Math.max(
      1,
      Math.round((opts.sampleRate * (opts.levelIntervalMs ?? 50)) / 1000),
    );
  }

  get numBins(): number {
    return this.stft.numBins;
  }

  /** Rebuild the note detector for a new frequency range (sidebar change). */
  setFreqRange(minFreqHz: number, maxFreqHz: number): void {
    if (minFreqHz === this.minFreqHz && maxFreqHz === this.maxFreqHz) return;
    this.minFreqHz = minFreqHz;
    this.maxFreqHz = maxFreqHz;
    this.detector = this.buildDetector();
    this.harmonyDetector = this.buildDetector(HARMONY_MAX_HZ);
  }

  /**
   * Swap the DISPLAY STFT framing (sidebar fftSize/hopSize change). Detection
   * keeps its own fixed framing, so the detector is untouched here.
   */
  setFraming(fftSize: number, hopSize: number): void {
    if (fftSize === this.stft.fftSize && hopSize === this.stft.hopSize) return;
    this.stft = new StreamingStft(fftSize, hopSize);
  }

  /** Rehearsal-view settings; a familyMarginDb change rebuilds the detector. */
  setHarmony(
    partial: Partial<Omit<HarmonyConfig, "sections">> & {
      sections?: Partial<Record<"B" | "T" | "A" | "S", boolean>>;
      familyMarginDb?: number;
      resetDrift?: boolean;
    },
  ): void {
    const { familyMarginDb, resetDrift, ...cfg } = partial;
    this.harmony.setConfig(cfg);
    if (resetDrift) this.harmony.resetDrift();
    if (familyMarginDb != null && familyMarginDb !== this.familyMarginDb) {
      this.familyMarginDb = familyMarginDb;
      this.detector = this.buildDetector();
      this.harmonyDetector = this.buildDetector(HARMONY_MAX_HZ);
    }
  }

  /** Drop streaming + detection state (new session or framing change). */
  reset(): void {
    this.stft.reset();
    this.detStft.reset();
    this.detector.reset();
    this.harmonyDetector.reset();
    this.harmony.reset();
    this.lastActiveMidi = [];
    this.levelPeak = 0;
    this.levelSumSquares = 0;
    this.levelSampleCount = 0;
  }

  pushSamples(chunk: Float32Array): DspResult {
    const level = this.accumulateLevel(chunk);

    // Detection path: its own higher-resolution STFT, which emits on a coarser
    // cadence than display. Run it first so display frames in this chunk carry
    // the freshest active-note set.
    const harmony: DspHarmony[] = [];
    for (const dmags of this.detStft.pushSamples(chunk)) {
      this.lastActiveMidi = Array.from(this.detector.analyze(dmags));
      harmony.push(
        this.analyzeHarmony(dmags, this.harmonyDetector.analyze(dmags)),
      );
    }

    // Display path: the configured STFT produces the quantized columns. Each
    // carries the latest detection result.
    const magFrames = this.stft.pushSamples(chunk);
    const frames: DspFrame[] = [];
    for (const mags of magFrames) {
      // Single-F0 (QIFFT) for the intonation trail, off the full-precision
      // display magnitudes before they are quantized.
      const f0 = estimateF0(mags, this.sampleRate, this.stft.fftSize, {
        minF0Hz: INTONATION_CONFIG.minF0Hz,
        maxF0Hz: INTONATION_CONFIG.maxF0Hz,
        voicedRatio: INTONATION_CONFIG.voicedRatio,
      });
      frames.push({
        bytes: quantize(mags),
        activeMidi: this.lastActiveMidi,
        f0Hz: f0 ? f0.frequencyHz : null,
        f0Confidence: f0 ? f0.confidence : 0,
      });
    }
    return { frames, level, harmony };
  }

  /**
   * Voices for the rehearsal view: each accepted note gets a sub-bin pitch
   * (parabola through the peak of its fundamental band on the detection
   * spectrum, log magnitudes), the detector's confidence, and a section label
   * from the pitch-range DP. Then the HarmonyAnalyzer turns that into chords,
   * targets, pairs, drift and scatter.
   */
  private analyzeHarmony(dmags: Float32Array, active: Set<number>): DspHarmony {
    const binHz = this.sampleRate / DETECTOR_CONFIG.fftSize;
    const conf = this.harmonyDetector.lastConfidences();
    const labels = labelSections(active, { confidence: conf });
    const voices: DspVoice[] = [];
    for (const lab of labels) {
      const range = this.harmonyDetector.binRangeForMidi(lab.midi);
      if (!range) continue;
      const [lo, hi] = range;
      let pk = lo;
      for (let b = lo; b <= hi; b++)
        if ((dmags[b] as number) > (dmags[pk] as number)) pk = b;
      let f0Hz = pk * binHz;
      if (pk > 0 && pk < dmags.length - 1) {
        const l = Math.log((dmags[pk - 1] as number) + EPS);
        const c = Math.log((dmags[pk] as number) + EPS);
        const r = Math.log((dmags[pk + 1] as number) + EPS);
        f0Hz = (pk + parabolicPeak(l, c, r)) * binHz;
      }
      voices.push({
        midi: lab.midi,
        f0Hz,
        salience: conf.get(lab.midi) ?? 1,
        section: lab.guess,
        label: lab.label,
        sectionConf: lab.confidence,
      });
    }
    // dB copy of the detection frame + a floor estimate for the scatter measure.
    if (this.detDb.length !== dmags.length)
      this.detDb = new Float32Array(dmags.length);
    const db = this.detDb;
    const sample: number[] = [];
    for (let b = 0; b < dmags.length; b++) {
      db[b] = 20 * Math.log10((dmags[b] as number) + EPS);
      if (b % 8 === 0) sample.push(db[b] as number);
    }
    sample.sort((a, b) => a - b);
    const floorDb = sample[sample.length >> 1] ?? -120;
    const input: VoiceIn[] = voices.map((v) => ({ ...v }));
    const result = this.harmony.analyze(input, db, binHz, floorDb, this.now());
    return { voices, result };
  }

  private buildDetector(maxFreqCap?: number): NoteDetector {
    return new NoteDetector({
      // Bound to the DETECTION STFT, not the display one.
      numBins: this.detStft.numBins,
      sampleRate: this.sampleRate,
      minFreqHz: this.minFreqHz,
      maxFreqHz:
        maxFreqCap != null
          ? Math.min(this.maxFreqHz, maxFreqCap)
          : this.maxFreqHz,
      harmonic: DETECTOR_CONFIG.harmonic,
      harmonicCount: DETECTOR_CONFIG.harmonicCount,
      salienceThreshold: DETECTOR_CONFIG.salienceThreshold,
      decayPerFrame: DETECTOR_CONFIG.decayPerFrame,
      familyMarginDb: this.familyMarginDb,
      // whiteningWindowHz + mergeRadius default from DETECTOR_CONFIG, scaled to
      // the detection STFT's bin spacing inside NoteDetector.
    });
  }

  /** Returns a level reading when a ~levelIntervalMs window has filled. */
  private accumulateLevel(chunk: Float32Array): DspLevel | null {
    for (let i = 0; i < chunk.length; i++) {
      const v = chunk[i] as number;
      const abs = v < 0 ? -v : v;
      if (abs > this.levelPeak) this.levelPeak = abs;
      this.levelSumSquares += v * v;
    }
    this.levelSampleCount += chunk.length;
    if (this.levelSampleCount < this.samplesPerLevelUpdate) return null;

    const rms = Math.sqrt(this.levelSumSquares / this.levelSampleCount);
    const level: DspLevel = {
      peakDb: 20 * Math.log10(this.levelPeak + EPS),
      rmsDb: 20 * Math.log10(rms + EPS),
    };
    this.levelPeak = 0;
    this.levelSumSquares = 0;
    this.levelSampleCount = 0;
    return level;
  }
}

/** Quantize a magnitude spectrum to one u8 per bin over the wire dB range. */
export function quantize(mags: Float32Array): Uint8Array {
  const bytes = new Uint8Array(mags.length);
  for (let b = 0; b < mags.length; b++) {
    const db = 20 * Math.log10((mags[b] as number) + EPS);
    let t = (db - SPEC_SRC_FLOOR_DB) / SRC_SPAN;
    t = t < 0 ? 0 : t > 1 ? 1 : t;
    bytes[b] = (t * 255 + 0.5) | 0;
  }
  return bytes;
}
