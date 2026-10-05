# Simulation architecture

`cascade-sim` is a single-threaded, headless core with fixed-width cell IDs and dense row-major storage. Dimensions are validated before allocation. The world boundary is closed; out-of-range neighbors are absent. Logical chunks are 32 by 32 and expose deduplicated dirty state for bounded renderer drains.

## Work and storage contracts

| Resource | Bound | Overflow/semantics |
| --- | --- | --- |
| Cells | `width * height <= u32::MAX` | Reject invalid dimensions before allocation |
| Cell record | 2 bytes/cell (`Material` plus burn countdown) | Compact authoritative material and fire state |
| Pending state | 2 bytes/cell (`PendingCell`) | Two flag bytes encode evaluation-pending, evaluation-queued, blast-queued; one blast-energy byte. No per-cell age array |
| Evaluation and blast ready rings | Caller-selected capacity per ring; defaults 32,768 each | `Option<Job>` fixed slots; duplicate suppression; pending state survives ring saturation |
| Command ring | Caller-selected capacity; default 256 descriptors | `Option<Command>` fixed slots; same-target paint coalesces, distinct full submissions reject |
| Dirty chunk flags | One byte per logical 32x32 chunk | Deduplicated; bounded round-robin drain |
| Reset cursor | One `usize` and one generation | Clears one cell per charged quantum; no extra world-sized reset buffer |

At 4096x4096 with default ready/command capacities, cell storage is 32 MiB and pending storage is 32 MiB. Using the compiled Rust layouts (`size_of::<Cell>() = 2`, `size_of::<PendingCell>() = 2`, `size_of::<Option<Job>>() = 12`, and `size_of::<Option<Command>>() = 8`), the resource report sums cell, pending, ready-slot, command-slot, and chunk-flag payload bytes to exactly 67,913,728 bytes (64.77 MiB; allocator metadata and process overhead are excluded). This is below the charter's 256 MiB CPU storage limit. The fixed ready rings total 65,536 jobs. User-selected capacities add their actual slot sizes to this report.

A job touches a constant number of cells and at most four cardinal neighbors. The conservative credit table is: selection 1, evaluate 12, blast 16, recovery probe 4, paint command 6, reset-cell clear 4. Each selection probe is charged, including an empty lane; a candidate is charged in full before execution. Every enabled cost fits the minimum accepted budget of 17 credits. The ready-capacity constructor argument applies independently to the evaluation and blast lanes. The current scheduler rotates among evaluation, blast, command, and recovery lanes, retaining the lane cursor across slices. Recovery advances one cell per charged quantum and retries deferred channels when it finds a free ready slot. Reset temporarily reserves slices for its charged cursor and blocks new mutations. If a candidate does not fit, its full work quantum is deferred and selection continues in the other bounded lanes. There is no loop that drains an unbounded queue in one slice.

`World::step` reports allowance and charged credits and never intentionally spends beyond the allowance. Pending-channel counts are maintained incrementally. Oldest pending age is a conservative high-water estimate while any work remains, not an exact queue-age statistic. Calls to `submit` and explicit `trigger_blast` are bounded admission operations. Local wake fan-out is at most four jobs. Ready storage is fixed, and every cell can retain at most one evaluation and one saturating blast channel.

## API and determinism

The renderer-facing surface consists of dimensions, `cell(x, y)`, chunk dimensions/dirty queries, bounded `drain_dirty_chunks`, and `SliceMetrics`. `state_hash` includes world cells, pending channels, queue contents and cursors, generation, and budget. Replay determinism is scoped to the same executable/rules, dimensions, admitted command order, and slice budget. No wall clock, graphics, random source, or hash-map iteration enters simulation state.

Reset increments a checked generation and empties queue frontiers in O(1), then clears cell and pending storage one cell at a time through charged slices. Mutations are rejected while reset is in progress. Old jobs carry their prior generation and cannot mutate the new world. Reset exposes cursor progress and marks chunks dirty as it clears cells. The per-cell pending state is 2 bytes, rather than the former 12-byte five-vector layout; at default dimensions cells plus pending state are 64 MiB.

Credits bound accounted algorithmic work, not operating-system or hardware latency. Resource totals report array payload sizes, not allocator metadata or process RSS.
