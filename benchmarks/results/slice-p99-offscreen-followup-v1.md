# Heavy-fixture slice p99 offscreen follow-up v1

Status: task-branch diagnostic; not windowed acceptance evidence. The eight-cell windowed rerun remains deferred until the operator returns and visible GUI availability is independently confirmed.

Candidate profile: 1,000,000 credits/slice, selection probe cost 1, default focus share 5%, background minimum service 20%. Release app, 4096² world, 1920×1080 offscreen target, 60-second scripted capture. Offscreen runs measure CPU simulation slices and the app loop; they do not measure visible presentation, display timing, or scanout. These captures were not accepted as the quiet-machine matrix and must not replace the published windowed results.

## Current candidate slice p99

| Fixture | Policy | Slice p99 ms | Frame p99 ms | Frames >33.3 ms | Result |
|---|---|---:|---:|---:|---|
| sand-release | bounded-focus | 4.076 | 19.779 | 1 | Focus target slightly missed |
| sand-release | bounded-fifo | 4.206 | 19.745 | 0 | FIFO target missed |
| reservoir-breach | bounded-focus | 3.972 | 19.667 | 2 | Focus target met in this diagnostic |
| reservoir-breach | bounded-fifo | 4.439 | 19.648 | 0 | FIFO target missed |
| burning-forest | bounded-focus | 3.913 | 19.731 | 0 | Focus target met in this diagnostic |
| burning-forest | bounded-fifo | 4.315 | 19.728 | 1 | FIFO target missed |
| mixed-overload | bounded-focus | 3.906 | 19.695 | 0 | Focus target met in this diagnostic |
| mixed-overload | bounded-fifo | 4.300 | 19.618 | 0 | FIFO target missed |

Three additional burning-forest bounded-focus diagnostics measured slice p99 3.861, 4.083, and 4.028 ms. Thus 2/3 repetitions were at or below 4 ms; the single matrix capture was 3.913 ms. The candidate does not establish reliable p99 ≤4 ms across focus cells, and none of the four FIFO cells meets the target. Frame p99 stayed below 20 ms in these diagnostics, but individual captures recorded one or two intervals above 33.3 ms; those outliers remain disclosed.

The previous accepted offscreen v1 suite, on revision `4a2f80e`, had three-run focus p99 means 5.031 ms sand-release, 5.047 ms reservoir-breach, 5.072 ms burning-forest, and 5.112 ms mixed-overload. The corresponding FIFO means were 4.265, 4.224, 4.204, and 4.197 ms. This is a historical cross-revision comparison, not a paired experiment. It supports a focus-share improvement, but not a causal claim for the small FIFO differences.

## Action and recovery checks

The 4096-slice mixed-overload action comparison at focus share 5% measured bounded-focus first-effect p95 1.207 ms and local-settle p95 1.239 ms. Settle p99 was 11.555 ms and max 51.869 ms. This remains a simulator/headless proxy, not visible-action timing.

A same-source, 4096² bounded mixed-overload burst comparison used 1,000,000 credits/slice, 72,000 slices, and 1,200 seeded disturbances at up to 64 per slice:

| Focus share | Completed work quanta | Final pending channels | Stable window |
|---:|---:|---:|---|
| 50% | 3,077,305,712 | 267,243 | none |
| 5% | 3,077,350,548 | 120,560 | none |

The 5% candidate's final pending count is about 55% lower, with effectively unchanged completed work. Two 5% runs produced the same final hash and work counts. They are diagnostic, not accepted timing runs: each run's one-minute post-run load exceeded the <3 gate (3.02 and 3.09), so recovery timing is not published as accepted evidence. The burst remains unresolved because neither setting reached the 60-slice stable window. No evidence indicates recovery regressed under the candidate.

## FIFO selection investigation

Lane attribution places selection work near the evaluation path. A task-branch source experiment caches focus-ring availability and skips a redundant empty focus-ring pop in FIFO/background selection; the measured selector interval now includes this queue-pop choice. One instrumented burning-forest FIFO capture measured selection p99 2.733 ms and 37.8 ns/probe on average. Three uninstrumented captures measured total slice p99 4.139, 4.121, and 4.189 ms (mean 4.150 ms). This is only a small change against historical FIFO captures and is not a paired or decisive improvement. A selection cost-2 experiment worsened burning-forest FIFO p99 to 4.757 ms and was reverted; selection cost remains 1. Bit-rotation and lane-table lookup experiments likewise did not establish a reliable gain and were reverted.

The selector change does not resolve any FIFO cell's 4 ms miss. Keep investigating FIFO source costs before treating this task as complete.

## Verification and next steps

The retained candidate preserves the 1M-credit allowance and selection cost 1. Workspace format, Clippy, and tests must pass before publishing. The windowed eight-cell acceptance rerun is still required when the operator returns. Offscreen measurements here are diagnostic only.
