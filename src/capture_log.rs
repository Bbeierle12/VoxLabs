//! Raw-audio export of a capture: while a recording runs, every analysis
//! frame the engine consumes is also appended to a WAV file, so the phone's
//! capture can be run through the `voxlab` study harness exactly like any
//! dataset file — same samples, same `FrameAnalyzer`, same tables.
//!
//! The sink is a process-wide slot: the UI arms it when a capture starts
//! and disarms it when the capture stops; the analysis thread (desktop GPU
//! loop or Android CPU loop) calls [`push`] on each frame it analyzes.
//! `push` is a single relaxed atomic load when nothing is armed, so the
//! cost to the analysis thread outside a capture is nil.
//!
//! Format: 32-bit float mono WAV at the engine's input rate. Float keeps the
//! export bit-identical to what the analyzer saw (a study tool should not
//! quantize its own evidence); the cost is ~11.5 MB/min at 48 kHz, which
//! the one-minute capture cap keeps bounded.

use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// Set while a sink is open; the fast-path check in [`push`].
static ARMED: AtomicBool = AtomicBool::new(false);
static SINK: Mutex<Option<Sink>> = Mutex::new(None);

struct Sink {
    writer: hound::WavWriter<BufWriter<File>>,
    path: PathBuf,
    sample_rate: u32,
    samples: u64,
    peak: f32,
    clipped: usize,
}

/// What a finished capture file looks like, for the UI and the archive.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptureFile {
    pub path: PathBuf,
    pub seconds: f32,
    pub peak_dbfs: f32,
    pub clipped: usize,
}

impl CaptureFile {
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}

/// File name for a capture started now: `capture-YYYYMMDD-HHMMSS.wav`,
/// local time, so files sort chronologically and match the session clock.
pub fn capture_file_name() -> String {
    let now = chrono::Local::now();
    format!("capture-{}.wav", now.format("%Y%m%d-%H%M%S"))
}

/// Open a new capture file under `dir` (created if missing) and start
/// mirroring analysis frames into it. Arming while a sink is already open
/// finalizes the old one first (returned to nobody: the UI never does this).
pub fn arm(dir: &Path, sample_rate: u32) -> std::io::Result<PathBuf> {
    let _ = disarm();
    std::fs::create_dir_all(dir)?;
    let path = dir.join(capture_file_name());
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let writer = hound::WavWriter::create(&path, spec).map_err(hound_io)?;
    let mut slot = SINK.lock().unwrap_or_else(|e| e.into_inner());
    *slot = Some(Sink {
        writer,
        path: path.clone(),
        sample_rate,
        samples: 0,
        peak: 0.0,
        clipped: 0,
    });
    ARMED.store(true, Ordering::Release);
    Ok(path)
}

/// Append one analysis frame. A no-op unless armed. Write errors disarm
/// the sink (and are logged) rather than being reported per frame.
pub fn push(frame: &[f32]) {
    if !ARMED.load(Ordering::Relaxed) {
        return;
    }
    let mut slot = SINK.lock().unwrap_or_else(|e| e.into_inner());
    let Some(sink) = slot.as_mut() else {
        return;
    };
    for &s in frame {
        let a = s.abs();
        if a > sink.peak {
            sink.peak = a;
        }
        if a >= 0.999 {
            sink.clipped += 1;
        }
        if let Err(e) = sink.writer.write_sample(s) {
            log::error!("capture export: write failed ({e}); export stopped");
            ARMED.store(false, Ordering::Release);
            *slot = None;
            return;
        }
    }
    sink.samples += frame.len() as u64;
}

/// Finalize the open capture file, if any, and describe it.
pub fn disarm() -> Option<CaptureFile> {
    ARMED.store(false, Ordering::Release);
    let sink = SINK.lock().unwrap_or_else(|e| e.into_inner()).take()?;
    let Sink {
        writer,
        path,
        sample_rate,
        samples,
        peak,
        clipped,
    } = sink;
    if let Err(e) = writer.finalize() {
        log::error!("capture export: finalize failed ({e})");
        return None;
    }
    Some(CaptureFile {
        path,
        seconds: samples as f32 / sample_rate.max(1) as f32,
        peak_dbfs: 20.0 * peak.max(1e-9).log10(),
        clipped,
    })
}

pub fn is_armed() -> bool {
    ARMED.load(Ordering::Relaxed)
}

fn hound_io(e: hound::Error) -> std::io::Error {
    match e {
        hound::Error::IoError(io) => io,
        other => std::io::Error::other(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The sink is process-global, so the tests here run under one lock to
    // keep them from arming over each other.
    static SERIAL: Mutex<()> = Mutex::new(());

    #[test]
    fn push_is_a_no_op_when_disarmed() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        assert!(disarm().is_none());
        push(&[0.5; 64]);
        assert!(!is_armed());
        assert!(disarm().is_none());
    }

    #[test]
    fn armed_frames_land_in_a_wav_the_decoder_reads_back() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("voxlabs-capture-log-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = arm(&dir, 48_000).expect("arm");
        assert!(is_armed());
        assert!(path.starts_with(&dir));
        let frame: Vec<f32> = (0..2048).map(|i| (i as f32 / 2047.0) - 0.5).collect();
        push(&frame);
        push(&frame);
        let done = disarm().expect("capture file");
        assert!(!is_armed());
        assert_eq!(done.path, path);
        assert!((done.seconds - 4096.0 / 48_000.0).abs() < 1e-6);
        assert!((done.peak_dbfs - 20.0 * 0.5f32.log10()).abs() < 0.01);
        assert_eq!(done.clipped, 0);

        let mut r = hound::WavReader::open(&path).expect("open");
        assert_eq!(r.spec().sample_rate, 48_000);
        assert_eq!(r.spec().channels, 1);
        let back: Vec<f32> = r.samples::<f32>().map(|s| s.unwrap()).collect();
        assert_eq!(back.len(), 4096);
        assert_eq!(&back[..2048], &frame[..]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clipping_is_counted() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("voxlabs-capture-clip-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        arm(&dir, 44_100).expect("arm");
        push(&[0.2, 1.0, -1.0, 0.9995, 0.1]);
        let done = disarm().unwrap();
        assert_eq!(done.clipped, 3);
        assert!(done.peak_dbfs.abs() < 0.01);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_names_sort_chronologically() {
        let n = capture_file_name();
        assert!(n.starts_with("capture-") && n.ends_with(".wav"));
        assert_eq!(n.len(), "capture-YYYYMMDD-HHMMSS.wav".len());
    }
}
