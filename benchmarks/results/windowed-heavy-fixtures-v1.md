# Native windowed heavy-fixture follow-up v1

Status: complete; 24/24 accepted captures for the eight requested bounded cells; 28 rejected attempts (all load-gate rejections). This targeted follow-up does not rerun or replace the all-fixture matrix.

App source revision: `2c0b1751b21c610217e448e243494573da6246a9`  
Suite runner revision: `2c0b1751b21c610217e448e243494573da6246a9`  
Suite started: 2026-10-07T18:20:55-04:00  
Reference host: `akmac`, MacBook Air (Mac14,2), Apple M2/8 CPU cores/16 GB RAM/integrated GPU, macOS 27.0.1; Rust `rustc 1.99.0 (b940084d7 2026-09-28)`.
Capture: release app, 4096² world, 1920×1080, 60 Hz target, profile `m2-16gb-v3`; Cargo.lock SHA-256 `d064050f2b9044bb0eecfb2b7ffee1b18644dd676e192f7b7d0b0008d42db06b`.
Power mode, thermal state, background activity, and physical display refresh are not controlled.

Each of the eight requested bounded fixture/policy cells requires three accepted 60-second captures. The run covers sand-release, reservoir-breach, burning-forest, and mixed-overload under bounded-focus and bounded-fifo only; no other fixture or traditional-policy captures were run. A run is accepted only when one-minute load is below 3 both immediately before and after the process, the app exits successfully with `SMOKE_RESULT ok` after its 60-second capture timer, reports the requested fixture/policy at 1920×1080 with nonzero frame samples, has no interval drops, and has complete slice telemetry. Frame counts are actual measured loop intervals, not an assumed 60 Hz count. Rejected attempts are retained below and never contribute to accepted percentiles.

## Accepted captures

| Fixture | Policy | Rep | Load before/after | Frames | Frame p99 ms | Max ms | >33.3 ms | Slice p99 ms | Slice max ms |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| sand-release | bounded-focus | 1 | 1.91/2.74 | 3599 | 18.482 | 34.560 | 1 | 5.133 | 5.613 |
| sand-release | bounded-fifo | 1 | 2.74/2.84 | 3599 | 20.415 | 33.771 | 1 | 4.221 | 5.644 |
| reservoir-breach | bounded-focus | 1 | 3.00/2.73 | 3596 | 19.059 | 33.310 | 0 | 2.450 | 4.898 |
| reservoir-breach | bounded-fifo | 1 | 2.73/2.35 | 3598 | 18.521 | 31.127 | 0 | 4.066 | 5.858 |
| burning-forest | bounded-focus | 1 | 2.35/2.05 | 3597 | 19.001 | 32.741 | 0 | 4.650 | 11.204 |
| burning-forest | bounded-fifo | 1 | 2.05/1.88 | 3598 | 19.302 | 32.311 | 0 | 5.077 | 12.559 |
| mixed-overload | bounded-focus | 1 | 1.88/2.08 | 3598 | 18.412 | 32.599 | 0 | 4.473 | 4.938 |
| mixed-overload | bounded-fifo | 1 | 2.08/2.04 | 3599 | 18.921 | 32.735 | 0 | 4.245 | 6.481 |
| sand-release | bounded-fifo | 2 | 2.04/2.38 | 3599 | 20.316 | 32.536 | 0 | 4.193 | 5.429 |
| sand-release | bounded-focus | 2 | 2.38/2.07 | 3599 | 18.805 | 32.873 | 0 | 5.433 | 5.890 |
| reservoir-breach | bounded-fifo | 2 | 2.07/2.40 | 3599 | 18.634 | 32.916 | 0 | 4.160 | 4.920 |
| reservoir-breach | bounded-focus | 2 | 2.40/2.27 | 3596 | 18.859 | 31.106 | 0 | 2.467 | 3.420 |
| burning-forest | bounded-fifo | 2 | 2.27/2.65 | 3598 | 19.618 | 31.694 | 0 | 5.095 | 11.239 |
| burning-forest | bounded-focus | 2 | 2.65/2.54 | 3597 | 19.022 | 32.947 | 0 | 4.664 | 12.560 |
| mixed-overload | bounded-fifo | 2 | 2.54/2.57 | 3598 | 18.830 | 34.860 | 2 | 4.193 | 5.170 |
| mixed-overload | bounded-focus | 2 | 2.57/2.38 | 3598 | 18.472 | 33.665 | 1 | 4.572 | 5.217 |
| sand-release | bounded-focus | 3 | 2.38/2.26 | 3599 | 18.690 | 34.801 | 1 | 5.140 | 5.861 |
| sand-release | bounded-fifo | 3 | 2.26/2.53 | 3599 | 20.221 | 32.146 | 0 | 4.099 | 5.083 |
| reservoir-breach | bounded-focus | 3 | 2.53/2.75 | 3599 | 18.665 | 30.632 | 0 | 2.418 | 3.813 |
| reservoir-breach | bounded-fifo | 3 | 2.75/2.38 | 3599 | 18.434 | 32.409 | 0 | 4.195 | 5.357 |
| burning-forest | bounded-focus | 3 | 2.38/2.80 | 3596 | 19.142 | 33.938 | 1 | 4.955 | 11.708 |
| burning-forest | bounded-fifo | 3 | 2.80/2.22 | 3598 | 19.321 | 33.970 | 1 | 4.960 | 12.618 |
| mixed-overload | bounded-focus | 3 | 2.22/2.22 | 3599 | 18.512 | 32.602 | 0 | 4.523 | 5.947 |
| mixed-overload | bounded-fifo | 3 | 2.22/2.30 | 3599 | 18.865 | 30.529 | 0 | 4.288 | 6.259 |

