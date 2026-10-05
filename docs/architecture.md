# Simulation architecture

`cascade-sim` is a single-threaded, headless core with fixed-width cell IDs and dense row-major storage. Dimensions are validated before allocation. The world boundary is closed; out-of-range neighbors do not exist. Logical chunks are 32 by 32 and expose deduplicated dirty state for bounded renderer drains.

## Work and storage contracts

| Resource | Bound | Overflow/semantics |
| --- | --- | --- |
| Cells | `width * height <= u32::MAX`; total payload at most 256 MiB | Reject dimensions before allocation |
| Cell record | 2 bytes/cell (`Material` and burn countdown) | Compact authoritative state |
| Pending state | 3 bytes/cell (`PendingCell`) | Evaluation/blast/paint flags, blast energy, direct paint-slot index |
| Traditional frontier snapshot | 1 byte/cell | Packs captured evaluation bit and blast energy; allocated before simulation, no per-slice growth |
| Background evaluation and blast rings | Capacity 1 through 32,768 each; 32,768 each by default | Fixed `Option<Job>` slots; duplicate suppression; pending state survives saturation |
| Focus evaluation and blast rings | Same bounded capacity per lane as background rings | Fixed `Option<Job>` slots; overflow falls back to background lanes or pending state |
| Focus regions | 8 fixed chunk rectangles | Slice-count expiry; O(8) membership check; oldest-expiring region replaced when full |
| Action records | 256 fixed records | Ring overwrites oldest; stores admission, action-applied, first-effect, and local-settle slice indices |
| Command ring | Capacity 1 through 256; 256 by default | Fixed slots; same-cell paint coalesces in O(1); distinct full commands reject |
| Dirty-chunk flags | One byte per logical 32x32 chunk | Deduplicated; each drain inspects at most 256 chunks |
| Reset and fixture cursors | Constant-size cursor and descriptor state | One cell per charged quantum; no world-sized staging buffer |

At 4096x4096 using default queue capacities, the compiled array payload totals exactly 102,276,096 bytes (97.53 MiB), including cells (33,554,432 bytes), pending state (50,331,648 bytes), captured frontier (16,777,216 bytes), four ready-ring arrays (1,572,864 bytes), command slots (2,048 bytes), chunk flags (16,384 bytes), and the fixed action-record ring plus fixed focus metadata. Layouts are `Cell = 2`, `PendingCell = 3`, `Option<Job> = 12`, and `Option<Command> = 8` bytes. This is below the 256 MiB CPU storage limit. Allocator metadata, fixed struct fields, and process RSS are excluded. Constructor validation rejects any configured arrays whose payload would exceed the limit.

## Quantum contracts and accounting

The maximum indivisible quantum cost is `C_max = 24` credits (`MAX_QUANTUM_COST`). The minimum slice allowance is 25 credits, including a selection probe. Charges conservatively cover the worst fixed path in each quantum. Each rule reads at most four neighbors, uses no scratch allocation, and performs bounded writes/admissions:

| Quantum | Charge | Maximum authoritative reads | Maximum authoritative writes | Maximum pending/queue work |
| --- | ---: | ---: | ---: | ---: |
| Selection/lane probe | 1 | One lane entry | 0 | One lane probe |
| Evaluate cell | 24 | 4 cells | 5 cells | Up to 21 fixed channel admissions in the burning-wood case |
| Propagate blast | 24 | 4 cells | 5 cells | Up to 24 bounded local channel admissions |
| Recovery probe / frontier scan record | 4 | 1 pending or captured-snapshot record | Clear pending flags; write one snapshot byte | Traditional scans do not admit work; bounded recovery may admit up to 2 ready jobs |
| Paint command | 12 | 1 target | 1 cell | Target plus 4 wakes |
| Reset cell | 4 | 1 cell and pending record | 1 cell | One pending clear and one dirty bit |
| Fixture cell | 12 | Fixed descriptor/coordinate state | 1 cell | One pending write, one dirty bit, at most one evaluation and one blast admission |

Bounded mode charges each selection probe and complete quantum before executing it. Focus-enabled mode adds evaluation and blast lanes with the same fixed capacities as their background counterparts. Eight fixed rectangular focus regions are stored as chunk-coordinate bounds with slice-count expiry. Target actions install a 3x3-chunk focus region when admitted for execution; viewport focus is a bounded command over chunk coordinates. Focus membership checks at most eight region records. Newly focused pending cells are promoted incrementally by the charged recovery cursor; an existing background entry may remain and later be charged as stale. A focus-ring overflow tries the corresponding background ring, then leaves work in per-cell pending state. The default profile gives focus 50% of scheduler selections while non-empty and guarantees at least 20% background service; the background receives the remainder of focus-excluded selections. Setting focus disabled routes new work to FIFO lanes and drains already queued focus entries there. Bounded mode retains its lane cursor across slices. Recovery inspects one cell per quantum and admits deferred pending channels when ring slots become available. A candidate that cannot fit waits for a later slice. Accepted work is coalesced, retained in per-cell pending state, or explicitly rejected at admission; it is never silently evicted.

