# Cascade: Bounded-Work Simulation Charter

Status: implementation charter for v0.1  
Audience: a coding agent starting from an empty repository  
Working name: Cascade

## 1. Mission

Build a small, playable, native 2D cellular sandbox that invites the player to overload it. Demonstrate an engine designed around a fixed amount of computation per update: when demand rises, effects take longer to resolve while interaction and rendering remain responsive on the measured reference system.

The engine technology is the product. Implement the simulation and its scheduler from scratch. Deliver an understandable experiment, reproducible evidence, and honest instrumentation before adding breadth or visual polish.

## 2. Core hypothesis and invariant

**Hypothesis:** a useful and entertaining cellular world can preserve responsive presentation under adversarial player input by bounding work and memory, accepting slower effect resolution as a visible game rule. Precisely: player actions may grow outstanding demand `Q_s` without limit (up to fixed storage, beyond which it is coalesced, recovered, or rejected), but cannot raise executed work `W_s` above the slice allowance.

**Required invariant in bounded mode:** for every scheduler slice `s`,

```text
sum(charged_cost of every executed quantum in s) <= B_sim
```

This holds only together with a second invariant: every indivisible quantum `w` has a worst-case cost `C(w) <= C_max` that its charge conservatively covers. A quantum that hides a search, scan, or graph walk makes the accounting fiction. Every quantum has a fixed upper bound on inspected cells, writes, queue operations, and emitted work. Selection, stale-job handling, admission, and bookkeeping also consume credits. No workload-dependent operation is hidden outside the accounting.

At most one simulation slice runs per presentation iteration. Input processing, world preparation, render uploads, UI generation, and diagnostics have separate finite caps. Maintain a resource table for all engine-owned queues, buffers, pools, and resident world storage. Player actions may increase demand but cannot increase these limits.

Credits bound algorithmic work, not elapsed milliseconds. A general-purpose OS, graphics driver, GPU, thermal throttling, page faults, or background activity can delay a frame. Do not claim a universal no-lag guarantee or a proven wall-clock deadline. The proof obligation is the work/resource bound; frame latency is an empirical result on disclosed hardware.

The research question is whether the resulting delay, staleness, and scheduling artifacts are acceptable and legible to the player.

### Positioning and prior art

The individual mechanisms are established: sleeping bodies, simulation level of detail, significance-driven tick rates, capped physics catch-up, time-critical collision detection with progressive refinement, multi-rate simulation, and hard real-time budgeting. Do not claim any of them as new. The project explores **bounded-work simulation**: making the per-slice work bound a first-class engine invariant, so that every player-triggerable workload is resumable bounded work under one common budget, rather than a per-subsystem optimization.

