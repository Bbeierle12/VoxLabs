//! In-app diagnostics log: the most recent log lines, kept in memory so a
//! phone without `adb` can still show them (Room → DIAGNOSTICS). Android
//! installs [`Tee`] around its logcat logger; other targets can push lines
//! directly.

use std::collections::VecDeque;
use std::sync::Mutex;

/// Lines retained. Enough to cover a launch, a permission round and a few
/// runner reports; small enough that a repaint can render the tail cheaply.
pub const CAPACITY: usize = 200;

static RING: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

/// Appends one line, dropping the oldest past [`CAPACITY`].
pub fn push(line: String) {
    let mut ring = match RING.lock() {
        Ok(r) => r,
        Err(p) => p.into_inner(),
    };
    if ring.len() >= CAPACITY {
        ring.pop_front();
    }
    ring.push_back(line);
}

/// The newest `last` lines, oldest first.
pub fn tail(last: usize) -> Vec<String> {
    let ring = match RING.lock() {
        Ok(r) => r,
        Err(p) => p.into_inner(),
    };
    let skip = ring.len().saturating_sub(last);
    ring.iter().skip(skip).cloned().collect()
}

/// A `log::Log` that forwards every record to `inner` and keeps the
/// Info-and-above lines in the ring.
pub struct Tee<L: log::Log> {
    pub inner: L,
}

impl<L: log::Log> log::Log for Tee<L> {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        self.inner.log(record);
        if record.level() <= log::Level::Info {
            let target = record
                .target()
                .rsplit("::")
                .next()
                .unwrap_or(record.target());
            push(format!("{} {target}: {}", record.level(), record.args()));
        }
    }

    fn flush(&self) {
        self.inner.flush()
    }
}
