import { log } from '../utils/log';

/**
 * Decode an audio File (WAV, MP3, FLAC, OGG — whatever the browser
 * supports) into an AudioBuffer. Uses a one-shot AudioContext that's
 * closed immediately so we don't leak audio resources.
 *
 * PROVENANCE CAVEAT: decodeAudioData resamples to the AudioContext's
 * device rate, so `buffer.sampleRate` is the post-resample rate, NOT the
 * file's native rate (which Web Audio does not expose). The same file
 * yields different sample streams on 44.1 kHz vs 48 kHz machines —
 * don't treat a rendered Hz readout as file provenance.
 *
 * Throws with a useful message on every failure mode: not-a-file,
 * unreadable bytes, decoder rejection.
 */
export async function decodeAudioFile(file: File): Promise<AudioBuffer> {
  if (!(file instanceof File)) {
    throw new Error(
      `decodeAudioFile: expected File, got ${typeof file}: ${String(file).slice(0, 80)}`
    );
  }
  if (file.size === 0) {
    throw new Error(`decodeAudioFile: file "${file.name}" is empty`);
  }

  log.info('decode:start', { name: file.name, size: file.size, type: file.type });

  let arrayBuffer: ArrayBuffer;
  try {
    arrayBuffer = await file.arrayBuffer();
  } catch (err) {
    throw new Error(
      `decodeAudioFile: failed to read bytes from "${file.name}": ${(err as Error).message}`
    );
  }

  const ctx = new AudioContext();
  try {
    const buffer = await ctx.decodeAudioData(arrayBuffer);
    log.info('decode:ok', {
      name: file.name,
      channels: buffer.numberOfChannels,
      sampleRate: buffer.sampleRate,
      durationSec: buffer.duration,
    });
    return buffer;
  } catch (err) {
    throw new Error(
      `decodeAudioFile: browser rejected "${file.name}" — unsupported codec or corrupt data (${(err as Error).message})`
    );
  } finally {
    void ctx.close();
  }
}

/**
 * SHA-256 of the file's bytes, truncated to 8 hex characters. Used as a
 * short reproducibility ID stamped on the rendered spectrogram. Not a
 * cryptographic identifier — eight chars is ~32 bits, enough to tell
 * "did I change the source file" but not collision-resistant at scale.
 */
export async function hashFile(file: File): Promise<string> {
  const bytes = await file.arrayBuffer();
  const digest = await crypto.subtle.digest('SHA-256', bytes);
  const view = new Uint8Array(digest, 0, 4);
  let hex = '';
  for (let i = 0; i < view.length; i++) {
    hex += (view[i] as number).toString(16).padStart(2, '0');
  }
  return hex;
}

/**
 * Extract a mono Float32Array from an AudioBuffer.
 * If multi-channel, averages all channels. For v1 we'll switch to
 * per-channel extraction, but the walking skeleton is single-channel.
 */
export function toMonoSamples(buffer: AudioBuffer): Float32Array {
  const channels = buffer.numberOfChannels;
  if (channels === 0) {
    throw new Error('toMonoSamples: AudioBuffer has zero channels');
  }
  if (channels === 1) {
    return buffer.getChannelData(0);
  }

  const length = buffer.length;
  const out = new Float32Array(length);
  for (let c = 0; c < channels; c++) {
    const data = buffer.getChannelData(c);
    for (let i = 0; i < length; i++) {
      out[i] = (out[i] as number) + (data[i] as number);
    }
  }
  const inv = 1 / channels;
  for (let i = 0; i < length; i++) {
    out[i] = (out[i] as number) * inv;
  }
  return out;
}
