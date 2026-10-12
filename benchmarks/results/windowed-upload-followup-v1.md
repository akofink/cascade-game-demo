# Windowed bounded upload-budget follow-up v1

Status: experiment complete; not eligible to merge. Sand-release and burning-forest bounded-FIFO frame p99 improved over v2 and all six target repetitions are below 20 ms, but reservoir-breach/bounded-FIFO's three-repetition frame-p99 range is entirely above its v2 range. Keep the branch; do not open or merge a PR. This is a focused 34-capture follow-up, not a rerun of the 72-capture matrix.

App source revision: `1b4baa200b96de9dcb605a79613ab2e003f973ab` (includes the row-wise upload copy from `8ba701c` and the 96-copy frame budget). Suite runner revision: same. Host: `akmac`, MacBook Air (Mac14,2), Apple M2/8 CPU cores/16 GB RAM/integrated GPU, macOS 27.0.1; Rust `1.99.0`. Cargo.lock SHA-256: `d064050f2b9044bb0eecfb2b7ffee1b18644dd676e192f7b7d0b0008d42db06b`.

Capture settings: release app, 4096² world, verified 1920×1080 window, 60 Hz target, `m2-16gb-v3`, 60-second captures with `--frame-breakdown`, unchanged one-minute load gate <3 both before and after, and 180-second cooling after each launched attempt. Suite thresholds, fixtures, and credit profile were unchanged. Power mode, thermal state, background activity, and physical display refresh were uncontrolled.

The required set was 24 bounded-FIFO captures (three per fixture), eight bounded-focus screens (one per fixture), plus two additional quiet-world/bounded-focus captures requested to resolve the prior flagged screen. All 34 required-scope captures passed the unchanged acceptance checks, had zero interval drops, and included validated frame-breakdown telemetry. There were 25 load-gate rejections: 24 pre-run and one post-run; each affected capture was retried, and no failed-gate attempt contributes to the accepted data below. The full 72-capture fixture/policy matrix was not rerun. v2 and earlier reports remain historical records.

## Accepted captures

