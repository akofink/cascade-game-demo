//! App-side controls and bounded simulation stepping.

use std::sync::OnceLock;

use cascade_sim::{
    Capacity, Command, Credits, Material, SchedulerPolicy, SliceMetrics, SubmitResult, World,
    fixtures::{
        DEFAULT_DISTURBANCE_LIMIT, DisturbanceCommandStream, FixtureId, MAX_DISTURBANCES_PER_SLICE,
        ScenarioDescriptor,
    },
};

pub const DEMO_WIDTH: u32 = 1024;
pub const DEMO_HEIGHT: u32 = 1024;
const CREDIT_PROFILE: &str = include_str!("../../../profiles/m2-16gb-v3.toml");
pub const MIN_CREDITS: u32 = 25;

#[derive(Clone, Copy)]
struct AppCreditProfile {
    default: u32,
    maximum: u32,
}

fn parse_profile_credits(contents: &str) -> AppCreditProfile {
    let profile: toml::Value =
        toml::from_str(contents).expect("embedded credit profile is valid TOML");
    let credits = |key: &str| {
        profile
            .get(key)
            .and_then(toml::Value::as_integer)
            .and_then(|credits| u32::try_from(credits).ok())
            .expect("embedded profile has a positive app credit value")
    };
    let default = credits("app_current_default_credits");
    let maximum = credits("app_current_max_credits");
    assert!(
        default >= MIN_CREDITS,
        "profile allowance is below app minimum"
    );
    assert!(default <= maximum, "profile default exceeds app maximum");
    assert!(
        maximum <= cascade_sim::MAX_BUDGET_CREDITS,
        "profile allowance exceeds simulator limit"
    );
    AppCreditProfile { default, maximum }
}

fn profile_credits() -> AppCreditProfile {
    static CREDITS: OnceLock<AppCreditProfile> = OnceLock::new();
    *CREDITS.get_or_init(|| parse_profile_credits(CREDIT_PROFILE))
}

pub fn default_credits() -> u32 {
    profile_credits().default
}

pub fn max_credits() -> u32 {
    profile_credits().maximum
}
pub const BRUSH_CELLS_PER_FRAME: usize = 64;

fn disturbance_stream() -> DisturbanceCommandStream {
    DisturbanceCommandStream::new(
        0x4341_5345_0001,
        DEFAULT_DISTURBANCE_LIMIT,
        MAX_DISTURBANCES_PER_SLICE,
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyChoice {
    Bounded,
    Traditional,
}
impl PolicyChoice {
    const fn scheduler(self) -> SchedulerPolicy {
        match self {
            Self::Bounded => SchedulerPolicy::Bounded,
            Self::Traditional => SchedulerPolicy::Traditional,
        }
    }
    const fn name(self) -> &'static str {
        self.scheduler().name()
    }
}

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
    pub ready_capacity: usize,
}

