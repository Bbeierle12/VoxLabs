//! The room, as calibration learned it: the ambient floor seed and the
//! fingerprinted steady interferer. Written by whichever loop runs the
//! calibration pass (the pipeline's hop observer, the desktop engine),
//! read by the `voicing` stage on the worker. A stage cannot receive a
//! mutable handle at `init`, so this process-wide slot is the seam; the
//! stage polls the generation counter once per hop and reloads only when
//! it changed.

use std::sync::Mutex;

use crate::math::Interferer;

#[derive(Clone, Copy, Debug, Default)]
pub struct RoomState {
    /// Bumped on every write; readers reload when it differs from theirs.
    pub generation: u64,
    /// Ambient RMS to seed the noise floor with (`None` = learn passively).
    pub floor_seed: Option<f32>,
    pub interferer: Option<Interferer>,
}

static ROOM: Mutex<RoomState> = Mutex::new(RoomState {
    generation: 0,
    floor_seed: None,
    interferer: None,
});

fn lock() -> std::sync::MutexGuard<'static, RoomState> {
    match ROOM.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// A completed calibration: seed the floor and install (or clear) the hum.
pub fn set_calibration(ambient_rms: f32, interferer: Option<Interferer>) {
    let mut r = lock();
    r.generation += 1;
    r.floor_seed = Some(ambient_rms);
    r.interferer = interferer;
}

pub fn snapshot() -> RoomState {
    *lock()
}

/// Test and relaunch hygiene: back to "nothing learned".
pub fn reset() {
    let mut r = lock();
    r.generation += 1;
    r.floor_seed = None;
    r.interferer = None;
}
