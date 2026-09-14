import { DETECTOR_CONFIG } from "../config/detector";

export interface NoteDetectorOptions {
  /** Number of FFT magnitude bins (== fftSize / 2). */
  numBins: number;
  /** Audio sample rate, Hz. */
  sampleRate: number;
  /** Lowest frequency to consider, Hz. */
  minFreqHz: number;
  /** Highest frequency to consider, Hz. */
  maxFreqHz: number;
  /** Per-frame multiplicative decay applied to each note's energy. Defaults to DETECTOR_CONFIG.decayPerFrame. */
  decayPerFrame?: number;
  /** Threshold in dBFS above which a note is reported active. Default -50. */
  thresholdDb?: number;
  /**
   * Use the harmonic-aware path: whiten the spectrum, score each note by
   * summing energy across its harmonics, and suppress octave phantoms.
   * Default false (legacy ±quarter-tone band detector). This is a mode
   * contract, not a tuning knob, so it is NOT sourced from DETECTOR_CONFIG —
   * the production path opts in explicitly via DETECTOR_CONFIG.harmonic.
   */
  harmonic?: boolean;
  /** Harmonics summed per candidate (K), including the fundamental. Defaults to DETECTOR_CONFIG.harmonicCount. */
  harmonicCount?: number;
  /** Unitless salience floor for the harmonic path. Defaults to DETECTOR_CONFIG.salienceThreshold. */
  salienceThreshold?: number;
  /**
   * Moving-average window (FFT bins) for the whitening envelope. If omitted,
   * defaults to DETECTOR_CONFIG.whiteningWindowHz converted to bins at this
   * detector's bin spacing — so the window stays ~constant in Hz across
   * fftSizes. Pass an explicit bin count only to override that.
   */
  whiteningWindowBins?: number;
  /**
   * Whitening envelope width in Hz. Converted to bins at this detector's bin
   * spacing; resolution-independent. Ignored if whiteningWindowBins is set.
   * Defaults to DETECTOR_CONFIG.whiteningWindowHz.
   */
  whiteningWindowHz?: number;

  /**
   * Collapse a contiguous run of active semitones (consecutive MIDI numbers
   * with gap ≤ this) to the single highest-salience note. Folds the
   * FFT-resolution smear of a chorus-spread note back into one note.
   * Harmonic path only. Defaults to DETECTOR_CONFIG.mergeRadius; 0 disables.
   * Tradeoff: a GENUINE semitone cluster (e.g. C4+C#4) also collapses — the
   * FFT can't reliably resolve those anyway, so this is an accepted limit.
   */
  mergeRadius?: number;
  /** Max simultaneous F0s the iterative canceller extracts per frame. Defaults to DETECTOR_CONFIG.maxPolyphony. */
  maxPolyphony?: number;
  /** Fraction of the modeled (smoothed-envelope) amplitude subtracted per cancellation, (0,1]. Defaults to DETECTOR_CONFIG.cancelFactor. */
  cancelFactor?: number;
  /**
   * Harmonics cancelled per ACCEPTED note (Kc). May exceed harmonicCount:
   * scoring only needs the perceptually dominant first K harmonics, but
   * cancellation must scrub the note's full harmonic series or its upper
   * partials get re-detected as phantom high notes. Like `harmonic`, this is
   * an algorithm-shape parameter with a local default (16), not a
   * DETECTOR_CONFIG knob.
   */
  cancelHarmonicCount?: number;
  /**
   * Headroom multiplier on the neighbour-envelope model during cancellation.
   * Values >1 let the subtraction fully swallow a harmonic that pokes
   * modestly above its neighbours (natural formant/rolloff variation), while
   * a genuine octave partner — which adds ~equal POWER, lifting the bin by
   * ~√2 — still exceeds the headroom and keeps its excess. Local default;
   * not a DETECTOR_CONFIG knob.
   */
  cancelHeadroom?: number;
  /**
   * Minimum whitened peak required in a candidate's FUNDAMENTAL band (on the
   * pre-cancellation whitened spectrum, where flat = 1.0) for the candidate
   * to be eligible at all. A sung note always carries fundamental energy —
   * even an octave partner whose f0 coincides with a lower voice's harmonic.
   * A subharmonic ghost (C2 "under" a real C3) has a physically EMPTY
   * fundamental band and harvests only cancellation crumbs; this gate kills
   * that class structurally. 0 disables. Local default; not a
   * DETECTOR_CONFIG knob.
   */
  fundamentalFloor?: number;
  /**
   * Harmonic-family test (Analyzer merge). When a candidate sits at an integer
   * multiple k of an already-accepted note, it survives only if its OWN
   * partial series (slots k, 2k, 3k of the parent) sits at least this many dB
   * above the parent's fitted harmonic envelope — at its fundamental AND on
   * average. A vocal-tract formant lifts one slot; a second singer lifts a
   * whole series. Replaces the amplitude-only heuristics that let a chest
   * voice's H2/H3 register as tenor/alto notes. 0 disables. Default 10.
   */
  familyMarginDb?: number;
  /**
   * Centre frequency of each spectrum bin, ascending. Supply for a CQT (or any
   * non-uniform) front end: bins map to harmonic ranges via this array instead
   * of the linear `b·binFreqHz`. When set, `numBins` must equal its length.
   * Omit for a uniform STFT spectrum (the default).
   */
  binFreqs?: Float32Array;
}

