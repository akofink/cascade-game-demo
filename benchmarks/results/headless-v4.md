# Full-size headless calibration v4: focus membership cache

Status: three bounded repetitions per calibration fixture; cache-enabled source; 4096 x 4096 world; 1,000,000 credits/slice.

## Method

Release `cascade-bench` on the cache-enabled task branch. Each fixture used a 120-slice warm-up, then fresh fixture preparation and 1,800 measured slices per repetition. A deterministic disturbance stream attempted 64 commands per slice (115,200 accepted per run; zero rejected/coalesced). The two fixtures are `burning-forest-v1` and `mixed-overload-v1`. Load average was 1.67–2.57 before/after the six runs (all below 3). Timing is monotonic wall-clock around `World::step`, not thread CPU time, so OS preemption remains in the samples. Preparation and warm-up are excluded. The CSV records evaluation, blast, recovery, and command counts per slice in addition to wall time and charged credits; raw captures are not committed.

## Results

| Fixture | Rep | Slices | p50 ms | p95 ms | p99 ms | max ms | >4 ms | Avg work quanta/slice | Avg credits used |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Burning forest | 1 | 1,800 | 1.501 | 2.244 | 2.760 | 4.318 | 1 | 50,630.4 | 1,000,000 |
| Burning forest | 2 | 1,800 | 1.504 | 2.205 | 2.784 | 4.258 | 1 | 50,630.4 | 1,000,000 |
| Burning forest | 3 | 1,800 | 1.505 | 2.315 | 2.990 | 9.767 | 6 | 50,630.4 | 1,000,000 |
| Mixed overload | 1 | 1,800 | 1.385 | 1.559 | 1.649 | 1.708 | 0 | 55,712.1 | 1,000,000 |
| Mixed overload | 2 | 1,800 | 1.389 | 1.563 | 1.652 | 1.697 | 0 | 55,712.1 | 1,000,000 |
| Mixed overload | 3 | 1,800 | 1.394 | 1.566 | 1.657 | 2.017 | 0 | 55,712.1 | 1,000,000 |

Pooled across 5,400 slices per fixture:

| Fixture | p50 ms | p95 ms | p99 ms | max ms | >4 ms | Avg work quanta/slice |
|---|---:|---:|---:|---:|---:|---:|
| Burning forest | 1.503 | 2.262 | 2.839 | 9.767 | 8 | 50,630.4 |
| Mixed overload | 1.390 | 1.563 | 1.652 | 2.017 | 0 | 55,712.1 |

Before the cache, the same six-run method/source immediately preceding the cache measured burning-forest pooled p50/p95/p99/max of 1.718/2.908/3.907/8.275 ms (47 samples >4 ms), and mixed-overload 1.602/1.898/2.077/2.271 ms (zero >4 ms). Per-slice work throughput was unchanged: 50,630.4 and 55,712.1 quanta/slice respectively before and after. Thus this hot-path optimization improves p99 without reducing the credit allowance or scheduler service; it does not eliminate adverse wall-clock outliers.

## Attribution and interpretation

The pre-cache full-size native attribution captures showed about 33–35k evaluation quanta plus 23–33k recovery quanta per maximum-cost slice, with 59–67k charged selection probes. Evaluation's 24-credit charge and per-quantum selection dominate the allowed work in these slices. Focus-enabled queue admissions repeatedly asked whether cells in the same logical chunk matched one of up to eight focus regions; the previous implementation rescanned those regions on each query. Recovery could also recheck membership separately for evaluation and blast channels on one cell.

The initial fix stored a 16-byte membership-cache entry per logical chunk, valid only for the current focus-region epoch, and reused a single recovery membership result for both channels. Focus/viewport changes, reset, and enable changes invalidate the cache. A follow-up packs the epoch and membership bit into one 32-bit entry, cutting cache storage from 262,144 to 65,536 bytes at 4096². No scheduler ordering, credit, or simulation rules changed. The 1,000,000-credit profile remains unchanged, so this storage optimization has no direct throughput cost in work quanta.

The headless 4 ms p99 goal is met by these pooled samples, but the strict all-sample maximum is not: eight burning-forest samples exceed 4 ms, including one 9.767 ms adverse outlier. These timings include operating-system interference and are not a wall-clock bound. A post-cache native 60-second capture meeting both the quiet-load and unoccluded-window requirements has not yet been obtained; no native after-fix claim follows from this headless table.
