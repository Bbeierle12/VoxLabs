//! Taps: a bounded channel from the worker to the shell. Every wire the
//! definition lists is cloned into the channel once per hop; a full channel
//! drops the newest message and counts the drop — the worker never blocks
//! on the UI.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::time::Instant;

use super::types::Wire;

/// One tapped value.
#[derive(Clone, Debug)]
pub struct TapMsg {
    /// Index into the pipeline's stage list.
    pub stage: usize,
    pub stage_name: Arc<str>,
    pub hop: u64,
    /// When the hop's samples were taken from the ring buffer: the start of
    /// the analysis leg of the mic-to-render latency.
    pub captured_at: Instant,
    /// When the worker published this message.
    pub published_at: Instant,
    pub value: Wire,
}

pub struct TapSender {
    tx: SyncSender<TapMsg>,
    drops: Arc<AtomicU64>,
}

impl TapSender {
    /// Non-blocking publish. Returns false (and counts) when the channel is
    /// full or the shell is gone.
    pub fn publish(&self, msg: TapMsg) -> bool {
        match self.tx.try_send(msg) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
                self.drops.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }

    pub fn drops(&self) -> Arc<AtomicU64> {
        self.drops.clone()
    }
}

pub struct TapReceiver {
    rx: Receiver<TapMsg>,
}

impl TapReceiver {
    /// Everything published since the last call, oldest first.
    pub fn drain(&self) -> Vec<TapMsg> {
        let mut out = Vec::new();
        while let Ok(m) = self.rx.try_recv() {
            out.push(m);
        }
        out
    }

    /// Blocks up to `timeout` for the next message (tests and harnesses).
    pub fn recv_timeout(&self, timeout: std::time::Duration) -> Option<TapMsg> {
        self.rx.recv_timeout(timeout).ok()
    }
}

/// A bounded tap channel of `capacity` messages.
pub fn tap_channel(capacity: usize) -> (TapSender, TapReceiver) {
    let (tx, rx) = sync_channel(capacity);
    (
        TapSender {
            tx,
            drops: Arc::new(AtomicU64::new(0)),
        },
        TapReceiver { rx },
    )
}