/**
 * Detect which MIDI notes have significant energy in a magnitude
 * spectrum. Two paths share one class:
 *
 * - **Legacy (default):** each note covers the FFT bins inside its
 *   ±quarter-tone band; per frame, the maximum magnitude across that band
 *   is decay-smoothed and compared against a dBFS threshold. Cheap and
 *   calibrated, but reports a voice's overtones as independent notes.
 *
 * - **Harmonic (opt-in via `harmonic: true`):** the spectrum is whitened
 *   (divided by a local moving-average envelope, which flattens the
 *   singer's formant), then notes are extracted by ITERATIVE HARMONIC
 *   CANCELLATION: repeatedly pick the strongest F0 (a weighted sum of the
 *   peak whitened energy across its first K harmonics, normalized so a flat
 *   spectrum scores 1.0), accept it, and subtract its explained harmonic
 *   energy from the residual so the notes/phantoms it accounts for stop
 *   competing. A lone overtone or a fifth/twelfth/octave phantom collapses
 *   once its parent note is cancelled; a real independent note survives.
 *   Harmonic lookups reach up to Nyquist, so candidates above the render
 *   ceiling still get full support.
 *
 * One instance is bound to a specific (numBins, sampleRate, freq range)
 * configuration. Build a new instance if any of those change — there
 * is no resize path.
 *
 * The decay-then-max-merge model gives notes a natural envelope shape:
 * fast attack (peak immediately overrides decayed history), exponential
 * release (silence decays at decayPerFrame per frame). This prevents
 * flicker when audio level grazes the threshold.
 */
export class NoteDetector {
  readonly midiMin: number;
  readonly midiMax: number;
  readonly noteCount: number;
  readonly decayPerFrame: number;
  readonly thresholdLinear: number;

  readonly harmonic: boolean;
  readonly harmonicCount: number;
  readonly salienceThreshold: number;
  readonly whiteningWindowBins: number;
  readonly mergeRadius: number;
  readonly maxPolyphony: number;
  readonly cancelFactor: number;
  readonly cancelHarmonicCount: number;
  readonly cancelHeadroom: number;
  readonly fundamentalFloor: number;
  readonly familyMarginDb: number;

  private readonly numBins: number;
  /** Packed pairs: bin range for MIDI (midiMin + i) lives at indices 2i, 2i+1. */
  private readonly binRanges: Int32Array;
  /** Decay-smoothed energy (legacy) or salience (harmonic) per note, indexed by midi - midiMin. */
  private readonly energy: Float32Array;
  /** Confidence (smoothed salience / threshold) of each emitted note, from the
   *  last analyze(). Reused per frame; valid until the next analyze() call. */
  private readonly lastConf = new Map<number, number>();

  // Harmonic-path state. Empty arrays in legacy mode.
  /** Row stride of harmonicBins: max(harmonicCount, cancelHarmonicCount). */
  private readonly binStride: number;
  /** Packed [lo,hi] per (note, harmonic): note i, harmonic k at ((i*binStride + k)*2). hi<lo means "skip". */
  private readonly harmonicBins: Int32Array;
  /** w(k) = 1/(k+1) for k in [0, K). */
  private readonly harmonicWeights: Float32Array;
  /** Per-note 1/Σw over present harmonics, so a flat whitened spectrum scores 1.0. */
  private readonly noteWeightNorm: Float32Array;
  /** Reused whitened-spectrum buffer. */
  private readonly whitened: Float32Array;
  /** Reused prefix-sum scratch for the box-average envelope. */
  private readonly prefix: Float64Array;
  /** Reused residual whitened spectrum for iterative cancellation. */
  private readonly residual: Float32Array;
  /** Reused per-note salience-at-acceptance buffer (this frame). */
  private readonly frameSalience: Float32Array;
  /** Reused per-harmonic amplitude scratch for spectral-smoothness cancellation. */
  private readonly cancelAmps: Float32Array;

