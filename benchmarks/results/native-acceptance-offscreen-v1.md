# Native all-fixture offscreen acceptance suite v1

Status: offscreen matrix complete; 72/72 accepted captures; 206 rejected attempts; windowed presentation confirmation pending.

App source revision: `4a2f80e05a1a10f9878550f9dfa2270c0f9a120a`<br>
Suite runner revision: `4a2f80e05a1a10f9878550f9dfa2270c0f9a120a`<br>
Suite started: 2026-10-07T02:19:31-04:00
Reference host: `akmac`, MacBook Air (Mac14,2), Apple M2/8 CPU cores/16 GB RAM/integrated GPU, macOS 27.0.1; Rust `rustc 1.99.0 (b940084d7 2026-09-28)`.
Capture: release app, 4096² world, 1920×1080, 60 Hz offscreen cadence, profile `m2-16gb-v3`; Cargo.lock SHA-256 `d064050f2b9044bb0eecfb2b7ffee1b18644dd676e192f7b7d0b0008d42db06b`.
Power mode, thermal state, background activity, and physical display refresh are not controlled.

Each fixture/policy cell requires three accepted 60-second captures. A run is accepted only when one-minute load is below 3 both immediately before and after the process, the app exits successfully with `SMOKE_RESULT ok` after its 60-second capture timer, reports the requested fixture/policy at 1920×1080 with nonzero frame samples, has no interval drops, and has complete slice telemetry. Frame counts are actual measured loop intervals, not an assumed 60 Hz count. Rejected attempts are retained below and never contribute to accepted percentiles.

## Charter acceptance scope

These captures are offscreen GPU-texture runs, not windowed evidence. They report app-side simulation slice CPU and fixed-cadence loop intervals. They do not measure display presentation, vsync, compositor, scanout, or visible camera/UI response. They cannot satisfy the presented frame-interval, camera/UI feedback p95, player action-to-first-visible-effect p95, or window/surface-behavior criteria. Treat offscreen frame-interval and scripted action-to-effect proxy results as diagnostic only for those criteria; windowed confirmation remains pending with the operator present.

## Accepted captures

