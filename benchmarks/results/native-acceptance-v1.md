# Native all-fixture acceptance suite v1

Status: complete; 72/72 accepted captures; 67 rejected attempts.

App source revision: `d05cb91705280ba9111d1fd217ed4aaa1993cc5d`  
Suite runner revision: `3433494628592e82a0fcf43bb48a52945db82cf6`  
Suite started: 2026-10-06T23:51:33-04:00  
Reference host: `akmac`, MacBook Air (Mac14,2), Apple M2/8 CPU cores/16 GB RAM/integrated GPU, macOS 27.0.1; Rust `rustc 1.99.0 (b940084d7 2026-09-28)`.
Capture: release app, 4096² world, 1920×1080, 60 Hz target, profile `m2-16gb-v3`; Cargo.lock SHA-256 `d064050f2b9044bb0eecfb2b7ffee1b18644dd676e192f7b7d0b0008d42db06b`.
Power mode, thermal state, background activity, and physical display refresh are not controlled.

Each fixture/policy cell requires three accepted 60-second captures. A run is accepted only when one-minute load is below 3 both immediately before and after the process, the app exits successfully with `SMOKE_RESULT ok` after its 60-second capture timer, reports the requested fixture/policy at 1920×1080 with nonzero frame samples, has no interval drops, and has complete slice telemetry. Frame counts are actual measured loop intervals, not an assumed 60 Hz count. Rejected attempts are retained below and never contribute to accepted percentiles.

## Accepted captures

