import {
  forwardRef,
  useCallback,
  useEffect,
  useImperativeHandle,
  useMemo,
  useRef,
} from 'react';
import { WATERFALL_CONFIG } from '../config/render';
import { buildSpectrogramLut } from '../audio/spectrogram-lut';
import { labelSections, type Label } from '../audio/section-labeler';
import { freqToNoteCents, summarize } from '../audio/intonation';
import { centsColor } from '../config/intonation';
import type { MicFrame } from '../audio/mic-pipeline';
import {
  canvasToPngBlob,
  pngFileName,
  savePng,
  type SavePngResult,
} from '../utils/save-png';

export type AxisMode = 'notes' | 'hz';

export interface LiveRenderParams {
  fftSize: number;
  hopSize: number;
  minDb: number;
  maxDb: number;
  minFreqHz: number;
  maxFreqHz: number;
  axisMode: AxisMode;
}

export interface LiveSpectrogramHandle {
  appendFrame(frame: MicFrame): void;
  clear(): void;
  /**
   * Save the current canvas as a timestamped PNG. Resolves once the browser
   * has taken the download or, in the Tauri shell, the file is written;
   * rejects if the write fails. Both paths live in utils/save-png.ts.
   */
  exportPng(): Promise<SavePngResult>;
}

interface Props {
  params: LiveRenderParams;
  /** When true, the waterfall stops scrolling (frozen) — capture keeps running. */
  frozen?: boolean;
}

// Fixed geometry, derived once (WATERFALL_CONFIG is constant).
const {
  width: W,
  height: H,
  keyboardHeight: KB,
  footerHeight: FT,
  cols: COLS,
  histRows: HIST_ROWS,
  scrollIntervalMs: SCROLL_MS,
} = WATERFALL_CONFIG;
/** Waterfall plot height: canvas minus the bottom keyboard + footer bands. */
const PLOT_H = H - KB - FT;

/** Viridis floor color — empty plot and history background. */
const FLOOR_FILL = 'rgb(68, 1, 84)';
/** Dark chrome background (axis + footer bands). */
const BG_FILL = 'rgb(10, 10, 12)';
const MONO_FONT = 'ui-monospace, SFMono-Regular, Menlo, monospace';

/**
 * Live spectrogram as a vertical **waterfall**: X = log frequency, Y = time
 * with the NEWEST row at the top, scrolling downward.
 *
 * The history lives in a small DATA-space offscreen buffer (COLS × HIST_ROWS)
 * that the GPU scales up to the plot in one drawImage — not a full
 * display-resolution buffer. Each advance scrolls that small buffer down one
 * row (self-blit, cheap at data resolution) and writes the newest row at the
 * top. The bottom carries a cached horizontal piano-keyboard / Hz axis and a
 * metadata footer.
 *
 * The frame pipeline is decoupled from paint AND throttled:
 *  - The DSP worker (off-thread) sends quantized u8 magnitude columns at the
 *    audio hop rate (~90 fps).
 *  - `appendFrame` only **max-pools** the frame into a pending row (per-bin
 *    max, so transient peaks survive) and arms a requestAnimationFrame.
 *  - `drainAndPaint` advances at most one row per `scrollIntervalMs`
 *    (~30 rows/s), decoupling scroll speed from the producer and keeping the
 *    paint cost flat. Frames arriving between advances fold into the pending
 *    row; their active-note sets union.
 *
 * Color is a 256-entry byte→RGB LUT keyed by the dB-clamp window, rebuilt
 * only when the clamp changes. (Render techniques borrowed from the
 * Resonator waterfall; Coral stays a TypeScript/Vite app.)
 *
 * Imperative-handle API (appendFrame / clear) bypasses React's render cycle.
 * Render params come through props and are stashed in a ref so the imperative
 * path always reads the latest values.
 *
 * NOT done: devicePixelRatio handling. The backing store is a fixed
 * W × H scaled by CSS, so text and 1-px separators soften on 2× displays
 * rendered large. Deliberately deferred to the WebGL2 renderer milestone.
 */
