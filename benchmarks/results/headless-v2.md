# Full-size headless results v2

Source revision `5825d77eae4c9b866dd8251c47fcc3e93403a9db`; release `cascade-bench`, Rust/Cargo 1.99.0. All eight section-12 descriptors ran at 4096 x 4096, 1,000,000 credits, 120 warm-up steps, then 300 measured steps, three runs per mode. Mode order alternated B/T, T/B, B/T for each fixture. After warm-up the benchmark re-prepared the fixture, so measurement starts at the descriptor state. Pooled interval sample count is 900 per fixture/mode. Percentiles use nearest lower indexed sample; durations are monotonic wall-clock around `World::step`.

| Fixture | Policy | N | p50 ms | p95 ms | p99 ms | max ms | >4 ms | avg quanta/slice | total quanta | backlog high-water | rejected | coalesced | empty slices |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| quiet-world | bounded | 900 | 0.000041 | 0.000042 | 0.000125 | 0.000458 | 0 | 0.0 | 0 | 0 | 0 | 0 | 900/900 |
| quiet-world | traditional | 900 | 0.000000 | 0.000042 | 0.000042 | 0.000417 | 0 | 0.0 | 0 | 0 | 0 | 0 | 900/900 |
| explosive-lattice | bounded | 900 | 0.000000 | 0.000042 | 0.000042 | 0.002292 | 0 | 0.0 | 27 | 0 | 0 | 0 | 900/900 |
| explosive-lattice | traditional | 900 | 0.000000 | 0.000042 | 0.000042 | 0.002208 | 0 | 0.0 | 15 | 4 | 0 | 0 | 897/900 |
| sand-release | bounded | 900 | 0.292333 | 0.300125 | 0.316334 | 0.352208 | 0 | 62500.0 | 56250006 | 9968750 | 0 | 0 | 0/900 |
| sand-release | traditional | 900 | 0.000041 | 0.000042 | 0.000042 | 0.163333 | 0 | 110.2 | 99204 | 9967232 | 0 | 0 | 0/900 |
| reservoir-breach | bounded | 900 | 0.508333 | 1.130959 | 1.150666 | 1.170583 | 0 | 79979.7 | 71981712 | 2062831 | 0 | 0 | 0/900 |
| reservoir-breach | traditional | 900 | 0.000041 | 0.000042 | 0.000042 | 0.248709 | 0 | 110.2 | 99204 | 2061313 | 0 | 0 | 0/900 |
| burning-forest | bounded | 900 | 1.311959 | 2.609208 | 2.883167 | 3.784958 | 0 | 62500.0 | 56250000 | 7355896 | 0 | 0 | 0/900 |
| burning-forest | traditional | 900 | 1.029916 | 2.498125 | 2.862917 | 8.014834 | 2 | 32480.4 | 29232351 | 6328380 | 0 | 0 | 0/900 |
| dirty-world-sweep | bounded | 900 | 0.000041 | 0.000042 | 0.000042 | 0.000667 | 0 | 0.0 | 0 | 0 | 0 | 0 | 900/900 |
| dirty-world-sweep | traditional | 900 | 0.000000 | 0.000042 | 0.000042 | 0.000458 | 0 | 0.0 | 0 | 0 | 0 | 0 | 900/900 |
| tiny-capacity | bounded | 900 | 0.463125 | 0.484875 | 0.511334 | 0.548208 | 0 | 72428.3 | 65185440 | 8501648 | 0 | 0 | 0/900 |
| tiny-capacity | traditional | 900 | 0.000041 | 0.000042 | 0.000084 | 0.000875 | 0 | 1.5 | 1377 | 8524063 | 0 | 0 | 0/900 |
| mixed-overload | bounded | 900 | 0.328208 | 0.926833 | 0.964083 | 1.007042 | 0 | 62500.0 | 56250000 | 8374807 | 0 | 0 | 0/900 |
| mixed-overload | traditional | 900 | 0.000041 | 0.000042 | 0.000042 | 0.210292 | 0 | 110.2 | 99210 | 8373288 | 0 | 0 | 0/900 |

## Preparation and completion timing

Preparation time includes both fixture preparation passes (one before warm-up and one fresh pass before measurement); world allocation is excluded. Each run used 120 warm-up slices. Warm-up wall time is only those 120 scheduler steps. An empty slice is reported as the runner completion predicate at that sample; fixtures can subsequently receive work only through new commands.

