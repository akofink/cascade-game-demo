//! Versioned headless scenario descriptors and scheduler-driven fixture generation.

use crate::{Capacity, CellId, Command, Credits, Material, SimError, SubmitResult, World};

pub const FIXTURE_CELL_COST: u32 = 12;
pub const MAX_DISTURBANCES_PER_SLICE: u32 = 64;
pub const DEFAULT_DISTURBANCE_LIMIT: u64 = 65_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FixtureId {
    QuietWorld,
    ExplosiveLattice,
    SandRelease,
    ReservoirBreach,
    BurningForest,
    DirtyWorldSweep,
    TinyCapacity,
    MixedOverload,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ScenarioDescriptor {
    pub id: FixtureId,
    pub version: u32,
    pub seed: u64,
}
impl ScenarioDescriptor {
    pub const fn get(id: FixtureId) -> Self {
        let (version, seed) = match id {
            FixtureId::QuietWorld => (1, 0x4341_5301),
            FixtureId::ExplosiveLattice => (1, 0x4341_5302),
            FixtureId::SandRelease => (1, 0x4341_5303),
            FixtureId::ReservoirBreach => (1, 0x4341_5304),
            FixtureId::BurningForest => (1, 0x4341_5305),
            FixtureId::DirtyWorldSweep => (1, 0x4341_5306),
            FixtureId::TinyCapacity => (1, 0x4341_5307),
            FixtureId::MixedOverload => (1, 0x4341_5308),
        };
        Self { id, version, seed }
    }
    pub const fn source(self) -> &'static str {
        match self.id {
            FixtureId::QuietWorld => include_str!("../../../scenarios/quiet-world-v1.toml"),
            FixtureId::ExplosiveLattice => {
                include_str!("../../../scenarios/explosive-lattice-v1.toml")
            }
            FixtureId::SandRelease => include_str!("../../../scenarios/sand-release-v1.toml"),
            FixtureId::ReservoirBreach => {
                include_str!("../../../scenarios/reservoir-breach-v1.toml")
            }
            FixtureId::BurningForest => include_str!("../../../scenarios/burning-forest-v1.toml"),
            FixtureId::DirtyWorldSweep => {
                include_str!("../../../scenarios/dirty-world-sweep-v1.toml")
            }
            FixtureId::TinyCapacity => include_str!("../../../scenarios/tiny-capacity-v1.toml"),
            FixtureId::MixedOverload => include_str!("../../../scenarios/mixed-overload-v1.toml"),
        }
    }
    pub const fn capacities(self) -> (Capacity, Capacity) {
        match self.id {
            FixtureId::TinyCapacity => (Capacity::new(2), Capacity::new(2)),
            _ => (
                Capacity::new(crate::DEFAULT_READY_CAPACITY),
                Capacity::new(crate::DEFAULT_COMMAND_CAPACITY),
            ),
        }
    }
    pub const fn name(self) -> &'static str {
        match self.id {
            FixtureId::QuietWorld => "quiet-world",
            FixtureId::ExplosiveLattice => "explosive-lattice",
            FixtureId::SandRelease => "sand-release",
            FixtureId::ReservoirBreach => "reservoir-breach",
            FixtureId::BurningForest => "burning-forest",
            FixtureId::DirtyWorldSweep => "dirty-world-sweep",
            FixtureId::TinyCapacity => "tiny-capacity",
            FixtureId::MixedOverload => "mixed-overload",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixtureError {
    AlreadyPreparing,
    ResetInProgress,
    CapacityMismatch,
    StaleGeneration,
    InvalidDescriptor,
}
impl From<SimError> for FixtureError {
    fn from(value: SimError) -> Self {
        match value {
            SimError::ResetInProgress => Self::ResetInProgress,
            _ => Self::InvalidDescriptor,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct FixtureProgress {
    pub descriptor: Option<ScenarioDescriptor>,
    pub prepared_cells: usize,
    pub total_cells: usize,
    pub complete: bool,
    pub cancelled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) struct PreparationCursor {
    descriptor: ScenarioDescriptor,
    width: u32,
    height: u32,
    generation: u32,
    cursor: usize,
    total: usize,
    finalized: bool,
}
impl PreparationCursor {
    pub(crate) fn new(
        descriptor: ScenarioDescriptor,
        width: u32,
        height: u32,
        generation: u32,
    ) -> Self {
        Self {
            descriptor,
            width,
            height,
            generation,
            cursor: 0,
            total: width as usize * height as usize,
            finalized: false,
        }
    }
    pub(crate) fn progress(self) -> FixtureProgress {
        FixtureProgress {
            descriptor: Some(self.descriptor),
            prepared_cells: self.cursor,
            total_cells: self.total,
            complete: self.finalized,
            cancelled: false,
        }
    }
    pub(crate) fn advance_one(&mut self, world: &mut World) -> Result<(), FixtureError> {
        if world.generation().get() != self.generation {
            return Err(FixtureError::StaleGeneration);
        }
        if self.cursor < self.total {
            let index = self.cursor;
            let x = (index % self.width as usize) as u32;
            let y = (index / self.width as usize) as u32;
            let (material, burning, blast, active) =
                cell_for(self.descriptor, x, y, self.width, self.height, index);
            world.prepare_fixture_cell(
                CellId::from_index(index),
                material,
                burning,
                blast,
                active,
            )?;
            self.cursor += 1;
        } else if !self.finalized {
            self.finalized = true;
        }
        Ok(())
    }
    pub(crate) fn finished(self) -> bool {
        self.finalized
    }
}

fn cell_for(
    d: ScenarioDescriptor,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    index: usize,
) -> (Material, u8, u8, bool) {
    let hash = mix64(d.seed ^ index as u64);
    match d.id {
        FixtureId::QuietWorld => (Material::Air, 0, 0, false),
        FixtureId::ExplosiveLattice => {
            let source = x == width / 2 && y == height / 2;
            let in_lattice = x.is_multiple_of(4) && y.is_multiple_of(4) && !hash.is_multiple_of(8);
            (
                if source || in_lattice {
                    Material::Explosive
                } else {
                    Material::Air
                },
                0,
                if source { 15 } else { 0 },
                false,
            )
        }
        FixtureId::SandRelease => {
            let sand_limit =
                10_000_000usize.min(width as usize * height.saturating_sub(4) as usize);
            let sand_rows = sand_limit / width as usize;
            let sand_extra = sand_limit % width as usize;
            let sand =
                (y as usize) < sand_rows || (y as usize == sand_rows && (x as usize) < sand_extra);
            let support = y >= height.saturating_sub(4);
            (
                if sand {
                    Material::Sand
                } else if support {
                    Material::Stone
                } else {
                    Material::Air
                },
                0,
                0,
                sand,
            )
        }
        FixtureId::ReservoirBreach => {
            let left = width / 4;
            let right = width / 4 * 3 + width % 4 * 3 / 4;
            let top = height / 4;
            let bottom = height / 4 * 3 + height % 4 * 3 / 4;
            let perimeter = x >= left
                && x <= right
                && y >= top
                && y <= bottom
                && (x == left || x == right || y == top || y == bottom);
            let breach = x == width / 2 && y == bottom;
            let inside = x > left && x < right && y > top && y < bottom;
            let water = inside && y > top + (bottom - top) / 2;
            (
                if perimeter && !breach {
                    Material::Stone
                } else if water {
                    Material::Water
                } else {
                    Material::Air
                },
                0,
                0,
                water,
            )
        }
        FixtureId::BurningForest => {
            let wood = !hash.is_multiple_of(5);
            let burning = wood && hash.is_multiple_of(257);
            (
                if wood { Material::Wood } else { Material::Air },
                if burning { 12 } else { 0 },
                0,
                burning,
            )
        }
        FixtureId::DirtyWorldSweep => (
            if hash & 1 == 0 {
                Material::Stone
            } else {
                Material::Air
            },
            0,
            0,
            false,
        ),
        FixtureId::TinyCapacity => {
            let material = match (x + y) % 4 {
                0 => Material::Sand,
                1 => Material::Water,
                2 => Material::Wood,
                _ => Material::Air,
            };
            (
                material,
                if material == Material::Wood && hash.is_multiple_of(31) {
                    12
                } else {
                    0
                },
                0,
                matches!(material, Material::Sand | Material::Water),
            )
        }
        FixtureId::MixedOverload => {
            let material = match (x >= width / 2, y >= height / 2) {
                (false, false) => Material::Sand,
                (true, false) => Material::Water,
                (false, true) => {
                    if hash.is_multiple_of(5) {
                        Material::Air
                    } else {
                        Material::Wood
                    }
                }
                (true, true) => {
                    if x.is_multiple_of(4) && y.is_multiple_of(4) {
                        Material::Explosive
                    } else {
                        Material::Stone
                    }
                }
            };
            let burning = material == Material::Wood && hash.is_multiple_of(193);
            let center_source =
                x == width / 4 * 3 + width % 4 * 3 / 4 && y == height / 4 * 3 + height % 4 * 3 / 4;
            (
                if center_source {
                    Material::Explosive
                } else {
                    material
                },
                if burning { 12 } else { 0 },
                if center_source { 15 } else { 0 },
                matches!(material, Material::Sand | Material::Water) || burning,
            )
        }
    }
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DisturbanceMetrics {
    pub attempted: u32,
    pub accepted: u32,
    pub coalesced: u32,
    pub rejected: u32,
}

/// Fixed-seed disturbance generator. It emits at most the configured (capped) number per call.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct DisturbanceCommandStream {
    seed: u64,
    emitted: u64,
    limit: u64,
    per_slice: u32,
}
impl DisturbanceCommandStream {
    pub const fn new(seed: u64, limit: u64, per_slice: u32) -> Self {
        Self {
            seed,
            emitted: 0,
            limit,
            per_slice: if per_slice < MAX_DISTURBANCES_PER_SLICE {
                per_slice
            } else {
                MAX_DISTURBANCES_PER_SLICE
            },
        }
    }
    pub const fn default_for(seed: u64) -> Self {
        Self::new(seed, DEFAULT_DISTURBANCE_LIMIT, 8)
    }
    pub const fn emitted(self) -> u64 {
        self.emitted
    }
    pub const fn finished(self) -> bool {
        self.emitted >= self.limit
    }
    pub fn admit_slice(&mut self, world: &mut World) -> DisturbanceMetrics {
        let mut metrics = DisturbanceMetrics::default();
        for _ in 0..self.per_slice {
            if self.finished() {
                break;
            }
            let sequence = self.emitted;
            let random = mix64(self.seed ^ sequence);
            let index = (random % world.cell_count() as u64) as usize;
            let material = match sequence % 4 {
                0 => Material::Sand,
                1 => Material::Water,
                2 => Material::Wood,
                _ => Material::Explosive,
            };
            let command = Command::Paint {
                cell: CellId::from_index(index),
                material,
            };
            metrics.attempted += 1;
            match world.submit(command) {
                SubmitResult::Accepted => metrics.accepted += 1,
                SubmitResult::Coalesced => metrics.coalesced += 1,
                SubmitResult::RejectedFull
                | SubmitResult::ResetInProgress
                | SubmitResult::FixtureInProgress => metrics.rejected += 1,
            }
            self.emitted += 1;
        }
        metrics
    }
}

// Keep a compile-time link to documented fixture capacities without introducing dynamic storage.
const _: usize = Capacity::new(256).get();
const _: Credits = Credits::new(FIXTURE_CELL_COST);

#[cfg(test)]
mod tests {
    use super::*;

    fn world(width: u32, height: u32) -> World {
        World::new(
            width,
            height,
            Credits::new(64),
            Capacity::new(4),
            Capacity::new(16),
        )
        .unwrap()
    }

    #[test]
    fn every_charter_scenario_has_versioned_seeded_descriptor() {
        let ids = [
            FixtureId::QuietWorld,
            FixtureId::ExplosiveLattice,
            FixtureId::SandRelease,
            FixtureId::ReservoirBreach,
            FixtureId::BurningForest,
            FixtureId::DirtyWorldSweep,
            FixtureId::TinyCapacity,
            FixtureId::MixedOverload,
        ];
        for id in ids {
            let descriptor = ScenarioDescriptor::get(id);
            let source = descriptor.source();
            assert!(source.contains(&format!("id = \"{}\"", descriptor.name())));
            assert!(source.contains(&format!("version = {}", descriptor.version)));
            assert!(source.contains(&format!("seed = {}", descriptor.seed)));
        }
        let tiny = ScenarioDescriptor::get(FixtureId::TinyCapacity);
        assert_eq!(tiny.capacities(), (Capacity::new(2), Capacity::new(2)));
        assert_eq!(
            world(8, 8).start_fixture(tiny),
            Err(FixtureError::CapacityMismatch)
        );
        let (ready, commands) = tiny.capacities();
        let mut constrained = World::new(8, 8, Credits::new(64), ready, commands).unwrap();
        constrained.start_fixture(tiny).unwrap();
    }

    #[test]
    fn all_charter_fixtures_prepare_within_bounded_slices() {
        let ids = [
            FixtureId::QuietWorld,
            FixtureId::ExplosiveLattice,
            FixtureId::SandRelease,
            FixtureId::ReservoirBreach,
            FixtureId::BurningForest,
            FixtureId::DirtyWorldSweep,
            FixtureId::TinyCapacity,
            FixtureId::MixedOverload,
        ];
        for id in ids {
            let descriptor = ScenarioDescriptor::get(id);
            let (ready, commands) = descriptor.capacities();
            let mut world = World::new(8, 8, Credits::new(64), ready, commands).unwrap();
            world.start_fixture(descriptor).unwrap();
            for _ in 0..100 {
                let metrics = world.step();
                assert!(metrics.charged <= metrics.allowed);
                if world
                    .fixture_progress()
                    .is_some_and(|progress| progress.complete)
                {
                    break;
                }
            }
            assert!(
                world
                    .fixture_progress()
                    .is_some_and(|progress| progress.complete),
                "{}",
                descriptor.name()
            );
        }
    }

    #[test]
    fn fixture_preparation_is_charged_incremental_and_cancellable() {
        let descriptor = ScenarioDescriptor::get(FixtureId::DirtyWorldSweep);
        let mut world = world(64, 64);
        world.start_fixture(descriptor).unwrap();
        assert_eq!(world.fixture_progress().unwrap().prepared_cells, 0);
        assert_eq!(
            world.submit(Command::Paint {
                cell: CellId::from_index(0),
                material: Material::Stone
            }),
            SubmitResult::ResetInProgress
        );
        let mut completed = false;
        for _ in 0..2000 {
            let metrics = world.step();
            assert!(metrics.charged <= metrics.allowed);
            if let Some(progress) = world.fixture_progress()
                && progress.complete
            {
                completed = true;
                break;
            }
        }
        assert!(completed);
        assert!(!world.fixture_progress().unwrap().cancelled);
        assert!(
            world
                .cells
                .iter()
                .any(|cell| cell.material == Material::Stone)
        );

        world
            .start_fixture(ScenarioDescriptor::get(FixtureId::MixedOverload))
            .unwrap();
        while world.reset_in_progress() {
            world.step();
        }
        world.step();
        let cancelled = world.cancel_fixture().unwrap();
        assert!(cancelled.cancelled);
        assert!(world.fixture_progress().unwrap().cancelled);
    }

    #[test]
    fn descriptor_preparation_replays_identically_and_marks_dirt() {
        fn run() -> World {
            let mut world = world(16, 12);
            world
                .start_fixture(ScenarioDescriptor::get(FixtureId::ExplosiveLattice))
                .unwrap();
            while world
                .fixture_progress()
                .is_some_and(|progress| !progress.complete)
            {
                world.step();
            }
            world
        }
        let a = run();
        let b = run();
        assert_eq!(a.state_hash(), b.state_hash());
        assert!(a.cell(8, 6).unwrap().state > 0);
        assert!(a.chunk_dirty(0, 0));
    }

    #[test]
    fn disturbance_stream_is_deterministic_and_capped() {
        let mut a = world(16, 16);
        let mut b = world(16, 16);
        let mut stream_a = DisturbanceCommandStream::new(99, 130, 500);
        let mut stream_b = DisturbanceCommandStream::new(99, 130, 500);
        let batch = stream_a.admit_slice(&mut a);
        assert_eq!(batch.attempted, MAX_DISTURBANCES_PER_SLICE);
        assert_eq!(stream_a.emitted(), MAX_DISTURBANCES_PER_SLICE as u64);
        for _ in 0..3 {
            stream_a.admit_slice(&mut a);
            stream_b.admit_slice(&mut b);
        }
        assert_eq!(stream_a, stream_b);
        for _ in 0..20 {
            a.step();
            b.step();
        }
        assert_eq!(a.state_hash(), b.state_hash());
    }
}
