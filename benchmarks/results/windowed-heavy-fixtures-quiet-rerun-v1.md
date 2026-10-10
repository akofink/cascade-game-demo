# Native windowed heavy-fixture quiet-host rerun v1

Status: complete for the requested eight bounded cells; 24/24 accepted captures; 14 load-gate-rejected attempts. This separate rerun covers only the eight bounded cells, not the full 72-capture fixture/policy matrix. Earlier baseline and final-source captures remain in [heavy-fixture follow-up v1](windowed-heavy-fixtures-v1.md) and [follow-up v2](windowed-heavy-fixtures-v2.md); none were replaced or removed. The operator accepts this quiet-host result for delivery, including the residual slice-p99 miss described below; the miss remains disclosed and is not claimed to meet baseline.

App source revision: `c5f75089d3ed8eba3b86af9be1cbc3d990f10489`
Suite runner revision: `c5f75089d3ed8eba3b86af9be1cbc3d990f10489`
Suite started: 2026-10-09T22:21:23-04:00
Reference host: `akmac`, MacBook Air (Mac14,2), Apple M2/8 CPU cores/16 GB RAM/integrated GPU, macOS 27.0.1; Rust `rustc 1.99.0 (b940084d7 2026-09-28)`.
Capture: release app, 4096² world, 1920×1080, 60 Hz target, profile `m2-16gb-v3`; Cargo.lock SHA-256 `d064050f2b9044bb0eecfb2b7ffee1b18644dd676e192f7b7d0b0008d42db06b`.
The operator reported quitting other activity and leaving the display awake. Power mode, thermal state, and physical display refresh are not controlled. Host load and process snapshots were sampled approximately every 10 seconds while the capture app was running. The accepted-capture gate still uses only the one-minute load immediately before and after each run; in-run load samples are diagnostic, not grounds to discard an otherwise valid capture. The sampled `ps` CPU field is not a per-interval CPU delta, so process names and reported percentages show what appeared in snapshots but do not establish exact CPU contention or causation.

Each fixture/policy cell requires three accepted 60-second captures. A run is accepted only when one-minute load is below 3 both immediately before and after the process, the app exits successfully with `SMOKE_RESULT ok` after its 60-second capture timer, reports the requested fixture/policy at 1920×1080 with nonzero frame samples, has no interval drops, and has complete slice telemetry. Frame counts are actual measured loop intervals, not an assumed 60 Hz count. Rejected attempts are retained below and never contribute to accepted percentiles.

## Accepted captures