| Fixture | Policy | Prep slices/run | Median prep ms (range) | First empty slice and elapsed time (median) | Measured window if none |
|---|---|---:|---:|---|---:|
| quiet-world | bounded | 606 | 199.891458 (198.853250 to 204.955333) | slice 1, 0.000666 ms |  |
| quiet-world | traditional | 606 | 204.329125 (195.490750 to 205.864875) | slice 1, 0.000625 ms |  |
| explosive-lattice | bounded | 606 | 209.024001 (208.434959 to 211.025667) | slice 1, 0.002250 ms |  |
| explosive-lattice | traditional | 606 | 208.184625 (208.170542 to 209.136709) | slice 2, 0.016458 ms |  |
| sand-release | bounded | 606 | 224.482084 (223.118208 to 227.989667) | not reached in 300 slices | 89.084666 to 89.147042 ms, censored |
| sand-release | traditional | 606 | 223.383875 (223.275125 to 225.098126) | not reached in 300 slices | 0.960584 to 1.005208 ms, censored |
| reservoir-breach | bounded | 606 | 250.953167 (250.743167 to 252.004166) | not reached in 300 slices | 193.028167 to 193.413084 ms, censored |
| reservoir-breach | traditional | 606 | 250.927542 (249.710458 to 252.995542) | not reached in 300 slices | 1.017292 to 1.163875 ms, censored |
| burning-forest | bounded | 606 | 289.880791 (288.861291 to 290.003458) | not reached in 300 slices | 474.109125 to 476.889833 ms, censored |
| burning-forest | traditional | 606 | 290.407792 (289.886791 to 290.549000) | not reached in 300 slices | 384.577583 to 404.838625 ms, censored |
| dirty-world-sweep | bounded | 606 | 342.724708 (341.804750 to 345.786583) | slice 1, 0.000917 ms |  |
| dirty-world-sweep | traditional | 606 | 342.162792 (342.140708 to 343.408083) | slice 1, 0.000708 ms |  |
| tiny-capacity | bounded | 606 | 228.242792 (227.240458 to 229.218249) | not reached in 300 slices | 140.039375 to 142.199000 ms, censored |
| tiny-capacity | traditional | 606 | 227.547083 (227.448084 to 227.556125) | not reached in 300 slices | 0.797625 to 0.813416 ms, censored |
| mixed-overload | bounded | 606 | 255.707958 (255.508167 to 256.437292) | not reached in 300 slices | 116.968583 to 118.038209 ms, censored |
| mixed-overload | traditional | 606 | 255.998791 (255.627167 to 256.410958) | not reached in 300 slices | 0.996083 to 1.026833 ms, censored |

## Upward calibration, full-size saturated workload

Calibration used 4096 x 4096 `burning-forest-v1` and `mixed-overload-v1`, each with 64 seeded disturbance requests per measured slice, 120 warm-up slices and a fresh re-preparation. The selected 1,000,000-credit profile has three repetitions of 1,800 measured slices for each workload. For burning forest, pooled p50/p95/p99/max were 1.106625/1.629417/2.590917/4.107667 ms; 1 of 5,400 samples exceeded 4 ms. Mixed overload p50/p95/p99/max were 0.822416/1.089083/1.113125/1.226584 ms; zero samples exceeded 4 ms. Mean completed work was 62,512 and 62,515 scheduler quanta/slice, respectively. Each run admitted 115,200 commands with zero rejection/coalescing. Backlog peaks: burning forest 12,672,372; mixed overload 8,374,984. The worst workload p99 is within the 2 to 3 ms calibration region, with 1.409083 ms below the 4 ms target. One isolated max exceeded 4 ms, so this is not a hard deadline claim.

### Burning-forest credit sweep

| Credits | p99 ms | max ms | avg quanta/slice | >4 ms |
|---:|---:|---:|---:|---:|
| 400000 | 1.259125 | 1.518916 | 25015.8 | 0 |
| 600000 | 1.802667 | 2.280500 | 37515.6 | 0 |
| 800000 | 2.358375 | 2.988959 | 50015.5 | 0 |
| 1000000 | 2.889084 | 3.757708 | 62515.3 | 0 |
| 1200000 | 3.373833 | 4.452958 | 75015.1 | 1 |
| 1400000 | 4.109583 | 5.561167 | 87514.9 | 4 |

Lower credits reduce per-slice work but also reduce completed work/slice. The 1.4M step exceeded the 4 ms p99 target in its one-run pilot; 1.6M caused a 6.201417 ms p99 on the unperturbed held-out burning-forest fixture, so was rejected. The 1.0M candidate is a conservative choice under calibration and held-out results. The simulator ceiling is 10M. The app currently caps at 100,000 credits and defaults to 20,000, so it cannot use this calibrated candidate until the integration-owned control cap/default is updated.

## Finite burst followed by recovery

Three paired full-size repetitions per mode: mixed-overload, 1,200 seeded commands at up to 64/slice, then no new commands for the remainder of 1,800 measured slices. Warm-up was 120 slices followed by fresh fixture prep; order alternated B/T, T/B, B/T. All 1,200 commands were accepted; rejection/coalescing zero. Neither mode reached an empty/stable state. Bounded backlog peak/end: 8,374,984/2,653,611; traditional: 9,754,779/9,718,117. Work per run: 112,500,286 bounded, 28,828,770 traditional. Measured wall windows: 1,508.275 to 1,510.383 ms bounded; 690.773 to 691.691 ms traditional. Time to stability is censored beyond 1,800 slices.

## Paired comparison and interpretation

Burning forest at 1.0M credits yielded bounded p99 2.883167 ms/max 3.784958 ms, versus traditional p99 2.862917 ms/max 8.014834 ms. Bounded had zero of 900 samples over 4 ms; traditional had 2 of 900. Traditional executed a mean 32,480 quanta/slice versus 62,500 bounded. The p99 improvement target is not met (p99 is effectively similar), despite a few long traditional slices. Treat the charter overloaded-baseline comparison as **inconclusive**, not as a demonstrated twofold responsiveness gain. Other traditional fixtures often drain their finite captured frontier during warm-up, leaving near-zero measured work.

The order-insensitive cross-mode quiet-fixture check passed in `cargo test -p cascade-bench`. Rejected/coalesced counts in the no-disturbance matrix are zero. Raw captures remain uncommitted. See [`docs/performance.md`](../../docs/performance.md) for methods, disclosures and remaining unmet criteria.
