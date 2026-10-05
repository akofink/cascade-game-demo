# Player-action focus comparison v1

Status: quiet-machine capture pending. Do not publish timing from the earlier busy-host run.

## Reproduction

```sh
cargo +1.99.0 run --release -p cascade-bench -- \
  --player-action-comparison --width 4096 --height 4096 \
  --budget 1000000 --slices 512
```

Code revision: `a566f2d` (benchmark runner on merged simulator `793ed4a`, including its bounded execution-visualization path).

Before and after the command, record `uptime`'s 1-minute load average. Accept timing only if both are below about 3; include both values with every policy row. Discard the capture if either exceeds the threshold.

Reference machine: MacBook Air (Mac14,2), Apple M2, 16 GB RAM, macOS 27.0.1, Rust/Cargo 1.99.0. The existing reference profile reports a 1,000,000-credit allowance. Each policy started from a newly prepared `mixed-overload-v1` world. Preparation is excluded from measurement. The scripted stream admitted 64 player actions at eight-slice intervals (paint wood, ignite, paint explosive, detonate, repeat) at a deterministic center-region sequence, and one seeded background disturbance every four slices (fixture seed XOR `0x5a110ad5f00d0001`). Action wall latency is host monotonic elapsed time from command admission until the benchmark observes the first-effect/settle record; it includes fixed benchmark polling overhead. Each row is one 512-slice run, with no repetitions or warm-up.

## Results

A clean capture is pending. Record `uptime` immediately before and after the command; accept and publish timing only when both 1-minute load averages are below about 3. Include both load values with the policy results.

Effect latency distributions include action records with an observed first effect. Settle distributions include only actions whose local 3x3 cell neighborhood settled within the 512-slice window; the other actions are right-censored. Wall times are milliseconds.

## Interpretation and limits

No wall-time comparison is publishable until a clean host-load capture is recorded. The result is intended as one 512-slice exploratory run, not the charter's three 60-second repetitions. It does not measure presentation/UI acknowledgment latency, native frame intervals, memory high-water marks, or sustained-load p99 with confidence.