| Fixture | Policy | Rep | Load before/after | Frames | Frame p99 ms | Max ms | >33.3 ms | Slice p99 ms | Slice max ms |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| sand-release | bounded-focus | 1 | 1.74/2.97 | 3600 | 18.723 | 20.124 | 0 | 5.185 | 5.653 |
| sand-release | bounded-fifo | 1 | 2.86/2.05 | 3600 | 19.437 | 21.855 | 0 | 4.289 | 5.088 |
| reservoir-breach | bounded-focus | 1 | 2.05/2.03 | 3600 | 18.607 | 20.432 | 0 | 2.408 | 2.655 |
| reservoir-breach | bounded-fifo | 1 | 2.03/1.87 | 3600 | 18.816 | 20.454 | 0 | 4.199 | 5.025 |
| burning-forest | bounded-focus | 1 | 1.87/2.08 | 3599 | 18.663 | 26.936 | 0 | 4.817 | 14.572 |
| burning-forest | bounded-fifo | 1 | 2.08/2.37 | 3600 | 18.964 | 21.256 | 0 | 5.209 | 11.570 |
| mixed-overload | bounded-focus | 1 | 2.37/2.35 | 3600 | 18.706 | 19.806 | 0 | 4.503 | 5.829 |
| mixed-overload | bounded-fifo | 1 | 2.35/2.28 | 3600 | 18.544 | 20.997 | 0 | 4.222 | 4.943 |
| sand-release | bounded-fifo | 2 | 2.28/2.62 | 3600 | 19.277 | 21.353 | 0 | 4.217 | 4.694 |
| sand-release | bounded-focus | 2 | 2.62/2.13 | 3600 | 18.729 | 20.518 | 0 | 5.129 | 5.553 |
| reservoir-breach | bounded-fifo | 2 | 2.13/2.22 | 3600 | 18.880 | 23.112 | 0 | 4.146 | 4.782 |
| reservoir-breach | bounded-focus | 2 | 2.22/2.99 | 3600 | 18.604 | 19.802 | 0 | 2.478 | 2.573 |
| burning-forest | bounded-fifo | 2 | 2.99/2.52 | 3599 | 19.072 | 29.854 | 0 | 5.061 | 12.200 |
| burning-forest | bounded-focus | 2 | 2.52/2.41 | 3599 | 18.727 | 29.341 | 0 | 4.858 | 13.073 |
| mixed-overload | bounded-fifo | 2 | 2.41/1.73 | 3600 | 18.617 | 20.222 | 0 | 4.302 | 4.812 |
| mixed-overload | bounded-focus | 2 | 1.73/2.20 | 3600 | 18.672 | 19.944 | 0 | 4.601 | 5.030 |
| sand-release | bounded-focus | 3 | 2.20/2.26 | 3600 | 18.684 | 19.838 | 0 | 5.288 | 5.692 |
| sand-release | bounded-fifo | 3 | 2.26/2.66 | 3600 | 19.343 | 22.296 | 0 | 4.296 | 5.890 |
| reservoir-breach | bounded-focus | 3 | 2.66/2.75 | 3600 | 18.581 | 20.254 | 0 | 2.465 | 3.497 |
| reservoir-breach | bounded-fifo | 3 | 2.75/2.87 | 3600 | 18.743 | 20.041 | 0 | 4.131 | 4.847 |
| burning-forest | bounded-focus | 3 | 2.87/2.33 | 3599 | 18.705 | 20.501 | 0 | 4.742 | 6.359 |
| burning-forest | bounded-fifo | 3 | 2.33/2.23 | 3599 | 18.927 | 20.657 | 0 | 4.932 | 8.846 |
| mixed-overload | bounded-focus | 3 | 2.23/2.23 | 3600 | 18.578 | 19.739 | 0 | 4.527 | 5.000 |
| mixed-overload | bounded-fifo | 3 | 2.23/2.68 | 3600 | 18.669 | 21.044 | 0 | 4.258 | 4.832 |

## Sampled host conditions by accepted capture

The table summarizes the approximately 10-second samples captured during each accepted window. `1m load range` is the observed one-minute load across those samples. `WindowServer` is its maximum sampled `ps` CPU field. `Highest other sampled process` excludes the app and WindowServer and is ranked by that same reported field; the field is not an instantaneous utilization measurement. The first bounded-FIFO attempt for sand-release was rejected by the normal gate and is listed under rejected attempts, not here.

| Fixture | Policy | Rep / attempt | Samples | 1m load range | WindowServer sampled max | Highest other sampled process |
|---|---|---:|---:|---:|---:|---|
| sand-release | bounded-focus | 1/1 | 6 | 1.71–2.97 | 48.7% | launchd (50.5%) |
| sand-release | bounded-fifo | 1/15 | 7 | 2.05–2.95 | 45.2% | runningboardd (13.0%) |
| reservoir-breach | bounded-focus | 1/1 | 6 | 1.90–2.15 | 45.1% | ChatGPT Renderer (8.2%) |
| reservoir-breach | bounded-fifo | 1/1 | 6 | 1.63–2.03 | 41.8% | ChatGPT Renderer (5.6%) |
| burning-forest | bounded-focus | 1/1 | 7 | 1.87–2.40 | 44.4% | duetexpertd (95.7%) |
| burning-forest | bounded-fifo | 1/1 | 6 | 2.25–2.65 | 46.5% | duetexpertd (12.1%) |
| mixed-overload | bounded-focus | 1/1 | 7 | 2.09–2.37 | 48.2% | Claude (19.6%) |
| mixed-overload | bounded-fifo | 1/1 | 6 | 2.20–2.43 | 45.0% | ChatGPT Renderer (7.6%) |
| sand-release | bounded-fifo | 2/1 | 7 | 2.28–2.76 | 45.2% | gh (9.0%) |
| sand-release | bounded-focus | 2/1 | 6 | 2.23–2.81 | 47.5% | NewsToday2 (50.5%) |
| reservoir-breach | bounded-fifo | 2/1 | 7 | 2.22–2.33 | 44.9% | bluetoothd (9.1%) |
| reservoir-breach | bounded-focus | 2/1 | 6 | 2.24–2.72 | 46.7% | duetexpertd (92.6%) |
| burning-forest | bounded-fifo | 2/1 | 7 | 2.47–3.01 | 47.8% | NotificationCenter (13.5%) |
| burning-forest | bounded-focus | 2/1 | 6 | 2.24–2.62 | 46.1% | ChatGPT Renderer (5.3%) |
| mixed-overload | bounded-fifo | 2/1 | 7 | 1.55–2.30 | 51.1% | NotificationCenter (26.9%) |
| mixed-overload | bounded-focus | 2/1 | 6 | 1.72–2.39 | 44.6% | Docker backend (14.9%) |
| sand-release | bounded-focus | 3/1 | 7 | 2.10–2.46 | 46.2% | launchd (55.2%) |
| sand-release | bounded-fifo | 3/1 | 6 | 2.54–2.73 | 45.9% | biomesyncd (57.3%) |
| reservoir-breach | bounded-focus | 3/1 | 7 | 2.68–2.88 | 46.8% | ChatGPT Renderer (7.3%) |
| reservoir-breach | bounded-fifo | 3/1 | 6 | 2.55–3.01 | 46.9% | ChatGPT Renderer (8.3%) |
| burning-forest | bounded-focus | 3/1 | 6 | 2.38–3.03 | 45.6% | NotificationCenter (16.4%) |
| burning-forest | bounded-fifo | 3/1 | 7 | 1.88–2.57 | 45.4% | BiomeAgent (18.2%) |
| mixed-overload | bounded-focus | 3/1 | 6 | 2.05–2.38 | 44.4% | biomesyncd (60.6%) |
| mixed-overload | bounded-fifo | 3/1 | 7 | 2.29–2.99 | 46.0% | pi (20.2%) |

