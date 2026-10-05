# Headless benchmark summary v1

Release `cascade-bench`; 256 x 256, 4096 credits, 300 measured slices/run, three repetitions per policy/fixture. Mode order alternated B/T, T/B, B/T. Each table row pools 900 per-slice samples. Fixture preparation is excluded and consumed 290 scheduler slices in each run; its wall duration was not captured. Raw CSVs are not committed. Times are milliseconds, computed from nanoseconds; percentile is nearest lower indexed sample after sorting.

| Fixture | Policy | N | p50 ms | p95 ms | p99 ms | max ms | >4 ms | backlog high-water | Rejected | Coalesced | Complete at 300 |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|:---:|
| quiet-world | bounded | 900 | 0.006500 | 0.010750 | 0.010792 | 0.010958 | 0 | 0 | 0 | 0 | yes |
| quiet-world | traditional | 900 | 0.000041 | 0.000042 | 0.000042 | 0.000125 | 0 | 0 | 0 | 0 | yes |
| explosive-lattice | bounded | 900 | 0.004875 | 0.005250 | 0.005250 | 0.005834 | 0 | 0 | 0 | 0 | yes |
| explosive-lattice | traditional | 900 | 0.000000 | 0.000042 | 0.000042 | 0.000416 | 0 | 4 | 0 | 0 | no |
| sand-release | bounded | 900 | 0.001208 | 0.001209 | 0.001291 | 0.005083 | 0 | 64384 | 0 | 0 | no |
| sand-release | traditional | 900 | 0.000041 | 0.000042 | 0.000042 | 0.166292 | 0 | 31744 | 0 | 0 | no |
| reservoir-breach | bounded | 900 | 0.002041 | 0.002333 | 0.002500 | 0.009625 | 0 | 7873 | 0 | 0 | no |
| reservoir-breach | traditional | 900 | 0.009584 | 0.012750 | 0.013083 | 0.056375 | 0 | 1107 | 0 | 0 | no |
| burning-forest | bounded | 900 | 0.003708 | 0.004667 | 0.005583 | 0.008500 | 0 | 25475 | 0 | 0 | no |
| burning-forest | traditional | 900 | 0.000042 | 0.283166 | 0.726583 | 0.779791 | 0 | 50728 | 0 | 0 | no |
| dirty-world-sweep | bounded | 900 | 0.004583 | 0.004833 | 0.004917 | 0.006334 | 0 | 0 | 0 | 0 | yes |
| dirty-world-sweep | traditional | 900 | 0.000000 | 0.000042 | 0.000042 | 0.000042 | 0 | 0 | 0 | 0 | yes |
| tiny-capacity | bounded | 900 | 0.001917 | 0.002000 | 0.002042 | 0.002667 | 0 | 33202 | 0 | 0 | no |
| tiny-capacity | traditional | 900 | 0.000042 | 0.000042 | 0.000084 | 0.000208 | 0 | 33295 | 0 | 0 | no |
| mixed-overload | bounded | 900 | 0.001500 | 0.003208 | 0.003666 | 0.003834 | 0 | 32721 | 0 | 0 | no |
| mixed-overload | traditional | 900 | 0.000042 | 0.001625 | 0.002208 | 0.197167 | 0 | 267 | 0 | 0 | no |

Completion denotes empty pending/command work and finished disturbance stream at the last captured slice, not a stability test. Rejected and coalesced totals sum per-slice disturbance counters; no disturbances were configured in these runs, so both are zero. The benchmark test `cross_mode_order_insensitive_fixture_check_passes` validates the order-insensitive quiet fixture plumbing. This matrix is not 60-second duration testing.

## Finite burst and recovery probe

Three additional mixed-overload runs per mode used a finite 1,200-command stream at up to eight admissions per slice, followed by 1,650 slices with no new requests (1,800 total measured). All 1,200 commands were accepted; rejected/coalesced counts were zero. Bounded backlog peaked at 32,754 and ended at 195; traditional backlog peaked at 12,260 and ended at 194. Neither mode reached an empty/stable state by the end of the 1,800 slices, so recovery-to-completion is unmet. This probe used bounded-first batch ordering, not the alternating order of the fixture matrix; it is supporting evidence only. See [`docs/performance.md`](../../docs/performance.md) for caveats.