| Fixture | Policy | Rep | Load before/after | Frames | Frame p99 ms | Max ms | >33.3 ms | Slice p99 ms | Slice max ms |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| quiet-world | bounded-focus | 1 | 2.35/2.04 | 3600 | 19.533 | 21.427 | 0 | 5.045 | 10.649 |
| quiet-world | bounded-fifo | 1 | 2.04/2.06 | 3599 | 19.753 | 30.202 | 0 | 4.289 | 8.428 |
| quiet-world | traditional | 1 | 2.84/2.96 | 1074 | 114.764 | 133.242 | 771 | 97.928 | 114.438 |
| explosive-lattice | bounded-focus | 1 | 2.96/2.64 | 3599 | 19.566 | 28.261 | 0 | 5.100 | 6.613 |
| explosive-lattice | bounded-fifo | 1 | 2.64/2.05 | 3598 | 19.761 | 36.828 | 1 | 4.295 | 16.928 |
| explosive-lattice | traditional | 1 | 2.05/2.94 | 1088 | 115.541 | 118.061 | 785 | 98.930 | 114.040 |
| sand-release | bounded-focus | 1 | 2.94/2.66 | 3600 | 19.577 | 20.201 | 0 | 5.033 | 10.677 |
| sand-release | bounded-fifo | 1 | 2.85/2.57 | 3599 | 19.630 | 33.581 | 1 | 4.271 | 8.532 |
| sand-release | traditional | 1 | 2.57/2.58 | 1066 | 114.984 | 119.706 | 763 | 97.914 | 114.675 |
| reservoir-breach | bounded-focus | 1 | 2.58/2.44 | 3598 | 19.605 | 49.529 | 1 | 5.020 | 37.570 |
| reservoir-breach | bounded-fifo | 1 | 2.44/2.02 | 3600 | 19.733 | 26.093 | 0 | 4.319 | 10.406 |
| reservoir-breach | traditional | 1 | 2.02/2.83 | 1092 | 114.871 | 120.092 | 789 | 97.816 | 114.558 |
| burning-forest | bounded-focus | 1 | 2.83/2.20 | 3600 | 19.590 | 20.448 | 0 | 5.039 | 7.373 |
| burning-forest | bounded-fifo | 1 | 2.20/2.24 | 3600 | 19.741 | 20.840 | 0 | 4.275 | 11.875 |
| burning-forest | traditional | 1 | 2.85/2.94 | 1071 | 103.314 | 120.143 | 768 | 94.184 | 109.630 |
| dirty-world-sweep | bounded-focus | 1 | 2.94/2.09 | 3598 | 19.629 | 34.213 | 1 | 5.075 | 19.689 |
| dirty-world-sweep | bounded-fifo | 1 | 2.09/1.80 | 3599 | 19.727 | 26.725 | 0 | 4.163 | 4.922 |
| dirty-world-sweep | traditional | 1 | 1.80/2.54 | 1073 | 114.684 | 120.153 | 770 | 97.943 | 114.356 |
| tiny-capacity | bounded-focus | 1 | 2.54/2.21 | 3600 | 19.457 | 20.595 | 0 | 5.069 | 5.776 |
| tiny-capacity | bounded-fifo | 1 | 2.21/2.32 | 3599 | 19.663 | 33.582 | 1 | 4.210 | 13.058 |
| tiny-capacity | traditional | 1 | 2.32/2.85 | 1072 | 114.709 | 120.132 | 769 | 98.247 | 109.409 |
| mixed-overload | bounded-focus | 1 | 2.85/2.41 | 3600 | 19.524 | 21.888 | 0 | 5.151 | 6.091 |
| mixed-overload | bounded-fifo | 1 | 2.41/2.58 | 3598 | 19.778 | 32.293 | 0 | 4.249 | 20.684 |
| mixed-overload | traditional | 1 | 2.85/2.57 | 827 | 116.148 | 119.751 | 827 | 104.475 | 110.374 |
| quiet-world | traditional | 2 | 2.57/2.75 | 1082 | 114.510 | 119.699 | 779 | 97.996 | 112.000 |
| quiet-world | bounded-fifo | 2 | 2.75/2.06 | 3600 | 19.716 | 20.430 | 0 | 4.228 | 7.389 |
| quiet-world | bounded-focus | 2 | 2.13/2.10 | 3600 | 19.352 | 21.522 | 0 | 5.032 | 7.132 |
| explosive-lattice | traditional | 2 | 2.10/2.75 | 1052 | 114.752 | 119.731 | 749 | 97.985 | 109.489 |
| explosive-lattice | bounded-fifo | 2 | 2.75/2.63 | 3599 | 19.680 | 33.725 | 1 | 4.240 | 18.422 |
| explosive-lattice | bounded-focus | 2 | 2.63/2.22 | 3600 | 19.484 | 20.466 | 0 | 4.991 | 6.125 |
| sand-release | traditional | 2 | 2.22/2.82 | 1081 | 114.835 | 135.847 | 778 | 97.977 | 115.071 |
| sand-release | bounded-fifo | 2 | 2.82/2.68 | 3598 | 19.684 | 35.458 | 1 | 4.173 | 16.899 |
| sand-release | bounded-focus | 2 | 2.68/2.29 | 3598 | 19.653 | 33.526 | 1 | 5.058 | 19.061 |
| reservoir-breach | traditional | 2 | 2.94/2.79 | 1077 | 114.862 | 119.721 | 774 | 98.033 | 109.396 |
| reservoir-breach | bounded-fifo | 2 | 2.79/2.42 | 3600 | 19.693 | 27.155 | 0 | 4.219 | 5.740 |
| reservoir-breach | bounded-focus | 2 | 2.42/2.28 | 3598 | 19.437 | 31.651 | 0 | 5.022 | 13.991 |
| burning-forest | traditional | 2 | 2.28/2.78 | 1055 | 115.397 | 134.040 | 752 | 97.922 | 114.531 |
| burning-forest | bounded-fifo | 2 | 2.78/2.32 | 3598 | 19.766 | 29.641 | 0 | 4.139 | 6.526 |
| burning-forest | bounded-focus | 2 | 2.32/2.22 | 3598 | 19.478 | 33.051 | 0 | 5.046 | 19.968 |
| dirty-world-sweep | traditional | 2 | 2.22/2.75 | 1068 | 114.809 | 133.534 | 765 | 97.979 | 115.303 |
| dirty-world-sweep | bounded-fifo | 2 | 2.69/2.42 | 3599 | 19.625 | 28.268 | 0 | 4.212 | 6.044 |
| dirty-world-sweep | bounded-focus | 2 | 2.42/2.38 | 3600 | 19.671 | 27.995 | 0 | 5.038 | 7.858 |
| tiny-capacity | traditional | 2 | 2.94/3.00 | 1065 | 114.951 | 119.724 | 762 | 98.003 | 109.385 |
| tiny-capacity | bounded-fifo | 2 | 3.00/2.81 | 3599 | 19.663 | 30.829 | 0 | 4.234 | 28.209 |
| tiny-capacity | bounded-focus | 2 | 2.81/2.90 | 3599 | 19.524 | 31.905 | 0 | 5.159 | 8.305 |
| mixed-overload | traditional | 2 | 2.97/2.94 | 847 | 116.374 | 119.755 | 847 | 103.790 | 109.436 |
| mixed-overload | bounded-fifo | 2 | 2.94/2.54 | 3600 | 19.676 | 20.356 | 0 | 4.203 | 7.906 |
| mixed-overload | bounded-focus | 2 | 2.54/2.69 | 3600 | 19.522 | 20.373 | 0 | 5.103 | 6.666 |
| quiet-world | bounded-focus | 3 | 2.69/2.31 | 3599 | 19.561 | 20.191 | 0 | 5.044 | 8.284 |
| quiet-world | bounded-fifo | 3 | 2.31/2.48 | 3600 | 19.631 | 20.422 | 0 | 4.158 | 7.134 |
| quiet-world | traditional | 3 | 2.48/2.73 | 1072 | 113.518 | 134.708 | 769 | 97.933 | 114.094 |
| explosive-lattice | bounded-focus | 3 | 2.73/2.87 | 3600 | 19.619 | 20.303 | 0 | 5.020 | 10.377 |
| explosive-lattice | bounded-fifo | 3 | 2.98/2.78 | 3598 | 19.710 | 35.023 | 1 | 4.333 | 24.796 |
| explosive-lattice | traditional | 3 | 2.78/2.26 | 1087 | 114.663 | 133.529 | 784 | 98.298 | 116.190 |
| sand-release | bounded-focus | 3 | 2.26/2.51 | 3600 | 19.589 | 22.034 | 0 | 5.002 | 6.066 |
| sand-release | bounded-fifo | 3 | 2.51/2.67 | 3600 | 19.505 | 20.231 | 0 | 4.352 | 9.580 |
| sand-release | traditional | 3 | 2.67/2.77 | 1059 | 114.937 | 134.671 | 756 | 98.112 | 115.282 |
| reservoir-breach | bounded-focus | 3 | 2.77/2.61 | 3599 | 19.499 | 26.343 | 0 | 5.099 | 12.319 |
| reservoir-breach | bounded-fifo | 3 | 2.61/2.60 | 3599 | 19.646 | 29.871 | 0 | 4.134 | 6.749 |
| reservoir-breach | traditional | 3 | 2.90/2.57 | 1079 | 114.484 | 119.708 | 776 | 98.034 | 109.299 |
| burning-forest | bounded-focus | 3 | 2.57/2.24 | 3599 | 19.573 | 28.163 | 0 | 5.132 | 8.023 |
| burning-forest | bounded-fifo | 3 | 2.24/1.74 | 3598 | 19.729 | 34.617 | 2 | 4.199 | 24.861 |
| burning-forest | traditional | 3 | 1.74/2.29 | 1070 | 114.841 | 133.993 | 767 | 97.895 | 114.542 |
| dirty-world-sweep | bounded-focus | 3 | 2.29/2.07 | 3600 | 19.517 | 20.264 | 0 | 5.055 | 7.457 |
| dirty-world-sweep | bounded-fifo | 3 | 2.07/2.08 | 3600 | 19.623 | 22.913 | 0 | 4.323 | 5.507 |
| dirty-world-sweep | traditional | 3 | 2.08/2.67 | 1062 | 113.093 | 135.207 | 759 | 97.849 | 114.559 |
| tiny-capacity | bounded-focus | 3 | 2.78/2.72 | 3600 | 19.488 | 20.316 | 0 | 5.065 | 6.004 |
| tiny-capacity | bounded-fifo | 3 | 2.72/2.30 | 3600 | 19.532 | 25.017 | 0 | 4.160 | 7.273 |
| tiny-capacity | traditional | 3 | 2.30/2.24 | 1081 | 114.795 | 120.112 | 778 | 97.976 | 109.444 |
| mixed-overload | bounded-focus | 3 | 2.24/2.55 | 3599 | 19.518 | 20.327 | 0 | 5.082 | 6.294 |
| mixed-overload | bounded-fifo | 3 | 2.55/2.99 | 3599 | 19.663 | 25.003 | 0 | 4.138 | 7.690 |
| mixed-overload | traditional | 3 | 2.88/2.55 | 827 | 116.435 | 120.116 | 827 | 103.833 | 109.561 |

