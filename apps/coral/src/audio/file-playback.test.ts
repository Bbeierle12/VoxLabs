import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { FilePlayback } from './file-playback';
import { PLAYBACK_CONFIG } from '../config/playback';

/**
 * Tick pacing under a mocked clock + rAF pump. The contract under test:
 * each tick ships exactly the audio elapsed since the last tick, EXCEPT
 * when the gap exceeds the per-tick cap (backgrounded tab — rAF was
 * suspended while performance.now() ran). Then the clock re-anchors and
 * nothing ships, so the time axis stays honest instead of folding the
 * missed span into one burst.
 */

// 1 kHz sample rate keeps the ms→samples math readable: 1 sample per ms.
const SR = 1000;

function fakeAudioBuffer(lengthSamples: number): AudioBuffer {
  const data = new Float32Array(lengthSamples);
  return {
    numberOfChannels: 1,
    length: lengthSamples,
    sampleRate: SR,
    duration: lengthSamples / SR,
    getChannelData: () => data,
  } as unknown as AudioBuffer;
}

class FakeWorker {
  posted: { type: string; samples?: Float32Array }[] = [];
  onmessage: ((ev: unknown) => void) | null = null;
  onerror: ((ev: unknown) => void) | null = null;
  postMessage(msg: { type: string; samples?: Float32Array }): void {
    this.posted.push(msg);
  }
  terminate(): void {}
}

let nowMs = 0;
let pendingRaf: FrameRequestCallback | null = null;

/** Advance the mocked clock and fire the pending rAF tick. */
function pump(advanceMs: number): void {
  nowMs += advanceMs;
  const cb = pendingRaf;
  pendingRaf = null;
  cb?.(nowMs);
}

function shippedSamples(worker: FakeWorker): number[] {
  return worker.posted
    .filter((m) => m.type === 'samples')
    .map((m) => m.samples!.length);
}

describe('FilePlayback tick pacing', () => {
  let workers: FakeWorker[];

  beforeEach(() => {
    nowMs = 0;
    pendingRaf = null;
    workers = [];
    vi.stubGlobal(
      'Worker',
      class {
        constructor() {
          const w = new FakeWorker();
          workers.push(w);
          return w;
        }
      }
    );
    vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback): number => {
      pendingRaf = cb;
      return 1;
    });
    vi.stubGlobal('cancelAnimationFrame', () => {
      pendingRaf = null;
    });
    vi.spyOn(performance, 'now').mockImplementation(() => nowMs);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  function start(durationSamples: number): { fp: FilePlayback; worker: FakeWorker } {
    const fp = new FilePlayback();
    fp.start({
      buffer: fakeAudioBuffer(durationSamples),
      fftSize: 2048,
      hopSize: 512,
      minFreqHz: 50,
      maxFreqHz: 8000,
    });
    return { fp, worker: workers[0]! };
  }

  it('ships exactly the elapsed audio on normal ticks', () => {
    const { fp, worker } = start(SR * 10);
    pump(100); // 100 ms → 100 samples at 1 kHz
    pump(50);
    expect(shippedSamples(worker)).toEqual([100, 50]);
    fp.stop();
  });

  it('re-anchors instead of bursting after a background-tab gap', () => {
    const { fp, worker } = start(SR * 60);
    pump(100); // normal tick
    pump(10_000); // tab was backgrounded: gap >> cap → re-anchor, ship nothing
    pump(50); // resumed: normal pacing from the pause point
    const shipped = shippedSamples(worker);
    expect(shipped).toEqual([100, 50]);
    const cap = Math.floor(PLAYBACK_CONFIG.maxTickAdvanceSec * SR);
    for (const n of shipped) expect(n).toBeLessThanOrEqual(cap);
    fp.stop();
  });

  it('never ships more than the per-tick cap in any single message', () => {
    const { fp, worker } = start(SR * 60);
    // A jittery sequence including several over-cap gaps.
    for (const gap of [16, 16, 400, 16, 1000, 33, 250, 16, 5000, 16]) pump(gap);
    const cap = Math.floor(PLAYBACK_CONFIG.maxTickAdvanceSec * SR);
    for (const n of shippedSamples(worker)) expect(n).toBeLessThanOrEqual(cap);
    fp.stop();
  });

  it('still completes playback after a re-anchor', () => {
    const onComplete = vi.fn();
    const fp = new FilePlayback({ onComplete });
    fp.start({
      buffer: fakeAudioBuffer(300), // 300 ms of audio
      fftSize: 2048,
      hopSize: 512,
      minFreqHz: 50,
      maxFreqHz: 8000,
    });
    pump(100);
    pump(10_000); // re-anchor
    pump(100);
    pump(100);
    pump(100); // cursor reaches the end
    expect(onComplete).toHaveBeenCalledTimes(1);
  });
});
