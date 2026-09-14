//! Re-framing a sample stream to fixed hops (D6): the first hop needs a
//! whole frame, every later hop `hop` new samples shifted into the frame.
//! Shared by the live runner (which drains a ring buffer into it) and the
//! offline driver (which feeds it a whole file), so both see the same
//! frames at the same hop indices.

use super::stage::StreamFormat;

pub struct Framer {
    pub(crate) frame: Vec<f32>,
    pub(crate) pending: Vec<f32>,
    hop: usize,
    primed: bool,
}

impl Framer {
    pub fn new(format: &StreamFormat, pending_frames: usize) -> Self {
        Self {
            frame: vec![0.0; format.frame_samples],
            pending: Vec::with_capacity(format.frame_samples * pending_frames),
            hop: format.hop,
            primed: false,
        }
    }

    /// Samples needed before the next hop can advance.
    pub fn need(&self) -> usize {
        if self.primed {
            self.hop
        } else {
            self.frame.len()
        }
    }

    pub fn push(&mut self, sample: f32) {
        self.pending.push(sample);
    }

    pub fn frame(&self) -> &[f32] {
        &self.frame
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// Advances one hop if enough samples are pending; the frame then holds
    /// the newest `frame_samples` samples.
    pub fn advance(&mut self) -> bool {
        let need = self.need();
        if self.pending.len() < need {
            return false;
        }
        let len = self.frame.len();
        if self.primed {
            self.frame.copy_within(self.hop..len, 0);
            self.frame[len - self.hop..].copy_from_slice(&self.pending[..need]);
        } else {
            self.frame.copy_from_slice(&self.pending[..need]);
            self.primed = true;
        }
        self.pending.drain(..need);
        true
    }

    /// Hops a stream of `n` samples yields: `floor((n - frame) / hop) + 1`,
    /// or 0 when it never fills a frame.
    pub fn hops_for(format: &StreamFormat, n: usize) -> u64 {
        if n < format.frame_samples {
            0
        } else {
            ((n - format.frame_samples) / format.hop + 1) as u64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_hold_the_newest_samples_whatever_the_push_sizes() {
        let fmt = StreamFormat {
            sample_rate_hz: 1.0,
            frame_samples: 8,
            hop: 4,
        };
        let mut f = Framer::new(&fmt, 2);
        let stream: Vec<f32> = (0..20).map(|i| i as f32).collect();
        let mut hops = 0;
        let mut frames = Vec::new();
        for chunk in stream.chunks(3) {
            for &s in chunk {
                f.push(s);
            }
            while f.advance() {
                frames.push(f.frame().to_vec());
                hops += 1;
            }
        }
        assert_eq!(hops, Framer::hops_for(&fmt, 20));
        assert_eq!(frames[0], (0..8).map(|i| i as f32).collect::<Vec<_>>());
        assert_eq!(frames[1], (4..12).map(|i| i as f32).collect::<Vec<_>>());
        assert_eq!(frames[3], (12..20).map(|i| i as f32).collect::<Vec<_>>());
        assert_eq!(Framer::hops_for(&fmt, 7), 0);
    }
}
