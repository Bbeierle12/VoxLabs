/**
 * Synthetic SATB choir-chord generator — the ground-truth foundation for the
 * section-identification MVP.
 *
 * You cannot make detection "intelligent" without a way to MEASURE it, and for
 * source separation that means ground truth you control. This module renders a
 * described SATB chord to **audio samples** (not magnitudes) so the output runs
 * through the exact same StreamingStft the live worker uses — the detector sees
 * what it would see from a real recording, and we know the right answer.
 *
 * Each section is a *chorus* of N singers, not one partial: per-singer detune,
 * independent vibrato (FM) and tremolo (AM), so a section reads as a slightly
 * fuzzy band the way a real choir does. A voice's harmonics share its vibrato
 * (k·φ off one integrated fundamental phase), which is the common-fate cue any
 * harmonic-grouping stage relies on. Deterministic given a seed.
 *
 * Out of scope here: running the detector (P1 wires synth → STFT → detector).
 */

const TWO_PI = Math.PI * 2;

/** MIDI note → frequency. Reference: A4 (MIDI 69) = 440 Hz. */
function midiToFreq(midi: number): number {
  return 440 * Math.pow(2, (midi - 69) / 12);
}

/** Small deterministic PRNG (mulberry32) so tests are reproducible by seed. */
function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Standard-normal sample via Box–Muller (unit variance, zero mean). */
function gaussian(rng: () => number): number {
  let u = 0;
  let v = 0;
  while (u === 0) u = rng();
  while (v === 0) v = rng();
  return Math.sqrt(-2 * Math.log(u)) * Math.cos(TWO_PI * v);
}

export type Section = "S" | "A" | "T" | "B";

export interface VoiceSpec {
  /** MIDI note this section sings. */
  midi: number;
  /** SATB section label (for ground truth; does not affect synthesis). */
  section: Section;
  /** Singers in the section (the chorus spread). Default 8. */
  singers?: number;
  /** Std-dev of per-singer static detune, cents. Default 12. */
  detuneCents?: number;
  /** Mean vibrato rate, Hz (jittered ±10% per singer). Default 5.5. */
  vibratoHz?: number;
  /** Vibrato depth, cents (peak deviation). Default 30. */
  vibratoCents?: number;
  /** Linear section gain before normalization. Default 1. */
  gain?: number;
  /** Per-voice singer's-formant center, Hz. Overrides the section default. */
  formantHz?: number;
  /**
   * Explicit per-harmonic amplitudes in dB (index 0 = fundamental). When set,
   * this REPLACES the 1/k · tilt · formant model for the voice, so a chest
   * voice whose H2/H3 sit above H1 (common on open vowels) can be rendered.
   * Harmonics beyond the array are silent.
   */
  harmonicDb?: readonly number[];
}

export interface ChoirChordSpec {
  sampleRate: number;
  durationSec: number;
  voices: VoiceSpec[];
  /** Max harmonics per voice (Nyquist-limited automatically). Default 16. */
  harmonics?: number;
  /** Spectral tilt: harmonic amp ×= exp(-f / (2·rolloffHz)). Default 4000. */
  rolloffHz?: number;
  /**
   * GLOBAL singer's-formant center override, Hz. If unset, each voice uses its
   * section default (B 2384 / T 2705 / A 2900 / S 3092 Hz); a per-voice
   * VoiceSpec.formantHz overrides both. 0 disables the formant.
   */
  formantHz?: number;
  /** Boost at the formant center, dB. Default 6. */
  formantGainDb?: number;
  /** Formant bandwidth, Hz. Default 1500. */
  formantBwHz?: number;
  /** Per-singer tremolo (AM) depth, 0..1. Default 0.04. */
  amDepth?: number;
  /** Added white-noise RMS (linear, relative to the normalized 0.9 peak). Default 0. */
  noiseFloor?: number;
  /** PRNG seed for reproducibility. Default 1. */
  seed?: number;
}

export interface ChoirChordResult {
  samples: Float32Array;
  sampleRate: number;
  /** Distinct sounding MIDI notes, ascending (the multi-F0 ground truth). */
  groundTruthMidi: number[];
  /** Per-section → MIDI mapping (keeps unison/doubling collisions visible). */
  sections: ReadonlyArray<{ section: Section; midi: number }>;
}

const DEFAULTS = {
  singers: 8,
  // Within-section F0 dispersion SD ~20–30 cents in real choirs (research brief).
  detuneCents: 25,
  vibratoHz: 5.5,
  // Ensemble vibrato extent ~44 cents peak-to-peak ⇒ ~±22 cents.
  vibratoCents: 22,
  harmonics: 16,
  rolloffHz: 4000,
  formantGainDb: 6,
  formantBwHz: 1500,
  amDepth: 0.04,
  noiseFloor: 0,
  seed: 1,
} as const;

/**
 * Singer's-formant center by section, Hz (Sundberg 2001; Alto interpolated
 * between Tenor and Soprano). Sections MUST differ here, or the SATB labeler's
 * SPR tiebreaker (2–4 kHz energy) has nothing to distinguish them by.
 */
const SECTION_FORMANT_HZ: Record<Section, number> = {
  B: 2384,
  T: 2705,
  A: 2900,
  S: 3092,
};