export const LiveSpectrogramView = forwardRef<LiveSpectrogramHandle, Props>(
  ({ params, frozen = false }, ref) => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const paramsRef = useRef(params);
    const frozenRef = useRef(frozen);

    // Color LUT: source byte → RGB through the [minDb, maxDb] window. Rebuilt
    // only when the clamp changes; held in a ref so the imperative paint path
    // reads it without re-binding callbacks.
    const lut = useMemo(
      () => buildSpectrogramLut(params.minDb, params.maxDb),
      [params.minDb, params.maxDb]
    );
    const lutRef = useRef(lut);

    // Per-column log-freq interpolation table (display column → FFT bin).
    // Rebuilt when numBins (== fftSize/2), sampleRate, or the freq range
    // changes — all independent of magnitude data.
    const binLoRef = useRef<Int32Array | null>(null);
    const binFracRef = useRef<Float32Array | null>(null);
    const cachedColKeyRef = useRef('');

    // Data-space history buffer (COLS × HIST_ROWS) + a reusable 1-row scratch.
    const bufRef = useRef<HTMLCanvasElement | null>(null);
    const bufCtxRef = useRef<CanvasRenderingContext2D | null>(null);
    const rowImageRef = useRef<ImageData | null>(null);
    // Per-row F0 (Hz) for the intonation trail, parallel to the buffer rows.
    // NaN = unvoiced. Scrolls in lockstep with the waterfall.
    const f0RingRef = useRef<Float32Array | null>(null);

    // Cached static keyboard/Hz axis layer, rebuilt on freq-range/axis change.
    const kbCanvasRef = useRef<HTMLCanvasElement | null>(null);
    const kbCtxRef = useRef<CanvasRenderingContext2D | null>(null);
    const cachedKbKeyRef = useRef('');

    // Producer→paint decoupling + throttle. Frames max-pool into pendingBytes;
    // a row advances at most once per SCROLL_MS.
    const pendingBytesRef = useRef<Uint8Array | null>(null);
    const pendingCountRef = useRef(0);
    const activeAccumRef = useRef<Set<number>>(new Set());
    const latestFrameRef = useRef<MicFrame | null>(null);
    const latestActiveMidiRef = useRef<ReadonlySet<number>>(EMPTY_ACTIVE_SET);
    const lastAdvanceMsRef = useRef(0);
    const rafIdRef = useRef<number | null>(null);
    const rafScheduledRef = useRef(false);

    useEffect(() => {
      lutRef.current = lut;
    }, [lut]);
    useEffect(() => {
      frozenRef.current = frozen;
    }, [frozen]);

    // Lazily build the column→bin table for the current framing + freq range.
    const ensureColumnTable = useCallback(
      (numBins: number, sampleRate: number) => {
        const p = paramsRef.current;
        const key = `${numBins}|${sampleRate}|${p.minFreqHz}|${p.maxFreqHz}`;
        if (key === cachedColKeyRef.current) return;

        const binFreqHz = sampleRate / (2 * numBins);
        const logFreqRatio = Math.log(p.maxFreqHz / p.minFreqHz);
        const binLo = new Int32Array(COLS);
        const binFrac = new Float32Array(COLS);
        for (let x = 0; x < COLS; x++) {
          const xNorm = x / (COLS - 1);
          const freq = p.minFreqHz * Math.exp(xNorm * logFreqRatio);
          const binFloat = freq / binFreqHz;
          const lo = Math.max(0, Math.min(numBins - 2, Math.floor(binFloat)));
          binLo[x] = lo;
          binFrac[x] = Math.max(0, Math.min(1, binFloat - lo));
        }
        binLoRef.current = binLo;
        binFracRef.current = binFrac;
        cachedColKeyRef.current = key;
      },
      []
    );

    // Lazily create the data-space history buffer, filled with floor color.
    const ensureBuffer = useCallback((): HTMLCanvasElement | null => {
      if (bufRef.current) return bufRef.current;
      const off = document.createElement('canvas');
      off.width = COLS;
      off.height = HIST_ROWS;
      const bctx = off.getContext('2d', { alpha: false });
      bufRef.current = off;
      bufCtxRef.current = bctx;
      rowImageRef.current = bctx ? bctx.createImageData(COLS, 1) : null;
      if (bctx) {
        bctx.fillStyle = FLOOR_FILL;
        bctx.fillRect(0, 0, COLS, HIST_ROWS);
      }
      return off;
    }, []);

    // Lazily (re)build the cached static keyboard/Hz axis layer.
    const ensureKeyboard = useCallback((): HTMLCanvasElement | null => {
      const p = paramsRef.current;
      const key = `${p.axisMode}|${p.minFreqHz}|${p.maxFreqHz}`;
      if (kbCanvasRef.current && key === cachedKbKeyRef.current) {
        return kbCanvasRef.current;
      }
      let kb = kbCanvasRef.current;
      if (!kb) {
        kb = document.createElement('canvas');
        kb.width = W;
        kb.height = KB;
        kbCanvasRef.current = kb;
        kbCtxRef.current = kb.getContext('2d', { alpha: false });
      }
      const kctx = kbCtxRef.current;
      if (kctx) {
        drawKeyboardLayer(kctx, p);
        cachedKbKeyRef.current = key;
      }
      return kb;
    }, []);

    // Composite the visible canvas: waterfall plot, grid, keyboard +
    // highlights, footer. Reads current buffer/active/frame from refs.
    const paint = useCallback(() => {
      const canvas = canvasRef.current;
      if (!canvas) return;
      const ctx = canvas.getContext('2d', { alpha: false });
      if (!ctx) return;
      const p = paramsRef.current;

      const buf = ensureBuffer();
      ctx.imageSmoothingEnabled = true;
      if (buf) {
        ctx.drawImage(buf, 0, 0, COLS, HIST_ROWS, 0, 0, W, PLOT_H);
      } else {
        ctx.fillStyle = FLOOR_FILL;
        ctx.fillRect(0, 0, W, PLOT_H);
      }

      drawPlotGrid(ctx, p);
      drawPitchTrail(ctx, p, f0RingRef.current);

      // Time-flow hint at the top-left of the plot.
      ctx.fillStyle = 'rgba(156, 163, 175, 0.75)';
      ctx.font = `10px ${MONO_FONT}`;
      ctx.textAlign = 'left';
      ctx.textBaseline = 'top';
      ctx.fillText('↓ time', 6, 6);

      drawLiveReadout(ctx, latestFrameRef.current);

      const kb = ensureKeyboard();
      if (kb) ctx.drawImage(kb, 0, PLOT_H);
      drawActiveKeys(ctx, p, latestActiveMidiRef.current);

      drawFooter(ctx, p, latestFrameRef.current, f0RingRef.current);
    }, [ensureBuffer, ensureKeyboard]);

    // Build the newest row from the pooled bytes, scroll the buffer down one
    // row, and write the new row at the top.
    const advanceRow = useCallback(
      (pend: Uint8Array) => {
        const frame = latestFrameRef.current;
        if (!frame) return;
        ensureColumnTable(frame.numBins, frame.sampleRate);
        ensureBuffer();
        const bctx = bufCtxRef.current;
        const buf = bufRef.current;
        const rowImg = rowImageRef.current;
        const binLo = binLoRef.current;
        const binFrac = binFracRef.current;
        if (!bctx || !buf || !rowImg || !binLo || !binFrac) return;

        const px = rowImg.data;
        const lutArr = lutRef.current;
        for (let x = 0; x < COLS; x++) {
          const lo = binLo[x] as number;
          const frac = binFrac[x] as number;
          const bLo = pend[lo] as number;
          const bHi = pend[lo + 1] as number;
          let bi = (bLo + frac * (bHi - bLo) + 0.5) | 0;
          if (bi < 0) bi = 0;
          else if (bi > 255) bi = 255;
          const li = bi * 3;
          const o = x * 4;
          px[o] = lutArr[li] as number;
          px[o + 1] = lutArr[li + 1] as number;
          px[o + 2] = lutArr[li + 2] as number;
          px[o + 3] = 255;
        }
        // Scroll down one row, then stamp the newest row at the top.
        bctx.drawImage(buf, 0, 0, COLS, HIST_ROWS - 1, 0, 1, COLS, HIST_ROWS - 1);
        bctx.putImageData(rowImg, 0, 0);

        // Scroll the F0 trail in lockstep; newest at row 0.
        let f0Ring = f0RingRef.current;
        if (!f0Ring) {
          f0Ring = new Float32Array(HIST_ROWS).fill(NaN);
          f0RingRef.current = f0Ring;
        }
        f0Ring.copyWithin(1, 0, HIST_ROWS - 1);
        f0Ring[0] = frame.f0Hz != null && frame.f0Confidence > 0 ? frame.f0Hz : NaN;
      },
      [ensureColumnTable, ensureBuffer]
    );

    // Consumer: advance at most one row per SCROLL_MS, then repaint. Re-armed
    // by appendFrame while frames arrive (and self-re-armed to flush a pending
    // row if the producer pauses mid-interval), so it never spins idle.
    const drainAndPaint = useCallback(() => {
      rafScheduledRef.current = false;
      const canvas = canvasRef.current;
      if (!canvas) return;
      const pend = pendingBytesRef.current;
      if (pendingCountRef.current === 0 || !pend) return;

      if (frozenRef.current) {
        // Frozen: hold the captured image, drop incoming frames. Capture keeps
        // running upstream, so the level meter still moves.
        pend.fill(0);
        pendingCountRef.current = 0;
        activeAccumRef.current.clear();
        return;
      }

      const now = performance.now();
      if (now - lastAdvanceMsRef.current < SCROLL_MS) {
        // Not time yet — wake again so the pending row still flushes if the
        // producer goes quiet before the next frame.
        if (!rafScheduledRef.current) {
          rafScheduledRef.current = true;
          rafIdRef.current = requestAnimationFrame(drainAndPaint);
        }
        return;
      }

      advanceRow(pend);
      pend.fill(0);
      pendingCountRef.current = 0;
      lastAdvanceMsRef.current = now;
      latestActiveMidiRef.current = new Set(activeAccumRef.current);
      activeAccumRef.current.clear();
      paint();
    }, [advanceRow, paint]);

    // Producer: max-pool the frame into the pending row and arm the loop. No
    // canvas work here.
    const appendFrame = useCallback(
      (frame: MicFrame) => {
        const fb = frame.bytes;
        let pend = pendingBytesRef.current;
        if (!pend || pend.length !== fb.length) {
          pend = new Uint8Array(fb.length);
          pendingBytesRef.current = pend;
        }
        for (let i = 0; i < fb.length; i++) {
          const v = fb[i] as number;
          if (v > (pend[i] as number)) pend[i] = v;
        }
        const accum = activeAccumRef.current;
        for (const m of frame.activeMidi) accum.add(m);
        latestFrameRef.current = frame;
        pendingCountRef.current++;

        if (!rafScheduledRef.current) {
          rafScheduledRef.current = true;
          rafIdRef.current = requestAnimationFrame(drainAndPaint);
        }
      },
      [drainAndPaint]
    );

    const clear = useCallback(() => {
      if (rafIdRef.current !== null) {
        cancelAnimationFrame(rafIdRef.current);
        rafIdRef.current = null;
      }
      rafScheduledRef.current = false;
      pendingCountRef.current = 0;
      pendingBytesRef.current?.fill(0);
      activeAccumRef.current.clear();
      latestFrameRef.current = null;
      latestActiveMidiRef.current = EMPTY_ACTIVE_SET;
      lastAdvanceMsRef.current = 0;
      f0RingRef.current?.fill(NaN);

      const buf = ensureBuffer();
      const bctx = bufCtxRef.current;
      if (buf && bctx) {
        bctx.fillStyle = FLOOR_FILL;
        bctx.fillRect(0, 0, COLS, HIST_ROWS);
      }
      paint();
    }, [ensureBuffer, paint]);

    const exportPng = useCallback(async (): Promise<SavePngResult> => {
      const canvas = canvasRef.current;
      if (!canvas) throw new Error('exportPng: canvas is not mounted');
      const blob = await canvasToPngBlob(canvas);
      return savePng(blob, pngFileName());
    }, []);

    useImperativeHandle(
      ref,
      () => ({ appendFrame, clear, exportPng }),
      [appendFrame, clear, exportPng]
    );

    // Initial paint, and repaint on param changes (freq range / axis mode /
    // clamp) so the view updates even while paused.
    useEffect(() => {
      paramsRef.current = params;
      paint();
    }, [params, paint]);

    // Cancel any pending animation frame on unmount.
    useEffect(() => {
      return () => {
        if (rafIdRef.current !== null) cancelAnimationFrame(rafIdRef.current);
      };
    }, []);

    return (
      <canvas
        ref={canvasRef}
        width={W}
        height={H}
        className="w-full border border-neutral-700 rounded bg-black"
      />
    );
  }
);

