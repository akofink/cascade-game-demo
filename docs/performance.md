# Performance evidence

## Current headless profile and status

See [corrected full-size headless results v3](../benchmarks/results/headless-v3.md) and [reference profile v3](../profiles/m2-16gb-v3.toml). The 1,000,000-credit/slice setting remains an empirical headless candidate. After the traditional-frontier correction and its additional preallocated snapshot, full-size saturated calibration observed p99 2.573875 ms on burning forest and 1.092084 ms on mixed overload; neither had a sample over 4 ms among 10,800 total samples. A separate burning-forest sweep includes adverse >4 ms outliers at 800,000 and 1,200,000 credits. These finite samples on one machine are not deadline guarantees.

The v2 report is superseded for the traditional-policy comparison: its traditional scheduler serviced ready-ring entries and only a single recovery probe instead of capturing the entire per-cell pending frontier. The v2 numbers remain in [the archived v2 report](../benchmarks/results/headless-v2.md) for traceability and must not be treated as results for the corrected policy. The initial v1 pilot is also superseded.

The simulator supports up to 10,000,000 credits. The app now reads its default allowance and UI cap from `profiles/m2-16gb-v3.toml`, currently 1,000,000 credits. A full-size release-mode native smoke against the corrected traditional frontier is recorded below; this short run is not sustained interactive acceptance.

## Traditional frontier correction

Traditional mode now treats per-cell pending state as authoritative, independent of ready-ring capacity. At each measured slice it performs two full scans of the per-cell storage: one to snapshot and clear pending evaluation/blast channels and ready rings, and one to process the captured one-byte-per-cell snapshot. It processes only the command count captured at slice entry. Every scan cell and every captured dispatch is charged and included in slice wall time; newly generated work waits for the next slice. At 4096 x 4096, the two scans alone account for 33,554,432 recovery quanta and 167,772,160 credits per slice, even when no cells are pending. This is the required batch-all baseline, not a general claim about conventional engines. The measured burning-forest p99 was 226.496334 ms traditional versus 2.978875 ms bounded, but much of the traditional interval is the mandatory full-world scan. See the v3 results for all fixtures, exact work, and the 1,200-command burst/recovery outcome.

Ready capacity remains 32,768 per lane (65,536 combined), with 256 command slots. Enlarging the rings would increase queued-storage allowance but would not increase bounded-mode service under its credit budget or reduce the traditional full-storage scan; the corrected traditional policy no longer treats ring occupancy as a frontier limit. The `tiny-capacity` descriptor separately tests two ready entries per lane.

Fixture disclosures: `explosive-lattice` has a seeded center blast (energy 15) pending at fresh measurement start; warm-up is followed by fixture re-preparation. `dirty-world-sweep` prepares a static stone/air pattern and has no active headless simulation channels. Its renderer dirty-chunk drain/upload is app-side and absent from the headless runner, although traditional's two full-world scans are still measured for it.

## Headless method and machine

Release build: `cargo build --release -p cascade-bench`. The runner reports preparation and warm-up durations separately, re-prepares the descriptor after optional warm-up, and records measured wall time around each CPU-only `World::step`. The timer is monotonic wall time, not thread CPU time, and includes operating-system preemption. See the v3 report for the exact calibration, paired-matrix and burst protocols, sample counts, percentiles, backlogs, work, and completion semantics. Calibration and validation use distinct disturbance streams but share the static burning-forest and mixed-overload descriptors; fixture-level holdout independence is incomplete.

Reference machine: MacBook Air (Mac14,2), Apple M2 with 8 CPU cores, 16 GB RAM and integrated M2 GPU; macOS 27.0.1; built-in 2560 x 1664 display. Rust 1.99.0 (`b940084d7`, aarch64-apple-darwin), Cargo 1.99.0 (`5f94df478`). `Cargo.lock` SHA-256: `c4084ec2080b2c4da29d605573b159b02504f86af481d13c5807e565d419afe7`. Power mode, thermal state, background load, and display refresh/presentation mode were not controlled.

## Historical native smoke, before the frontier correction

The following M2 development-profile smoke was run by the app integration on main before the corrected traditional frontier. It is preserved as historical native-renderer evidence only and does not measure the traditional behavior described above.

The run used `cargo run -p cascade-app -- --smoke`, a 1024 x 1024 mixed-overload world, 20,000 credits, 120 held-disturbance frames per policy, and 64 capped attempts per presented frame. It checked GPU readback, pan/zoom/resize, and upload settlement.