The in-run samples show some accepted runs briefly at or above load 3 despite passing both required before/after checks. The rejected first sand-release/FIFO attempt had six in-run samples with one-minute load 2.89–6.20; WindowServer was reported at 40.9–42.3% and CoreSpotlight at 24.7% in the recorded top-process snapshots. Process snapshots for accepted runs also show WindowServer and background services or applications active during captures. This was therefore a quiet-host attempt, not a fully isolated or process-idle measurement.

## Comparison with baseline and decision gate

All 24 rerun frame p99 values are at or below 20 ms, including all three sand-release/bounded-fifo repetitions. All eight cells that met the frame target in baseline v1 meet it here; sand-release/bounded-fifo also now meets it. No accepted rerun interval exceeded 33.3 ms.

The baseline slice ranges below are the min/max of the three accepted v1 repetitions per cell. Rerun values are listed in repetition order. Under a strict all-repetitions-within-range reading, only burning-forest/bounded-focus has all three values inside its baseline range. By per-cell mean, sand-release/bounded-fifo remains outside its baseline range: rerun mean 4.267 ms versus baseline range 4.099–4.221 ms (baseline mean 4.171 ms). This residual miss is accepted for delivery by the operator; it remains unmet against baseline, and no regression-free result is claimed.

| Fixture | Policy | Rerun frame p99 ms (reps 1–3) | Baseline slice p99 range ms | Rerun slice p99 ms (reps 1–3) |
|---|---|---|---|---|
| sand-release | bounded-focus | 18.723, 18.729, 18.684 | 5.133–5.433 | 5.185, 5.129, 5.288 |
| sand-release | bounded-fifo | 19.437, 19.277, 19.343 | 4.099–4.221 | 4.289, 4.217, 4.296 |
| reservoir-breach | bounded-focus | 18.607, 18.604, 18.581 | 2.418–2.467 | 2.408, 2.478, 2.465 |
| reservoir-breach | bounded-fifo | 18.816, 18.880, 18.743 | 4.066–4.195 | 4.199, 4.146, 4.131 |
| burning-forest | bounded-focus | 18.663, 18.727, 18.705 | 4.650–4.955 | 4.817, 4.858, 4.742 |
| burning-forest | bounded-fifo | 18.964, 19.072, 18.927 | 4.960–5.095 | 5.209, 5.061, 4.932 |
| mixed-overload | bounded-focus | 18.706, 18.672, 18.578 | 4.473–4.572 | 4.503, 4.601, 4.527 |
| mixed-overload | bounded-fifo | 18.544, 18.617, 18.669 | 4.193–4.288 | 4.222, 4.302, 4.258 |

