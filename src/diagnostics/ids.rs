//! Identifier and clock helpers for the diagnostics runtime: epoch
//! milliseconds, short hex ids, and UUID-shaped record ids. No `rand` or
//! `uuid` dependency: a xorshift generator seeded from the clock, the
//! process id and a stack address is plenty for ids that only need to be
//! unique within one phone's diagnostics folder.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static STATE: AtomicU64 = AtomicU64::new(0);

pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn next_u64() -> u64 {
    let mut x = STATE.load(Ordering::Relaxed);
    if x == 0 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        let addr = &x as *const u64 as u64;
        x = nanos ^ (u64::from(std::process::id()) << 32) ^ addr.rotate_left(17);
        if x == 0 {
            x = 0x9E37_79B9_7F4A_7C15;
        }
    }
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    STATE.store(x, Ordering::Relaxed);
    x
}

/// `n` lowercase hex characters.
pub fn random_hex(n: usize) -> String {
    let mut s = String::with_capacity(n);
    while s.len() < n {
        s.push_str(&format!("{:016x}", next_u64()));
    }
    s.truncate(n);
    s
}

/// A UUID-shaped id (`8-4-4-4-12` hex), as the source app's record ids.
pub fn uuid_like() -> String {
    let h = random_hex(32);
    format!(
        "{}-{}-{}-{}-{}",
        &h[0..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..32]
    )
}

/// `HH:MM:SS` of an epoch-millisecond stamp, in UTC.
pub fn clock_utc(millis: u64) -> String {
    let secs = millis / 1000;
    let day = secs % 86_400;
    format!("{:02}:{:02}:{:02}", day / 3600, (day % 3600) / 60, day % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_have_the_right_shape_and_differ() {
        let a = uuid_like();
        let b = uuid_like();
        assert_eq!(a.len(), 36);
        assert_ne!(a, b);
        assert_eq!(random_hex(8).len(), 8);
        assert_eq!(clock_utc(3_723_000), "01:02:03");
    }
}