LiveSpectrogramView.displayName = 'LiveSpectrogramView';

/** MIDI note → frequency. Reference: A4 (MIDI 69) = 440 Hz. */
function midiToFreq(midi: number): number {
  return 440 * Math.pow(2, (midi - 69) / 12);
}

/** Inverse: freq → MIDI note number (continuous). */
function freqToMidi(freq: number): number {
  return 12 * Math.log2(freq / 440) + 69;
}

/** X position (px) for a frequency on the log axis spanning the plot width. */
function xFromFreq(freq: number, p: LiveRenderParams): number {
  const logFreqRatio = Math.log(p.maxFreqHz / p.minFreqHz);
  return (Math.log(freq / p.minFreqHz) / logFreqRatio) * (W - 1);
}

/** Natural notes (white keys): C, D, E, F, G, A, B. */
const NATURAL_PCS = new Set([0, 2, 4, 5, 7, 9, 11]);
const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

/** Reused for the initial paint when no notes are active yet. */
const EMPTY_ACTIVE_SET: ReadonlySet<number> = new Set();

interface HzTick {
  freq: number;
  label: string;
}

/** Hz ticks at log-friendly engineering values. */
const HZ_TICKS: ReadonlyArray<HzTick> = [
  20, 50, 100, 200, 500, 1000, 2000, 5000, 8000, 10000, 16000, 20000,
].map((hz) => ({
  freq: hz,
  label: hz < 1000 ? `${hz}` : `${hz / 1000}k`,
}));

