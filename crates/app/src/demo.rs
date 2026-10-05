//! App-side controls and bounded simulation stepping.

use cascade_sim::{
    Capacity, Command, Credits, Material, SliceMetrics, SubmitResult, World,
    fixtures::{DisturbanceCommandStream, FixtureId, ScenarioDescriptor},
};

pub const DEMO_WIDTH: u32 = 1024;
pub const DEMO_HEIGHT: u32 = 1024;
pub const DEFAULT_CREDITS: u32 = 20_000;
pub const MIN_CREDITS: u32 = 25;
pub const MAX_CREDITS: u32 = 100_000;
pub const BRUSH_CELLS_PER_FRAME: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialChoice {
    Air,
    Stone,
    Wood,
    Sand,
    Explosive,
    Water,
}

impl MaterialChoice {
    pub const ALL: [Self; 6] = [
        Self::Air,
        Self::Stone,
        Self::Wood,
        Self::Sand,
        Self::Explosive,
        Self::Water,
    ];
    pub const fn material(self) -> Material {
        match self {
            Self::Air => Material::Air,
            Self::Stone => Material::Stone,
            Self::Wood => Material::Wood,
            Self::Sand => Material::Sand,
            Self::Explosive => Material::Explosive,
            Self::Water => Material::Water,
        }
    }
    pub const fn name(self) -> &'static str {
        match self {
            Self::Air => "Air / erase",
            Self::Stone => "Stone",
            Self::Wood => "Wood",
            Self::Sand => "Sand",
            Self::Explosive => "Explosive",
            Self::Water => "Water",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DemoMetrics {
    pub slice: SliceMetrics,
    pub paused: bool,
    pub policy: &'static str,
    pub selected_fixture: FixtureId,
    pub fixture_progress: Option<cascade_sim::fixtures::FixtureProgress>,
    pub reset_in_progress: bool,
    pub destroy_held: bool,
    pub disturbance_emitted: u64,
    pub resource_bytes: usize,
}

pub struct Demo {
    world: World,
    alternate_world: World,
    tiny_capacity_active: bool,
    paused: bool,
    selected_fixture: FixtureId,
    selected_material: MaterialChoice,
    credits: u32,
    step_once: bool,
    destroy_held: bool,
    disturbances: DisturbanceCommandStream,
    last_metrics: SliceMetrics,
    last_disturbance: u64,
}

impl Demo {
    pub fn new() -> Result<Self, String> {
        let world = World::new(
            DEMO_WIDTH,
            DEMO_HEIGHT,
            Credits::new(DEFAULT_CREDITS),
            Capacity::new(cascade_sim::DEFAULT_READY_CAPACITY),
            Capacity::new(cascade_sim::DEFAULT_COMMAND_CAPACITY),
        )
        .map_err(|error| format!("create simulation: {error:?}"))?;
        let (tiny_ready, tiny_commands) =
            ScenarioDescriptor::get(FixtureId::TinyCapacity).capacities();
        let alternate_world = World::new(
            DEMO_WIDTH,
            DEMO_HEIGHT,
            Credits::new(DEFAULT_CREDITS),
            tiny_ready,
            tiny_commands,
        )
        .map_err(|error| format!("create tiny-capacity simulation: {error:?}"))?;
        Ok(Self {
            world,
            alternate_world,
            tiny_capacity_active: false,
            paused: false,
            selected_fixture: FixtureId::MixedOverload,
            selected_material: MaterialChoice::Sand,
            credits: DEFAULT_CREDITS,
            step_once: false,
            destroy_held: false,
            disturbances: DisturbanceCommandStream::default_for(0x4341_5345_0001),
            last_metrics: SliceMetrics::default(),
            last_disturbance: 0,
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.world.dimensions()
    }
    pub fn world(&self) -> &World {
        &self.world
    }
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
    pub fn selected_material(&self) -> MaterialChoice {
        self.selected_material
    }
    pub fn set_selected_material(&mut self, material: MaterialChoice) {
        self.selected_material = material;
    }
    pub fn credits(&self) -> u32 {
        self.credits
    }

    pub fn set_credits(&mut self, credits: u32) -> bool {
        if !(MIN_CREDITS..=MAX_CREDITS).contains(&credits)
            || self.world.set_budget(Credits::new(credits)).is_err()
            || self
                .alternate_world
                .set_budget(Credits::new(credits))
                .is_err()
        {
            return false;
        }
        self.credits = credits;
        true
    }

    pub fn metrics(&self) -> DemoMetrics {
        DemoMetrics {
            slice: self.last_metrics,
            paused: self.paused,
            policy: "bounded",
            selected_fixture: self.selected_fixture,
            fixture_progress: self.world.fixture_progress(),
            reset_in_progress: self.world.reset_in_progress(),
            destroy_held: self.destroy_held,
            disturbance_emitted: self.last_disturbance,
            resource_bytes: self.world.resources().total_bytes
                + self.alternate_world.resources().total_bytes,
        }
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }
    pub fn toggle_paused(&mut self) {
        self.paused = !self.paused;
    }
    pub fn single_step(&mut self) {
        self.step_once = true;
    }
    pub fn set_destroy_held(&mut self, held: bool) {
        self.destroy_held = held;
    }
    pub fn select_fixture(&mut self, fixture: FixtureId) {
        self.selected_fixture = fixture;
    }

    pub fn start_fixture(&mut self) -> Result<(), String> {
        let descriptor = ScenarioDescriptor::get(self.selected_fixture);
        if self.world.reset_in_progress() {
            return Err("reset is still in progress".to_string());
        }
        if self
            .world
            .fixture_progress()
            .is_some_and(|progress| !progress.complete && !progress.cancelled)
        {
            return Err(
                "cancel the current fixture preparation before starting another".to_string(),
            );
        }
        let wants_tiny = descriptor.id == FixtureId::TinyCapacity;
        if wants_tiny != self.tiny_capacity_active {
            std::mem::swap(&mut self.world, &mut self.alternate_world);
            self.tiny_capacity_active = wants_tiny;
        }
        self.world
            .start_fixture(descriptor)
            .map_err(|error| format!("start fixture: {error:?}"))?;
        self.disturbances = DisturbanceCommandStream::default_for(0x4341_5345_0001);
        self.last_disturbance = 0;
        Ok(())
    }

    pub fn cancel_fixture(&mut self) -> bool {
        self.world.cancel_fixture().is_some()
    }

    pub fn reset(&mut self) -> Result<(), String> {
        self.world
            .reset()
            .map_err(|error| format!("reset: {error:?}"))
    }

    pub fn submit_brush(&mut self, points: &[(u32, u32)], erase: bool) -> usize {
        let material = if erase {
            Material::Air
        } else {
            self.selected_material.material()
        };
        let mut admitted = 0;
        for &(x, y) in points.iter().take(BRUSH_CELLS_PER_FRAME) {
            if let Some(cell) = self.world.cell_id(x, y)
                && matches!(
                    self.world.submit(Command::Paint { cell, material }),
                    SubmitResult::Accepted | SubmitResult::Coalesced
                )
            {
                admitted += 1;
            }
        }
        admitted
    }

    pub fn submit_brush_disk(
        &mut self,
        x: u32,
        y: u32,
        radius: u32,
        erase: bool,
        limit: usize,
    ) -> usize {
        let material = if erase {
            Material::Air
        } else {
            self.selected_material.material()
        };
        let radius = radius.min(4) as i32;
        let mut attempted = 0;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy > radius * radius
                    || attempted == limit.min(BRUSH_CELLS_PER_FRAME)
                {
                    continue;
                }
                attempted += 1;
                let (Some(cx), Some(cy)) = (x.checked_add_signed(dx), y.checked_add_signed(dy))
                else {
                    continue;
                };
                let Some(cell) = self.world.cell_id(cx, cy) else {
                    continue;
                };
                let _ = self.world.submit(Command::Paint { cell, material });
            }
        }
        attempted
    }

    pub fn ignite_at(&mut self, x: u32, y: u32) {
        if let Some(cell) = self.world.cell_id(x, y) {
            let _ = self.world.ignite(cell);
        }
    }
    pub fn detonate_at(&mut self, x: u32, y: u32) {
        if let Some(cell) = self.world.cell_id(x, y) {
            let _ = self.world.trigger_blast(cell, 12);
        }
    }

    /// Advance no more than one scheduler slice per presentation iteration.
    pub fn tick(&mut self) {
        let preparing = self.world.reset_in_progress()
            || self
                .world
                .fixture_progress()
                .is_some_and(|progress| !progress.complete && !progress.cancelled);
        if !self.paused || self.step_once || preparing {
            if self.destroy_held && !preparing && !self.disturbances.finished() {
                let batch = self.disturbances.admit_slice(&mut self.world);
                self.last_disturbance = self
                    .last_disturbance
                    .saturating_add(u64::from(batch.attempted));
            }
            self.last_metrics = self.world.step();
            self.step_once = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credit_adjustment_obeys_app_and_scheduler_bounds() {
        let mut demo = Demo::new().unwrap();
        assert!(!demo.set_credits(MIN_CREDITS - 1));
        assert!(!demo.set_credits(MAX_CREDITS + 1));
        assert!(demo.set_credits(MIN_CREDITS));
        demo.tick();
        assert!(demo.metrics().slice.charged <= MIN_CREDITS);
    }

    #[test]
    fn brush_admission_is_capped_and_sim_work_advances_by_slice() {
        let mut demo = Demo::new().unwrap();
        let points = vec![(100, 100); BRUSH_CELLS_PER_FRAME + 32];
        assert!(demo.submit_brush(&points, false) <= BRUSH_CELLS_PER_FRAME);
        demo.tick();
        assert!(demo.metrics().slice.charged <= demo.credits());
        assert!(demo.metrics().slice.commands <= 1);
    }

    #[test]
    fn fixture_capacity_profiles_are_applied_and_restored() {
        let mut demo = Demo::new().unwrap();
        demo.select_fixture(FixtureId::TinyCapacity);
        demo.start_fixture().unwrap();
        assert_eq!(demo.world().resources().ready_capacity.get(), 4);
        demo.cancel_fixture();
        while demo.world().reset_in_progress() {
            demo.world_mut().step();
        }
        demo.select_fixture(FixtureId::QuietWorld);
        demo.start_fixture().unwrap();
        assert_eq!(
            demo.world().resources().ready_capacity.get(),
            2 * cascade_sim::DEFAULT_READY_CAPACITY
        );
    }

    #[test]
    fn fixture_preparation_runs_even_when_paused() {
        let mut demo = Demo::new().unwrap();
        demo.start_fixture().unwrap();
        demo.set_paused(true);
        for _ in 0..300 {
            demo.tick();
            if demo
                .metrics()
                .fixture_progress
                .is_some_and(|p| p.prepared_cells > 0)
            {
                break;
            }
        }
        let progress = demo.metrics().fixture_progress.unwrap();
        assert!(progress.prepared_cells > 0);
        assert!(demo.metrics().paused);
    }
}
