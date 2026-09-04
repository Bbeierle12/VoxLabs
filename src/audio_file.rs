//! Audio-file decoding for the app's file-import path and the `voxlab`
//! study harness: any container/codec symphonia handles (WAV, FLAC, MP3,
//! AAC in MP4/M4A, Vorbis/Ogg) to mono f32, plus a windowed-sinc resampler
//! to the engine rate. Native targets only (desktop + Android); the web
//! build has no filesystem.
//!
//! A file decoded here and pushed through `frame::FrameAnalyzer` produces
//! exactly what a live capture of the same audio would — that equivalence
//! is what lets a phone import, a desktop import, and a harness run share
//! one set of numbers.

use std::fs::File;
use std::path::{Path, PathBuf};

/// Extensions the import folder lists and the harness walks. Matching is
/// case-insensitive.
pub const AUDIO_EXTENSIONS: [&str; 7] = ["wav", "flac", "mp3", "m4a", "aac", "ogg", "oga"];

/// True for a path whose extension is one of [`AUDIO_EXTENSIONS`].
pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let e = e.to_ascii_lowercase();
            AUDIO_EXTENSIONS.contains(&e.as_str())
        })
        .unwrap_or(false)
}

/// One entry of an import-folder listing.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioEntry {
    pub path: PathBuf,
    pub name: String,
    pub bytes: u64,
    /// Modification time, seconds since the Unix epoch (0 if unknown).
    pub modified: u64,
}

/// Audio files directly inside `dir`, newest first. A missing directory is
/// an empty listing, not an error (the folder is created on first use).
pub fn list_dir(dir: &Path) -> Vec<AudioEntry> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<AudioEntry> = rd
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.path())
        .filter(|p| is_audio_file(p))
        .map(|path| {
            let meta = std::fs::metadata(&path).ok();
            AudioEntry {
                name: path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                bytes: meta.as_ref().map_or(0, |m| m.len()),
                modified: meta
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map_or(0, |d| d.as_secs()),
                path,
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.modified
            .cmp(&a.modified)
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

/// A path in `dir` for `name` that does not exist yet: `name`, else
/// `stem-2.ext`, `stem-3.ext`, … Used when a shared file is copied into the
/// import folder, so a second share of the same name never overwrites.
pub fn unique_path(dir: &Path, name: &str) -> PathBuf {
    let first = dir.join(name);
    if !first.exists() {
        return first;
    }
    let p = Path::new(name);
    let stem = p
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| name.to_string());
    let ext = p
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    (2..)
        .map(|n| dir.join(format!("{stem}-{n}{ext}")))
        .find(|c| !c.exists())
        .expect("unbounded")
}

/// Decoded audio: mono samples at the file's native rate.
#[derive(Clone, Debug)]
pub struct Decoded {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl Decoded {
    pub fn seconds(&self) -> f32 {
        self.samples.len() as f32 / self.sample_rate.max(1) as f32
    }
}

/// Decode any supported container/codec to mono f32 at its native rate.
/// Multi-channel audio is averaged to mono.
pub fn decode(path: &Path) -> anyhow::Result<Decoded> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
    use symphonia::core::errors::Error;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| anyhow::anyhow!("no decodable audio track"))?;
    let track_id = track.id;
    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| anyhow::anyhow!("unknown sample rate"))?;
    let mut decoder =
        symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;
    let mut samples: Vec<f32> = Vec::new();
    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(Error::ResetRequired) => break,
            Err(e) => return Err(e.into()),
        };
        if packet.track_id() != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                let ch = spec.channels.count().max(1);
                let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
                buf.copy_interleaved_ref(decoded);
                for fr in buf.samples().chunks(ch) {
                    samples.push(fr.iter().sum::<f32>() / ch as f32);
                }
            }
            // A corrupt packet (common at MP3 boundaries) is skipped, not fatal.
            Err(Error::DecodeError(_)) => continue,
            Err(e) => return Err(e.into()),
        }
    }
    if samples.is_empty() {
        anyhow::bail!("no audio decoded");
    }
    Ok(Decoded {
        samples,
        sample_rate,
    })
}

