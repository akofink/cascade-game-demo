# Native windowed heavy-fixture regression follow-up v2

Status: data complete for the requested subset, 24/24 accepted captures across eight bounded heavy-fixture cells; 58 load-gate-rejected attempts. Performance outcome is unresolved: the frame-p99 target is met in only 2/8 cells, and no-worse presentation tails are not established. This is not the 72-capture all-fixture/policy matrix; the other fixtures and traditional policy were not run.

App source revision: `431118a57a3883121e9d4b77b4f23e6d91df83af`; suite runner revision: `431118a57a3883121e9d4b77b4f23e6d91df83af`; suite started: 2026-10-09T21:04:54-04:00.
Reference host: `akmac`, MacBook Air (Mac14,2), Apple M2/8 CPU cores/16 GB RAM/integrated GPU, macOS 27.0.1; Rust `rustc 1.99.0 (b940084d7 2026-09-28)`.
Capture: release app, 4096² world, 1920×1080, 60 Hz target, profile `m2-16gb-v3`; Cargo.lock SHA-256 `d064050f2b9044bb0eecfb2b7ffee1b18644dd676e192f7b7d0b0008d42db06b`.
Command: `python3 scripts/native_acceptance_suite.py --binary target/release/cascade-app --output benchmarks/results/windowed-heavy-fixtures-v2.md --fixtures sand-release reservoir-breach burning-forest mixed-overload --policies bounded-focus bounded-fifo --repetitions 3 --seconds 60`.
Power mode, thermal state, background activity, and physical display refresh are not controlled.

Each fixture/policy cell requires three accepted 60-second captures. A run is accepted only when one-minute load is below 3 both immediately before and after the process, the app exits successfully with `SMOKE_RESULT ok` after its 60-second capture timer, reports the requested fixture/policy at 1920×1080 with nonzero frame samples, has no interval drops, and has complete slice telemetry. Frame counts are actual measured loop intervals, not an assumed 60 Hz count. Rejected attempts are retained below and never contribute to accepted percentiles.

## Accepted captures

