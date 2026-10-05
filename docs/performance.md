# Performance evidence

## Headless simulation evidence

### Scope and status

These are release-build measurements of the headless simulation step, not interactive frame-latency results. The charter's full milestone 5 acceptance suite is **not met**: the captures below are 300 slices per run, not at least 60 seconds per fixture, and do not measure preparation wall time, renderer/frame intervals, UI latency, or a post-burst recovery duration. The reference profile is provisional. No result here supports a general no-lag or deadline guarantee.

### Method

`cascade-bench` was built with `cargo build --release -p cascade-bench`. Each run created a fresh world, prepared its descriptor incrementally before measurement, then measured exactly 300 `World::step` calls with a monotonic wall clock around each call. Fixture preparation is excluded from slice durations. Three repetitions were performed per policy and fixture. Policy order alternated by run, bounded then traditional, for each repetition. The quiet-world baseline and all eight section-12 fixture descriptors were included as held-out workloads; mixed-overload alone was used as the calibration pilot. Raw CSV captures remain outside git.

World dimensions were 256 x 256, budget 4096 credits, and descriptor-defined capacities. These small dimensions were chosen to complete the full fixture matrix economically; they are not the charter 4096 x 4096 reference world. No warm-up slices were excluded: the current runner has no warm-up option. Samples include OS scheduling effects and use wall time, not thread CPU time. The interval summaries pool 900 samples per fixture and policy from three runs. `p50`, `p95`, and `p99` use the nearest lower indexed sample after sorting; max is the maximum observed. `>4 ms` is an exact count. Completion is the runner's pending-work and command completion flag at the end of the 300-slice run, not evidence of stable state after sustained load.

Machine: MacBook Air (Mac14,2), Apple M2, 8 CPU cores, 16 GB RAM, integrated Apple M2 GPU; macOS 27.0.1, built-in 2560 x 1664 display. Rust 1.89.0, aarch64-apple-darwin, Cargo 1.89.0. Repository revision: `fe3c0c2f0cb0891eb31e755a21817d1dab913c84`. Display refresh target, power mode, thermal state, and background load were not controlled or recorded. The benchmark is headless and does not exercise a GPU backend.

### Results

See [curated per-fixture summary](../benchmarks/results/headless-v1.md) for sample counts and the p50/p95/p99/max, over-budget count, backlog high-water, and completion status for bounded and traditional modes across all fixtures. In this run set, bounded slice durations remained below 4 ms for every sample, but this is only a 256 x 256 headless observation. Traditional mode's long or zero-work slices demonstrate its captured-frontier semantics; it is not a universal conventional-engine baseline.

The order-insensitive cross-mode plumbing test compares independent inert paint commands in a quiet fixture. It is a correctness check, not evidence that general bounded and traditional trajectories must match. Per-mode repeated final hashes were deterministic in the runner's tests.

### Credit calibration

`profiles/m2-16gb-v1.toml` records a provisional allowance of 4096 credits. The pilot used mixed-overload v1 at 256 x 256 with 1800 measured slices and a finite disturbance stream of 14,400 attempts at up to eight per slice. The single pilot run observed p99 0.004375 ms and max 0.005625 ms, with zero slices above 4 ms. Additional tested allowances were 2048, 1024, 512, and 256 credits. Their respective p99/max slice times were 0.006708/0.032291 ms, 0.002042/0.005583 ms, 0.002375/0.054000 ms, and 0.001292/0.022458 ms. Lower budgets reduced p99 in this pilot, but the experiment did not calculate completed-work throughput to quantify that trade-off; 256 credits also rejected 5,093 disturbance requests and coalesced 55. The 4096-credit setting was retained as a provisional candidate, not shown optimal. Calibration inputs are distinct from the held-out section-12 matrix.

This is not adequate calibration with expensive saturated workloads and headroom at representative/full world scale: it is one pilot repetition on a small world, without warm-up or control of machine state. The selected value is a provisional working profile, not certification. Although its observed pilot p99 is far below 4 ms, the desired acceptance claim remains unverified at the required conditions.

