# Player-action focus comparison v1

Status: quiet-machine full-size capture pending. Do not publish timing from the earlier busy-host run or the incomplete 1,024-slice diagnostic.

## Reproduction

```sh
cargo +1.99.0 run --release -p cascade-bench -- \
  --player-action-comparison --width 4096 --height 4096 \
  --budget 1000000 --slices 4096
```

Before and after each capture, record `uptime`'s 1-minute load average. Accept timing only if both are below about 3; include both readings with every result. Discard the capture if either exceeds the threshold.

Reference machine: MacBook Air (Mac14,2), Apple M2, 16 GB RAM, macOS 27.0.1, Rust/Cargo 1.99.0. Each policy starts from a newly prepared `mixed-overload-v1` world; preparation and deterministic target selection are excluded from measured wall time. The scripted stream attempts paint, ignite, and detonate commands every eight slices. Paint targets are sampled from the inert upper half of the fixture; the following ignite targets that newly painted wood or explosive, both supported by `Command::Ignite`. Detonate targets are sampled from fixture explosives and revalidated at admission. One seeded background disturbance is admitted every four slices (fixture seed XOR `0x5a110ad5f00d0001`). Action wall latency is monotonic elapsed time from command admission until the benchmark observes its first-effect or local-settle record; it includes benchmark polling overhead. The runner fails if any policy has fewer than 30 first-effect or local-settle samples per action type.

## Results

A clean full-size capture is pending. Record `uptime` immediately before and after the command; accept and publish timing only when both 1-minute load averages are below about 3. Include both load values with the policy results.

The CSV includes aggregate and per-action-type sample counts, p50/p95 effect and settle latency in slices and wall nanoseconds, action-effect candidate probes, background work, oldest pending age, final backlog, and deterministic final hashes. Settle distributions include only actions whose local 3x3 cell neighborhood settled within the 4,096-slice window; unsettled actions are right-censored.

## Interpretation and limits

No wall-time comparison is publishable until a clean host-load capture is recorded. This comparison provides deterministic per-policy workloads and enough samples for percentiles, but it is not a substitute for the charter's three 60-second acceptance repetitions. It does not measure presentation/UI acknowledgment latency, native frame intervals, or memory high-water marks.
