# Windowed bounded upload-path follow-up v1

Status: experiment complete; not eligible to merge. Target frame p99 improved in both targeted FIFO cells relative to v2 averages, but sand-release remains above 20 ms and two bounded cells exceeded their v2 frame-p99 repetition ranges. The required no-regression gate therefore failed. This is a focused follow-up, not a rerun of the 72-capture matrix.

Source revision: `8ba701c` (`perf(app): iterate chunk rows for texture uploads`), based on `d0bd78b`.
Suite runner revision: `d0bd78b` (unchanged).
Host: `akmac`, MacBook Air (Mac14,2), Apple M2/8 CPU cores/16 GB RAM, macOS 27.0.1; Rust 1.99.0.
Capture settings: release app, 4096² world, 1920×1080, 60 Hz target, `m2-16gb-v3` profile, one-minute load <3 before and after, 60-second runs, `--frame-breakdown`, and 180-second cooldown after every launched attempt. The suite acceptance gate and charter/profile values were unchanged. Power mode, thermal state, background activity, and physical display refresh were uncontrolled.

The measurement set consists of three accepted captures each for sand-release and burning-forest bounded-FIFO, plus one accepted capture of each of the 16 bounded-focus/bounded-FIFO fixture cells. Thus the target fixtures also have a separate regression-check capture. All 22 captures had zero interval drops and complete phase telemetry. The suite rejected load-gate attempts as recorded in the ignored runner logs; only accepted runs appear below. A full 72-capture matrix was not rerun on this source.

## Target bounded-FIFO captures

| Fixture | Rep | Load before/after | Frames | Frame p99 ms | Max ms | >33.3 ms | Slice p99 ms | Slice max ms |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| sand-release | 1 | 1.84/2.67 | 3601 | 20.584 | 24.035 | 0 | 6.143 | 8.017 |
| sand-release | 2 | 2.25/2.62 | 3601 | 20.368 | 25.994 | 0 | 6.084 | 8.509 |
| sand-release | 3 | 1.99/2.22 | 3600 | 20.641 | 25.176 | 0 | 6.259 | 7.230 |
| burning-forest | 1 | 1.91/2.89 | 3600 | 19.573 | 32.933 | 0 | 8.163 | 21.995 |
| burning-forest | 2 | 2.96/2.81 | 3598 | 19.708 | 32.972 | 0 | 8.079 | 21.439 |
| burning-forest | 3 | 2.32/2.52 | 3600 | 19.687 | 37.631 | 1 | 8.021 | 25.205 |

Sand-release frame p99 averaged 20.531 ms (20.368–20.641), versus the v2 mean 20.861 ms (20.827–20.916). All three runs improved against the lowest v2 repetition, but all three still miss the 20 ms frame target. Burning-forest averaged 19.656 ms (19.573–19.708), versus the v2 mean 19.907 ms (19.673–20.058); all three new runs are below 20 ms. Its slice p99 remains above the charter's 4 ms threshold in all repetitions. The targeted improvement does not establish that the complete acceptance cell passes.

Breakdown CSVs were validated one-to-one against measured intervals by the unchanged runner. Across the three target repetitions, conditional median chunk-upload CPU time on intervals over 20 ms was 2.886–3.138 ms for sand and 1.051–1.804 ms for forest; the corresponding median on intervals at or below 20 ms was 0.003 ms. The original breakdown baseline reported conditional median increases of 3.275 ms for sand and 2.825 ms for forest. This is consistent with reduced upload work on heavy frames, especially forest, but does not remove the sand p99 miss or establish causation for every long frame. The opt-in phase timer is diagnostic and is not acceptance evidence by itself.

## Bounded-cell regression check against v2

Frame p99 is compared to the min/max of the three accepted repetitions in [native acceptance v2](native-acceptance-v2.md). The one-capture comparison is a regression screen, not a replacement for the full matrix.

| Fixture | Policy | v2 frame p99 range ms | Follow-up frame p99 ms | In range? | Follow-up slice p99 ms |
|---|---|---:|---:|---|---:|
| quiet-world | bounded-focus | 17.975–18.163 | 18.218 | no, +0.055 | 0.008 |
| quiet-world | bounded-fifo | 17.888–18.238 | 18.120 | yes | 0.004 |
| explosive-lattice | bounded-focus | 17.957–18.239 | 17.945 | yes | 0.007 |
| explosive-lattice | bounded-fifo | 18.039–18.374 | 17.972 | yes | 0.004 |
| sand-release | bounded-focus | 19.409–19.589 | 19.544 | yes | 7.618 |
| sand-release | bounded-fifo | 20.827–20.916 | 20.357 | yes | 6.352 |
| reservoir-breach | bounded-focus | 18.372–18.826 | 18.382 | yes | 2.381 |
| reservoir-breach | bounded-fifo | 18.535–18.862 | 19.012 | no, +0.150 | 6.053 |
| burning-forest | bounded-focus | 19.549–19.621 | 19.207 | yes | 7.230 |
| burning-forest | bounded-fifo | 19.673–20.058 | 19.725 | yes | 8.155 |
| dirty-world-sweep | bounded-focus | 17.979–18.202 | 18.062 | yes | 0.007 |
| dirty-world-sweep | bounded-fifo | 17.978–18.140 | 18.029 | yes | 0.004 |
| tiny-capacity | bounded-focus | 18.952–19.071 | 18.809 | yes | 1.833 |
| tiny-capacity | bounded-fifo | 19.074–19.429 | 19.328 | yes | 2.416 |
| mixed-overload | bounded-focus | 18.615–18.880 | 18.719 | yes | 6.158 |
| mixed-overload | bounded-fifo | 19.028–19.231 | 19.040 | yes | 6.355 |

The frame-p99 no-regression rule is not met: quiet-world/bounded-focus and reservoir-breach/bounded-fifo are above their v2 repetition ranges. Slice p99 also exceeds the v2 repetition range in several cells (including sand-release/bounded-FIFO, reservoir-breach/bounded-FIFO, and both burning-forest policies). These are single follow-up measurements and do not establish a source-caused regression, but the task's evidence rule is conservative: do not merge this change on these results.

## Change and interpretation

The uploader previously fetched each pixel's rendered cell and pending flag through two separate coordinate-validation/indexing calls. The follow-up adds a clipped row iterator that zips the existing cell and pending arrays and converts each world/pending pair once. It preserves the same row-pitched staging bytes, FIFO upload plan, 240 scheduled plus 16 priority copy cap, queue write, and frame command copies. It defers no uploads and does not alter chunk staleness, camera/action scheduling, or credit/profile values. No material-visible latency samples were generated by fixture captures; the no-deferral implementation introduces no intentional camera/action delay. Slice p99 observations are reported above and are not claimed unchanged.

Verification on the source revision: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` passed. The row iterator test checks cell-view/pending equivalence and clipping at partial world edges.

## Decision

Keep the task branch and worktree; do not open or merge a PR. The cheap row-wise copy path lowered or was consistent with lower chunk-upload CPU time, and both target frame-p99 means improved from v2. However, sand-release still misses 20 ms, several slice-p99 values remain above 4 ms, and two bounded frame-p99 regression screens exceeded their v2 repetition ranges. The target/overall no-regression delivery gate is unmet. The full 72-capture matrix was not rerun on this source, and v2 remains the published complete-matrix history.