| Fixture | Policy | Rep | Load before/after | Frames | Frame p99 ms | Max ms | >33.3 ms | Slice p99 ms | Slice max ms |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| sand-release | bounded-focus | 1 | 2.96/2.71 | 3598 | 18.708 | 35.465 | 1 | 5.149 | 6.853 |
| sand-release | bounded-fifo | 1 | 2.99/2.67 | 3598 | 19.497 | 30.808 | 0 | 4.288 | 5.378 |
| reservoir-breach | bounded-focus | 1 | 2.67/2.36 | 3596 | 18.653 | 32.162 | 0 | 2.488 | 2.764 |
| reservoir-breach | bounded-fifo | 1 | 2.36/2.47 | 3597 | 18.783 | 33.886 | 1 | 4.275 | 5.864 |
| burning-forest | bounded-focus | 1 | 2.90/2.48 | 3597 | 18.784 | 36.439 | 2 | 4.695 | 12.489 |
| burning-forest | bounded-fifo | 1 | 2.93/2.41 | 3599 | 18.944 | 28.831 | 0 | 4.997 | 14.125 |
| mixed-overload | bounded-focus | 1 | 2.41/2.31 | 3597 | 18.695 | 35.484 | 2 | 4.549 | 6.188 |
| mixed-overload | bounded-fifo | 1 | 2.31/2.72 | 3597 | 18.642 | 35.446 | 1 | 4.277 | 5.532 |
| sand-release | bounded-fifo | 2 | 2.72/2.24 | 3595 | 19.562 | 33.941 | 3 | 4.324 | 6.372 |
| sand-release | bounded-focus | 2 | 2.24/2.16 | 3597 | 18.690 | 34.608 | 2 | 5.084 | 5.825 |
| reservoir-breach | bounded-fifo | 2 | 2.16/2.71 | 3598 | 18.865 | 33.857 | 2 | 4.466 | 5.598 |
| reservoir-breach | bounded-focus | 2 | 2.71/2.37 | 3596 | 18.776 | 32.501 | 0 | 2.472 | 4.362 |
| burning-forest | bounded-fifo | 2 | 2.37/2.35 | 3596 | 19.114 | 35.309 | 2 | 5.040 | 12.281 |
| burning-forest | bounded-focus | 2 | 2.35/2.33 | 3594 | 18.740 | 33.757 | 2 | 4.834 | 15.303 |
| mixed-overload | bounded-fifo | 2 | 2.88/2.78 | 3562 | 19.293 | 329.073 | 14 | 4.418 | 5.266 |
| mixed-overload | bounded-focus | 2 | 2.78/2.46 | 3539 | 34.788 | 37.730 | 57 | 5.057 | 5.900 |
| sand-release | bounded-focus | 3 | 2.46/2.32 | 3524 | 34.290 | 37.505 | 57 | 5.683 | 8.404 |
| sand-release | bounded-fifo | 3 | 2.32/2.21 | 3571 | 20.953 | 36.845 | 18 | 4.363 | 5.931 |
| reservoir-breach | bounded-focus | 3 | 2.21/2.26 | 3600 | 18.265 | 20.190 | 0 | 1.956 | 4.102 |
| reservoir-breach | bounded-fifo | 3 | 2.87/2.28 | 3564 | 30.390 | 35.946 | 23 | 4.643 | 5.893 |
| burning-forest | bounded-focus | 3 | 2.28/2.51 | 3549 | 33.754 | 36.719 | 43 | 5.140 | 12.882 |
| burning-forest | bounded-fifo | 3 | 2.51/2.51 | 3557 | 33.132 | 38.810 | 34 | 5.602 | 13.127 |
| mixed-overload | bounded-focus | 3 | 2.47/2.74 | 3542 | 34.746 | 37.726 | 53 | 4.932 | 7.540 |
| mixed-overload | bounded-fifo | 3 | 2.99/2.78 | 3579 | 19.033 | 35.512 | 13 | 4.483 | 5.838 |

## Diagnosis and source change

Pre-change windowed diagnostics reproduced the sand-release/bounded-FIFO frame-p99 miss at 20.442 and 20.178 ms. One accepted diagnostic run measured surface-acquisition p99 14.623 ms, preparation p99 1.131 ms, and present-call p99 0.110 ms. Delaying acquisition until after CPU work was rejected: its accepted frame p99 was 33.481 ms with 43 intervals above 33.3 ms. The source keeps early acquisition and batches dirty-chunk bytes through a reusable CPU staging allocation and one queue buffer write, then encodes per-chunk buffer-to-texture copies in the frame command buffer. The 240 scheduled + 16 priority-copy cap, simulator, profile values, selection cost, and focus share are unchanged.

A focused instrumented post-batch capture measured upload CPU p99 2.720 ms and frame p99 19.638 ms. That was one diagnostic run on the upload-change revision, not the final acceptance build. The final clean-source matrix removed per-frame diagnostic timing probes before acceptance. In that matrix, sand-release/bounded-FIFO frame p99s were 19.497, 19.562, and 20.953 ms, versus 20.415, 20.316, and 20.221 ms before the change. The mean improved from 20.317 to 20.004 ms, but only two of the three final runs meet the 20 ms target; the third is a valid quiet-gated miss with 18 intervals above 33.3 ms. Slice p99s for this cell are 4.288, 4.324, and 4.363 ms, versus 4.221, 4.193, and 4.099 ms before. Thus the frame-tail target and no-worse slice-p99 condition are not established.