  constructor(opts: NoteDetectorOptions) {
    const {
      numBins,
      sampleRate,
      minFreqHz,
      maxFreqHz,
      decayPerFrame = DETECTOR_CONFIG.decayPerFrame,
      thresholdDb = -50,
      harmonic = false,
      harmonicCount = DETECTOR_CONFIG.harmonicCount,
      salienceThreshold = DETECTOR_CONFIG.salienceThreshold,
      whiteningWindowBins,
      whiteningWindowHz = DETECTOR_CONFIG.whiteningWindowHz,
      mergeRadius = DETECTOR_CONFIG.mergeRadius,
      maxPolyphony = DETECTOR_CONFIG.maxPolyphony,
      cancelFactor = DETECTOR_CONFIG.cancelFactor,
      cancelHarmonicCount = 16,
      cancelHeadroom = 1.0,
      fundamentalFloor = 1.0,
      familyMarginDb = DETECTOR_CONFIG.familyMarginDb,
      binFreqs,
    } = opts;

    if (!Number.isInteger(numBins) || numBins <= 0) {
      throw new Error(
        `NoteDetector: numBins must be a positive integer, got ${numBins}`,
      );
    }
    if (!Number.isFinite(sampleRate) || sampleRate <= 0) {
      throw new Error(
        `NoteDetector: sampleRate must be positive and finite, got ${sampleRate}`,
      );
    }
    if (!(minFreqHz > 0) || !(maxFreqHz > minFreqHz)) {
      throw new Error(
        `NoteDetector: require 0 < minFreqHz < maxFreqHz, got min=${minFreqHz} max=${maxFreqHz}`,
      );
    }
    if (!(decayPerFrame > 0) || !(decayPerFrame < 1)) {
      throw new Error(
        `NoteDetector: decayPerFrame must be in (0, 1), got ${decayPerFrame}`,
      );
    }
    if (!Number.isInteger(mergeRadius) || mergeRadius < 0) {
      throw new Error(
        `NoteDetector: mergeRadius must be a non-negative integer, got ${mergeRadius}`,
      );
    }
    if (binFreqs && whiteningWindowBins === undefined) {
      throw new Error(
        "NoteDetector: whiteningWindowBins must be explicitly specified when using geometric binFreqs",
      );
    }

    const binFreqHz = sampleRate / (2 * numBins);
    // Whitening window: an explicit bin count overrides; otherwise convert the
    // Hz spec at this detector's bin spacing, so the window stays ~constant in
    // Hz across fftSizes (a fixed bin count silently mistunes when fftSize
    // changes — e.g. 96 bins is ~2 kHz at fft 2048 but ~0.5 kHz at fft 8192).
    const resolvedWhiteningBins =
      whiteningWindowBins ??
      Math.max(1, Math.round(whiteningWindowHz / binFreqHz));

    if (harmonic) {
      if (!Number.isInteger(harmonicCount) || harmonicCount < 1) {
        throw new Error(
          `NoteDetector: harmonicCount must be a positive integer, got ${harmonicCount}`,
        );
      }
      if (!(salienceThreshold > 0)) {
        throw new Error(
          `NoteDetector: salienceThreshold must be positive, got ${salienceThreshold}`,
        );
      }
      if (
        whiteningWindowBins !== undefined &&
        (!Number.isInteger(whiteningWindowBins) || whiteningWindowBins < 1)
      ) {
        throw new Error(
          `NoteDetector: whiteningWindowBins must be a positive integer, got ${whiteningWindowBins}`,
        );
      }

      if (!Number.isInteger(maxPolyphony) || maxPolyphony < 1) {
        throw new Error(
          `NoteDetector: maxPolyphony must be a positive integer, got ${maxPolyphony}`,
        );
      }
      if (!(cancelFactor > 0) || !(cancelFactor <= 1)) {
        throw new Error(
          `NoteDetector: cancelFactor must be in (0, 1], got ${cancelFactor}`,
        );
      }
      if (!Number.isInteger(cancelHarmonicCount) || cancelHarmonicCount < 1) {
        throw new Error(
          `NoteDetector: cancelHarmonicCount must be a positive integer, got ${cancelHarmonicCount}`,
        );
      }
      if (!(cancelHeadroom >= 1)) {
        throw new Error(
          `NoteDetector: cancelHeadroom must be >= 1, got ${cancelHeadroom}`,
        );
      }
    }

    // Bin↔frequency mapping. STFT: uniform (bin b is at b·binFreqHz). CQT (or
    // any non-uniform front end): the caller supplies each bin's centre
    // frequency, and a target Hz range maps to bins via the geometric spacing.
    const cqt = binFreqs !== undefined;
    const effNumBins = cqt ? binFreqs.length : numBins;
    const cqtFmin = cqt ? (binFreqs[0] as number) : 0;
    const cqtPerBinLog = cqt
      ? Math.log((binFreqs[1] as number) / (binFreqs[0] as number))
      : 1;
    /** Bin range [lo,hi] covering [flo,fhi] Hz; [0,-1] = out of range (skip). */
    const bandFor = (flo: number, fhi: number): readonly [number, number] => {
      if (cqt) {
        const loRaw = Math.log(flo / cqtFmin) / cqtPerBinLog;
        const hiRaw = Math.log(fhi / cqtFmin) / cqtPerBinLog;
        if (hiRaw < 0 || loRaw > effNumBins - 1) return [0, -1];
        return [
          Math.max(0, Math.round(loRaw)),
          Math.min(effNumBins - 1, Math.round(hiRaw)),
        ];
      }
      const lo = Math.floor(flo / binFreqHz);
      const hi = Math.ceil(fhi / binFreqHz);
      if (lo > effNumBins - 1 || hi < 0) return [0, -1];
      return [Math.max(0, lo), Math.min(effNumBins - 1, hi)];
    };

    this.numBins = effNumBins;
    this.midiMin = Math.max(
      0,
      Math.floor(12 * Math.log2(minFreqHz / 440) + 69),
    );
    this.midiMax = Math.min(
      127,
      Math.ceil(12 * Math.log2(maxFreqHz / 440) + 69),
    );
    this.noteCount = Math.max(0, this.midiMax - this.midiMin + 1);
    this.decayPerFrame = decayPerFrame;
    // dBFS → linear amplitude. 20·log10 because we're working with
    // amplitude magnitudes; 10·log10 would be the power form.
    this.thresholdLinear = Math.pow(10, thresholdDb / 20);

    this.harmonic = harmonic;
    this.harmonicCount = harmonicCount;
    this.salienceThreshold = salienceThreshold;
    this.whiteningWindowBins = resolvedWhiteningBins;
    this.mergeRadius = mergeRadius;
    this.maxPolyphony = maxPolyphony;
    this.cancelFactor = cancelFactor;
    this.cancelHarmonicCount = cancelHarmonicCount;
    this.cancelHeadroom = cancelHeadroom;
    this.fundamentalFloor = fundamentalFloor;
    this.familyMarginDb = familyMarginDb;
    this.binStride = Math.max(
      harmonicCount,
      harmonic ? cancelHarmonicCount : 0,
    );

    const halfStep = Math.pow(2, 1 / 24);
    this.binRanges = new Int32Array(this.noteCount * 2);
    for (let i = 0; i < this.noteCount; i++) {
      const midi = this.midiMin + i;
      const freq = 440 * Math.pow(2, (midi - 69) / 12);
      const [lo, hi] = bandFor(freq / halfStep, freq * halfStep);
      this.binRanges[i * 2] = lo;
      this.binRanges[i * 2 + 1] = hi;
    }
    this.energy = new Float32Array(this.noteCount);

    if (harmonic) {
      const K = harmonicCount;
      const stride = this.binStride;
      this.harmonicWeights = new Float32Array(K);
      for (let k = 0; k < K; k++) this.harmonicWeights[k] = 1 / (k + 1);

      // Per (note, harmonic) ±quarter-tone band around k·f0, out to the
      // LARGER of the scoring/cancellation harmonic counts. Harmonics above
      // the spectrum's top frequency get a hi<lo sentinel so their scan is a
      // no-op. (In CQT space these bands sit at fixed bin offsets per octave.)
      this.harmonicBins = new Int32Array(this.noteCount * stride * 2);
      for (let i = 0; i < this.noteCount; i++) {
        const midi = this.midiMin + i;
        const f0 = 440 * Math.pow(2, (midi - 69) / 12);
        for (let k = 1; k <= stride; k++) {
          const fk = k * f0;
          const [lo, hi] = bandFor(fk / halfStep, fk * halfStep);
          const idx = (i * stride + (k - 1)) * 2;
          this.harmonicBins[idx] = lo;
          this.harmonicBins[idx + 1] = hi;
        }
      }
      // Per-note normalization over the SCORING harmonics that actually fit
      // below Nyquist. With it, a flat (featureless) whitened region scores
      // exactly 1.0 for every note regardless of how many harmonics fit,
      // so salienceThreshold reads as a multiple above the flat baseline.
      this.noteWeightNorm = new Float32Array(this.noteCount);
      for (let i = 0; i < this.noteCount; i++) {
        let sumW = 0;
        for (let k = 0; k < K; k++) {
          const idx = (i * stride + k) * 2;
          if (
            (this.harmonicBins[idx + 1] as number) >=
            (this.harmonicBins[idx] as number)
          ) {
            sumW += this.harmonicWeights[k] as number;
          }
        }
        this.noteWeightNorm[i] = sumW > 0 ? 1 / sumW : 0;
      }

      this.whitened = new Float32Array(effNumBins);
      this.prefix = new Float64Array(effNumBins + 1);
      this.residual = new Float32Array(effNumBins);
      this.frameSalience = new Float32Array(this.noteCount);
      this.cancelAmps = new Float32Array(stride);
    } else {
      this.harmonicWeights = new Float32Array(0);
      this.noteWeightNorm = new Float32Array(0);
      this.harmonicBins = new Int32Array(0);
      this.whitened = new Float32Array(0);
      this.prefix = new Float64Array(0);
      this.residual = new Float32Array(0);
      this.frameSalience = new Float32Array(0);
      this.cancelAmps = new Float32Array(0);
    }
  }

