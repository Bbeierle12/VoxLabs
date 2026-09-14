// Mic capture worklet.
//
// Runs in the audio rendering thread. The process() callback fires every
// 128 samples (~2.9ms at 44.1kHz). We copy the input channel's samples
// — the underlying Float32Array is reused by the runtime, so posting
// the raw reference would let the main thread read stale data — and
// post the copy to the main thread.
//
// One worklet instance is created per AudioContext per mic session.
// Disconnect from the main thread to stop; we always return true while
// active.
class MicCaptureProcessor extends AudioWorkletProcessor {
  process(inputs) {
    const input = inputs[0];
    if (!input || input.length === 0) return true;
    const channel = input[0];
    if (!channel || channel.length === 0) return true;

    // Copy out — the runtime reuses these buffers across process() calls.
    const copy = new Float32Array(channel.length);
    copy.set(channel);
    this.port.postMessage(copy, [copy.buffer]);
    return true;
  }
}

registerProcessor('mic-capture', MicCaptureProcessor);