## Section 15 checklist rows

| Fixture | Policy | Accepted runs | Frame p99 ≤20 ms | Slice p99 ≤4 ms | >33.3 ms count / max disclosed |
|---|---|---:|---|---|---|
| quiet-world | bounded-focus | 0/3 | incomplete | incomplete | incomplete |
| quiet-world | bounded-fifo | 0/3 | incomplete | incomplete | incomplete |
| quiet-world | traditional | 0/3 | incomplete | incomplete | incomplete |
| explosive-lattice | bounded-focus | 0/3 | incomplete | incomplete | incomplete |
| explosive-lattice | bounded-fifo | 0/3 | incomplete | incomplete | incomplete |
| explosive-lattice | traditional | 0/3 | incomplete | incomplete | incomplete |
| sand-release | bounded-focus | 3/3 | met | unmet | reported |
| sand-release | bounded-fifo | 3/3 | met | unmet | reported |
| sand-release | traditional | 0/3 | incomplete | incomplete | incomplete |
| reservoir-breach | bounded-focus | 3/3 | met | met | reported |
| reservoir-breach | bounded-fifo | 3/3 | met | unmet | reported |
| reservoir-breach | traditional | 0/3 | incomplete | incomplete | incomplete |
| burning-forest | bounded-focus | 3/3 | met | unmet | reported |
| burning-forest | bounded-fifo | 3/3 | met | unmet | reported |
| burning-forest | traditional | 0/3 | incomplete | incomplete | incomplete |
| dirty-world-sweep | bounded-focus | 0/3 | incomplete | incomplete | incomplete |
| dirty-world-sweep | bounded-fifo | 0/3 | incomplete | incomplete | incomplete |
| dirty-world-sweep | traditional | 0/3 | incomplete | incomplete | incomplete |
| tiny-capacity | bounded-focus | 0/3 | incomplete | incomplete | incomplete |
| tiny-capacity | bounded-fifo | 0/3 | incomplete | incomplete | incomplete |
| tiny-capacity | traditional | 0/3 | incomplete | incomplete | incomplete |
| mixed-overload | bounded-focus | 3/3 | met | unmet | reported |
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
| mixed-overload | bounded-focus | 1 | 240 / 18.38 | 240 / 18.33 | 240 / 18.48 | 240 / 18.46 |
| mixed-overload | bounded-fifo | 1 | 240 / 17.93 | 240 / 18.06 | 240 / 17.93 | 240 / 17.71 |
| sand-release | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| mixed-overload | bounded-fifo | 2 | 240 / 17.83 | 240 / 17.87 | 240 / 18.15 | 240 / 18.24 |
| mixed-overload | bounded-focus | 2 | 240 / 18.38 | 240 / 18.36 | 240 / 18.47 | 240 / 18.29 |
| sand-release | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| mixed-overload | bounded-focus | 3 | 240 / 18.25 | 240 / 18.18 | 240 / 18.08 | 240 / 18.08 |
| mixed-overload | bounded-fifo | 3 | 240 / 18.17 | 240 / 18.15 | 240 / 18.02 | 240 / 18.11 |

## Rejected attempts

| Fixture | Policy | Rep | Attempt | Load before/after | Reason | Log |
|---|---|---:|---:|---:|---|---|
| sand-release | bounded-fifo | 1 | 1 | 2.97/3.93 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/sand-release-bounded-fifo-rep1-attempt1.log` |
| sand-release | bounded-fifo | 1 | 2 | 3.93/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 3 | 3.70/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 4 | 3.56/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 5 | 3.76/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 6 | 3.62/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 7 | 3.65/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 8 | 3.43/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 9 | 3.32/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 10 | 3.13/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 11 | 3.04/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 12 | 3.12/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 13 | 3.11/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 14 | 3.02/n/a | pre-run one-minute load not below 3 | `-` |

## Interpretation and limits

Frame intervals are native windowed event-loop intervals; they include presentation pacing and are not GPU duration. Slice CPU values are monotonic wall-time samples around the app's simulation dispatch, not thread CPU time. Fixture captures do not add external disturbances or player actions, except mixed-overload, which uses the app's scripted sustained-overload action stream to exercise focus latency. The load gate does not exclude all OS/driver interference.

Raw per-attempt logs are retained locally under `benchmarks/tmp/` and are not committed.