pub struct Demo {
    world: World,
    alternate_world: World,
    tiny_capacity_active: bool,
    policy: PolicyChoice,
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
        Self::new_with_size(DEMO_WIDTH, DEMO_HEIGHT)
    }

    pub fn new_with_size(width: u32, height: u32) -> Result<Self, String> {
        if !(32..=crate::MAX_WORLD_AXIS).contains(&width)
            || !(32..=crate::MAX_WORLD_AXIS).contains(&height)
            || !width.is_multiple_of(32)
            || !height.is_multiple_of(32)
        {
            return Err("world dimensions must be multiples of 32 in 32..=4096".to_string());
        }
        let initial_credits = default_credits();
        let world = World::new(
            width,
            height,
            Credits::new(initial_credits),
            Capacity::new(cascade_sim::DEFAULT_READY_CAPACITY),
            Capacity::new(cascade_sim::DEFAULT_COMMAND_CAPACITY),
        )
        .map_err(|error| format!("create simulation: {error:?}"))?;
        let (tiny_ready, tiny_commands) =
            ScenarioDescriptor::get(FixtureId::TinyCapacity).capacities();
        let alternate_world = World::new(
            width,
            height,
            Credits::new(initial_credits),
            tiny_ready,
            tiny_commands,
        )
        .map_err(|error| format!("create tiny-capacity simulation: {error:?}"))?;
        Ok(Self {
            world,
            alternate_world,
            tiny_capacity_active: false,
            policy: PolicyChoice::Bounded,
            paused: false,
            selected_fixture: FixtureId::MixedOverload,
            selected_material: MaterialChoice::Sand,
            credits: initial_credits,
            step_once: false,
            destroy_held: false,
            disturbances: disturbance_stream(),
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
        if !(MIN_CREDITS..=max_credits()).contains(&credits)
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
            policy: self.policy.name(),
            selected_fixture: self.selected_fixture,
            fixture_progress: self.world.fixture_progress(),
            reset_in_progress: self.world.reset_in_progress(),
            destroy_held: self.destroy_held,
            disturbance_emitted: self.last_disturbance,
            resource_bytes: self.world.resources().total_bytes
                + self.alternate_world.resources().total_bytes,
            ready_capacity: self.world.resources().ready_capacity.get(),
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
    pub fn set_policy(&mut self, policy: PolicyChoice) {
        self.policy = policy;
    }
    pub fn policy(&self) -> PolicyChoice {
        self.policy
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
            .start_fixture_with_policy(descriptor, self.policy.scheduler())
            .map_err(|error| format!("start fixture: {error:?}"))?;
        self.disturbances = disturbance_stream();
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
    fn startup_world_size_is_validated_before_allocation() {
        assert!(Demo::new_with_size(0, 1024).is_err());
        assert!(Demo::new_with_size(33, 1024).is_err());
        assert!(Demo::new_with_size(4097, 4096).is_err());
        let demo = Demo::new_with_size(64, 96).unwrap();
        assert_eq!(demo.dimensions(), (64, 96));
        let full_size = Demo::new_with_size(4096, 4096).unwrap();
        assert_eq!(full_size.dimensions(), (4096, 4096));
        assert!(full_size.metrics().resource_bytes < 256 * 1024 * 1024);
    }

    #[test]
    fn profile_supplies_the_default_and_maximum_credit_allowance() {
        let test_profile = parse_profile_credits(
            "app_current_default_credits = 1234\napp_current_max_credits = 2345\n",
        );
        assert_eq!(test_profile.default, 1234);
        assert_eq!(test_profile.maximum, 2345);
        let allowance = profile_credits();
        assert_eq!(max_credits(), allowance.maximum);
        let demo = Demo::new().unwrap();
        assert_eq!(demo.credits(), allowance.default);
    }

    #[test]
    fn credit_adjustment_obeys_app_and_scheduler_bounds() {
        let mut demo = Demo::new().unwrap();
        assert!(!demo.set_credits(MIN_CREDITS - 1));
        assert!(!demo.set_credits(max_credits() + 1));
        assert!(demo.set_credits(MIN_CREDITS));
        assert!(demo.set_credits(max_credits()));
        demo.tick();
        assert!(demo.metrics().slice.charged <= max_credits());
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
    fn policy_switch_restarts_the_same_fixture_with_the_selected_scheduler() {
        let mut demo = Demo::new().unwrap();
        let fixture = demo.metrics().selected_fixture;
        demo.set_policy(PolicyChoice::Traditional);
        demo.start_fixture().unwrap();
        assert_eq!(demo.world().policy(), SchedulerPolicy::Traditional);
        assert_eq!(demo.metrics().selected_fixture, fixture);
        demo.cancel_fixture();
        while demo.world().reset_in_progress() {
            demo.world_mut().step();
        }
        demo.set_policy(PolicyChoice::Bounded);
        demo.start_fixture().unwrap();
        assert_eq!(demo.world().policy(), SchedulerPolicy::Bounded);
        assert_eq!(demo.metrics().selected_fixture, fixture);
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
