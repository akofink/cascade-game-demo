# Performance evidence

## Default native timing workflow

For unattended native timing captures, build release once before checking the quiet-machine gate, then use the offscreen GPU-texture path (one fresh process per policy):

```sh
cargo build --release -p cascade-app
caffeinate -dimsu target/release/cascade-app --offscreen-capture --capture-policy bounded-focus --capture-seconds 60 --world-size 4096
```

Use `bounded-fifo` and `traditional` for the other policy runs. The offscreen path renders the same 1920 x 1080 grid and overlay passes (with a fixed 2x logical scale matching the reference Mac's 960 x 540 window), advances one normal simulation slice per iteration, and observes the same capped chunk uploads. It paces from a monotonic clock at a fixed 60 Hz target and does not create a window or surface. It never reads the render target back during a capture. Its frame intervals include CPU work and cadence waiting, but **do not measure display presentation, vsync, compositor scheduling, or scanout**. Scripted game-feel values are CPU-side action/upload/render-submission proxies, not visible-action latency.

Use windowed `--capture-policy POLICY --capture-seconds 60` captures when making presentation-level claims. Windowed results remain separate evidence and are not replaced by offscreen values. Paired offscreen and windowed results, including the limits of comparison, are in [native offscreen comparison v1](../benchmarks/results/native-offscreen-v1.md).

Before and after each capture, check the one-minute system load; accept results only when it is below 3 at both checks. Run one capture process at a time, reject and retain a note of attempts that miss the gate, and do not suppress long samples from accepted runs.

## Current headless profile and status

See [corrected full-size headless results v4](../benchmarks/results/headless-v4.md), [archived headless results v3](../benchmarks/results/headless-v3.md), and [reference profile v3](../profiles/m2-16gb-v3.toml). The 1,000,000-credit/slice setting remains an empirical candidate. The v4 follow-up profiles a same-chunk focus-membership cache on 4096² burning-forest and mixed-overload calibration. Pooled p99 improved from 3.906750 to 2.838709 ms on burning forest and 2.077292 to 1.652375 ms on mixed overload, with no reduction in quanta/slice; however, burning-forest still had 8 samples above 4 ms and a 9.767333 ms maximum among 5,400 slices. The 1M profile was retained, not reduced. These finite wall-clock samples include OS preemption and are not deadline guarantees.

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

At 4096 x 4096 with default ready capacities, one `World` accounts for 103,063,040 bytes (98.29 MiB) of simulation-owned arrays, including focus rings, the 262,144-byte per-chunk focus membership cache, the action-record ring, its per-chunk action-effect index, and its one-byte-per-cell traditional snapshot. The app holds a normal-capacity world and a tiny-capacity alternate world; each has a separately bounded allocation. The CPU material grid adds 16 MiB; the material texture is a separate 16 MiB GPU resource. Allocator metadata and process RSS are excluded. Startup allocation and full-size fixture-preparation latency are separate from steady-state measurements.

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

## Native three-policy smoke after action-effect indexing

The native smoke ran twice in release mode on main revision `3016f6f` with `--smoke --world-size 4096`; both completed with `SMOKE_RESULT ok`. The reported 1-minute load moved from 2.4 to 2.7. The detailed second run was 120 frames per policy:

| Metric | Bounded FIFO | Bounded focus | Traditional |
| --- | ---: | ---: | ---: |
| Frame interval p99 / max | 17.98 / 17.99 ms | not reported | 136.68 / 136.85 ms |
| Maximum simulation CPU | 4.08 ms | not reported | 146.25 ms |
| Camera p50 / p95 | 16.61 / 17.74 ms | 16.42 / 17.10 ms | 67.25 / 136.67 ms |
| Paint visible p50 / p95 | 16.30 / 16.75 ms | 16.09 / 16.51 ms | 66.62 / 66.63 ms |
| Ignite visible | 815.82 ms (n=1) | 17.04 ms (n=1) | 136.67 ms (n=1) |
| Detonate visible p50 / p95 | 17.75 / 1019.28 ms (n=4) | no samples | 67.15 / 92.66 ms (n=4) |

Traditional frame p99 of 136.68 ms is substantially below the previous 820.68 ms regression, but remains about 1.6x the pre-focus 80-86 ms range. This is one detailed native run; p50/p95 values with n=1 or n=4 are descriptive, not stable percentile estimates. The bounded-focus detonate stream still has no visible-effect sample. This is an app smoke scripting gap tracked in [issue #26](https://github.com/akofink/cascade-game-demo/issues/26); the simulator follow-up did not edit app code.

## Traditional action-tracking overhead and sustained native results

Traditional mode continues to bypass bounded focus-lane selection and perform the charter's full capture and execution scans. The native app does not need simulator action-latency records: it measures visible-effect latency from admitted UI actions through texture upload. `Demo` therefore disables the optional fixed action-record ring in both native worlds. This avoids scanning up to 256 action records after each traditional slice, probing the action-effect chunk index for every captured work item, and polling the action ring for local settlement. Headless benchmark worlds retain action tracking by default. A sim regression test verifies that disabled tracking creates no records or action-effect candidates, does not add traditional frontier work, and that traditional still accounts exactly two storage scans. Traditional cleanup only clears focus-queued flags when they are present, preserving stale-state cleanup without unconditional focus bookkeeping across the world.

A 4096 x 4096 release native smoke with at least 30 visible paint, ignite, and detonate samples per policy completed with `SMOKE_RESULT ok`. The measured 120-frame smoke reported bounded focus max simulation CPU 3.65 ms and traditional 111.47 ms; its one-minute load average was 2.62 before and 2.50 after. The earlier 146 ms smoke used action tracking and is not the post-fix comparison. The 111 ms maximum is not a universal cost bound; traditional intentionally captures every pending cell/channel and dispatches captured work without the slice credit cap.

The original three 60-second release captures per policy at a verified 1920 x 1080 surface are preserved as the pre-cache baseline in [native sustained capture v1](../benchmarks/results/native-sustained-v1.md). The post-cache three-per-policy rerun is published in [native sustained capture v2](../benchmarks/results/native-sustained-v2.md), including rejected attempts and exact load gates. The bounded-focus visible-action p95s were below 18 ms; bounded FIFO below 19 ms. Traditional paint/ignite p95 was about 67 ms and detonate p95 about 134 ms.

## Bounded slice CPU profile and focus-membership optimization

The pre-cache native attribution capture found bounded-focus worst-slice counts of 33,337 evaluation quanta, 33,300 recovery quanta, and 66,652 selection probes at 5.963 ms; bounded FIFO's 5.140 ms maximum had 35,291 evaluations, 23,526 recoveries, and 58,828 selections. The 1M allowance is exhausted primarily by evaluation work (24 credits/evaluation) plus per-dispatch selection. Recovery previously repeated the same up-to-eight-region focus-membership search for both pending channels and could repeat it again while enqueueing each channel. The post-cache captures continue to show evaluation-heavy maxima. Upload CPU peaks are separately measured and had zero backlog/no corresponding >33.3 ms bounded intervals; they do not explain the long simulation slices. The 34.342 ms prior frame outlier did not recur in 21,600 post-cache bounded intervals.

The simulator now memoizes focus membership by logical chunk for the current slice/region epoch, reuses one result for both channels in recovery, and invalidates on focus changes. The cache is fixed at one 16-byte entry per chunk and adds 262,144 bytes at 4096². `cascade-bench` now exports per-slice evaluation, blast, recovery, and command counts; native captures export work counts at the maximum simulation slice and timings at the slowest >33.3 ms presentation frame. See the full headless calibration and adverse samples in [v4](../benchmarks/results/headless-v4.md).

The post-cache valid native suite reports focus simulation maxima 4.876–5.062 ms and FIFO 5.113–5.600 ms. Focus frame p99 improved to 17.654–17.709 ms and met 20 ms; FIFO p99 improved to 20.506–20.621 ms but remains above 20 ms. There were no bounded intervals above 33.3 ms among 21,600 samples, so the earlier 34.342 ms focus outlier did not recur. Max-slice counts remain evaluation-heavy: focus had about 33–34k evaluation quanta and 45k selection probes; FIFO about 35k evaluations and 59k selections. The cache does not affect FIFO mode. Credit allowance remains 1M; no scheduler work-throughput reduction was made. Exact run chronology, gated-out attempts, upload/action timings, and method deviations are in v2.

## Remaining unmet targets

- The post-cache native full-size bounded simulation CPU maxima remain 4.876–5.600 ms, above the 4 ms target. Headless v4 burning-forest p99 is below 4 ms, but its worst sample is 9.767 ms and 8/5,400 samples exceed 4 ms. Do not claim the CPU timing target is closed.
- Post-cache bounded FIFO frame p99 remains slightly above 20 ms (20.506–20.621 ms); post-cache bounded focus met the 20 ms target. The prior 34.342 ms focus interval did not recur, but remains historical evidence.
- The traditional full-frontier policy had every measured frame interval over 33.3 ms and is far beyond the 4 ms simulation-slice target by design. This is the charter's intentionally expensive batch-all baseline, not evidence about all conventional engines.
- GPU timing was unavailable; native visible-action latency excludes display scanout/composition. Captures cover one M2 Mac and one mixed-overload fixture, not all fixtures or devices.
- The 1,200-command full-size headless burst did not resolve or reach an empty/stable state for either policy within 1,800 slices; completion time is right-censored.
- The app maximum/default reads the selected 1,000,000-credit allowance from profile v3. Focus service-share values are initial scheduler parameters and have not been calibrated.
- Static descriptors overlap between headless calibration and validation; only disturbance command streams are distinct.

Results are empirical on one machine. Bounded-mode credits constrain accounted algorithmic work, not operating-system or hardware latency.