## Native M2 smoke

- Host: MacBook Air (M2, 8-core, 16 GB), macOS 27.0.1.
- Toolchain: Rust 1.99.0, aarch64-apple-darwin.
- Command: `cargo run -p cascade-app -- --smoke` (development profile).
- World: 1024 x 1024, mixed-overload fixture, 20,000 scheduler credits, default queue capacities.
- The smoke prepared the fixture under each policy, replayed 120 held-disturbance frames per policy (64 capped attempts per presented frame), checked a GPU readback, exercised pan/zoom/resize, and waited for upload settlement.

Latest completed paired run:

| Metric | Bounded | Traditional |
| --- | ---: | ---: |
| Measured frames | 120 | 120 |
| Pending channels, first to last | 525,394 -> 452,287 | 525,394 -> 492,723 |
| Maximum pending channels | 525,394 | 525,394 |
| Maximum ready occupancy | 32,769 | 32,769 |
| Maximum simulation CPU time per presented slice | 0.39 ms | 6.45 ms |
| Maximum upload backlog during measured frames | 0 chunks | 0 chunks |
| Frame interval p99 | 19.64 ms | 18.36 ms |
| Maximum frame interval | 23.48 ms | 18.87 ms |

The smoke passed its functional checks: 2,172 presents, pan and zoom input, GPU readback, stale-texture observation during fixture preparation, resize handling, zero-size handling, and three surface reconfigurations. It reported 20,174 uploaded chunks over the full run.

This is one short development-profile run, not sustained performance acceptance. The measured traditional slice used substantially more simulation CPU time, but that did not produce a consistently slower frame interval in this run: bounded p99 was higher, and one bounded frame reached 23.48 ms. Other paired smokes varied, so the native frame-time comparison does not yet establish a visible traditional stall or a 2x improvement. Pending work remained high in both runs and declined over the measured windows; the smoke does not claim that pending count grew during those exact windows. Upload backlog was zero during the measured intervals. Repeat in release mode over sustained runs, and tune/retake the comparison before making a stronger performance claim.

## World-size bounds

The app defaults to 1024 x 1024. `--world-size 4096` selects the charter-size square world at startup; dimensions are validated before allocation and remain fixed for that run. At 4096 x 4096, the normal and tiny-capacity profiles together account for about 161.5 MiB of simulation-owned CPU arrays, and the material texture is 16 MiB. This fits the current per-app CPU storage limit, but startup allocation and full-size fixture-preparation latency were not included in the M2 smoke above.

## Remaining measurements

- Run each fixture at least 60 seconds, three times, with explicit warm-up exclusion and alternating mode order, at fixed 1920 x 1080 interactive render settings. The headless suite and the short native smoke do not satisfy the duration or rendering conditions.
- Calibrate with multiple expensive, saturated workloads and headroom on the reference world size, while retaining distinct held-out validation inputs.
- Report per-run preparation duration separately and completion/stable-state slices and wall time. The runner reports 290 preparation scheduler slices for the 256 x 256 runs, but does not time preparation separately.
- A finite burst/recovery probe used 1,200 disturbances followed by 1,650 no-new-input slices (1,800 total) across three repetitions. Both modes accepted all 1,200 with no rejection/coalescing. Bounded backlog high-water/end were 32,754/195; traditional were 12,260/194. Neither became empty/stable in the window. The probe order was not alternated, so it is supporting, not paired comparative evidence.
- Measure interactive frame interval p50/p95/p99/max and intervals over 33.3 ms over sustained release-profile windows, plus camera/UI feedback, uploads, and GPU timing. The native smoke is one short development-profile sample and does not establish a consistent traditional frame stall.
- Verify full-size interactive GPU and workload behavior at 4,096 x 4,096. The app test allocates both CPU-side profiles under the storage limit, but no full-size render or preparation run is claimed.

These gaps are reported rather than inferred from the headless slice measurements.