| Fixture | Policy | Rep | Load before/after | Frames | Frame p99 ms | Max ms | >33.3 ms | Slice p99 ms | Slice max ms | Max upload CPU ms | Max queued dirty chunks |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| quiet-world | bounded-fifo | 1 | 2.53/2.08 | 3601 | 17.957 | 19.089 | 0 | 0.004 | 0.013 | 2.753 | 7168 |
| explosive-lattice | bounded-fifo | 1 | 1.92/1.87 | 3600 | 18.133 | 19.662 | 0 | 0.006 | 0.010 | 2.756 | 7168 |
| sand-release | bounded-fifo | 1 | 1.95/2.34 | 3600 | 19.439 | 23.069 | 0 | 6.122 | 6.895 | 2.413 | 7168 |
| reservoir-breach | bounded-fifo | 1 | 1.93/2.38 | 3600 | 18.889 | 21.791 | 0 | 5.869 | 7.045 | 2.375 | 7168 |
| burning-forest | bounded-fifo | 1 | 1.75/1.90 | 3599 | 19.579 | 35.719 | 1 | 7.968 | 22.409 | 1.892 | 14168 |
| dirty-world-sweep | bounded-fifo | 1 | 2.85/2.39 | 3601 | 18.071 | 19.354 | 0 | 0.004 | 0.010 | 2.802 | 7168 |
| tiny-capacity | bounded-fifo | 1 | 2.51/2.94 | 3601 | 18.975 | 21.216 | 0 | 2.309 | 2.605 | 2.735 | 7553 |
| mixed-overload | bounded-fifo | 1 | 2.83/2.34 | 3600 | 19.016 | 22.862 | 0 | 6.350 | 7.111 | 2.366 | 7168 |
| quiet-world | bounded-fifo | 2 | 2.48/2.17 | 3601 | 18.159 | 19.895 | 0 | 0.006 | 0.017 | 2.640 | 7168 |
| explosive-lattice | bounded-fifo | 2 | 2.24/2.14 | 3601 | 17.996 | 20.138 | 0 | 0.005 | 0.039 | 2.501 | 7168 |
| sand-release | bounded-fifo | 2 | 1.88/2.24 | 3600 | 19.356 | 22.788 | 0 | 5.986 | 7.174 | 2.555 | 7168 |
| reservoir-breach | bounded-fifo | 2 | 1.95/2.57 | 3600 | 19.137 | 22.559 | 0 | 5.697 | 8.490 | 2.141 | 7168 |
| burning-forest | bounded-fifo | 2 | 1.99/2.88 | 3599 | 19.646 | 35.411 | 1 | 7.948 | 22.483 | 2.007 | 14168 |
| dirty-world-sweep | bounded-fifo | 2 | 2.12/2.15 | 3601 | 18.062 | 19.977 | 0 | 0.004 | 0.035 | 2.607 | 7168 |
| tiny-capacity | bounded-fifo | 2 | 2.70/2.68 | 3601 | 18.971 | 22.330 | 0 | 2.283 | 2.580 | 3.073 | 7553 |
| mixed-overload | bounded-fifo | 2 | 2.01/2.81 | 3600 | 18.744 | 22.106 | 0 | 6.318 | 7.109 | 2.387 | 7168 |
| quiet-world | bounded-fifo | 3 | 2.47/2.24 | 3601 | 18.249 | 21.434 | 0 | 0.005 | 0.034 | 2.343 | 7168 |
| explosive-lattice | bounded-fifo | 3 | 2.28/2.91 | 3601 | 18.104 | 20.325 | 0 | 0.005 | 0.011 | 2.725 | 7168 |
| sand-release | bounded-fifo | 3 | 2.08/1.91 | 3600 | 19.486 | 22.825 | 0 | 6.038 | 7.709 | 2.601 | 7168 |
| reservoir-breach | bounded-fifo | 3 | 1.48/2.43 | 3600 | 19.048 | 21.992 | 0 | 5.988 | 7.359 | 2.068 | 7168 |
| burning-forest | bounded-fifo | 3 | 2.02/2.58 | 3600 | 19.596 | 36.784 | 1 | 8.071 | 24.281 | 2.068 | 14168 |
| dirty-world-sweep | bounded-fifo | 3 | 1.97/1.77 | 3601 | 18.138 | 20.335 | 0 | 0.004 | 0.009 | 2.750 | 7168 |
| tiny-capacity | bounded-fifo | 3 | 2.95/2.92 | 3601 | 18.952 | 22.789 | 0 | 2.424 | 2.570 | 3.094 | 7553 |
| mixed-overload | bounded-fifo | 3 | 2.39/2.78 | 3600 | 19.127 | 21.967 | 0 | 6.357 | 8.294 | 2.759 | 7168 |
| quiet-world | bounded-focus | 1 | 2.38/2.21 | 3601 | 17.891 | 19.335 | 0 | 0.008 | 0.010 | 2.469 | 7168 |
| explosive-lattice | bounded-focus | 1 | 2.31/2.05 | 3601 | 18.219 | 20.589 | 0 | 0.007 | 0.021 | 2.577 | 7168 |
| sand-release | bounded-focus | 1 | 2.54/2.61 | 3601 | 19.488 | 21.185 | 0 | 7.674 | 8.423 | 3.375 | 7168 |
| reservoir-breach | bounded-focus | 1 | 2.31/2.82 | 3601 | 18.539 | 20.203 | 0 | 2.424 | 2.681 | 2.626 | 7168 |
| burning-forest | bounded-focus | 1 | 1.76/2.47 | 3598 | 19.035 | 42.977 | 2 | 6.938 | 30.502 | 2.875 | 14157 |
| dirty-world-sweep | bounded-focus | 1 | 1.87/2.47 | 3601 | 18.274 | 19.673 | 0 | 0.008 | 0.011 | 2.652 | 7168 |
| tiny-capacity | bounded-focus | 1 | 2.05/2.12 | 3601 | 19.033 | 21.031 | 0 | 1.910 | 2.398 | 3.050 | 7245 |
| mixed-overload | bounded-focus | 1 | 2.99/2.91 | 3601 | 18.849 | 21.551 | 0 | 6.265 | 7.680 | 2.335 | 7168 |
| quiet-world | bounded-focus | 2 | 2.72/1.74 | 3601 | 18.034 | 22.768 | 0 | 0.006 | 0.012 | 4.190 | 7168 |
| quiet-world | bounded-focus | 3 | 1.80/2.01 | 3601 | 17.937 | 20.446 | 0 | 0.007 | 0.049 | 2.853 | 7168 |

