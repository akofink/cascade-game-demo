# Headless performance evidence

## Scope and status

These are release-build measurements of the headless simulation step, not interactive frame-latency results. The charter's full milestone 5 acceptance suite is **not met**: the captures below are 300 slices per run, not at least 60 seconds per fixture, and do not measure preparation wall time, renderer/frame intervals, UI latency, or a post-burst recovery duration. The reference profile is provisional. No result here supports a general no-lag or deadline guarantee.

## Method

`cascade-bench` was built with `cargo build --release -p cascade-bench`. Each run created a fresh world, prepared its descriptor incrementally before measurement, then measured exactly 300 `World::step` calls with a monotonic wall clock around each call. Fixture preparation is excluded from slice durations. Three repetitions were performed per policy and fixture. Policy order alternated by run, bounded then traditional, for each repetition. The quiet-world baseline and all eight section-12 fixture descriptors were included as held-out workloads; mixed-overload alone was used as the calibration pilot. Raw CSV captures remain outside git.

World dimensions were 256 x 256, budget 4096 credits, and descriptor-defined capacities. These small dimensions were chosen to complete the full fixture matrix economically; they are not the charter 4096 x 4096 reference world. No warm-up slices were excluded: the current runner has no warm-up option. Samples include OS scheduling effects and use wall time, not thread CPU time. The interval summaries pool 900 samples per fixture and policy from three runs. `p50`, `p95`, and `p99` use the nearest lower indexed sample after sorting; max is the maximum observed. `>4 ms` is an exact count. Completion is the runner's pending-work and command completion flag at the end of the 300-slice run, not evidence of stable state after sustained load.

Machine: MacBook Air (Mac14,2), Apple M2, 8 CPU cores, 16 GB RAM, integrated Apple M2 GPU; macOS 27.0.1, built-in 2560 x 1664 display. Rust 1.89.0, aarch64-apple-darwin, Cargo 1.89.0. Repository revision: `fe3c0c2f0cb0891eb31e755a21817d1dab913c84`. Display refresh target, power mode, thermal state, and background load were not controlled or recorded. The benchmark is headless and does not exercise a GPU backend.

## Results

See [curated per-fixture summary](../benchmarks/results/headless-v1.md) for sample counts and the p50/p95/p99/max, over-budget count, backlog high-water, and completion status for bounded and traditional modes across all fixtures. In this run set, bounded slice durations remained below 4 ms for every sample, but this is only a 256 x 256 headless observation. Traditional mode's long or zero-work slices demonstrate its captured-frontier semantics; it is not a universal conventional-engine baseline.

The order-insensitive cross-mode plumbing test compares independent inert paint commands in a quiet fixture. It is a correctness check, not evidence that general bounded and traditional trajectories must match. Per-mode repeated final hashes were deterministic in the runner's tests.

## Credit calibration

`profiles/m2-16gb-v1.toml` records a provisional allowance of 4096 credits. The pilot used mixed-overload v1 at 256 x 256 with 1800 measured slices and a finite disturbance stream of 14,400 attempts at up to eight per slice. The single pilot run observed p99 0.004375 ms and max 0.005625 ms, with zero slices above 4 ms. Additional tested allowances were 2048, 1024, 512, and 256 credits. Their respective p99/max slice times were 0.006708/0.032291 ms, 0.002042/0.005583 ms, 0.002375/0.054000 ms, and 0.001292/0.022458 ms. Lower budgets reduced p99 in this pilot, but the experiment did not calculate completed-work throughput to quantify that trade-off; 256 credits also rejected 5,093 disturbance requests and coalesced 55. The 4096-credit setting was retained as a provisional candidate, not shown optimal. Calibration inputs are distinct from the held-out section-12 matrix.

This is not adequate calibration with expensive saturated workloads and headroom at representative/full world scale: it is one pilot repetition on a small world, without warm-up or control of machine state. The selected value is a provisional working profile, not certification. Although its observed pilot p99 is far below 4 ms, the desired acceptance claim remains unverified at the required conditions.

## Unmet targets and next measurements

- Run each fixture at least 60 seconds, three times, with explicit warm-up exclusion and alternating mode order, at fixed 1920 x 1080 interactive render settings. The current runner suite does not satisfy duration or rendering conditions.
- Calibrate with multiple expensive, saturated workloads and headroom on the reference world size, while retaining distinct held-out validation inputs.
- Report per-run preparation duration separately and completion/stable-state slices and wall time. The runner reports 290 preparation scheduler slices for the 256 x 256 runs, but does not time preparation separately.
- A finite burst/recovery probe used 1,200 disturbances followed by 1,650 no-new-input slices (1,800 total) across three repetitions. Both modes accepted all 1,200 with no rejection/coalescing. Bounded backlog high-water/end were 32,754/195; traditional were 12,260/194. Neither became empty/stable in the window. The probe order was not alternated, so it is supporting, not paired comparative evidence.
- Measure interactive frame interval p50/p95/p99/max, intervals over 33.3 ms, camera/UI feedback, uploads, and GPU timing in the app. This work intentionally did not modify `crates/app`; app-side interactive capture is outside these headless results.
- Verify full-size memory/high-water behavior and the charter's 4,096 x 4,096 workload. No full-size run is claimed here.

These gaps are reported rather than inferred from the headless slice measurements.
