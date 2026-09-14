import { useEffect, useRef } from 'react';
import { RENDER_CONFIG } from '../config/render';
import { FFT_CONFIG } from '../config/fft';
import { viridis } from '../utils/colormap';
import { log } from '../utils/log';
import type { SpectrogramData } from '../audio/spectrogram';

/**
 * Reproducibility metadata baked into the rendered artifact's footer.
 * Everything needed to reconstruct the picture from the source file.
 */
export interface RenderMetadata {
  fileHash: string;
  durationSec: number;
}

interface Props {
  data: SpectrogramData | null;
  metadata: RenderMetadata | null;
}

/**
 * Renders a SpectrogramData buffer to a 2D canvas.
 *
 * X axis is time (linear, left to right), max-pooled within each pixel
 * column so transients don't disappear between frames when numFrames > width.
 * Y axis is frequency (log scale, low at bottom), with linear interpolation
 * between adjacent FFT bins to smooth log-row aliasing.
 * Color encodes magnitude in dB, clamped to [minDb, maxDb] from config.
 *
 * The bottom `metadataHeight` rows are reserved for an analysis-parameter
 * footer — research-grade exports carry their own provenance.
 *
 * Renders once per data change. For interactive scrub this becomes a
 * WebGL2 pass — but for the walking skeleton, ImageData + putImageData
 * is fast enough for 1200x480 and trivially debuggable.
 */
export function SpectrogramView({ data, metadata }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!data || !metadata) return;
    const canvas = canvasRef.current;
    if (!canvas) return;

    const t0 = performance.now();
    renderSpectrogram(canvas, data, metadata);
    log.info('render:done', {
      ms: Math.round(performance.now() - t0),
      frames: data.numFrames,
      bins: data.numBins,
    });
  }, [data, metadata]);

  return (
    <canvas
      ref={canvasRef}
      width={RENDER_CONFIG.width}
      height={RENDER_CONFIG.height}
      className="w-full border border-neutral-700 rounded bg-black"
    />
  );
}

function renderSpectrogram(
  canvas: HTMLCanvasElement,
  data: SpectrogramData,
  metadata: RenderMetadata
): void {
  const {
    width,
    height,
    metadataHeight,
    minDb,
    maxDb,
    minFreqHz,
    maxFreqHz,
    magnitudeEpsilon,
  } = RENDER_CONFIG;

  const ctx = canvas.getContext('2d', { alpha: false });
  if (!ctx) {
    throw new Error('SpectrogramView: 2D context unavailable');
  }

  const specHeight = height - metadataHeight;
  const { magnitudes, numFrames, numBins, sampleRate } = data;
  const binFreqHz = sampleRate / (2 * numBins); // Nyquist / numBins
  const dbRange = maxDb - minDb;
  const logFreqRatio = Math.log(maxFreqHz / minFreqHz);

  // -- Time-axis: max-pool magnitudes from each column's frame range
  // into a flat per-column buffer. Preserves transient peaks that
  // floor()-sampling alone would skip when numFrames > width.
  const pooled = new Float32Array(width * numBins);
  for (let x = 0; x < width; x++) {
    const fStart = Math.floor((x * numFrames) / width);
    const fEnd = Math.max(fStart + 1, Math.floor(((x + 1) * numFrames) / width));
    const colOffset = x * numBins;
    // Seed column with the first frame's bins.
    const seedFrameOffset = fStart * numBins;
    for (let b = 0; b < numBins; b++) {
      pooled[colOffset + b] = magnitudes[seedFrameOffset + b] as number;
    }
    // Max-merge the rest of the frames in this column's range.
    for (let f = fStart + 1; f < fEnd; f++) {
      const frameOffset = f * numBins;
      for (let b = 0; b < numBins; b++) {
        const m = magnitudes[frameOffset + b] as number;
        if (m > (pooled[colOffset + b] as number)) {
          pooled[colOffset + b] = m;
        }
      }
    }
  }

  // -- Frequency-axis: pre-compute fractional bin index for each output
  // row so the inner render loop can linearly interpolate between the
  // two adjacent FFT bins. Smooths low-freq terracing and high-freq
  // banding inherent in nearest-bin lookup on a log axis.
  const binLoForRow = new Int32Array(specHeight);
  const binFracForRow = new Float32Array(specHeight);
  for (let y = 0; y < specHeight; y++) {
    const yNorm = 1 - y / (specHeight - 1); // 0 at bottom-y, 1 at top-y
    const freq = minFreqHz * Math.exp(yNorm * logFreqRatio);
    const binFloat = freq / binFreqHz;
    const lo = Math.max(0, Math.min(numBins - 2, Math.floor(binFloat)));
    binLoForRow[y] = lo;
    binFracForRow[y] = Math.max(0, Math.min(1, binFloat - lo));
  }

  // -- Render. ImageData buffer covers the full canvas; spectrogram
  // pixels go in rows [0, specHeight), the metadata band is drawn
  // afterwards on top with regular 2D ops.
  const image = ctx.createImageData(width, height);
  const px = image.data;

  for (let x = 0; x < width; x++) {
    const colOffset = x * numBins;

    for (let y = 0; y < specHeight; y++) {
      const lo = binLoForRow[y] as number;
      const frac = binFracForRow[y] as number;
      const magLo = pooled[colOffset + lo] as number;
      const magHi = pooled[colOffset + lo + 1] as number;
      const mag = magLo + frac * (magHi - magLo);

      const db = 20 * Math.log10(mag + magnitudeEpsilon);
      const norm = Math.max(0, Math.min(1, (db - minDb) / dbRange));

      const [r, g, b] = viridis(norm);
      const i = (y * width + x) * 4;
      px[i] = r;
      px[i + 1] = g;
      px[i + 2] = b;
      px[i + 3] = 255;
    }
  }

  // Black-fill the footer band in the ImageData so the colormap doesn't
  // bleed when the canvas is scaled by CSS.
  for (let y = specHeight; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4;
      px[i] = 10;
      px[i + 1] = 10;
      px[i + 2] = 12;
      px[i + 3] = 255;
    }
  }

  ctx.putImageData(image, 0, 0);

  // Footer text: every parameter needed to recompute this picture.
  const footerLine =
    `SHA ${metadata.fileHash} · ${sampleRate} Hz · ` +
    `${FFT_CONFIG.fftSize} fft · ${FFT_CONFIG.hopSize} hop · Hann · ` +
    `[${minDb}, ${maxDb}] dBFS · ` +
    `${minFreqHz}–${maxFreqHz} Hz · ` +
    `${numFrames} frames · ${metadata.durationSec.toFixed(2)} s`;
  ctx.fillStyle = '#9ca3af';
  ctx.font = '11px ui-monospace, SFMono-Regular, Menlo, monospace';
  ctx.textBaseline = 'middle';
  ctx.fillText(footerLine, 8, specHeight + metadataHeight / 2);
}
