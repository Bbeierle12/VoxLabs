import { describe, it, expect } from 'vitest';
// Vite ?raw import: the worklet source as a string, no node:fs needed.
import source from '../../public/mic-worklet.js?raw';

/**
 * String-level contract test for the AudioWorklet processor.
 *
 * `public/mic-worklet.js` runs in the audio rendering thread and can't
 * be imported here (AudioWorkletProcessor doesn't exist outside that
 * scope), and it sits outside the typed worker protocol. So we pin the
 * contract MicPipeline depends on at the source level: the processor
 * name it registers under, and that it posts a transferred Float32Array
 * copy (posting the runtime's reused buffer would hand the main thread
 * stale data). Cheap, and catches renames or a dropped copy/transfer.
 */
describe('mic-worklet source contract', () => {
  it("registers under the name MicPipeline instantiates ('mic-capture')", () => {
    expect(source).toMatch(/registerProcessor\(\s*'mic-capture'/);
  });

  it('posts a fresh Float32Array copy, transferred', () => {
    expect(source).toMatch(/new Float32Array\(channel\.length\)/);
    expect(source).toMatch(/copy\.set\(channel\)/);
    expect(source).toMatch(/postMessage\(copy,\s*\[copy\.buffer\]\)/);
  });

  it('keeps the processor alive by returning true', () => {
    expect(source).toMatch(/return true;\s*\}\s*\}/);
  });
});
