//! Fair dirty-chunk uploads. Caps are fixed; leftover chunks stay dirty.

use std::collections::VecDeque;

use crate::grid::{CHUNK_SIZE, ChunkCoord};

pub const MAX_CHUNKS_PER_FRAME: usize = 256;
pub const MAX_PAYLOAD_BYTES_PER_FRAME: usize = 1024 * 1024;
pub const MAX_COPIES_PER_FRAME: usize = 256;

#[derive(Clone, Copy, Debug)]
pub struct UploadBudget {
    pub max_chunks: usize,
    pub max_bytes: usize,
    pub max_copies: usize,
}

impl Default for UploadBudget {
    fn default() -> Self {
        Self {
            max_chunks: MAX_CHUNKS_PER_FRAME,
            max_bytes: MAX_PAYLOAD_BYTES_PER_FRAME,
            max_copies: MAX_COPIES_PER_FRAME,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UploadPlan {
    pub chunks: [ChunkCoord; MAX_CHUNKS_PER_FRAME],
    pub count: usize,
    pub payload_bytes: usize,
    pub backlog: usize,
    pub oldest_dirty_ticks: Option<u64>,
    pub stale: bool,
}

impl Default for UploadPlan {
    fn default() -> Self {
        Self {
            chunks: [ChunkCoord::default(); MAX_CHUNKS_PER_FRAME],
            count: 0,
            payload_bytes: 0,
            backlog: 0,
            oldest_dirty_ticks: None,
            stale: false,
        }
    }
}

/// Deduped FIFO. Re-marking a queued chunk does not move it; the grid holds the latest bytes.
#[derive(Clone, Debug)]
pub struct UploadScheduler {
    chunks_x: u32,
    chunks_y: u32,
    queued: Vec<bool>,
    enqueued_at: Vec<Option<u64>>,
    order: VecDeque<u32>,
    now: u64,
}

impl UploadScheduler {
    pub fn new(chunks_x: u32, chunks_y: u32) -> Self {
        let count = (chunks_x as usize).saturating_mul(chunks_y as usize);
        let mut order = VecDeque::new();
        order.reserve(count);
        Self {
            chunks_x,
            chunks_y,
            queued: vec![false; count],
            enqueued_at: vec![None; count],
            order,
            now: 0,
        }
    }

    pub fn set_clock(&mut self, now: u64) {
        self.now = now;
    }

    pub fn mark_dirty(&mut self, chunk: ChunkCoord) -> bool {
        let Some(index) = self.index(chunk) else {
            return false;
        };
        if self.queued[index] {
            return false;
        }
        self.queued[index] = true;
        self.enqueued_at[index] = Some(self.now);
        self.order.push_back(index as u32);
        true
    }

    pub fn mark_all(&mut self) {
        for y in 0..self.chunks_y {
            for x in 0..self.chunks_x {
                let _ = self.mark_dirty(ChunkCoord { x, y });
            }
        }
    }

    pub fn backlog(&self) -> usize {
        self.order.len()
    }

    pub fn plan(&mut self, bytes_per_chunk: usize, budget: UploadBudget) -> UploadPlan {
        let mut plan = UploadPlan::default();
        let limit = budget
            .max_chunks
            .min(budget.max_copies)
            .min(MAX_CHUNKS_PER_FRAME);
        if bytes_per_chunk == 0 || bytes_per_chunk > budget.max_bytes {
            plan.backlog = self.order.len();
            plan.oldest_dirty_ticks = self.oldest_age();
            plan.stale = plan.backlog > 0;
            return plan;
        }
        while plan.count < limit {
            let Some(&index) = self.order.front() else {
                break;
            };
            let next_bytes = plan.payload_bytes.saturating_add(bytes_per_chunk);
            if next_bytes > budget.max_bytes {
                break;
            }
            self.order.pop_front();
            let index = index as usize;
            self.queued[index] = false;
            self.enqueued_at[index] = None;
            plan.chunks[plan.count] = self.coord(index);
            plan.count += 1;
            plan.payload_bytes = next_bytes;
        }
        plan.backlog = self.order.len();
        plan.oldest_dirty_ticks = self.oldest_age();
        plan.stale = plan.backlog > 0;
        plan
    }

    fn oldest_age(&self) -> Option<u64> {
        let index = *self.order.front()? as usize;
        let enqueued = self.enqueued_at[index]?;
        Some(self.now.saturating_sub(enqueued))
    }

    fn index(&self, chunk: ChunkCoord) -> Option<usize> {
        if chunk.x >= self.chunks_x || chunk.y >= self.chunks_y {
            return None;
        }
        Some((chunk.y as usize) * (self.chunks_x as usize) + chunk.x as usize)
    }

    fn coord(&self, index: usize) -> ChunkCoord {
        let width = self.chunks_x as usize;
        ChunkCoord {
            x: (index % width) as u32,
            y: (index / width) as u32,
        }
    }
}

pub fn bytes_per_chunk(bytes_per_texel: u32) -> usize {
    (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize) * (bytes_per_texel as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scheduler(chunks: u32) -> UploadScheduler {
        UploadScheduler::new(chunks, chunks)
    }

    #[test]
    fn chunk_cap_binds_before_the_byte_cap_for_r8() {
        let mut scheduler = scheduler(32);
        scheduler.mark_all();
        let plan = scheduler.plan(bytes_per_chunk(1), UploadBudget::default());
        assert_eq!(plan.count, MAX_CHUNKS_PER_FRAME);
        assert_eq!(plan.payload_bytes, MAX_CHUNKS_PER_FRAME * 1024);
        assert!(plan.payload_bytes < MAX_PAYLOAD_BYTES_PER_FRAME);
        assert!(plan.stale);
        assert_eq!(plan.backlog, 32 * 32 - MAX_CHUNKS_PER_FRAME);
        assert_eq!(plan.chunks[0], ChunkCoord { x: 0, y: 0 });
    }

    #[test]
    fn byte_cap_stops_before_the_chunk_cap() {
        let mut scheduler = scheduler(20);
        scheduler.mark_all();
        let bytes = 8192;
        let plan = scheduler.plan(bytes, UploadBudget::default());
        assert_eq!(plan.count, MAX_PAYLOAD_BYTES_PER_FRAME / bytes);
        assert_eq!(plan.payload_bytes, MAX_PAYLOAD_BYTES_PER_FRAME);
        assert!(plan.count < MAX_CHUNKS_PER_FRAME);
        assert!(plan.stale);
        let second = scheduler.plan(bytes, UploadBudget::default());
        let next = plan.count as u32;
        assert_eq!(
            second.chunks[0],
            ChunkCoord {
                x: next % 20,
                y: next / 20,
            }
        );
    }

    #[test]
    fn fifo_does_not_let_a_re_dirtied_chunk_cut_the_line() {
        let mut scheduler = UploadScheduler::new(4, 1);
        assert!(scheduler.mark_dirty(ChunkCoord { x: 0, y: 0 }));
        assert!(scheduler.mark_dirty(ChunkCoord { x: 1, y: 0 }));
        assert!(!scheduler.mark_dirty(ChunkCoord { x: 0, y: 0 }));
        let budget = UploadBudget {
            max_chunks: 1,
            max_bytes: 1024,
            max_copies: 1,
        };
        let first = scheduler.plan(1024, budget);
        assert_eq!(first.chunks[0], ChunkCoord { x: 0, y: 0 });
        assert!(scheduler.mark_dirty(ChunkCoord { x: 0, y: 0 }));
        let second = scheduler.plan(1024, budget);
        assert_eq!(second.chunks[0], ChunkCoord { x: 1, y: 0 });
        let third = scheduler.plan(1024, budget);
        assert_eq!(third.chunks[0], ChunkCoord { x: 0, y: 0 });
        assert!(!third.stale);
    }

    #[test]
    fn oversized_chunk_is_not_uploaded_and_age_advances() {
        let mut scheduler = UploadScheduler::new(2, 1);
        scheduler.set_clock(10);
        scheduler.mark_dirty(ChunkCoord { x: 1, y: 0 });
        scheduler.set_clock(25);
        let plan = scheduler.plan(MAX_PAYLOAD_BYTES_PER_FRAME + 1, UploadBudget::default());
        assert_eq!(plan.count, 0);
        assert!(plan.stale);
        assert_eq!(plan.oldest_dirty_ticks, Some(15));
        assert_eq!(scheduler.backlog(), 1);
    }

    #[test]
    fn out_of_range_chunks_are_rejected() {
        let mut scheduler = UploadScheduler::new(2, 2);
        assert!(!scheduler.mark_dirty(ChunkCoord { x: 2, y: 0 }));
        assert_eq!(scheduler.backlog(), 0);
    }
}
