//! Headless benchmark runner for reproducible fixture/policy comparisons.

pub mod focus;

use cascade_sim::fixtures::{
    DisturbanceCommandStream, DisturbanceMetrics, FixtureError, FixtureId,
    MAX_DISTURBANCES_PER_SLICE, ScenarioDescriptor,
};
use cascade_sim::{Capacity, Credits, SchedulerPolicy, SimError, SliceMetrics, World};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputFormat {
    Csv,
    Json,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BenchConfig {
    pub fixture: ScenarioDescriptor,
    pub policy: SchedulerPolicy,
    pub width: u32,
    pub height: u32,
    pub budget: u32,
    pub slices: u64,
    pub warmup_slices: u64,
    pub disturbances: u64,
    pub disturbances_per_slice: u32,
}
impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            fixture: ScenarioDescriptor::get(FixtureId::MixedOverload),
            policy: SchedulerPolicy::Bounded,
            width: 256,
            height: 256,
            budget: 4_096,
            slices: 300,
            warmup_slices: 0,
            disturbances: 0,
            disturbances_per_slice: 8,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BenchTiming {
    elapsed_wall_ns: u128,
    completed_work_quanta: u64,
    preparation_slices: u64,
    preparation_wall_ns: u128,
    warmup_slices: u64,
    warmup_wall_ns: u128,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BenchRow {
    pub slice: u64,
    pub credits_allowed: u32,
    pub credits_used: u32,
    pub selection_probes: u32,
    pub executed_quanta: u32,
    pub backlog: usize,
    pub ready_jobs: usize,
    pub pending_channels: usize,
    pub command_backlog: usize,
    pub slice_cpu_ns: u128,
    pub elapsed_wall_ns: u128,
    pub completed_work_quanta: u64,
    pub preparation_slices: u64,
    pub preparation_wall_ns: u128,
    pub warmup_slices: u64,
    pub warmup_wall_ns: u128,
    pub disturbance_attempted: u32,
    pub disturbance_accepted: u32,
    pub disturbance_coalesced: u32,
    pub disturbance_rejected: u32,
    pub complete: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BenchSummary {
    pub fixture: ScenarioDescriptor,
    pub policy: SchedulerPolicy,
    pub width: u32,
    pub height: u32,
    pub budget: u32,
    pub preparation_slices: u64,
    pub preparation_wall_ns: u128,
    pub warmup_slices: u64,
    pub warmup_wall_ns: u128,
    pub measured_slices: u64,
    pub measured_wall_ns: u128,
    pub first_completion_slice: Option<u64>,
    pub completion_wall_ns: Option<u128>,
    pub completed_work_quanta: u64,
    pub final_hash: u64,
    pub complete: bool,
}

#[derive(Debug)]
pub enum BenchError {
    Io(io::Error),
    Simulation(SimError),
    Fixture(FixtureError),
    PreparationTimedOut,
}
impl From<io::Error> for BenchError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<SimError> for BenchError {
    fn from(value: SimError) -> Self {
        Self::Simulation(value)
    }
}
impl From<FixtureError> for BenchError {
    fn from(value: FixtureError) -> Self {
        Self::Fixture(value)
    }
}
impl std::fmt::Display for BenchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "benchmark output failed: {error}"),
            Self::Simulation(error) => write!(f, "simulation setup failed: {error:?}"),
            Self::Fixture(error) => write!(f, "fixture setup failed: {error:?}"),
            Self::PreparationTimedOut => {
                write!(f, "fixture preparation exceeded its structural watchdog")
            }
        }
    }
}
impl std::error::Error for BenchError {}

