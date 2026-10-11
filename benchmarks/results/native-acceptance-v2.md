# Native all-fixture acceptance suite v2

Status: complete; 72/72 accepted captures; 123 rejected attempts.

App source revision: `237aaa373e101182c908c7bc2fcde5b8a5817ae2`
Suite runner revision: `237aaa373e101182c908c7bc2fcde5b8a5817ae2`
Suite started: 2026-10-11T10:26:50-04:00
Reference host: `akmac`, MacBook Air (Mac14,2), Apple M2/8 CPU cores/16 GB RAM/integrated GPU, macOS 27.0.1; Rust `rustc 1.99.0 (b940084d7 2026-09-28)`.
Capture: release app, 4096² world, 1920×1080, 60 Hz target, profile `m2-16gb-v3`; Cargo.lock SHA-256 `d064050f2b9044bb0eecfb2b7ffee1b18644dd676e192f7b7d0b0008d42db06b`.
Power mode, thermal state, background activity, and physical display refresh are not controlled.

Each fixture/policy cell requires three accepted 60-second captures. A run is accepted only when one-minute load is below 3 both immediately before and after the process, the app exits successfully with `SMOKE_RESULT ok` after its 60-second capture timer, reports the requested fixture/policy at 1920×1080 with nonzero frame samples, has no interval drops, and has complete slice telemetry. Frame counts are actual measured loop intervals, not an assumed 60 Hz count. Rejected attempts are retained below and never contribute to accepted percentiles.

## Accepted captures

