use cascade_sim::{
    CHUNK_SIDE, Capacity, Command, Credits, Material, SchedulerPolicy, SubmitResult, World,
};
use wasm_bindgen::prelude::*;

const SIDE: u32 = 128;
const READY_CAPACITY: usize = 64;
const COMMAND_CAPACITY: usize = 256;

#[wasm_bindgen]
pub struct TourWorld {
    world: World,
    width: u32,
    height: u32,
    last_allowed: u32,
    last_charged: u32,
    last_evaluations: u32,
    last_blasts: u32,
    last_recoveries: u32,
    last_commands: u32,
    last_age: u64,
    last_slice_nanos: f64,
    scene: u8,
}

#[wasm_bindgen]
impl TourWorld {
    #[wasm_bindgen(constructor)]
    pub fn new(budget: u32, traditional: bool, scene: u8) -> Result<TourWorld, JsValue> {
        let world = build_world(budget, traditional, scene)?;
        Ok(Self {
            world,
            width: SIDE,
            height: SIDE,
            last_allowed: budget.clamp(25, 10_000_000),
            last_charged: 0,
            last_evaluations: 0,
            last_blasts: 0,
            last_recoveries: 0,
            last_commands: 0,
            last_age: 0,
            last_slice_nanos: 0.0,
            scene: scene % 6,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn set_budget(&mut self, budget: u32) {
        let value = budget.clamp(25, 10_000_000);
        if self.world.set_budget(Credits::new(value)).is_ok() {
            self.last_allowed = value;
        }
    }
    pub fn reset(&mut self) -> Result<(), JsValue> {
        self.world = build_world(
            self.last_allowed,
            self.world.policy() == SchedulerPolicy::Traditional,
            self.scene,
        )?;
        self.last_charged = 0;
        self.last_evaluations = 0;
        self.last_blasts = 0;
        self.last_recoveries = 0;
        self.last_commands = 0;
        self.last_age = 0;
        self.last_slice_nanos = 0.0;
        Ok(())
    }
    pub fn paint(&mut self, x: u32, y: u32, material: u8) -> bool {
        let Some(cell) = self.world.cell_id(x, y) else {
            return false;
        };
        let material = match material {
            1 => Material::Stone,
            2 => Material::Wood,
            3 => Material::Sand,
            4 => Material::Explosive,
            5 => Material::Water,
            _ => Material::Air,
        };
        matches!(
            self.world.submit(Command::Paint { cell, material }),
            SubmitResult::Accepted | SubmitResult::Coalesced
        )
    }
    pub fn ignite(&mut self, x: u32, y: u32) -> bool {
        self.world.cell_id(x, y).is_some_and(|cell| {
            matches!(
                self.world.submit(Command::Ignite { cell }),
                SubmitResult::Accepted | SubmitResult::Coalesced
            )
        })
    }
    pub fn seed_scene(&mut self, scene: u8) {
        let w = self.width;
        let floor = self.height - 12;
        if scene % 6 != 3 {
            for x in 4..w - 4 {
                let _ = self.paint(x, floor, 1);
            }
        }
        match scene % 6 {
            0 => {
                for x in 24..104 {
                    let _ = self.paint(x, 36, 3);
                }
                for x in 54..74 {
                    let _ = self.paint(x, 88, 5);
                }
                for x in 42..86 {
                    let _ = self.paint(x, 70, 2);
                }
                let _ = self.paint(64, 70, 4);
                let _ = self.ignite(64, 70);
            }
            1 => {
                for x in 28..100 {
                    let _ = self.paint(x, 76, 2);
                }
                for x in 32..96 {
                    let _ = self.paint(x, 74, 4);
                }
                let _ = self.ignite(64, 74);
            }
            2 => {
                for x in 28..100 {
                    let _ = self.paint(x, 48, 3);
                }
                for x in 40..88 {
                    let _ = self.paint(x, 60, 5);
                }
            }
            3 => {
                for y in 36..44 {
                    for x in 40..48 {
                        let _ = self.paint(x, y, 2);
                    }
                    for x in 52..60 {
                        let _ = self.paint(x, y, 4);
                    }
                }
                for x in 32..96 {
                    let _ = self.paint(x, 78, 4);
                }
                for y in 24..56 {
                    let _ = self.paint(32, y, 2);
                    let _ = self.paint(95, y, 2);
                }
            }
            4 => {
                for x in [24, 48, 72, 96] {
                    for y in (38..74).step_by(12) {
                        let _ = self.paint(x, y, 2);
                    }
                }
                for x in (32..96).step_by(8) {
                    let _ = self.paint(x, 84, 4);
                }
                let _ = self.ignite(32, 84);
            }
            _ => {
                for x in 26..102 {
                    let _ = self.paint(x, 60, 3);
                    let _ = self.paint(x, 62, 5);
                }
                for x in (34..96).step_by(8) {
                    let _ = self.paint(x, 84, 4);
                }
                let _ = self.ignite(64, 84);
            }
        }
    }
    pub fn step(&mut self) {
        let start = browser_now_ms();
        let m = self.world.step();
        self.last_slice_nanos = (browser_now_ms() - start) * 1_000_000.0;
        self.last_allowed = m.allowed;
        self.last_charged = m.charged;
        self.last_evaluations = m.evaluations;
        self.last_blasts = m.blasts;
        self.last_recoveries = m.recoveries;
        self.last_commands = m.commands;
        self.last_age = m.oldest_pending_age;
    }
    pub fn cells(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity((self.width * self.height * 5) as usize);
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.world.cell(x, y).expect("validated grid coordinate");
                let id = self.world.cell_id(x, y).expect("validated grid coordinate");
                out.push(cell.material as u8);
                out.push(cell.state.max(cell.burning));
                out.push(u8::from(self.world.cell_pending(x, y).unwrap_or(false)));
                out.push(u8::from(self.world.cell_executed_last(id)));
                let focused = self.world.active_focus_regions().any(|region| {
                    let chunk_x = x / CHUNK_SIDE;
                    let chunk_y = y / CHUNK_SIDE;
                    (region.min_chunk_x..region.max_chunk_x).contains(&chunk_x)
                        && (region.min_chunk_y..region.max_chunk_y).contains(&chunk_y)
                });
                out.push(u8::from(focused));
            }
        }
        out
    }
    pub fn allowed(&self) -> u32 {
        self.last_allowed
    }
    pub fn charged(&self) -> u32 {
        self.last_charged
    }
    pub fn evaluations(&self) -> u32 {
        self.last_evaluations
    }
    pub fn blasts(&self) -> u32 {
        self.last_blasts
    }
    pub fn executed_cells(&self) -> u32 {
        self.world.executed_quantum_count()
    }
    pub fn executed_cells_sampled(&self) -> u32 {
        self.world.visualized_cell_count() as u32
    }
    pub fn recoveries(&self) -> u32 {
        self.last_recoveries
    }
    pub fn commands(&self) -> u32 {
        self.last_commands
    }
    pub fn queued_commands(&self) -> u32 {
        self.world.command_len().min(u32::MAX as usize) as u32
    }
    pub fn pending(&self) -> u32 {
        self.world.pending_channels().min(u32::MAX as usize) as u32
    }
    pub fn oldest_age(&self) -> u32 {
        self.last_age.min(u32::MAX as u64) as u32
    }
    pub fn ready(&self) -> u32 {
        self.world.ready_len().min(u32::MAX as usize) as u32
    }
    pub fn slice_time_ms(&self) -> f64 {
        self.last_slice_nanos / 1_000_000.0
    }
    pub fn slice(&self) -> u32 {
        self.world.slice_index().min(u32::MAX as u64) as u32
    }
    pub fn reset_done(&self) -> bool {
        !self.world.reset_in_progress()
    }
    pub fn focus_regions(&self) -> u32 {
        self.world
            .active_focus_regions()
            .count()
            .min(u32::MAX as usize) as u32
    }
}

fn build_world(budget: u32, traditional: bool, scene: u8) -> Result<World, JsValue> {
    let setup_budget = 10_000;
    let policy = if traditional {
        SchedulerPolicy::Traditional
    } else {
        SchedulerPolicy::Bounded
    };
    let world = World::new_with_policy(
        SIDE,
        SIDE,
        Credits::new(setup_budget),
        Capacity::new(READY_CAPACITY),
        Capacity::new(COMMAND_CAPACITY),
        policy,
    )
    .map_err(|error| JsValue::from_str(&format!("could not create world: {error:?}")))?;
    let mut tour = TourWorld {
        world,
        width: SIDE,
        height: SIDE,
        last_allowed: setup_budget,
        last_charged: 0,
        last_evaluations: 0,
        last_blasts: 0,
        last_recoveries: 0,
        last_commands: 0,
        last_age: 0,
        last_slice_nanos: 0.0,
        scene: scene % 6,
    };
    tour.seed_scene(scene);
    tour.world.step();
    let trigger = match scene % 6 {
        0 => Some((64, 70)),
        1 => Some((64, 74)),
        5 => Some((34, 84)),
        _ => None,
    };
    if let Some((x, y)) = trigger {
        let _ = tour.ignite(x, y);
    }
    for _ in 0..8 {
        tour.world.step();
    }
    tour.world
        .set_budget(Credits::new(budget.clamp(25, 10_000_000)))
        .map_err(|error| JsValue::from_str(&format!("set initial budget: {error:?}")))?;
    Ok(tour.world)
}

// Browser timing is illustrative UI telemetry, never an input to simulation decisions.
fn browser_now_ms() -> f64 {
    web_sys::window()
        .and_then(|window| window.performance())
        .map_or(0.0, |performance| performance.now())
}
