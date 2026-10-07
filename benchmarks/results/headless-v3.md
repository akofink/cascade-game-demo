# Full-size headless results v3: full traditional frontier

This replaces the v2 comparison after correcting traditional mode to capture all per-cell pending channels, not just ready-ring entries. The v2 report remains as an archive, but its traditional measurements and cross-policy conclusions do not describe this policy. The new traditional baseline also performs two full-world scans per slice; those scan costs are charged and timed, and they dominate otherwise idle slices.

## Paired fixture matrix

Release `cascade-bench`, source revision `07b0d3c` (traditional frontier fix, `RULE_VERSION = 4`; the captured `cascade-sim` and `cascade-bench` files are tree-identical to this commit), Rust/Cargo 1.99.0. Each of the eight section-12 descriptors ran at 4096 x 4096 and 1,000,000 credits/slice. Each policy had three 300-slice measured runs per fixture (900 pooled slice samples). Every run had 120 warm-up slices, followed by fresh fixture preparation so measurement began from the same descriptor state. Policy order alternated bounded/traditional, traditional/bounded, bounded/traditional. There were no external disturbances in this matrix.

Times are monotonic wall-clock around `World::step`, in milliseconds. Percentiles use the lower indexed sample after sorting. `>4 ms` counts exact samples. Work is the sum of executed scheduler quanta, including both traditional full-world scans; backlog is the maximum observed pending-channel backlog. `Complete` counts samples where the runner's pending-channel and command completion predicate was true, not a claim that a dynamic world remains stable indefinitely. No fixture had rejected or coalesced benchmark commands because this matrix supplied none.

| Fixture | Policy | N | p50 ms | p95 ms | p99 ms | max ms | >4 ms | avg quanta/slice | total quanta | backlog high-water | complete samples |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| quiet-world | bounded | 900 | 0.000000 | 0.000042 | 0.000042 | 0.000333 | 0 | 0.0 | 0 | 0 | 900/900 |
| quiet-world | traditional | 900 | 24.673208 | 25.324208 | 25.907375 | 27.169041 | 900 | 33,554,432.0 | 30,198,988,800 | 0 | 900/900 |
| explosive-lattice | bounded | 900 | 0.000041 | 0.000042 | 0.000042 | 0.001084 | 0 | 0.0 | 27 | 0 | 900/900 |
| explosive-lattice | traditional | 900 | 24.665208 | 25.119084 | 25.996125 | 34.511000 | 900 | 33,554,432.0 | 30,198,988,815 | 4 | 897/900 |
| sand-release | bounded | 900 | 0.295417 | 0.310000 | 0.319292 | 0.340500 | 0 | 62,500.0 | 56,250,006 | 9,968,750 | 0/900 |
| sand-release | traditional | 900 | 44.017291 | 62.231708 | 63.633208 | 118.770875 | 900 | 35,428,903.7 | 31,886,013,312 | 3,690,496 | 0/900 |
| reservoir-breach | bounded | 900 | 0.515541 | 1.149209 | 1.183209 | 1.218916 | 0 | 79,979.7 | 71,981,712 | 2,062,831 | 0/900 |
| reservoir-breach | traditional | 900 | 24.628291 | 25.308958 | 26.232708 | 33.511125 | 900 | 33,563,862.7 | 30,207,476,460 | 4,922 | 0/900 |
| burning-forest | bounded | 900 | 1.276667 | 2.631208 | 2.978875 | 4.110792 | 1 | 62,500.0 | 56,250,000 | 7,355,896 | 0/900 |
| burning-forest | traditional | 900 | 24.698500 | 128.662292 | 226.496334 | 235.224875 | 900 | 34,126,669.1 | 30,714,002,184 | 12,352,247 | 747/900 |
| dirty-world-sweep | bounded | 900 | 0.000041 | 0.000042 | 0.000042 | 0.000666 | 0 | 0.0 | 0 | 0 | 900/900 |
| dirty-world-sweep | traditional | 900 | 24.650750 | 24.843542 | 26.367125 | 32.382875 | 900 | 33,554,432.0 | 30,198,988,800 | 0 | 900/900 |
| tiny-capacity | bounded | 900 | 0.450333 | 0.463709 | 0.478333 | 0.490167 | 0 | 72,428.3 | 65,185,440 | 8,501,648 | 0/900 |
| tiny-capacity | traditional | 900 | 24.642125 | 26.101042 | 29.659958 | 71.251083 | 900 | 33,587,812.3 | 30,229,031,076 | 135,457 | 867/900 |
| mixed-overload | bounded | 900 | 0.332917 | 0.901667 | 0.941000 | 0.966875 | 0 | 62,500.0 | 56,250,000 | 8,374,807 | 0/900 |
| mixed-overload | traditional | 900 | 25.186667 | 50.889917 | 77.354458 | 78.705167 | 900 | 33,770,223.0 | 30,393,200,724 | 3,236,204 | 0/900 |

