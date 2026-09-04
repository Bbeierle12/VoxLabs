//! In-app file import: an audio file goes through the same
//! `frame::FrameAnalyzer` a live capture does, on a worker thread, and its
//! per-frame profiles stream back to the UI, which feeds them into the
//! capture state machine exactly as if the microphone had produced them.
//! The result card, the voiceprint match, the session archive and the
//! sidecar export are all the live ones — a file is just another capture.
//!
//! Files come from the import folder (`<data>/import/`, filled by `adb
//! push`, a file manager, or the Android share sheet) and are listed on
//! the Sessions screen.
//!
//! The web build has no files; it gets an inert stub with the same API so
//! the UI compiles unchanged.

use crate::types::VocalProfile;
use std::path::{Path, PathBuf};

/// One file in the import folder.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub path: PathBuf,
    pub name: String,
    pub bytes: u64,
}

/// What the worker sends back, in order: one `Meta`, then a `Profile` per
/// analysis frame, then `Done` — or `Error` at any point. The profile is
/// boxed: it is ~250 bytes against ~30 for the rest, and each one crosses
/// a channel.
#[derive(Clone, Debug)]
pub enum Msg {
    Meta {
        frames: usize,
        seconds: f32,
        sr_native: u32,
        peak_dbfs: f32,
        clipped: usize,
    },
    Profile(Box<VocalProfile>),
    Done,
    Error(String),
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::{Entry, Msg};
    use crate::audio_file;
    use crate::frame::{ANALYSIS_FRAME, FrameAnalyzer};
    use std::path::{Path, PathBuf};
    use std::sync::mpsc::{Receiver, TryRecvError, channel};

    /// Audio files in the import folder, newest first.
    pub fn list(dir: &Path) -> Vec<Entry> {
        audio_file::list_dir(dir)
            .into_iter()
            .map(|e| Entry {
                path: e.path,
                name: e.name,
                bytes: e.bytes,
            })
            .collect()
    }

    /// A running (or finished) import. Dropping it cancels the worker: its
    /// next send fails and it returns.
    pub struct Job {
        pub name: String,
        pub path: PathBuf,
        rx: Receiver<Msg>,
        pub frames: usize,
        pub done: usize,
        pub seconds: f32,
        pub sr_native: u32,
        pub peak_dbfs: f32,
        pub clipped: usize,
    }

    impl Job {
        /// Decode `path` and analyze it at `sample_rate` on a worker thread.
        pub fn start(path: PathBuf, sample_rate: f32) -> Job {
            let (tx, rx) = channel();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let worker_path = path.clone();
            std::thread::Builder::new()
                .name("voxlabs-import".into())
                .spawn(move || {
                    let dec = match audio_file::decode(&worker_path) {
                        Ok(d) => d,
                        Err(e) => {
                            let _ = tx.send(Msg::Error(format!("could not decode: {e}")));
                            return;
                        }
                    };
                    let sr_native = dec.sample_rate;
                    let seconds = dec.seconds();
                    let samples = audio_file::resample(&dec.samples, sr_native as f32, sample_rate);
                    drop(dec);
                    let mut peak = 0.0f32;
                    let mut clipped = 0usize;
                    for &s in &samples {
                        let a = s.abs();
                        if a > peak {
                            peak = a;
                        }
                        if a >= 0.999 {
                            clipped += 1;
                        }
                    }
                    let frames = samples.len() / ANALYSIS_FRAME;
                    if frames == 0 {
                        let _ = tx.send(Msg::Error("file shorter than one analysis frame".into()));
                        return;
                    }
                    if tx
                        .send(Msg::Meta {
                            frames,
                            seconds,
                            sr_native,
                            peak_dbfs: 20.0 * peak.max(1e-9).log10(),
                            clipped,
                        })
                        .is_err()
                    {
                        return;
                    }
                    let mut analyzer = FrameAnalyzer::new(sample_rate);
                    for frame in samples.chunks_exact(ANALYSIS_FRAME) {
                        let r = analyzer.analyze(frame);
                        if tx.send(Msg::Profile(Box::new(r.profile))).is_err() {
                            return; // cancelled
                        }
                    }
                    let _ = tx.send(Msg::Done);
                })
                .expect("spawn import worker");
            Job {
                name,
                path,
                rx,
                frames: 0,
                done: 0,
                seconds: 0.0,
                sr_native: 0,
                peak_dbfs: -120.0,
                clipped: 0,
            }
        }