| Fixture | Policy | Rep | Load before/after | Frames | Frame p99 ms | Max ms | >33.3 ms | Slice p99 ms | Slice max ms |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| quiet-world | bounded-focus | 1 | 2.32/2.93 | 3599 | 18.163 | 33.267 | 0 | 0.008 | 0.020 |
| quiet-world | bounded-fifo | 1 | 2.55/2.92 | 3600 | 18.129 | 19.277 | 0 | 0.005 | 0.041 |
| quiet-world | traditional | 1 | 2.92/2.90 | 1193 | 66.353 | 67.836 | 1193 | 47.948 | 91.364 |
| explosive-lattice | bounded-focus | 1 | 2.90/2.69 | 3600 | 18.114 | 20.440 | 0 | 0.009 | 0.034 |
| explosive-lattice | bounded-fifo | 1 | 2.69/2.72 | 3600 | 18.155 | 19.142 | 0 | 0.005 | 0.026 |
| explosive-lattice | traditional | 1 | 2.72/2.88 | 1191 | 66.552 | 67.864 | 1191 | 48.470 | 84.080 |
| sand-release | bounded-focus | 1 | 2.88/2.94 | 3600 | 19.589 | 20.473 | 0 | 7.693 | 8.214 |
| sand-release | bounded-fifo | 1 | 2.94/2.34 | 3600 | 20.916 | 23.871 | 0 | 6.231 | 7.685 |
| sand-release | traditional | 1 | 2.78/2.47 | 496 | 183.857 | 184.997 | 496 | 173.474 | 174.522 |
| reservoir-breach | bounded-focus | 1 | 2.47/2.40 | 3600 | 18.724 | 20.479 | 0 | 2.442 | 2.849 |
| reservoir-breach | bounded-fifo | 1 | 2.80/2.54 | 3600 | 18.862 | 21.388 | 0 | 5.914 | 6.559 |
| reservoir-breach | traditional | 1 | 2.54/2.60 | 1191 | 66.621 | 67.218 | 1191 | 48.026 | 99.043 |
| burning-forest | bounded-focus | 1 | 2.60/2.61 | 3599 | 19.583 | 22.708 | 0 | 7.074 | 22.080 |
| burning-forest | bounded-fifo | 1 | 2.61/2.28 | 3598 | 20.058 | 30.823 | 0 | 8.011 | 19.675 |
| burning-forest | traditional | 1 | 2.28/2.52 | 1098 | 239.423 | 289.905 | 1098 | 224.146 | 288.149 |
| dirty-world-sweep | bounded-focus | 1 | 2.52/2.08 | 3600 | 17.979 | 19.061 | 0 | 0.006 | 0.043 |
| dirty-world-sweep | bounded-fifo | 1 | 2.08/2.97 | 3600 | 18.140 | 18.963 | 0 | 0.005 | 0.008 |
| dirty-world-sweep | traditional | 1 | 2.97/2.90 | 1192 | 66.493 | 68.064 | 1192 | 47.500 | 82.095 |
| tiny-capacity | bounded-focus | 1 | 2.90/2.29 | 3600 | 18.977 | 20.517 | 0 | 1.831 | 2.110 |
| tiny-capacity | bounded-fifo | 1 | 2.29/2.19 | 3600 | 19.074 | 22.380 | 0 | 1.791 | 2.602 |
| tiny-capacity | traditional | 1 | 2.19/2.64 | 1171 | 67.037 | 68.493 | 1171 | 50.048 | 124.983 |
| mixed-overload | bounded-focus | 1 | 2.64/2.16 | 3600 | 18.615 | 20.537 | 0 | 6.203 | 7.366 |
| mixed-overload | bounded-fifo | 1 | 2.16/2.31 | 3600 | 19.231 | 22.468 | 0 | 6.287 | 8.103 |
| mixed-overload | traditional | 1 | 2.31/2.33 | 948 | 109.562 | 120.219 | 948 | 98.101 | 125.684 |
| quiet-world | traditional | 2 | 2.33/2.51 | 1192 | 66.632 | 67.788 | 1192 | 47.906 | 81.336 |
| quiet-world | bounded-fifo | 2 | 2.51/2.63 | 3600 | 17.888 | 19.214 | 0 | 0.006 | 0.009 |
| quiet-world | bounded-focus | 2 | 2.63/2.17 | 3600 | 17.975 | 20.599 | 0 | 0.007 | 0.061 |
| explosive-lattice | traditional | 2 | 2.17/2.44 | 1191 | 66.671 | 67.680 | 1191 | 48.150 | 83.341 |
| explosive-lattice | bounded-fifo | 2 | 2.44/2.32 | 3600 | 18.039 | 19.059 | 0 | 0.005 | 0.031 |
| explosive-lattice | bounded-focus | 2 | 2.92/2.39 | 3600 | 17.957 | 19.061 | 0 | 0.008 | 0.021 |
| sand-release | traditional | 2 | 2.39/2.61 | 497 | 183.956 | 184.740 | 497 | 174.092 | 175.018 |
| sand-release | bounded-fifo | 2 | 2.61/2.47 | 3600 | 20.827 | 24.245 | 0 | 6.153 | 8.106 |
| sand-release | bounded-focus | 2 | 2.94/2.53 | 3600 | 19.410 | 22.619 | 0 | 7.691 | 8.800 |
| reservoir-breach | traditional | 2 | 2.53/2.37 | 1193 | 66.627 | 68.339 | 1193 | 47.652 | 95.358 |
| reservoir-breach | bounded-fifo | 2 | 2.37/1.87 | 3600 | 18.852 | 21.837 | 0 | 5.878 | 7.014 |
| reservoir-breach | bounded-focus | 2 | 1.87/1.99 | 3600 | 18.372 | 20.759 | 0 | 2.358 | 2.585 |
| burning-forest | traditional | 2 | 1.99/2.87 | 1099 | 239.438 | 290.576 | 1099 | 213.914 | 288.879 |
| burning-forest | bounded-fifo | 2 | 2.87/2.30 | 3598 | 19.990 | 27.381 | 0 | 8.150 | 25.229 |
| burning-forest | bounded-focus | 2 | 2.30/2.28 | 3599 | 19.621 | 30.799 | 0 | 7.083 | 14.852 |
| dirty-world-sweep | traditional | 2 | 2.28/2.95 | 1191 | 66.370 | 68.399 | 1191 | 48.391 | 84.620 |
| dirty-world-sweep | bounded-fifo | 2 | 2.93/2.60 | 3599 | 17.978 | 32.870 | 0 | 0.005 | 0.013 |
| dirty-world-sweep | bounded-focus | 2 | 2.60/2.23 | 3600 | 18.202 | 20.926 | 0 | 0.006 | 0.011 |
| tiny-capacity | traditional | 2 | 2.23/2.19 | 1171 | 66.970 | 69.923 | 1171 | 50.240 | 130.681 |
| tiny-capacity | bounded-fifo | 2 | 2.19/2.91 | 3600 | 19.148 | 22.326 | 0 | 2.456 | 3.017 |
| tiny-capacity | bounded-focus | 2 | 2.84/2.40 | 3600 | 19.071 | 20.376 | 0 | 1.827 | 2.105 |
| mixed-overload | traditional | 2 | 2.40/2.31 | 944 | 109.722 | 120.583 | 944 | 98.258 | 117.586 |
| mixed-overload | bounded-fifo | 2 | 2.31/2.53 | 3600 | 19.028 | 21.117 | 0 | 6.302 | 7.370 |
| mixed-overload | bounded-focus | 2 | 2.53/2.22 | 3599 | 18.880 | 23.057 | 0 | 6.166 | 8.266 |
| quiet-world | bounded-focus | 3 | 2.22/2.50 | 3600 | 18.019 | 19.691 | 0 | 0.007 | 0.020 |
| quiet-world | bounded-fifo | 3 | 2.50/2.67 | 3600 | 18.238 | 19.414 | 0 | 0.005 | 0.009 |
| quiet-world | traditional | 3 | 2.67/2.35 | 1194 | 65.430 | 67.979 | 1194 | 48.085 | 87.708 |
| explosive-lattice | bounded-focus | 3 | 2.89/2.47 | 3599 | 18.239 | 32.488 | 0 | 0.007 | 0.024 |
| explosive-lattice | bounded-fifo | 3 | 2.97/2.48 | 3600 | 18.374 | 22.717 | 0 | 0.005 | 0.031 |
| explosive-lattice | traditional | 3 | 2.77/2.81 | 1188 | 66.714 | 67.892 | 1188 | 48.748 | 88.436 |
| sand-release | bounded-focus | 3 | 2.81/2.53 | 3600 | 19.409 | 20.695 | 0 | 7.601 | 8.584 |
| sand-release | bounded-fifo | 3 | 2.53/2.87 | 3600 | 20.839 | 24.964 | 0 | 6.193 | 8.487 |
| sand-release | traditional | 3 | 2.87/2.43 | 496 | 184.271 | 184.861 | 496 | 173.516 | 174.622 |
| reservoir-breach | bounded-focus | 3 | 2.43/1.98 | 3600 | 18.826 | 20.160 | 0 | 2.467 | 2.576 |
| reservoir-breach | bounded-fifo | 3 | 1.98/2.74 | 3600 | 18.535 | 22.570 | 0 | 5.741 | 6.785 |
| reservoir-breach | traditional | 3 | 2.74/2.59 | 1186 | 66.700 | 67.850 | 1186 | 48.675 | 94.090 |
| burning-forest | bounded-focus | 3 | 2.59/2.44 | 3599 | 19.549 | 24.837 | 0 | 7.014 | 19.645 |
| burning-forest | bounded-fifo | 3 | 2.44/2.71 | 3599 | 19.673 | 24.113 | 0 | 7.959 | 19.544 |
| burning-forest | traditional | 3 | 2.71/2.64 | 1100 | 217.528 | 290.273 | 1100 | 213.857 | 288.594 |
| dirty-world-sweep | bounded-focus | 3 | 2.64/2.27 | 3600 | 18.184 | 19.316 | 0 | 0.007 | 0.111 |
| dirty-world-sweep | bounded-fifo | 3 | 2.27/2.45 | 3600 | 18.123 | 19.238 | 0 | 0.005 | 0.017 |
| dirty-world-sweep | traditional | 3 | 2.45/2.34 | 1191 | 66.580 | 68.213 | 1191 | 47.851 | 87.602 |
| tiny-capacity | bounded-focus | 3 | 2.34/1.85 | 3600 | 18.952 | 21.137 | 0 | 1.843 | 2.149 |
| tiny-capacity | bounded-fifo | 3 | 1.85/2.07 | 3600 | 19.429 | 24.809 | 0 | 2.472 | 2.791 |
| tiny-capacity | traditional | 3 | 2.07/2.42 | 1172 | 66.972 | 68.269 | 1172 | 50.070 | 122.897 |
| mixed-overload | bounded-focus | 3 | 2.42/2.42 | 3600 | 18.664 | 20.754 | 0 | 6.185 | 6.967 |
| mixed-overload | bounded-fifo | 3 | 2.42/2.15 | 3600 | 19.149 | 24.292 | 0 | 6.422 | 7.238 |
| mixed-overload | traditional | 3 | 2.77/2.84 | 945 | 109.806 | 121.338 | 945 | 98.182 | 117.177 |

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
| reservoir-breach | bounded-focus | 3/3 | met | met | reported |
| reservoir-breach | bounded-fifo | 3/3 | met | unmet | reported |
| reservoir-breach | traditional | 3/3 | unmet | unmet | reported |
| burning-forest | bounded-focus | 3/3 | met | unmet | reported |
| burning-forest | bounded-fifo | 3/3 | unmet | unmet | reported |
| burning-forest | traditional | 3/3 | unmet | unmet | reported |
| dirty-world-sweep | bounded-focus | 3/3 | met | met | reported |
| dirty-world-sweep | bounded-fifo | 3/3 | met | met | reported |
| dirty-world-sweep | traditional | 3/3 | unmet | unmet | reported |
| tiny-capacity | bounded-focus | 3/3 | met | met | reported |
| tiny-capacity | bounded-fifo | 3/3 | met | met | reported |
| tiny-capacity | traditional | 3/3 | unmet | unmet | reported |
| mixed-overload | bounded-focus | 3/3 | met | unmet | reported |
| mixed-overload | bounded-fifo | 3/3 | met | unmet | reported |
| mixed-overload | traditional | 3/3 | unmet | unmet | reported |

