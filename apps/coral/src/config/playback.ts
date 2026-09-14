/**
 * File-playback pacing parameters.
 */
export const PLAYBACK_CONFIG = {
  /**
   * Maximum audio that one rAF tick may ship to the DSP worker, in
   * seconds. Normal ticks advance ~16 ms; a gap larger than this means
   * the clock ran while ticks didn't (backgrounded tab — rAF suspends
   * but performance.now() doesn't). Shipping the whole gap would burst
   * N seconds of frames that the renderer max-pools into ~one row,
   * silently corrupting the time axis. Instead we re-anchor the clock
   * and resume from the cursor, so backgrounding behaves as a pause.
   */
  maxTickAdvanceSec: 0.25,
} as const;