The final matrix has seven valid p99 captures above 20 ms across six cells, leaving only two of eight cells meeting the frame-p99 target; the previous follow-up had seven of eight meeting it. The high values are clustered in late repetitions and have normal p50/p95 but bursts of missed presentation intervals: sand-release/bounded-focus rep 3 is 34.290 ms (57 intervals >33.3 ms), reservoir-breach/bounded-fifo rep 3 is 30.390 ms (23), burning-forest/bounded-focus rep 3 is 33.754 ms (43), burning-forest/bounded-fifo rep 3 is 33.132 ms (34), and mixed-overload/bounded-focus reps 2/3 are 34.788/34.746 ms (57/53). Mixed-overload/bounded-fifo rep 2 also has a 329.073 ms maximum despite a 19.293 ms p99. Several corresponding logs show simulation/upload/submit maxima below about 13/4/3 ms, so the long intervals do not correlate with a large measured CPU stage. The external load gate was met before and after every accepted sample, but transient OS/WindowServer/driver interference within the minute is not excluded. This dataset therefore does not prove the outliers are caused by the upload change, and it also cannot be used to claim no regression. Treat the frame-p99 goal as unmet pending further diagnosis or a cleaner host window.

## Section 15 checklist rows

| Fixture | Policy | Accepted runs | Frame p99 ≤20 ms | Slice p99 ≤4 ms | >33.3 ms count / max disclosed |
|---|---|---:|---|---|---|
| quiet-world | bounded-focus | 0/3 | incomplete | incomplete | incomplete |
| quiet-world | bounded-fifo | 0/3 | incomplete | incomplete | incomplete |
| quiet-world | traditional | 0/3 | incomplete | incomplete | incomplete |
| explosive-lattice | bounded-focus | 0/3 | incomplete | incomplete | incomplete |
| explosive-lattice | bounded-fifo | 0/3 | incomplete | incomplete | incomplete |
| explosive-lattice | traditional | 0/3 | incomplete | incomplete | incomplete |
| sand-release | bounded-focus | 3/3 | unmet | unmet | reported |
| sand-release | bounded-fifo | 3/3 | unmet | unmet | reported |
| sand-release | traditional | 0/3 | incomplete | incomplete | incomplete |
| reservoir-breach | bounded-focus | 3/3 | met | met | reported |
| reservoir-breach | bounded-fifo | 3/3 | unmet | unmet | reported |
| reservoir-breach | traditional | 0/3 | incomplete | incomplete | incomplete |
| burning-forest | bounded-focus | 3/3 | unmet | unmet | reported |
| burning-forest | bounded-fifo | 3/3 | unmet | unmet | reported |
| burning-forest | traditional | 0/3 | incomplete | incomplete | incomplete |
| dirty-world-sweep | bounded-focus | 0/3 | incomplete | incomplete | incomplete |
| dirty-world-sweep | bounded-fifo | 0/3 | incomplete | incomplete | incomplete |
| dirty-world-sweep | traditional | 0/3 | incomplete | incomplete | incomplete |
| tiny-capacity | bounded-focus | 0/3 | incomplete | incomplete | incomplete |
| tiny-capacity | bounded-fifo | 0/3 | incomplete | incomplete | incomplete |
| tiny-capacity | traditional | 0/3 | incomplete | incomplete | incomplete |
| mixed-overload | bounded-focus | 3/3 | unmet | unmet | reported |
| mixed-overload | bounded-fifo | 3/3 | met | unmet | reported |
| mixed-overload | traditional | 0/3 | incomplete | incomplete | incomplete |

## Interaction feedback samples

These CPU-side event/action-to-visible-upload p95 values use the app's documented method. Counts below 30 are descriptive only; quiet-world intentionally has no scripted interaction samples.

| Fixture | Policy | Rep | Camera n / p95 ms | Paint n / p95 ms | Ignite n / p95 ms | Detonate n / p95 ms |
|---|---|---:|---:|---:|---:|---:|
| sand-release | bounded-focus | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | bounded-fifo | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-focus | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-fifo | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-focus | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-fifo | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| mixed-overload | bounded-focus | 1 | 240 / 18.35 | 240 / 18.23 | 240 / 18.35 | 240 / 18.36 |
| mixed-overload | bounded-fifo | 1 | 240 / 18.10 | 240 / 17.87 | 240 / 17.98 | 240 / 18.01 |
| sand-release | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| mixed-overload | bounded-fifo | 2 | 240 / 17.74 | 240 / 18.07 | 240 / 17.76 | 240 / 17.83 |
| mixed-overload | bounded-focus | 2 | 240 / 18.51 | 240 / 18.51 | 240 / 18.73 | 240 / 18.39 |
| sand-release | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| mixed-overload | bounded-focus | 3 | 240 / 18.24 | 240 / 18.16 | 240 / 18.07 | 240 / 18.42 |
| mixed-overload | bounded-fifo | 3 | 240 / 17.68 | 240 / 17.91 | 240 / 17.75 | 240 / 17.85 |

