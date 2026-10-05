//! Fixed frame-interval history. Percentiles are nearest-rank over the ring.

/// Intervals strictly longer than this missed a 60 Hz presentation target.
pub const TARGET_FRAME_NS: u64 = 16_666_667;

const RING: usize = 240;

#[derive(Clone, Debug)]
pub struct FrameHistory {
    intervals_ns: [u64; RING],
    len: usize,
    cursor: usize,
    missed_target: u64,
}

impl Default for FrameHistory {
    fn default() -> Self {
        Self {
            intervals_ns: [0; RING],
            len: 0,
            cursor: 0,
            missed_target: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameSummary {
    pub samples: usize,
    pub p50_ns: u64,
    pub p95_ns: u64,
    pub p99_ns: u64,
    pub max_ns: u64,
    pub missed_target: u64,
}

impl FrameHistory {
    pub fn record_interval(&mut self, interval_ns: u64) {
        if interval_ns > TARGET_FRAME_NS {
            self.missed_target += 1;
        }
        self.intervals_ns[self.cursor] = interval_ns;
        self.cursor = (self.cursor + 1) % RING;
        if self.len < RING {
            self.len += 1;
        }
    }

    pub fn missed_target(&self) -> u64 {
        self.missed_target
    }

    /// Oldest to newest. Returns how many samples were written.
    pub fn copy_intervals_ns(&self, out: &mut [u64]) -> usize {
        let n = self.len.min(out.len());
        let start = if self.len < RING { 0 } else { self.cursor };
        for (dst, offset) in (0..n).enumerate() {
            out[dst] = self.intervals_ns[(start + offset) % RING];
        }
        n
    }

    pub fn summary(&self) -> FrameSummary {
        if self.len == 0 {
            return FrameSummary {
                samples: 0,
                p50_ns: 0,
                p95_ns: 0,
                p99_ns: 0,
                max_ns: 0,
                missed_target: self.missed_target,
            };
        }
        let mut sorted = [0_u64; RING];
        let n = self.copy_intervals_ns(&mut sorted);
        sorted[..n].sort_unstable();
        FrameSummary {
            samples: n,
            p50_ns: percentile(&sorted[..n], 50),
            p95_ns: percentile(&sorted[..n], 95),
            p99_ns: percentile(&sorted[..n], 99),
            max_ns: sorted[n - 1],
            missed_target: self.missed_target,
        }
    }
}

fn percentile(sorted_asc: &[u64], pct: usize) -> u64 {
    let n = sorted_asc.len();
    let rank = (pct * n).div_ceil(100);
    let index = rank.saturating_sub(1).min(n - 1);
    sorted_asc[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_percentiles_and_misses() {
        let mut history = FrameHistory::default();
        for value in [10_u64, 20, 30, 40, 100] {
            history.record_interval(value);
        }
        history.record_interval(TARGET_FRAME_NS);
        history.record_interval(TARGET_FRAME_NS + 1);
        let summary = history.summary();
        assert_eq!(summary.samples, 7);
        assert_eq!(summary.missed_target, 1);
        assert_eq!(summary.max_ns, TARGET_FRAME_NS + 1);
        let mut only = FrameHistory::default();
        for value in [10_u64, 20, 30, 40, 100] {
            only.record_interval(value);
        }
        let summary = only.summary();
        assert_eq!(summary.p50_ns, 30);
        assert_eq!(summary.p95_ns, 100);
        assert_eq!(summary.p99_ns, 100);
        assert_eq!(summary.max_ns, 100);
    }

    #[test]
    fn ring_keeps_chronological_order_and_cumulative_misses() {
        let mut history = FrameHistory::default();
        for value in 0..RING as u64 + 5 {
            history.record_interval(if value == 3 {
                TARGET_FRAME_NS + 5
            } else {
                value
            });
        }
        let mut out = [0_u64; RING];
        let n = history.copy_intervals_ns(&mut out);
        assert_eq!(n, RING);
        assert_eq!(out[0], 5);
        assert_eq!(out[RING - 1], RING as u64 + 4);
        assert_eq!(history.missed_target(), 1);
        assert_eq!(history.summary().samples, RING);
    }

    #[test]
    fn empty_summary_is_zero() {
        let summary = FrameHistory::default().summary();
        assert_eq!(summary.samples, 0);
        assert_eq!(summary.p99_ns, 0);
    }
}