  /**
   * Analyze one magnitude frame and return the set of MIDI numbers
   * reported active. Dispatches to the harmonic or legacy path.
   *
   * Mutates internal decay state. A given (NoteDetector, magnitudes)
   * pair is NOT idempotent — calling analyze twice with the same input
   * yields different results because the second call sees the decayed
   * energy from the first.
   */
  analyze(magnitudes: Float32Array): Set<number> {
    if (magnitudes.length < this.numBins) {
      throw new Error(
        `NoteDetector.analyze: magnitude frame length (${magnitudes.length}) is smaller than expected numBins (${this.numBins})`,
      );
    }
    if (this.harmonic) return this.analyzeHarmonic(magnitudes);

    const active = new Set<number>();
    const decay = this.decayPerFrame;
    const threshold = this.thresholdLinear;

    for (let i = 0; i < this.noteCount; i++) {
      const lo = this.binRanges[i * 2] as number;
      const hi = this.binRanges[i * 2 + 1] as number;
      let peak = 0;
      for (let b = lo; b <= hi; b++) {
        const m = magnitudes[b] as number;
        if (m > peak) peak = m;
      }
      const decayed = (this.energy[i] as number) * decay;
      const next = peak > decayed ? peak : decayed;
      this.energy[i] = next;
      if (next > threshold) active.add(this.midiMin + i);
    }
    return active;
  }

