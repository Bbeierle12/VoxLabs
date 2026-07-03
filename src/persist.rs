//! Disk persistence for the enrolled reference voiceprint and the session
//! archive.
//!
//! The whole archive is a small JSON blob written to a per-platform data dir
//! and reloaded at startup, so enrollment and captures survive an app restart.
//! The store path is chosen by each entry point (desktop resolves an XDG-style
//! data dir; Android uses its private `internal_data_path`; web has no disk and
//! passes `None`), so this module is path-agnostic: give it a path and it reads
//! or writes there.
//!
//! Durability contract: `save` is atomic (temp file + fsync + rename), so a
//! crash mid-write can never truncate the previous archive; the written file is
//! private (0600) on unix. `load` never fails — a missing file is a fresh
//! install, and an unreadable file is *backed up* (renamed with a timestamp)
//! and reported via [`Loaded::notice`] so the UI can tell the user instead of
//! silently starting over. Failed saves return a user-facing message.

use crate::types::Voiceprint;
use crate::ui::Session;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Current archive schema version, written on every save. Bump when the shape
/// changes in a way `#[serde(default)]` can't absorb. `0` marks archives
/// written before the version field existed.
pub const ARCHIVE_VERSION: u32 = 1;

/// The complete persisted state: the enrolled reference (if any) plus every
/// saved capture. Mirrors the archive-owning fields of `ui::DashboardApp`.
/// Every field carries a serde default so an archive from an older (or newer,
/// field-dropping) build deserializes instead of discarding the user's data.
#[derive(Serialize, Deserialize)]
pub struct ArchiveState {
    /// Missing in pre-versioning archives → `0` via the field default.
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub enrolled: Option<Voiceprint>,
    #[serde(default)]
    pub enrolled_id: Option<String>,
    #[serde(default = "first_session_num")]
    pub next_session_num: u32,
    #[serde(default)]
    pub sessions: Vec<Session>,
}

/// A never-saved archive numbers its first session VS-001, not VS-000.
fn first_session_num() -> u32 {
    1
}

impl Default for ArchiveState {
    fn default() -> Self {
        Self {
            version: ARCHIVE_VERSION,
            enrolled: None,
            enrolled_id: None,
            next_session_num: first_session_num(),
            sessions: Vec::new(),
        }
    }
}

/// What [`load`] produced: the archive plus a user-facing notice when the file
/// existed but couldn't be used (it is backed up, never silently discarded).
#[derive(Default)]
pub struct Loaded {
    pub state: ArchiveState,
    pub notice: Option<String>,
}

/// Read the archive at `path`. Never fails: a missing file is a fresh install
/// (no notice); an unreadable one is renamed to `<name>.bak-<unix-secs>` and
/// reported via the notice so the UI can surface it.
pub fn load(path: &Path) -> Loaded {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return Loaded::default(),
    };
    match serde_json::from_slice(&bytes) {
        Ok(state) => Loaded {
            state,
            notice: None,
        },
        Err(e) => {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let bak = path.with_extension(format!("json.bak-{secs}"));
            let backed_up = std::fs::rename(path, &bak).is_ok();
            log::warn!(
                "archive at {} is unreadable ({e}); backed up: {backed_up}",
                path.display()
            );
            let notice = if backed_up {
                format!(
                    "Saved archive couldn't be read — starting fresh (backup kept at {})",
                    bak.display()
                )
            } else {
                "Saved archive couldn't be read — starting fresh".to_string()
            };
            Loaded {
                state: ArchiveState::default(),
                notice: Some(notice),
            }
        }
    }
}

/// Write the archive atomically: serialize → private temp file → fsync →
/// rename over `path`. A crash at any point leaves either the old archive or
/// the new one, never a truncated file. Returns a user-facing message on
/// failure (also logged) so the UI can say the save didn't stick.
pub fn save(path: &Path, state: &ArchiveState) -> Result<(), String> {
    let fail = |what: &str, e: &dyn std::fmt::Display| {
        let msg = format!("Session not saved to disk ({what}: {e})");
        log::warn!("{msg} — path {}", path.display());
        msg
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| fail("create dir", &e))?;
    }
    let json = serde_json::to_vec_pretty(state).map_err(|e| fail("serialize", &e))?;
    let tmp = path.with_extension("json.tmp");
    write_private(&tmp, &json).map_err(|e| fail("write", &e))?;
    std::fs::rename(&tmp, path).map_err(|e| fail("rename", &e))?;
    Ok(())
}

/// Create-or-truncate `path` with owner-only permissions (the archive holds
/// voice-biometric-adjacent data) and flush it to disk before returning, so
/// the rename that follows publishes fully-written bytes.
#[cfg(unix)]
fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(bytes)?;
    f.sync_all()
}

#[cfg(not(unix))]
fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    std::fs::write(path, bytes)
}