Traditional mode is selected when a fixture starts and uses the same world, rules, capacities, queues, and coalescing. The ready rings are an admission accelerator, not the authoritative frontier. At slice entry the policy scans every per-cell pending record, charging one selection plus one recovery/frontier-scan cost per cell, snapshots both pending channels into the preallocated one-byte-per-cell buffer, and clears the captured channel state and queued ring frontier. A second full pass reads and clears the snapshot, charging the same per-cell scan cost; each captured evaluation/blast also pays its selection and rule cost. `SliceMetrics.recoveries` includes one record probe per cell in each pass. It then processes only the command count captured at entry. All captured cell work is uncapped. The snapshot pass completes before any captured work executes, so newly emitted work is queued for the next slice, never accidentally absorbed by a later scan position. These two finite full-storage passes can make even an idle traditional slice expensive; both passes are included in charged work and wall time as part of this deliberately batch-all baseline. It does not recursively drain newly generated work to quiescence. `SliceMetrics.allowed` continues to report the configured allowance and `charged` may exceed it in traditional mode. Reset and fixture preparation remain charged cursor work in either policy.

## Fixtures and commands

`scenarios/*.toml` contains seeded, versioned descriptors for the eight charter scenarios. `ScenarioDescriptor` binds each descriptor to its checked-in seed/version and capacity profile. Starting a fixture selects its policy, schedules a generation-safe reset, then advances one fixture-cell cursor quantum per charged `World::step`. Preparation reports progress and can be cancelled in O(1); cancellation leaves a partial world, and starting another fixture resets it first. External mutations are rejected during reset or preparation.

Paint coalescing uses a per-cell command-ring slot index, so admission never scans the command queue. The queue is capped at 256. `DisturbanceCommandStream` is deterministic from its seed and sequence, has a finite configurable total, defaults to 8 commands per call, and enforces a hard maximum of 64 per call. It reports attempted, accepted, coalesced, and rejected requests.

## Public API work audit

No public simulation API performs world-content-scaled mutation inline. `trigger_blast` merges one cell channel; `ignite` updates one cell and a fixed cardinal neighborhood; `submit` does O(1) admission/coalescing; dirty-chunk drains have a hard 256-item cap. Reset and fixture generation advance only through scheduler quanta. Disturbance admission is separately capped. `state_hash` is an offline replay/benchmark operation that scans the world and must remain outside timed slices.

The renderer-facing API remains dimensions, `cell(x, y)`, chunk dimensions/dirty queries, bounded dirty drains, `SliceMetrics`, and additive burning/reset/fixture progress fields. `Command::Ignite`, `Command::Detonate`, and `Command::FocusViewport` provide admitted, replayable action/focus entry points. `World::action_records` exposes the bounded action-latency ring, and `World::active_focus_regions` returns at most eight chunk rectangles with remaining slice lifetimes; per-slice service counts distinguish focused and background evaluations/blasts. No graphics, wall clock, or random source enters the simulation crate.

## Reset, time, and replay

Reset increments a checked generation immediately, clears queue frontiers in O(1), blocks mutations, then clears each cell and pending record through charged cursor quanta. Old jobs carry their generation and cannot mutate the new world. Chunks are marked dirty as reset proceeds.

Timers advance on service, not wall time. Local updates are asynchronous, and regions can progress at different rates under load. This is not uniform time dilation. `state_hash` includes cells, pending state, queues/cursors, policy, fixture/reset cursors, generation, and budget. Replays are scoped to identical executable/rules, dimensions, fixture version/seed, policy, capacities, admitted commands, and slice schedule. Hash computation is outside measured slices.

## Headless benchmark

`cascade-bench` prepares a fresh descriptor before timing and runs a fixed number of slices under either policy. Optional warm-up advances a prepared fixture, then the runner re-prepares the descriptor so measured policy work starts from the same fixture state. CSV/JSON include credits allowed/used, selection probes, executed quanta, cumulative work, ready/backlog counts, deterministic disturbance counters, slice elapsed nanoseconds, measured elapsed wall time and completion. Preparation and warm-up durations are timed separately. Slice time uses a host monotonic clock around the CPU-only step and can include operating-system preemption; it is not a thread CPU clock. The replay hash covers world state, descriptor, and future disturbance-stream state. Tests include deterministic replay and an order-insensitive quiet-fixture comparison with independent paint requests across both policies.

Credits bound accounted algorithmic work, not operating-system or hardware latency. CPU times are observations, not deadline guarantees.