  /** Clear decay state. Use on capture restart or after a long silence. */
  reset(): void {
    this.energy.fill(0);
  }

  /** Per-MIDI fundamental bin range, exposed for testing and debugging. */
  binRangeForMidi(midi: number): readonly [number, number] | null {
    if (midi < this.midiMin || midi > this.midiMax) return null;
    const i = midi - this.midiMin;
    return [
      this.binRanges[i * 2] as number,
      this.binRanges[i * 2 + 1] as number,
    ];
  }

  /**
   * Per-MIDI harmonic bin ranges (one [lo,hi] per harmonic, fundamental
   * first), exposed for testing. Null in legacy mode or out of range.
   * A pair with hi < lo denotes a harmonic at/above Nyquist (no data).
   */
  harmonicBinsForMidi(
    midi: number,
  ): ReadonlyArray<readonly [number, number]> | null {
    if (!this.harmonic) return null;
    if (midi < this.midiMin || midi > this.midiMax) return null;
    const i = midi - this.midiMin;
    const K = this.harmonicCount;
    const stride = this.binStride;
    const out: Array<readonly [number, number]> = [];
    for (let k = 0; k < K; k++) {
      const idx = (i * stride + k) * 2;
      out.push([
        this.harmonicBins[idx] as number,
        this.harmonicBins[idx + 1] as number,
      ]);
    }
    return out;
  }

  /**
   * Whiten `mags` into `this.whitened`: divide each bin by a local
   * box-average envelope (width whiteningWindowBins), flattening broad
   * features like the singer's formant. Bins below the dBFS noise gate
   * (thresholdLinear) contribute zero, so silence and noise can't
   * normalize up to ~1 and trip the harmonic sum.
   */
  private whiten(mags: Float32Array): void {
    const n = this.numBins;
    const prefix = this.prefix;
    const whitened = this.whitened;
    const gate = this.thresholdLinear;
    const half = this.whiteningWindowBins >> 1;

    prefix[0] = 0;
    let total = 0;
    for (let b = 0; b < n; b++) {
      total += mags[b] as number;
      prefix[b + 1] = total;
    }

    for (let b = 0; b < n; b++) {
      const lo = b - half < 0 ? 0 : b - half;
      const hi = b + half >= n ? n - 1 : b + half;
      const sum = (prefix[hi + 1] as number) - (prefix[lo] as number);
      const env = sum / (hi - lo + 1);
      const m = mags[b] as number;
      whitened[b] = m >= gate ? m / (env > 1e-12 ? env : 1e-12) : 0;
    }
  }

  /** Raw magnitude frame of the current analyze() call (family test reads levels here, not whitened). */
  private rawMags: Float32Array = new Float32Array(0);

