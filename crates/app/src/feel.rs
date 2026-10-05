//! Player-visible acknowledgment and latency rings.
//!
//! Latency is a CPU timestamp at input receipt to `queue.present` of the first frame that
//! includes the change. It excludes display scanout and does not wait on the GPU.

use cascade_sim::Material;

pub const LATENCY_SAMPLES: usize = 240;
pub const MAX_ACKS: usize = 64;
pub const MAX_FOCUS_RECTS: usize = 8;
const MAX_PENDING_FEEDBACK: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionKind {
    Paint,
    Ignite,
    Detonate,
}

impl ActionKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Paint => "paint",
            Self::Ignite => "ignite",
            Self::Detonate => "detonate",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AckState {
    Pending,
    Visible,
    Rejected,
    NoEffect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LatencySummary {
    pub samples: usize,
    pub p50_ns: u64,
    pub p95_ns: u64,
}

#[derive(Clone, Debug)]
pub struct LatencyRing {
    samples: [u64; LATENCY_SAMPLES],
    len: usize,
    cursor: usize,
}

impl Default for LatencyRing {
    fn default() -> Self {
        Self {
            samples: [0; LATENCY_SAMPLES],
            len: 0,
            cursor: 0,
        }
    }
}

impl LatencyRing {
    pub fn record(&mut self, ns: u64) {
        self.samples[self.cursor] = ns;
        self.cursor = (self.cursor + 1) % LATENCY_SAMPLES;
        if self.len < LATENCY_SAMPLES {
            self.len += 1;
        }
    }

    pub fn summary(&self) -> LatencySummary {
        if self.len == 0 {
            return LatencySummary {
                samples: 0,
                p50_ns: 0,
                p95_ns: 0,
            };
        }
        let mut sorted = [0_u64; LATENCY_SAMPLES];
        let start = if self.len < LATENCY_SAMPLES {
            0
        } else {
            self.cursor
        };
        for (offset, slot) in sorted.iter_mut().take(self.len).enumerate() {
            *slot = self.samples[(start + offset) % LATENCY_SAMPLES];
        }
        sorted[..self.len].sort_unstable();
        LatencySummary {
            samples: self.len,
            p50_ns: percentile(&sorted[..self.len], 50),
            p95_ns: percentile(&sorted[..self.len], 95),
        }
    }
}

fn percentile(sorted_asc: &[u64], pct: usize) -> u64 {
    let n = sorted_asc.len();
    let rank = (pct * n).div_ceil(100);
    let index = rank.saturating_sub(1).min(n - 1);
    sorted_asc[index]
}

#[derive(Clone, Copy, Debug)]
pub struct Ack {
    pub kind: ActionKind,
    pub x: u32,
    pub y: u32,
    pub state: AckState,
    pub admitted_ns: u64,
    pub seen_stamp: u32,
    pub baseline_material: u8,
    pub baseline_burning: u8,
    pub paint_material: u8,
    pub visible_frames: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusKind {
    Viewport,
    Action,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FocusRect {
    pub min_chunk_x: u32,
    pub min_chunk_y: u32,
    pub max_chunk_x: u32,
    pub max_chunk_y: u32,
    pub kind: FocusKind,
}

#[derive(Clone, Debug)]
pub struct Feel {
    camera: LatencyRing,
    paint: LatencyRing,
    ignite: LatencyRing,
    detonate: LatencyRing,
    feedback_ns: [u64; MAX_PENDING_FEEDBACK],
    feedback_len: usize,
    acks: [Option<Ack>; MAX_ACKS],
    ack_cursor: usize,
    stamps: Vec<u32>,
    chunks_x: u32,
    focus: [Option<FocusRect>; MAX_FOCUS_RECTS],
    focus_len: usize,
}

impl Feel {
    pub fn new(chunks_x: u32, chunks_y: u32) -> Self {
        let count = (chunks_x as usize).saturating_mul(chunks_y as usize);
        Self {
            camera: LatencyRing::default(),
            paint: LatencyRing::default(),
            ignite: LatencyRing::default(),
            detonate: LatencyRing::default(),
            feedback_ns: [0; MAX_PENDING_FEEDBACK],
            feedback_len: 0,
            acks: [None; MAX_ACKS],
            ack_cursor: 0,
            stamps: vec![0; count],
            chunks_x,
            focus: [None; MAX_FOCUS_RECTS],
            focus_len: 0,
        }
    }

    pub fn clear_latencies(&mut self) {
        self.camera = LatencyRing::default();
        self.paint = LatencyRing::default();
        self.ignite = LatencyRing::default();
        self.detonate = LatencyRing::default();
        self.feedback_len = 0;
    }

    pub fn camera_summary(&self) -> LatencySummary {
        self.camera.summary()
    }
    pub fn paint_summary(&self) -> LatencySummary {
        self.paint.summary()
    }
    pub fn ignite_summary(&self) -> LatencySummary {
        self.ignite.summary()
    }
    pub fn detonate_summary(&self) -> LatencySummary {
        self.detonate.summary()
    }

    pub fn note_feedback(&mut self, now_ns: u64) {
        if self.feedback_len < MAX_PENDING_FEEDBACK {
            self.feedback_ns[self.feedback_len] = now_ns;
            self.feedback_len += 1;
        }
    }

    pub fn finish_feedback(&mut self, present_ns: u64) {
        for index in 0..self.feedback_len {
            self.camera
                .record(present_ns.saturating_sub(self.feedback_ns[index]));
        }
        self.feedback_len = 0;
    }

    pub fn admit(&mut self, ack: Ack) {
        self.acks[self.ack_cursor] = Some(ack);
        self.ack_cursor = (self.ack_cursor + 1) % MAX_ACKS;
    }

    pub fn acks(&self) -> impl Iterator<Item = &Ack> {
        self.acks.iter().filter_map(Option::as_ref)
    }

    pub fn pending_count(&self) -> usize {
        self.acks
            .iter()
            .filter(|ack| matches!(ack, Some(ack) if ack.state == AckState::Pending))
            .count()
    }

    pub fn exempt_cells(&self, out: &mut [(u32, u32)]) -> usize {
        let mut count = 0;
        for ack in self.acks() {
            if count == out.len() {
                break;
            }
            if matches!(ack.state, AckState::Pending | AckState::Visible) {
                out[count] = (ack.x, ack.y);
                count += 1;
            }
        }
        count
    }

    pub fn stamp_of(&self, chunk_x: u32, chunk_y: u32) -> u32 {
        let index = (chunk_y * self.chunks_x + chunk_x) as usize;
        self.stamps.get(index).copied().unwrap_or(0)
    }

    pub fn note_uploads(&mut self, chunks: &[(u32, u32)]) {
        for &(x, y) in chunks {
            let index = (y * self.chunks_x + x) as usize;
            if let Some(stamp) = self.stamps.get_mut(index) {
                *stamp = stamp.saturating_add(1);
            }
        }
    }

    pub fn observe_actions(
        &mut self,
        now_ns: u64,
        visual_at: impl Fn(u32, u32) -> Option<(u8, u8)>,
    ) {
        for index in 0..self.acks.len() {
            let Some(mut ack) = self.acks[index] else {
                continue;
            };
            if ack.state == AckState::Visible {
                ack.visible_frames = ack.visible_frames.saturating_add(1);
                self.acks[index] = (ack.visible_frames <= 45).then_some(ack);
                continue;
            }
            if matches!(ack.state, AckState::Rejected | AckState::NoEffect) {
                ack.visible_frames = ack.visible_frames.saturating_add(1);
                self.acks[index] = (ack.visible_frames <= 20).then_some(ack);
                continue;
            }
            let Some((material, burning)) = visual_at(ack.x, ack.y) else {
                continue;
            };
            let stamp = self.stamp_of(ack.x / 32, ack.y / 32);
            if stamp <= ack.seen_stamp {
                continue;
            }
            let changed = match ack.kind {
                ActionKind::Paint => material == ack.paint_material,
                ActionKind::Ignite => {
                    burning > ack.baseline_burning
                        || (ack.baseline_material == Material::Wood as u8
                            && (burning > 0 || material != ack.baseline_material))
                        || (ack.baseline_material == Material::Explosive as u8
                            && material != ack.baseline_material)
                }
                ActionKind::Detonate => {
                    material != ack.baseline_material || burning > ack.baseline_burning
                }
            };
            if changed {
                let latency = now_ns.saturating_sub(ack.admitted_ns);
                match ack.kind {
                    ActionKind::Paint => self.paint.record(latency),
                    ActionKind::Ignite => self.ignite.record(latency),
                    ActionKind::Detonate => self.detonate.record(latency),
                }
                ack.state = AckState::Visible;
                ack.visible_frames = 0;
                self.acks[index] = Some(ack);
            }
        }
    }

    pub fn set_focus(&mut self, rects: &[FocusRect]) {
        self.focus = [None; MAX_FOCUS_RECTS];
        self.focus_len = rects.len().min(MAX_FOCUS_RECTS);
        for (index, rect) in rects.iter().take(MAX_FOCUS_RECTS).enumerate() {
            self.focus[index] = Some(*rect);
        }
    }

    pub fn focus_rects(&self) -> impl Iterator<Item = FocusRect> + '_ {
        self.focus
            .iter()
            .take(self.focus_len)
            .filter_map(|rect| *rect)
    }
}

/// Inclusive brush cells along a segment, capped and deduped. Radius is in cells.
pub fn brush_cells(
    from: (i32, i32),
    to: (i32, i32),
    radius: i32,
    limit: usize,
    out: &mut [(u32, u32)],
) -> usize {
    let limit = limit.min(out.len());
    if limit == 0 {
        return 0;
    }
    let radius = radius.clamp(0, 4);
    let mut count = 0;
    let mut stamp = |x: i32, y: i32, count: &mut usize| {
        if *count == limit || x < 0 || y < 0 {
            return;
        }
        let cell = (x as u32, y as u32);
        if out[..*count].contains(&cell) {
            return;
        }
        out[*count] = cell;
        *count += 1;
    };
    let steps = (to.0 - from.0).abs().max((to.1 - from.1).abs()).max(1);
    for step in 0..=steps {
        if count == limit {
            break;
        }
        let x = from.0 + (to.0 - from.0) * step / steps;
        let y = from.1 + (to.1 - from.1) * step / steps;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy > radius * radius {
                    continue;
                }
                stamp(x + dx, y + dy, &mut count);
            }
        }
    }
    count
}