| Fixture | Policy | Rep | Load before/after | Frames | Frame p99 ms | Max ms | >33.3 ms | Slice p99 ms | Slice max ms |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| quiet-world | traditional | 1 | 2.40/2.66 | 1190 | 66.646 | 67.612 | 1190 | 48.878 | 68.663 |
| quiet-world | bounded-focus | 1 | 2.96/2.01 | 3600 | 17.260 | 18.255 | 0 | 0.004 | 0.026 |
| quiet-world | bounded-fifo | 1 | 2.01/2.40 | 3600 | 17.337 | 19.428 | 0 | 0.004 | 0.018 |
| explosive-lattice | bounded-focus | 1 | 1.83/2.30 | 3594 | 18.140 | 33.386 | 1 | 0.004 | 0.016 |
| explosive-lattice | bounded-fifo | 1 | 2.30/2.31 | 3600 | 18.118 | 21.131 | 0 | 0.003 | 0.019 |
| explosive-lattice | traditional | 1 | 2.31/2.15 | 1140 | 67.721 | 69.408 | 1140 | 51.314 | 52.370 |
| sand-release | bounded-focus | 1 | 2.15/1.94 | 3599 | 19.453 | 35.669 | 1 | 6.277 | 7.321 |
| sand-release | bounded-fifo | 1 | 1.94/2.71 | 3598 | 20.261 | 33.191 | 0 | 4.275 | 4.756 |
| sand-release | traditional | 1 | 2.71/2.52 | 486 | 184.481 | 187.021 | 486 | 175.551 | 180.049 |
| reservoir-breach | bounded-focus | 1 | 2.52/1.87 | 3597 | 18.571 | 33.895 | 2 | 4.181 | 6.237 |
| reservoir-breach | bounded-fifo | 1 | 1.87/2.35 | 3599 | 18.286 | 32.601 | 0 | 3.949 | 4.901 |
| reservoir-breach | traditional | 1 | 2.35/2.43 | 1182 | 66.993 | 68.987 | 1182 | 50.296 | 68.048 |
| burning-forest | bounded-focus | 1 | 2.84/2.09 | 3598 | 18.475 | 34.238 | 2 | 4.965 | 9.410 |
| burning-forest | bounded-fifo | 1 | 2.09/2.07 | 3599 | 19.300 | 30.693 | 0 | 5.430 | 12.147 |
| burning-forest | traditional | 1 | 2.07/2.51 | 1081 | 233.513 | 311.131 | 1081 | 230.797 | 306.516 |
| dirty-world-sweep | bounded-focus | 1 | 2.51/2.01 | 3599 | 18.053 | 33.839 | 1 | 0.004 | 0.015 |
| dirty-world-sweep | bounded-fifo | 1 | 2.01/2.37 | 3600 | 17.975 | 18.455 | 0 | 0.003 | 0.102 |
| dirty-world-sweep | traditional | 1 | 2.37/2.25 | 1177 | 66.948 | 68.110 | 1177 | 50.417 | 63.728 |
| tiny-capacity | bounded-focus | 1 | 2.25/1.90 | 3598 | 20.723 | 32.419 | 0 | 2.081 | 3.839 |
| tiny-capacity | bounded-fifo | 1 | 1.90/2.55 | 3599 | 19.417 | 24.697 | 0 | 2.561 | 2.811 |
| tiny-capacity | traditional | 1 | 2.55/2.41 | 1160 | 67.219 | 69.088 | 1160 | 50.603 | 110.285 |
| mixed-overload | bounded-focus | 1 | 2.92/2.50 | 3597 | 18.274 | 34.549 | 2 | 4.741 | 6.487 |
| mixed-overload | bounded-fifo | 1 | 2.50/2.57 | 3597 | 18.765 | 34.512 | 2 | 4.389 | 8.109 |
| mixed-overload | traditional | 1 | 2.57/2.44 | 926 | 112.026 | 129.143 | 926 | 100.142 | 113.091 |
| quiet-world | traditional | 2 | 2.44/2.09 | 1182 | 66.874 | 68.178 | 1182 | 50.347 | 72.202 |
| quiet-world | bounded-fifo | 2 | 2.09/2.46 | 3599 | 17.912 | 32.330 | 0 | 0.003 | 0.009 |
| quiet-world | bounded-focus | 2 | 2.46/2.44 | 3596 | 18.004 | 34.253 | 1 | 0.004 | 0.013 |
| explosive-lattice | traditional | 2 | 2.44/2.75 | 1177 | 66.888 | 68.484 | 1177 | 50.576 | 72.282 |
| explosive-lattice | bounded-fifo | 2 | 2.75/2.68 | 3598 | 18.087 | 32.619 | 0 | 0.004 | 0.019 |
| explosive-lattice | bounded-focus | 2 | 2.68/2.14 | 3598 | 18.048 | 32.340 | 0 | 0.004 | 0.012 |
| sand-release | traditional | 2 | 2.14/2.20 | 487 | 185.455 | 196.415 | 487 | 177.320 | 178.299 |
| sand-release | bounded-fifo | 2 | 2.20/2.80 | 3595 | 20.137 | 34.727 | 3 | 4.214 | 4.655 |
| sand-release | bounded-focus | 2 | 2.80/2.80 | 3600 | 19.601 | 23.572 | 0 | 6.074 | 7.513 |
| reservoir-breach | traditional | 2 | 2.80/2.70 | 1177 | 67.127 | 68.887 | 1177 | 50.616 | 75.495 |
| reservoir-breach | bounded-fifo | 2 | 2.70/2.96 | 3599 | 18.352 | 35.967 | 1 | 3.958 | 4.665 |
| reservoir-breach | bounded-focus | 2 | 2.96/2.45 | 3600 | 18.470 | 19.893 | 0 | 4.066 | 5.547 |
| burning-forest | traditional | 2 | 2.92/2.88 | 1084 | 234.458 | 297.531 | 1084 | 230.297 | 292.963 |
| burning-forest | bounded-fifo | 2 | 2.93/2.89 | 3596 | 19.218 | 34.757 | 2 | 5.137 | 6.383 |
| burning-forest | bounded-focus | 2 | 2.89/2.46 | 3599 | 18.546 | 34.978 | 1 | 4.986 | 6.750 |
| dirty-world-sweep | traditional | 2 | 2.46/2.72 | 1186 | 66.703 | 68.580 | 1186 | 49.013 | 65.551 |
| dirty-world-sweep | bounded-fifo | 2 | 2.90/2.18 | 3598 | 18.042 | 32.706 | 0 | 0.003 | 0.178 |
| dirty-world-sweep | bounded-focus | 2 | 2.18/2.40 | 3598 | 18.029 | 33.401 | 1 | 0.005 | 0.018 |
| tiny-capacity | traditional | 2 | 2.40/2.11 | 1164 | 67.798 | 69.236 | 1164 | 50.606 | 109.975 |
| tiny-capacity | bounded-fifo | 2 | 2.11/2.01 | 3599 | 19.348 | 31.721 | 0 | 2.554 | 2.638 |
| tiny-capacity | bounded-focus | 2 | 2.01/2.36 | 3598 | 20.209 | 39.499 | 1 | 2.072 | 2.432 |
| mixed-overload | traditional | 2 | 2.95/2.33 | 937 | 109.811 | 121.147 | 937 | 99.571 | 109.963 |
| mixed-overload | bounded-fifo | 2 | 2.33/2.63 | 3600 | 18.499 | 24.041 | 0 | 4.279 | 4.764 |
| mixed-overload | bounded-focus | 2 | 2.63/2.79 | 3600 | 19.071 | 21.220 | 0 | 6.072 | 6.908 |
| quiet-world | bounded-focus | 3 | 2.79/2.41 | 3595 | 18.184 | 34.002 | 2 | 0.005 | 0.009 |
| quiet-world | bounded-fifo | 3 | 2.41/2.62 | 3598 | 18.063 | 32.695 | 0 | 0.003 | 0.007 |
| quiet-world | traditional | 3 | 2.62/2.35 | 1184 | 66.912 | 76.349 | 1184 | 50.354 | 57.106 |
| explosive-lattice | bounded-focus | 3 | 2.35/2.01 | 3598 | 18.118 | 33.853 | 1 | 0.005 | 0.043 |
| explosive-lattice | bounded-fifo | 3 | 2.01/2.92 | 3598 | 18.140 | 32.628 | 0 | 0.003 | 0.028 |
| explosive-lattice | traditional | 3 | 2.99/2.56 | 1186 | 66.830 | 67.674 | 1186 | 50.274 | 69.624 |
| sand-release | bounded-focus | 3 | 2.56/2.12 | 3597 | 19.079 | 33.432 | 1 | 4.538 | 7.238 |
| sand-release | bounded-fifo | 3 | 2.12/1.85 | 3599 | 20.298 | 31.605 | 0 | 4.238 | 5.643 |
| sand-release | traditional | 3 | 1.85/2.25 | 487 | 185.208 | 195.456 | 487 | 176.450 | 187.044 |
| reservoir-breach | bounded-focus | 3 | 2.25/2.06 | 3599 | 18.453 | 33.351 | 1 | 4.242 | 5.445 |
| reservoir-breach | bounded-fifo | 3 | 2.06/2.25 | 3596 | 18.527 | 34.592 | 1 | 4.072 | 5.566 |
| reservoir-breach | traditional | 3 | 2.25/2.97 | 1186 | 66.713 | 67.489 | 1186 | 49.543 | 83.048 |
| burning-forest | bounded-focus | 3 | 2.84/2.69 | 3594 | 19.104 | 34.607 | 6 | 5.072 | 6.873 |
| burning-forest | bounded-fifo | 3 | 2.69/1.93 | 3598 | 19.372 | 32.581 | 0 | 5.184 | 10.969 |
| burning-forest | traditional | 3 | 1.93/1.91 | 1087 | 234.045 | 297.031 | 1087 | 229.762 | 292.642 |
| dirty-world-sweep | bounded-focus | 3 | 1.91/1.66 | 3600 | 17.946 | 21.402 | 0 | 0.004 | 0.018 |
| dirty-world-sweep | bounded-fifo | 3 | 1.66/2.30 | 3596 | 18.039 | 33.306 | 0 | 0.003 | 0.036 |
| dirty-world-sweep | traditional | 3 | 2.30/2.33 | 1183 | 66.880 | 68.984 | 1183 | 50.281 | 70.866 |
| tiny-capacity | bounded-focus | 3 | 2.86/2.04 | 3598 | 20.643 | 33.713 | 1 | 2.087 | 2.416 |
| tiny-capacity | bounded-fifo | 3 | 2.04/2.25 | 3599 | 19.227 | 32.130 | 0 | 2.538 | 2.727 |
| tiny-capacity | traditional | 3 | 2.25/2.77 | 1160 | 67.303 | 70.340 | 1160 | 50.947 | 97.656 |
| mixed-overload | bounded-focus | 3 | 2.88/2.05 | 3597 | 18.960 | 36.250 | 2 | 5.929 | 7.287 |
| mixed-overload | bounded-fifo | 3 | 2.05/2.67 | 3598 | 18.701 | 33.815 | 2 | 4.455 | 5.185 |
| mixed-overload | traditional | 3 | 2.67/2.69 | 934 | 109.697 | 120.552 | 934 | 99.581 | 109.607 |

