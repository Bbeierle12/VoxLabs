//! Bridges the `log` facade into the diagnostics session log: every
//! warning or error the app logs also lands as a `log/<target>` event, so
//! the Engineering Console's "Recent session events" shows what logcat
//! would — the bug report for a phone without adb.

use std::collections::BTreeMap;

use crate::diagnostics::{Severity, runtime};

/// A `log::Log` that forwards every record to `inner` and files the
/// warnings and errors as diagnostics events.
pub struct Tee<L: log::Log> {
    pub inner: L,
}

impl<L: log::Log> log::Log for Tee<L> {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        self.inner.log(record);
        let severity = match record.level() {
            log::Level::Error => Severity::Error,
            log::Level::Warn => Severity::Warning,
            _ => return,
        };
        let target = record
            .target()
            .rsplit("::")
            .next()
            .unwrap_or(record.target());
        runtime::log(
            "log",
            target,
            &record.args().to_string(),
            BTreeMap::new(),
            severity,
        );
    }

    fn flush(&self) {
        self.inner.flush()
    }
}