All eight fixtures produced zero rejected/coalesced commands. On the overloaded burning-forest fixture, traditional p99 was 226.496334 ms versus 2.978875 ms bounded; the bounded p99 was about 76 times lower. Traditional did more work per slice, but much of its reported quantum count is the full-storage scan. This is a result for the specified two-pass batch-all policy, not a generic conventional-engine comparison or an isolated scheduler-overhead claim. The idle traditional fixtures show the baseline cost directly: two scans of 16,777,216 pending records produce 33,554,432 recovery quanta, 167,772,160 charged credits, and about 25 ms per slice even with no active simulation work.

### Fixture notes

- `explosive-lattice` includes a deterministic center source with blast energy 15. The warm-up run is followed by fixture re-preparation, so the source is pending again at measured slice 1; it is not a quiet lattice accidentally left inert.
- `dirty-world-sweep` prepares a static stone/air pattern and has no active headless simulation channels. Its intended dirty-chunk drain and renderer upload are app-side and are not measured by `cascade-bench`. The traditional full-world scans still run, so this descriptor is a useful idle-scan baseline, not an upload benchmark.
- `quiet-world` is intentionally inert. Its traditional cost comes from the required full frontier capture and execution passes, not hidden fixture work.
- `tiny-capacity` uses the descriptor's ready capacity of two per lane. Traditional capture processes pending work beyond that ring; it does not depend on widening the ring.

## Full-size credit calibration

Calibration used the 4096 x 4096 burning-forest and mixed-overload descriptors with a deterministic 64-command-per-slice disturbance stream, 120 warm-up slices, fresh fixture re-preparation, and three repetitions of 1,800 measured slices. Each run admitted 115,200 commands, with zero rejection and coalescing. Each pooled calibration set contains 5,400 slice samples. The 1,000,000-credit candidate was chosen for the sub-3 ms pooled p99 at higher service than lower settings, not as a hard deadline guarantee.

Selected 1,000,000-credit measurements:

| Workload | N | p50 ms | p95 ms | p99 ms | max ms | >4 ms | avg quanta/slice | work quanta | backlog high-water |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| burning-forest + disturbances | 5,400 | 1.086125 | 1.564250 | 2.573875 | 3.776291 | 0 | 62,512.7 | 337,568,634 | 12,672,372 |
| mixed-overload + disturbances | 5,400 | 0.807875 | 1.068125 | 1.092084 | 1.199959 | 0 | 62,515.3 | 337,582,776 | 8,374,984 |

The burning-forest upward sweep, three repetitions and 5,400 samples at each setting:

| Credits | p50 ms | p95 ms | p99 ms | max ms | >4 ms | avg quanta/slice |
|---:|---:|---:|---:|---:|---:|---:|
| 400,000 | 0.495458 | 0.938333 | 1.087333 | 1.571458 | 0 | 25,013.8 |
| 600,000 | 0.709541 | 1.256541 | 1.616083 | 2.573292 | 0 | 37,513.3 |
| 800,000 | 0.878000 | 1.453500 | 2.411500 | 6.654250 | 5 | 50,013.0 |
| 1,000,000 | 1.086125 | 1.564250 | 2.573875 | 3.776291 | 0 | 62,512.7 |
| 1,200,000 | 1.321625 | 1.730541 | 3.083750 | 4.602958 | 3 | 75,012.5 |
| 1,400,000 | 1.558125 | 1.922750 | 3.515667 | 5.475500 | 12 | 87,512.2 |

Tails are not monotonic: the 800,000-credit point had five samples over 4 ms and a 6.654250 ms maximum despite a 2.411500 ms p99; 1,200,000 also had three misses. Preserve these adverse observations. At 1,000,000, neither calibration workload exceeded 4 ms, but this is a finite sample on one machine, not a guarantee. The calibration and validation sets use different disturbance inputs but share the static `burning-forest-v1` and `mixed-overload-v1` descriptors, so fixture-level holdout independence is incomplete.

## Finite burst and recovery probe

Three paired full-size repetitions per policy used mixed-overload, 1,200 seeded commands at up to 64 per slice, then no new commands for the remainder of 1,800 measured slices. Warm-up was 120 slices followed by fresh fixture preparation; policy order alternated B/T, T/B, B/T. All 1,200 commands per run were accepted; rejection and coalescing were zero. No run reached the runner's empty completion predicate within 1,800 slices. For future recovery claims, stable state requires no new input, no pending cells or commands, and unchanged authoritative cell contents for 60 consecutive slices. The saved run has zero empty samples and therefore cannot meet that criterion; recovery remains right-censored beyond 1,800 slices. Persistent water-driven work is activity, not stability.