## Interaction feedback samples

These CPU-side event/action-to-visible-upload p95 values use the app's documented method. Counts below 30 are descriptive only; quiet-world intentionally has no scripted interaction samples.

| Fixture | Policy | Rep | Camera n / p95 ms | Paint n / p95 ms | Ignite n / p95 ms | Detonate n / p95 ms |
|---|---|---:|---:|---:|---:|---:|
| quiet-world | bounded-focus | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| quiet-world | bounded-fifo | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
| quiet-world | traditional | 1 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 | 0 / 0.00 |
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
| mixed-overload | bounded-focus | 1 | 240 / 18.19 | 240 / 17.93 | 240 / 18.19 | 240 / 18.10 |
| mixed-overload | bounded-fifo | 1 | 240 / 17.76 | 240 / 18.01 | 240 / 17.94 | 240 / 18.08 |
| mixed-overload | traditional | 1 | 240 / 67.08 | 240 / 67.12 | 240 / 67.27 | 240 / 133.66 |
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
| mixed-overload | traditional | 2 | 240 / 67.20 | 240 / 67.15 | 240 / 67.13 | 240 / 134.01 |
| mixed-overload | bounded-fifo | 2 | 240 / 17.98 | 240 / 17.77 | 240 / 17.94 | 240 / 18.09 |
| mixed-overload | bounded-focus | 2 | 240 / 18.28 | 240 / 18.08 | 240 / 18.22 | 240 / 18.15 |
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
| mixed-overload | bounded-focus | 3 | 240 / 17.96 | 240 / 18.24 | 240 / 18.07 | 240 / 18.00 |
| mixed-overload | bounded-fifo | 3 | 240 / 17.86 | 240 / 18.05 | 240 / 17.69 | 240 / 18.02 |
| mixed-overload | traditional | 3 | 240 / 67.68 | 240 / 67.73 | 240 / 67.72 | 240 / 133.81 |