/// Windowed-sinc resampler (Hann window, 32 taps each side, cutoff at
/// 0.45 × the lower rate). Identity when the rates already match.
pub fn resample(x: &[f32], sr_in: f32, sr_out: f32) -> Vec<f32> {
    if (sr_in - sr_out).abs() < 0.5 || x.is_empty() {
        return x.to_vec();
    }
    let ratio = sr_out as f64 / sr_in as f64;
    let n_out = (x.len() as f64 * ratio).floor() as usize;
    let fc = 0.45 * sr_in.min(sr_out) / sr_in; // cycles per input sample
    let taps: isize = 32;
    let mut out = Vec::with_capacity(n_out);
    for i in 0..n_out {
        let t = i as f64 / ratio;
        let c = t.floor() as isize;
        let frac = (t - c as f64) as f32;
        let mut acc = 0.0f32;
        for k in -taps..=taps {
            let idx = c + k;
            if idx < 0 || idx >= x.len() as isize {
                continue;
            }
            let d = k as f32 - frac; // sample offset from t
            let arg = 2.0 * fc * d;
            let sinc = if arg.abs() < 1e-6 {
                1.0
            } else {
                (std::f32::consts::PI * arg).sin() / (std::f32::consts::PI * arg)
            };
            let w = 0.5 + 0.5 * (std::f32::consts::PI * d / taps as f32).cos();
            acc += x[idx as usize] * 2.0 * fc * sinc * w;
        }
        out.push(acc);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("voxlabs-audio-file-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn extension_filter_is_case_insensitive() {
        assert!(is_audio_file(Path::new("a/b/take.WAV")));
        assert!(is_audio_file(Path::new("song.m4a")));
        assert!(!is_audio_file(Path::new("notes.txt")));
        assert!(!is_audio_file(Path::new("noext")));
    }

    #[test]
    fn listing_is_newest_first_and_skips_non_audio() {
        let d = tmp("list");
        std::fs::write(d.join("old.wav"), b"x").unwrap();
        std::fs::write(d.join("readme.md"), b"x").unwrap();
        // Force a later mtime on the second file without sleeping.
        std::fs::write(d.join("new.flac"), b"xx").unwrap();
        let later = std::time::SystemTime::now() + std::time::Duration::from_secs(5);
        std::fs::File::options()
            .write(true)
            .open(d.join("new.flac"))
            .unwrap()
            .set_modified(later)
            .unwrap();
        let l = list_dir(&d);
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].name, "new.flac");
        assert_eq!(l[0].bytes, 2);
        assert_eq!(l[1].name, "old.wav");
        assert!(list_dir(&d.join("missing")).is_empty());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn unique_path_never_overwrites() {
        let d = tmp("unique");
        assert_eq!(unique_path(&d, "take.wav"), d.join("take.wav"));
        std::fs::write(d.join("take.wav"), b"x").unwrap();
        assert_eq!(unique_path(&d, "take.wav"), d.join("take-2.wav"));
        std::fs::write(d.join("take-2.wav"), b"x").unwrap();
        assert_eq!(unique_path(&d, "take.wav"), d.join("take-3.wav"));
        assert_eq!(unique_path(&d, "noext"), d.join("noext"));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn wav_roundtrip_through_the_decoder() {
        let d = tmp("wav");
        let path = d.join("tone.wav");
        let sr = 44_100u32;
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: sr,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&path, spec).unwrap();
        let n = sr as usize; // 1 s
        for i in 0..n {
            let s = (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sr as f32).sin() * 0.5;
            let v = (s * i16::MAX as f32) as i16;
            w.write_sample(v).unwrap(); // L
            w.write_sample(v).unwrap(); // R (downmixed to the same mono)
        }
        w.finalize().unwrap();
        let dec = decode(&path).expect("decode");
        assert_eq!(dec.sample_rate, sr);
        assert_eq!(dec.samples.len(), n);
        assert!((dec.seconds() - 1.0).abs() < 1e-3);
        let peak = dec.samples.iter().fold(0.0f32, |m, &s| m.max(s.abs()));
        assert!((peak - 0.5).abs() < 0.01, "peak {peak}");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn resample_preserves_a_tone() {
        let sr_in = 44_100.0;
        let sr_out = 48_000.0;
        let x: Vec<f32> = (0..44_100)
            .map(|i| (2.0 * std::f32::consts::PI * 220.0 * i as f32 / sr_in).sin())
            .collect();
        let y = resample(&x, sr_in, sr_out);
        assert!((y.len() as f32 - 48_000.0).abs() < 2.0);
        // Away from the edges, the resampled tone has the same amplitude and
        // period (220 Hz → 218.18 samples at 48 kHz).
        let mid = &y[10_000..30_000];
        let peak = mid.iter().fold(0.0f32, |m, &s| m.max(s.abs()));
        assert!((peak - 1.0).abs() < 0.02, "peak {peak}");
        let crossings = mid.windows(2).filter(|w| w[0] < 0.0 && w[1] >= 0.0).count();
        let expected = 20_000.0 * 220.0 / sr_out;
        assert!(
            (crossings as f32 - expected).abs() <= 1.0,
            "{crossings} vs {expected}"
        );
        assert_eq!(resample(&x, sr_in, sr_in), x);
    }
}