  private analyzeHarmonic(magnitudes: Float32Array): Set<number> {
    this.rawMags = magnitudes;
    this.whiten(magnitudes);
    const n = this.noteCount;
    const threshold = this.salienceThreshold;

    // Iterative harmonic cancellation. Repeatedly pick the strongest F0 on the
    // RESIDUAL whitened spectrum, accept it, and subtract its explained
    // harmonic energy so whatever it accounts for stops competing. A real note
    // keeps independent harmonic support across rounds and survives; a phantom
    // (a fifth / twelfth / octave whose energy belonged to a stronger note)
    // collapses once that note is cancelled — which is the principled
    // replacement for the old octave/subharmonic suppression heuristics.
    const residual = this.residual;
    residual.set(this.whitened);
    const frameSalience = this.frameSalience;
    frameSalience.fill(0);

    const gamma = this.cancelFactor;
    const fFloor = this.fundamentalFloor;
    const familyMargin = this.familyMarginDb;
    /** Accepted notes this frame with their fitted harmonic envelopes (dB vs log2 k). */
    const accepted: Array<{
      i: number;
      parts: Array<number | null>;
      fits: Map<number, (k: number) => number>;
    }> = [];
    /** Candidates rejected by the family test this frame (explained by a parent). */
    const explained = new Uint8Array(n);
    for (let iter = 0; iter < this.maxPolyphony; iter++) {
      let bestI = -1;
      let bestSal = threshold; // must strictly exceed the floor to be picked
      for (let i = 0; i < n; i++) {
        if ((frameSalience[i] as number) > 0) continue; // already accepted
        if (explained[i]) continue;
        // Fundamental-support gate: a candidate whose f0 band holds no
        // energy in the PRE-cancellation whitened spectrum cannot be a sung
        // note — only a subharmonic ghost fed by cancellation crumbs.
        // Gating on the original (not residual) spectrum keeps genuine
        // octave partners eligible even after their shared fundamental bin
        // was partially cancelled.
        if (fFloor > 0 && !this.hasFundamentalSupport(i, fFloor)) continue;
        const sal = this.salienceOf(i, residual);
        if (sal > bestSal) {
          // Family test: is this candidate just a partial of a note already
          // accepted this frame? Evaluated on the ORIGINAL whitened spectrum.
          if (
            familyMargin > 0 &&
            this.explainedByParent(i, accepted, familyMargin)
          ) {
            explained[i] = 1;
            // Its energy belongs to the parent: scrub it from the residual so it
            // cannot feed further phantoms, but do not report it.
            this.cancel(i, residual, gamma);
            continue;
          }
          bestSal = sal;
          bestI = i;
        }
      }
      if (bestI < 0) break;
      frameSalience[bestI] = bestSal;
      accepted.push({
        i: bestI,
        parts: this.partialLevels(bestI),
        fits: new Map(),
      });
      this.cancel(bestI, residual, gamma);
    }

    // Family resolution pass. Acceptance order is by salience, and a chest
    // voice's H2/H3 can out-score its own fundamental, so the octave may be
    // accepted BEFORE the parent and never get tested against it. Re-test every
    // accepted note, ascending, against the accepted notes below it; a note
    // explained by a lower parent is retracted (its energy was already
    // cancelled, so nothing else is disturbed).
    if (familyMargin > 0 && accepted.length > 1) {
      const asc = accepted.slice().sort((a, b) => a.i - b.i);
      const kept: typeof accepted = [];
      for (const a of asc) {
        if (kept.length && this.explainedByParent(a.i, kept, familyMargin)) {
          frameSalience[a.i] = 0;
          continue;
        }
        kept.push(a);
      }
    }

    // Temporal smoothing (decay-then-max) on the accepted salience, then emit +
    // merge. A note accepted this frame refreshes its energy; others release.
    const energy = this.energy;
    const decay = this.decayPerFrame;
    const active = new Set<number>();
    for (let i = 0; i < n; i++) {
      const s = frameSalience[i] as number;
      const decayed = (energy[i] as number) * decay;
      const e = s > decayed ? s : decayed;
      energy[i] = e;
      if (e > threshold) active.add(this.midiMin + i);
    }
    const merged = this.mergeRadius > 0 ? this.mergeAdjacent(active) : active;
    this.lastConf.clear();
    for (const midi of merged) {
      this.lastConf.set(
        midi,
        (energy[midi - this.midiMin] as number) / threshold,
      );
    }
    return merged;
  }

  /**
   * Per-note confidence for the notes emitted by the last analyze(): smoothed
   * salience normalized by the threshold, so 1.0 = just-active and higher = more
   * certain. The downstream section labeler reasons over these soft values
   * (e.g. to break A/T ties) rather than a flat binary set. Valid until the
   * next analyze(); do not retain across frames.
   */
  lastConfidences(): ReadonlyMap<number, number> {
    return this.lastConf;
  }

  /**
   * Peak level (dB) of note i's k-th harmonic band on the RAW magnitude
   * spectrum (0 dB = full-scale sine), or null if the band is above Nyquist / empty.
   */
  private partialDb(i: number, k: number): number | null {
    if (k < 1 || k > this.binStride) return null;
    const base = i * this.binStride * 2;
    const lo = this.harmonicBins[base + (k - 1) * 2] as number;
    const hi = this.harmonicBins[base + (k - 1) * 2 + 1] as number;
    if (hi < lo) return null;
    const src =
      this.rawMags.length >= this.numBins ? this.rawMags : this.whitened;
    let peak = 0;
    for (let b = lo; b <= hi; b++) {
      const v = src[b] as number;
      if (v > peak) peak = v;
    }
    if (peak <= 0) return null;
    return 20 * Math.log10(peak);
  }

  /** Raw-spectrum level (dB) of note i's partials k = 1..cancelHarmonicCount (null where absent). */
  private partialLevels(i: number): Array<number | null> {
    const out: Array<number | null> = [];
    const kMax = Math.min(this.cancelHarmonicCount, this.binStride);
    for (let k = 1; k <= kMax; k++) {
      const d = this.partialDb(i, k);
      out.push(d != null && d > -90 ? d : null);
    }
    return out;
  }

  /**
   * Fit a parent's partial levels as a line in dB against log2(k), EXCLUDING
   * the slots that are multiples of `k0` (the slots a candidate at k0 would
   * occupy). Points well above the trend are foreign (other voices only ever
   * add energy): drop them and refit, two passes. Slope clamped ≤ 0.
   */
  private fitEnvelopeExcluding(
    parts: ReadonlyArray<number | null>,
    k0: number,
  ): (k: number) => number {
    let pts: Array<[number, number]> = [];
    for (let idx = 0; idx < parts.length; idx++) {
      const k = idx + 1;
      const d = parts[idx];
      if (d == null || k % k0 === 0) continue;
      pts.push([Math.log2(k), d]);
    }
    const fit = (
      P: Array<[number, number]>,
    ): { slope: number; icpt: number } => {
      const m = P.length;
      if (m === 0) return { slope: -6, icpt: -60 };
      if (m === 1) return { slope: -6, icpt: (P[0] as [number, number])[1] };
      let sx = 0;
      let sy = 0;
      let sxx = 0;
      let sxy = 0;
      for (const [x, y] of P) {
        sx += x;
        sy += y;
        sxx += x * x;
        sxy += x * y;
      }
      const den = m * sxx - sx * sx;
      const slope = den !== 0 ? (m * sxy - sx * sy) / den : -6;
      const sl = Math.min(0, slope);
      return { slope: sl, icpt: (sy - sl * sx) / m };
    };
    let f = fit(pts);
    for (let pass = 0; pass < 2 && pts.length > 2; pass++) {
      const keep = pts.filter(([x, y]) => y - (f.icpt + f.slope * x) <= 4);
      if (keep.length === pts.length || keep.length < 2) break;
      pts = keep;
      f = fit(pts);
    }
    const { slope, icpt } = f;
    return (k: number) => icpt + slope * Math.log2(k);
  }