pub fn brush_radius_cells(cells_per_pixel: f32) -> i32 {
    if !cells_per_pixel.is_finite() || cells_per_pixel <= 0.0 {
        return 1;
    }
    (4.0 / cells_per_pixel).round().clamp(1.0, 3.0) as i32
}

/// Clamp a wheel delta so one event cannot skip several zoom levels.
pub fn clamped_zoom_lines(lines: f32) -> f32 {
    if !lines.is_finite() {
        0.0
    } else {
        lines.clamp(-1.25, 1.25)
    }
}

pub fn presented_byte(
    material: u8,
    burning: u8,
    pending: bool,
    deferred: bool,
    exempt: bool,
) -> u8 {
    if deferred && pending && !exempt {
        6
    } else if burning > 0 && material == Material::Wood as u8 {
        7
    } else {
        material
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_latency_percentiles() {
        let mut ring = LatencyRing::default();
        for value in [10_u64, 20, 30, 40, 100] {
            ring.record(value);
        }
        let summary = ring.summary();
        assert_eq!(summary.samples, 5);
        assert_eq!(summary.p50_ns, 30);
        assert_eq!(summary.p95_ns, 100);
    }

    #[test]
    fn brush_stamps_the_press_and_fills_a_fast_drag() {
        let mut out = [(0, 0); 64];
        let count = brush_cells((4, 4), (4, 4), 1, 64, &mut out);
        assert!(count >= 1);
        assert!(out[..count].contains(&(4, 4)));
        let count = brush_cells((0, 0), (6, 0), 0, 64, &mut out);
        assert_eq!(count, 7);
        assert_eq!(out[0], (0, 0));
        assert_eq!(out[6], (6, 0));
    }

    #[test]
    fn brush_obeys_the_frame_cap() {
        let mut out = [(0, 0); 64];
        let count = brush_cells((0, 0), (40, 0), 2, 8, &mut out);
        assert_eq!(count, 8);
    }

    #[test]
    fn visible_effect_waits_for_an_upload_after_admission() {
        let mut feel = Feel::new(4, 4);
        feel.admit(Ack {
            kind: ActionKind::Paint,
            x: 3,
            y: 3,
            state: AckState::Pending,
            admitted_ns: 1_000,
            seen_stamp: 0,
            baseline_material: Material::Sand as u8,
            baseline_burning: 0,
            paint_material: Material::Stone as u8,
            visible_frames: 0,
        });
        feel.observe_actions(2_000, |_, _| Some((Material::Stone as u8, 0)));
        assert_eq!(feel.acks().next().unwrap().state, AckState::Pending);
        assert_eq!(feel.paint_summary().samples, 0);
        feel.note_uploads(&[(0, 0)]);
        feel.observe_actions(3_000, |_, _| Some((Material::Stone as u8, 0)));
        assert_eq!(feel.acks().next().unwrap().state, AckState::Visible);
        assert_eq!(feel.paint_summary().samples, 1);
        assert_eq!(feel.paint_summary().p50_ns, 2_000);
    }

    #[test]
    fn zoom_lines_are_finite_and_clamped() {
        assert_eq!(clamped_zoom_lines(8.0), 1.25);
        assert_eq!(clamped_zoom_lines(-4.0), -1.25);
        assert_eq!(clamped_zoom_lines(f32::NAN), 0.0);
    }
}