/// Prepare a fresh world outside measured slices, then emit one record per simulation slice.
pub fn run<W: Write>(
    config: BenchConfig,
    format: OutputFormat,
    output: &mut W,
) -> Result<BenchSummary, BenchError> {
    let (ready_capacity, command_capacity) = config.fixture.capacities();
    let mut world = World::new(
        config.width,
        config.height,
        Credits::new(config.budget),
        ready_capacity,
        command_capacity,
    )?;
    let preparation_started = Instant::now();
    world.start_fixture_with_policy(config.fixture, config.policy)?;

    let preparation_watchdog = world.cell_count().saturating_mul(2).saturating_add(16);
    let mut preparation_slices = 0u64;
    loop {
        match world.fixture_progress() {
            Some(progress) if progress.complete => break,
            Some(progress) if progress.cancelled => return Err(BenchError::PreparationTimedOut),
            Some(_) => {}
            None => return Err(BenchError::PreparationTimedOut),
        }
        if preparation_slices as usize >= preparation_watchdog {
            return Err(BenchError::PreparationTimedOut);
        }
        let metrics = world.step();
        if config.policy == SchedulerPolicy::Bounded && metrics.charged > metrics.allowed {
            return Err(BenchError::PreparationTimedOut);
        }
        preparation_slices += 1;
    }
    let mut preparation_wall_ns = preparation_started.elapsed().as_nanos();

    let warmup_started = Instant::now();
    for _ in 0..config.warmup_slices {
        let metrics = world.step();
        if config.policy == SchedulerPolicy::Bounded && metrics.charged > metrics.allowed {
            return Err(BenchError::PreparationTimedOut);
        }
    }
    let warmup_wall_ns = warmup_started.elapsed().as_nanos();

    if config.warmup_slices > 0 {
        let preparation_started = Instant::now();
        world.start_fixture_with_policy(config.fixture, config.policy)?;
        let mut measured_prep_slices = 0u64;
        loop {
            match world.fixture_progress() {
                Some(progress) if progress.complete => break,
                Some(progress) if progress.cancelled => {
                    return Err(BenchError::PreparationTimedOut);
                }
                Some(_) => {}
                None => return Err(BenchError::PreparationTimedOut),
            }
            if measured_prep_slices as usize >= preparation_watchdog {
                return Err(BenchError::PreparationTimedOut);
            }
            let metrics = world.step();
            if config.policy == SchedulerPolicy::Bounded && metrics.charged > metrics.allowed {
                return Err(BenchError::PreparationTimedOut);
            }
            measured_prep_slices += 1;
        }
        preparation_slices += measured_prep_slices;
        preparation_wall_ns += preparation_started.elapsed().as_nanos();
    }

    match format {
        OutputFormat::Csv => writeln!(
            output,
            "fixture,fixture_version,seed,policy,width,height,budget,slice,credits_allowed,credits_used,selection_probes,executed_quanta,completed_work_quanta,elapsed_wall_ns,preparation_slices,preparation_wall_ns,warmup_slices,warmup_wall_ns,backlog,ready_jobs,pending_channels,command_backlog,disturbance_attempted,disturbance_accepted,disturbance_coalesced,disturbance_rejected,slice_cpu_ns,complete"
        )?,
        OutputFormat::Json => write!(
            output,
            "{{\"fixture\":\"{}\",\"fixture_version\":{},\"seed\":{},\"policy\":\"{}\",\"width\":{},\"height\":{},\"budget\":{},\"preparation_slices\":{},\"preparation_wall_ns\":{},\"warmup_slices\":{},\"warmup_wall_ns\":{},\"disturbance_limit\":{},\"disturbances_per_slice\":{},\"slices\":[",
            config.fixture.name(),
            config.fixture.version,
            config.fixture.seed,
            config.policy.name(),
            config.width,
            config.height,
            config.budget,
            preparation_slices,
            preparation_wall_ns,
            config.warmup_slices,
            warmup_wall_ns,
            config.disturbances,
            config
                .disturbances_per_slice
                .min(MAX_DISTURBANCES_PER_SLICE)
        )?,
    }

    let mut first_json_row = true;
    let mut completed_work_quanta = 0u64;
    let measured_started = Instant::now();
    let mut first_completion_slice = None;
    let mut completion_wall_ns = None;
    let mut disturbance_stream = DisturbanceCommandStream::new(
        config.fixture.seed ^ 0xd157_0b00_0000_0001,
        config.disturbances,
        config
            .disturbances_per_slice
            .min(MAX_DISTURBANCES_PER_SLICE),
    );
    for slice in 1..=config.slices {
        let disturbance_metrics = disturbance_stream.admit_slice(&mut world);
        let started = Instant::now();
        let metrics = world.step();
        let slice_cpu_ns = started.elapsed().as_nanos();
        completed_work_quanta = completed_work_quanta.saturating_add(
            (metrics.evaluations
                + metrics.blasts
                + metrics.recoveries
                + metrics.commands
                + metrics.reset_cells
                + metrics.prepared_cells) as u64,
        );
        let elapsed_wall_ns = measured_started.elapsed().as_nanos();
        let row = make_row(
            slice,
            metrics,
            &world,
            slice_cpu_ns,
            BenchTiming {
                elapsed_wall_ns,
                completed_work_quanta,
                preparation_slices,
                preparation_wall_ns,
                warmup_slices: config.warmup_slices,
                warmup_wall_ns,
            },
            disturbance_metrics,
            disturbance_stream.finished(),
        );
        if row.complete && first_completion_slice.is_none() {
            first_completion_slice = Some(slice);
            completion_wall_ns = Some(elapsed_wall_ns);
        }
        match format {
            OutputFormat::Csv => write_csv_row(output, config.fixture, config.policy, config, row)?,
            OutputFormat::Json => {
                if !first_json_row {
                    write!(output, ",")?;
                }
                first_json_row = false;
                write_json_row(output, row)?;
            }
        }
    }
    let measured_wall_ns = measured_started.elapsed().as_nanos();
    if format == OutputFormat::Json {
        write!(output, "]")?;
    }
    let mut hasher = DefaultHasher::new();
    config.fixture.hash(&mut hasher);
    world.state_hash().hash(&mut hasher);
    disturbance_stream.hash(&mut hasher);
    let final_hash = hasher.finish();
    let complete =
        disturbance_stream.finished() && world.pending_channels() == 0 && world.command_len() == 0;
    if format == OutputFormat::Json {
        let first_completion_slice_json =
            first_completion_slice.map_or_else(|| "null".to_string(), |value| value.to_string());
        let completion_wall_ns_json =
            completion_wall_ns.map_or_else(|| "null".to_string(), |value| value.to_string());
        writeln!(
            output,
            ",\"measured_wall_ns\":{measured_wall_ns},\"completed_work_quanta\":{completed_work_quanta},\"first_completion_slice\":{first_completion_slice_json},\"completion_wall_ns\":{completion_wall_ns_json},\"final_hash\":\"{final_hash:016x}\",\"complete\":{complete}}}"
        )?;
    }
    Ok(BenchSummary {
        fixture: config.fixture,
        policy: config.policy,
        width: config.width,
        height: config.height,
        budget: config.budget,
        preparation_slices,
        preparation_wall_ns,
        warmup_slices: config.warmup_slices,
        warmup_wall_ns,
        measured_slices: config.slices,
        measured_wall_ns,
        first_completion_slice,
        completion_wall_ns,
        completed_work_quanta,
        final_hash,
        complete,
    })
}