## Offscreen section 15 diagnostic rows

| Fixture | Policy | Accepted runs | Offscreen loop p99 ≤20 ms (diagnostic) | Slice p99 ≤4 ms | >33.3 ms count / max disclosed |
|---|---|---:|---|---|---|
| quiet-world | bounded-focus | 3/3 | met | unmet | reported |
| quiet-world | bounded-fifo | 3/3 | met | unmet | reported |
| quiet-world | traditional | 3/3 | unmet | unmet | reported |
| explosive-lattice | bounded-focus | 3/3 | met | unmet | reported |
| explosive-lattice | bounded-fifo | 3/3 | met | unmet | reported |
| explosive-lattice | traditional | 3/3 | unmet | unmet | reported |
| sand-release | bounded-focus | 3/3 | met | unmet | reported |
| sand-release | bounded-fifo | 3/3 | met | unmet | reported |
| sand-release | traditional | 3/3 | unmet | unmet | reported |
| reservoir-breach | bounded-focus | 3/3 | met | unmet | reported |
| reservoir-breach | bounded-fifo | 3/3 | met | unmet | reported |
| reservoir-breach | traditional | 3/3 | unmet | unmet | reported |
| burning-forest | bounded-focus | 3/3 | met | unmet | reported |
| burning-forest | bounded-fifo | 3/3 | met | unmet | reported |
| burning-forest | traditional | 3/3 | unmet | unmet | reported |
| dirty-world-sweep | bounded-focus | 3/3 | met | unmet | reported |
| dirty-world-sweep | bounded-fifo | 3/3 | met | unmet | reported |
| dirty-world-sweep | traditional | 3/3 | unmet | unmet | reported |
| tiny-capacity | bounded-focus | 3/3 | met | unmet | reported |
| tiny-capacity | bounded-fifo | 3/3 | met | unmet | reported |
| tiny-capacity | traditional | 3/3 | unmet | unmet | reported |
| mixed-overload | bounded-focus | 3/3 | met | unmet | reported |
| mixed-overload | bounded-fifo | 3/3 | met | unmet | reported |
| mixed-overload | traditional | 3/3 | unmet | unmet | reported |