## Section 15 checklist rows

| Fixture | Policy | Accepted runs | Frame p99 ≤20 ms | Slice p99 ≤4 ms | >33.3 ms count / max disclosed |
|---|---|---:|---|---|---|
| quiet-world | bounded-focus | 3/3 | met | met | reported |
| quiet-world | bounded-fifo | 3/3 | met | met | reported |
| quiet-world | traditional | 3/3 | unmet | unmet | reported |
| explosive-lattice | bounded-focus | 3/3 | met | met | reported |
| explosive-lattice | bounded-fifo | 3/3 | met | met | reported |
| explosive-lattice | traditional | 3/3 | unmet | unmet | reported |
| sand-release | bounded-focus | 3/3 | met | unmet | reported |
| sand-release | bounded-fifo | 3/3 | unmet | unmet | reported |
| sand-release | traditional | 3/3 | unmet | unmet | reported |
| reservoir-breach | bounded-focus | 3/3 | met | unmet | reported |
| reservoir-breach | bounded-fifo | 3/3 | met | unmet | reported |
| reservoir-breach | traditional | 3/3 | unmet | unmet | reported |
| burning-forest | bounded-focus | 3/3 | met | unmet | reported |
| burning-forest | bounded-fifo | 3/3 | met | unmet | reported |
| burning-forest | traditional | 3/3 | unmet | unmet | reported |
| dirty-world-sweep | bounded-focus | 3/3 | met | met | reported |
| dirty-world-sweep | bounded-fifo | 3/3 | met | met | reported |
| dirty-world-sweep | traditional | 3/3 | unmet | unmet | reported |
| tiny-capacity | bounded-focus | 3/3 | unmet | met | reported |
| tiny-capacity | bounded-fifo | 3/3 | met | met | reported |
| tiny-capacity | traditional | 3/3 | unmet | unmet | reported |
| mixed-overload | bounded-focus | 3/3 | met | unmet | reported |
| mixed-overload | bounded-fifo | 3/3 | met | unmet | reported |
| mixed-overload | traditional | 3/3 | unmet | unmet | reported |