/**
 * Draw the static keyboard/Hz axis into the cached layer (no highlights).
 * In Notes mode this is a horizontal DAW-style piano keyboard: every
 * semitone gets a vertical strip, naturals/accidentals get distinct shading,
 * octave C's and A4 are labeled. In Hz mode it draws engineering ticks.
 */
function drawKeyboardLayer(ctx: CanvasRenderingContext2D, p: LiveRenderParams): void {
  ctx.fillStyle = BG_FILL;
  ctx.fillRect(0, 0, W, KB);

  if (p.axisMode === 'notes') drawPianoKeys(ctx, p);
  else drawHzTicks(ctx, p);

  // Separator line between the plot and the keyboard band.
  ctx.strokeStyle = 'rgb(38, 38, 42)';
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(0, 0.5);
  ctx.lineTo(W, 0.5);
  ctx.stroke();
}

function drawPianoKeys(ctx: CanvasRenderingContext2D, p: LiveRenderParams): void {
  const midiMin = Math.floor(freqToMidi(p.minFreqHz)) - 1;
  const midiMax = Math.ceil(freqToMidi(p.maxFreqHz)) + 1;
  const halfStep = Math.pow(2, 1 / 24);

  const NATURAL_FILL = 'rgb(26, 26, 30)';
  const ACCIDENTAL_FILL = 'rgb(8, 8, 11)';
  const SEMITONE_LINE = 'rgba(60, 60, 65, 0.5)';
  const OCTAVE_LINE = 'rgba(120, 120, 130, 0.7)';

  // Pass 1: semitone strips, each centered on the note ±half a semitone.
  for (let midi = midiMin; midi <= midiMax; midi++) {
    const freq = midiToFreq(midi);
    const pc = ((midi % 12) + 12) % 12;
    const isNatural = NATURAL_PCS.has(pc);
    const xL = xFromFreq(freq / halfStep, p);
    const xR = xFromFreq(freq * halfStep, p);
    const x0 = Math.max(0, Math.floor(xL));
    const x1 = Math.min(W, Math.ceil(xR));
    if (x1 <= x0) continue;
    ctx.fillStyle = isNatural ? NATURAL_FILL : ACCIDENTAL_FILL;
    ctx.fillRect(x0, 0, x1 - x0, KB);
  }

  // Pass 2: separator lines between semitones, brighter on octave (C) edges.
  ctx.lineWidth = 1;
  for (let midi = midiMin; midi <= midiMax + 1; midi++) {
    const freq = midiToFreq(midi);
    const xEdge = xFromFreq(freq / halfStep, p);
    if (xEdge < 0 || xEdge > W) continue;
    const pc = ((midi % 12) + 12) % 12;
    ctx.strokeStyle = pc === 0 ? OCTAVE_LINE : SEMITONE_LINE;
    ctx.beginPath();
    ctx.moveTo(Math.round(xEdge) + 0.5, 0);
    ctx.lineTo(Math.round(xEdge) + 0.5, KB);
    ctx.stroke();
  }

  // Pass 3: labels for octave C's and A4 (keeps the strip readable without
  // clutter; active notes get their own label at paint time).
  ctx.font = `10px ${MONO_FONT}`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  for (let midi = midiMin; midi <= midiMax; midi++) {
    const pc = ((midi % 12) + 12) % 12;
    const isA4 = midi === 69;
    if (pc !== 0 && !isA4) continue;
    const freq = midiToFreq(midi);
    if (freq < p.minFreqHz || freq > p.maxFreqHz) continue;
    const octave = Math.floor(midi / 12) - 1;
    const name = isA4 ? 'A4' : `C${octave}`;
    ctx.fillStyle = isA4 ? '#ffffff' : '#e5e7eb';
    ctx.fillText(name, xFromFreq(freq, p), KB / 2);
  }
}

