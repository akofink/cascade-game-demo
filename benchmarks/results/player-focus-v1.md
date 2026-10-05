# Player-action focus comparison v1

Status: one full-size headless capture; exploratory, not a release acceptance run.

## Reproduction

```sh
cargo +1.99.0 run --release -p cascade-bench -- \
  --player-action-comparison --width 4096 --height 4096 \
  --budget 1000000 --slices 512
```

Code revision: `e3a6a09` (`fix(sim): guarantee background focus service`), including the benchmark implementation from `e9e567b`.

Reference machine: MacBook Air (Mac14,2), Apple M2, 16 GB RAM, macOS 27.0.1, Rust/Cargo 1.99.0. The existing reference profile reports a 1,000,000-credit allowance. Each policy started from a newly prepared `mixed-overload-v1` world. Preparation is excluded from measurement. The scripted stream admitted 64 player actions at eight-slice intervals (paint wood, ignite, paint explosive, detonate, repeat) at a deterministic center-region sequence, and one seeded background disturbance every four slices (fixture seed XOR `0x5a110ad5f00d0001`). Action wall latency is host monotonic elapsed time from command admission until the benchmark observes the first-effect/settle record; it includes fixed benchmark polling overhead. Each row is one 512-slice run, with no repetitions or warm-up.

## Results

Effect latency distributions include action records with an observed first effect. Settle distributions include only actions whose local 3x3 cell neighborhood settled within the 512-slice window; the other actions are right-censored. Wall times are milliseconds.

| Policy | Effect samples / admitted | Effect slices p50/p95/p99/max | Effect wall ms p50/p95/p99/max | Settle samples / admitted | Settle slices p50/p95/p99/max | Settle wall ms p50/p95/p99/max | Background quanta | Oldest pending age | Pending channels at end | Measured wall time |
| --- | ---: | --- | --- | ---: | --- | --- | ---: | ---: | ---: | ---: |
| Traditional | 64 / 64 | 1 / 1 / 1 / 1 | 169.6 / 385.1 / 1430.0 / 1430.0 | 64 / 64 | 9 / 273 / 297 / 297 | 860.0 / 20322.7 / 27807.8 / 27807.8 | 98,434,091 | 0 | 194,777 | 52.59 s |
| Bounded FIFO | 64 / 64 | 0 / 16 / 24 / 24 | 8.2 / 132.8 / 232.6 / 232.6 | 62 / 64 | 26 / 194 / 218 / 218 | 263.5 / 1353.4 / 1495.7 / 1495.7 | 18,054,587 | 730 | 2,428,698 | 4.23 s |
| Bounded with focus | 64 / 64 | 0 / 0 / 0 / 0 | 9.0 / 14.3 / 29.8 / 29.8 | 64 / 64 | 0 / 58 / 82 / 82 | 10.4 / 442.1 / 591.4 / 591.4 | 8,577,169 | 730 | 5,728,761 | 5.21 s |

## Interpretation and limits

Bounded focus measured action-to-first-effect p95 wall latency of 14.3 ms, below the charter's 50 ms target, versus 132.8 ms bounded FIFO and 385.1 ms traditional in this capture. All 64 focused actions observed a first effect. Focus local-settle p95 was 442.1 ms versus 1353.4 ms for bounded FIFO; 64/64 focus actions and 62/64 FIFO actions settled within this capture window.

Background service remained nonzero and met its configured minimum share in the scheduler test. Its measured quantum count was about 52.5% lower with focus than bounded FIFO (8.58M vs 18.05M), showing the performance tradeoff of the configured 50% focus share under this load. Focus ended with a higher pending-channel count (5.73M vs 2.43M FIFO). Traditional's oldest-age counter was zero at sampled slice boundaries despite residual pending channels; do not compare that value as a measure of full-frontier completion. The traditional wall-time comparison also includes its charter-defined full-world frontier scans.

This is one 512-slice capture, not the charter's three 60-second repetitions. It does not measure presentation/UI acknowledgment latency, native frame intervals, memory high-water marks, or sustained-load p99 with confidence. The single-capture action p95 is below 50 ms, but the extended repeat protocol and release acceptance remain unverified.