## Rejected attempts

| Fixture | Policy | Rep | Attempt | Load before/after | Reason | Log |
|---|---|---:|---:|---:|---|---|
| sand-release | bounded-focus | 1 | 1 | 2.75/3.04 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/sand-release-bounded-focus-rep1-attempt1.log` |
| sand-release | bounded-focus | 1 | 2 | 3.04/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 1 | 2.71/3.57 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/sand-release-bounded-fifo-rep1-attempt1.log` |
| sand-release | bounded-fifo | 1 | 2 | 3.57/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 3 | 3.61/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 4 | 3.32/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 5 | 3.29/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 6 | 3.27/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 7 | 3.17/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 8 | 3.07/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 1 | 4.27/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 2 | 4.17/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 3 | 4.00/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 4 | 4.00/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 5 | 3.76/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 6 | 3.54/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 7 | 3.49/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 8 | 3.29/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 9 | 3.27/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 10 | 3.17/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 11 | 3.15/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 12 | 3.06/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 1 | 1 | 2.48/3.98 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/burning-forest-bounded-fifo-rep1-attempt1.log` |
| burning-forest | bounded-fifo | 1 | 2 | 3.98/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 1 | 3 | 3.90/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 1 | 4 | 3.75/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 1 | 5 | 3.53/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 1 | 6 | 3.25/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 1 | 7 | 3.07/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 1 | 8 | 3.06/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 1 | 9 | 3.06/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 1 | 10 | 3.13/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 1 | 11 | 3.12/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 1 | 12 | 3.11/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 1 | 13 | 3.10/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-fifo | 2 | 1 | 3.75/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-fifo | 2 | 2 | 3.53/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-fifo | 2 | 3 | 3.33/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-fifo | 2 | 4 | 3.14/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-fifo | 2 | 5 | 3.05/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-fifo | 2 | 6 | 3.04/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 3 | 1 | 2.26/3.08 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/reservoir-breach-bounded-fifo-rep3-attempt1.log` |
| reservoir-breach | bounded-fifo | 3 | 2 | 3.08/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 3 | 3 | 3.24/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 3 | 4 | 3.06/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 3 | 5 | 3.13/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 3 | 6 | 3.12/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 3 | 7 | 3.03/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 3 | 8 | 2.95/3.54 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/reservoir-breach-bounded-fifo-rep3-attempt8.log` |
| reservoir-breach | bounded-fifo | 3 | 9 | 3.54/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 3 | 10 | 3.50/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 3 | 11 | 3.38/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 3 | 12 | 3.35/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 3 | 13 | 3.40/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 3 | 14 | 3.21/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 3 | 15 | 3.03/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-fifo | 3 | 1 | 2.74/3.07 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/mixed-overload-bounded-fifo-rep3-attempt1.log` |
| mixed-overload | bounded-fifo | 3 | 2 | 3.07/n/a | pre-run one-minute load not below 3 | `-` |

## Interpretation and limits

Frame intervals are native windowed event-loop intervals; they include presentation pacing and are not GPU duration. Slice CPU values are monotonic wall-time samples around the app's simulation dispatch, not thread CPU time. Fixture captures do not add external disturbances or player actions, except mixed-overload, which uses the app's scripted sustained-overload action stream to exercise focus latency. The load gate does not exclude all OS/driver interference.

Raw per-attempt logs are retained locally under `benchmarks/tmp/` and are not committed.