function drawHzTicks(ctx: CanvasRenderingContext2D, p: LiveRenderParams): void {
  ctx.fillStyle = '#9ca3af';
  ctx.font = `10px ${MONO_FONT}`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.strokeStyle = '#6b7280';
  for (const tick of HZ_TICKS) {
    if (tick.freq < p.minFreqHz || tick.freq > p.maxFreqHz) continue;
    const x = xFromFreq(tick.freq, p);
    ctx.beginPath();
    ctx.moveTo(x + 0.5, 0);
    ctx.lineTo(x + 0.5, 8);
    ctx.stroke();
    ctx.fillText(tick.label, x, KB / 2);
  }
}

/** Per-SATB-section highlight colors (fill + text). `A/T?` = neutral gray. */
const SECTION_STYLE: Record<Label, { fill: string; text: string }> = {
  B: { fill: 'rgba(59, 130, 246, 0.38)', text: '#93c5fd' }, // blue
  T: { fill: 'rgba(20, 184, 166, 0.38)', text: '#5eead4' }, // teal
  A: { fill: 'rgba(34, 197, 94, 0.38)', text: '#86efac' }, // green
  S: { fill: 'rgba(245, 158, 11, 0.38)', text: '#fcd34d' }, // amber
  'A/T?': { fill: 'rgba(156, 163, 175, 0.30)', text: '#d1d5db' }, // gray
};

