# Performance evidence

## Charter section 15 acceptance checklist

Statuses describe the evidence currently published here; “not measured” means no qualifying measurement is available, while “unmet” means an available result misses the criterion.

| Section 15 criterion | Status | Evidence |
| --- | --- | --- |
| Every bounded slice stays within its credit allowance, including saturation and stale jobs | met | [Headless calibration v4](../benchmarks/results/headless-v4.md); scheduler adversarial tests in `crates/sim/src/lib.rs` |
| Every interactive engine-owned loop has a fixed limit or charged resumable cursor | met | [Architecture work/resource contracts](architecture.md); [native sustained capture v2](../benchmarks/results/native-sustained-v2.md) |
| Queue, pool, upload, and resident storage limits hold under stress fixtures | met | [Resource accounting below](#world-size-resource-bounds); [headless v4](../benchmarks/results/headless-v4.md); [native sustained capture v2](../benchmarks/results/native-sustained-v2.md) |
| Accepted deferred work is retained or explicitly canceled; repeat requests do not grow unbounded history | met | [Architecture and overload semantics](architecture.md); queue saturation and recovery tests in `crates/sim/src/lib.rs` |
| Finite pending demand receives fair service; stale generations cannot mutate reset worlds | met | Scheduler fairness, reset, and stale-generation tests in `crates/sim/src/lib.rs` |
| Sand/water conservation and blast/fire conversion rules pass tests | met | [Rules and conservation tests](rules.md) |
| Replays reproduce full future-affecting state hashes | met | Replay/hash tests in `crates/sim/src/lib.rs`; [headless results](../benchmarks/results/headless-v4.md) |
| No simulation/scheduler hot-path heap growth after initialization, verified by allocation instrumentation | met | [Allocation and GPU diagnostics](#allocation-and-gpu-diagnostics): saturated-slice global-allocator test observed zero allocation/reallocation calls |
| Each fixture: three release captures of at least 60 seconds after warm-up, 1920 x 1080, 60 Hz, including quiet baseline | windowed matrix complete; criterion results vary | [Native all-fixture windowed acceptance v1](../benchmarks/results/native-acceptance-v1.md) has 72 accepted captures across all eight fixtures, three policies, and three repetitions, plus 67 retained load-gate rejections. This provides windowed event-loop interval and slice telemetry at the requested resolution; individual frame and slice thresholds are reported separately. The separate [offscreen matrix](../benchmarks/results/native-acceptance-offscreen-v1.md) is not presentation evidence. A task-specific post-PR #49 eight-cell rerun remains incomplete: one bounded functional attempt did not confirm a visible window, and no accepted rerun followed. See [heavy-fixture slice p99 follow-up v1](../benchmarks/results/slice-p99-offscreen-followup-v1.md). |
| Simulation slice p99 at or below 4 ms | unmet in portions of the complete windowed matrix | [Native all-fixture windowed acceptance v1](../benchmarks/results/native-acceptance-v1.md) reports slice p99 and maxima for all 72 accepted captures; its per-fixture/policy checklist identifies misses, including traditional-policy captures. [Native sustained capture v3](../benchmarks/results/native-sustained-v3.md) separately reports three FIFO captures with p99 4.372, 4.391, and 4.398 ms and maxima 4.904, 4.972, and 4.981 ms. These are CPU wall-time samples, not thread CPU time. The task-specific post-PR #49 offscreen results and failed windowed confirmation are in [heavy-fixture slice p99 follow-up v1](../benchmarks/results/slice-p99-offscreen-followup-v1.md); they do not replace acceptance evidence. |
| Frame interval p99 at or below 20 ms, with >33.3 ms counts and maximum disclosed | measured across all fixtures; unmet in portions of the matrix | [Native all-fixture windowed acceptance v1](../benchmarks/results/native-acceptance-v1.md) reports p99, maximum, and >33.3 ms counts for all 72 accepted captures; the per-fixture/policy checklist marks each threshold result. Traditional-policy captures and several bounded captures exceed the p99 target. [Native sustained capture v3](../benchmarks/results/native-sustained-v3.md) retains its separate three-run FIFO comparison. Frame intervals include event-loop presentation pacing but are not GPU duration or scanout timing. |
| Camera/UI feedback p95 at or below 50 ms with documented method | met | [Game-feel measurement](#game-feel-measurement) and [native sustained capture v2](../benchmarks/results/native-sustained-v2.md); display scanout is excluded |
| Focus-enabled action-to-first-effect p95 at or below 50 ms under sustained overload, with minimum background service | met | [Player-focus results v1](../benchmarks/results/player-focus-v1.md): bounded-focus p95 2.754 ms across scripted headless actions; native material-visible latency is reported separately |
| Full-size mixed overload stays within resource limits without crash, deadlock, or loss of accepted work | not measured | [Full-size native smoke](#full-size-release-smoke-after-the-traditional-frontier-correction) passed functionally. A short Metal System Trace measured 52.875 MiB of app-attributed Metal allocation; the full sustained acceptance and memory outside the app's Metal-device metric remain unmeasured. See [resource audit](#world-size-resource-bounds). |
| Finite stress burst drains or reaches a verified stable state in a measured recovery window | unmet | [Headless recovery follow-up v5](../benchmarks/results/headless-v5.md) records the accepted 72,000-slice share-50 baseline at 267,243 pending channels. The task follow-up's accepted share-5 run retained 120,560 but reached no stable window; its source-paired share-50 control failed the load gate. See [slice-p99 offscreen follow-up v1](../benchmarks/results/slice-p99-offscreen-followup-v1.md).
| Paired heavy-work fixture materially reduces bounded p99 frame latency while showing completion cost; target 2x | met | [Native sustained capture v2](../benchmarks/results/native-sustained-v2.md): mixed-overload comparison shows the bounded/traditional latency difference and completion/backlog context; traditional is the specified full-frontier baseline |

## Reproducible all-fixture acceptance command

After a compatible simulation revision is on `main` and a quiet-machine window is available, run this single command from the repository root:

```sh
./scripts/native_acceptance_suite.py
```

It builds the release app once, then captures all eight section-12 fixtures under bounded focus, bounded FIFO, and traditional scheduling for three 60-second repetitions per fixture/policy (72 accepted runs total). Fixture captures do not add external disturbances or player actions, except `mixed-overload`, which uses the native sustained-overload action stream to exercise focus latency; quiet-world is the quiet baseline. A fresh app process prepares and warms each fixture before timing. Each attempt checks the one-minute host load immediately before and after, accepts only load <3 at both points, and verifies the 1920×1080 window, requested duration, successful app result, and complete frame/slice telemetry. Frame sample counts reflect actual presentation rate, not an assumed 3,600 intervals. Failed-gate and invalid attempts are retained in the generated report; load-gate failures retry automatically. A diagnosed window-occlusion startup failure before the capture timer begins may retry once because no timing data was collected; other non-load failures stop the suite. Raw per-attempt logs go to ignored `benchmarks/tmp/native-acceptance/` and are not committed. The curated summary, including fixture/policy checklist rows and available camera/action p95 sample counts, is written to `benchmarks/results/native-acceptance-v1.md`.

The run is lengthy. It is not a CI timing test. The implementation can be exercised cheaply with the hidden subset options, `--seconds 1`, `--binary PATH`, and `--no-caffeinate`; those options do not produce acceptance evidence.

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

The task branch coalesces refreshes of an unchanged active focus rectangle, retains chunk membership cache entries across slices until a region changes or expires, and skips chunk lookups when no focus region is active. These bookkeeping changes preserve scheduler ordering and quantum charges; they have not yet demonstrated the required native p99 improvement.

The v2 report is superseded for the traditional-policy comparison: its traditional scheduler serviced ready-ring entries and only a single recovery probe instead of capturing the entire per-cell pending frontier. The v2 numbers remain in [the archived v2 report](../benchmarks/results/headless-v2.md) for traceability and must not be treated as results for the corrected policy. The initial v1 pilot is also superseded. Recovery pricing, water flow, sampled quantum attribution, and the latest burst results are documented in [headless recovery follow-up v5](../benchmarks/results/headless-v5.md); current accepted native timing is in [native sustained capture v3](../benchmarks/results/native-sustained-v3.md).

The simulator supports up to 10,000,000 credits. The app now reads its default allowance and UI cap from `profiles/m2-16gb-v3.toml`, currently 1,000,000 credits. A full-size release-mode native smoke against the corrected traditional frontier is recorded below; this short run is not sustained interactive acceptance.

## Traditional frontier correction

Traditional mode now treats per-cell pending state as authoritative, independent of ready-ring capacity. At each measured slice it performs two full scans of the per-cell storage: one to snapshot and clear pending evaluation/blast channels and ready rings, and one to process the captured one-byte-per-cell snapshot. It processes only the command count captured at slice entry. Every scan cell and every captured dispatch is charged and included in slice wall time; newly generated work waits for the next slice. At 4096 x 4096, the two scans alone account for 33,554,432 recovery quanta and 704,643,072 credits per slice at the current 20-credit recovery and 1-credit selection costs, even when no cells are pending. This is the required batch-all baseline, not a general claim about conventional engines. The measured burning-forest p99 was 226.496334 ms traditional versus 2.978875 ms bounded, but much of the traditional interval is the mandatory full-world scan. See the v3 results for all fixtures, exact work, and the 1,200-command burst/recovery outcome.

Ready capacity remains 32,768 per lane (65,536 combined), with 256 command slots. Enlarging the rings would increase queued-storage allowance but would not increase bounded-mode service under its credit budget or reduce the traditional full-storage scan; the corrected traditional policy no longer treats ring occupancy as a frontier limit. The `tiny-capacity` descriptor separately tests two ready entries per lane.

Fixture disclosures: `explosive-lattice` has a seeded center blast (energy 15) pending at fresh measurement start; warm-up is followed by fixture re-preparation. `dirty-world-sweep` prepares a static stone/air pattern and has no active headless simulation channels. Its renderer dirty-chunk drain/upload is app-side and absent from the headless runner, although traditional's two full-world scans are still measured for it.

## Headless method and machine

Release build: `cargo build --release -p cascade-bench`. The runner reports preparation and warm-up durations separately, re-prepares the descriptor after optional warm-up, and records measured wall time around each CPU-only `World::step`. The timer is monotonic wall time, not thread CPU time, and includes operating-system preemption. With the `cascade-sim/quantum-timing` feature, bounded-mode CSV/JSON also estimates time inside evaluation, blast, recovery, command, and scheduler-selection work by timing every eighth operation and scaling samples by eight. Enable it with `cargo run --release -p cascade-bench -F cascade-sim/quantum-timing -- ...`. These diagnostic timers still add overhead and are for relative attribution, not acceptance slice-duration captures; the remaining slice time includes queue handling and other dispatch work. The app's `quantum-diagnostics` feature adds the same sampled body timing, split by focus/background lane for evaluation and blast, and reports selection p99/mean separately; its sampler spans slice boundaries so sparse operations are not systematically missed. Capture output labels the time remaining after all sampled bodies as a residual estimate. These instrumented app timings are diagnostic only, not acceptance evidence. The runner reports a stable window only after 60 consecutive post-input slices have zero pending channels, ready jobs, commands, and fixture/reset work. With no runnable work in those slices, authoritative cell contents cannot change. See the v3 report for the exact calibration, paired-matrix and burst protocols, sample counts, percentiles, backlogs, work, and historical completion semantics. The v3 profile now assigns calibration and held-out validation roles to disjoint scenario descriptors; historical v3 calibration and validation streams shared descriptors and do not provide fixture-level holdout evidence.

Reference machine: MacBook Air (Mac14,2), Apple M2 with 8 CPU cores, 16 GB RAM and integrated M2 GPU; macOS 27.0.1; built-in 2560 x 1664 display. Rust 1.99.0 (`b940084d7`, aarch64-apple-darwin), Cargo 1.99.0 (`5f94df478`). `Cargo.lock` SHA-256: `c4084ec2080b2c4da29d605573b159b02504f86af481d13c5807e565d419afe7`. Power mode, thermal state, background load, and display refresh/presentation mode were not controlled.

## Allocation and GPU diagnostics

`cargo test -p cascade-sim --test hot_path_allocations` installs an instrumented system allocator in a dedicated test executable, initializes a world and queues 256 paint commands before measurement, then checks 256 slices at the minimum 25-credit allowance. The test asserts zero allocations, reallocations, and newly allocated bytes while the scheduler services queued work, and asserts the credit limit on every slice. This verifies the simulation/scheduler hot path after initialization in the tested build; it does not cover app presentation code or prove behavior for every possible event path.

For app-side visibility, build with `cargo run --release -p cascade-app --features allocation-diagnostics -- --offscreen-capture --capture-policy bounded-focus --capture-seconds 60 --world-size 4096`. The diagnostic global allocator is sampled around each `World::step` dispatched by `Demo::tick`; the overlay reports allocation-call delta per simulation frame. Ordinary builds show that diagnostic allocation counts are unavailable. Instrumentation overhead means diagnostic builds are for allocation observation, not timing publication.

GPU duration is unavailable on the reference adapter, with direct capability evidence from `cargo run -p cascade-app --example adapter_caps`: backend `Metal`, adapter `Apple M2` (`IntegratedGpu`), `TIMESTAMP_QUERY=true`, `TIMESTAMP_QUERY_INSIDE_ENCODERS=false`, and `TIMESTAMP_QUERY_INSIDE_PASSES=false`. The base query feature alone cannot write a timestamp: wgpu 30 requires the encoder or pass feature for timestamp writes. Its feature documentation lists those write features for Metal AMD/Intel, not Apple GPUs ([encoder timestamp feature](https://docs.rs/wgpu-types/30.0.1/wgpu_types/struct.FeaturesWGPU.html#associatedconstant.TIMESTAMP_QUERY_INSIDE_ENCODERS), [pass timestamp feature](https://docs.rs/wgpu-types/30.0.1/wgpu_types/struct.FeaturesWGPU.html#associatedconstant.TIMESTAMP_QUERY_INSIDE_PASSES)). The adapter therefore lacks a usable timestamp-write path, so delayed GPU duration queries cannot be implemented through this wgpu backend; the overlay labels timing unavailable rather than presenting CPU submission time as GPU duration. This is not a generic claim about every backend.

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

### Metal allocation high-water diagnostic

On the reference Apple M2 / Metal host, `scripts/measure_metal_memory.py` launches a short functional capture under Xcode Instruments' `Metal System Trace` and extracts the maximum `metal-current-allocated-size` event (`MTLDevice.currentAllocatedSize`) for the app process. The measured windowed high-water was 55,443,456 B (52.875 MiB), 41.3% of the charter's 128 MiB GPU budget. This measures Metal-device memory allocated by the application, including renderer-managed Metal objects visible to Instruments; it is not a timing run. It does not account for allocations outside that device metric, including any WindowServer-owned surface backing or driver-private/global residency not attributed to the process. Those unreported amounts remain unknown, not zero. See [the Metal memory result](../benchmarks/results/metal-memory-v1.md) for the exact capture details and limits.

Reproduce with `./scripts/measure_metal_memory.py` on macOS with Xcode Instruments installed. The generated `.trace` and XML are retained locally under ignored `benchmarks/tmp/metal-memory/`; do not commit them.

At 4096 x 4096 with default ready capacities, one normal `World` accounts for 103,063,040 bytes (98.29 MiB) of simulation-owned arrays. The app also keeps a tiny-capacity alternate world, a CPU material grid, and one GPU material texture. The two worlds together account for about 196.6 MiB, the CPU material grid is 16 MiB, and listed application-owned CPU storage is therefore about 212.6 MiB (83.0% of the charter's 256 MiB CPU budget). The tiny world's exact resource count is exposed by `World::resources()`; this sum rounds its size to the normal-world figure and is conservative.

### Explicit GPU resource inventory

The app's explicit GPU resources are listed below; texture/buffer sizes are their descriptor sizes, not driver-heap measurements.

| Path/resource | Size at 4096² world and 1920×1080 output | Lifetime/notes |
| --- | ---: | --- |
| R8Uint material texture | 16,777,216 B (16 MiB) | Persistent, one byte per world cell; windowed and offscreen |
| Frame uniform buffer | 160 B | Persistent |
| Offscreen RGBA8 render target | 8,294,400 B (7.91 MiB) | Persistent for one offscreen process; absent in windowed mode |
| App-created steady-state GPU staging buffer | 0 B | Uploads use `Queue::write_texture`; the app creates no persistent GPU staging buffer. Wgpu/backend transient staging allocations are not exposed as app-owned resources. |
| Egui textures and vertex/index buffers | Not queryable from app descriptors | Renderer-managed and dynamically sized; not included in known subtotal |
| Window surface images | Not queryable from app descriptors | Surface/backend-managed; image count and allocation are not exposed as app-owned allocations |
| Optional smoke screenshot readback | `align_up(width × 4, 256) × height` B | Transient; 8,294,400 B at 1920×1080, created only for screenshot smoke |
| Optional 8×8 smoke sample texture/readback | 256 B texture + 2,048 B readback buffer | Transient post-measurement verification |

The known persistent windowed subtotal is 16,777,376 B (16 MiB plus the 160 B uniform), 12.5% of the charter's 128 MiB GPU budget. The offscreen known subtotal is 25,071,776 B (23.91 MiB), 18.7% of that budget. The current upload path prepares and reuses a fixed 8 KiB stack chunk with 256-byte rows per `Queue::write_texture` call. At the 256-copy frame cap, telemetry reports at most 2 MiB of padded staging input per frame; this is CPU-side write data, not a claim that wgpu retains 2 MiB of resident GPU staging. Wgpu/backend-managed transient staging is not an app-owned buffer. The Metal System Trace reports an app-attributed allocation high-water separately, but does not enumerate how much of the excess comes from egui, surface images, or backend resources. Any surface backing or driver/backend-private residency outside `MTLDevice.currentAllocatedSize` remains unknown, so the explicit descriptor subtotal is still a lower bound on the wider system-level GPU footprint. Process RSS is not the charter's application-owned CPU storage measure. Startup allocations and fixture-preparation latency are separate from steady-state measurements.

## Fixture roles and held-out validation

Each scenario descriptor TOML declares `benchmark_role`. The v3 profile names `burning-forest-v1` and `mixed-overload-v1` as calibration descriptors and six distinct descriptors as held-out validation: quiet-world, explosive-lattice, sand-release, reservoir-breach, dirty-world-sweep, and tiny-capacity. Their descriptor-level groups do not overlap. The existing v3 fixture-matrix report includes the calibration fixtures and is not itself a held-out-only result; the profile's short validation measurements are limited to the disjoint fixture matrix evidence and do not satisfy the sustained 60-second acceptance protocol. The old statement that calibration and validation share static descriptors refers to that earlier mixed-purpose matrix and is superseded by the explicit descriptor roles.


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

The simulator memoizes focus membership by logical chunk for the current focus-region epoch and reuses one result for both channels in recovery. Focus/viewport changes, reset, and enable changes invalidate the cache. The epoch and membership bit are packed into one 32-bit entry, using 65,536 bytes at 4096², down from the initial 262,144-byte cache. `cascade-bench` exports per-slice evaluation, blast, recovery, and command counts; native captures export work counts at the maximum simulation slice and timings at the slowest >33.3 ms presentation frame. See the full headless calibration and adverse samples in [v4](../benchmarks/results/headless-v4.md).

The post-cache valid native suite reports focus simulation maxima 4.876–5.062 ms and FIFO 5.113–5.600 ms. Focus frame p99 improved to 17.654–17.709 ms and met 20 ms; FIFO p99 improved to 20.506–20.621 ms but remained above 20 ms before the upload fix. There were no bounded intervals above 33.3 ms among 21,600 samples, so the earlier 34.342 ms focus outlier did not recur. Max-slice counts remain evaluation-heavy: focus had about 33–34k evaluation quanta and 45k selection probes; FIFO about 35k evaluations and 59k selections. The cache does not affect FIFO mode. Credit allowance remains 1M; no scheduler work-throughput reduction was made. Exact original run chronology, gated-out attempts, upload/action timings, and method deviations are in v2.

## Upload timing-tail follow-up, 2026-10-07

The old R8 upload path copied a 32-byte row using `Queue::write_texture`. In wgpu-core 30.0.1, a 32-byte row is padded to the 256-byte buffer-copy pitch, taking the chunked staging path: 32 row copies and an 8,192-byte staging allocation for each 1,024-byte chunk payload. The pixel conversion also linearly searched up to 64 acknowledgment targets for every texel. A planned 256 uploads plus as many as 16 additional player-target uploads could therefore reach 272 copies/frame and about 17.8 million acknowledgment comparisons/frame.

The fix uses a fixed chunk-local acknowledgment mask and writes R8 chunks with 256-byte-aligned rows, selecting wgpu's contiguous staging-write path. It reserves 16 priority slots inside the existing 256-copy frame cap, limiting the ordinary queue plan to 240. At 4096², a 32x32 chunk now contributes 1,024 logical bytes, 7,968 staging bytes, and 6,944 row-padding bytes. Captures report both logical payload and staging work, including values for the frame with maximum upload CPU. The 1M simulation allowance was not reduced.

Three 60-second, 1920x1080 windowed FIFO captures on `dc237d2` plus the max-slice credit diagnostic passed the quiet-machine gate and reported no occlusion or interval-buffer loss:

| Rep | Load before/after | Intervals | p50 ms | p95 ms | p99 ms | max ms | >33.3 ms | max sim CPU ms | max upload CPU ms | max chunks | max payload bytes | max staging bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 2.27 / 1.92 | 3,600 | 16.679 | 17.335 | 18.096 | 20.023 | 0 | 5.287 | 3.468 | 242 | 247,808 | 1,928,256 |
| 2 | 1.94 / 2.85 | 3,600 | 16.685 | 17.288 | 17.944 | 21.365 | 0 | 5.188 | 3.729 | 242 | 247,808 | 1,928,256 |
| 3 | 2.76 / 2.89 | 3,600 | 16.682 | 17.318 | 18.000 | 20.468 | 0 | 5.410 | 3.538 | 242 | 247,808 | 1,928,256 |

The upload-CPU peak frames used 132, 132, and 128 chunks, with logical payloads of 135,168, 135,168, and 131,072 bytes; staging sizes of 1,051,776, 1,051,776, and 1,019,904 bytes; and row padding of 916,608, 916,608, and 888,832 bytes. Compared with the prior valid FIFO captures in v2, p99 fell from 20.506–20.621 ms to 17.944–18.096 ms, and max upload CPU fell from 8.865–9.382 ms to 3.468–3.729 ms. All three p99s meet 20 ms for this fixture; other fixtures remain unmeasured.

The maximum simulation slices in all three captures still exceed 4 ms. Rep 1 executed 35,290 evaluations, 3 blasts, 23,531 recoveries, 1 command, and 58,832 selections, exactly totaling 1,000,000 credits at the current 24/24/4/12/1 charges. Rep 2 executed 35,291 evaluations, 2 blasts, 23,532 recoveries, 1 command, and 58,828 selections; rep 3 executed 35,290 evaluations, 4 blasts, 23,527 recoveries, 1 command, and 58,824 selections. Each also totals exactly 1,000,000 credits. This is aggregate budget saturation, not evidence of a credit-limit violation. The reported 98.29 MiB simulation footprint cannot reside in CPU caches; hardware cache counters were not collected, so no cache-miss rate is claimed. Existing headless v4 reports p99 below 4 ms but eight burning-forest samples over 4 ms and a 9.767 ms maximum. We retained the profile because lowering credits would reduce useful quanta; these aggregate profiles show full accounted credit use, not evidence of a per-quantum undercharge.

One full-size offscreen attempt on `dc237d2` was excluded after load rose from 2.29 to 3.44. An earlier accepted offscreen run on the upload-fix branch measured FIFO p99 19.505 ms and max upload CPU 3.683 ms, but it is separate evidence from the windowed runs. A second windowed attempt with load 2.27/3.08 was likewise excluded. Offscreen and rejected-attempt details are retained in [native offscreen comparison v1](../benchmarks/results/native-offscreen-v1.md).

## Remaining unmet targets

- The headless v4 calibration p99 target remains met for burning-forest and mixed-overload, but accepted native FIFO p99 is 4.372–4.398 ms and misses the 4 ms requirement. The three current windowed maxima are 4.904–4.981 ms; headless burning-forest had 8/5,400 samples over 4 ms and a 9.767 ms maximum. The full fixture acceptance suite remains incomplete.
- The three post-fix windowed FIFO frame p99s are below 20 ms (18.485, 18.521, and 18.661 ms); the all-fixture suite remains incomplete. The prior 34.342 ms focus interval did not recur, but remains historical evidence.
- The traditional full-frontier policy had every measured frame interval over 33.3 ms and is far beyond the 4 ms simulation-slice target by design. This is the charter's intentionally expensive batch-all baseline, not evidence about all conventional engines.
- GPU timing is unavailable in the current app build. A short mixed-overload Metal System Trace measured the app-attributed Metal allocation high-water, but complete system GPU residency remains unmeasured because WindowServer-owned surface backing and driver/global allocations outside the app device metric are unknown. Native visible-action latency excludes display scanout/composition. Captures cover one M2 Mac and one mixed-overload fixture, not all fixtures or devices.
- The 1,200-command full-size bounded burst at the 1,000,000-credit profile remains unresolved. The accepted share-50 v5 run ended with 267,243 pending channels; an accepted share-5 task follow-up ended with 120,560 but reached no stable window. The same-source share-50 control run failed the post-load gate, so the comparison is not fully accepted as a pair. A separate 120,000-slice v5 follow-up retained 97,468 channels but failed the post-run load gate and is diagnostic only. Pending-channel census was predominantly sand, suggesting granular settling is now the main residual activity after the water-direction correction, but not proving it is the only cause. Stable means 60 consecutive no-input slices with no pending channels, no commands, and no ready jobs, so no authoritative cell contents can change during the interval. A persistent positive backlog is not called stable. See [headless recovery follow-up v5](../benchmarks/results/headless-v5.md) and [slice-p99 offscreen follow-up v1](../benchmarks/results/slice-p99-offscreen-followup-v1.md).
- The app maximum/default reads the selected 1,000,000-credit allowance from profile v3. The 5% focus-share candidate improves offscreen focus p99, but FIFO p99 still misses 4 ms. One bounded post-PR #49 functional attempt did not confirm a visible window; per task dispatch, no further windowed attempts will be made. See [heavy-fixture slice p99 follow-up v1](../benchmarks/results/slice-p99-offscreen-followup-v1.md).
- Historical calibration and validation runs shared the burning-forest and mixed-overload descriptors. Scenario descriptors now carry disjoint calibration/held-out-validation roles; historical calibration measurements are not held-out results. See [fixture roles](#fixture-roles-and-held-out-validation).

Results are empirical on one machine. Bounded-mode credits constrain accounted algorithmic work, not operating-system or hardware latency.
