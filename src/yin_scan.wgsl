// YIN CMND denominator: inclusive parallel prefix-sum of the difference function.
//
// Single-workgroup Hillis-Steele INCLUSIVE scan over SCAN_LEN lags. Reads d(tau)
// from `data_in` and writes the inclusive cumulative sum
//     cumsum[tau] = sum_{j=0..=tau} d(j)
// into a SEPARATE `scan_out` buffer (never in-place — the host still needs the
// raw d(tau) to form d'(tau) = d(tau) * tau / cumsum[tau]).
//
// 256 invocations each own 4 strided elements (256 * 4 = 1024 = SCAN_LEN), so a
// single workgroup covers the whole lag range with NO cross-block reduction pass
// — sidestepping the multi-block hazard that made the original scan incorrect.
// Each Hillis-Steele step gathers all predecessors into registers, barriers,
// then writes, so every read observes the previous step's values (no in-place
// race). SCAN_LEN must equal the host DIFF_LEN.

@group(0) @binding(0) var<storage, read> data_in: array<f32>;
@group(0) @binding(1) var<storage, read_write> scan_out: array<f32>;

const SCAN_LEN: u32 = 1024u;
const THREADS: u32 = 256u;
const PER_THREAD: u32 = 4u; // SCAN_LEN / THREADS

var<workgroup> temp: array<f32, 1024>;

@compute @workgroup_size(256)
fn compute_scan(@builtin(local_invocation_id) lid: vec3<u32>) {
    let t = lid.x;
    let n = arrayLength(&data_in);

    // Cooperative load: thread t loads indices t, t+256, t+512, t+768.
    for (var k = 0u; k < PER_THREAD; k = k + 1u) {
        let idx = t + k * THREADS;
        if (idx < SCAN_LEN) {
            temp[idx] = select(0.0, data_in[idx], idx < n);
        }
    }
    workgroupBarrier();

    // Hillis-Steele inclusive scan: log2(SCAN_LEN) = 10 passes.
    for (var offset = 1u; offset < SCAN_LEN; offset = offset << 1u) {
        let i0 = t;
        let i1 = t + THREADS;
        let i2 = t + 2u * THREADS;
        let i3 = t + 3u * THREADS;

        // Read phase — gather predecessors before anyone writes this pass.
        var add0 = 0.0;
        var add1 = 0.0;
        var add2 = 0.0;
        var add3 = 0.0;
        if (i0 >= offset) { add0 = temp[i0 - offset]; }
        if (i1 >= offset) { add1 = temp[i1 - offset]; }
        if (i2 >= offset) { add2 = temp[i2 - offset]; }
        if (i3 >= offset) { add3 = temp[i3 - offset]; }
        workgroupBarrier();

        // Write phase.
        if (i0 >= offset) { temp[i0] = temp[i0] + add0; }
        if (i1 >= offset) { temp[i1] = temp[i1] + add1; }
        if (i2 >= offset) { temp[i2] = temp[i2] + add2; }
        if (i3 >= offset) { temp[i3] = temp[i3] + add3; }
        workgroupBarrier();
    }

    // Store inclusive results.
    for (var k = 0u; k < PER_THREAD; k = k + 1u) {
        let idx = t + k * THREADS;
        if (idx < SCAN_LEN && idx < arrayLength(&scan_out)) {
            scan_out[idx] = temp[idx];
        }
    }
}
