# Player-action focus comparison v1

Status: three quiet-host full-size headless repetitions accepted. The native smoke sampling gap has since been fixed. The follow-up three-policy, 60-second native results are in [native sustained capture v1](native-sustained-v1.md). Native frame and action observations are separate from the headless results below.

## Reproduction

```sh
cargo +1.99.0 run --release -p cascade-bench -- \
  --player-action-comparison --width 4096 --height 4096 \
  --budget 1000000 --slices 4096
```

Code revision: `7c529d2`. Each policy starts from a newly prepared `mixed-overload-v1` world. Preparation and deterministic target selection are excluded from measured wall time. Each repetition runs 4,096 slices per policy, admits one seeded background disturbance every four slices, and attempts paint, ignite, and detonate commands every eight slices. Paint targets come from the inert upper half; the following ignite targets that newly painted wood or explosive. Detonate targets come from fixture explosives and are revalidated at admission. The runner fails unless every policy/action type has at least 30 first-effect and local-settle samples.

Only accept a repetition when `uptime`'s 1-minute load average is below about 3 both before and after. All three included runs satisfy that gate:

| Repetition | Start (EDT) | 1-minute load before | End (EDT) | 1-minute load after |
| --- | --- | ---: | --- | ---: |
| 1 | 2026-10-05 18:36:13 | 2.40 | 18:40:49 | 2.33 |
| 2 | 2026-10-05 19:09:05 | 2.07 | 19:13:41 | 2.17 |
| 3 | 2026-10-05 19:13:49 | 2.23 | 19:18:26 | 2.73 |

Two other 4,096-slice attempts were discarded because their post-run loads were 3.11 and 4.84. An earlier 1,024-slice diagnostic failed the sample-count guard. None of those timings are included below.

Reference machine: MacBook Air (Mac14,2), Apple M2, 16 GB RAM, macOS 27.0.1, Rust/Cargo 1.99.0. Per-run action wall latency is monotonic elapsed time from command admission until the benchmark observes its first-effect or local-settle record; it includes fixed benchmark polling overhead.

## Results

Policy-level values below are medians across the three accepted repetitions; wall latency percentiles are milliseconds. Measured wall time is the median total for 4,096 slices per policy.

| Policy | Measured wall time | First-effect samples; p50 / p95 | Local-settle samples; p50 / p95 | Action-effect probes | Background quanta | Final pending channels | Final hash |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Traditional | 261.122 s | 512; 127.444 / 133.372 ms | 511; 191.542 / 841.602 ms | 8,510 | 867,085,009 | 38,962 | `e0dfb915bcf471da` |
| Bounded FIFO | 6.015 s | 512; 1.417 / 16.518 ms | 458; 1.777 / 649.949 ms | 58,283 | 144,530,901 | 841,055 | `308dba951bacdfac` |
| Bounded focus | 7.998 s | 512; 1.736 / 2.754 ms | 512; 1.745 / 2.764 ms | 1,546 | 126,015,770 | 1,065,285 | `0daeaf9189082885` |

Per-action distributions are medians of each repetition's nearest-rank percentiles. Each focus action type has enough first-effect and settle observations for p50/p95:

| Policy | Action | First effect n; p50 / p95 | Local settle n; p50 / p95 |
| --- | --- | ---: | ---: |
| Traditional | Paint | 171; 127.569 / 132.986 ms | 171; 127.569 / 132.986 ms |
| Traditional | Ignite | 171; 126.864 / 133.405 ms | 170; 434.983 / 858.164 ms |
| Traditional | Detonate | 170; 127.673 / 133.667 ms | 170; 191.542 / 200.019 ms |
| Bounded FIFO | Paint | 171; 1.412 / 16.544 ms | 166; 1.369 / 610.103 ms |
| Bounded FIFO | Ignite | 171; 1.442 / 56.389 ms | 162; 13.685 / 649.949 ms |
| Bounded FIFO | Detonate | 170; 1.404 / 1.796 ms | 130; 1.347 / 714.207 ms |
| Bounded focus | Paint | 171; 1.970 / 2.777 ms | 171; 1.973 / 2.788 ms |
| Bounded focus | Ignite | 171; 1.580 / 2.709 ms | 171; 1.598 / 2.745 ms |
| Bounded focus | Detonate | 170; 1.691 / 2.755 ms | 170; 1.691 / 2.755 ms |

All three repetitions produced identical final hashes and deterministic work/backlog counts for each policy. The action-effect index examined 8,510 candidates in traditional mode over the full run instead of probing every action record for every simulation cell. The regression tests also assert zero candidates for completed actions and unrelated traditional chunks.

## Historical native smoke follow-up

Two release-mode native `--smoke --world-size 4096` runs on main revision `3016f6f` completed with `SMOKE_RESULT ok`, but the bounded-focus window had no detonate samples and all action counts were too small for stable percentiles. This point-in-time native result is superseded by [native sustained capture v1](native-sustained-v1.md), which reports the corrected script and three 60-second repetitions at 1920 x 1080.

The residual investigation disabled unused simulator action tracking in the native app, guarded focus-flag clearing, added a traditional scan/work regression test, and replaced the smoke action stream with paired paint/ignite and distinct detonation targets. Headless benchmark action tracking remains enabled by default.

## Interpretation and limits

Under this headless workload, bounded focus records first effects and local settle for paint, ignite, and detonate with p95 values below 2.8 ms. Traditional action-to-first-effect p95 is about 133 ms in this batch-all workload. These are simulator timestamps, not native frame intervals or display presentation latency, and they do not directly replace the previously observed native traditional p99.

These headless simulator timestamps remain separate from the native frame intervals and presentation feedback in [native sustained capture v1](native-sustained-v1.md). The headless benchmark does not measure camera/UI acknowledgment latency, display scanout, or memory high-water marks.
