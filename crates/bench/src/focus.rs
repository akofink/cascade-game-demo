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
    pub action_types: [ActionTypeResult; 3],
    pub action_effect_candidates: u64,
    pub background_quanta: u64,
    pub oldest_pending_age: u64,
    pub final_pending_channels: usize,
    pub final_hash: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionTypeResult {
    pub kind: &'static str,
    pub admitted: usize,
    pub effect_slices: Distribution,
    pub effect_wall_ns: Distribution,
    pub settle_slices: Distribution,
    pub settle_wall_ns: Distribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActionKind {
    Paint,
    Ignite,
    Detonate,
}
impl ActionKind {
    const ALL: [Self; 3] = [Self::Paint, Self::Ignite, Self::Detonate];
    const fn index(self) -> usize {
        match self {
            Self::Paint => 0,
            Self::Ignite => 1,
            Self::Detonate => 2,
        }
    }
    const fn name(self) -> &'static str {
        match self {
            Self::Paint => "paint",
            Self::Ignite => "ignite",
            Self::Detonate => "detonate",
        }
    }
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
    kind: ActionKind,
    sequence: u64,
    admitted_slice: u64,
    admitted_wall_ns: u128,
    effect: Option<(u64, u128)>,
    settle: Option<(u64, u128)>,
}

fn action_targets(
    world: &World,
    width: u32,
    height: u32,
    total_actions: usize,
) -> [Vec<(u32, u32)>; 3] {
    let mut counts = [0_usize; 3];
    for y in 0..height {
        for x in 0..width {
            let Some(cell) = world.cell(x, y) else {
                continue;
            };
            for kind in ActionKind::ALL {
                if kind == ActionKind::Paint && y >= height / 2 {
                    continue;
                }
                if action_target_matches(kind, cell.material, cell.burning) {
                    counts[kind.index()] += 1;
                }
            }
        }
    }
    let quotas: [usize; 3] =
        std::array::from_fn(|index| (total_actions.saturating_add(2 - index) / 3).max(1024));
    let mut targets: [Vec<(u32, u32)>; 3] =
        std::array::from_fn(|index| Vec::with_capacity(quotas[index].min(256)));
    let mut seen = [0_usize; 3];
    for y in 0..height {
        for x in 0..width {
            let Some(cell) = world.cell(x, y) else {
                continue;
            };
            for kind in ActionKind::ALL {
                if (kind == ActionKind::Paint && y >= height / 2)
                    || !action_target_matches(kind, cell.material, cell.burning)
                {
                    continue;
                }
                let index = kind.index();
                let rank = seen[index];
                seen[index] += 1;
                let quota = quotas[index].min(counts[index]);
                let selected = quota > 0
                    && ((rank as u128 * quota as u128) / counts[index] as u128)
                        < (((rank + 1) as u128 * quota as u128) / counts[index] as u128);
                if selected {
                    targets[index].push((x, y));
                }
            }
        }
    }
    for target_list in &mut targets {
        if target_list.is_empty() {
            target_list.push((width / 2, height / 2));
        }
    }
    targets
}

fn action_target_matches(kind: ActionKind, material: Material, burning: u8) -> bool {
    match kind {
        ActionKind::Paint => true,
        ActionKind::Ignite => {
            (material == Material::Wood && burning == 0) || material == Material::Explosive
        }
        ActionKind::Detonate => material == Material::Explosive,
    }
}

fn target_for_action(
    world: &World,
    kind: ActionKind,
    targets: &[(u32, u32)],
    ordinal: usize,
    kind_actions: usize,
) -> Option<(u32, u32)> {
    let start = ordinal.saturating_mul(targets.len()) / kind_actions.max(1);
    (0..targets.len()).find_map(|offset| {
        let (x, y) = targets[(start + offset) % targets.len()];
        let cell = world.cell(x, y)?;
        let valid = match kind {
            ActionKind::Paint => {
                let material = if ordinal.is_multiple_of(2) {
                    Material::Wood
                } else {
                    Material::Explosive
                };
                cell.material != material
            }
            ActionKind::Ignite => {
                (cell.material == Material::Wood && cell.burning == 0)
                    || cell.material == Material::Explosive
            }
            ActionKind::Detonate => cell.material == Material::Explosive,
        };
        valid.then_some((x, y))
    })
}