fn make_row(
    slice: u64,
    metrics: SliceMetrics,
    world: &World,
    slice_cpu_ns: u128,
    timing: BenchTiming,
    disturbance: DisturbanceMetrics,
    stream_finished: bool,
) -> BenchRow {
    let command_backlog = world.command_len();
    let BenchTiming {
        elapsed_wall_ns,
        completed_work_quanta,
        preparation_slices,
        preparation_wall_ns,
        warmup_slices,
        warmup_wall_ns,
    } = timing;
    BenchRow {
        slice,
        credits_allowed: metrics.allowed,
        credits_used: metrics.charged,
        selection_probes: metrics.selections,
        executed_quanta: metrics.evaluations
            + metrics.blasts
            + metrics.recoveries
            + metrics.commands
            + metrics.reset_cells
            + metrics.prepared_cells,
        backlog: metrics.pending_cells + command_backlog,
        ready_jobs: metrics.ready_len,
        pending_channels: metrics.pending_cells,
        command_backlog,
        slice_cpu_ns,
        elapsed_wall_ns,
        completed_work_quanta,
        preparation_slices,
        preparation_wall_ns,
        warmup_slices,
        warmup_wall_ns,
        disturbance_attempted: disturbance.attempted,
        disturbance_accepted: disturbance.accepted,
        disturbance_coalesced: disturbance.coalesced,
        disturbance_rejected: disturbance.rejected,
        complete: stream_finished && metrics.pending_cells == 0 && command_backlog == 0,
    }
}

fn write_csv_row<W: Write>(
    output: &mut W,
    fixture: ScenarioDescriptor,
    policy: SchedulerPolicy,
    config: BenchConfig,
    row: BenchRow,
) -> io::Result<()> {
    writeln!(
        output,
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        fixture.name(),
        fixture.version,
        fixture.seed,
        policy.name(),
        config.width,
        config.height,
        config.budget,
        row.slice,
        row.credits_allowed,
        row.credits_used,
        row.selection_probes,
        row.executed_quanta,
        row.completed_work_quanta,
        row.elapsed_wall_ns,
        row.preparation_slices,
        row.preparation_wall_ns,
        row.warmup_slices,
        row.warmup_wall_ns,
        row.backlog,
        row.ready_jobs,
        row.pending_channels,
        row.command_backlog,
        row.disturbance_attempted,
        row.disturbance_accepted,
        row.disturbance_coalesced,
        row.disturbance_rejected,
        row.slice_cpu_ns,
        row.complete
    )
}