Prior work maps onto three overload responses: **defer** the same computation (Unity's capped catch-up), **refine** a coarse answer as budget allows (Hubbard; Dingliana and O'Sullivan, who also carry bounded work into collision response), and **substitute** a cheaper model (Chenney's proxy simulation). Unreal's Significance Manager supplies priority information but no global admission control. Those techniques bound one subsystem or advise game code; none of these sources makes a bounded-work contract mandatory for all player-caused simulation. v0.1 implements **defer** with documented coalescing. Refine and substitute are post-v0.1 experiments. Do not claim the composition has never been built; a broader literature search would be required first.

Cite prior art in `README.md`, including Hubbard's time-critical collision detection (ACM TOG 1996), Dingliana and O'Sullivan's graceful degradation of collision handling (CGF 2000), Chenney's simulation level of detail (GDC 2001), Unity's Maximum Allowed Timestep, and Unreal's Significance Manager. Verify each citation against its primary source before publishing.

## 3. Demo concept

The player is an engineer and saboteur in a cellular test facility. They paint materials, build reservoirs and explosive chains, ignite wood, remove supports beneath loose sand, and trigger disasters. The challenge is to create more work than the simulator can immediately resolve.

Display the tradeoff directly: frame-time history stays near its target while pending work and effect latency grow. Show deferred regions and material updates so a slow explosion is visibly an intentional scheduling consequence.

Required controls:

- Pan and zoom; select material; paint and erase with a capped brush.
- Ignite or detonate a selected cell.
- Pause, single-step one scheduler slice, reset, and load a named fixture.
- Adjust simulation credits within validated limits.
- Switch between bounded and traditional comparison runs by restarting a fixture. This comparison is the demo's headline: the same trigger under both policies, with frame-interval and pending-work graphs side by side or recorded per run.
- Activate a prominently labeled **DESTROY PERFORMANCE** button.
- Toggle metrics and a deferred-work overlay.

There is no player character or campaign in v0.1. Simple challenges such as igniting an entire explosive chain are sufficient.

## 4. Non-goals

Do not implement 3D, rigid bodies, structural engineering, arbitrary fracture, pressure fluids, networking, multiplayer, audio, scripting, a scene graph, an asset pipeline, infinite worlds, streaming terrain, or a general editor.

Do not use a game engine, physics library, or ECS framework. Do not start with GPU simulation, worker-thread simulation, adaptive wall-clock budgeting, or hierarchical aggregation. Those are later research directions, not prerequisites.

Do not advertise deferred local updates as physically accurate global slow motion. Do not build millions of independently allocated objects or hide overload by silently deleting authoritative state.

## 5. v0.1 scope and technical stack

| Layer | Decision |
| --- | --- |
| Language/build | Stable Rust and Cargo; pin the tested toolchain and commit `Cargo.lock` |
| Window/input | `winit` |
| GPU rendering | `wgpu` with WGSL shaders |
| Simulation | Custom CPU cellular rules, single-threaded |
| Scheduler | Custom bounded, resumable work-unit scheduler |
| Physics | Local cellular rules only; no physics library |
| ECS | None; explicit arrays, IDs, bitsets, queues, and pools |
| UI | Small bounded debug overlay; `egui` is permitted if it simplifies delivery and its cost is measured |
| Utilities | Small justified crates for POD transfer, error reporting, logging, and test support |
| Profiling | Coarse tracing spans and native profiling tools |
| Targets | Native desktop first; validate on the development OS and document other platforms as unverified until tested |

Choose mutually compatible stable dependency versions during bootstrap using official documentation. Do not inherit unverified version numbers from earlier discussion. Avoid speculative abstraction and unnecessary dependencies.

Required simulation features: air, stone, wood, sand, water, explosive; local gravity for sand and water; local water spreading; wood ignition and burning; bounded explosion propagation; dormant and active state.

Stone and wood remain fixed except when destroyed. Removing support only causes granular material to fall. Fire is cell state, not an additional material requiring a particle system.

## 6. World and cell model

Default demonstration world: `4096 x 4096`, or 16,777,216 cells. Support small dimensions for tests and a smaller interactive fixture for early milestones. World size is fixed per run and validated before allocation.

Partition the world into logical `32 x 32` chunks for activity and rendering metadata. Use dense, preallocated cell storage initially. Chunk compression and residency streaming are deferred.

Target 4 to 8 bytes per cell, yielding 64 to 128 MiB of raw cell storage at the default size. Account separately for pending flags, effect state, queues, chunk metadata, textures, staging buffers, and diagnostics. Provisional limits are 256 MiB of application-owned CPU storage and 128 MiB of application-owned GPU resources; audit actual layouts and revise explicitly if evidence requires it. These are not process-RSS guarantees.

Use fixed-width integers for material IDs, flags, heat/fuel or lifetime, and bounded explosion energy as needed. Prefer simple contiguous representations; use structure-of-arrays only where access patterns justify it. A cell ID is a validated integer index, never a heap object or pointer-based identity.

Define a closed world boundary. Water and sand move through atomic source/destination updates and conserve cell counts except for documented destruction rules. Specify tie-breaking, neighborhood order, heat decay, ignition threshold, burn duration, blast attenuation, and material conversion in a rules document and tests.

Dormant cells receive no recurring evaluation. Mutations wake a fixed neighborhood. Each local rule inspects at most eight neighbors and has a fixed fan-out. No rule performs a flood fill, full-column search, connected-component scan, or arbitrary-radius iteration in one quantum.

## 7. Architecture principles

1. Keep the simulation crate independent of windows, graphics, wall clocks, and UI.
2. Make unbounded synchronous simulation structurally hard: no public `sim` API performs workload-dependent work inline. Operations that can scale with world content (detonate, paint region, reset, fixture preparation, flood or connectivity queries) admit a bounded descriptor or return a job/cursor that advances only under the scheduler. The traditional comparison policy changes only how many quanta run per slice, never the quantum contracts.
3. Separate world state, pending demand, scheduler policy, and presentation state.
4. Represent long work as explicit cursors or finite state machines that yield after one bounded quantum.
5. Preallocate hot-path storage. No runtime growth, blocking file I/O, recursive propagation, or synchronous logging in the interactive hot path.
6. Treat overload as part of the model: defer, coalesce with documented semantics, or reject before admission.
7. Keep a single owner of authoritative simulation state. Rendering may show an older texture but must report that staleness.
8. Make scheduling decisions inspectable and reproducible. A feature is incomplete until its worst-case work, capacity, overflow behavior, and tests are defined.
9. Prefer the simplest correct implementation. Optimize from profiles after the bounds are demonstrable.

## 8. Bounded scheduler design

### Work quanta and charging

Start with a closed job enum such as:

```text
EvaluateCell(cell_id)
PropagateBlast(cell_id)
PrepareRegion(cursor)
RecoverPending(cursor)
```

`EvaluateCell` handles a finite local rule set. Persistent heat and blast energy live in bounded cell-indexed state, not in an unbounded event history. `PrepareRegion` covers brush strokes, fixture generation, clearing, and reset work. Its cursor touches only a fixed number of cells per advance.

For each kind, define maximum reads, writes, neighbor admissions, scratch bytes, and integer credit cost. Costs are conservative weights, not claims that one credit has a fixed duration. Benchmark both common and worst observed paths, including stale jobs and saturated admission.

Charge before execution. A quantum must not begin unless its entire cost plus required scheduler overhead fits. Every enabled job type must fit within the minimum supported slice allowance. A job that cannot fit the remaining allowance waits for a later slice.

Conceptual dispatch:

```text
remaining = B_sim
while remaining can pay the bounded selection probe:
    charge selection probe
    candidate = inspect next lane using bounded operations
    if no eligible candidate:
        stop after a fixed maximum number of lane probes
    else if candidate cost fits remaining:
        reserve/charge candidate cost
        advance one quantum, including bounded admission and bookkeeping
    else:
        try another lane within the probe limit or stop
```

There must be no free loop over stale entries, empty lanes, expired jobs, or jobs too expensive to run. End-of-slice bookkeeping is charged or covered by an explicit fixed reserve.

### Storage, coalescing, and admission

Use a fixed-capacity ready ring with duplicate suppression and fixed-size per-cell pending state. A practical initial ready-ring capacity is 65,536 IDs; make it a recorded profile parameter.

When a cell is marked pending and the ready ring is full, retain its pending state. A budgeted recovery cursor incrementally visits cell metadata and admits pending cells as slots become available. Never scan the entire world to refill the ring in one slice. Reserve capacity or dispatch credits for recovery so it cannot starve behind newly generated demand.

Treat local update jobs as requests to reevaluate current state. Coalescing duplicate requests is therefore defined behavior. For blast propagation, specify a finite saturating energy merge rule and attenuation; do not claim additive physical shock-wave simulation. A fixed number of distinct pending channels per cell bounds memory even under repeated ignition.

Bound external commands separately, initially to 256 descriptors. Coalesce replaceable pointer/brush targets; reject other new commands when full and expose a counter. Do not silently evict accepted commands. Limit accepted commands processed per frame, initially to 64. Large accepted operations use cursors rather than eagerly expanding into per-cell jobs.

Every accepted deferred command must eventually finish under finite demand or be canceled by an explicit reset. Reset increments a world generation and clears storage incrementally while simulation is paused; stale jobs cannot affect the new generation. Generation rollover must be handled deliberately.

### Fairness and priority

Begin with deterministic FIFO/round-robin scheduling and explicit service shares for cell work, blast work, preparation, and recovery. Retain lane deficits/cursors across slices. Fixed quotas must leave every enabled lane a nonzero service opportunity.

Prove eventual servicing for finite accepted demand and test it. Do not promise bounded effect latency under sustained demand exceeding service capacity. Record oldest pending age and service counts by lane.

Camera-distance and visibility priority are optional after the baseline works. If introduced, retain minimum background service and record camera inputs in replays. Never resort the entire pending set each frame.

### Frame-loop contract

Poll input, advance bounded preparation/simulation work, perform capped texture uploads, draw the UI/world, and present. No loop attempts to repay an accumulated wall-time deficit with unlimited simulation slices. After a stall, resume the normal allowance and record missed presentation opportunities.

A provisional performance profile targets a 60 Hz presentation cadence and approximately 4 ms of simulation CPU time per slice. Calibrate a fixed credit allowance to the reference machine. Keep the allowance fixed within a comparison run. Optional 120 Hz profiles come later.

## 9. Time and determinism

v0.1 uses asynchronous local cellular semantics. A scheduler slice is a scheduling interval, not a promise that every cell experienced one global physics tick. Burn timers and flow transitions advance when serviced according to documented rules. Under load, different regions may progress at different effective rates.

Do not integrate an arbitrarily large elapsed wall-time delta when a cell wakes. Do not call local delay uniform time dilation. Display slice count, completed work, pending age, and wall-clock resolution latency; avoid inventing a single global simulation-speed ratio.

For a fixed executable/rules version, initial world, seed, scheduler configuration, credit schedule, and slice-indexed command stream, require identical state and scheduler hashes after the same number of slices.

Use integer or explicitly specified fixed-point authoritative math, explicit overflow behavior, stable iteration order, and stable IDs. Avoid hash-map iteration order, wall-clock decisions, and floating-point state in simulation. Use a specified PRNG or coordinate/counter-derived randomness with golden test vectors.

Record commands at their admitted slice and sequence number. Replays include capacities, seed, rule version, dimensions, and budget configuration. Hash world state plus pending flags, queues, continuation cursors, PRNG state, and other future-affecting state. Compute large hashes outside timed runs or incrementally under a separate budget.

Changing the budget, scheduling policy, or overload policy can change outcomes. Cross-budget identical trajectories and cross-platform determinism are not assumed; test and document any stronger claim separately.

## 10. Rendering

Use CPU simulation and GPU rendering. Store compact cell material/state in a GPU texture and render it with a full-screen triangle or camera-aligned quad and a WGSL palette lookup. Avoid geometry or draw calls per cell.

Track dirty chunks with deduplicated bounded metadata. Provisional upload limits: 256 chunks and 1 MiB of payload per frame, plus a fixed submission/call cap. Use bounded staging resources and respect backend row-alignment requirements. Uploads left unfinished remain dirty; multiple changes to a dirty chunk collapse into its latest state.

Render stale texture data when upload demand exceeds the cap. Show pending upload count and oldest dirty age. Upload ordering must be fair. Initial texture population, world reset, fixture generation, and shader/resource setup have explicit loading/preparation states; keep their times separate from steady-state measurements.

Cap render resolution and UI vertices in the performance profile. Handle resize storms with coalescing and avoid repeated resource recreation within a frame. Do not add GPU readbacks or synchronous GPU waits to the hot path. Device loss may end a run with a clear error; it is outside the no-overrun claim.

## 11. Instrumentation

The default overlay must show:

- Mode, fixture, seed, budget profile, world size, and slice index.
- Frame interval and rolling p50/p95/p99/max, plus missed-target count.
- CPU time for simulation, scheduler overhead, uploads, rendering submission, and UI.
- GPU duration when supported through delayed, nonblocking measurement; otherwise mark unavailable.
- Credits allowed/used, executed quanta by type, and budget violations.
- Ready-ring occupancy/capacity, distinct pending cells/channels, oldest pending age, and completion latency.
- Coalesced/rejected commands, saturation counts, and recovery service.
- Active/dormant chunks, dirty upload backlog, and presentation staleness.
- Application-owned CPU/GPU memory and hot-path allocation counts in diagnostic builds.

Use a fixed-size history ring and bounded histogram updates. Calculate expensive summaries after the run or incrementally. Do not allocate or emit one log line per job. Export machine-readable CSV or JSON after measurement; report sample loss if a bounded telemetry buffer fills.

A graph must show actual frame intervals, not smoothed FPS alone. Counter labels must distinguish exact pending counts from estimated future work. Cursors must not masquerade as millions of materialized jobs.

## 12. Stress scenarios and DESTROY PERFORMANCE

All fixtures are seeded, versioned, reproducible, available headlessly, and prepared incrementally or before timing begins.

| Scenario | Required pressure |
| --- | --- |
| Quiet world | Full-size mostly dormant world; verify near-zero simulation demand |
| Explosive lattice | Up to one million explosive cells with controlled gaps and chain paths |
| Sand release | Up to ten million sand cells above a removable support band; settle through local gravity |
| Reservoir breach | Large water volume released through a narrow opening; sustained local activity |
| Burning forest | Dense wood and many ignition sources; heat and blast interactions |
| Dirty-world sweep | Touch every chunk, then pan/zoom rapidly; saturate render uploads |
| Tiny-capacity test | Deliberately small ready ring and command queue; exercise saturation and recovery |
| Mixed overload | Explosions, sand, water, fire, painting, and camera motion concurrently |

**DESTROY PERFORMANCE** loads the mixed-overload fixture and starts a deterministic stream of repeated disturbance commands. Holding the button continues requesting disturbances at a capped admission rate. Release stops new requests; it does not erase admitted work.

Fixture preparation must not itself synchronously fill millions of cells. Show preparation progress and allow cancellation. Repeated button activation must not allocate more worlds, reset preparation endlessly without feedback, or grow queues.

Run both a finite burst, which must resolve or reach a stable state, and a 60-second sustained overload, which need not catch up. Follow the sustained run with a no-new-input recovery interval. Structural failures and rigid-body particle limits belong in a future scenario suite.

## 13. Traditional versus bounded comparison

Provide two scheduler policies over the same world representation, local rules, seed, capacities, coalescing semantics, and renderer:

- **Bounded:** consume no more than the configured slice allowance.
- **Traditional:** process every finite pending evaluation and blast channel found by a full per-cell storage scan at update entry, plus the command count captured at entry, without the slice credit cap. Ready rings are admission accelerators, not the authority for this frontier. Charge and time the full capture and execution scans plus every dispatched channel; newly generated work joins the next update.

The traditional policy snapshots and clears captured channels before executing any of them, so work emitted during execution cannot leak into the current frontier. Do not drain recursively until quiescence: flowing water or persistent activity may never terminate. The full scan cost is part of this intentionally expensive batch-all-eligible-work policy, not a claim about every conventional engine.

Restart the same fixture for each run; never compare different live-world histories by merely toggling a flag. For the cleanest baseline, use a preloaded fixture and a single trigger with no camera-dependent priority. Report time to completed work or settling alongside frame latency and rejected/coalesced work. Neither policy gets different visual effects or cheaper physics.

Memory bounds remain enabled in both modes. A traditional update may stall visibly; an external benchmark watchdog may terminate it. Report such a run as aborted, never silently turn it into bounded mode. Compare saturation-free and saturated fixtures separately so overflow semantics are not mistaken for scheduling gains.

Require deterministic repetition within each mode, not necessarily identical final states across modes when update grouping or external command admission changes. Include at least one order-insensitive fixture with equal final outcomes to validate comparison plumbing.

## 14. Milestones

1. **Headless foundation:** Cargo workspace, pinned toolchain, world layout, resource caps, integer rules, job contract, credit accounting, fixed queues, and replay tests. Exit: adversarial generated sequences cannot exceed credits or capacities.
2. **Visible vertical slice:** window, texture renderer, input, overlay, sand and explosives, bounded uploads. Exit: a small interactive chain reaction with visible work/backlog metrics.
3. **Complete v0.1 rules:** water, fire, dormancy/waking, bounded brush/reset/fixture preparation, and recovery cursor. Exit: local-rule tests and full-size world run without dynamic queue growth.
4. **Adversarial demo:** all required stress fixtures, DESTROY PERFORMANCE, the in-app traditional versus bounded switch, command saturation feedback, and recovery tests. Exit: sustained overload preserves every structural invariant and responsive controls on the reference profile, and the same fixture visibly stalls in traditional mode.
5. **Comparison and evidence:** replayable paired runs, latency distributions, resource audit, and benchmark report. Exit: acceptance criteria below are met or honestly reported as unmet.

Do not start a later research feature to avoid fixing a failed milestone.

## 15. Acceptance criteria

### Structural correctness, mandatory on every supported profile

- Every bounded slice uses at most its credit allowance, including saturation and stale-job paths.
- Every interactive engine-owned loop has a documented fixed limit or a charged resumable cursor.
- Queue, pool, upload, and resident storage limits hold under all stress fixtures.
- Accepted deferred work is retained or explicitly canceled; repeated requests cannot allocate an unbounded history.
- Finite pending demand receives fair service; stale generations never mutate a reset world.
- Sand/water conservation and blast/fire conversion rules pass tests.
- Replaying a recorded run reproduces full future-affecting state hashes.
- No hot-path heap growth in the simulation/scheduler after initialization, verified with allocation instrumentation.

### Measured release targets on the recorded reference system

After warm-up, run each fixture for at least 60 seconds, three times, at a fixed 1920 x 1080 render size and a 60 Hz target. Include a quiet baseline.

- Simulation slice duration: p99 at or below 4 ms after fixed-profile calibration.
- Frame interval: p99 at or below 20 ms, with every interval over 33.3 ms counted and maximum disclosed.
- Camera/UI feedback latency: p95 at or below 50 ms using a documented measurement method. This is distinct from material-effect resolution latency.
- Full-size mixed overload stays within resource limits and does not crash, deadlock, or lose accepted work.
- Finite stress bursts drain or reach a verified stable state within a measured recovery window; report the duration rather than hiding slow completion.
- In at least one paired heavy-work fixture, bounded mode materially reduces p99 frame latency while the report shows its effect-completion cost. Target a twofold reduction; if the baseline cannot be overloaded within declared caps, report the comparison as inconclusive.

These timings are targets, not fabricated results or portable guarantees. If unmet, retain the hard invariants, report evidence, and tune the profile or reduce scope explicitly. Never weaken a threshold silently.

## 16. Repository structure

```text
CHARTER.md
README.md
Cargo.toml
Cargo.lock
rust-toolchain.toml
crates/
  sim/                  # World, rules, jobs, scheduler, commands, replay
  app/                  # winit loop, input, overlay, renderer, WGSL
  bench/                # Headless runner and result export
scenarios/              # Small versioned descriptors and seeds
profiles/               # Resource caps and calibrated credit configurations
tests/                  # Integration fixtures or workspace test support
docs/
  architecture.md       # Work contracts, resource table, time semantics
  rules.md              # Material transitions and coalescing semantics
  performance.md        # Reproduction method and measured results
benchmarks/
  results/              # Small curated result summaries, not bulk captures
```

Keep GPU details out of `sim`. Begin with modules rather than many tiny crates. Document commands for running the app, tests, headless scenarios, and paired benchmarks in `README.md` as they become functional.

## 17. Coding conventions and testing strategy

Use `rustfmt`, Clippy, descriptive types for IDs/credits/generations, and explicit capacity-bearing constructors. Keep APIs narrow and errors actionable. Avoid `unsafe`; any necessary use requires a written safety argument and focused tests. Do not unwrap external input, GPU availability, or resource-allocation failures.

Comments should explain invariants and semantic choices. Every new work kind requires a cost contract, capacity/overflow description, and adversarial test. Keep changes small and runnable; do not introduce abstractions for hypothetical future engines.

Required checks:

- Unit tests for materials, neighbor boundaries, conservation, saturation math, wake/sleep transitions, charge-before-execute, and insufficient-credit behavior.
- Property tests over random worlds, seeds, commands, budgets, and tiny capacities, asserting credit and memory invariants after every slice.
- Tests for queue-full recovery, duplicate suppression, stale generations, long cursors, cancellation, and finite-demand fairness.
- Replay/golden tests for deterministic state and scheduler hashes; different render rates must not affect a headless replay.
- Integration tests for all fixture descriptors and the order-insensitive cross-mode comparison.
- Manual native smoke tests for resize, minimize/restore, pan/zoom, pause/step, reset during preparation, and repeated DESTROY PERFORMANCE input.
- Release-build performance tests kept separate from ordinary CI correctness tests; shared CI timing is not a reliable frame-latency gate.

Run formatting, linting, and workspace tests before each milestone. Headless correctness tests must run without a GPU.

## 18. Performance methodology

Record commit, toolchain, dependency lock, OS, CPU, GPU/backend, RAM, power mode, display/presentation mode, render size, world size, seed, capacities, and credit profile with each result.

Use release builds. Warm up shader pipelines and touch preallocated memory before steady-state captures. Measure initialization and fixture preparation separately. Benchmark both the headless core and the complete interactive app so rendering and upload costs cannot disappear from the claim.

Calibrate credits using representative expensive rules, saturated queues, recovery scans, and mixed workloads with headroom. Keep calibration inputs separate from held-out validation fixtures. Do not choose credits from average trivial-job time alone.

Report sample counts, p50/p95/p99/max, deadline misses, memory high-water marks, completed work, effect resolution latency, queue saturation, and rejected commands. Alternate paired mode order across repetitions. Preserve adverse results and disclose OS interference; do not selectively remove long frames.

Never publish illustrative FPS values as measured evidence. Keep raw captures out of version control unless small and intentional; commit reproducible descriptors and concise result summaries.

## 19. Risks and open questions

- Weighted credits may model CPU cost poorly across memory-access patterns and machines. Mitigate with bounded primitive operations, held-out stress tests, and conservative per-machine calibration.
- Deferred local updates can create implausible flow, uneven burning, or slow blast fronts. These artifacts are central experimental results, not cosmetic bugs to hide.
- Ready-ring saturation and per-cell coalescing change event semantics. Validate finite-demand recovery and explain which information is intentionally merged.
- Upload limits can keep CPU work bounded while visuals fall far behind. Measure this separately and tune upload fairness before adding simulation complexity.
- Fixed world size still entails substantial memory and initialization work. Validate storage estimates before allocating the largest world.
- GPU/OS behavior remains outside the structural proof. Maintain a clear boundary between invariants, performance observations, and unsupported claims.
- A naive traditional baseline can exaggerate the benefit. Use the same rules/resources and report throughput and completion time alongside responsiveness.

Open experiments after v0.1: **refine** and **substitute** overload responses, such as aggregated or proxy models for dormant or distant regions with lazy re-materialization (noting Chenney's caveat that per-object proxy upkeep still scales with object count), incremental progress reports carrying an error or completeness estimate so the scheduler can spend work where it most reduces visible error, value-driven selection over pending work, camera-aware priority with fairness, better logical-time semantics, aggregated dormant structures, adaptive credits with recorded schedules, worker threads, GPU compute, and eventually 3D. None are authorization to expand v0.1.

## 20. First tasks for the coding agent

1. Read this charter and repository instructions. Summarize the work invariant, overload semantics, and first milestone in a short implementation plan; proceed without waiting for approval of routine choices.
2. Inspect the installed Rust/toolchain and native graphics environment. Select compatible stable dependencies from official documentation, pin them, and create the minimal Cargo workspace.
3. Implement a small headless grid and explicit credit/capacity types. Write the resource table and work-quantum contracts before adding graphics.
4. Implement fixed pending storage, ready ring, duplicate suppression, recovery cursor, and a scheduler with charged selection and execution.
5. Add tests that repeatedly submit more demand than can fit. Prove credit limits, bounded storage, and recovery with tiny capacities before scaling up.
6. Add sand and a finite attenuating explosion rule; establish deterministic replay and conservation/conversion tests.
7. Build the smallest window/texture/metrics vertical slice. Connect bounded commands and uploads, then expand through the milestones.

Deliver source, tests, runnable commands, and measured evidence. When a feature conflicts with a bound, redesign the feature or defer it. The first success is a small world whose workload stays bounded under hostile input, with the cost of that choice plainly visible.