| Metric | Bounded | Traditional (pre-correction) |
|---|---:|---:|
| Measured frames | 120 | 120 |
| Pending channels, first to last | 525,394 -> 452,287 | 525,394 -> 492,723 |
| Maximum pending channels | 525,394 | 525,394 |
| Maximum ready occupancy | 32,769 | 32,769 |
| Maximum simulation CPU time per presented slice | 0.39 ms | 6.45 ms |
| Maximum upload backlog during measured frames | 0 chunks | 0 chunks |
| Frame interval p99 | 19.64 ms | 18.36 ms |
| Maximum frame interval | 23.48 ms | 18.87 ms |

Functional smoke passed with 2,172 presents, 20,174 total uploaded chunks, stale-texture observation during preparation, and three surface reconfigurations. It was one short development-profile run, not sustained acceptance; it did not validate the 1,000,000-credit profile. These old frame statistics must not be used to infer post-correction interactive performance.

## Full-size release smoke after the traditional-frontier correction

Command: `cargo run --release -p cascade-app -- --smoke --world-size 4096`, using the v3 profile-backed allowance of 1,000,000 credits on the MacBook Air M2 / 16 GB. The same mixed-overload fixture and 64-attempt-per-present disturbance stream ran for 120 measured frames per policy. `SMOKE_RESULT ok`; 888 presents; GPU readback, pan, zoom, resize, zero-size surface handling, and reconfiguration passed. The smoke uploaded 88,062 chunks and observed stale texture during preparation.

| Metric | Bounded | Traditional (corrected full frontier) |
|---|---:|---:|
| Measured frames | 120 | 120 |
| Pending channels, first to last | 8,406,053 -> 4,745,728 | 8,406,053 -> 26,028 |
| Maximum pending channels | 8,406,053 | 8,406,053 |
| Maximum ready occupancy | 32,769 | 32,845 |
| Maximum simulation CPU time per presented slice | 2.18 ms | 86.38 ms |
| Maximum upload backlog during measured frames | 0 chunks | 0 chunks |
| Frame interval p99 | 18.77 ms | 79.89 ms |
| Maximum frame interval | 19.39 ms | 80.89 ms |

This run clearly shows a traditional stall in this workload: traditional p99 frame interval was 79.89 ms versus 18.77 ms bounded, while the full-frontier traditional slices reached 86.38 ms simulation CPU. Traditional drained most pending channels during the short window; bounded pending work also declined. This is one 120-frame-per-policy release smoke, not sustained acceptance, and does not establish general performance across fixtures or render loads. The 4.26x p99 ratio is an observed result for this run, not a portable guarantee.

## World-size resource bounds

At 4096 x 4096 with default ready capacities, one `World` accounts for 101,468,160 bytes (96.77 MiB) of simulation-owned arrays, including its one-byte-per-cell traditional snapshot. The app holds a normal-capacity world and a tiny-capacity alternate world; together their simulation arrays account for 202,147,904 bytes (192.78 MiB). The CPU material grid adds 16 MiB; the material texture is a separate 16 MiB GPU resource. Allocator metadata and process RSS are excluded. Startup allocation and full-size fixture-preparation latency are separate from steady-state measurements.

## Remaining unmet targets

- Paired headless fixtures are 300 measured slices per run, not 60 seconds per fixture. Calibration and burst/recovery runs are 1,800 slices and also fall short of 60 seconds.
- The defined traditional full-scan policy is clearly over 4 ms at full size, but its idle two-scan cost dominates many slices. The measured p99 comparison is policy-specific and is not evidence about optimized conventional engines.
- The 1,200-command full-size burst did not resolve or reach an empty/stable state for either policy within 1,800 slices; completion time is right-censored.
- Native evidence includes one 120-frame-per-policy full-size release smoke against the corrected traditional policy, not three sustained 60-second captures. Longer frame-interval distributions, camera/UI responsiveness under stalls, uploads, and GPU timings remain outstanding.
- The app maximum/default now read the selected 1,000,000-credit allowance from profile v3.
- Static descriptors overlap between calibration and validation; only their disturbance command streams are distinct.
- Full-size startup, fixture preparation, and rendering completed in this smoke; sustained full-size interaction remains unmeasured.

Results are empirical on one machine. Bounded-mode credits constrain accounted algorithmic work, not operating-system or hardware latency.
