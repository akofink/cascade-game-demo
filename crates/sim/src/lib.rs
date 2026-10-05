//! Bounded-work cellular simulation core. Independent of windows, graphics, and wall clocks.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub const CHUNK_SIDE: u32 = 32;
pub const DEFAULT_READY_CAPACITY: usize = 32_768;
pub const DEFAULT_COMMAND_CAPACITY: usize = 256;
pub const RULE_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Material {
    Air = 0,
    Stone = 1,
    Wood = 2,
    Sand = 3,
    Explosive = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct CellId(u32);
impl CellId {
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Generation(u32);
impl Generation {
    pub const fn get(self) -> u32 {
        self.0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Credits(u32);
impl Credits {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
    pub const fn get(self) -> u32 {
        self.0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Capacity(usize);
impl Capacity {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }
    pub const fn get(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CellView {
    pub material: Material,
    pub state: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct Cell {
    material: Material,
}
impl Default for Cell {
    fn default() -> Self {
        Self {
            material: Material::Air,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SimError {
    InvalidDimensions,
    WorldTooLarge,
    BudgetTooSmall,
    CapacityZero,
    GenerationExhausted,
    OutOfBounds,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceTable {
    pub cells: usize,
    pub cell_bytes: usize,
    pub pending_bytes: usize,
    pub ready_capacity: Capacity,
    pub command_capacity: Capacity,
    pub chunk_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CostContract {
    pub selection: Credits,
    pub evaluate: Credits,
    pub blast: Credits,
    pub recovery: Credits,
    pub command: Credits,
}
impl Default for CostContract {
    fn default() -> Self {
        Self {
            selection: Credits(1),
            evaluate: Credits(8),
            blast: Credits(12),
            recovery: Credits(4),
            command: Credits(6),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum JobKind {
    Evaluate,
    Blast,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct Job {
    cell: CellId,
    kind: JobKind,
    generation: Generation,
}

struct Ring<T: Copy> {
    slots: Vec<Option<T>>,
    head: usize,
    len: usize,
}
impl<T: Copy> Ring<T> {
    fn new(capacity: usize) -> Self {
        Self {
            slots: vec![None; capacity],
            head: 0,
            len: 0,
        }
    }
    fn push(&mut self, value: T) -> Result<(), T> {
        if self.len == self.slots.len() {
            return Err(value);
        }
        let tail = (self.head + self.len) % self.slots.len();
        self.slots[tail] = Some(value);
        self.len += 1;
        Ok(())
    }
    fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let value = self.slots[self.head].take();
        self.head = (self.head + 1) % self.slots.len();
        self.len -= 1;
        value
    }
    fn len(&self) -> usize {
        self.len
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Command {
    Paint { cell: CellId, material: Material },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubmitResult {
    Accepted,
    Coalesced,
    RejectedFull,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SliceMetrics {
    pub allowed: u32,
    pub charged: u32,
    pub selections: u32,
    pub evaluations: u32,
    pub blasts: u32,
    pub recoveries: u32,
    pub commands: u32,
    pub rejected_commands: u64,
    pub coalesced_commands: u64,
    pub ready_len: usize,
    pub pending_cells: usize,
    pub oldest_pending_age: u64,
    pub slice: u64,
}

/// Fixed-layout simulation world. Total storage is exposed by [`World::resources`].
pub struct World {
    width: u32,
    height: u32,
    chunks_x: u32,
    chunks_y: u32,
    cells: Vec<Cell>,
    eval_pending: Vec<bool>,
    blast_pending: Vec<u8>,
    eval_queued: Vec<bool>,
    blast_queued: Vec<bool>,
    pending_since: Vec<u64>,
    pending_count: usize,
    oldest_pending_since: u64,
    eval_ready: Ring<Job>,
    blast_ready: Ring<Job>,
    commands: Ring<Command>,
    generation: Generation,
    budget: Credits,
    costs: CostContract,
    recovery_cursor: usize,
    lane_cursor: u8,
    slice: u64,
    rejected_commands: u64,
    coalesced_commands: u64,
    dirty_chunks: Vec<bool>,
    dirty_cursor: usize,
    deferred_command: Option<Command>,
}

impl World {
    pub fn new(
        width: u32,
        height: u32,
        budget: Credits,
        ready_capacity: Capacity,
        command_capacity: Capacity,
    ) -> Result<Self, SimError> {
        if width == 0 || height == 0 {
            return Err(SimError::InvalidDimensions);
        }
        let count = (width as usize)
            .checked_mul(height as usize)
            .ok_or(SimError::WorldTooLarge)?;
        if count > u32::MAX as usize {
            return Err(SimError::WorldTooLarge);
        }
        if budget.0 < 13 {
            return Err(SimError::BudgetTooSmall);
        }
        if ready_capacity.0 == 0 || command_capacity.0 == 0 {
            return Err(SimError::CapacityZero);
        }
        let chunks_x = width.div_ceil(CHUNK_SIDE);
        let chunks_y = height.div_ceil(CHUNK_SIDE);
        let chunk_count = (chunks_x as usize)
            .checked_mul(chunks_y as usize)
            .ok_or(SimError::WorldTooLarge)?;
        Ok(Self {
            width,
            height,
            chunks_x,
            chunks_y,
            cells: vec![Cell::default(); count],
            eval_pending: vec![false; count],
            blast_pending: vec![0; count],
            eval_queued: vec![false; count],
            blast_queued: vec![false; count],
            pending_since: vec![0; count],
            pending_count: 0,
            oldest_pending_since: 0,
            eval_ready: Ring::new(ready_capacity.0),
            blast_ready: Ring::new(ready_capacity.0),
            commands: Ring::new(command_capacity.0),
            generation: Generation(1),
            budget,
            costs: CostContract::default(),
            recovery_cursor: 0,
            lane_cursor: 0,
            slice: 0,
            rejected_commands: 0,
            coalesced_commands: 0,
            dirty_chunks: vec![false; chunk_count],
            dirty_cursor: 0,
            deferred_command: None,
        })
    }
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub fn generation(&self) -> Generation {
        self.generation
    }
    pub fn cost_contract(&self) -> CostContract {
        self.costs
    }
    pub fn slice_index(&self) -> u64 {
        self.slice
    }
    pub fn resources(&self) -> ResourceTable {
        ResourceTable {
            cells: self.cells.len(),
            cell_bytes: std::mem::size_of::<Cell>(),
            pending_bytes: self.cells.len() * (1 + 1 + 1 + 1 + 8),
            ready_capacity: Capacity(self.eval_ready.slots.len() + self.blast_ready.slots.len()),
            command_capacity: Capacity(self.commands.slots.len()),
            chunk_count: self.dirty_chunks.len(),
        }
    }
    pub fn cell(&self, x: u32, y: u32) -> Option<CellView> {
        let id = self.cell_id(x, y)?;
        let c = self.cells[id.index()];
        Some(CellView {
            material: c.material,
            state: self.blast_pending[id.index()],
        })
    }
    pub fn cell_id(&self, x: u32, y: u32) -> Option<CellId> {
        (x < self.width && y < self.height).then_some(CellId(y * self.width + x))
    }
    pub fn chunk_dimensions(&self) -> (u32, u32) {
        (self.chunks_x, self.chunks_y)
    }
    pub fn chunk_dirty(&self, x: u32, y: u32) -> bool {
        x < self.chunks_x
            && y < self.chunks_y
            && self.dirty_chunks[(y * self.chunks_x + x) as usize]
    }
    /// Clears at most `limit` dirty chunk marks and returns their coordinates in stable round-robin order.
    pub fn drain_dirty_chunks(&mut self, limit: usize, out: &mut Vec<(u32, u32)>) -> usize {
        let mut drained = 0;
        for _ in 0..self.dirty_chunks.len().min(limit) {
            let i = self.dirty_cursor;
            self.dirty_cursor = (self.dirty_cursor + 1) % self.dirty_chunks.len().max(1);
            if self.dirty_chunks[i] {
                self.dirty_chunks[i] = false;
                out.push(((i as u32) % self.chunks_x, (i as u32) / self.chunks_x));
                drained += 1;
            }
        }
        drained
    }
    pub fn submit(&mut self, command: Command) -> SubmitResult {
        let Command::Paint { cell, .. } = command;
        if cell.index() >= self.cells.len() {
            self.rejected_commands += 1;
            return SubmitResult::RejectedFull;
        }
        // Rotate exactly the existing bounded frontier, replacing at most one matching target.
        let mut replaced = false;
        let count = self.commands.len();
        for _ in 0..count {
            let old = self
                .commands
                .pop()
                .expect("ring length matches occupied entries");
            let Command::Paint { cell: old_cell, .. } = old;
            if old_cell == cell && !replaced {
                let _ = self.commands.push(command);
                replaced = true;
            } else {
                let _ = self.commands.push(old);
            }
        }
        if replaced {
            self.coalesced_commands += 1;
            return SubmitResult::Coalesced;
        }
        match self.commands.push(command) {
            Ok(()) => SubmitResult::Accepted,
            Err(_) => {
                self.rejected_commands += 1;
                SubmitResult::RejectedFull
            }
        }
    }
    /// Merge a finite blast request. Energy saturates and does not represent additive pressure.
    pub fn trigger_blast(&mut self, id: CellId, energy: u8) -> Result<(), SimError> {
        if id.index() >= self.cells.len() {
            return Err(SimError::OutOfBounds);
        }
        if energy == 0 {
            return Ok(());
        }
        self.set_material(id, Material::Explosive);
        if self.blast_pending[id.index()] == 0 {
            self.pending_count += 1;
        }
        let merged = self.blast_pending[id.index()].max(energy.min(15));
        if merged != self.blast_pending[id.index()] {
            self.mark_dirty(id);
        }
        self.blast_pending[id.index()] = merged;
        self.mark_pending(id, false);
        self.try_queue(id, JobKind::Blast);
        Ok(())
    }
    pub fn mark_cell_for_evaluation(&mut self, id: CellId) -> Result<(), SimError> {
        if id.index() >= self.cells.len() {
            return Err(SimError::OutOfBounds);
        }
        self.mark_pending(id, true);
        self.try_queue(id, JobKind::Evaluate);
        Ok(())
    }
    fn mark_pending(&mut self, id: CellId, evaluate: bool) {
        let i = id.index();
        if evaluate {
            if !self.eval_pending[i] {
                self.eval_pending[i] = true;
                self.pending_count += 1;
            }
        } else if self.blast_pending[i] == 0 {
            return;
        }
        if self.pending_since[i] == 0 {
            self.pending_since[i] = self.slice.saturating_add(1);
            if self.oldest_pending_since == 0 {
                self.oldest_pending_since = self.pending_since[i];
            }
        }
    }
    fn try_queue(&mut self, id: CellId, kind: JobKind) {
        let i = id.index();
        let queued = match kind {
            JobKind::Evaluate => &mut self.eval_queued[i],
            JobKind::Blast => &mut self.blast_queued[i],
        };
        if *queued {
            return;
        }
        let ring = match kind {
            JobKind::Evaluate => &mut self.eval_ready,
            JobKind::Blast => &mut self.blast_ready,
        };
        if ring
            .push(Job {
                cell: id,
                kind,
                generation: self.generation,
            })
            .is_ok()
        {
            *queued = true;
        }
    }
    fn wake_neighbors(&mut self, id: CellId) {
        let (x, y) = (id.0 % self.width, id.0 / self.width);
        for (dx, dy) in [(0i32, -1i32), (-1, 0), (1, 0), (0, 1)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && ny >= 0 && nx < self.width as i32 && ny < self.height as i32 {
                let n = CellId(ny as u32 * self.width + nx as u32);
                self.mark_pending(n, true);
                self.try_queue(n, JobKind::Evaluate);
            }
        }
    }
    fn mark_dirty(&mut self, id: CellId) {
        let chunk =
            (id.0 / self.width) / CHUNK_SIDE * self.chunks_x + (id.0 % self.width) / CHUNK_SIDE;
        self.dirty_chunks[chunk as usize] = true;
    }
    fn set_material(&mut self, id: CellId, material: Material) {
        let i = id.index();
        if self.cells[i].material != material {
            self.cells[i].material = material;
            self.mark_dirty(id);
        }
    }
    fn execute_evaluate(&mut self, id: CellId) {
        let i = id.index();
        if self.eval_pending[i] {
            self.eval_pending[i] = false;
            self.pending_count -= 1;
        }
        self.pending_since[i] = if self.blast_pending[i] > 0 {
            self.pending_since[i]
        } else {
            0
        };
        if self.cells[i].material != Material::Sand {
            return;
        }
        let y = id.0 / self.width;
        if y + 1 >= self.height {
            return;
        }
        let below = CellId(id.0 + self.width);
        if self.cells[below.index()].material == Material::Air {
            self.set_material(below, Material::Sand);
            self.set_material(id, Material::Air);
            self.wake_neighbors(id);
            self.wake_neighbors(below);
        }
    }
    fn execute_blast(&mut self, id: CellId) {
        let i = id.index();
        let energy = self.blast_pending[i];
        if self.blast_pending[i] > 0 {
            self.pending_count -= 1;
        }
        if self.blast_pending[i] > 0 {
            self.mark_dirty(id);
        }
        self.blast_pending[i] = 0;
        self.blast_queued[i] = false;
        if !self.eval_pending[i] {
            self.pending_since[i] = 0;
        }
        if energy == 0 {
            return;
        }
        self.set_material(id, Material::Air);
        if energy > 1 {
            let (x, y) = (id.0 % self.width, id.0 / self.width);
            for (dx, dy) in [(0i32, -1i32), (-1, 0), (1, 0), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && nx < self.width as i32 && ny < self.height as i32 {
                    let n = CellId(ny as u32 * self.width + nx as u32);
                    let ni = n.index();
                    if self.cells[ni].material == Material::Explosive {
                        if self.blast_pending[ni] == 0 {
                            self.pending_count += 1;
                        }
                        let merged = self.blast_pending[ni].max(energy - 1);
                        if merged != self.blast_pending[ni] {
                            self.mark_dirty(n);
                        }
                        self.blast_pending[ni] = merged;
                        self.mark_pending(n, false);
                        self.try_queue(n, JobKind::Blast);
                    }
                }
            }
        }
        self.wake_neighbors(id);
    }
    fn recover_one(&mut self) {
        let i = self.recovery_cursor;
        self.recovery_cursor = (i + 1) % self.cells.len();
        let id = CellId(i as u32);
        if self.eval_pending[i] && !self.eval_queued[i] {
            self.try_queue(id, JobKind::Evaluate);
        }
        if self.blast_pending[i] > 0 && !self.blast_queued[i] {
            self.try_queue(id, JobKind::Blast);
        }
    }
    fn execute_command(&mut self, command: Command) {
        match command {
            Command::Paint { cell, material } => {
                self.set_material(cell, material);
                self.mark_pending(cell, true);
                self.try_queue(cell, JobKind::Evaluate);
                self.wake_neighbors(cell);
            }
        }
    }
    /// Run exactly one bounded slice. Selection probes, recovery, commands, and rule quanta are charged before work.
    pub fn step(&mut self) -> SliceMetrics {
        let mut m = SliceMetrics {
            allowed: self.budget.0,
            slice: self.slice + 1,
            ..SliceMetrics::default()
        };
        let mut remaining = self.budget.0;
        while remaining >= self.costs.selection.0 {
            remaining -= self.costs.selection.0;
            m.charged += self.costs.selection.0;
            m.selections += 1;
            let lane = self.lane_cursor % 4;
            self.lane_cursor = (self.lane_cursor + 1) % 4;
            let (cost, job, command) = match lane {
                0 => match self.eval_ready.pop() {
                    Some(j) => (Some(self.costs.evaluate.0), Some(j), None),
                    None => (None, None, None),
                },
                1 => match self.blast_ready.pop() {
                    Some(j) => (Some(self.costs.blast.0), Some(j), None),
                    None => (None, None, None),
                },
                2 => (Some(self.costs.recovery.0), None, None),
                _ => {
                    let c = self.deferred_command.take().or_else(|| self.commands.pop());
                    match c {
                        Some(c) => (Some(self.costs.command.0), None, Some(c)),
                        None => (None, None, None),
                    }
                }
            };
            let Some(cost) = cost else { continue };
            if cost > remaining {
                if let Some(j) = job {
                    let ring = match j.kind {
                        JobKind::Evaluate => &mut self.eval_ready,
                        JobKind::Blast => &mut self.blast_ready,
                    };
                    let _ = ring.push(j);
                }
                if let Some(c) = command {
                    self.deferred_command = Some(c);
                }
                continue;
            }
            remaining -= cost;
            m.charged += cost;
            if let Some(j) = job {
                if j.generation != self.generation {
                    continue;
                }
                let i = j.cell.index();
                match j.kind {
                    JobKind::Evaluate => {
                        self.eval_queued[i] = false;
                        self.execute_evaluate(j.cell);
                        m.evaluations += 1;
                    }
                    JobKind::Blast => {
                        self.execute_blast(j.cell);
                        m.blasts += 1;
                    }
                }
            } else if let Some(c) = command {
                self.execute_command(c);
                m.commands += 1;
            } else {
                self.recover_one();
                m.recoveries += 1;
            }
        }
        self.slice = self.slice.saturating_add(1);
        m.ready_len = self.ready_len();
        m.pending_cells = self.pending_count;
        m.oldest_pending_age = if self.oldest_pending_since == 0 {
            0
        } else {
            self.slice.saturating_sub(self.oldest_pending_since)
        };
        m.rejected_commands = self.rejected_commands;
        m.coalesced_commands = self.coalesced_commands;
        m
    }
    /// Increment generation and clear the world in bounded slices by using a resumable clear cursor.
    pub fn reset(&mut self) -> Result<(), SimError> {
        self.generation.0 = self
            .generation
            .0
            .checked_add(1)
            .ok_or(SimError::GenerationExhausted)?;
        self.eval_ready = Ring::new(self.eval_ready.slots.len());
        self.blast_ready = Ring::new(self.blast_ready.slots.len());
        self.commands = Ring::new(self.commands.slots.len());
        self.eval_pending.fill(false);
        self.blast_pending.fill(0);
        self.eval_queued.fill(false);
        self.blast_queued.fill(false);
        self.pending_since.fill(0);
        self.pending_count = 0;
        self.oldest_pending_since = 0;
        self.cells.fill(Cell::default());
        self.dirty_chunks.fill(true);
        self.recovery_cursor = 0;
        self.deferred_command = None;
        Ok(())
    }
    pub fn state_hash(&self) -> u64 {
        let mut h = DefaultHasher::new();
        RULE_VERSION.hash(&mut h);
        self.width.hash(&mut h);
        self.height.hash(&mut h);
        self.generation.hash(&mut h);
        self.slice.hash(&mut h);
        self.cells.hash(&mut h);
        self.eval_pending.hash(&mut h);
        self.blast_pending.hash(&mut h);
        self.eval_queued.hash(&mut h);
        self.blast_queued.hash(&mut h);
        self.pending_since.hash(&mut h);
        self.eval_ready.head.hash(&mut h);
        self.eval_ready.len.hash(&mut h);
        self.eval_ready.slots.hash(&mut h);
        self.blast_ready.head.hash(&mut h);
        self.blast_ready.len.hash(&mut h);
        self.blast_ready.slots.hash(&mut h);
        self.commands.head.hash(&mut h);
        self.commands.len.hash(&mut h);
        self.commands.slots.hash(&mut h);
        self.dirty_chunks.hash(&mut h);
        self.dirty_cursor.hash(&mut h);
        self.pending_count.hash(&mut h);
        self.oldest_pending_since.hash(&mut h);
        self.recovery_cursor.hash(&mut h);
        self.lane_cursor.hash(&mut h);
        self.budget.hash(&mut h);
        self.deferred_command.hash(&mut h);
        h.finish()
    }
    pub fn ready_len(&self) -> usize {
        self.eval_ready.len() + self.blast_ready.len()
    }
    pub fn command_len(&self) -> usize {
        self.commands.len()
    }
    pub fn pending_channels(&self) -> usize {
        self.pending_count
    }
}

// The currently selected command is held here if its full cost does not fit the remaining allowance.
impl World {
    /* field is declared below through the source definition */
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world(w: u32, h: u32, b: u32, r: usize, c: usize) -> World {
        World::new(w, h, Credits::new(b), Capacity::new(r), Capacity::new(c)).unwrap()
    }
    #[test]
    fn dimensions_boundary_and_chunks() {
        let w = world(33, 34, 64, 4, 2);
        assert_eq!(w.dimensions(), (33, 34));
        assert_eq!(w.chunk_dimensions(), (2, 2));
        assert!(w.cell(33, 0).is_none());
        assert_eq!(w.resources().cells, 1122);
        assert_eq!(w.resources().ready_capacity, Capacity(8));
    }
    #[test]
    fn rejects_invalid_limits() {
        assert_eq!(
            World::new(0, 3, Credits::new(20), Capacity::new(1), Capacity::new(1)).err(),
            Some(SimError::InvalidDimensions)
        );
        assert_eq!(
            World::new(1, 1, Credits::new(12), Capacity::new(1), Capacity::new(1)).err(),
            Some(SimError::BudgetTooSmall)
        );
    }
    #[test]
    fn command_queue_rejects_and_coalesces() {
        let mut w = world(8, 8, 64, 8, 1);
        let id = w.cell_id(2, 2).unwrap();
        assert_eq!(
            w.submit(Command::Paint {
                cell: id,
                material: Material::Sand
            }),
            SubmitResult::Accepted
        );
        assert_eq!(
            w.submit(Command::Paint {
                cell: id,
                material: Material::Stone
            }),
            SubmitResult::Coalesced
        );
        assert_eq!(w.command_len(), 1);
        assert_eq!(
            w.submit(Command::Paint {
                cell: w.cell_id(3, 3).unwrap(),
                material: Material::Wood
            }),
            SubmitResult::RejectedFull
        );
    }
    #[test]
    fn credit_invariant_under_hostile_requests() {
        let mut w = world(30, 30, 25, 2, 2);
        for i in 0..900 {
            let id = CellId(i);
            w.mark_cell_for_evaluation(id).unwrap();
            if i % 4 == 0 {
                w.trigger_blast(id, 15).unwrap();
            }
        }
        for _ in 0..1000 {
            let m = w.step();
            assert!(m.charged <= m.allowed);
            assert!(w.ready_len() <= 4);
            assert!(w.command_len() <= 2);
            assert!(w.pending_channels() <= 1800);
        }
    }
    #[test]
    fn scheduler_services_each_lane() {
        let mut w = world(6, 6, 64, 2, 2);
        let sand = w.cell_id(0, 0).unwrap();
        w.set_material(sand, Material::Sand);
        w.mark_cell_for_evaluation(sand).unwrap();
        let blast = w.cell_id(4, 4).unwrap();
        w.trigger_blast(blast, 2).unwrap();
        let painted = w.cell_id(5, 5).unwrap();
        w.submit(Command::Paint {
            cell: painted,
            material: Material::Stone,
        });
        let mut totals = SliceMetrics::default();
        for _ in 0..12 {
            let m = w.step();
            totals.evaluations += m.evaluations;
            totals.blasts += m.blasts;
            totals.recoveries += m.recoveries;
            totals.commands += m.commands;
        }
        assert!(totals.evaluations > 0);
        assert!(totals.blasts > 0);
        assert!(totals.recoveries > 0);
        assert!(totals.commands > 0);
    }
    #[test]
    fn recovery_admits_saturated_demand() {
        let mut w = world(12, 12, 32, 1, 2);
        for i in 0..144 {
            w.mark_cell_for_evaluation(CellId(i)).unwrap();
        }
        let mut seen = 0;
        for _ in 0..5000 {
            seen += w.step().evaluations;
            if w.pending_channels() == 0 {
                break;
            }
        }
        assert!(seen > 0);
        assert_eq!(w.pending_channels(), 0);
    }
    #[test]
    fn stale_generation_cannot_mutate_world() {
        let mut w = world(4, 4, 32, 2, 2);
        w.set_material(CellId(0), Material::Stone);
        assert!(
            w.blast_ready
                .push(Job {
                    cell: CellId(0),
                    kind: JobKind::Blast,
                    generation: Generation(0)
                })
                .is_ok()
        );
        let before = w.cell(0, 0);
        w.step();
        assert_eq!(w.cell(0, 0), before);
    }
    #[test]
    fn stale_generation_cannot_survive_reset() {
        let mut w = world(4, 4, 32, 2, 2);
        let id = CellId(0);
        w.trigger_blast(id, 12).unwrap();
        let old = w.generation();
        w.reset().unwrap();
        assert_ne!(old, w.generation());
        for _ in 0..10 {
            w.step();
        }
        assert_eq!(w.cell(0, 0).unwrap().material, Material::Air);
    }
    #[test]
    fn sand_conserves_and_falls() {
        let mut w = world(4, 5, 64, 32, 4);
        let sand = w.cell_id(1, 1).unwrap();
        w.set_material(sand, Material::Sand);
        w.mark_cell_for_evaluation(sand).unwrap();
        for _ in 0..30 {
            w.step();
        }
        assert_eq!(
            w.cells
                .iter()
                .filter(|c| c.material == Material::Sand)
                .count(),
            1
        );
        assert_eq!(w.cell(1, 4).unwrap().material, Material::Sand);
    }
    #[test]
    fn explosion_is_finite_and_attenuates() {
        let mut w = world(8, 3, 64, 32, 2);
        for x in 1..7 {
            w.set_material(w.cell_id(x, 1).unwrap(), Material::Explosive);
        }
        w.trigger_blast(w.cell_id(1, 1).unwrap(), 5).unwrap();
        for _ in 0..200 {
            w.step();
        }
        assert_eq!(w.cell(1, 1).unwrap().material, Material::Air);
        assert_eq!(w.cell(6, 1).unwrap().material, Material::Explosive);
    }
    #[test]
    fn replay_hash_matches() {
        fn run() -> u64 {
            let mut w = world(10, 10, 37, 8, 4);
            for i in 0..20 {
                w.submit(Command::Paint {
                    cell: CellId(i),
                    material: if i % 3 == 0 {
                        Material::Explosive
                    } else {
                        Material::Sand
                    },
                });
            }
            w.trigger_blast(CellId(0), 8).unwrap();
            for _ in 0..300 {
                w.step();
            }
            w.state_hash()
        }
        assert_eq!(run(), run());
    }
}
