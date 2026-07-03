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
//! Both `load` and `save` are best-effort — a missing, unreadable, or corrupt
//! file yields a fresh empty archive rather than an error, and a failed write
//! is logged but never interrupts the UI. Losing an as-yet-unpersisted capture
//! is preferable to crashing the instrument mid-session.

use crate::types::Voiceprint;
use crate::ui::Session;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The complete persisted state: the enrolled reference (if any) plus every
/// saved capture. Mirrors the archive-owning fields of `ui::DashboardApp`.
#[derive(Serialize, Deserialize)]
pub struct ArchiveState {
    pub enrolled: Option<Voiceprint>,
    pub enrolled_id: Option<String>,
    pub next_session_num: u32,
    pub sessions: Vec<Session>,
}

impl Default for ArchiveState {
    fn default() -> Self {
        // A never-saved archive: nothing enrolled, no captures, and the first
        // session numbered VS-001 (not VS-000).
        Self {
            enrolled: None,
            enrolled_id: None,
            next_session_num: 1,
            sessions: Vec::new(),
        }
    }
}

/// Read the archive at `path`. A missing or corrupt file is treated as an empty
/// archive (the app simply starts fresh), so this never fails.
pub fn load(path: &Path) -> ArchiveState {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return ArchiveState::default(),
    };
    match serde_json::from_slice(&bytes) {
        Ok(state) => state,
        Err(e) => {
            log::warn!("archive at {} is unreadable ({e}); starting fresh", path.display());
            ArchiveState::default()
        }
    }
}

/// Write the archive to `path`, creating parent directories as needed. Failures
/// are logged, not propagated — persistence is a convenience, never a blocker.
pub fn save(path: &Path, state: &ArchiveState) {
    if let Some(dir) = path.parent()
        && let Err(e) = std::fs::create_dir_all(dir)
    {
        log::warn!("could not create archive dir {}: {e}", dir.display());
        return;
    }
    match serde_json::to_vec_pretty(state) {
        Ok(json) => {
            if let Err(e) = std::fs::write(path, json) {
                log::warn!("could not write archive to {}: {e}", path.display());
            }
        }
        Err(e) => log::warn!("could not serialize archive: {e}"),
    }
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
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share")))?;
    Some(base.join("VoxLabs").join("archive.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_loads_empty_archive() {
        let path = std::env::temp_dir().join("voxlabs-persist-does-not-exist-xyz.json");
        let _ = std::fs::remove_file(&path);
        let state = load(&path);
        assert!(state.enrolled.is_none());
        assert!(state.sessions.is_empty());
        // First session must be VS-001, so the counter starts at 1.
        assert_eq!(state.next_session_num, 1);
    }

    #[test]
    fn save_then_load_round_trips_the_reference() {
        let vp = Voiceprint {
            formants: [520.0, 1490.0, 2610.0],
            centroid_hz: 1830.0,
            tilt_db_oct: -9.4,
            profile: std::array::from_fn(|i| (i as f32 + 1.0) / 16.0),
        };
        let state = ArchiveState {
            enrolled: Some(vp),
            enrolled_id: Some("V-F576".to_string()),
            next_session_num: 3,
            sessions: Vec::new(),
        };
        let path = std::env::temp_dir().join("voxlabs-persist-roundtrip.json");
        save(&path, &state);
        let back = load(&path);
        assert_eq!(back.enrolled, Some(vp));
        assert_eq!(back.enrolled_id.as_deref(), Some("V-F576"));
        assert_eq!(back.next_session_num, 3);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn corrupt_file_loads_empty_archive() {
        let path = std::env::temp_dir().join("voxlabs-persist-corrupt.json");
        std::fs::write(&path, b"{ this is not valid json").unwrap();
        let state = load(&path);
        assert!(state.enrolled.is_none());
        assert_eq!(state.next_session_num, 1);
        let _ = std::fs::remove_file(&path);
    }
}
