# Native sustained capture v1

Status: three valid 60-second release captures per policy; one reference machine

## Method

Command per capture:

```sh
caffeinate -dimsu target/release/cascade-app --world-size 4096 \
  --capture-policy POLICY --capture-seconds 60
```

Each fresh process prepares `mixed-overload-v1`, warms/presents the renderer, then measures one 60-second window while DESTROY PERFORMANCE disturbance admission is held. The capture window is refused unless the native surface is 1920 x 1080. The scripted stream admits paint, ignite, and detonate actions in rotation, plus a one-pixel camera nudge each presented frame. Paint is followed by a queued ignition on that painted cell; detonations target seeded explosives. Frame intervals are collected in a fixed 16,384-sample buffer and summarized with nearest-rank p50/p95/p99 and max. `over_33_3_ms` counts intervals strictly above 33.3 ms. No samples were lost. Per-window game-feel output reports the most recent 240 samples per action kind and camera; these CPU-time-to-visible-upload measurements exclude display scanout.

Policy orders alternate by repetition: focus, FIFO, traditional; traditional, FIFO, focus; FIFO, focus, traditional. Fixture preparation and renderer warm-up are outside the measured 60 seconds. Captures were made on 2026-10-05 EDT on the built-in display, held awake with `caffeinate -dimsu`. The native window presented throughout; every measured run completed with `SMOKE_RESULT ok`, the 1920 x 1080 surface check passed, and no occlusion abort occurred. The one-minute load average stayed below 3 immediately before and after each run:

| Rep/policy | Start EDT, load before | End EDT, load after |
| --- | --- | --- |
| 1 focus | 22:06:09, 2.77 | 22:07:14, 2.55 |
| 1 FIFO | 22:07:21, 2.39 | 22:08:26, 2.36 |
| 1 traditional | 22:08:32, 2.33 | 22:09:37, 2.62 |
| 2 traditional | 22:09:43, 2.49 | 22:10:49, 2.05 |
| 2 FIFO | 22:10:55, 2.13 | 22:12:00, 2.02 |
| 2 focus | 22:12:06, 2.74 | 22:13:11, 2.88 |
| 3 FIFO | 22:13:20, 2.98 | 22:14:25, 2.74 |
| 3 focus | 22:14:34, 2.62 | 22:15:39, 2.12 |
| 3 traditional | 22:15:47, 2.03 | 22:16:52, 2.08 |

Reference system: MacBook Air (Mac14,2), Apple M2, 8 CPU cores, 16 GB RAM, integrated M2 GPU, macOS 27.0.1, Rust/Cargo 1.99.0, built-in 2560 x 1664 display; captured native render surface 1920 x 1080. Release build. Power/thermal state and unrelated background work were not controlled; the load gate is disclosed, not a universal isolation guarantee.

## Frame intervals and simulation CPU

| Rep | Policy | Intervals | p50 ms | p95 ms | p99 ms | max ms | >33.3 ms | max simulation CPU ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | Bounded focus | 3,600 | 16.652 | 17.487 | 18.132 | 20.968 | 0 | 5.976 |
| 2 | Bounded focus | 3,599 | 16.655 | 17.583 | 18.224 | 34.342 | 1 | 6.175 |
| 3 | Bounded focus | 3,600 | 16.652 | 17.545 | 18.183 | 20.199 | 0 | 6.227 |
| 1 | Bounded FIFO | 3,600 | 16.682 | 17.978 | 20.758 | 26.685 | 0 | 4.903 |
| 2 | Bounded FIFO | 3,599 | 16.667 | 18.008 | 20.773 | 34.883 | 1 | 5.536 |
| 3 | Bounded FIFO | 3,600 | 16.679 | 18.069 | 20.763 | 25.582 | 0 | 5.494 |
| 1 | Traditional | 947 | 66.570 | 67.235 | 106.254 | 120.035 | 947 | 107.394 |
| 2 | Traditional | 946 | 66.573 | 67.332 | 108.058 | 120.869 | 946 | 107.459 |
| 3 | Traditional | 946 | 66.586 | 67.299 | 105.842 | 119.799 | 946 | 107.474 |

Across the three windows, bounded focus had 1 interval above 33.3 ms among 10,799; bounded FIFO had 1 among 10,799. Traditional had 2,839 among 2,839, consistent with its required full-frontier scans and roughly 66.6 ms median frame interval. The measured tails, including the two 34 ms bounded outliers, are retained.

## Game-feel p95

Each repetition produced 240 camera/UI-feedback samples and 240 visible-effect samples for each action type. Values below are the median of the three per-run nearest-rank p95s, in milliseconds:

| Policy | Camera/UI | Paint | Ignite | Detonate |
| --- | ---: | ---: | ---: | ---: |
| Bounded focus | 17.84 | 17.66 | 17.65 | 17.58 |
| Bounded FIFO | 17.84 | 18.05 | 17.62 | 18.72 |
| Traditional | 67.01 | 67.49 | 67.02 | 134.22 |

The measured focus action p95s meet the charter's 50 ms goal. Camera/UI feedback also met 50 ms in both bounded modes, but not traditional. Traditional detonation p95 was 134.61, 134.22, and 134.13 ms across repetitions.

## Interpretation and unmet criteria

- Bounded-focus p99 met the 20 ms target in all repetitions. Bounded FIFO p99 was 20.758-20.773 ms, slightly over the target in each repetition. Each bounded policy had one >33.3 ms interval across all three captures. Bounded focus max was 34.342 ms; FIFO max was 34.883 ms.
- Bounded simulation CPU maxima were 4.903-6.227 ms, above the 4 ms slice target. Credits bound algorithmic work, not elapsed CPU time.
- Traditional's full two-pass frontier scan remains intentionally unbounded by credits and visibly slow. Its high p99/max and every interval above 33.3 ms are expected under this charter-specific baseline, not representative of all traditional schedulers.
- GPU timing was unavailable. Visible-effect latency stops at presentation bookkeeping and excludes display scanout/composition.
- These captures cover this full-size mixed-overload fixture on one Mac, not every fixture or device. The separate headless burst/recovery and static-descriptor holdout limitations remain.
