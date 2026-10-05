//! Three-policy deterministic player-action latency comparison for the full mixed-overload fixture.

use cascade_sim::fixtures::{DisturbanceCommandStream, FixtureId, ScenarioDescriptor};
use cascade_sim::{Command, Credits, Material, SchedulerPolicy, SimError, World};
use std::io::{self, Write};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Distribution {
    pub samples: usize,
    pub p50: Option<u128>,
    pub p95: Option<u128>,
    pub p99: Option<u128>,
    pub max: Option<u128>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PolicyResult {
    pub policy: &'static str,
    pub measured_slices: u64,
    pub measured_wall_ns: u128,
    pub action_count: usize,
    pub effect_slices: Distribution,
    pub effect_wall_ns: Distribution,
    pub settle_slices: Distribution,
    pub settle_wall_ns: Distribution,
    pub background_quanta: u64,
    pub oldest_pending_age: u64,
    pub final_pending_channels: usize,
    pub final_hash: u64,
}

fn distribution(mut values: Vec<u128>) -> Distribution {
    if values.is_empty() {
        return Distribution {
            samples: 0,
            p50: None,
            p95: None,
            p99: None,
            max: None,
        };
    }
    values.sort_unstable();
    let at = |percent: usize| {
        values[values
            .len()
            .saturating_mul(percent)
            .div_ceil(100)
            .saturating_sub(1)]
    };
    Distribution {
        samples: values.len(),
        p50: Some(at(50)),
        p95: Some(at(95)),
        p99: Some(at(99)),
        max: values.last().copied(),
    }
}

#[derive(Clone, Copy)]
struct TrackedAction {
    sequence: u64,
    admitted_slice: u64,
    admitted_wall_ns: u128,
    effect: Option<(u64, u128)>,
    settle: Option<(u64, u128)>,
}

fn run_policy(
    policy: SchedulerPolicy,
    focus_enabled: bool,
    width: u32,
    height: u32,
    budget: u32,
    slices: u64,
) -> Result<PolicyResult, SimError> {
    let fixture = ScenarioDescriptor::get(FixtureId::MixedOverload);
    let (ready_capacity, command_capacity) = fixture.capacities();
    let mut world = World::new(
        width,
        height,
        Credits::new(budget),
        ready_capacity,
        command_capacity,
    )?;
    world.set_focus_enabled(focus_enabled);
    world
        .start_fixture_with_policy(fixture, policy)
        .map_err(|_| SimError::ResourceLimitExceeded)?;
    let watchdog = world.cell_count().saturating_mul(2).saturating_add(16);
    let mut prep_slices = 0usize;
    while !world
        .fixture_progress()
        .is_some_and(|progress| progress.complete)
    {
        if prep_slices >= watchdog {
            return Err(SimError::ResourceLimitExceeded);
        }
        let metrics = world.step();
        if policy == SchedulerPolicy::Bounded && metrics.charged > metrics.allowed {
            return Err(SimError::BudgetTooSmall);
        }
        prep_slices += 1;
    }

    let mut disturbances = DisturbanceCommandStream::new(
        fixture.seed ^ 0x5a11_0ad5_f00d_0001,
        slices / 4 + u64::from(!slices.is_multiple_of(4)),
        1,
    );
    let mut actions = Vec::<TrackedAction>::with_capacity(128);
    let mut background_quanta = 0u64;
    let mut oldest_pending_age = 0u64;
    let measured_started = Instant::now();
    for slice in 1..=slices {
        if slice % 4 == 0 {
            disturbances.admit_slice(&mut world);
        }
        if slice % 8 == 1 {
            let seq = (slice - 1) / 8;
            let x = width / 2 + ((seq / 4) % 3) as u32;
            let y = height / 2 + ((seq / 4) % 3) as u32;
            let Some(cell) = world.cell_id(x.min(width - 1), y.min(height - 1)) else {
                continue;
            };
            let command = match seq % 4 {
                0 => Command::Paint {
                    cell,
                    material: Material::Wood,
                },
                1 => Command::Ignite { cell },
                2 => Command::Paint {
                    cell,
                    material: Material::Explosive,
                },
                _ => Command::Detonate { cell, energy: 8 },
            };
            if world.submit(command) == cascade_sim::SubmitResult::Accepted {
                let admitted_wall_ns = measured_started.elapsed().as_nanos();
                let sequence = world.latest_action_sequence().unwrap_or(0);
                actions.push(TrackedAction {
                    sequence,
                    admitted_slice: world.slice_index() + 1,
                    admitted_wall_ns,
                    effect: None,
                    settle: None,
                });
            }
        }
        let metrics = world.step();
        if policy == SchedulerPolicy::Bounded && metrics.charged > metrics.allowed {
            return Err(SimError::BudgetTooSmall);
        }
        background_quanta += if policy == SchedulerPolicy::Traditional {
            (metrics.evaluations + metrics.blasts) as u64
        } else {
            (metrics.background_evaluations + metrics.background_blasts) as u64
        };
        oldest_pending_age = oldest_pending_age.max(metrics.oldest_pending_age);
        let now_ns = measured_started.elapsed().as_nanos();
        for action in &mut actions {
            if let Some(record) = world.action_record(action.sequence) {
                if action.effect.is_none()
                    && let Some(slice) = record.first_effect_slice
                {
                    action.effect = Some((
                        slice.saturating_sub(action.admitted_slice),
                        now_ns.saturating_sub(action.admitted_wall_ns),
                    ));
                }
                if action.settle.is_none()
                    && let Some(slice) = record.local_settle_slice
                {
                    action.settle = Some((
                        slice.saturating_sub(action.admitted_slice),
                        now_ns.saturating_sub(action.admitted_wall_ns),
                    ));
                }
            }
        }
    }
    let measured_wall_ns = measured_started.elapsed().as_nanos();
    let effect_slices = distribution(
        actions
            .iter()
            .filter_map(|a| a.effect.map(|v| v.0 as u128))
            .collect(),
    );
    let effect_wall_ns = distribution(
        actions
            .iter()
            .filter_map(|a| a.effect.map(|v| v.1))
            .collect(),
    );
    let settle_slices = distribution(
        actions
            .iter()
            .filter_map(|a| a.settle.map(|v| v.0 as u128))
            .collect(),
    );
    let settle_wall_ns = distribution(
        actions
            .iter()
            .filter_map(|a| a.settle.map(|v| v.1))
            .collect(),
    );
    Ok(PolicyResult {
        policy: match policy {
            SchedulerPolicy::Traditional => "traditional",
            SchedulerPolicy::Bounded if focus_enabled => "bounded-focus",
            SchedulerPolicy::Bounded => "bounded-fifo",
        },
        measured_slices: slices,
        measured_wall_ns,
        action_count: actions.len(),
        effect_slices,
        effect_wall_ns,
        settle_slices,
        settle_wall_ns,
        background_quanta,
        oldest_pending_age,
        final_pending_channels: world.pending_channels(),
        final_hash: world.state_hash(),
    })
}

/// Run traditional, bounded FIFO, and bounded-with-focus over identical full-size mixed overload.
pub fn run_player_action_comparison<W: Write>(
    width: u32,
    height: u32,
    budget: u32,
    slices: u64,
    output: &mut W,
) -> Result<[PolicyResult; 3], Box<dyn std::error::Error>> {
    let results = [
        run_policy(
            SchedulerPolicy::Traditional,
            false,
            width,
            height,
            budget,
            slices,
        )
        .map_err(|error| io::Error::other(format!("traditional benchmark failed: {error:?}")))?,
        run_policy(
            SchedulerPolicy::Bounded,
            false,
            width,
            height,
            budget,
            slices,
        )
        .map_err(|error| io::Error::other(format!("bounded FIFO benchmark failed: {error:?}")))?,
        run_policy(
            SchedulerPolicy::Bounded,
            true,
            width,
            height,
            budget,
            slices,
        )
        .map_err(|error| io::Error::other(format!("bounded focus benchmark failed: {error:?}")))?,
    ];
    writeln!(
        output,
        "policy,width,height,budget,slices,actions,effect_n,effect_p50_slices,effect_p95_slices,effect_p99_slices,effect_max_slices,effect_p50_wall_ns,effect_p95_wall_ns,effect_p99_wall_ns,effect_max_wall_ns,settle_n,settle_p50_slices,settle_p95_slices,settle_p99_slices,settle_max_slices,settle_p50_wall_ns,settle_p95_wall_ns,settle_p99_wall_ns,settle_max_wall_ns,background_quanta,oldest_pending_age,final_pending_channels,measured_wall_ns,final_hash"
    )?;
    for result in &results {
        writeln!(
            output,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:016x}",
            result.policy,
            width,
            height,
            budget,
            slices,
            result.action_count,
            result.effect_slices.samples,
            result.effect_slices.p50.unwrap_or(0),
            result.effect_slices.p95.unwrap_or(0),
            result.effect_slices.p99.unwrap_or(0),
            result.effect_slices.max.unwrap_or(0),
            result.effect_wall_ns.p50.unwrap_or(0),
            result.effect_wall_ns.p95.unwrap_or(0),
            result.effect_wall_ns.p99.unwrap_or(0),
            result.effect_wall_ns.max.unwrap_or(0),
            result.settle_slices.samples,
            result.settle_slices.p50.unwrap_or(0),
            result.settle_slices.p95.unwrap_or(0),
            result.settle_slices.p99.unwrap_or(0),
            result.settle_slices.max.unwrap_or(0),
            result.settle_wall_ns.p50.unwrap_or(0),
            result.settle_wall_ns.p95.unwrap_or(0),
            result.settle_wall_ns.p99.unwrap_or(0),
            result.settle_wall_ns.max.unwrap_or(0),
            result.background_quanta,
            result.oldest_pending_age,
            result.final_pending_channels,
            result.measured_wall_ns,
            result.final_hash
        )?;
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_distribution_uses_nearest_rank() {
        assert_eq!(distribution(vec![4, 1, 3, 2]).p50, Some(2));
        assert_eq!(distribution(Vec::new()).p95, None);
    }

    #[test]
    fn comparison_runs_three_reproducible_policies() {
        let mut output = Vec::new();
        let results = run_player_action_comparison(64, 64, 4096, 64, &mut output).unwrap();
        assert_eq!(
            results.map(|result| result.policy),
            ["traditional", "bounded-fifo", "bounded-focus"]
        );
        assert!(results.iter().all(|result| result.action_count > 0));
        assert!(
            results
                .iter()
                .all(|result| result.effect_slices.samples > 0)
        );
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("bounded-focus"));
        let rows: Vec<_> = output.lines().collect();
        assert_eq!(rows[0].split(',').count(), rows[1].split(',').count());
    }
}
