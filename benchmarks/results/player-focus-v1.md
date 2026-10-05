# Player-action focus comparison v1

Status: one full-size headless capture; exploratory, not a release acceptance run.

## Reproduction

```sh
cargo +1.99.0 run --release -p cascade-bench -- \
  --player-action-comparison --width 4096 --height 4096 \
  --budget 1000000 --slices 512
```

Code revision: `e9e567b` (`bench: measure player focus action latency`).

Reference machine: MacBook Air (Mac14,2), Apple M2, 16 GB RAM, macOS 27.0.1, Rust/Cargo 1.99.0. The existing reference profile reports a 1,000,000-credit allowance. Each policy started from a newly prepared `mixed-overload-v1` world. Preparation is excluded from measurement. The scripted stream admitted 64 player actions at eight-slice intervals (paint wood, ignite, paint explosive, detonate, repeat) at a deterministic center-region sequence, and one seeded background disturbance every four slices (fixture seed XOR `0x5a110ad5f00d0001`). Action wall latency is host monotonic elapsed time from command admission until the benchmark observes the first-effect/settle record; it includes fixed benchmark polling overhead. Each row is one 512-slice run, with no repetitions or warm-up.

## Results

Effect latency distributions include action records with an observed first effect. Settle distributions include only actions whose local 3x3 cell neighborhood settled within the 512-slice window; the other actions are right-censored. Wall times are milliseconds.

| Policy | Effect samples / admitted | Effect slices p50/p95/p99/max | Effect wall ms p50/p95/p99/max | Settle samples / admitted | Settle slices p50/p95/p99/max | Settle wall ms p50/p95/p99/max | Background quanta | Oldest pending age | Pending channels at end | Measured wall time |
| --- | ---: | --- | --- | ---: | --- | --- | ---: | ---: | ---: | ---: |
| Traditional | 64 / 64 | 1 / 1 / 1 / 1 | 170.1 / 281.0 / 1418.6 / 1418.6 | 64 / 64 | 9 / 273 / 297 / 297 | 836.1 / 20325.3 / 27606.2 / 27606.2 | 98,434,091 | 0 | 194,777 | 52.54 s |
| Bounded FIFO | 64 / 64 | 1 / 24 / 24 / 24 | 17.7 / 156.6 / 168.8 / 168.8 | 37 / 64 | 63 / 247 / 255 / 255 | 486.1 / 1545.7 / 1588.3 / 1588.3 | 15,999,845 | 730 | 2,204,730 | 3.72 s |
| Bounded with focus | 63 / 64 | 0 / 8 / 40 / 40 | 8.5 / 87.1 / 390.3 / 390.3 | 11 / 64 | 0 / 38 / 38 / 38 | 7.2 / 260.7 / 260.7 / 260.7 | 6,364,004 | 730 | 2,072,145 | 4.24 s |

## Interpretation and limits

Focus reduced measured action-to-first-effect p95 wall latency by about 44% versus bounded FIFO (156.6 ms to 87.1 ms), but did **not** meet the charter's 50 ms target. Against the specified traditional baseline, p95 was 281.0 ms. Bounded-with-focus local-settle p95 was 260.7 ms among only 11 resolved actions, versus 1545.7 ms among 37 bounded-FIFO resolutions; these small, censored samples are suggestive, not acceptance evidence. One focus action had no first effect observed by the end of the run.

Background service remained nonzero. Its measured quantum count was about 60% lower with focus than bounded FIFO, showing the cost of the configured 50% focus share under this load. Traditional's oldest-age counter was zero at sampled slice boundaries despite residual pending channels; do not compare that value as a measure of full-frontier completion. The traditional wall-time comparison also includes its charter-defined full-world frontier scans.

This is one 512-slice capture, not the charter's three 60-second repetitions. It does not measure presentation/UI acknowledgment latency, native frame intervals, memory high-water marks, or sustained-load p99 with confidence. The 50 ms action target and extended repeat protocol remain unmet.
