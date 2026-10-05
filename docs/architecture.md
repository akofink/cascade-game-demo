# Simulation architecture

`cascade-sim` is a single-threaded, headless core with fixed-width cell IDs and dense row-major storage. Dimensions are validated before allocation. The world boundary is closed; out-of-range neighbors are absent. Logical chunks are 32 by 32 and expose deduplicated dirty state for bounded renderer drains.

## Work and storage contracts

| Resource | Bound | Overflow/semantics |
| --- | --- | --- |
| Cells | `width * height <= u32::MAX` | Reject invalid dimensions before allocation |
| Cell record | 2 bytes/cell (`Material` plus burn countdown) | Compact authoritative material and fire state |
| Pending state | 3 bytes/cell (`PendingCell`) | Flags for evaluation/blast/paint channels, blast energy, and O(1) command-slot index; no per-cell age array |
| Evaluation and blast ready rings | Caller-selected capacity per ring; defaults 32,768 each | `Option<Job>` fixed slots; duplicate suppression; pending state survives ring saturation |
| Command ring | Capacity 1 through 256; default 256 descriptors | `Option<Command>` fixed slots; same-target paint coalesces through per-cell slot index in O(1); full distinct submissions reject |
| Dirty chunk flags | One byte per logical 32x32 chunk | Deduplicated; drains at most 256 chunks per call |
| Reset and fixture cursors | One cursor plus descriptor/generation state | Clear or prepare one cell per charged quantum; no extra world-sized staging buffer |

At 4096x4096 with default ready/command capacities, cell storage is 32 MiB and pending storage is 48 MiB. Using compiled Rust layouts (`Cell` 2 bytes, `PendingCell` 3 bytes, `Option<Job>` 12 bytes, `Option<Command>` 8 bytes), the resource report sums cell, pending, ready-slot, command-slot, and chunk-flag payload bytes to exactly 84,690,944 bytes (80.77 MiB; allocator metadata and process overhead are excluded). This is below the charter's 256 MiB CPU storage limit. The fixed ready rings total 65,536 jobs; command capacity is capped at 256. User-selected ready capacities add their actual slot sizes to this report.

A job touches a constant number of cells and at most four cardinal neighbors. The maximum indivisible quantum charge is `C_max = 24` credits (`MAX_QUANTUM_COST`); minimum slice allowance is 25 including one selection probe. The bounded operation table is:

| Quantum | Charge | Maximum authoritative cell reads | Maximum authoritative cell writes | Pending/queue work | Scratch |
| --- | ---: | ---: | ---: | ---: | ---: |
| Selection/lane probe | 1 | 0 | 0 | 1 lane entry | 0 |
| Evaluate cell | 24 | 4 | 5 | at most 4 neighbor inspections and 21 fixed channel admissions in burning-wood case | 0 |
| Propagate blast | 24 | 4 | 5 | at most 4 neighbor inspections and 24 channel admissions | 0 |
| Recovery probe | 4 | 1 pending record | 0 | at most 2 ready admissions | 0 |
| Paint command | 12 | 1 target | 1 | target plus 4 wakes | 0 |
| Reset cell | 4 | 1 cell, 1 pending record | 1 | 1 pending-record clear and 1 dirty bit | 0 |
| Fixture cell | 12 | descriptor/coordinate state | 1 | 1 pending-record write, 1 dirty bit, at most 1 evaluation and 1 blast admission | 0 |

Each selection probe is charged, including an empty lane; a candidate is charged in full before execution. The ready-capacity constructor argument applies independently to evaluation and blast lanes. The scheduler rotates among evaluation, blast, command, and recovery lanes, retaining the lane cursor across slices. Recovery advances one cell per charged quantum and retries deferred channels when it finds a free ready slot. Reset temporarily reserves slices for its charged cursor and blocks new mutations. Fixture start schedules reset followed by preparation; preparation slices spend their allowance only on cursor quanta, report progress, and support cancellation. No public API fills fixture cells inline. If a candidate does not fit, its full work quantum is deferred and selection continues in the other bounded lanes. There is no loop that drains an unbounded queue in one slice. The ready-capacity constructor argument applies independently to the evaluation and blast lanes. The scheduler rotates among evaluation, blast, command, and recovery lanes, retaining the lane cursor across slices. Recovery advances one cell per charged quantum and retries deferred channels when it finds a free ready slot. Reset temporarily reserves slices for its charged cursor and blocks new mutations. Fixture start schedules reset followed by preparation; preparation slices spend their allowance only on cursor quanta, report progress, and support cancellation. No public API fills fixture cells inline. If a candidate does not fit, its full work quantum is deferred and selection continues in the other bounded lanes. There is no loop that drains an unbounded queue in one slice.

`World::step` reports allowance and charged credits and never intentionally spends beyond the allowance. Pending-channel counts are maintained incrementally. Oldest pending age is a conservative high-water estimate while any work remains, not an exact queue-age statistic. Calls to `submit` and explicit `trigger_blast` are bounded admission operations. Local wake fan-out is at most four jobs. Ready storage is fixed, and every cell can retain at most one evaluation, one saturating blast, and one coalesced paint channel.

## API and determinism

The renderer-facing surface consists of dimensions, `cell(x, y)`, chunk dimensions/dirty queries, bounded `drain_dirty_chunks` (hard cap 256), and `SliceMetrics`. `state_hash` includes world cells, pending channels, queue contents/cursors, fixture/reset cursors, generation, and budget. Hashing scans world storage, so it is an offline replay/benchmark operation, not a per-frame API. Replay determinism is scoped to the same executable/rules, dimensions, admitted command order, and slice budget. No wall clock, graphics, random source, or hash-map iteration enters simulation state.

Reset increments a checked generation and empties queue frontiers in O(1), then clears cell and pending storage one cell at a time through charged slices. Mutations are rejected while reset is in progress. Old jobs carry their prior generation and cannot mutate the new world. Reset exposes cursor progress and marks chunks dirty as it clears cells. Pending state is 3 bytes/cell (rather than the former 12-byte five-vector layout); at default dimensions cells plus pending state are 80 MiB.

## Public API work audit

Public mutation APIs only admit bounded work: `trigger_blast` coalesces one cell channel, `ignite` touches one cell and its fixed cardinal neighborhood, and `submit` uses the pending cell's direct command-ring slot rather than scanning the queue. Command admission is externally capped (64 commands per frame by default, hard maximum 64 for the deterministic disturbance stream). Dirty-chunk drains have a hard 256-chunk cap. Reset and fixture generation only advance through charged `World::step` quanta; starting a fixture schedules a reset cursor and then a preparation cursor, and cancellation drops only the cursor in O(1). Each fixture quantum writes one cell and can queue at most one local evaluation and one blast channel. `state_hash` is intentionally workload-dependent and must only be called for offline replay/benchmark verification, outside timed slices.

Credits bound accounted algorithmic work, not operating-system or hardware latency. Resource totals report array payload sizes, not allocator metadata or process RSS.