## Rejected attempts

| Fixture | Policy | Rep | Attempt | Load before/after | Reason | Log |
|---|---|---:|---:|---:|---|---|
| quiet-world | bounded-fifo | 1 | 1 | 2.93/9.59 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/quiet-world-bounded-fifo-rep1-attempt1.log` |
| quiet-world | bounded-fifo | 1 | 2 | 9.59/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-fifo | 1 | 3 | 9.06/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-fifo | 1 | 4 | 8.42/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-fifo | 1 | 5 | 8.06/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-fifo | 1 | 6 | 7.50/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-fifo | 1 | 7 | 6.98/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-fifo | 1 | 8 | 6.58/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-fifo | 1 | 9 | 6.03/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-fifo | 1 | 10 | 5.70/n/a | pre-run one-minute load not below 3 | `-` |
| quiet-world | bounded-fifo | 1 | 11 | 5.49/n/a | pre-run one-minute load not below 3 | `benchmarks/tmp/native-acceptance-suite-runner.log` |
| sand-release | traditional | 1 | 1 | 2.34/3.21 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/sand-release-traditional-rep1-attempt1.log` |
| sand-release | traditional | 1 | 2 | 3.21/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | traditional | 1 | 3 | 3.12/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | traditional | 1 | 4 | 3.11/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | traditional | 1 | 5 | 3.02/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 1 | 1 | 2.40/3.57 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/reservoir-breach-bounded-fifo-rep1-attempt1.log` |
| reservoir-breach | bounded-fifo | 1 | 2 | 3.57/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 1 | 3 | 4.09/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 1 | 4 | 4.08/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 1 | 5 | 3.83/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 1 | 6 | 3.61/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 1 | 7 | 3.40/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 1 | 8 | 3.21/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 1 | 9 | 3.03/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 1 | 10 | 2.87/3.05 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/reservoir-breach-bounded-fifo-rep1-attempt10.log` |
| reservoir-breach | bounded-fifo | 1 | 11 | 3.05/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | bounded-fifo | 1 | 12 | 3.04/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 2 | 1 | 2.32/3.05 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/explosive-lattice-bounded-focus-rep2-attempt1.log` |
| explosive-lattice | bounded-focus | 2 | 2 | 3.05/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 2 | 3 | 3.52/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 2 | 4 | 3.40/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 2 | 5 | 3.19/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 2 | 6 | 3.10/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 2 | 7 | 3.09/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 2 | 1 | 2.47/3.24 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/sand-release-bounded-focus-rep2-attempt1.log` |
| sand-release | bounded-focus | 2 | 2 | 3.24/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 2 | 3 | 3.38/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 2 | 4 | 3.27/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 2 | 5 | 3.41/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 2 | 6 | 3.45/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 2 | 7 | 3.26/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 2 | 8 | 3.64/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 2 | 9 | 3.50/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 2 | 10 | 3.38/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-focus | 2 | 11 | 3.19/n/a | pre-run one-minute load not below 3 | `-` |
| dirty-world-sweep | bounded-fifo | 2 | 1 | 2.95/3.70 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/dirty-world-sweep-bounded-fifo-rep2-attempt1.log` |
| dirty-world-sweep | bounded-fifo | 2 | 2 | 3.70/n/a | pre-run one-minute load not below 3 | `-` |
| dirty-world-sweep | bounded-fifo | 2 | 3 | 3.81/n/a | pre-run one-minute load not below 3 | `-` |
| dirty-world-sweep | bounded-fifo | 2 | 4 | 3.90/n/a | pre-run one-minute load not below 3 | `-` |
| dirty-world-sweep | bounded-fifo | 2 | 5 | 3.67/n/a | pre-run one-minute load not below 3 | `-` |
| dirty-world-sweep | bounded-fifo | 2 | 6 | 3.46/n/a | pre-run one-minute load not below 3 | `-` |
| dirty-world-sweep | bounded-fifo | 2 | 7 | 3.26/n/a | pre-run one-minute load not below 3 | `-` |
| dirty-world-sweep | bounded-fifo | 2 | 8 | 3.16/n/a | pre-run one-minute load not below 3 | `-` |
| dirty-world-sweep | bounded-fifo | 2 | 9 | 3.23/n/a | pre-run one-minute load not below 3 | `-` |
| dirty-world-sweep | bounded-fifo | 2 | 10 | 3.21/n/a | pre-run one-minute load not below 3 | `-` |
| dirty-world-sweep | bounded-fifo | 2 | 11 | 3.19/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 1 | 2.91/5.62 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/tiny-capacity-bounded-focus-rep2-attempt1.log` |
| tiny-capacity | bounded-focus | 2 | 2 | 5.33/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 3 | 5.07/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 4 | 4.90/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 5 | 4.91/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 6 | 4.60/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 7 | 4.39/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 8 | 4.12/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 9 | 4.03/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 10 | 3.86/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 11 | 3.55/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 12 | 3.43/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 13 | 3.48/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 14 | 3.44/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 15 | 3.24/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 16 | 3.46/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 17 | 3.43/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 18 | 3.39/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 19 | 3.36/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 20 | 3.33/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 21 | 3.22/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 22 | 3.13/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 23 | 3.12/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 24 | 3.11/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 25 | 3.18/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | bounded-focus | 2 | 26 | 3.00/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 1 | 2.35/5.77 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/explosive-lattice-bounded-focus-rep3-attempt1.log` |
| explosive-lattice | bounded-focus | 3 | 2 | 5.77/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 3 | 5.71/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 4 | 5.49/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 5 | 5.29/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 6 | 5.03/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 7 | 4.86/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 8 | 4.63/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 9 | 4.42/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 10 | 4.23/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 11 | 4.05/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 12 | 4.05/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 13 | 4.12/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 14 | 3.95/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 15 | 3.96/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 16 | 3.96/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 17 | 3.72/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 18 | 3.82/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 19 | 4.16/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 20 | 4.30/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 21 | 4.28/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 22 | 4.26/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 23 | 4.00/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 24 | 3.92/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 25 | 3.68/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 26 | 3.47/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 27 | 3.35/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 28 | 3.29/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 29 | 3.27/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 30 | 3.00/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 31 | 3.16/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 32 | 3.23/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-focus | 3 | 33 | 3.05/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 1 | 2.47/3.06 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/explosive-lattice-bounded-fifo-rep3-attempt1.log` |
| explosive-lattice | bounded-fifo | 3 | 2 | 3.06/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | traditional | 3 | 1 | 2.48/3.10 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/explosive-lattice-traditional-rep3-attempt1.log` |
| explosive-lattice | traditional | 3 | 2 | 3.10/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | traditional | 3 | 3 | 3.02/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 1 | 2.15/3.01 | one-minute load gate failed | `benchmarks/tmp/native-acceptance/mixed-overload-traditional-rep3-attempt1.log` |
| mixed-overload | traditional | 3 | 2 | 3.01/n/a | pre-run one-minute load not below 3 | `-` |

## Interpretation and limits

Frame intervals are native windowed event-loop intervals; they include presentation pacing and are not GPU duration. Slice CPU values are monotonic wall-time samples around the app's simulation dispatch, not thread CPU time. Fixture captures do not add external disturbances or player actions, except mixed-overload, which uses the app's scripted sustained-overload action stream to exercise focus latency. The load gate does not exclude all OS/driver interference.

Raw per-attempt logs are retained locally under `benchmarks/tmp/` and are not committed.