/**
 * Highlight currently-sounding notes on the keyboard band, tinted + lettered by
 * SATB section. Sections are labeled locally from the active MIDI set
 * (pitch-range priors, no SPR — matching the P3 production decision); a mid note
 * the labeler can't pin shows `?` (the honest A/T abstention).
 */
function drawActiveKeys(
  ctx: CanvasRenderingContext2D,
  p: LiveRenderParams,
  activeMidi: ReadonlySet<number>
): void {
  if (activeMidi.size === 0) return;
  const labelByMidi = new Map(labelSections(activeMidi).map((l) => [l.midi, l.label]));
  const halfStep = Math.pow(2, 1 / 24);
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  for (const midi of activeMidi) {
    const freq = midiToFreq(midi);
    if (freq < p.minFreqHz || freq > p.maxFreqHz) continue;
    const xL = xFromFreq(freq / halfStep, p);
    const xR = xFromFreq(freq * halfStep, p);
    const x0 = Math.max(0, Math.floor(xL));
    const x1 = Math.min(W, Math.ceil(xR));
    const cx = (x0 + x1) / 2;
    const label = labelByMidi.get(midi) ?? 'A/T?';
    const style = SECTION_STYLE[label];

    ctx.fillStyle = style.fill;
    ctx.fillRect(x0, PLOT_H, x1 - x0, KB);

    // Section letter on top, note name below.
    ctx.fillStyle = style.text;
    ctx.font = `bold 12px ${MONO_FONT}`;
    ctx.fillText(label === 'A/T?' ? '?' : label, cx, PLOT_H + KB * 0.32);

    const pc = ((midi % 12) + 12) % 12;
    const octave = Math.floor(midi / 12) - 1;
    ctx.font = `9px ${MONO_FONT}`;
    ctx.fillText(`${NOTE_NAMES[pc]}${octave}`, cx, PLOT_H + KB * 0.68);
  }
}

