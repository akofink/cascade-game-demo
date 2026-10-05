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

## Player-action focus latency

The repeatable three-policy runner is `cargo +1.99.0 run --release -p cascade-bench -- --player-action-comparison --width 4096 --height 4096 --budget 1000000 --slices 4096`. It re-prepares `mixed-overload-v1` independently for traditional, bounded FIFO, and bounded focus; admits one seeded background disturbance every four slices and scripted paint/ignite/detonate player actions every eight slices. It reports action-to-first-rule-effect and local-settle distributions in slices and wall time, background evaluation/blast throughput, oldest pending age, and final backlog. The action local-settle sample is censored when the neighborhood has not settled by the run end.

The full-size three-policy runner and three accepted quiet-host repetitions are documented in [player-focus results v1](../benchmarks/results/player-focus-v1.md). Its scripted stream pairs safe upper-half paint commands with an immediate follow-up ignite on the newly painted wood or explosive, and selects detonation targets from fixture explosives. Each full run has 4,096 slices per policy and at least 30 first-effect and local-settle samples per action type. `SliceMetrics.action_effect_candidates` exposes action-record probes, and a per-chunk fixed bitset keeps unrelated traditional work from scanning pending action history. Across the three accepted runs, bounded-focus action-to-first-effect p95 was 2.754 ms aggregate, with paint/ignite/detonate p95 of 2.777/2.709/2.755 ms; all three types had 170 or 171 effect and settle samples. Traditional headless action-to-first-effect p95 was 133.372 ms. These are simulator action latencies, not native frame intervals. The post-fix native smoke did not produce a valid `SMOKE_RESULT`, so no native frame p99 is claimed. The short 512-slice capture remains exploratory, not the charter's three 60-second acceptance repetitions; camera/UI acknowledgment and native presentation latency remain outside this headless measurement.

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

A follow-up screenshot smoke ran the same command with `--screenshot docs/cascade-overload.png`. Its separate 120-frame windows measured bounded p99/max 19.62/20.13 ms with max simulation CPU 4.11 ms, and traditional p99/max 79.64/80.39 ms with max simulation CPU 88.98 ms; pending channels again moved from 8,406,053 to 4,745,728 bounded and 26,028 traditional. The 2560 x 1440 PNG is 283,291 bytes. GPU readback and PNG encoding happen after both measured windows, so the capture does not contribute to those frame summaries. This remains a second short smoke, not sustained performance evidence.

## World-size resource bounds

At 4096 x 4096 with default ready capacities, one `World` accounts for 102,800,896 bytes (98.04 MiB) of simulation-owned arrays, including focus rings, the action-record ring, its per-chunk action-effect index, and its one-byte-per-cell traditional snapshot. The app holds a normal-capacity world and a tiny-capacity alternate world; each has a separately bounded allocation. The CPU material grid adds 16 MiB; the material texture is a separate 16 MiB GPU resource. Allocator metadata and process RSS are excluded. Startup allocation and full-size fixture-preparation latency are separate from steady-state measurements.

## Game-feel measurement

Camera/UI feedback latency and action-to-first-visible-effect latency are CPU timestamps, not GPU timestamp queries and not a display-scanout measurement.

- The clock starts when winit delivers the event that changes the camera, overlay, or admitted player action.
- It stops at the end of the presented frame whose camera uniform, acknowledgment, or uploaded chunk first includes that change. The sample is taken in that frame's post-present bookkeeping, before the next input pump. Display scanout and composition are excluded. The one-time smoke screenshot readback happens after both measured windows, so it is not part of those latency samples.
- Percentiles are nearest-rank p50/p95 over a fixed 240-sample ring. The overlay shows the rings. Native smoke prints `SMOKE_FEEL` for each measured policy window.
- A paint effect is visible when the uploaded texture contains the requested material. Ignition of wood is visible when the upload encodes burning wood as palette index 7. Detonation is visible when the uploaded material or burn state differs from the pre-action cell. A chunk upload that happens before the authoritative change does not count. Rejected or no-effect clicks are acknowledged and are not latency samples.
- The acknowledgment ring is screen-space and is not a material. Pending rings stay up until the upload shows the effect, the command is rejected, or the click had no target.
- Smoke admits a scripted paint, ignite, and detonate stream plus a 1 px camera nudge every 10 measured frames while DESTROY PERFORMANCE is admitting disturbances. That stream is part of the measured windows, not a separate quiet baseline.
- Player focus is linked when the simulator exports `set_focus_enabled`, `Command::FocusViewport`, action commands, and `active_focus_regions`. The overlay draws those simulator regions. FIFO restarts with focus disabled. The smoke runs a third bounded-focus window when that API is present.

## Release smoke, 2026-10-05, after game-feel changes

