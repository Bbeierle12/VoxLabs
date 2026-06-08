// YIN Difference Function Compute Shader.
//
// One invocation per lag `tau`, computing
//   d(tau) = sum_{j=0}^{WINDOW_SIZE-1} (audio_in[j] - audio_in[j+tau])^2.
//
// SIZING CONTRACT (this was the original bug): `audio_in` must hold at least
// WINDOW_SIZE + max(tau) samples so `j + tau` never leaves the array. The host
// sizes audio_in to ANALYSIS_FRAME (2048) and diff_out to DIFF_LEN (1024, a
// power of two for the prefix-sum scan), so the largest tau is 1023 and the
// largest read is x[1023 + 1023] = x[2046] — in-bounds. (Previously audio_in
// was only 1024 long, so the inner bounds-check silently dropped terms and
// biased the high lags.)

@group(0) @binding(0) var<storage, read> audio_in: array<f32>;
@group(0) @binding(1) var<storage, read_write> diff_out: array<f32>;

const WINDOW_SIZE: u32 = 1024u; // must equal host YIN_WINDOW

@compute @workgroup_size(64)
fn compute_diff(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tau = global_id.x;
    
    // Bounds check
    if (tau >= arrayLength(&diff_out)) {
        return;
    }

    var diff_sum: f32 = 0.0;
    
    // Compute squared difference for this lag (tau)
    for (var j = 0u; j < WINDOW_SIZE; j = j + 1u) {
        let idx1 = j;
        let idx2 = j + tau;
        
        // Ensure we don't read out of bounds on the input array
        if (idx2 < arrayLength(&audio_in)) {
            let diff = audio_in[idx1] - audio_in[idx2];
            diff_sum = diff_sum + diff * diff;
        }
    }
    
    diff_out[tau] = diff_sum;
}