        /// Next message, if one is waiting. `Meta` is folded into the job's
        /// own fields and returned as well; `Profile` bumps `done`.
        pub fn try_recv(&mut self) -> Option<Msg> {
            match self.rx.try_recv() {
                Ok(m) => {
                    match &m {
                        Msg::Meta {
                            frames,
                            seconds,
                            sr_native,
                            peak_dbfs,
                            clipped,
                        } => {
                            self.frames = *frames;
                            self.seconds = *seconds;
                            self.sr_native = *sr_native;
                            self.peak_dbfs = *peak_dbfs;
                            self.clipped = *clipped;
                        }
                        Msg::Profile(_) => self.done += 1,
                        _ => {}
                    }
                    Some(m)
                }
                Err(TryRecvError::Empty) => None,
                // Worker gone without `Done`: it panicked or was killed.
                Err(TryRecvError::Disconnected) => Some(Msg::Error("import worker stopped".into())),
            }
        }

        /// 0..1 progress through the file's frames (0 until `Meta` arrives).
        pub fn progress(&self) -> f32 {
            if self.frames == 0 {
                0.0
            } else {
                (self.done as f32 / self.frames as f32).min(1.0)
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod native {
    use super::{Entry, Msg};
    use std::path::{Path, PathBuf};

    pub fn list(_dir: &Path) -> Vec<Entry> {
        Vec::new()
    }

    pub struct Job {
        pub name: String,
        pub path: PathBuf,
        pub frames: usize,
        pub done: usize,
        pub seconds: f32,
        pub sr_native: u32,
        pub peak_dbfs: f32,
        pub clipped: usize,
    }

    impl Job {
        pub fn start(path: PathBuf, _sample_rate: f32) -> Job {
            Job {
                name: String::new(),
                path,
                frames: 0,
                done: 0,
                seconds: 0.0,
                sr_native: 0,
                peak_dbfs: -120.0,
                clipped: 0,
            }
        }

        pub fn try_recv(&mut self) -> Option<Msg> {
            Some(Msg::Error("no file import on web".into()))
        }

        pub fn progress(&self) -> f32 {
            0.0
        }
    }
}

pub use native::{Job, list};

/// Human-readable size for the import list ("1.2 MB").
pub fn size_label(bytes: u64) -> String {
    if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1e6)
    } else {
        format!("{} kB", bytes.div_ceil(1000))
    }
}

/// The name a session stores for an imported file, distinguishable from a
/// live capture's export (`capture-….wav`).
pub fn session_file_name(path: &Path) -> String {
    format!(
        "import/{}",
        path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    )
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn a_wav_streams_its_frames_then_done() {
        let dir = std::env::temp_dir().join(format!("voxlabs-import-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("vowel.wav");
        let sr = 48_000u32;
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: sr,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut w = hound::WavWriter::create(&path, spec).unwrap();
        // 1 s harmonic-rich 150 Hz vowel: every frame should be voiced.
        for i in 0..sr as usize {
            let t = i as f32 / sr as f32;
            let s: f32 = (1i32..=10)
                .map(|k| {
                    0.6f32.powi(k - 1) * (2.0 * std::f32::consts::PI * 150.0 * k as f32 * t).sin()
                })
                .sum::<f32>()
                * 0.2;
            w.write_sample(s).unwrap();
        }
        w.finalize().unwrap();

        let mut job = Job::start(path.clone(), 48_000.0);
        assert_eq!(job.name, "vowel.wav");
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut profiles = 0;
        let mut done = false;
        while Instant::now() < deadline && !done {
            match job.try_recv() {
                Some(Msg::Profile(p)) => {
                    profiles += 1;
                    assert!(p.valid, "frame {profiles} unvoiced");
                    assert!((p.f0 - 150.0).abs() < 3.0, "f0 {}", p.f0);
                }
                Some(Msg::Done) => done = true,
                Some(Msg::Error(e)) => panic!("{e}"),
                Some(Msg::Meta { .. }) | None => std::thread::sleep(Duration::from_millis(2)),
            }
        }
        assert!(done, "worker did not finish");
        assert_eq!(job.frames, 48_000 / 2048);
        assert_eq!(profiles, job.frames);
        assert_eq!(job.done, job.frames);
        assert!((job.seconds - 1.0).abs() < 1e-3);
        assert_eq!(job.sr_native, sr);
        assert_eq!(job.clipped, 0);
        assert!((job.progress() - 1.0).abs() < 1e-6);

        let l = list(&dir);
        assert_eq!(l.len(), 1);
        assert_eq!(l[0].name, "vowel.wav");
        assert_eq!(session_file_name(&path), "import/vowel.wav");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_bad_file_reports_an_error() {
        let dir = std::env::temp_dir().join(format!("voxlabs-import-bad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("junk.mp3");
        std::fs::write(&path, b"this is not audio").unwrap();
        let mut job = Job::start(path, 48_000.0);
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match job.try_recv() {
                Some(Msg::Error(_)) => break,
                Some(other) => panic!("unexpected {other:?}"),
                None if Instant::now() > deadline => panic!("no error"),
                None => std::thread::sleep(Duration::from_millis(2)),
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn size_labels() {
        assert_eq!(size_label(999), "1 kB");
        assert_eq!(size_label(1_500_000), "1.5 MB");
    }
}