  /**
   * Prior harmonic envelope for a sung voice, dB relative to its fundamental
   * (a generous upper bound, not a mean): open-vowel chest voices routinely put
   * H2 up to ~6 dB and H3 up to ~3 dB ABOVE the fundamental, and the series
   * falls away above that. A candidate sitting on a parent's k-th harmonic has
   * to beat this bound by the margin to count as a second voice. A fitted
   * envelope was tried first and rejected: a robust fit drops the parent's own
   * strong partials as "foreign", which is exactly backwards for chest voice.
   */
  private static priorDb(k: number): number {
    if (k === 2) return 6;
    if (k === 3) return 3;
    return -Infinity; // above H3 the fitted envelope is the better model
  }

  /**
   * Family test. If candidate i sits at an integer multiple k (2..12, within a
   * quarter tone) of an accepted note, compare the candidate's OWN first three
   * partials — which are the parent's slots k, 2k, 3k — against the parent's
   * fitted envelope. The candidate is an independent voice only if its
   * fundamental AND its series average sit ≥ margin dB above that envelope;
   * otherwise it is a partial of the parent and is rejected.
   */
  private explainedByParent(
    i: number,
    accepted: ReadonlyArray<{
      i: number;
      parts: Array<number | null>;
      fits: Map<number, (k: number) => number>;
    }>,
    marginDb: number,
  ): boolean {
    const midi = this.midiMin + i;
    for (const p of accepted) {
      const pm = this.midiMin + p.i;
      const semis = midi - pm;
      if (semis <= 0) continue;
      const ratio = Math.pow(2, semis / 12);
      const k = Math.round(ratio);
      if (k < 2 || k > 12) continue;
      if (Math.abs(1200 * Math.log2(ratio / k)) > 50) continue; // not on a harmonic
      const h1 = p.parts[0];
      if (h1 == null) continue;
      let fitEnv = p.fits.get(k);
      if (!fitEnv) {
        fitEnv = this.fitEnvelopeExcluding(p.parts, k);
        p.fits.set(k, fitEnv);
      }
      // Parent level at slot kk: the larger of the fitted envelope (right for
      // smooth 1/k voices) and the chest-voice prior (right when the parent's
      // own H2/H3 dominate). Whichever is higher is the level a partner must beat.
      const fe = fitEnv;
      const env = (kk: number): number =>
        Math.max(fe(kk), h1 + NoteDetector.priorDb(kk));
      const excess: number[] = [];
      for (let m = 1; m <= 3; m++) {
        const own = this.partialDb(i, m);
        if (own == null) continue;
        excess.push(own - env(k * m));
      }
      if (excess.length === 0) continue;
      const mean = excess.reduce((a, b) => a + b, 0) / excess.length;
      const independent = (excess[0] as number) >= marginDb && mean >= marginDb;
      if (!independent) return true;
    }
    return false;
  }

  /** Whitened peak in note i's fundamental band exceeds `floor`? Evaluated
   *  on the pre-cancellation whitened spectrum (flat = 1.0). */
  private hasFundamentalSupport(i: number, floor: number): boolean {
    const base = i * this.binStride * 2;
    const lo = this.harmonicBins[base] as number;
    const hi = this.harmonicBins[base + 1] as number;
    const whitened = this.whitened;
    for (let b = lo; b <= hi; b++) {
      if ((whitened[b] as number) > floor) return true;
    }
    return false;
  }

  /** Normalized harmonic-sum salience of note i over spectrum `spec`. */
  private salienceOf(i: number, spec: Float32Array): number {
    const K = this.harmonicCount;
    const weights = this.harmonicWeights;
    const hbins = this.harmonicBins;
    const base = i * this.binStride * 2;
    let salience = 0;
    for (let k = 0; k < K; k++) {
      const lo = hbins[base + k * 2] as number;
      const hi = hbins[base + k * 2 + 1] as number;
      let peak = 0;
      for (let b = lo; b <= hi; b++) {
        const v = spec[b] as number;
        if (v > peak) peak = v;
      }
      salience += (weights[k] as number) * peak;
    }
    return salience * (this.noteWeightNorm[i] as number);
  }