| Policy | N | p50 ms | p95 ms | p99 ms | max ms | >4 ms | avg quanta/slice | work quanta/run | backlog high-water/end | measured wall/run | complete runs |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| bounded | 5,400 | 0.875375 | 1.140875 | 1.206042 | 4.691583 | 1 | 62,500.2 | 112,500,286 | 8,374,984 / 2,653,611 | 1,474.253 to 1,527.422 ms | 0/3 |
| traditional | 5,400 | 26.936292 | 27.248958 | 42.862708 | 78.405000 | 5,400 | 33,795,795.6 | 60,832,432,090 | 3,236,408 / 272,072 | 48,640.314 to 48,677.901 ms | 0/3 |

Traditional's first measured slice reduced pending backlog to 194,469, then backlog peaked at 3,236,408 and ended at 272,072; it did not settle. Bounded backlog ended at 2,653,611. The distinct service and latency numbers include traditional's two mandatory world sweeps each slice. Completion time is right-censored beyond this 1,800-slice capture.

### Extended bounded recovery follow-up (36,000 slices)

A new release-mode 4096² bounded run used the current 1,000,000-credit profile, mixed-overload-v1, and a finite 1,200-command stream (64 attempted per slice) followed by 36,000 measured slices with no new requests after the stream ended at slice 19. The one-minute system load was 2.41 before and 2.97 after, both below the `<3` quiet-host gate. All 1,200 commands were accepted. Preparation (606 slices / 406 ms) and warm-up (120 slices / 79 ms) were outside measurement.

| Metric | Result |
| --- | ---: |
| Measured slices / wall window | 36,000 / 57.999 s |
| Slices after final input | 35,981 |
| Completed work quanta | 2,117,631,240 |
| Backlog high-water / final pending channels | 8,390,120 / 696,272 |
| Final command backlog | 0 |
| Empty samples / verified stable windows | 0 / 0 |
| Recovery duration | right-censored beyond 36,000 slices and 57.999 s |

Stable means 60 consecutive post-input slices with zero pending channels, zero commands, and zero ready jobs. With no queued work in those slices, authoritative cell contents cannot change; a continuing water-work backlog deliberately does not count as settled. This run never had even one empty sample, so it neither drained nor reached the stable criterion. The nonzero final backlog shows unresolved work, but this capture does not isolate which material or rule sustained it; do not attribute the remaining backlog solely to flowing water. Raw CSV was streamed to a summary and is not committed.

## Method and machine disclosure

The benchmark measures wall-clock `World::step` time, not thread CPU time; OS preemption is included. Fixture preparation and warm-up are timed separately from measured slices. When warm-up is enabled, the fixture is prepared again before measured slice 1; world allocation is excluded. The benchmark's `complete` field means no pending channels or commands at that sample and is not a proof of long-term stability.

Reference hardware: MacBook Air (Mac14,2), Apple M2, 8 CPU cores, 16 GB RAM, integrated Apple M2 GPU; macOS 27.0.1; built-in 2560 x 1664 display. Rust 1.99.0 (`b940084d7`, aarch64-apple-darwin), Cargo 1.99.0 (`5f94df478`). `Cargo.lock` SHA-256: `c4084ec2080b2c4da29d605573b159b02504f86af481d13c5807e565d419afe7`. Power mode, thermal state, background load, and display refresh/presentation mode were not controlled. Raw CSV captures are not committed.

## Scope and remaining unmet targets

- The headless paired matrix is 300 measured slices per run, not 60 seconds per fixture. The 1,800-slice calibrations and burst/recovery capture also fall short of the charter's 60-second target.
- This two-pass traditional policy clearly exceeds 4 ms on the full-size idle and active cases, and the overloaded burning-forest p99 reduction is greater than twofold. That observed comparison is specific to this defined full-scan baseline; it is not evidence about optimized conventional engines or interactive frame latency.
- The 1,200-command burst did not resolve or reach a stable/empty state in 1,800 slices for either policy.
- At the time this report was published, no native smoke had been run after the full-frontier correction. The subsequent full-size M2 release comparison is recorded in [`docs/performance.md`](../../docs/performance.md).
- At capture time the app defaulted to 20,000 and capped at 100,000 credits; this headless-evidence change did not modify `crates/app`. The subsequent profile-backed app integration is recorded in [`docs/performance.md`](../../docs/performance.md).
- Full-size interactive rendering, GPU/upload behavior, sustained frame intervals, UI feedback, and 60-second acceptance remain unmeasured.

Credits cap accounted work only in bounded mode and are not elapsed-time deadlines. Results are empirical on one machine.
