//! Stream format: the analysis frame every CPU and GPU consumer runs on.

use super::stage_config;

stage_config! {
    /// Frame geometry shared by `frame::FrameAnalyzer` and the desktop
    /// `analysis::AnalysisEngine`.
    pub struct StreamConfig, section = "stream" {
        /// Mono samples per analysis frame (2048 @ 48 kHz = 42.7 ms). Must
        /// hold `yin.window + sample_rate / yin.f0_min_hz`. Range: power of
        /// two, 1024..=8192; the GPU shaders are sized from it.
        frame_samples: usize = 2048,
        /// Length of the GPU difference-function / prefix-sum buffers, in
        /// lags: a power of two, ≤ frame_samples − yin.window, ≥ sample_rate /
        /// yin.f0_min_hz. `yin_scan.wgsl`'s SCAN_LEN must match.
        /// Range: exactly 1024 unless the shaders change.
        gpu_diff_len: usize = 1024,
        /// Workgroup size of the GPU difference pass (`yin_diff.wgsl`).
        /// Range: exactly 64 unless the shader changes.
        gpu_diff_workgroup: u32 = 64,
    }
}
