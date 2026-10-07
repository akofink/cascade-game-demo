# Headless recovery and quantum-cost follow-up v5

Status: water-direction correction and recovery repricing are merged in `50a0fc5`; the 1,000,000-credit profile is unchanged; finite-burst recovery remains unresolved.

## Sampled operation timing

An opt-in `quantum-timing` diagnostic run used the 4096² `burning-forest-v1` fixture, bounded scheduler, 1,000,000 credits/slice, 120 warm-up slices, 1,800 measured slices, and 115,200 seeded disturbance commands. The one-minute host load was 1.77 before and 2.03 after. Per-operation timers sampled one in 64 operations and scaled the sample; these values are for relative attribution only. Timer overhead and unsampled scheduler/queue work mean the reported slice durations are not acceptance evidence.

| Diagnostic | Value |
|---|---:|
| Completed work quanta | 75,230,136 total; 41,794.5/slice |
| Mean evaluations / blasts / recoveries / commands | 30,689.7 / 14.3 / 11,026.6 / 64 per slice |
| Mean selection probes | 41,805.9 per slice |
| Credits used | 1,000,000/slice |
| Estimated evaluation body | 39.4 ns/operation |
| Estimated blast body | 79.1 ns/operation |
| Estimated recovery body | 20.0 ns/operation |
| Estimated command body | 382.3 ns/operation |

The timing feature's measured-slice p99/max were 2.769/3.920 ms in this diagnostic, but these are instrumented values from a different fixture than the native sustained capture and are not used to claim that the 4 ms target is met. The uninstrumented bounded FIFO capture remains the acceptance source: see [native sustained capture v3](native-sustained-v3.md).

At 20 credits, recovery is priced conservatively against the estimated operation-body costs. The 1M allowance is retained, but measured work throughput fell about 26.9% from the earlier 58,464 to 42,757 average quanta/slice comparison. Repricing alone therefore does not solve the finite-burst target.

## Finite-burst outcome

The accepted 72,000-slice, 4096² mixed-overload burst accepted all 1,200 commands but ended with 267,243 pending channels and no 60-consecutive-slice stable window. Recovery is unmet.

A longer 120,000-slice run was diagnostic only: its one-minute load rose from 1.85 before to 3.43 after, so it is excluded from accepted timing results. It accepted the same 1,200-command burst, measured 5,128,543,712 completed work quanta (42,738/slice), and ended with 97,468 pending channels and no stable window. The reported pending-channel material breakdown was:

| Material | Pending channels |
|---|---:|
| Air | 20,237 |
| Stone | 0 |
| Wood | 33 |
| Sand | 77,188 |
| Explosive | 0 |
| Water | 10 |
| **Total** | **97,468** |

The finite water-bounce cause was addressed by retaining horizontal direction; this late backlog is predominantly sand, suggesting continued granular settling is now the dominant residual activity. The material census does not prove that sand is the only source of delay. The load-gated 72,000-slice result is the accepted recovery evidence; the 120,000-slice probe is explicitly exploratory and has no accepted timing status.

## Outcome and remaining gaps

- The water oscillation regression test verifies that horizontally flowing water settles without immediate reversal.
- The 1,000,000-credit profile and its application default remain unchanged.
- Recovery cost 20 materially reduces available work quanta/slice versus the prior cost-4 profile; this is disclosed rather than represented as a free improvement.
- No measured stable window exists. The 4 ms native simulation p99/max target is also unmet, although three accepted windowed runs improved the maxima relative to the previous source. See [native sustained capture v3](native-sustained-v3.md).
- The all-fixture sustained suite remains outside this single-fixture task and is not claimed complete here.