fn write_json_row<W: Write>(output: &mut W, row: BenchRow) -> io::Result<()> {
    write!(
        output,
        "{{\"slice\":{},\"credits_allowed\":{},\"credits_used\":{},\"selection_probes\":{},\"executed_quanta\":{},\"completed_work_quanta\":{},\"elapsed_wall_ns\":{},\"preparation_slices\":{},\"preparation_wall_ns\":{},\"warmup_slices\":{},\"warmup_wall_ns\":{},\"backlog\":{},\"ready_jobs\":{},\"pending_channels\":{},\"command_backlog\":{},\"disturbance_attempted\":{},\"disturbance_accepted\":{},\"disturbance_coalesced\":{},\"disturbance_rejected\":{},\"slice_cpu_ns\":{},\"complete\":{}}}",
        row.slice,
        row.credits_allowed,
        row.credits_used,
        row.selection_probes,
        row.executed_quanta,
        row.completed_work_quanta,
        row.elapsed_wall_ns,
        row.preparation_slices,
        row.preparation_wall_ns,
        row.warmup_slices,
        row.warmup_wall_ns,
        row.backlog,
        row.ready_jobs,
        row.pending_channels,
        row.command_backlog,
        row.disturbance_attempted,
        row.disturbance_accepted,
        row.disturbance_coalesced,
        row.disturbance_rejected,
        row.slice_cpu_ns,
        row.complete
    )
}

/// Order-insensitive plumbing check using independent inert paint requests over a quiet fixture.
pub fn cross_mode_order_insensitive_check() -> Result<bool, BenchError> {
    fn prepare(policy: SchedulerPolicy) -> Result<World, BenchError> {
        let mut world = World::new(
            16,
            16,
            Credits::new(512),
            Capacity::new(32),
            Capacity::new(16),
        )?;
        world.start_fixture_with_policy(ScenarioDescriptor::get(FixtureId::QuietWorld), policy)?;
        while world
            .fixture_progress()
            .is_some_and(|progress| !progress.complete)
        {
            let metrics = world.step();
            if metrics.charged > metrics.allowed {
                return Err(BenchError::PreparationTimedOut);
            }
        }
        for index in [17usize, 51, 109] {
            let id = world.cell_id(index as u32 % 16, index as u32 / 16).unwrap();
            world.submit(cascade_sim::Command::Paint {
                cell: id,
                material: cascade_sim::Material::Stone,
            });
        }
        Ok(world)
    }
    let mut bounded = prepare(SchedulerPolicy::Bounded)?;
    let mut traditional = prepare(SchedulerPolicy::Traditional)?;
    for _ in 0..64 {
        bounded.step();
        traditional.step();
        if bounded.pending_channels() == 0
            && bounded.command_len() == 0
            && traditional.pending_channels() == 0
            && traditional.command_len() == 0
        {
            break;
        }
    }
    for y in 0..16 {
        for x in 0..16 {
            if bounded.cell(x, y) != traditional.cell(x, y) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_mode_replay_hash_is_deterministic() {
        let config = BenchConfig {
            fixture: ScenarioDescriptor::get(FixtureId::ExplosiveLattice),
            width: 24,
            height: 24,
            slices: 30,
            disturbances: 90,
            disturbances_per_slice: 3,
            ..BenchConfig::default()
        };
        let mut a = Vec::new();
        let mut b = Vec::new();
        let a_summary = run(config, OutputFormat::Json, &mut a).unwrap();
        let b_summary = run(config, OutputFormat::Json, &mut b).unwrap();
        assert_eq!(a_summary.final_hash, b_summary.final_hash);
        assert_eq!(a_summary.preparation_slices, b_summary.preparation_slices);
    }

    #[test]
    fn warmup_is_excluded_and_measured_fixture_is_reprepared() {
        let base = BenchConfig {
            fixture: ScenarioDescriptor::get(FixtureId::QuietWorld),
            width: 8,
            height: 8,
            slices: 3,
            ..BenchConfig::default()
        };
        let mut cold_output = Vec::new();
        let cold = run(base, OutputFormat::Csv, &mut cold_output).unwrap();
        let mut warm_output = Vec::new();
        let warm = run(
            BenchConfig {
                warmup_slices: 2,
                ..base
            },
            OutputFormat::Csv,
            &mut warm_output,
        )
        .unwrap();
        assert_eq!(warm.warmup_slices, 2);
        assert!(warm.warmup_wall_ns > 0);
        assert!(warm.preparation_slices >= cold.preparation_slices * 2);
        let cold_output = String::from_utf8(cold_output).unwrap();
        let warm_output = String::from_utf8(warm_output).unwrap();
        let cold_rows: Vec<_> = cold_output.lines().collect();
        let warm_rows: Vec<_> = warm_output.lines().collect();
        assert_eq!(warm_rows[0].split(',').count(), 28);
        assert_eq!(warm_rows[1].split(',').count(), 28);
        for column in [11, 18] {
            assert_eq!(
                cold_rows[1].split(',').nth(column),
                warm_rows[1].split(',').nth(column)
            );
        }
    }

    #[test]
    fn cross_mode_order_insensitive_fixture_check_passes() {
        assert!(cross_mode_order_insensitive_check().unwrap());
    }
}
