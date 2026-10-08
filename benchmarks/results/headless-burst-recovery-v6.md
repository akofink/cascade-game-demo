# Headless finite-burst recovery v6

Status: accepted headless recovery at the 1,000,000-credit profile. No simulator or app source changed for this result.

## Finding

The 72,000-slice endpoint was right-censored, not a persistent-work deadlock. The 4096² mixed-overload fixture starts with a large active sand/water/fire workload. Sand movement continues after input ends and wakes nearby evaluation work as grains descend through the fixture. The pending set therefore changes and regenerates while this finite granular cascade settles. The accepted 72,000-slice share-5 result stopped with 120,560 channels; a longer 120,000-slice diagnostic had 31,974. A repeated 180,000-slice run shows all channels drain and a stable window begins at slice 134,367.

No evidence of duplicate channels, water reversal, or lane starvation was found. Pending evaluation and blast work is coalesced per cell; water's horizontal direction is retained by the already-merged rule correction. At slice 72,000 the scheduler was servicing about 25,637 evaluations and 17,098 recovery probes per slice. The declining pending totals and eventual zero backlog demonstrate continued service, not a stuck queue. An inert-neighbor wake-filter experiment was reverted because it changed scheduling and did not improve the measured pending backlog.

## Accepted burst

Command, from the repository root:

```sh
cargo build --release -p cascade-bench
target/release/cascade-bench --fixture mixed-overload --policy bounded \
  --width 4096 --height 4096 --budget 1000000 --slices 180000 \
  --disturbances 1200 --disturbances-per-slice 64
```

| Property | Result |
|---|---:|
| Source revision | `9bebdca` (`cascade-sim` and `cascade-app` source unchanged since offscreen follow-up `dd05773`) |
| Fixture / seed | mixed-overload v1 / 1,128,354,568 |
| Profile | 1,000,000 credits/slice; focus share 5%; background minimum 20%; selection cost 1 |
| Quiet-host load, before / after | 2.25 / 2.43 (accepted, both below 3) |
| Measured slices | 180,000 |
| Accepted / coalesced / rejected commands | 1,200 / 0 / 0 |
| Completed work quanta through stable window | 5,742,534,591 |
| Work throughput through stable window | 42,737.7 quanta/slice |
| First completion and stable-window start | slice 134,367 |
| Wall time to stable window | 92.474 s |
| Stable criterion | 60 consecutive no-input slices with zero pending channels, ready jobs, commands, and fixture/reset work |
| Final pending channels | 0 |
| Final state hash | `5c40d3fa4a07bdd6` |

The burst work rate is unchanged from the earlier share-5 measurements (about 42,741 quanta/slice); no slice budget, pricing, focus share, or throughput tradeoff was introduced. The run continued to the requested 180,000-slice bound after the stable window so the result includes a complete final state/hash.

The accepted run's backlog was 120,560 at slice 72,000 and 31,974 at slice 120,000, then reached zero at slice 134,367. These points were read from the same run, which passed the quiet-host gate. The final 45,633 slices remained stable.

## Timing and remaining targets

This task made no simulation or app source changes. The published [heavy-fixture offscreen follow-up v1](slice-p99-offscreen-followup-v1.md) therefore remains the source-identical offscreen check: its eight 4096² heavy-fixture/policy captures had frame p99 19.618–19.779 ms; bounded-focus slice p99 was 3.906–4.076 ms and bounded-FIFO 4.206–4.439 ms. These are offscreen diagnostics, not visible-window acceptance. The source identity was verified between `dd05773` and `9bebdca`; a separate fresh app build did not complete under the host's high shared load, so no new offscreen captures are claimed.

Finite-burst recovery is now met for this workload at a measured 134,367-slice / 92.474-second recovery window. Other charter targets are unchanged: the complete all-fixture acceptance matrix remains incomplete, and slice p99 still exceeds 4 ms in several heavy-fixture cells. See the offscreen and windowed reports for exact per-cell status. GPU timing and complete system GPU-memory high-water also remain unavailable/unmeasured as previously disclosed.
