//! Bounded-work cellular simulation core. Independent of windows, graphics, and wall clocks.

pub mod fixtures;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub const CHUNK_SIDE: u32 = 32;
pub const DEFAULT_READY_CAPACITY: usize = 32_768;
pub const DEFAULT_COMMAND_CAPACITY: usize = 256;
pub const FOCUS_REGION_CAPACITY: usize = 8;
pub const ACTION_RECORD_CAPACITY: usize = 256;
pub const PLAYER_ACTION_FOCUS_LIFETIME_SLICES: u16 = 8;
pub const RULE_VERSION: u32 = 6;
pub const MAX_QUANTUM_COST: u32 = 24;
pub const MAX_BUDGET_CREDITS: u32 = 10_000_000;
pub const APPLICATION_CPU_STORAGE_LIMIT: usize = 256 * 1024 * 1024;
const VISUALIZED_CELLS_PER_SLICE: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Material {
    Air = 0,
    Stone = 1,
    Wood = 2,
    Sand = 3,
    Explosive = 4,
    Water = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct CellId(u32);
impl CellId {
    pub const fn index(self) -> usize {
        self.0 as usize
    }
    pub(crate) const fn from_index(index: usize) -> Self {
        Self(index as u32)
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SchedulerPolicy {
    Bounded,
    Traditional,
}
impl SchedulerPolicy {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Bounded => "bounded",
            Self::Traditional => "traditional",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CellView {
    pub material: Material,
    /// Pending blast energy, preserved for the existing renderer-facing view.
    pub state: u8,
    pub burning: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct Cell {
    material: Material,
    burning: u8,
}
impl Default for Cell {
    fn default() -> Self {
        Self {
            material: Material::Air,
            burning: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SimError {
    InvalidDimensions,
    WorldTooLarge,
    BudgetTooSmall,
    BudgetTooLarge,
    CapacityZero,
    GenerationExhausted,
    OutOfBounds,
    ResetInProgress,
    FixtureInProgress,
    CommandCapacityExceeded,
    ReadyCapacityExceeded,
    ResourceLimitExceeded,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
#[repr(C)]
struct PendingCell {
    flags: u8,
    blast: u8,
    paint_slot: u8,
}

// The traditional frontier packs evaluation and blast channels into one byte per cell.
const CAPTURED_EVAL: u8 = 0x80;
const CAPTURED_BLAST: u8 = 0x0f;
impl PendingCell {
    const EVAL_PENDING: u8 = 1;
    const EVAL_QUEUED: u8 = 2;
    const BLAST_QUEUED: u8 = 4;
    const PAINT_QUEUED: u8 = 8;
    const EVAL_FOCUS_QUEUED: u8 = 16;
    const BLAST_FOCUS_QUEUED: u8 = 32;
    fn has(self, flag: u8) -> bool {
        self.flags & flag != 0
    }
    fn set(&mut self, flag: u8, value: bool) {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceTable {
    pub cells: usize,
    pub cell_bytes: usize,
    pub pending_bytes: usize,
    pub captured_frontier_bytes: usize,
    pub ready_bytes: usize,
    pub focus_ready_bytes: usize,
    pub action_record_bytes: usize,
    pub command_bytes: usize,
    pub chunk_bytes: usize,
    pub visualization_bytes: usize,
    pub total_bytes: usize,
    pub ready_capacity: Capacity,
    pub focus_ready_capacity: Capacity,
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
    pub reset_cell: Credits,
    pub fixture_cell: Credits,
}
impl Default for CostContract {
    fn default() -> Self {
        Self {
            selection: Credits(1),
            evaluate: Credits(24),
            blast: Credits(24),
            recovery: Credits(4),
            command: Credits(12),
            reset_cell: Credits(4),
            fixture_cell: Credits(crate::fixtures::FIXTURE_CELL_COST),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum JobKind {
    Evaluate,
    Blast,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
struct FocusRegion {
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
    expires_after: u64,
}
impl FocusRegion {
    fn contains(self, x: u32, y: u32, slice: u64) -> bool {
        self.expires_after > slice
            && x >= self.min_x
            && x < self.max_x
            && y >= self.min_y
            && y < self.max_y
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct ActionRecord {
    pub sequence: u64,
    pub admitted_slice: u64,
    pub action_applied_slice: Option<u64>,
    pub first_effect_slice: Option<u64>,
    pub local_settle_slice: Option<u64>,
    pub target: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FocusRegionView {
    pub min_chunk_x: u32,
    pub min_chunk_y: u32,
    pub max_chunk_x: u32,
    pub max_chunk_y: u32,
    pub remaining_slices: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ServiceShares {
    pub focus_percent: u8,
    pub background_min_percent: u8,
}
impl Default for ServiceShares {
    fn default() -> Self {
        Self {
            focus_percent: 50,
            background_min_percent: 20,
        }
    }
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
        self.push_index(value).map(|_| ())
    }
    fn push_index(&mut self, value: T) -> Result<usize, T> {
        if self.len == self.slots.len() {
            return Err(value);
        }
        let tail = (self.head + self.len) % self.slots.len();
        self.slots[tail] = Some(value);
        self.len += 1;
        Ok(tail)
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
    fn clear(&mut self) {
        self.head = 0;
        self.len = 0;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Command {
    Paint {
        cell: CellId,
        material: Material,
    },
    Ignite {
        cell: CellId,
    },
    Detonate {
        cell: CellId,
        energy: u8,
    },
    FocusViewport {
        min_chunk_x: u32,
        min_chunk_y: u32,
        max_chunk_x: u32,
        max_chunk_y: u32,
        lifetime_slices: u16,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubmitResult {
    Accepted,
    Coalesced,
    RejectedFull,
    ResetInProgress,
    FixtureInProgress,
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
    pub focus_evaluations: u32,
    pub focus_blasts: u32,
    pub background_evaluations: u32,
    pub background_blasts: u32,
    pub rejected_commands: u64,
    pub coalesced_commands: u64,
    pub ready_len: usize,
    pub pending_cells: usize,
    pub oldest_pending_age: u64,
    pub slice: u64,
    pub reset_cells: u32,
    pub reset_in_progress: bool,
    pub prepared_cells: u32,
    pub fixture_in_progress: bool,
}

/// Fixed-layout simulation world. Total storage is exposed by [`World::resources`].
pub struct World {
    width: u32,
    height: u32,
    chunks_x: u32,
    chunks_y: u32,
    cells: Vec<Cell>,
    pending: Vec<PendingCell>,
    reset_cursor: Option<usize>,
    pending_count: usize,
    oldest_pending_since: u64,
    eval_ready: Ring<Job>,
    blast_ready: Ring<Job>,
    focus_eval_ready: Ring<Job>,
    focus_blast_ready: Ring<Job>,
    focus_regions: [FocusRegion; FOCUS_REGION_CAPACITY],
    focus_enabled: bool,
    shares: ServiceShares,
    action_records: Vec<Option<ActionRecord>>,
    action_cursor: usize,
    action_sequence: u64,
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
    captured_frontier: Vec<u8>,
    dirty_cursor: usize,
    deferred_command: Option<Command>,
    fixture: Option<fixtures::PreparationCursor>,
    fixture_waiting: Option<fixtures::ScenarioDescriptor>,
    fixture_status: fixtures::FixtureProgress,
    policy: SchedulerPolicy,
    last_executed_cells: [u32; VISUALIZED_CELLS_PER_SLICE],
    last_executed_len: usize,
    last_executed_count: u32,
}

impl World {
    pub fn new(
        width: u32,
        height: u32,
        budget: Credits,
        ready_capacity: Capacity,
        command_capacity: Capacity,
    ) -> Result<Self, SimError> {
        Self::new_with_policy(
            width,
            height,
            budget,
            ready_capacity,
            command_capacity,
            SchedulerPolicy::Bounded,
        )
    }
    pub fn new_with_policy(
        width: u32,
        height: u32,
        budget: Credits,
        ready_capacity: Capacity,
        command_capacity: Capacity,
        policy: SchedulerPolicy,
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
        if budget.0 < 25 {
            return Err(SimError::BudgetTooSmall);
        }
        if budget.0 > MAX_BUDGET_CREDITS {
            return Err(SimError::BudgetTooLarge);
        }
        if ready_capacity.0 == 0 || command_capacity.0 == 0 {
            return Err(SimError::CapacityZero);
        }
        if ready_capacity.0 > DEFAULT_READY_CAPACITY {
            return Err(SimError::ReadyCapacityExceeded);
        }
        if command_capacity.0 > DEFAULT_COMMAND_CAPACITY {
            return Err(SimError::CommandCapacityExceeded);
        }
        let chunks_x = width.div_ceil(CHUNK_SIDE);
        let chunks_y = height.div_ceil(CHUNK_SIDE);
        let chunk_count = (chunks_x as usize)
            .checked_mul(chunks_y as usize)
            .ok_or(SimError::WorldTooLarge)?;
        let resource_bytes = count
            .checked_mul(
                std::mem::size_of::<Cell>()
                    + std::mem::size_of::<PendingCell>()
                    + std::mem::size_of::<u8>(),
            )
            .and_then(|bytes| {
                bytes.checked_add(
                    (ready_capacity.0 * 4).checked_mul(std::mem::size_of::<Option<Job>>())?,
                )
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    ACTION_RECORD_CAPACITY
                        .checked_mul(std::mem::size_of::<Option<ActionRecord>>())?,
                )
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    command_capacity
                        .0
                        .checked_mul(std::mem::size_of::<Option<Command>>())?,
                )
            })
            .and_then(|bytes| {
                bytes.checked_add(chunk_count.checked_mul(std::mem::size_of::<bool>())?)
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    VISUALIZED_CELLS_PER_SLICE.checked_mul(std::mem::size_of::<u32>())?,
                )
            })
            .ok_or(SimError::WorldTooLarge)?;
        if resource_bytes > APPLICATION_CPU_STORAGE_LIMIT {
            return Err(SimError::ResourceLimitExceeded);
        }
        Ok(Self {
            width,
            height,
            chunks_x,
            chunks_y,
            cells: vec![Cell::default(); count],
            pending: vec![PendingCell::default(); count],
            reset_cursor: None,
            pending_count: 0,
            oldest_pending_since: 0,
            eval_ready: Ring::new(ready_capacity.0),
            blast_ready: Ring::new(ready_capacity.0),
            focus_eval_ready: Ring::new(ready_capacity.0),
            focus_blast_ready: Ring::new(ready_capacity.0),
            focus_regions: [FocusRegion::default(); FOCUS_REGION_CAPACITY],
            focus_enabled: true,
            shares: ServiceShares::default(),
            action_records: vec![None; ACTION_RECORD_CAPACITY],
            action_cursor: 0,
            action_sequence: 0,
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
            captured_frontier: vec![0; count],
            dirty_cursor: 0,
            deferred_command: None,
            fixture: None,
            fixture_waiting: None,
            fixture_status: fixtures::FixtureProgress::default(),
            policy,
            last_executed_cells: [u32::MAX; VISUALIZED_CELLS_PER_SLICE],
            last_executed_len: 0,
            last_executed_count: 0,
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
    /// Change the bounded slice allowance without changing any queued work.
    pub fn set_budget(&mut self, budget: Credits) -> Result<(), SimError> {
        if budget.0 < (self.costs.selection.0 + MAX_QUANTUM_COST) {
            return Err(SimError::BudgetTooSmall);
        }
        if budget.0 > MAX_BUDGET_CREDITS {
            return Err(SimError::BudgetTooLarge);
        }
        self.budget = budget;
        Ok(())
    }
    pub fn policy(&self) -> SchedulerPolicy {
        self.policy
    }
    /// Bounded sample of cells executed during the most recent slice.
    pub fn cell_executed_last(&self, id: CellId) -> bool {
        self.last_executed_cells[..self.last_executed_len].contains(&id.0)
    }
    /// Number of cells recorded for the most recent-slice overlay (at most 128).
    pub fn visualized_cell_count(&self) -> usize {
        self.last_executed_len
    }
    /// Total cell quanta executed in the most recent slice, including any beyond the visualization sample.
    pub fn executed_quantum_count(&self) -> u32 {
        self.last_executed_count
    }
    fn record_executed_cell(&mut self, id: CellId) {
        if self.last_executed_len < VISUALIZED_CELLS_PER_SLICE {
            self.last_executed_cells[self.last_executed_len] = id.0;
            self.last_executed_len += 1;
        } else {
            let slot = self.last_executed_count as usize % VISUALIZED_CELLS_PER_SLICE;
            self.last_executed_cells[slot] = id.0;
        }
        self.last_executed_count = self.last_executed_count.saturating_add(1);
    }
    pub fn set_focus_enabled(&mut self, enabled: bool) {
        self.focus_enabled = enabled;
    }
    pub fn set_service_shares(&mut self, shares: ServiceShares) -> Result<(), SimError> {
        if shares.focus_percent == 0
            || shares.focus_percent >= 100
            || shares.background_min_percent == 0
            || shares.focus_percent as u16 + shares.background_min_percent as u16 > 100
        {
            return Err(SimError::BudgetTooSmall);
        }
        self.shares = shares;
        Ok(())
    }
    pub fn service_shares(&self) -> ServiceShares {
        self.shares
    }
    pub fn active_focus_regions(&self) -> impl Iterator<Item = FocusRegionView> + '_ {
        self.focus_regions.iter().filter_map(|region| {
            (self.focus_enabled && region.expires_after > self.slice).then_some(FocusRegionView {
                min_chunk_x: region.min_x,
                min_chunk_y: region.min_y,
                max_chunk_x: region.max_x,
                max_chunk_y: region.max_y,
                remaining_slices: region.expires_after.saturating_sub(self.slice),
            })
        })
    }
    pub fn action_records(&self) -> impl Iterator<Item = ActionRecord> + '_ {
        self.action_records.iter().flatten().copied()
    }
    pub fn latest_action_sequence(&self) -> Option<u64> {
        (self.action_sequence > 0).then_some(self.action_sequence)
    }
    pub fn action_record(&self, sequence: u64) -> Option<ActionRecord> {
        self.action_records
            .iter()
            .flatten()
            .find(|record| record.sequence == sequence)
            .copied()
    }
    fn chunk_xy(&self, id: CellId) -> (u32, u32) {
        let x = id.0 % self.width;
        let y = id.0 / self.width;
        (x / CHUNK_SIDE, y / CHUNK_SIDE)
    }
    fn is_focused(&self, id: CellId) -> bool {
        if !self.focus_enabled {
            return false;
        }
        let (x, y) = self.chunk_xy(id);
        self.focus_regions
            .iter()
            .any(|r| r.contains(x, y, self.slice))
    }
    fn focus_cell(&mut self, id: CellId, lifetime: u16) {
        if !self.focus_enabled {
            return;
        }
        let (x, y) = self.chunk_xy(id);
        let region = FocusRegion {
            min_x: x.saturating_sub(1),
            min_y: y.saturating_sub(1),
            max_x: (x + 2).min(self.chunks_x),
            max_y: (y + 2).min(self.chunks_y),
            expires_after: self.slice.saturating_add(lifetime.max(1) as u64 + 1),
        };
        let slot = self
            .focus_regions
            .iter()
            .position(|r| r.expires_after <= self.slice)
            .unwrap_or_else(|| {
                self.focus_regions
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, r)| r.expires_after)
                    .map_or(0, |(i, _)| i)
            });
        self.focus_regions[slot] = region;
        // Recovery promotes deferred channels incrementally; no chunk/world scan occurs here.
    }
    pub fn slice_index(&self) -> u64 {
        self.slice
    }
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }
    pub fn start_fixture(
        &mut self,
        descriptor: fixtures::ScenarioDescriptor,
    ) -> Result<(), fixtures::FixtureError> {
        self.start_fixture_with_policy(descriptor, SchedulerPolicy::Bounded)
    }
    pub fn start_fixture_with_policy(
        &mut self,
        descriptor: fixtures::ScenarioDescriptor,
        policy: SchedulerPolicy,
    ) -> Result<(), fixtures::FixtureError> {
        if self.reset_cursor.is_some() {
            return Err(fixtures::FixtureError::ResetInProgress);
        }
        if self.fixture.is_some() || self.fixture_waiting.is_some() {
            return Err(fixtures::FixtureError::AlreadyPreparing);
        }
        if descriptor != fixtures::ScenarioDescriptor::get(descriptor.id) {
            return Err(fixtures::FixtureError::InvalidDescriptor);
        }
        if descriptor.id == fixtures::FixtureId::TinyCapacity
            && (self.eval_ready.slots.len() != 2
                || self.blast_ready.slots.len() != 2
                || self.commands.slots.len() != 2)
        {
            return Err(fixtures::FixtureError::CapacityMismatch);
        }
        self.reset().map_err(fixtures::FixtureError::from)?;
        self.policy = policy;
        self.fixture_waiting = Some(descriptor);
        self.fixture_status = fixtures::FixtureProgress {
            descriptor: Some(descriptor),
            total_cells: self.cells.len(),
            ..fixtures::FixtureProgress::default()
        };
        Ok(())
    }
    pub fn fixture_progress(&self) -> Option<fixtures::FixtureProgress> {
        self.fixture
            .map(fixtures::PreparationCursor::progress)
            .or_else(|| self.fixture_status.descriptor.map(|_| self.fixture_status))
    }
    pub fn cancel_fixture(&mut self) -> Option<fixtures::FixtureProgress> {
        let had_cursor = self.fixture.is_some();
        let cursor_progress = self
            .fixture
            .take()
            .map(fixtures::PreparationCursor::progress);
        let pending = self.fixture_waiting.take().is_some();
        if !had_cursor && !pending {
            return None;
        }
        let mut progress = cursor_progress.unwrap_or(self.fixture_status);
        progress.cancelled = true;
        progress.complete = false;
        self.fixture_status = progress;
        Some(progress)
    }
    pub fn resources(&self) -> ResourceTable {
        ResourceTable {
            cells: self.cells.len(),
            cell_bytes: std::mem::size_of::<Cell>(),
            pending_bytes: self.pending.len() * std::mem::size_of::<PendingCell>(),
            captured_frontier_bytes: self.captured_frontier.len(),
            ready_bytes: (self.eval_ready.slots.len() + self.blast_ready.slots.len())
                * std::mem::size_of::<Option<Job>>(),
            focus_ready_bytes: (self.focus_eval_ready.slots.len()
                + self.focus_blast_ready.slots.len())
                * std::mem::size_of::<Option<Job>>(),
            action_record_bytes: self.action_records.len()
                * std::mem::size_of::<Option<ActionRecord>>(),
            command_bytes: self.commands.slots.len() * std::mem::size_of::<Option<Command>>(),
            chunk_bytes: self.dirty_chunks.len() * std::mem::size_of::<bool>(),
            visualization_bytes: VISUALIZED_CELLS_PER_SLICE * std::mem::size_of::<u32>(),
            total_bytes: self.cells.len() * std::mem::size_of::<Cell>()
                + self.pending.len() * std::mem::size_of::<PendingCell>()
                + self.captured_frontier.len()
                + (self.eval_ready.slots.len()
                    + self.blast_ready.slots.len()
                    + self.focus_eval_ready.slots.len()
                    + self.focus_blast_ready.slots.len())
                    * std::mem::size_of::<Option<Job>>()
                + self.action_records.len() * std::mem::size_of::<Option<ActionRecord>>()
                + self.commands.slots.len() * std::mem::size_of::<Option<Command>>()
                + self.dirty_chunks.len() * std::mem::size_of::<bool>()
                + VISUALIZED_CELLS_PER_SLICE * std::mem::size_of::<u32>(),
            ready_capacity: Capacity(self.eval_ready.slots.len() + self.blast_ready.slots.len()),
            focus_ready_capacity: Capacity(
                self.focus_eval_ready.slots.len() + self.focus_blast_ready.slots.len(),
            ),
            command_capacity: Capacity(self.commands.slots.len()),
            chunk_count: self.dirty_chunks.len(),
        }
    }
    pub fn cell(&self, x: u32, y: u32) -> Option<CellView> {
        let id = self.cell_id(x, y)?;
        let c = self.cells[id.index()];
        Some(CellView {
            material: c.material,
            state: self.pending[id.index()].blast,
            burning: c.burning,
        })
    }
    /// Return whether this cell has deferred simulation work, using its fixed pending record.
    pub fn cell_pending(&self, x: u32, y: u32) -> Option<bool> {
        let id = self.cell_id(x, y)?;
        let pending = self.pending[id.index()];
        Some(pending.has(PendingCell::EVAL_PENDING) || pending.blast > 0)
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
        for _ in 0..self.dirty_chunks.len().min(limit).min(256) {
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
        if self.reset_cursor.is_some() {
            return SubmitResult::ResetInProgress;
        }
        if self.fixture.is_some() || self.fixture_waiting.is_some() {
            return SubmitResult::FixtureInProgress;
        }
        if let Command::FocusViewport {
            min_chunk_x,
            min_chunk_y,
            max_chunk_x,
            max_chunk_y,
            ..
        } = command
            && (min_chunk_x >= max_chunk_x
                || min_chunk_y >= max_chunk_y
                || min_chunk_x >= self.chunks_x
                || min_chunk_y >= self.chunks_y)
        {
            self.rejected_commands += 1;
            return SubmitResult::RejectedFull;
        }
        let target = match command {
            Command::Paint { cell, .. }
            | Command::Ignite { cell }
            | Command::Detonate { cell, .. } => Some(cell),
            Command::FocusViewport { .. } => None,
        };
        if let Some(cell) = target
            && cell.index() >= self.cells.len()
        {
            self.rejected_commands += 1;
            return SubmitResult::RejectedFull;
        }
        let index = target.map_or(0, CellId::index);
        if matches!(command, Command::Paint { .. })
            && self.pending[index].has(PendingCell::PAINT_QUEUED)
        {
            let cell = target.expect("paint command has target");
            let slot = self.pending[index].paint_slot as usize;
            if self
                .commands
                .slots
                .get(slot)
                .is_some_and(|entry| entry.is_some())
            {
                self.commands.slots[slot] = Some(command);
            } else if matches!(self.deferred_command, Some(Command::Paint { cell: deferred, .. }) if deferred == cell)
            {
                self.deferred_command = Some(command);
            } else {
                self.rejected_commands += 1;
                return SubmitResult::RejectedFull;
            }
            self.coalesced_commands += 1;
            self.register_action(cell);
            return SubmitResult::Coalesced;
        }
        match self.commands.push_index(command) {
            Ok(slot) => {
                if let Some(cell) = target {
                    if matches!(command, Command::Paint { .. }) {
                        self.pending[index].paint_slot = slot as u8;
                        self.pending[index].set(PendingCell::PAINT_QUEUED, true);
                    }
                    self.register_action(cell);
                }
                SubmitResult::Accepted
            }
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
        if self.reset_cursor.is_some() {
            return Err(SimError::ResetInProgress);
        }
        if self.fixture.is_some() || self.fixture_waiting.is_some() {
            return Err(SimError::FixtureInProgress);
        }
        if energy == 0 {
            return Ok(());
        }
        self.set_material(id, Material::Explosive);
        if self.pending[id.index()].blast == 0 {
            self.pending_count += 1;
        }
        let merged = self.pending[id.index()].blast.max(energy.min(15));
        if merged != self.pending[id.index()].blast {
            self.mark_dirty(id);
        }
        self.pending[id.index()].blast = merged;
        self.mark_pending(id, false);
        self.try_queue(id, JobKind::Blast);
        Ok(())
    }
    /// Ignite wood or an explosive using the same bounded cell-state channels as natural fire.
    pub fn ignite(&mut self, id: CellId) -> Result<(), SimError> {
        if self.reset_cursor.is_some() {
            return Err(SimError::ResetInProgress);
        }
        if self.fixture.is_some() || self.fixture_waiting.is_some() {
            return Err(SimError::FixtureInProgress);
        }
        if id.index() >= self.cells.len() {
            return Err(SimError::OutOfBounds);
        }
        match self.cells[id.index()].material {
            Material::Wood => self.ignite_wood(id),
            Material::Explosive => self.trigger_blast(id, 8)?,
            _ => {}
        }
        Ok(())
    }
    fn ignite_wood(&mut self, id: CellId) {
        let cell = &mut self.cells[id.index()];
        if cell.material == Material::Wood && cell.burning == 0 {
            cell.burning = 12;
            self.mark_dirty(id);
            self.mark_pending(id, true);
            self.try_queue(id, JobKind::Evaluate);
            self.wake_neighbors(id);
        }
    }
    pub fn mark_cell_for_evaluation(&mut self, id: CellId) -> Result<(), SimError> {
        if self.reset_cursor.is_some() {
            return Err(SimError::ResetInProgress);
        }
        if self.fixture.is_some() || self.fixture_waiting.is_some() {
            return Err(SimError::FixtureInProgress);
        }
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
            if !self.pending[i].has(PendingCell::EVAL_PENDING) {
                self.pending[i].set(PendingCell::EVAL_PENDING, true);
                self.pending_count += 1;
            }
        } else if self.pending[i].blast == 0 {
            return;
        }
        if self.oldest_pending_since == 0 {
            self.oldest_pending_since = self.slice.saturating_add(1);
        }
    }
    fn try_queue(&mut self, id: CellId, kind: JobKind) {
        let i = id.index();
        let focused = self.is_focused(id);
        let queued_flag = match (kind, focused) {
            (JobKind::Evaluate, true) => PendingCell::EVAL_FOCUS_QUEUED,
            (JobKind::Blast, true) => PendingCell::BLAST_FOCUS_QUEUED,
            (JobKind::Evaluate, false) => PendingCell::EVAL_QUEUED,
            (JobKind::Blast, false) => PendingCell::BLAST_QUEUED,
        };
        if self.pending[i].has(queued_flag) {
            return;
        }
        let job = Job {
            cell: id,
            kind,
            generation: self.generation,
        };
        if focused {
            let admitted_focus = match kind {
                JobKind::Evaluate => self.focus_eval_ready.push(job),
                JobKind::Blast => self.focus_blast_ready.push(job),
            }
            .is_ok();
            if admitted_focus {
                self.pending[i].set(queued_flag, true);
                return;
            }
        }
        let admitted_background = match kind {
            JobKind::Evaluate => self.eval_ready.push(job),
            JobKind::Blast => self.blast_ready.push(job),
        }
        .is_ok();
        if admitted_background {
            self.pending[i].set(
                match kind {
                    JobKind::Evaluate => PendingCell::EVAL_QUEUED,
                    JobKind::Blast => PendingCell::BLAST_QUEUED,
                },
                true,
            );
        }
    }
    fn wake_neighbors(&mut self, id: CellId) {
        for neighbor in self.neighbors(id).into_iter().flatten() {
            self.mark_pending(neighbor, true);
            self.try_queue(neighbor, JobKind::Evaluate);
        }
    }
    fn mark_dirty(&mut self, id: CellId) {
        let chunk =
            (id.0 / self.width) / CHUNK_SIDE * self.chunks_x + (id.0 % self.width) / CHUNK_SIDE;
        self.dirty_chunks[chunk as usize] = true;
    }
    pub(crate) fn prepare_fixture_cell(
        &mut self,
        id: CellId,
        material: Material,
        burning: u8,
        blast: u8,
        active: bool,
    ) -> Result<(), SimError> {
        if self.reset_cursor.is_some() {
            return Err(SimError::ResetInProgress);
        }
        if id.index() >= self.cells.len() {
            return Err(SimError::OutOfBounds);
        }
        self.set_material(id, material);
        self.cells[id.index()].burning = burning;
        self.mark_dirty(id);
        if (active && matches!(material, Material::Sand | Material::Water)) || burning > 0 {
            self.mark_pending(id, true);
            self.try_queue(id, JobKind::Evaluate);
        }
        if blast > 0 {
            self.pending[id.index()].blast = blast.min(15);
            self.pending_count += 1;
            self.mark_pending(id, false);
            self.try_queue(id, JobKind::Blast);
        }
        Ok(())
    }
    fn set_material(&mut self, id: CellId, material: Material) {
        let i = id.index();
        if self.cells[i].material != material {
            self.cells[i].material = material;
            self.cells[i].burning = 0;
            self.mark_dirty(id);
        }
    }
    fn execute_evaluate(&mut self, id: CellId) {
        let i = id.index();
        if self.pending[i].has(PendingCell::EVAL_PENDING) {
            self.pending[i].set(PendingCell::EVAL_PENDING, false);
            self.pending_count -= 1;
        }
        if self.pending_count == 0 {
            self.oldest_pending_since = 0;
        }
        self.execute_evaluate_rule(id);
    }
    fn execute_evaluate_captured(&mut self, id: CellId) {
        self.execute_evaluate_rule(id);
    }
    fn execute_evaluate_rule(&mut self, id: CellId) {
        let i = id.index();
        let material = self.cells[i].material;
        let (x, y) = (id.0 % self.width, id.0 / self.width);
        if material == Material::Sand || material == Material::Water {
            let below = (y + 1 < self.height).then_some(CellId(id.0 + self.width));
            if let Some(below) = below.filter(|b| self.cells[b.index()].material == Material::Air) {
                self.set_material(below, material);
                self.set_material(id, Material::Air);
                self.wake_neighbors(id);
                self.wake_neighbors(below);
                return;
            }
            if material == Material::Water {
                let first_left = (x.wrapping_add(y) & 1) == 0;
                let neighbors = self.neighbors(id);
                let sides = if first_left {
                    [neighbors[1], neighbors[2]]
                } else {
                    [neighbors[2], neighbors[1]]
                };
                for side in sides.into_iter().flatten() {
                    if self.cells[side.index()].material == Material::Air {
                        self.set_material(side, Material::Water);
                        self.set_material(id, Material::Air);
                        self.wake_neighbors(id);
                        self.wake_neighbors(side);
                        return;
                    }
                }
            }
        }
        if material == Material::Wood && self.cells[i].burning > 0 {
            let neighbors = self.neighbors(id);
            for neighbor in neighbors.into_iter().flatten() {
                match self.cells[neighbor.index()].material {
                    Material::Wood => self.ignite_wood(neighbor),
                    Material::Explosive => {
                        let _ = self.trigger_blast(neighbor, 8);
                    }
                    _ => {}
                }
            }
            if self.cells[i].burning > 0 {
                self.cells[i].burning -= 1;
                self.mark_dirty(id);
                if self.cells[i].burning == 0 {
                    self.set_material(id, Material::Air);
                } else {
                    self.mark_pending(id, true);
                    self.try_queue(id, JobKind::Evaluate);
                }
            }
        }
    }
    fn neighbors(&self, id: CellId) -> [Option<CellId>; 4] {
        let (x, y) = (id.0 % self.width, id.0 / self.width);
        [
            (y > 0).then(|| CellId(id.0 - self.width)),
            (x > 0).then(|| CellId(id.0 - 1)),
            (x + 1 < self.width).then(|| CellId(id.0 + 1)),
            (y + 1 < self.height).then(|| CellId(id.0 + self.width)),
        ]
    }
    fn execute_blast(&mut self, id: CellId) {
        let i = id.index();
        let energy = self.pending[i].blast;
        if energy > 0 {
            self.pending_count -= 1;
        }
        self.pending[i].blast = 0;
        self.pending[i].set(PendingCell::BLAST_QUEUED, false);
        if self.pending_count == 0 {
            self.oldest_pending_since = 0;
        }
        self.execute_blast_rule(id, energy);
    }
    fn execute_blast_captured(&mut self, id: CellId, energy: u8) {
        self.execute_blast_rule(id, energy);
    }
    fn execute_blast_rule(&mut self, id: CellId, energy: u8) {
        if energy == 0 {
            return;
        }
        self.mark_dirty(id);
        self.set_material(id, Material::Air);
        for neighbor in self.neighbors(id).into_iter().flatten() {
            let ni = neighbor.index();
            match self.cells[ni].material {
                Material::Explosive if energy > 1 => {
                    if self.pending[ni].blast == 0 {
                        self.pending_count += 1;
                    }
                    let merged = self.pending[ni].blast.max(energy - 1);
                    if merged != self.pending[ni].blast {
                        self.mark_dirty(neighbor);
                    }
                    self.pending[ni].blast = merged;
                    self.mark_pending(neighbor, false);
                    self.try_queue(neighbor, JobKind::Blast);
                }
                Material::Wood => self.ignite_wood(neighbor),
                _ => {}
            }
        }
        self.wake_neighbors(id);
    }
    fn recover_one(&mut self) {
        let i = self.recovery_cursor;
        self.recovery_cursor = (i + 1) % self.cells.len();
        let id = CellId(i as u32);
        if self.pending[i].has(PendingCell::EVAL_PENDING) {
            if self.is_focused(id) && !self.pending[i].has(PendingCell::EVAL_FOCUS_QUEUED) {
                let job = Job {
                    cell: id,
                    kind: JobKind::Evaluate,
                    generation: self.generation,
                };
                if self.focus_eval_ready.push(job).is_ok() {
                    self.pending[i].set(PendingCell::EVAL_FOCUS_QUEUED, true);
                }
            } else if !self.pending[i].has(PendingCell::EVAL_QUEUED)
                && !self.pending[i].has(PendingCell::EVAL_FOCUS_QUEUED)
            {
                self.try_queue(id, JobKind::Evaluate);
            }
        }
        if self.pending[i].blast > 0 {
            if self.is_focused(id) && !self.pending[i].has(PendingCell::BLAST_FOCUS_QUEUED) {
                let job = Job {
                    cell: id,
                    kind: JobKind::Blast,
                    generation: self.generation,
                };
                if self.focus_blast_ready.push(job).is_ok() {
                    self.pending[i].set(PendingCell::BLAST_FOCUS_QUEUED, true);
                }
            } else if !self.pending[i].has(PendingCell::BLAST_QUEUED)
                && !self.pending[i].has(PendingCell::BLAST_FOCUS_QUEUED)
            {
                self.try_queue(id, JobKind::Blast);
            }
        }
    }
    fn register_action(&mut self, cell: CellId) {
        self.action_sequence = self.action_sequence.wrapping_add(1).max(1);
        self.action_records[self.action_cursor] = Some(ActionRecord {
            sequence: self.action_sequence,
            admitted_slice: self.slice.saturating_add(1),
            action_applied_slice: None,
            first_effect_slice: None,
            local_settle_slice: None,
            target: cell.0,
        });
        self.action_cursor = (self.action_cursor + 1) % self.action_records.len();
    }
    fn note_action_effect(&mut self, cell: CellId) {
        let slice = self.slice.saturating_add(1);
        for record in self.action_records.iter_mut().flatten() {
            let target = record.target;
            let (tx, ty) = (target % self.width, target / self.width);
            let (x, y) = (cell.0 % self.width, cell.0 / self.width);
            if tx.abs_diff(x) <= 1
                && ty.abs_diff(y) <= 1
                && record.action_applied_slice.is_some()
                && record.first_effect_slice.is_none()
            {
                record.first_effect_slice = Some(slice);
            }
        }
    }
    fn update_action_settle(&mut self) {
        let slice = self.slice;
        for index in 0..self.action_records.len() {
            let Some(record) = self.action_records[index] else {
                continue;
            };
            if record.first_effect_slice.is_none() || record.local_settle_slice.is_some() {
                continue;
            }
            let x = record.target % self.width;
            let y = record.target / self.width;
            let mut active = false;
            for ny in y.saturating_sub(1)..=(y + 1).min(self.height - 1) {
                for nx in x.saturating_sub(1)..=(x + 1).min(self.width - 1) {
                    let p = self.pending[(ny * self.width + nx) as usize];
                    active |= p.has(PendingCell::EVAL_PENDING) || p.blast > 0;
                }
            }
            if !active {
                self.action_records[index]
                    .as_mut()
                    .unwrap()
                    .local_settle_slice = Some(slice);
            }
        }
    }
    fn note_action_applied(&mut self, cell: CellId) {
        let slice = self.slice.saturating_add(1);
        for record in self.action_records.iter_mut().flatten() {
            if record.target == cell.0 && record.action_applied_slice.is_none() {
                record.action_applied_slice = Some(slice);
            }
        }
    }
    fn set_focus_region(&mut self, min_x: u32, min_y: u32, max_x: u32, max_y: u32, lifetime: u16) {
        if !self.focus_enabled {
            return;
        }
        let region = FocusRegion {
            min_x: min_x.min(self.chunks_x),
            min_y: min_y.min(self.chunks_y),
            max_x: max_x.min(self.chunks_x),
            max_y: max_y.min(self.chunks_y),
            expires_after: self.slice.saturating_add(lifetime.max(1) as u64 + 1),
        };
        if region.min_x >= region.max_x || region.min_y >= region.max_y {
            return;
        }
        let slot = self
            .focus_regions
            .iter()
            .position(|r| r.expires_after <= self.slice)
            .unwrap_or_else(|| {
                self.focus_regions
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, r)| r.expires_after)
                    .map_or(0, |(i, _)| i)
            });
        self.focus_regions[slot] = region;
    }
    fn execute_command(&mut self, command: Command) {
        match command {
            Command::Paint { cell, material } => {
                self.note_action_applied(cell);
                self.focus_cell(cell, PLAYER_ACTION_FOCUS_LIFETIME_SLICES);
                self.set_material(cell, material);
                self.mark_pending(cell, true);
                self.try_queue(cell, JobKind::Evaluate);
                self.wake_neighbors(cell);
            }
            Command::Ignite { cell } => {
                self.note_action_applied(cell);
                self.focus_cell(cell, PLAYER_ACTION_FOCUS_LIFETIME_SLICES);
                let _ = self.ignite(cell);
            }
            Command::Detonate { cell, energy } => {
                self.note_action_applied(cell);
                self.focus_cell(cell, PLAYER_ACTION_FOCUS_LIFETIME_SLICES);
                let _ = self.trigger_blast(cell, energy);
            }
            Command::FocusViewport {
                min_chunk_x,
                min_chunk_y,
                max_chunk_x,
                max_chunk_y,
                lifetime_slices,
            } => {
                self.set_focus_region(
                    min_chunk_x,
                    min_chunk_y,
                    max_chunk_x,
                    max_chunk_y,
                    lifetime_slices,
                );
            }
        }
    }
    /// Run exactly one policy slice. Traditional mode drains only the ready frontier captured at entry.
    pub fn step(&mut self) -> SliceMetrics {
        if self.policy == SchedulerPolicy::Traditional
            && self.reset_cursor.is_none()
            && self.fixture.is_none()
            && self.fixture_waiting.is_none()
        {
            return self.step_traditional();
        }
        self.step_bounded()
    }
    /// Bounded mode charges every probe and quantum before work.
    fn step_bounded(&mut self) -> SliceMetrics {
        self.last_executed_len = 0;
        self.last_executed_count = 0;
        let mut m = SliceMetrics {
            allowed: self.budget.0,
            slice: self.slice + 1,
            ..SliceMetrics::default()
        };
        let mut remaining = self.budget.0;
        if let Some(mut cursor) = self.reset_cursor {
            while cursor < self.cells.len()
                && remaining >= self.costs.selection.0 + self.costs.reset_cell.0
            {
                remaining -= self.costs.selection.0 + self.costs.reset_cell.0;
                m.charged += self.costs.selection.0 + self.costs.reset_cell.0;
                m.selections += 1;
                let id = CellId(cursor as u32);
                self.cells[cursor] = Cell::default();
                self.pending[cursor] = PendingCell::default();
                self.mark_dirty(id);
                cursor += 1;
                m.reset_cells += 1;
            }
            self.reset_cursor = (cursor < self.cells.len()).then_some(cursor);
            if self.reset_cursor.is_none()
                && let Some(descriptor) = self.fixture_waiting.take()
            {
                self.fixture = Some(fixtures::PreparationCursor::new(
                    descriptor,
                    self.width,
                    self.height,
                    self.generation.get(),
                ));
            }
            self.slice = self.slice.saturating_add(1);
            m.reset_in_progress = self.reset_cursor.is_some();
            m.fixture_in_progress = self.fixture.is_some() || self.fixture_waiting.is_some();
            m.ready_len = self.ready_len();
            m.pending_cells = self.pending_count;
            m.rejected_commands = self.rejected_commands;
            m.coalesced_commands = self.coalesced_commands;
            return m;
        }
        if self.fixture.is_some() {
            while remaining >= self.costs.selection.0 + self.costs.fixture_cell.0 {
                remaining -= self.costs.selection.0 + self.costs.fixture_cell.0;
                m.charged += self.costs.selection.0 + self.costs.fixture_cell.0;
                m.selections += 1;
                if let Some(mut cursor) = self.fixture.take() {
                    let before = cursor.progress().prepared_cells;
                    match cursor.advance_one(self) {
                        Ok(()) => {
                            if cursor.progress().prepared_cells > before {
                                m.prepared_cells += 1;
                            }
                            if cursor.finished() {
                                self.fixture_status = cursor.progress();
                                self.fixture = None;
                            } else {
                                self.fixture = Some(cursor);
                            }
                        }
                        Err(_) => {
                            self.fixture_status.cancelled = true;
                            self.fixture = None;
                        }
                    }
                }
                if self.fixture.is_none() {
                    break;
                }
            }
            self.slice = self.slice.saturating_add(1);
            m.fixture_in_progress = self.fixture.is_some();
            m.ready_len = self.ready_len();
            m.pending_cells = self.pending_count;
            m.rejected_commands = self.rejected_commands;
            m.coalesced_commands = self.coalesced_commands;
            return m;
        }
        let mut empty_lanes = 0u8;
        while remaining >= self.costs.selection.0 {
            remaining -= self.costs.selection.0;
            m.charged += self.costs.selection.0;
            m.selections += 1;
            let phase = self.lane_cursor;
            self.lane_cursor = (self.lane_cursor + 1) % 100;
            let focus_eval = self.focus_enabled && self.focus_eval_ready.len() > 0;
            let focus_blast = self.focus_enabled && self.focus_blast_ready.len() > 0;
            let focus_demand = focus_eval || focus_blast;
            let background_eval = self.eval_ready.len() > 0
                || (!self.focus_enabled && self.focus_eval_ready.len() > 0);
            let background_blast = self.blast_ready.len() > 0
                || (!self.focus_enabled && self.focus_blast_ready.len() > 0);
            let background_demand = background_eval
                || background_blast
                || self.pending_count > 0
                || self.command_len() > 0;
            let minimum_background_turn = (background_eval || background_blast)
                && phase >= self.shares.focus_percent
                && (phase as u16)
                    < self.shares.focus_percent as u16 + self.shares.background_min_percent as u16;
            let focus_turn = focus_demand
                && !minimum_background_turn
                && (phase < self.shares.focus_percent || !background_demand);
            let lane = if focus_turn {
                if focus_eval && focus_blast {
                    if phase & 1 == 0 { 0 } else { 1 }
                } else if focus_eval {
                    0
                } else {
                    1
                }
            } else if minimum_background_turn {
                if background_eval && background_blast {
                    if phase & 1 == 0 { 2 } else { 3 }
                } else if background_eval {
                    2
                } else {
                    3
                }
            } else {
                let demanded = [
                    background_eval,
                    background_blast,
                    self.pending_count > 0,
                    self.command_len() > 0,
                ];
                (0..4)
                    .map(|offset| (phase as usize % 4 + offset) % 4)
                    .find(|index| demanded[*index])
                    .map_or(6, |index| index as u8 + 2)
            };
            let pick_focus = lane == 0 || lane == 1;
            let (cost, job, command) = match lane {
                0 => self.focus_eval_ready.pop().map_or((None, None, None), |j| {
                    (Some(self.costs.evaluate.0), Some(j), None)
                }),
                1 => self
                    .focus_blast_ready
                    .pop()
                    .map_or((None, None, None), |j| {
                        (Some(self.costs.blast.0), Some(j), None)
                    }),
                2 => {
                    let background_first = self.focus_enabled || self.lane_cursor & 1 == 0;
                    let job = if background_first {
                        self.eval_ready
                            .pop()
                            .or_else(|| self.focus_eval_ready.pop())
                    } else {
                        self.focus_eval_ready
                            .pop()
                            .or_else(|| self.eval_ready.pop())
                    };
                    job.map_or((None, None, None), |j| {
                        (Some(self.costs.evaluate.0), Some(j), None)
                    })
                }
                3 => {
                    let background_first = self.focus_enabled || self.lane_cursor & 1 == 0;
                    let job = if background_first {
                        self.blast_ready
                            .pop()
                            .or_else(|| self.focus_blast_ready.pop())
                    } else {
                        self.focus_blast_ready
                            .pop()
                            .or_else(|| self.blast_ready.pop())
                    };
                    job.map_or((None, None, None), |j| {
                        (Some(self.costs.blast.0), Some(j), None)
                    })
                }
                4 if self.pending_count > 0 => (Some(self.costs.recovery.0), None, None),
                4 => (None, None, None),
                _ => {
                    let c = self.deferred_command.take().or_else(|| self.commands.pop());
                    c.map_or((None, None, None), |c| {
                        (Some(self.costs.command.0), None, Some(c))
                    })
                }
            };
            let Some(cost) = cost else {
                empty_lanes += 1;
                if empty_lanes >= 4 {
                    break;
                }
                continue;
            };
            empty_lanes = 0;
            if cost > remaining {
                if let Some(j) = job {
                    let ring = match (j.kind, pick_focus) {
                        (JobKind::Evaluate, true) => &mut self.focus_eval_ready,
                        (JobKind::Blast, true) => &mut self.focus_blast_ready,
                        (JobKind::Evaluate, false) => &mut self.eval_ready,
                        (JobKind::Blast, false) => &mut self.blast_ready,
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
                        self.record_executed_cell(j.cell);
                        let focused_job = pick_focus;
                        self.pending[i].set(
                            if focused_job {
                                PendingCell::EVAL_FOCUS_QUEUED
                            } else {
                                PendingCell::EVAL_QUEUED
                            },
                            false,
                        );
                        if !self.focus_enabled {
                            self.pending[i].set(PendingCell::EVAL_FOCUS_QUEUED, false);
                            self.pending[i].set(PendingCell::EVAL_QUEUED, false);
                        }
                        if self.pending[i].has(PendingCell::EVAL_PENDING) {
                            self.note_action_effect(j.cell);
                            self.execute_evaluate(j.cell);
                            m.evaluations += 1;
                            if focused_job {
                                m.focus_evaluations += 1;
                            } else {
                                m.background_evaluations += 1;
                            }
                        }
                    }
                    JobKind::Blast => {
                        self.record_executed_cell(j.cell);
                        let focused_job = pick_focus;
                        self.pending[i].set(
                            if focused_job {
                                PendingCell::BLAST_FOCUS_QUEUED
                            } else {
                                PendingCell::BLAST_QUEUED
                            },
                            false,
                        );
                        if !self.focus_enabled {
                            self.pending[i].set(PendingCell::BLAST_FOCUS_QUEUED, false);
                            self.pending[i].set(PendingCell::BLAST_QUEUED, false);
                        }
                        if self.pending[i].blast > 0 {
                            self.note_action_effect(j.cell);
                            self.execute_blast(j.cell);
                            m.blasts += 1;
                            if focused_job {
                                m.focus_blasts += 1;
                            } else {
                                m.background_blasts += 1;
                            }
                        }
                    }
                }
            } else if let Some(c) = command {
                if let Command::Paint { cell, .. } = c {
                    self.record_executed_cell(cell);
                    self.pending[cell.index()].set(PendingCell::PAINT_QUEUED, false);
                }
                self.execute_command(c);
                m.commands += 1;
            } else {
                self.recover_one();
                m.recoveries += 1;
            }
        }
        self.slice = self.slice.saturating_add(1);
        self.update_action_settle();
        m.ready_len = self.ready_len();
        m.pending_cells = self.pending_count;
        m.oldest_pending_age = if self.oldest_pending_since == 0 {
            0
        } else {
            self.slice.saturating_sub(self.oldest_pending_since)
        };
        m.rejected_commands = self.rejected_commands;
        m.coalesced_commands = self.coalesced_commands;
        m.fixture_in_progress = self.fixture.is_some() || self.fixture_waiting.is_some();
        m
    }
    /// Begin a generation-safe reset. Cell storage is cleared by charged, resumable quanta in `step`.
    pub fn reset(&mut self) -> Result<(), SimError> {
        if self.reset_cursor.is_some() {
            return Err(SimError::ResetInProgress);
        }
        self.generation.0 = self
            .generation
            .0
            .checked_add(1)
            .ok_or(SimError::GenerationExhausted)?;
        self.eval_ready.clear();
        self.blast_ready.clear();
        self.focus_eval_ready.clear();
        self.focus_blast_ready.clear();
        self.focus_regions = [FocusRegion::default(); FOCUS_REGION_CAPACITY];
        self.commands.clear();
        self.pending_count = 0;
        self.oldest_pending_since = 0;
        self.recovery_cursor = 0;
        self.deferred_command = None;
        if let Some(cursor) = self.fixture.take() {
            self.fixture_status = cursor.progress();
            self.fixture_status.cancelled = true;
            self.fixture_status.complete = false;
        } else if self.fixture_waiting.is_some() {
            self.fixture_status.cancelled = true;
            self.fixture_status.complete = false;
        } else {
            self.fixture_status = fixtures::FixtureProgress::default();
        }
        self.fixture_waiting = None;
        self.reset_cursor = Some(0);
        Ok(())
    }
    pub fn reset_progress(&self) -> Option<(usize, usize)> {
        self.reset_cursor.map(|cursor| (cursor, self.cells.len()))
    }
    pub fn reset_in_progress(&self) -> bool {
        self.reset_cursor.is_some()
    }
    fn step_traditional(&mut self) -> SliceMetrics {
        self.last_executed_len = 0;
        self.last_executed_count = 0;
        let mut metrics = SliceMetrics {
            allowed: self.budget.0,
            slice: self.slice + 1,
            ..SliceMetrics::default()
        };
        let command_frontier = self.command_len();

        // Move every pending channel into a fixed snapshot before executing any work.
        // This captures demand beyond both ready rings and keeps generated work for next slice.
        self.eval_ready.clear();
        self.blast_ready.clear();
        self.focus_eval_ready.clear();
        self.focus_blast_ready.clear();
        for i in 0..self.pending.len() {
            let pending = self.pending[i];
            let mut captured = 0;
            if pending.has(PendingCell::EVAL_PENDING) {
                captured |= CAPTURED_EVAL;
                self.pending_count -= 1;
            }
            if pending.blast > 0 {
                captured |= pending.blast & CAPTURED_BLAST;
                self.pending_count -= 1;
            }
            self.captured_frontier[i] = captured;
            self.pending[i].set(PendingCell::EVAL_PENDING, false);
            self.pending[i].blast = 0;
            self.pending[i].set(PendingCell::EVAL_QUEUED, false);
            self.pending[i].set(PendingCell::BLAST_QUEUED, false);
            self.pending[i].set(PendingCell::EVAL_FOCUS_QUEUED, false);
            self.pending[i].set(PendingCell::BLAST_FOCUS_QUEUED, false);
            metrics.charged += self.costs.selection.0 + self.costs.recovery.0;
            metrics.selections += 1;
            metrics.recoveries += 1;
        }
        if self.pending_count == 0 {
            self.oldest_pending_since = 0;
        }

        for i in 0..self.captured_frontier.len() {
            metrics.charged += self.costs.selection.0 + self.costs.recovery.0;
            metrics.selections += 1;
            metrics.recoveries += 1;
            let captured = std::mem::take(&mut self.captured_frontier[i]);
            let id = CellId::from_index(i);
            if captured & CAPTURED_EVAL != 0 {
                self.record_executed_cell(id);
                metrics.charged += self.costs.selection.0 + self.costs.evaluate.0;
                metrics.selections += 1;
                self.note_action_effect(id);
                self.execute_evaluate_captured(id);
                metrics.evaluations += 1;
                metrics.background_evaluations += 1;
            }
            let blast = captured & CAPTURED_BLAST;
            if blast > 0 {
                self.record_executed_cell(id);
                metrics.charged += self.costs.selection.0 + self.costs.blast.0;
                metrics.selections += 1;
                self.note_action_effect(id);
                self.execute_blast_captured(id, blast);
                metrics.blasts += 1;
                metrics.background_blasts += 1;
            }
        }

        for _ in 0..command_frontier {
            let Some(command) = self.deferred_command.take().or_else(|| self.commands.pop()) else {
                break;
            };
            metrics.charged += self.costs.selection.0 + self.costs.command.0;
            metrics.selections += 1;
            if let Command::Paint { cell, .. } = command {
                self.record_executed_cell(cell);
                self.pending[cell.index()].set(PendingCell::PAINT_QUEUED, false);
            }
            self.execute_command(command);
            metrics.commands += 1;
        }

        self.slice = self.slice.saturating_add(1);
        self.update_action_settle();
        metrics.ready_len = self.ready_len();
        metrics.pending_cells = self.pending_count;
        metrics.oldest_pending_age = if self.oldest_pending_since == 0 {
            0
        } else {
            self.slice.saturating_sub(self.oldest_pending_since)
        };
        metrics.rejected_commands = self.rejected_commands;
        metrics.coalesced_commands = self.coalesced_commands;
        metrics
    }
    pub fn state_hash(&self) -> u64 {
        let mut h = DefaultHasher::new();
        RULE_VERSION.hash(&mut h);
        self.width.hash(&mut h);
        self.height.hash(&mut h);
        self.generation.hash(&mut h);
        self.policy.hash(&mut h);
        self.slice.hash(&mut h);
        self.cells.hash(&mut h);
        self.pending.hash(&mut h);
        self.fixture.hash(&mut h);
        self.fixture_waiting.hash(&mut h);
        self.fixture_status.hash(&mut h);
        self.eval_ready.head.hash(&mut h);
        self.eval_ready.len.hash(&mut h);
        self.eval_ready.slots.hash(&mut h);
        self.focus_eval_ready.head.hash(&mut h);
        self.focus_eval_ready.len.hash(&mut h);
        self.focus_eval_ready.slots.hash(&mut h);
        self.focus_blast_ready.head.hash(&mut h);
        self.focus_blast_ready.len.hash(&mut h);
        self.focus_blast_ready.slots.hash(&mut h);
        self.focus_regions.hash(&mut h);
        self.focus_enabled.hash(&mut h);
        self.shares.hash(&mut h);
        self.action_records.hash(&mut h);
        self.action_cursor.hash(&mut h);
        self.action_sequence.hash(&mut h);
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
        self.reset_cursor.hash(&mut h);
        self.recovery_cursor.hash(&mut h);
        self.lane_cursor.hash(&mut h);
        self.budget.hash(&mut h);
        self.deferred_command.hash(&mut h);
        h.finish()
    }
    pub fn ready_len(&self) -> usize {
        self.eval_ready.len()
            + self.blast_ready.len()
            + self.focus_eval_ready.len()
            + self.focus_blast_ready.len()
    }
    pub fn command_len(&self) -> usize {
        self.commands.len() + usize::from(self.deferred_command.is_some())
    }
    pub fn pending_channels(&self) -> usize {
        self.pending_count
    }
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
        assert_eq!(w.resources().focus_ready_capacity, Capacity(8));
    }
    #[test]
    fn quiet_bounded_world_does_not_scan_recovery_without_pending_work() {
        let mut w = world(64, 64, 4_096, 64, 16);
        w.start_fixture(fixtures::ScenarioDescriptor::get(
            fixtures::FixtureId::QuietWorld,
        ))
        .unwrap();
        while w
            .fixture_progress()
            .is_some_and(|progress| !progress.complete)
        {
            w.step();
        }
        let metrics = w.step();
        assert_eq!(metrics.pending_cells, 0);
        assert_eq!(metrics.evaluations, 0);
        assert_eq!(metrics.blasts, 0);
        assert_eq!(metrics.recoveries, 0);
        assert_eq!(metrics.commands, 0);
    }

    #[test]
    fn budget_adjustment_preserves_the_minimum_quantum_contract() {
        let mut w = world(4, 4, 64, 2, 2);
        assert_eq!(
            w.set_budget(Credits::new(24)),
            Err(SimError::BudgetTooSmall)
        );
        assert_eq!(w.set_budget(Credits::new(25)), Ok(()));
        assert_eq!(
            w.set_budget(Credits::new(MAX_BUDGET_CREDITS + 1)),
            Err(SimError::BudgetTooLarge)
        );
        let metrics = w.step();
        assert_eq!(metrics.allowed, 25);
        assert!(metrics.charged <= metrics.allowed);
    }

    #[test]
    fn rejects_invalid_limits() {
        assert_eq!(
            World::new(0, 3, Credits::new(25), Capacity::new(1), Capacity::new(1)).err(),
            Some(SimError::InvalidDimensions)
        );
        assert_eq!(
            World::new(1, 1, Credits::new(24), Capacity::new(1), Capacity::new(1)).err(),
            Some(SimError::BudgetTooSmall)
        );
        assert_eq!(
            World::new(1, 1, Credits::new(25), Capacity::new(1), Capacity::new(257)).err(),
            Some(SimError::CommandCapacityExceeded)
        );
        assert_eq!(
            World::new(
                1,
                1,
                Credits::new(25),
                Capacity::new(DEFAULT_READY_CAPACITY + 1),
                Capacity::new(1)
            )
            .err(),
            Some(SimError::ReadyCapacityExceeded)
        );
        assert_eq!(
            World::new(
                60_000_000,
                1,
                Credits::new(25),
                Capacity::new(1),
                Capacity::new(1)
            )
            .err(),
            Some(SimError::ResourceLimitExceeded)
        );
        let costs = CostContract::default();
        assert_eq!(costs.blast.get(), MAX_QUANTUM_COST);
        assert!(costs.evaluate.get() <= MAX_QUANTUM_COST);
        assert!(costs.fixture_cell.get() <= MAX_QUANTUM_COST);
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
            assert!(w.ready_len() <= 8);
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
    fn focus_retains_credit_bound_and_background_minimum_service() {
        let mut w = world(64, 64, 26, 64, 64);
        let background_cell = CellId(0);
        let target = CellId(40 * 64 + 40);
        w.set_focus_region(1, 1, 2, 2, 1000);
        let mut background = 0;
        let mut focus = 0;
        for _ in 0..100 {
            w.mark_cell_for_evaluation(background_cell).unwrap();
            w.mark_cell_for_evaluation(target).unwrap();
            let metrics = w.step();
            background += metrics.background_evaluations + metrics.background_blasts;
            focus += metrics.focus_evaluations + metrics.focus_blasts;
            assert!(metrics.charged <= metrics.allowed);
        }
        assert!(focus > 0);
        assert!(
            background > 0,
            "background work must progress under sustained focus demand"
        );
        assert!(
            background * 100 >= (background + focus) * 20,
            "background share must meet its configured minimum: background={background}, focus={focus}"
        );
    }

    #[test]
    fn focus_expiry_and_replay_are_deterministic() {
        fn run() -> u64 {
            let mut w = world(64, 64, 128, 16, 8);
            let cmd = Command::FocusViewport {
                min_chunk_x: 0,
                min_chunk_y: 0,
                max_chunk_x: 1,
                max_chunk_y: 1,
                lifetime_slices: 2,
            };
            assert_eq!(w.submit(cmd), SubmitResult::Accepted);
            for _ in 0..20 {
                w.step();
            }
            w.state_hash()
        }
        assert_eq!(run(), run());
        let mut w = world(64, 64, 128, 16, 8);
        w.set_focus_region(0, 0, 1, 1, 2);
        let view = w.active_focus_regions().next().unwrap();
        assert_eq!(
            (
                view.min_chunk_x,
                view.min_chunk_y,
                view.max_chunk_x,
                view.max_chunk_y
            ),
            (0, 0, 1, 1)
        );
        assert_eq!(view.remaining_slices, 3);
        assert!(w.is_focused(CellId(0)));
        w.step();
        assert!(w.is_focused(CellId(0)));
        w.step();
        assert!(w.is_focused(CellId(0)));
        w.step();
        assert!(!w.is_focused(CellId(0)));
    }

    #[test]
    fn recovery_promotes_without_double_execution() {
        let mut w = world(64, 64, 256, 64, 8);
        let id = CellId(10 * 64 + 10);
        w.mark_cell_for_evaluation(id).unwrap();
        assert!(w.pending[id.index()].has(PendingCell::EVAL_QUEUED));
        w.set_focus_region(0, 0, 1, 1, 10);
        w.recovery_cursor = id.index();
        w.recover_one();
        assert!(w.pending[id.index()].has(PendingCell::EVAL_FOCUS_QUEUED));
        for _ in 0..20 {
            w.step();
        }
        assert_eq!(w.pending_channels(), 0);
        assert_eq!(w.focus_eval_ready.len(), 0);
        assert_eq!(w.eval_ready.len(), 0);
    }

    #[test]
    fn action_latency_records_are_fixed_and_observable() {
        let mut w = world(8, 8, 256, 16, 8);
        let id = w.cell_id(4, 4).unwrap();
        assert_eq!(
            w.submit(Command::Paint {
                cell: id,
                material: Material::Stone
            }),
            SubmitResult::Accepted
        );
        let admitted = w.action_records().next().unwrap().admitted_slice;
        for _ in 0..20 {
            w.step();
        }
        let record = w.action_records().next().unwrap();
        assert_eq!(record.admitted_slice, admitted);
        assert!(record.action_applied_slice.is_some());
        assert!(record.first_effect_slice.is_some());
        assert!(record.local_settle_slice.is_some());
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
        assert_eq!(w.reset_progress(), Some((0, 16)));
        assert_eq!(
            w.submit(Command::Paint {
                cell: id,
                material: Material::Stone
            }),
            SubmitResult::ResetInProgress
        );
        while w.reset_in_progress() {
            let metrics = w.step();
            assert!(metrics.charged <= metrics.allowed);
        }
        assert_eq!(w.reset_progress(), None);
        assert_eq!(w.cell(0, 0).unwrap().material, Material::Air);
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
    fn reset_is_incremental_and_charged() {
        let mut w = world(16, 4, 25, 2, 2);
        for i in 0..64 {
            w.set_material(CellId(i), Material::Stone);
        }
        w.reset().unwrap();
        let first = w.step();
        assert_eq!(first.reset_cells, 5);
        assert_eq!(first.charged, 25);
        assert!(first.reset_in_progress);
        assert_eq!(w.cell(0, 0).unwrap().material, Material::Air);
        assert_eq!(w.cell(5, 0).unwrap().material, Material::Stone);
        while w.reset_in_progress() {
            assert!(w.step().charged <= 25);
        }
        assert!(w.cells.iter().all(|cell| cell.material == Material::Air));
    }
    #[test]
    fn compact_storage_fits_default_world_budget() {
        let w = world(
            4096,
            4096,
            32,
            DEFAULT_READY_CAPACITY,
            DEFAULT_COMMAND_CAPACITY,
        );
        let resources = w.resources();
        assert_eq!(resources.cell_bytes, 2);
        assert_eq!(resources.pending_bytes, 3 * 4096 * 4096);
        assert_eq!(resources.captured_frontier_bytes, 4096 * 4096);
        assert_eq!(resources.total_bytes, 102_276_608);
        assert!(resources.total_bytes < 256 * 1024 * 1024);
    }
    #[test]
    fn execution_visualization_marks_latest_slice_and_then_clears() {
        let mut w = world(8, 8, 128, 32, 16);
        let id = w.cell_id(3, 4).unwrap();
        w.mark_cell_for_evaluation(id).unwrap();
        let metrics = w.step();
        assert_eq!(metrics.evaluations, 1);
        assert!(w.cell_executed_last(id));
        assert_eq!(w.visualized_cell_count(), 1);
        assert_eq!(w.executed_quantum_count(), 1);
        w.step();
        assert!(!w.cell_executed_last(id));
        assert_eq!(w.visualized_cell_count(), 0);
    }

    #[test]
    fn water_gravity_and_spreading_conserve_cells() {
        let mut w = world(7, 5, 64, 32, 4);
        let water = w.cell_id(3, 0).unwrap();
        w.set_material(water, Material::Water);
        w.mark_cell_for_evaluation(water).unwrap();
        for _ in 0..80 {
            w.step();
        }
        assert_eq!(
            w.cells
                .iter()
                .filter(|c| c.material == Material::Water)
                .count(),
            1
        );
        assert!((0..7).any(|x| w.cell(x, 4).unwrap().material == Material::Water));
    }
    #[test]
    fn fire_burns_wood_and_ignites_explosives_and_blast_ignites_wood() {
        let mut w = world(8, 3, 64, 32, 4);
        let wood = w.cell_id(1, 1).unwrap();
        let wood2 = w.cell_id(2, 1).unwrap();
        let explosive = w.cell_id(3, 1).unwrap();
        w.set_material(wood, Material::Wood);
        w.set_material(wood2, Material::Wood);
        w.set_material(explosive, Material::Explosive);
        w.ignite(wood).unwrap();
        assert!(w.cell(1, 1).unwrap().burning > 0);
        for _ in 0..100 {
            w.step();
        }
        assert_eq!(w.cell(1, 1).unwrap().material, Material::Air);
        assert_eq!(w.cell(2, 1).unwrap().material, Material::Air);
        assert_eq!(w.cell(3, 1).unwrap().material, Material::Air);

        let mut blast_world = world(4, 3, 64, 16, 4);
        let blastwood = blast_world.cell_id(2, 1).unwrap();
        let blaster = blast_world.cell_id(1, 1).unwrap();
        blast_world.set_material(blastwood, Material::Wood);
        blast_world.trigger_blast(blaster, 2).unwrap();
        blast_world.step();
        assert!(blast_world.cell(2, 1).unwrap().burning > 0);
    }
    #[test]
    fn traditional_policy_drains_the_captured_frontier_without_credit_cap() {
        use crate::fixtures::{FixtureId, ScenarioDescriptor};
        let mut w = World::new(
            16,
            16,
            Credits::new(25),
            Capacity::new(64),
            Capacity::new(16),
        )
        .unwrap();
        w.start_fixture_with_policy(
            ScenarioDescriptor::get(FixtureId::QuietWorld),
            SchedulerPolicy::Traditional,
        )
        .unwrap();
        while w
            .fixture_progress()
            .is_some_and(|progress| !progress.complete)
        {
            w.step();
        }
        assert_eq!(w.policy(), SchedulerPolicy::Traditional);
        for i in 0..3 {
            assert_eq!(
                w.submit(Command::Paint {
                    cell: CellId::from_index(20 + i),
                    material: Material::Sand
                }),
                SubmitResult::Accepted
            );
        }
        let first = w.step();
        assert_eq!(first.commands, 3);
        assert_eq!(first.recoveries, 2 * 16 * 16);
        assert!(first.charged >= 16 * 16 * 10);
        assert_eq!(
            first.evaluations, 0,
            "paint wakes join the next traditional frontier"
        );
        assert!(
            first.charged > first.allowed,
            "traditional mode has no slice credit cap"
        );
        let pending_at_entry = w
            .pending
            .iter()
            .filter(|pending| pending.has(PendingCell::EVAL_PENDING))
            .count();
        let second = w.step();
        assert_eq!(second.evaluations as usize, pending_at_entry);
        assert!(
            w.ready_len() > 0,
            "work created by evaluations remains for the next update"
        );
    }
    #[test]
    fn traditional_captures_pending_work_beyond_ready_rings() {
        use crate::fixtures::{FixtureId, ScenarioDescriptor};
        let mut w = World::new(
            32,
            32,
            Credits::new(25),
            Capacity::new(2),
            Capacity::new(16),
        )
        .unwrap();
        w.start_fixture_with_policy(
            ScenarioDescriptor::get(FixtureId::QuietWorld),
            SchedulerPolicy::Traditional,
        )
        .unwrap();
        while w
            .fixture_progress()
            .is_some_and(|progress| !progress.complete)
        {
            w.step();
        }
        for index in 0..10 {
            w.mark_cell_for_evaluation(CellId::from_index(index * 3))
                .unwrap();
        }
        assert_eq!(w.eval_ready.len(), 2);
        assert_eq!(w.pending_count, 10);
        let metrics = w.step();
        assert_eq!(metrics.evaluations, 10);
        assert_eq!(metrics.recoveries, 2 * 32 * 32);
        assert!(metrics.charged >= 32 * 32 * 10);
        assert_eq!(metrics.pending_cells, 0);
        assert!(metrics.charged > metrics.allowed);
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