## Section 15 checklist rows

| Fixture | Policy | Accepted runs | Frame p99 ≤20 ms | Slice p99 ≤4 ms | >33.3 ms count / max disclosed |
|---|---|---:|---|---|---|
| sand-release | bounded-focus | 3/3 | met | unmet | reported |
| sand-release | bounded-fifo | 3/3 | unmet | unmet | reported |
| reservoir-breach | bounded-focus | 3/3 | met | met | reported |
| reservoir-breach | bounded-fifo | 3/3 | met | unmet | reported |
| burning-forest | bounded-focus | 3/3 | met | unmet | reported |
| burning-forest | bounded-fifo | 3/3 | met | unmet | reported |
| mixed-overload | bounded-focus | 3/3 | met | unmet | reported |
| mixed-overload | bounded-fifo | 3/3 | met | unmet | reported |

## Interaction feedback samples

These CPU-side event/action-to-visible-upload p95 values use the app's documented method. Counts below 30 are descriptive only; only mixed-overload uses the scripted interaction stream in this fixture subset.

| Fixture | Policy | Rep | Camera n / p95 ms | Paint n / p95 ms | Ignite n / p95 ms | Detonate n / p95 ms |
|---|---|---:|---:|---:|---:|---:|
| sand-release | bounded-focus | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | bounded-fifo | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-focus | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-fifo | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-focus | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-fifo | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| mixed-overload | bounded-focus | 1 | 240 / 17.94 | 240 / 18.05 | 240 / 18.18 | 240 / 17.80 |
| mixed-overload | bounded-fifo | 1 | 240 / 17.82 | 240 / 17.74 | 240 / 17.81 | 240 / 17.97 |
| sand-release | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| mixed-overload | bounded-fifo | 2 | 240 / 17.80 | 240 / 17.97 | 240 / 17.91 | 240 / 17.88 |
| mixed-overload | bounded-focus | 2 | 240 / 18.05 | 240 / 18.09 | 240 / 18.00 | 240 / 18.22 |
| sand-release | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| mixed-overload | bounded-focus | 3 | 240 / 17.97 | 240 / 18.03 | 240 / 18.17 | 240 / 18.13 |
| mixed-overload | bounded-fifo | 3 | 240 / 18.03 | 240 / 18.11 | 240 / 17.85 | 240 / 17.91 |

## Rejected attempts

| Fixture | Policy | Rep | Attempt | Load before/after | Reason | Log |
|---|---|---:|---:|---:|---|---|
| sand-release | bounded-focus | 1 | 1 | 14.66/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 1 | 2 | 13.57/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 1 | 3 | 12.56/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 1 | 4 | 11.63/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 1 | 5 | 10.86/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 1 | 6 | 10.07/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 1 | 7 | 9.35/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 1 | 8 | 8.84/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 1 | 2.84/5.21 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/reservoir-breach-bounded-focus-rep1-attempt1.log` |
| reservoir-breach | bounded-focus | 1 | 2 | 4.95/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 3 | 4.64/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 4 | 4.35/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 5 | 4.08/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 6 | 3.83/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 7 | 3.76/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 8 | 3.54/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 9 | 3.34/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 10 | 3.15/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 11 | 2.98/3.61 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/reservoir-breach-bounded-focus-rep1-attempt11.log` |
| reservoir-breach | bounded-focus | 1 | 12 | 3.61/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 13 | 3.40/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 14 | 3.21/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 15 | 3.27/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 16 | 3.17/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 17 | 3.00/3.19 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/reservoir-breach-bounded-focus-rep1-attempt17.log` |
| reservoir-breach | bounded-focus | 1 | 18 | 3.19/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 19 | 3.09/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-focus | 1 | 20 | 3.08/n/a | pre-run one-minute load not below 3 | `-` |

## Interpretation and limits

The bounded FIFO slice p99 misses are reported without qualification: all four FIFO cells exceed 4 ms (4.099–4.288 ms across their three captures). Seven of eight cells meet the frame p99 ≤20 ms target; sand-release/bounded-fifo does not (20.221–20.415 ms). Only reservoir-breach/bounded-focus meets the slice p99 ≤4 ms target (2.418–2.467 ms). The frame >33.3 ms counts and maxima are shown for every accepted capture above.

Frame intervals are native windowed event-loop intervals; they include presentation pacing and are not GPU duration. Slice CPU values are monotonic wall-time samples around the app's simulation dispatch, not thread CPU time. Fixture captures do not add external disturbances or player actions, except mixed-overload, which uses the app's scripted sustained-overload action stream to exercise focus latency. The load gate does not exclude all OS/driver interference.

Raw per-attempt logs are retained locally under `benchmarks/tmp/` and are not committed.