## Interaction feedback samples

These CPU-side event/action-to-visible-upload p95 values use the app's documented method. Counts below 30 are descriptive only; quiet-world intentionally has no scripted interaction samples.

| Fixture | Policy | Rep | Camera n / p95 ms | Paint n / p95 ms | Ignite n / p95 ms | Detonate n / p95 ms |
|---|---|---:|---:|---:|---:|---:|
| quiet-world | traditional | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| quiet-world | bounded-focus | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| quiet-world | bounded-fifo | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| explosive-lattice | bounded-focus | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| explosive-lattice | bounded-fifo | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| explosive-lattice | traditional | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | bounded-focus | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | bounded-fifo | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | traditional | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-focus | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-fifo | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | traditional | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-focus | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-fifo | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | traditional | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| dirty-world-sweep | bounded-focus | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| dirty-world-sweep | bounded-fifo | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| dirty-world-sweep | traditional | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| tiny-capacity | bounded-focus | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| tiny-capacity | bounded-fifo | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| tiny-capacity | traditional | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| mixed-overload | bounded-focus | 1 | 240 / 17.46 | 240 / 17.42 | 240 / 17.35 | 240 / 17.38 |
| mixed-overload | bounded-fifo | 1 | 240 / 17.57 | 240 / 17.81 | 240 / 17.77 | 240 / 17.73 |
| mixed-overload | traditional | 1 | 240 / 67.44 | 240 / 67.27 | 240 / 67.92 | 240 / 133.92 |
| quiet-world | traditional | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| quiet-world | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| quiet-world | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| explosive-lattice | traditional | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| explosive-lattice | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| explosive-lattice | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | traditional | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | traditional | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | traditional | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| dirty-world-sweep | traditional | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| dirty-world-sweep | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| dirty-world-sweep | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| tiny-capacity | traditional | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| tiny-capacity | bounded-fifo | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| tiny-capacity | bounded-focus | 2 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| mixed-overload | traditional | 2 | 240 / 67.79 | 240 / 67.43 | 240 / 67.24 | 240 / 134.45 |
| mixed-overload | bounded-fifo | 2 | 240 / 17.75 | 240 / 17.83 | 240 / 17.75 | 240 / 17.78 |
| mixed-overload | bounded-focus | 2 | 240 / 17.99 | 240 / 18.04 | 240 / 18.52 | 240 / 18.30 |
| quiet-world | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| quiet-world | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| quiet-world | traditional | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| explosive-lattice | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| explosive-lattice | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| explosive-lattice | traditional | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| sand-release | traditional | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| reservoir-breach | traditional | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| burning-forest | traditional | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| dirty-world-sweep | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| dirty-world-sweep | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| dirty-world-sweep | traditional | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| tiny-capacity | bounded-focus | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| tiny-capacity | bounded-fifo | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| tiny-capacity | traditional | 3 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| mixed-overload | bounded-focus | 3 | 240 / 17.97 | 240 / 18.24 | 240 / 18.24 | 240 / 17.82 |
| mixed-overload | bounded-fifo | 3 | 240 / 17.83 | 240 / 17.80 | 240 / 17.91 | 240 / 17.94 |
| mixed-overload | traditional | 3 | 240 / 67.38 | 240 / 67.96 | 240 / 67.27 | 240 / 134.66 |

