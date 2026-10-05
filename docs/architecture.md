# Simulation architecture

`cascade-sim` is a single-threaded, headless core with fixed-width cell IDs and dense row-major storage. Dimensions are validated before allocation. The world boundary is closed; out-of-range neighbors are absent. Logical chunks are 32 by 32 and expose deduplicated dirty state for bounded renderer drains.

## Work and storage contracts

| Resource | Bound | Overflow/semantics |
| --- | --- | --- |
| Cells | `width * height <= u32::MAX` | Reject invalid dimensions before allocation |
| Cell record | `size_of::<Cell>()` bytes per cell | Material and compact blast state only |
| Pending state | 12 bytes per cell in current layout | One reevaluation flag, blast energy, queue flags, pending-since slice |
| Evaluation and blast ready rings | Caller-selected capacity per ring; 32,768 each gives 65,536 total ready jobs | Suppress duplicates; leave accepted demand pending for recovery |
| Command ring | Caller-selected capacity; default 256 descriptors | Coalesce same-target paint; reject other full-queue submissions |
| Dirty chunk flags | One bool per logical chunk | Deduplicated; drain probes at most the requested number of chunks |

A job touches a constant number of cells and at most four cardinal neighbors. The conservative credit table is: selection 1, evaluate 8, blast 12, recovery probe 4, paint command 6. Each selection probe is charged, including an empty lane; a candidate is charged in full before execution. Every enabled cost fits the minimum accepted budget of 13 credits. The ready-capacity constructor argument applies independently to the evaluation and blast lanes. The current scheduler rotates among evaluation, blast, command, and recovery lanes, retaining the lane cursor across slices. Recovery advances one cell per charged quantum and retries deferred channels when it finds a free ready slot. If a candidate does not fit, its full work quantum is deferred and selection continues in the other bounded lanes. There is no loop that drains an unbounded queue in one slice.

`World::step` reports allowance and charged credits and never intentionally spends beyond the allowance. Pending-channel counts are maintained incrementally. Oldest pending age is a conservative high-water estimate while any work remains, not an exact queue-age statistic. Calls to `submit` and explicit `trigger_blast` are bounded admission operations. Local wake fan-out is at most four jobs. Ready storage is fixed, and every cell can retain at most one evaluation and one saturating blast channel.

## API and determinism

The renderer-facing surface consists of dimensions, `cell(x, y)`, chunk dimensions/dirty queries, bounded `drain_dirty_chunks`, and `SliceMetrics`. `state_hash` includes world cells, pending channels, queue contents and cursors, generation, and budget. Replay determinism is scoped to the same executable/rules, dimensions, admitted command order, and slice budget. No wall clock, graphics, random source, or hash-map iteration enters simulation state.

Reset increments a checked generation, empties deferred queues and resets state; jobs carry their generation so a future incremental reset implementation can reject stale work. Reset currently clears dense vectors synchronously, so it is intended for small milestone fixtures and is not a bounded interactive reset operation.

This milestone does not claim a wall-clock deadline or an allocation-free reset. Credits bound accounted algorithmic work, not operating-system or hardware latency.