/// Desktop store path: `$XDG_DATA_HOME/VoxLabs/archive.json`, falling back to
/// `$HOME/.local/share/...`. Linux/XDG is the real deployment target; other
/// desktops land in the same `$HOME`-relative location, which is fine for this
/// single-user tool. Android and web pick their own path (or none), so this is
/// desktop-only.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn default_store_path() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share"))
        })?;
    Some(base.join("VoxLabs").join("archive.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_loads_empty_archive() {
        let path = std::env::temp_dir().join("voxlabs-persist-does-not-exist-xyz.json");
        let _ = std::fs::remove_file(&path);
        let loaded = load(&path);
        assert!(loaded.state.enrolled.is_none());
        assert!(loaded.state.sessions.is_empty());
        // First session must be VS-001, so the counter starts at 1.
        assert_eq!(loaded.state.next_session_num, 1);
        // A fresh install is normal — no scary banner.
        assert!(loaded.notice.is_none());
    }

    #[test]
    fn save_then_load_round_trips_the_reference() {
        let vp = Voiceprint {
            formants: [520.0, 1490.0, 2610.0],
            centroid_hz: 1830.0,
            tilt_db_oct: Some(-9.4),
            profile: std::array::from_fn(|i| (i as f32 + 1.0) / 16.0),
        };
        let state = ArchiveState {
            enrolled: Some(vp),
            enrolled_id: Some("V-F576".to_string()),
            next_session_num: 3,
            ..Default::default()
        };
        let path = std::env::temp_dir().join("voxlabs-persist-roundtrip.json");
        save(&path, &state).expect("save");
        let back = load(&path).state;
        assert_eq!(back.version, ARCHIVE_VERSION);
        assert_eq!(back.enrolled, Some(vp));
        assert_eq!(back.enrolled_id.as_deref(), Some("V-F576"));
        assert_eq!(back.next_session_num, 3);
        // The atomic write must not leave its temp file behind.
        assert!(!path.with_extension("json.tmp").exists());
        let _ = std::fs::remove_file(&path);
    }

    #[cfg(unix)]
    #[test]
    fn archive_is_written_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let path = std::env::temp_dir().join("voxlabs-persist-perms.json");
        save(&path, &ArchiveState::default()).expect("save");
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(
            mode & 0o777,
            0o600,
            "archive should be private, got {mode:o}"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_failure_is_reported_not_silent() {
        // A regular file where the parent directory should be: create_dir_all
        // must fail and the error must come back as a user-facing message.
        let blocker = std::env::temp_dir().join("voxlabs-persist-not-a-dir");
        std::fs::write(&blocker, b"x").unwrap();
        let path = blocker.join("archive.json");
        let err = save(&path, &ArchiveState::default());
        assert!(err.is_err(), "save into a file-as-dir should fail");
        let _ = std::fs::remove_file(&blocker);
    }

    #[test]
    fn pre_option_archives_still_load() {
        // Archives written before `tilt_db_oct`/`match_pct` became Option
        // stored plain numbers; they must load as `Some`, not reset the user.
        let json = r#"{
            "enrolled": {
                "formants": [520.0, 1490.0, 2610.0],
                "centroid_hz": 1830.0,
                "tilt_db_oct": -9.4,
                "profile": [1.0,0.5,0.33,0.25,0.2,0.17,0.14,0.13,0.11,0.1,0.09,0.08,0.08,0.07,0.07,0.06]
            },
            "enrolled_id": "V-F576",
            "next_session_num": 2,
            "sessions": [{
                "id": "VS-001", "subj": "V-F576", "date": "2026-07-01",
                "f0": 118.2, "match_pct": 100.0,
                "hnr_db": 18.1, "h1_h2_db": null, "vibrato": null,
                "steadiness_cents": null, "jitter_pct": null, "shimmer_db": null,
                "cpp_db": null, "centroid_hz": 1830.0,
                "profile": [1.0,0.5,0.33,0.25,0.2,0.17,0.14,0.13,0.11,0.1,0.09,0.08,0.08,0.07,0.07,0.06],
                "formants": null
            }]
        }"#;
        let path = std::env::temp_dir().join("voxlabs-persist-pre-option.json");
        std::fs::write(&path, json).unwrap();
        let loaded = load(&path);
        // No version field in the file → v0, loaded without complaint.
        assert_eq!(loaded.state.version, 0);
        assert!(loaded.notice.is_none());
        assert_eq!(
            loaded
                .state
                .enrolled
                .expect("old reference must load")
                .tilt_db_oct,
            Some(-9.4)
        );
        assert_eq!(
            loaded.state.sessions.len(),
            1,
            "old session must deserialize"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn corrupt_file_is_backed_up_and_reported() {
        let dir = std::env::temp_dir();
        let path = dir.join("voxlabs-persist-corrupt-e2e.json");
        std::fs::write(&path, b"{ this is not valid json").unwrap();
        let loaded = load(&path);
        // Fresh state, but with a notice and the original bytes preserved.
        assert!(loaded.state.enrolled.is_none());
        assert_eq!(loaded.state.next_session_num, 1);
        assert!(loaded.notice.is_some(), "corruption must be surfaced");
        assert!(!path.exists(), "corrupt file should have been renamed away");
        let mut baks: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("voxlabs-persist-corrupt-e2e.json.bak-"))
            })
            .collect();
        assert!(!baks.is_empty(), "backup file should exist");
        for b in baks.drain(..) {
            let _ = std::fs::remove_file(b);
        }
    }
}