fn action_type_result(kind: ActionKind, actions: &[TrackedAction]) -> ActionTypeResult {
    let selected: Vec<_> = actions
        .iter()
        .filter(|action| action.kind == kind)
        .collect();
    ActionTypeResult {
        kind: kind.name(),
        admitted: selected.len(),
        effect_slices: distribution(
            selected
                .iter()
                .filter_map(|a| a.effect.map(|v| v.0 as u128))
                .collect(),
        ),
        effect_wall_ns: distribution(
            selected
                .iter()
                .filter_map(|a| a.effect.map(|v| v.1))
                .collect(),
        ),
        settle_slices: distribution(
            selected
                .iter()
                .filter_map(|a| a.settle.map(|v| v.0 as u128))
                .collect(),
        ),
        settle_wall_ns: distribution(
            selected
                .iter()
                .filter_map(|a| a.settle.map(|v| v.1))
                .collect(),
        ),
    }
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
    let total_actions = if slices == 0 {
        0
    } else {
        ((slices - 1) / 8 + 1) as usize
    };
    let action_targets = action_targets(&world, width, height, total_actions);
    let mut actions = Vec::<TrackedAction>::with_capacity(total_actions);
    let mut action_effect_candidates = 0u64;
    let mut last_painted_target = None;
    let mut background_quanta = 0u64;
    let mut oldest_pending_age = 0u64;
    let measured_started = Instant::now();
    for slice in 1..=slices {
        if slice % 4 == 0 {
            disturbances.admit_slice(&mut world);
        }
        if slice % 8 == 1 {
            let seq = (slice - 1) / 8;
            let kind = ActionKind::ALL[(seq % 3) as usize];
            let ordinal = (seq / 3) as usize;
            let targets = &action_targets[kind.index()];
            let kind_actions = total_actions.saturating_add(2 - kind.index()) / 3;
            let paint_material = if ordinal.is_multiple_of(2) {
                Material::Wood
            } else {
                Material::Explosive
            };
            let paired_target = (kind == ActionKind::Ignite)
                .then_some(last_painted_target)
                .flatten()
                .filter(|(x, y, material)| {
                    world.cell(*x, *y).is_some_and(|cell| {
                        cell.material == *material
                            && (*material == Material::Explosive || cell.burning == 0)
                    })
                })
                .map(|(x, y, _)| (x, y));
            let target = paired_target
                .or_else(|| target_for_action(&world, kind, targets, ordinal, kind_actions));
            let Some((x, y)) = target else {
                continue;
            };
            let Some(cell) = world.cell_id(x, y) else {
                continue;
            };
            let command = match kind {
                ActionKind::Paint => Command::Paint {
                    cell,
                    material: paint_material,
                },
                ActionKind::Ignite => Command::Ignite { cell },
                ActionKind::Detonate => Command::Detonate { cell, energy: 8 },
            };
            if world.submit(command) == cascade_sim::SubmitResult::Accepted {
                if kind == ActionKind::Paint {
                    last_painted_target = Some((x, y, paint_material));
                }
                let admitted_wall_ns = measured_started.elapsed().as_nanos();
                let sequence = world.latest_action_sequence().unwrap_or(0);
                actions.push(TrackedAction {
                    kind,
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
        action_effect_candidates += metrics.action_effect_candidates as u64;
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
        action_types: ActionKind::ALL.map(|kind| action_type_result(kind, &actions)),
        action_effect_candidates,
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
    if slices >= 720 {
        for result in &results {
            for action in result.action_types {
                if action.admitted < 30 || action.effect_slices.samples < 30 {
                    return Err(io::Error::other(format!(
                        "{} {} actions have insufficient visible-effect samples: admitted={}, effects={}",
                        result.policy, action.kind, action.admitted, action.effect_slices.samples
                    ))
                    .into());
                }
                if slices >= 4096 && action.settle_slices.samples < 30 {
                    return Err(io::Error::other(format!(
                        "{} {} actions have insufficient local-settle samples: {}",
                        result.policy, action.kind, action.settle_slices.samples
                    ))
                    .into());
                }
            }
        }
    }
    write!(
        output,
        "policy,width,height,budget,slices,actions,effect_n,effect_p50_slices,effect_p95_slices,effect_p99_slices,effect_max_slices,effect_p50_wall_ns,effect_p95_wall_ns,effect_p99_wall_ns,effect_max_wall_ns,settle_n,settle_p50_slices,settle_p95_slices,settle_p99_slices,settle_max_slices,settle_p50_wall_ns,settle_p95_wall_ns,settle_p99_wall_ns,settle_max_wall_ns,background_quanta,oldest_pending_age,final_pending_channels,measured_wall_ns,action_effect_candidates"
    )?;
    for kind in ActionKind::ALL {
        write!(
            output,
            ",{}_admitted,{}_effect_n,{}_effect_p50_slices,{}_effect_p95_slices,{}_effect_p50_wall_ns,{}_effect_p95_wall_ns,{}_settle_n,{}_settle_p50_slices,{}_settle_p95_slices,{}_settle_p50_wall_ns,{}_settle_p95_wall_ns",
            kind.name(),
            kind.name(),
            kind.name(),
            kind.name(),
            kind.name(),
            kind.name(),
            kind.name(),
            kind.name(),
            kind.name(),
            kind.name(),
            kind.name()
        )?;
    }
    writeln!(output, ",final_hash")?;
    for result in &results {
        write!(
            output,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
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
            result.action_effect_candidates,
        )?;
        for action in result.action_types {
            write!(
                output,
                ",{},{},{},{},{},{},{},{},{},{},{}",
                action.admitted,
                action.effect_slices.samples,
                action.effect_slices.p50.unwrap_or(0),
                action.effect_slices.p95.unwrap_or(0),
                action.effect_wall_ns.p50.unwrap_or(0),
                action.effect_wall_ns.p95.unwrap_or(0),
                action.settle_slices.samples,
                action.settle_slices.p50.unwrap_or(0),
                action.settle_slices.p95.unwrap_or(0),
                action.settle_wall_ns.p50.unwrap_or(0),
                action.settle_wall_ns.p95.unwrap_or(0),
            )?;
        }
        writeln!(output, ",{:#018x}", result.final_hash)?;
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
        let results = run_player_action_comparison(256, 256, 4096, 4096, &mut output).unwrap();
        assert_eq!(
            results.map(|result| result.policy),
            ["traditional", "bounded-fifo", "bounded-focus"]
        );
        assert!(results.iter().all(|result| result.action_count >= 30));
        for result in &results {
            for action in result.action_types {
                assert!(action.admitted >= 30);
                assert!(action.effect_slices.samples >= 30);
                assert!(action.settle_slices.samples >= 30);
            }
        }
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("bounded-focus"));
        let rows: Vec<_> = output.lines().collect();
        assert_eq!(rows[0].split(',').count(), rows[1].split(',').count());
    }
}