/**
 * Render an SATB chord to mono audio plus its ground truth.
 *
 * Synthesis is singer-major: each singer integrates one fundamental phase
 * across the whole buffer (so vibrato is continuous) and sums Nyquist-limited
 * harmonics off k·φ. The mix is peak-normalized to 0.9 so loudness is
 * comparable across chords; optional noise is added after normalization.
 */
export function synthesizeChoirChord(spec: ChoirChordSpec): ChoirChordResult {
  const sampleRate = spec.sampleRate;
  if (!(sampleRate > 0)) {
    throw new Error(
      `synthesizeChoirChord: sampleRate must be positive, got ${sampleRate}`,
    );
  }
  if (!(spec.durationSec > 0)) {
    throw new Error(
      `synthesizeChoirChord: durationSec must be positive, got ${spec.durationSec}`,
    );
  }
  if (spec.voices.length === 0) {
    throw new Error("synthesizeChoirChord: at least one voice is required");
  }

  const len = Math.max(1, Math.round(spec.durationSec * sampleRate));
  const nyquist = sampleRate / 2;
  const maxK = spec.harmonics ?? DEFAULTS.harmonics;
  const rolloffHz = spec.rolloffHz ?? DEFAULTS.rolloffHz;
  const formantLin =
    Math.pow(10, (spec.formantGainDb ?? DEFAULTS.formantGainDb) / 20) - 1;
  const formantBwHz = spec.formantBwHz ?? DEFAULTS.formantBwHz;
  const amDepth = spec.amDepth ?? DEFAULTS.amDepth;
  const rng = mulberry32(spec.seed ?? DEFAULTS.seed);

  const samples = new Float32Array(len);

  for (const voice of spec.voices) {
    const f0base = midiToFreq(voice.midi);
    const singers = voice.singers ?? DEFAULTS.singers;
    const detuneCents = voice.detuneCents ?? DEFAULTS.detuneCents;
    const vibratoHz = voice.vibratoHz ?? DEFAULTS.vibratoHz;
    const vibDepthOct = (voice.vibratoCents ?? DEFAULTS.vibratoCents) / 1200;
    // Equal-power summing: section loudness stays ~constant as singers grow.
    const perSingerGain = (voice.gain ?? 1) / Math.sqrt(singers);
    // Per-section singer's formant (or an explicit override).
    const voiceFormantHz =
      voice.formantHz ?? spec.formantHz ?? SECTION_FORMANT_HZ[voice.section];

    for (let s = 0; s < singers; s++) {
      const f0 = f0base * Math.pow(2, (gaussian(rng) * detuneCents) / 1200);
      const vibRate = vibratoHz * (1 + 0.1 * gaussian(rng));
      const vibPhase = rng() * TWO_PI;
      const amRate = 3 + rng() * 3; // 3–6 Hz tremolo
      const amPhase = rng() * TWO_PI;
      let phi = rng() * TWO_PI; // integrated fundamental phase

      // Precompute Nyquist-limited harmonic amplitudes (formant + tilt + 1/k).
      const amps = new Float64Array(maxK + 1);
      let kMax = 0;
      const explicit = voice.harmonicDb;
      for (let k = 1; k <= maxK; k++) {
        const fk = k * f0;
        if (fk >= nyquist) break;
        if (explicit) {
          if (k > explicit.length) break;
          amps[k] = Math.pow(10, (explicit[k - 1] as number) / 20);
          kMax = k;
          continue;
        }
        let a = 1 / k;
        if (voiceFormantHz > 0 && formantLin > 0) {
          const r = (fk - voiceFormantHz) / formantBwHz;
          a *= 1 + formantLin * Math.exp(-r * r);
        }
        a *= Math.exp(-fk / (2 * rolloffHz));
        amps[k] = a;
        kMax = k;
      }

      const wVib = TWO_PI * vibRate;
      const wAm = TWO_PI * amRate;
      const invSr = 1 / sampleRate;
      for (let n = 0; n < len; n++) {
        const t = n * invSr;
        const vib = Math.pow(2, vibDepthOct * Math.sin(wVib * t + vibPhase));
        phi += TWO_PI * f0 * vib * invSr;
        const am = 1 + amDepth * Math.sin(wAm * t + amPhase);
        let acc = 0;
        for (let k = 1; k <= kMax; k++) {
          acc += (amps[k] as number) * Math.sin(k * phi);
        }
        samples[n] = (samples[n] as number) + perSingerGain * am * acc;
      }
    }
  }

  // Peak-normalize to 0.9 so chords are loudness-comparable, then add noise.
  let peak = 0;
  for (let n = 0; n < len; n++) {
    const a = Math.abs(samples[n] as number);
    if (a > peak) peak = a;
  }
  if (peak > 0) {
    const g = 0.9 / peak;
    for (let n = 0; n < len; n++) samples[n] = (samples[n] as number) * g;
  }
  const noiseFloor = spec.noiseFloor ?? DEFAULTS.noiseFloor;
  if (noiseFloor > 0) {
    for (let n = 0; n < len; n++) {
      samples[n] = (samples[n] as number) + gaussian(rng) * noiseFloor;
    }
  }

  const groundTruthMidi = Array.from(
    new Set(spec.voices.map((v) => v.midi)),
  ).sort((a, b) => a - b);
  const sections = spec.voices.map((v) => ({
    section: v.section,
    midi: v.midi,
  }));

  return { samples, sampleRate, groundTruthMidi, sections };
}
