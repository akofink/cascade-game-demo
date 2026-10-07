use cascade_bench::focus::run_player_action_comparison;
use cascade_bench::{BenchConfig, OutputFormat, run};
use cascade_sim::SchedulerPolicy;
use cascade_sim::fixtures::{FixtureId, ScenarioDescriptor};
use std::io;

fn main() {
    if let Err(error) = execute() {
        eprintln!("cascade-bench: {error}");
        std::process::exit(2);
    }
}

fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = BenchConfig::default();
    let mut format = OutputFormat::Csv;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut index = 0;
    let mut action_comparison = false;
    while index < args.len() {
        let option = args[index].as_str();
        if option == "--help" || option == "-h" {
            print_help();
            return Ok(());
        }
        if option == "--player-action-comparison" {
            action_comparison = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--fixture" => config.fixture = ScenarioDescriptor::get(parse_fixture(value)?),
            "--policy" => {
                config.policy = match value.as_str() {
                    "bounded" => SchedulerPolicy::Bounded,
                    "traditional" => SchedulerPolicy::Traditional,
                    _ => return Err(format!("invalid policy: {value}").into()),
                }
            }
            "--format" => {
                format = match value.as_str() {
                    "csv" => OutputFormat::Csv,
                    "json" => OutputFormat::Json,
                    _ => return Err(format!("invalid format: {value}").into()),
                }
            }
            "--slices" => config.slices = value.parse()?,
            "--warmup-slices" => config.warmup_slices = value.parse()?,
            "--width" => config.width = value.parse()?,
            "--height" => config.height = value.parse()?,
            "--budget" => config.budget = value.parse()?,
            "--disturbances" => config.disturbances = value.parse()?,
            "--disturbances-per-slice" => config.disturbances_per_slice = value.parse()?,
            _ => return Err(format!("unknown option: {option}").into()),
        }
        index += 2;
    }
    let stdout = io::stdout();
    let mut output = stdout.lock();
    if action_comparison {
        run_player_action_comparison(
            config.width,
            config.height,
            config.budget,
            config.slices,
            &mut output,
        )?;
        return Ok(());
    }
    let summary = run(config, format, &mut output)?;
    if format == OutputFormat::Csv {
        eprintln!(
            "final_hash={:016x} complete={} preparation_slices={} preparation_wall_ns={} warmup_slices={} warmup_wall_ns={} measured_slices={} measured_wall_ns={} completed_work_quanta={} first_completion_slice={:?} completion_wall_ns={:?} stable_window_start_slice={:?} stable_window_wall_ns={:?} pending_channels_by_material={:?}",
            summary.final_hash,
            summary.complete,
            summary.preparation_slices,
            summary.preparation_wall_ns,
            summary.warmup_slices,
            summary.warmup_wall_ns,
            summary.measured_slices,
            summary.measured_wall_ns,
            summary.completed_work_quanta,
            summary.first_completion_slice,
            summary.completion_wall_ns,
            summary.stable_window_start_slice,
            summary.stable_window_wall_ns,
            summary.pending_channels_by_material
        );
    }
    Ok(())
}

fn parse_fixture(value: &str) -> Result<FixtureId, String> {
    match value {
        "quiet-world" => Ok(FixtureId::QuietWorld),
        "explosive-lattice" => Ok(FixtureId::ExplosiveLattice),
        "sand-release" => Ok(FixtureId::SandRelease),
        "reservoir-breach" => Ok(FixtureId::ReservoirBreach),
        "burning-forest" => Ok(FixtureId::BurningForest),
        "dirty-world-sweep" => Ok(FixtureId::DirtyWorldSweep),
        "tiny-capacity" => Ok(FixtureId::TinyCapacity),
        "mixed-overload" => Ok(FixtureId::MixedOverload),
        _ => Err(format!("unknown fixture: {value}")),
    }
}

fn print_help() {
    println!(
        "cascade-bench [--fixture NAME] [--policy bounded|traditional] [--slices N] [--warmup-slices N] [--format csv|json] [--width N] [--height N] [--budget CREDITS] [--disturbances N] [--disturbances-per-slice N] [--player-action-comparison]"
    );
    println!(
        "Fixture preparation and optional warm-up are timed separately and excluded from measured slice durations. The player-action comparison runs all three policies against mixed overload."
    );
}