## Interaction feedback samples

Offscreen scripted camera/action-to-CPU-effect/upload-queue-write proxies only; these are not visible response measurements. Counts below 30 are descriptive only; quiet-world intentionally has no scripted interaction samples.

| Fixture | Policy | Rep | Camera n / p95 ms | Paint n / p95 ms | Ignite n / p95 ms | Detonate n / p95 ms |
|---|---|---:|---:|---:|---:|---:|
| quiet-world | bounded-focus | 1 | 240 / 18.78 | 240 / 18.70 | 240 / 18.70 | 240 / 18.89 |
| quiet-world | bounded-fifo | 1 | 240 / 18.99 | 240 / 19.53 | 240 / 19.29 | 240 / 19.48 |
| quiet-world | traditional | 1 | 240 / 85.29 | 240 / 83.56 | 240 / 83.82 | 240 / 166.93 |
| explosive-lattice | bounded-focus | 1 | 240 / 18.70 | 240 / 18.97 | 240 / 18.83 | 240 / 18.59 |
| explosive-lattice | bounded-fifo | 1 | 240 / 19.15 | 240 / 19.70 | 240 / 19.38 | 240 / 19.41 |
| explosive-lattice | traditional | 1 | 240 / 83.82 | 240 / 83.51 | 240 / 83.54 | 240 / 166.86 |
| sand-release | bounded-focus | 1 | 240 / 18.81 | 240 / 18.83 | 240 / 18.88 | 240 / 19.17 |
| sand-release | bounded-fifo | 1 | 240 / 19.14 | 240 / 19.28 | 240 / 19.39 | 240 / 19.26 |
| sand-release | traditional | 1 | 240 / 83.79 | 240 / 83.60 | 240 / 83.57 | 240 / 166.87 |
| reservoir-breach | bounded-focus | 1 | 240 / 19.01 | 240 / 18.76 | 240 / 18.70 | 240 / 19.01 |
| reservoir-breach | bounded-fifo | 1 | 240 / 19.15 | 240 / 19.48 | 240 / 19.04 | 240 / 19.30 |
| reservoir-breach | traditional | 1 | 240 / 83.96 | 240 / 83.56 | 240 / 83.53 | 240 / 166.92 |
| burning-forest | bounded-focus | 1 | 240 / 18.83 | 240 / 18.92 | 240 / 18.52 | 240 / 19.12 |
| burning-forest | bounded-fifo | 1 | 240 / 18.97 | 240 / 19.36 | 240 / 19.10 | 240 / 18.88 |
| burning-forest | traditional | 1 | 240 / 83.82 | 240 / 83.70 | 240 / 83.53 | 240 / 166.98 |
| dirty-world-sweep | bounded-focus | 1 | 240 / 18.47 | 240 / 18.71 | 240 / 18.87 | 240 / 18.46 |
| dirty-world-sweep | bounded-fifo | 1 | 240 / 18.55 | 240 / 19.13 | 240 / 18.63 | 240 / 19.19 |
| dirty-world-sweep | traditional | 1 | 240 / 83.80 | 240 / 83.46 | 240 / 83.50 | 240 / 166.90 |
| tiny-capacity | bounded-focus | 1 | 240 / 18.73 | 240 / 18.76 | 240 / 18.82 | 240 / 18.66 |
| tiny-capacity | bounded-fifo | 1 | 240 / 19.17 | 240 / 18.85 | 240 / 19.45 | 240 / 19.34 |
| tiny-capacity | traditional | 1 | 240 / 83.88 | 240 / 83.67 | 240 / 83.51 | 240 / 166.99 |
| mixed-overload | bounded-focus | 1 | 240 / 18.86 | 240 / 18.81 | 240 / 18.97 | 240 / 18.86 |
| mixed-overload | bounded-fifo | 1 | 240 / 18.01 | 240 / 18.97 | 240 / 19.49 | 240 / 19.34 |
| mixed-overload | traditional | 1 | 240 / 83.88 | 240 / 83.72 | 240 / 83.70 | 240 / 166.86 |
| quiet-world | traditional | 2 | 240 / 83.82 | 240 / 83.47 | 240 / 83.53 | 240 / 166.86 |
| quiet-world | bounded-fifo | 2 | 240 / 18.59 | 240 / 19.54 | 240 / 18.78 | 240 / 19.27 |
| quiet-world | bounded-focus | 2 | 240 / 18.88 | 240 / 18.94 | 240 / 18.82 | 240 / 18.73 |
| explosive-lattice | traditional | 2 | 240 / 83.94 | 240 / 83.64 | 240 / 83.81 | 240 / 166.84 |
| explosive-lattice | bounded-fifo | 2 | 240 / 18.68 | 240 / 19.16 | 240 / 19.09 | 240 / 18.46 |
| explosive-lattice | bounded-focus | 2 | 240 / 18.84 | 240 / 18.72 | 240 / 18.78 | 240 / 19.06 |
| sand-release | traditional | 2 | 240 / 84.99 | 240 / 83.61 | 240 / 83.64 | 240 / 166.96 |
| sand-release | bounded-fifo | 2 | 240 / 18.65 | 240 / 19.05 | 240 / 19.16 | 240 / 19.46 |
| sand-release | bounded-focus | 2 | 240 / 18.86 | 240 / 18.88 | 240 / 18.87 | 240 / 18.80 |
| reservoir-breach | traditional | 2 | 240 / 83.75 | 240 / 83.50 | 240 / 83.58 | 240 / 166.81 |
| reservoir-breach | bounded-fifo | 2 | 240 / 19.05 | 240 / 19.12 | 240 / 19.18 | 240 / 18.92 |
| reservoir-breach | bounded-focus | 2 | 240 / 18.71 | 240 / 18.63 | 240 / 18.67 | 240 / 18.75 |
| burning-forest | traditional | 2 | 240 / 84.21 | 240 / 83.72 | 240 / 83.71 | 240 / 166.96 |
| burning-forest | bounded-fifo | 2 | 240 / 19.13 | 240 / 19.46 | 240 / 19.41 | 240 / 19.12 |
| burning-forest | bounded-focus | 2 | 240 / 18.74 | 240 / 18.85 | 240 / 19.02 | 240 / 18.54 |
| dirty-world-sweep | traditional | 2 | 240 / 83.87 | 240 / 83.43 | 240 / 83.54 | 240 / 166.77 |
| dirty-world-sweep | bounded-fifo | 2 | 240 / 19.19 | 240 / 19.28 | 240 / 19.36 | 240 / 18.88 |
| dirty-world-sweep | bounded-focus | 2 | 240 / 18.82 | 240 / 18.68 | 240 / 18.73 | 240 / 18.57 |
| tiny-capacity | traditional | 2 | 240 / 84.12 | 240 / 83.58 | 240 / 83.67 | 240 / 166.90 |
| tiny-capacity | bounded-fifo | 2 | 240 / 18.00 | 240 / 18.68 | 240 / 18.86 | 240 / 19.22 |
| tiny-capacity | bounded-focus | 2 | 240 / 18.85 | 240 / 18.97 | 240 / 19.23 | 240 / 19.13 |
| mixed-overload | traditional | 2 | 240 / 83.79 | 240 / 83.59 | 240 / 83.72 | 240 / 166.81 |
| mixed-overload | bounded-fifo | 2 | 240 / 18.14 | 240 / 19.33 | 240 / 18.90 | 240 / 18.58 |
| mixed-overload | bounded-focus | 2 | 240 / 18.61 | 240 / 18.81 | 240 / 18.81 | 240 / 18.69 |
| quiet-world | bounded-focus | 3 | 240 / 18.75 | 240 / 18.70 | 240 / 18.87 | 240 / 18.88 |
| quiet-world | bounded-fifo | 3 | 240 / 18.43 | 240 / 18.89 | 240 / 19.59 | 240 / 18.71 |
| quiet-world | traditional | 3 | 240 / 83.88 | 240 / 83.70 | 240 / 83.50 | 240 / 167.04 |
| explosive-lattice | bounded-focus | 3 | 240 / 18.80 | 240 / 18.75 | 240 / 19.24 | 240 / 19.32 |
| explosive-lattice | bounded-fifo | 3 | 240 / 18.68 | 240 / 18.82 | 240 / 18.97 | 240 / 18.95 |
| explosive-lattice | traditional | 3 | 240 / 83.85 | 240 / 83.58 | 240 / 83.58 | 240 / 166.84 |
| sand-release | bounded-focus | 3 | 240 / 18.93 | 240 / 18.82 | 240 / 18.66 | 240 / 18.97 |
| sand-release | bounded-fifo | 3 | 240 / 18.68 | 240 / 18.91 | 240 / 18.86 | 240 / 18.64 |
| sand-release | traditional | 3 | 240 / 83.88 | 240 / 83.67 | 240 / 83.71 | 240 / 166.81 |
| reservoir-breach | bounded-focus | 3 | 240 / 18.78 | 240 / 18.78 | 240 / 18.76 | 240 / 18.61 |
| reservoir-breach | bounded-fifo | 3 | 240 / 18.82 | 240 / 18.59 | 240 / 19.26 | 240 / 19.28 |
| reservoir-breach | traditional | 3 | 240 / 84.02 | 240 / 83.62 | 240 / 83.59 | 240 / 167.02 |
| burning-forest | bounded-focus | 3 | 240 / 18.95 | 240 / 19.00 | 240 / 18.96 | 240 / 18.76 |
| burning-forest | bounded-fifo | 3 | 240 / 18.66 | 240 / 19.27 | 240 / 19.33 | 240 / 19.23 |
| burning-forest | traditional | 3 | 240 / 84.13 | 240 / 83.47 | 240 / 83.58 | 240 / 166.83 |
| dirty-world-sweep | bounded-focus | 3 | 240 / 18.75 | 240 / 18.80 | 240 / 19.04 | 240 / 18.87 |
| dirty-world-sweep | bounded-fifo | 3 | 240 / 18.27 | 240 / 19.06 | 240 / 19.20 | 240 / 18.72 |
| dirty-world-sweep | traditional | 3 | 240 / 84.25 | 240 / 83.54 | 240 / 83.70 | 240 / 166.90 |
| tiny-capacity | bounded-focus | 3 | 240 / 18.96 | 240 / 18.68 | 240 / 18.91 | 240 / 18.85 |
| tiny-capacity | bounded-fifo | 3 | 240 / 18.89 | 240 / 18.85 | 240 / 18.97 | 240 / 18.92 |
| tiny-capacity | traditional | 3 | 240 / 83.91 | 240 / 83.89 | 240 / 83.58 | 240 / 167.15 |
| mixed-overload | bounded-focus | 3 | 240 / 18.86 | 240 / 18.89 | 240 / 18.70 | 240 / 18.86 |
| mixed-overload | bounded-fifo | 3 | 240 / 18.54 | 240 / 19.10 | 240 / 18.82 | 240 / 19.11 |
| mixed-overload | traditional | 3 | 240 / 84.22 | 240 / 83.51 | 240 / 83.79 | 240 / 166.86 |

