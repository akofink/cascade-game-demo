# Performance evidence

## Current headless profile and status

See [full-size headless results v2](../benchmarks/results/headless-v2.md) and [reference profile v2](../profiles/m2-16gb-v2.toml). The selected headless credit allowance is 1,000,000 credits/slice. It was calibrated upward on full-size saturated workloads: p99 was 2.590917 ms on burning forest and 1.113125 ms on mixed overload, with 120 warm-up slices excluded and a fresh fixture prepared before measurement. Across those two calibration sets, one of 10,800 samples exceeded 4 ms; the maximum was 4.107667 ms. The profile is an empirical candidate with p99 headroom, not a hard deadline guarantee.

The simulator supports up to 10,000,000 credits, but the app still caps credits at 100,000 and defaults to 20,000. The app cannot use this profile until its control cap/default is updated. This work did not modify `crates/app`.

All eight section-12 descriptors were measured headlessly at 4096 x 4096, 1,000,000 credits, three paired runs per policy, alternating order. The bounded burning-forest p99 was 2.883167 ms, while traditional was 2.862917 ms; only 2 of 900 traditional samples exceeded 4 ms. Thus the charter's twofold p99 comparison and clearly overloaded traditional baseline are **inconclusive/unmet**, despite isolated traditional peaks. The full-size finite burst did not reach stable state within 1,800 measured slices.

## Headless method and machine disclosure

Release build: `cargo build --release -p cascade-bench`. The runner times preparation separately, supports `--warmup-slices`, re-prepares the same fixture after warm-up, then records measured slice times, per-slice and cumulative scheduler quanta, cumulative wall elapsed, run-level preparation/warm-up time, measured wall time and first completion slice/time. Preparation wall duration includes both fixture passes when warm-up is enabled; world allocation is excluded. Warm-up wall duration covers only warm-up scheduler steps. Slice duration is monotonic wall time around `World::step`, including OS preemption, not thread CPU time.

Reference hardware: MacBook Air (Mac14,2), Apple M2 with 8 CPU cores, 16 GB RAM, integrated Apple M2 GPU; macOS 27.0.1; built-in 2560 x 1664 display. Rust 1.99.0 (`b940084d7`, aarch64-apple-darwin), Cargo 1.99.0. Headless source revision: `5825d77eae4c9b866dd8251c47fcc3e93403a9db`; `Cargo.lock` SHA-256 `c4084ec2080b2c4da29d605573b159b02504f86af481d13c5807e565d419afe7`. Power mode, thermal state, background load, and display refresh/presentation mode were not controlled.

Calibration used full-size burning-forest and mixed-overload descriptors, each with a deterministic 64-command-per-slice disturbance stream, 120 warm-up slices, fresh re-preparation, and three repetitions of 1,800 measured slices at 1,000,000 credits. The selected point admitted 115,200 commands per run with no rejection/coalescing; backlog high-water was 12,672,372 for burning forest and 8,374,984 for mixed overload. The upward sweeps, p50/p95/p99/max, completed work, and tuning points are in the v2 summary.

The held-out matrix uses the eight section-12 descriptors with no external disturbance stream. Calibration and validation therefore use distinct command-stream inputs, but the static `burning-forest-v1` and `mixed-overload-v1` descriptors appear in both sets. Descriptor-level separation is incomplete and disclosed; the report does not claim a fully independent fixture holdout.

## Native M2 smoke evidence

The app-side integration on current `main` ran `cargo run -p cascade-app -- --smoke` in development mode on the same M2. This short paired run used a 1024 x 1024 mixed-overload world, 20,000 credits and 120 held-disturbance frames per policy, with 64 capped attempts per presented frame. It checked GPU readback, pan/zoom/resize, and upload settlement. The run reported:

| Metric | Bounded | Traditional |
|---|---:|---:|
| Measured frames | 120 | 120 |
| Pending channels, first to last | 525,394 -> 452,287 | 525,394 -> 492,723 |
| Maximum pending channels | 525,394 | 525,394 |
| Maximum ready occupancy | 32,769 | 32,769 |
| Maximum simulation CPU time per presented slice | 0.39 ms | 6.45 ms |
| Maximum upload backlog during measured frames | 0 chunks | 0 chunks |
| Frame interval p99 | 19.64 ms | 18.36 ms |
| Maximum frame interval | 23.48 ms | 18.87 ms |

The functional smoke passed and reported 2,172 presents, 20,174 total uploaded chunks, stale-texture observation during preparation, and three surface reconfigurations. This is one short development-profile run, not sustained acceptance. Traditional used substantially more simulation CPU on its slowest slice but did not produce a slower p99 frame interval; bounded p99 was higher, and its maximum frame interval was 23.48 ms. Other short paired smokes varied, so these measurements do not establish a visible traditional stall or a twofold frame-latency improvement. Pending work declined during the measured windows; no claim that it grew in those exact windows is made. The smoke does not validate the 1,000,000-credit headless profile.

## World-size bounds

The app defaults to a 1024 x 1024 world. `--world-size 4096` selects the charter-size square world at startup; dimensions are validated before allocation and remain fixed for the run. At 4096 x 4096, the normal and tiny-capacity simulation profiles together account for about 161.5 MiB of simulation-owned CPU arrays, and the material texture is 16 MiB. This fits the current per-app CPU storage limit, but startup allocation and full-size fixture-preparation latency are separate from steady-state measurements.

## Remaining unmet targets

- Headless paired fixture comparisons are 300 slices/run, not at least 60 seconds per fixture. Selected calibration repetitions were 1,800 slices and approximately 1.47 to 2.17 seconds measured wall time, also below 60 seconds.
- The traditional p99 is not clearly overloaded in the paired full-size held-out matrix, and the paired slice-p99 reduction is not twofold. Comparison is inconclusive.
- A 1,200-command full-size finite burst followed by no new input was measured for 1,800 slices; neither policy reached an empty/stable state. Completion time is right-censored beyond the capture.
- The native smoke is one short development-profile run, not three sustained 60-second release captures at fixed render settings. Frame interval, >33.3 ms, camera/UI feedback, upload, and GPU distributions still need sustained paired capture.
- App maximum/default remain 100,000/20,000 credits. The selected 1,000,000-credit profile needs an integration-owned app-control update before interactive use.
- Calibration command streams differ from held-out inputs, but static fixture descriptors overlap as stated above.
- Full-size app rendering and preparation with the calibrated profile were not measured.

These results are empirical on one machine, not portable guarantees. Credits bound accounted algorithmic work, not elapsed-time deadlines.