/** Subtle vertical reference lines over the plot (octave C's, or Hz ticks). */
function drawPlotGrid(ctx: CanvasRenderingContext2D, p: LiveRenderParams): void {
  ctx.lineWidth = 1;
  ctx.strokeStyle = 'rgba(120, 120, 130, 0.14)';
  if (p.axisMode === 'notes') {
    const midiMin = Math.floor(freqToMidi(p.minFreqHz));
    const midiMax = Math.ceil(freqToMidi(p.maxFreqHz));
    for (let midi = midiMin; midi <= midiMax; midi++) {
      if (((midi % 12) + 12) % 12 !== 0) continue;
      const freq = midiToFreq(midi);
      if (freq < p.minFreqHz || freq > p.maxFreqHz) continue;
      const x = Math.round(xFromFreq(freq, p)) + 0.5;
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, PLOT_H);
      ctx.stroke();
    }
  } else {
    for (const tick of HZ_TICKS) {
      if (tick.freq < p.minFreqHz || tick.freq > p.maxFreqHz) continue;
      const x = Math.round(xFromFreq(tick.freq, p)) + 0.5;
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, PLOT_H);
      ctx.stroke();
    }
  }
}

const centsSign = (c: number): string => (c >= 0 ? '+' : '−');

function drawFooter(
  ctx: CanvasRenderingContext2D,
  p: LiveRenderParams,
  frame: MicFrame | null,
  f0Ring: Float32Array | null
): void {
  const yTop = PLOT_H + KB;
  ctx.fillStyle = BG_FILL;
  ctx.fillRect(0, yTop, W, FT);
  ctx.font = `11px ${MONO_FONT}`;
  ctx.textBaseline = 'middle';
  const cy = yTop + FT / 2;

  // Left: capture metadata.
  const sampleRate = frame ? `${frame.sampleRate} Hz` : 'mic idle';
  ctx.fillStyle = '#9ca3af';
  ctx.textAlign = 'left';
  ctx.fillText(
    `LIVE · ${sampleRate} · ${p.fftSize} fft · ${p.hopSize} hop · ${p.minFreqHz}–${p.maxFreqHz} Hz`,
    8,
    cy
  );

  // Right: rolling intonation summary over the visible F0 trail.
  if (frame && f0Ring) {
    const scrollRate = 1000 / SCROLL_MS;
    const arr = Array.from(f0Ring, (v) => (v > 0 ? v : null));
    const s = summarize(arr, frame.sampleRate / scrollRate, frame.sampleRate);
    if (s.voicedFraction > 0.05 && s.pitchRange) {
      const vib = s.vibratoRateHz ? ` · vib ${s.vibratoRateHz.toFixed(1)} Hz` : '';
      const txt =
        `${Math.round(s.voicedFraction * 100)}% voiced · ${s.pitchRange[0]}–${s.pitchRange[1]} · ` +
        `mean ${centsSign(s.meanCents)}${Math.abs(Math.round(s.meanCents))}¢ · RMS ${Math.round(s.rmsCents)}¢${vib}`;
      ctx.fillStyle = '#d1d5db';
      ctx.textAlign = 'right';
      ctx.fillText(txt, W - 8, cy);
    }
  }
}

/** The F0 intonation trail: one colored mark per row at its pitch, by cents-from-ET. */
function drawPitchTrail(
  ctx: CanvasRenderingContext2D,
  p: LiveRenderParams,
  f0Ring: Float32Array | null
): void {
  if (!f0Ring) return;
  const rowH = Math.max(1, PLOT_H / HIST_ROWS) + 0.6;
  for (let ri = 0; ri < HIST_ROWS; ri++) {
    const f0 = f0Ring[ri] as number;
    if (!(f0 > 0) || f0 < p.minFreqHz || f0 > p.maxFreqHz) continue;
    const x = xFromFreq(f0, p);
    const y = (ri / HIST_ROWS) * PLOT_H;
    ctx.fillStyle = centsColor(freqToNoteCents(f0).cents);
    ctx.fillRect(x - 1.5, y, 3, rowH);
  }
}

/** Big top-right readout of the current pitch + cents deviation. */
function drawLiveReadout(ctx: CanvasRenderingContext2D, frame: MicFrame | null): void {
  if (!frame || frame.f0Hz == null || frame.f0Confidence <= 0) return;
  const { noteName, cents } = freqToNoteCents(frame.f0Hz);
  const r = Math.round(cents);
  ctx.font = `bold 18px ${MONO_FONT}`;
  ctx.textAlign = 'right';
  ctx.textBaseline = 'top';
  ctx.fillStyle = centsColor(cents);
  ctx.fillText(`${noteName} ${centsSign(r)}${Math.abs(r)}¢`, W - 8, 6);
}