“Max queued dirty chunks” is the uploader's reported dirty backlog, not a count of visibly stale on-screen chunks. The overall observed maximum was 14,168. No presentation intervals were lost. Three burning-forest FIFO reps each had one interval over 33.3 ms; the single burning-forest focus screen had two. Its worst frame was 42.977 ms with 30.502 ms simulation CPU and 0.795 ms upload CPU, so the new upload cap does not eliminate non-upload frame outliers. Maximum upload CPU was 4.190 ms in quiet-world/focus rep 2. Camera/action timing is listed separately below; captures without scripted interaction have no such samples.

## Frame-p99 comparison with v2

Three repetitions are shown as a new range and mean for every FIFO fixture. Focus rows have one capture except quiet-world/focus, which has three per the follow-up. v2 ranges are from [native acceptance v2](native-acceptance-v2.md).

| Fixture | Policy | v2 frame-p99 range ms | New range or sample ms | New mean ms | Assessment |
|---|---|---:|---:|---:|---|
| quiet-world | bounded-focus | 17.975–18.163 | 17.891–18.034 | 17.954 | In v2 range |
| quiet-world | bounded-fifo | 17.888–18.238 | 17.957–18.249 | 18.122 | High end +0.011 ms; not a clear regression |
| explosive-lattice | bounded-focus | 17.957–18.239 | 18.219 | 18.219 | In v2 range |
| explosive-lattice | bounded-fifo | 18.039–18.374 | 17.996–18.133 | 18.078 | In v2 range |
| sand-release | bounded-focus | 19.409–19.589 | 19.488 | 19.488 | In v2 range |
| sand-release | bounded-fifo | 20.827–20.916 | 19.356–19.486 | 19.427 | Improved; all three now below 20 ms |
| reservoir-breach | bounded-focus | 18.372–18.826 | 18.539 | 18.539 | In v2 range |
| reservoir-breach | bounded-fifo | 18.535–18.862 | 18.889–19.137 | 19.025 | All three above v2; mean +0.275 ms; no-regression gate fails |
| burning-forest | bounded-focus | 19.549–19.621 | 19.035 | 19.035 | Below v2 range |
| burning-forest | bounded-fifo | 19.673–20.058 | 19.579–19.646 | 19.607 | Improved; all three now below 20 ms |
| dirty-world-sweep | bounded-focus | 17.979–18.202 | 18.274 | 18.274 | Single screen +0.072 ms; not enough to establish a clear regression |
| dirty-world-sweep | bounded-fifo | 17.978–18.140 | 18.062–18.138 | 18.090 | In v2 range |
| tiny-capacity | bounded-focus | 18.952–19.071 | 19.033 | 19.033 | In v2 range |
| tiny-capacity | bounded-fifo | 19.074–19.429 | 18.952–18.975 | 18.966 | Below v2 range |
| mixed-overload | bounded-focus | 18.615–18.880 | 18.849 | 18.849 | In v2 range |
| mixed-overload | bounded-fifo | 19.028–19.231 | 18.744–19.127 | 18.962 | Improved mean; new range overlaps v2 |

The explicit conservative merge rule is not met: reservoir-breach/bounded-FIFO has three accepted p99 values (18.889, 19.137, 19.048 ms), each above the v2 high of 18.862 ms. The lowest separation is 0.027 ms and the new mean is 0.275 ms above the v2 mean. This is a small shift in absolute terms, but the entire repeated range is above v2, so the branch is retained rather than waived through as noise. The prior single regression screen was 19.012 ms, consistent with this follow-up. Dirty-world-sweep/bounded-focus's single screen is 0.072 ms above v2's high and is treated as inconclusive, not a clear regression.