Command: `cargo run --release -p cascade-app -- --smoke --world-size 4096 --screenshot docs/cascade-overload.png` on the MacBook Air M2 / 16 GB. `SMOKE_RESULT ok`. 888 presents. The measured windows are 120 frames per policy with DESTROY PERFORMANCE admitting disturbances and a scripted paint/ignite/detonate action plus a 1 px camera nudge every 10 frames. Player marks are re-applied after disturbance admission and their chunks are uploaded without waiting for the world dirty cursor. Focus is unlinked.

| Metric | Bounded FIFO | Traditional |
|---|---:|---:|
| Frame interval p99 / max | 17.61 / 17.78 ms | 85.82 / 86.75 ms |
| Max simulation CPU | 1.52 ms | 87.21 ms |
| Pending channels, first to last | 8,406,053 -> 4,746,886 | 8,406,053 -> 26,060 |
| Camera/UI p50 / p95 (n=11) | 16.95 / 17.41 ms | 33.20 / 86.74 ms |
| Paint visible p50 / p95 (n=3) | 17.23 / 17.41 ms | 32.86 / 33.03 ms |
| Ignite visible p50 / p95 | 16.82 / 513.59 ms (n=4) | 86.74 / 86.74 ms (n=1) |
| Detonate visible p50 / p95 (n=4) | 16.73 / 17.00 ms | 33.42 / 59.98 ms |

Bounded frame p99 is 17.61 ms, down from the prior full-size smoke's about 18.8 to 19.6 ms, and still a little over the 16.7 ms presentation target. One ignite sample in the bounded window was about 514 ms; with n=4 that sample is the p95. Traditional frame p99 was 85.82 ms. These are one short run, not three sustained 60-second captures. The bounded-focus column is not in the run above. A later local smoke that includes the simulator focus branch is below. It is not on `main` until that simulator change merges.

## Three-policy smoke with focus linked

Command: `cargo run --release -p cascade-app -- --smoke --world-size 4096 --screenshot docs/cascade-overload.png` on 2026-10-05 about 17:27 EDT. One-minute load average at start was 2.74; at the end it was 3.35. `SMOKE_RESULT ok`. 1310 presents. Each policy window is 120 frames with DESTROY PERFORMANCE admitting disturbances and a scripted paint, ignite, and detonate plus a 1 px camera nudge every 10 frames.

| Metric | Bounded FIFO | Bounded focus | Traditional |
|---|---:|---:|---:|
| Frame interval p99 / max | 17.17 / 32.58 ms | not in the legacy summary; camera p95 17.73 ms | 820.68 / 837.81 ms |
| Max simulation CPU | 12.25 ms | not in the legacy summary | 1947.87 ms |
| Pending channels, first to last | 8,406,053 -> 6,144,742 | not separately summarized | 8,406,053 -> 26,060 |
| Camera/UI p50 / p95 (n=11) | 16.59 / 16.99 ms | 16.95 / 17.73 ms | 67.09 / 753.84 ms |
| Paint visible p50 / p95 (n=3) | 16.62 / 16.99 ms | 16.95 / 17.09 ms | 80.84 / 83.90 ms |
| Ignite visible p50 / p95 | 815.65 ms (n=1) | 16.20 ms (n=1) | 753.84 ms (n=1) |
| Detonate visible p50 / p95 | 16.68 / 1017.21 ms (n=4) | no samples | 67.09 / 372.59 ms (n=4) |

The frame-interval summary is per presented policy window. Bounded focus is the third window; its camera and paint latencies are about one frame. Its detonate actions were admitted, but none produced a visible-effect sample in that window. Traditional simulation CPU reached 1947.87 ms, about 20 times the 87 ms maximum in the pre-focus full-size smoke above, while the one-minute load average was under 3 at the start. That traditional column is reported as adverse. It is not explained by the load gate alone and is not a claim that the focus scheduler made traditional mode that slow. The earlier 85.82 ms traditional p99 remains the quiet pre-focus comparison.

## Remaining unmet targets

- Paired headless fixtures are 300 measured slices per run, not 60 seconds per fixture. Calibration and burst/recovery runs are 1,800 slices and also fall short of 60 seconds.
- The defined traditional full-scan policy is clearly over 4 ms at full size, but its idle two-scan cost dominates many slices. The measured p99 comparison is policy-specific and is not evidence about optimized conventional engines.
- The 1,200-command full-size burst did not resolve or reach an empty/stable state for either policy within 1,800 slices; completion time is right-censored.
- Native evidence includes one 120-frame-per-policy full-size release smoke against the corrected traditional policy, not three sustained 60-second captures. Longer frame-interval distributions, camera/UI responsiveness under stalls, uploads, and GPU timings remain outstanding.
- The app maximum/default read the selected 1,000,000-credit allowance from profile v3. Its focus service-share values are initial scheduler parameters and have not been calibrated.
- Static descriptors overlap between calibration and validation; only their disturbance command streams are distinct.
- Full-size startup, fixture preparation, and rendering completed in this smoke; sustained full-size interaction remains unmeasured.

Results are empirical on one machine. Bounded-mode credits constrain accounted algorithmic work, not operating-system or hardware latency.