## Resumption notes

- Confirmed explosive-lattice/bounded-focus rep 1 failed before capture start because the native window was occluded; no timing data was collected.
- Confirmed explosive-lattice/bounded-focus rep 1 failed before capture start because the native window was occluded; no timing data was collected.

## Rejected attempts

| Fixture | Policy | Rep | Attempt | Load before/after | Reason | Log |
|---|---|---:|---:|---:|---|---|
| quiet-world | bounded-focus | 1 | 1 | 2.19/3.04 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/quiet-world-bounded-focus-rep1-attempt1.log` |
| quiet-world | bounded-focus | 1 | 2 | 3.04/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-focus | 1 | 3 | 3.04/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-focus | 1 | 4 | 2.87/3.47 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/quiet-world-bounded-focus-rep1-attempt4.log` |
| quiet-world | bounded-focus | 1 | 5 | 3.47/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-focus | 1 | 6 | 3.27/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-focus | 1 | 7 | 3.25/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-focus | 1 | 8 | 3.07/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-focus | 1 | 9 | 3.06/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 1 | 1 | 1.68/1.41 | window occluded before timing capture; no capture timer started (startup retry permitted) | `benchmarks/tmp/native-acceptance/explosive-lattice-bounded-focus-rep1-attempt1.log` |
| explosive-lattice | bounded-focus | 1 | 2 | 1.48/1.82 | window occluded before timing capture; no capture timer started (startup retry permitted) | `benchmarks/tmp/native-acceptance/explosive-lattice-bounded-focus-rep1-attempt2.log` |
| burning-forest | bounded-focus | 1 | 1 | 2.43/3.56 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/burning-forest-bounded-focus-rep1-attempt1.log` |
| burning-forest | bounded-focus | 1 | 2 | 3.56/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 3 | 3.27/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 4 | 3.09/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 5 | 3.08/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 6 | 3.55/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 7 | 3.35/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 8 | 3.64/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 9 | 3.43/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 10 | 3.23/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 11 | 3.22/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 12 | 3.36/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 13 | 3.09/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 14 | 3.00/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 1 | 15 | 3.00/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 1 | 1 | 2.41/4.09 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/mixed-overload-bounded-focus-rep1-attempt1.log` |
| mixed-overload | bounded-focus | 1 | 2 | 4.09/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 1 | 3 | 4.01/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 1 | 4 | 3.77/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 1 | 5 | 3.70/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 1 | 6 | 3.49/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 1 | 7 | 3.29/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 1 | 8 | 3.34/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 1 | 9 | 3.40/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 1 | 10 | 3.20/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 1 | 11 | 3.03/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 1 | 12 | 3.19/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 1 | 13 | 3.09/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 2 | 1 | 2.45/3.04 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/burning-forest-traditional-rep2-attempt1.log` |
| burning-forest | traditional | 2 | 2 | 3.04/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 2 | 3 | 2.87/3.09 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/burning-forest-traditional-rep2-attempt3.log` |
| burning-forest | traditional | 2 | 4 | 3.09/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 2 | 5 | 3.00/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 2 | 6 | 3.00/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 2 | 1 | 2.88/3.40 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/burning-forest-bounded-fifo-rep2-attempt1.log` |
| burning-forest | bounded-fifo | 2 | 2 | 3.40/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 2 | 3 | 3.29/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-fifo | 2 | 4 | 3.10/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 1 | 2.36/3.13 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/mixed-overload-traditional-rep2-attempt1.log` |
| mixed-overload | traditional | 2 | 2 | 3.13/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 3 | 3.12/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 4 | 2.95/3.03 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/mixed-overload-traditional-rep2-attempt4.log` |
| mixed-overload | traditional | 2 | 5 | 3.03/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | traditional | 3 | 1 | 2.92/3.08 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/explosive-lattice-traditional-rep3-attempt1.log` |
| explosive-lattice | traditional | 3 | 2 | 3.08/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | bounded-focus | 3 | 1 | 2.97/3.00 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/burning-forest-bounded-focus-rep3-attempt1.log` |
| burning-forest | bounded-focus | 3 | 2 | 3.00/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 3 | 1 | 2.77/3.35 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/mixed-overload-bounded-focus-rep3-attempt1.log` |
| mixed-overload | bounded-focus | 3 | 2 | 3.35/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 3 | 3 | 3.16/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 3 | 4 | 3.39/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 3 | 5 | 3.28/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 3 | 6 | 3.25/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 3 | 7 | 3.31/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 3 | 8 | 3.05/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | bounded-focus | 3 | 9 | 3.04/n/a | pre-run one-minute load not below 3 | `-` |

## Interpretation and limits

Frame intervals are native windowed event-loop intervals; they include presentation pacing and are not GPU duration. Slice CPU values are monotonic wall-time samples around the app's simulation dispatch, not thread CPU time. Fixture captures do not add external disturbances or player actions, except mixed-overload, which uses the app's scripted sustained-overload action stream to exercise focus latency. The load gate does not exclude all OS/driver interference.

Raw per-attempt logs are retained locally under `benchmarks/tmp/` and are not committed.