  /**
   * Remove note i's MODELED contribution from `spec` (spectral-smoothness
   * cancellation, after Klapuri 2003). Whitening flattens spectral tilt, so a
   * single voice's whitened harmonics form a SMOOTH envelope across k. A
   * harmonic that sticks up above the local envelope of its neighbours is
   * carrying a second source (an octave/twelfth partner whose harmonics
   * coincide) — so per harmonic we subtract min(observed, smoothed-envelope),
   * leaving the partner's excess in the residual. Subtraction is done in the
   * POWER domain because incoherent voices add in power: the excess left at a
   * shared bin is then sqrt(total² − model²) ≈ exactly the partner's own
   * magnitude, rather than the quadratically-squashed remainder a linear
   * subtraction would leave.
   */
  private cancel(i: number, spec: Float32Array, gamma: number): void {
    const Kc = this.cancelHarmonicCount;
    const headroom = this.cancelHeadroom;
    const hbins = this.harmonicBins;
    const base = i * this.binStride * 2;
    const amps = this.cancelAmps;

    // Measure this note's per-harmonic peak amplitude on the current residual,
    // out to Kc (beyond the K scoring harmonics — an accepted note's UPPER
    // partials must be scrubbed too, or they get re-detected as phantom high
    // notes). -1 marks a band above Nyquist (no data) so smoothing skips it.
    for (let k = 0; k < Kc; k++) {
      const lo = hbins[base + k * 2] as number;
      const hi = hbins[base + k * 2 + 1] as number;
      if (hi < lo) {
        amps[k] = -1;
        continue;
      }
      let peak = 0;
      for (let b = lo; b <= hi; b++) {
        const v = spec[b] as number;
        if (v > peak) peak = v;
      }
      amps[k] = peak;
    }

    for (let k = 0; k < Kc; k++) {
      if ((amps[k] as number) < 0) continue;
      // Smoothed envelope d_k (Klapuri 2003, Eq. 12 in 3-point form): mean
      // of the harmonic and its immediate neighbours, self INCLUDED. Window
      // width is an empirical compromise measured on the SATB battery: the
      // octave-wide window of the original paper dilutes the note's own
      // share enough that residual crumbs at its harmonics come back as
      // super-harmonic ghosts, while excluding self makes every natural
      // envelope wobble leak. Including self pulls the model toward the
      // observation (near-full removal of clean harmonics); a genuine
      // octave partner still lifts its bin by ~√2 in power-additive
      // magnitude — above what one shared point can drag the 3-point mean —
      // so min(observed, d_k) leaves the partner's excess in the residual.
      let sum = 0;
      let cnt = 0;
      for (let j = k - 1; j <= k + 1; j++) {
        if (j < 0 || j >= Kc) continue;
        const a = amps[j] as number;
        if (a < 0) continue;
        sum += a;
        cnt++;
      }
      const smoothed = (cnt > 0 ? sum / cnt : (amps[k] as number)) * headroom;
      const own = amps[k] as number;
      const model = gamma * (own < smoothed ? own : smoothed);
      if (model <= 0) continue;
      const m2 = model * model;

      const lo = hbins[base + k * 2] as number;
      const hi = hbins[base + k * 2 + 1] as number;
      for (let b = lo; b <= hi; b++) {
        const v = spec[b] as number;
        const p = v * v - m2;
        spec[b] = p > 0 ? Math.sqrt(p) : 0;
      }
    }
  }

  /**
   * Collapse contiguous runs of active semitones (consecutive MIDI numbers a
   * gap of ≤ mergeRadius apart) to a single note. The FFT-resolution + ensemble
   * spread of a note lights its ±1 neighbours; this folds them back to the
   * energy-weighted CENTROID of the run (not the raw peak), so a smear whose
   * peak landed off-centre still resolves to the true note instead of a ±1
   * neighbour. A genuine semitone cluster also collapses — accepted, since the
   * FFT can't reliably resolve those anyway.
   */
  private mergeAdjacent(active: Set<number>): Set<number> {
    if (active.size <= 1) return active;
    const sorted = Array.from(active).sort((a, b) => a - b);
    const radius = this.mergeRadius;
    const energy = this.energy;
    const merged = new Set<number>();
    let runStart = 0;
    for (let idx = 0; idx < sorted.length; idx++) {
      const isLast = idx === sorted.length - 1;
      const gap = isLast
        ? Infinity
        : (sorted[idx + 1] as number) - (sorted[idx] as number);
      if (gap > radius) {
        // Close the run [runStart, idx]: pick the member nearest the
        // energy-weighted centroid.
        let wsum = 0;
        let esum = 0;
        for (let j = runStart; j <= idx; j++) {
          const m = sorted[j] as number;
          const e = energy[m - this.midiMin] as number;
          wsum += m * e;
          esum += e;
        }
        const centroid = esum > 0 ? wsum / esum : (sorted[runStart] as number);
        let bestMidi = sorted[runStart] as number;
        let bestDist = Math.abs(bestMidi - centroid);
        for (let j = runStart + 1; j <= idx; j++) {
          const m = sorted[j] as number;
          const dist = Math.abs(m - centroid);
          if (dist < bestDist) {
            bestDist = dist;
            bestMidi = m;
          }
        }
        merged.add(bestMidi);
        runStart = idx + 1;
      }
    }
    return merged;
  }
}
