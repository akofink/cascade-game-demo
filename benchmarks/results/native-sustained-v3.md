# Native sustained capture v3: recovery pricing and water-direction follow-up

Status: three valid 60-second windowed FIFO captures on merged source `50a0fc5`; MacBook Air M2; 4096² world; 1920×1080 surface; 1,000,000 credits/slice; recovery cost 20.

## Method and quiet-machine gate

Each fresh release process ran `caffeinate -dimsu target/release/cascade-app --capture-policy bounded-fifo --capture-seconds 60 --world-size 4096`. All captures printed `SMOKE_CAPTURE_STARTED`, measured 3,600 frame intervals, completed with `SMOKE_RESULT ok`, and reported zero interval-buffer drops. The one-minute system load was below 3 before and after every retained run.

| Rep | Load before/after | Intervals | Frame p50 ms | Frame p95 ms | Frame p99 ms | Max interval ms | >33.3 ms | Sim p99 ms | Max sim CPU ms | Max upload CPU ms |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1.93 / 2.07 | 3,600 | 16.669 | 17.536 | 18.661 | 21.019 | 0 | 4.398 | 4.972 | 4.663 |
| 2 | 1.70 / 1.94 | 3,600 | 16.665 | 17.509 | 18.485 | 21.662 | 0 | 4.372 | 4.981 | 5.123 |
| 3 | 1.80 / 2.20 | 3,600 | 16.665 | 17.576 | 18.521 | 21.570 | 0 | 4.391 | 4.904 | 5.062 |

The bounded FIFO frame-interval p99 target of 20 ms is met in all three captures, with no interval above 33.3 ms. The simulation-slice p99 target of 4 ms is **not met**: all three p99s are 4.372–4.398 ms, and all three maxima are 4.904–4.981 ms. The prior source's 5.188–5.410 ms maxima improved, but the strict 4 ms target remains open. This evidence is for one fixture and one M2 host, not the all-fixture acceptance matrix or a portable timing guarantee.

Per-slice max-work counts were similar across repetitions: about 25,636 evaluations, 3 blasts, 17,095 recoveries, 1 command, and 42,750 selection probes. Each capture allowed 1,000,000 credits and reported a saturated maximum slice. Maximum upload backlog was 304 chunks; the native frame interval results remain under 20 ms p99. The upload metric is app CPU time and is not GPU execution time.

## Interaction feedback

The app reported 240 samples per category per capture. Camera p95 was 17.65–17.74 ms; paint p95 17.49–17.63 ms; ignite p95 17.70–17.93 ms; detonate p95 17.33–17.53 ms. These CPU-side interaction measurements exclude display scanout and composition.

## Offscreen companion

One accepted, separate 60-second offscreen FIFO run used the same world and credit profile. Its load was 2.50 before and 2.23 after. It measured frame-interval p50/p95/p99/max of 16.661/18.480/19.584/31.463 ms, zero intervals above 33.3 ms, simulation p99 4.311 ms, and maximum simulation CPU 6.695 ms. Offscreen measurements do not include a window, presentation, vsync, compositor, or scanout and must not be combined with the windowed distribution.

## Interpretation

Recovery probes were repriced from 4 to 20 credits, duplicate queue/focus checks for already-queued channels were avoided, and water retains horizontal direction to avoid immediate left-right backtracking. The 1,000,000-credit profile was preserved. The changes improved the observed windowed FIFO maximum relative to the previous 5.188–5.410 ms range, but simulation p99 and maxima still miss 4 ms. They also reduce work throughput; full-size headless evidence and the finite-burst outcome are in [headless recovery follow-up v5](headless-v5.md). Recovery does not yet have an accepted stable-window result.