## Rejected attempts

| Fixture | Policy | Rep | Attempt | Load before/after | Reason | Log |
|---|---|---:|---:|---:|---|---|
| quiet-world | traditional | 1 | 1 | 2.05/3.00 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/quiet-world-traditional-rep1-attempt1.log` |
| quiet-world | traditional | 1 | 2 | 3.00/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 1 | 2.66/3.49 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/sand-release-bounded-fifo-rep1-attempt1.log` |
| sand-release | bounded-fifo | 1 | 2 | 3.49/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 3 | 3.45/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 4 | 3.50/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 5 | 3.38/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 6 | 3.19/n/a | pre-run one-minute load not below 3 | `-` |
| sand-release | bounded-fifo | 1 | 7 | 3.01/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 1 | 2.24/3.56 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/burning-forest-traditional-rep1-attempt1.log` |
| burning-forest | traditional | 1 | 2 | 3.56/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 3 | 3.51/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 4 | 3.55/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 5 | 3.43/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 6 | 3.23/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 7 | 3.05/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 8 | 3.13/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 9 | 3.28/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 10 | 3.10/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 11 | 3.17/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 12 | 3.06/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 13 | 2.97/3.93 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/burning-forest-traditional-rep1-attempt13.log` |
| burning-forest | traditional | 1 | 14 | 3.93/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 15 | 3.69/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 16 | 3.80/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 17 | 4.05/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 18 | 3.89/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 19 | 3.74/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 20 | 3.68/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 21 | 3.46/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 22 | 3.35/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 23 | 3.16/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 24 | 3.07/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 25 | 2.90/3.81 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/burning-forest-traditional-rep1-attempt25.log` |
| burning-forest | traditional | 1 | 26 | 3.81/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 27 | 3.75/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 28 | 3.53/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 29 | 3.41/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 30 | 3.21/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 31 | 3.20/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 32 | 3.10/n/a | pre-run one-minute load not below 3 | `-` |
| burning-forest | traditional | 1 | 33 | 3.01/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 1 | 1 | 2.58/3.02 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/mixed-overload-traditional-rep1-attempt1.log` |
| mixed-overload | traditional | 1 | 2 | 3.02/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 2 | 1 | 2.29/3.06 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/reservoir-breach-traditional-rep2-attempt1.log` |
| reservoir-breach | traditional | 2 | 2 | 3.06/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 2 | 3 | 3.06/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 2 | 4 | 3.77/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 2 | 5 | 3.55/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 2 | 6 | 3.59/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 2 | 7 | 3.38/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 2 | 8 | 3.27/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 2 | 9 | 3.09/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 2 | 10 | 3.00/3.26 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/reservoir-breach-traditional-rep2-attempt10.log` |
| reservoir-breach | traditional | 2 | 11 | 3.26/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 2 | 12 | 3.40/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 2 | 13 | 3.21/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 2 | 14 | 3.03/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 2 | 15 | 2.87/3.02 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/reservoir-breach-traditional-rep2-attempt15.log` |
| reservoir-breach | traditional | 2 | 16 | 3.02/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 2 | 17 | 3.02/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 1 | 2.38/3.22 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/tiny-capacity-traditional-rep2-attempt1.log` |
| tiny-capacity | traditional | 2 | 2 | 3.22/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 3 | 3.04/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 4 | 3.20/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 5 | 3.02/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 6 | 2.86/3.27 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/tiny-capacity-traditional-rep2-attempt6.log` |
| tiny-capacity | traditional | 2 | 7 | 3.27/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 8 | 4.21/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 9 | 4.12/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 10 | 4.03/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 11 | 3.94/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 12 | 3.79/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 13 | 3.72/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 14 | 3.51/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 15 | 3.31/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 16 | 3.12/n/a | pre-run one-minute load not below 3 | `-` |
| tiny-capacity | traditional | 2 | 17 | 2.95/3.10 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/tiny-capacity-traditional-rep2-attempt17.log` |
| tiny-capacity | traditional | 2 | 18 | 3.10/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 1 | 2.90/3.34 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/mixed-overload-traditional-rep2-attempt1.log` |
| mixed-overload | traditional | 2 | 2 | 3.34/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 3 | 3.39/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 4 | 3.20/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 5 | 3.02/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 6 | 2.86/3.06 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/mixed-overload-traditional-rep2-attempt6.log` |
| mixed-overload | traditional | 2 | 7 | 3.06/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 8 | 3.14/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 9 | 3.04/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 10 | 2.88/3.23 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/mixed-overload-traditional-rep2-attempt10.log` |
| mixed-overload | traditional | 2 | 11 | 3.23/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 12 | 3.29/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 13 | 3.11/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 14 | 3.18/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 15 | 3.09/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 16 | 3.00/3.20 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/mixed-overload-traditional-rep2-attempt16.log` |
| mixed-overload | traditional | 2 | 17 | 3.20/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 18 | 3.02/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 19 | 2.94/3.10 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/mixed-overload-traditional-rep2-attempt19.log` |
| mixed-overload | traditional | 2 | 20 | 3.10/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 21 | 2.94/3.02 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/mixed-overload-traditional-rep2-attempt21.log` |
| mixed-overload | traditional | 2 | 22 | 3.02/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 23 | 3.10/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 24 | 3.01/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 25 | 2.93/3.28 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/mixed-overload-traditional-rep2-attempt25.log` |
| mixed-overload | traditional | 2 | 26 | 3.28/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 27 | 3.09/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 28 | 3.09/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 29 | 3.24/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 30 | 3.06/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 31 | 2.97/3.40 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/mixed-overload-traditional-rep2-attempt31.log` |
| mixed-overload | traditional | 2 | 32 | 3.40/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 33 | 3.44/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 34 | 3.25/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 35 | 3.23/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 36 | 3.21/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 37 | 3.27/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 38 | 3.41/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 39 | 3.46/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 40 | 3.50/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 41 | 3.46/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 42 | 3.34/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 43 | 3.24/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 44 | 3.22/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 45 | 3.04/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 46 | 3.28/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 47 | 3.25/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 48 | 3.07/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 49 | 4.59/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 50 | 4.62/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 51 | 4.33/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 52 | 4.22/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 53 | 4.13/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 54 | 4.12/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 55 | 4.11/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 56 | 3.86/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 57 | 3.63/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 58 | 3.42/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 59 | 3.94/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 60 | 3.95/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 61 | 3.95/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 62 | 4.04/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 63 | 3.87/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 64 | 3.72/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 65 | 3.58/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 66 | 3.46/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 67 | 3.37/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 68 | 3.50/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 69 | 3.30/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 70 | 3.12/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 71 | 3.03/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 72 | 2.95/3.57 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/mixed-overload-traditional-rep2-attempt72.log` |
| mixed-overload | traditional | 2 | 73 | 3.57/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 74 | 3.53/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 75 | 3.32/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 76 | 3.14/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 77 | 3.13/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 78 | 3.04/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 79 | 2.87/3.39 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/mixed-overload-traditional-rep2-attempt79.log` |
| mixed-overload | traditional | 2 | 80 | 3.39/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 81 | 3.20/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 82 | 3.10/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 83 | 3.09/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 84 | 3.01/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 85 | 2.85/3.14 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/mixed-overload-traditional-rep2-attempt85.log` |
| mixed-overload | traditional | 2 | 86 | 3.14/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 2 | 87 | 2.96/3.14 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/mixed-overload-traditional-rep2-attempt87.log` |
| mixed-overload | traditional | 2 | 88 | 3.14/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 1 | 2.87/8.26 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/explosive-lattice-bounded-fifo-rep3-attempt1.log` |
| explosive-lattice | bounded-fifo | 3 | 2 | 8.26/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 3 | 7.76/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 4 | 7.46/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 5 | 6.69/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 6 | 6.23/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 7 | 5.81/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 8 | 5.67/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 9 | 5.29/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 10 | 5.03/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 11 | 4.71/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 12 | 4.49/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 13 | 4.37/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 14 | 4.10/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 15 | 3.85/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 16 | 3.62/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 17 | 3.41/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 18 | 3.30/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 19 | 3.11/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 20 | 3.10/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 21 | 3.26/n/a | pre-run one-minute load not below 3 | `-` |
| explosive-lattice | bounded-fifo | 3 | 22 | 3.16/n/a | pre-run one-minute load not below 3 | `-` |
| reservoir-breach | traditional | 3 | 1 | 2.60/3.07 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/reservoir-breach-traditional-rep3-attempt1.log` |
| reservoir-breach | traditional | 3 | 2 | 3.07/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 1 | 2.99/3.33 | one-minute load gate failed | `benchmarks/tmp/native-acceptance-offscreen/mixed-overload-traditional-rep3-attempt1.log` |
| mixed-overload | traditional | 3 | 2 | 3.33/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 3 | 3.14/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 4 | 4.01/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 5 | 4.09/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 6 | 3.92/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 7 | 3.85/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 8 | 3.86/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 9 | 3.63/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 10 | 3.42/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 11 | 3.47/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 12 | 3.35/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 13 | 3.24/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 14 | 3.14/n/a | pre-run one-minute load not below 3 | `-` |
| mixed-overload | traditional | 3 | 15 | 3.05/n/a | pre-run one-minute load not below 3 | `-` |

## Interpretation and limits

Frame intervals are offscreen fixed-cadence loop intervals around GPU command submission; they exclude presentation, vsync, compositor, and scanout, and are not GPU duration. Slice CPU values are monotonic wall-time samples around the app's simulation dispatch, not thread CPU time. Mixed-overload uses the scripted action stream, but its measured effect/upload points do not establish visible response. The load gate does not exclude all OS/driver interference.

Raw per-attempt logs are retained locally under `benchmarks/tmp/` and are not committed.