The two target FIFO means improved from v2: sand-release 20.861 to 19.427 ms; burning-forest 19.907 to 19.607 ms. The 20 ms frame target is now met in all six target captures. The 4 ms slice-p99 target remains unmet in all six: sand-release 5.986–6.122 ms and burning-forest 7.948–8.071 ms. These results do not turn the unmet slice target into a pass.

## Staleness and camera/action latency

The configured per-frame allowance is 96 copies: up to 80 dirty FIFO chunks and 16 reserved priority copies, split between recent action targets and fixed interior camera-view samples. Camera samples are uploaded only while their chunks are queued dirty. The ordinary queue keeps its own 80-copy service opportunity and remains FIFO. The reported maximum queued-dirty backlog ranged from 7,168 to 14,168 chunks across captures. It was 7,168 for every sand-release capture and 14,168 for all three burning-forest FIFO captures plus its focus screen. This is an observed queue backlog, not a count of every currently stale visible chunk; no total display-staleness pixel/chunk measurement is exposed by this suite.

Mixed-overload supplies 240 camera, paint, ignite, and detonate latency samples in each FIFO repetition and in the focus screen. Values are CPU-side event-to-presented-feedback p95 in ms; they exclude scanout/composition.

| Policy | Samples per action | New camera / paint / ignite / detonate p95 ms | v2 p95 values by repetition, camera / paint / ignite / detonate ms |
|---|---:|---|---|
| bounded-fifo | 240 each × 3 | rep1 18.03 / 18.18 / 17.91 / 18.05; rep2 17.57 / 17.71 / 17.65 / 17.66; rep3 18.00 / 18.03 / 18.12 / 18.25 | rep1 17.76 / 18.01 / 17.94 / 18.08; rep2 17.98 / 17.77 / 17.94 / 18.09; rep3 17.86 / 18.05 / 17.69 / 18.02 |
| bounded-focus | 240 each × 1 | 18.29 / 18.09 / 18.03 / 18.04 | rep1 18.19 / 17.93 / 18.19 / 18.10; rep2 18.28 / 18.08 / 18.22 / 18.15; rep3 17.96 / 18.24 / 18.07 / 18.00 |

FIFO three-repetition means were 17.87 / 17.97 / 17.89 / 18.00 ms, compared with v2 means of 17.87 / 17.94 / 17.86 / 18.06 ms, respectively. The differences are small (0.00 / +0.03 / +0.04 / −0.06 ms); no material camera/action latency change is evident. The single focus sample is descriptive and does not support a stable percentile comparison. The other fixture captures had no camera/action samples. Slice p99 and stale queue counts for every accepted capture are in the table above.

## Change, verification, and decision

The app now limits scheduled FIFO work to 80 chunks each frame and reserves 16 additional copies for up to eight refreshed action targets and eight queued dirty chunks sampled at fixed camera-view positions. The 96-copy limit is below the unchanged hard staging/copy cap of 256. Camera samples do not upload clean chunks. Priority copies do not dequeue their FIFO entry, so duplicate later copies may occur, but they cannot displace the FIFO's 80-copy allotment. The row-wise copy optimization remains. No simulation credit, charter threshold, fixture, acceptance gate, or credit profile changed.

`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (80 tests), release app build, and repository-root `mdbook build` passed on the implementation. The capture runner independently validated all 34 accepted breakdown captures. `git diff --check` passed before the implementation commits. The full 72-capture matrix was not rerun on this source; v2 and earlier results remain history.

Decision: retain the branch and worktree and do not push, open a PR, or merge. The target frame-p99 improvement is promising, but the explicit no-regression evidence gate fails on reservoir-breach/bounded-FIFO. The new frame budget reduced target upload work while maintaining finite FIFO service, and observed mixed-overload camera/action p95s were essentially unchanged. Further investigation should determine whether the reservoir shift repeats on a new quiet-host set before proposing another merge attempt.
