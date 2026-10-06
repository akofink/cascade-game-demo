# Native offscreen versus windowed capture v1

Status: one accepted 60-second capture per mode and policy on the same release build; single-machine comparison, not a repeatability study.

## Method

The release app source is commit `f9989ed32210919a9c9f9fbf729b5bf5b51f9669`; the release app was built once and run as a fresh process per policy. Both modes used the `mixed-overload-v1` fixture, 4096 x 4096 world, 1,000,000-credit bounded profile, 1920 x 1080 render size, identical scripted action/disturbance stream, and 60-second measured window. Fixture preparation and renderer warm-up preceded measurement. The machine-load gate required the one-minute load to be below 3 before and after each run. Each accepted capture met that gate; two failed windowed focus attempts and one failed windowed traditional attempt are listed below, not included in the comparison.

Reference machine: MacBook Air (Mac14,2), Apple M2/8 CPU cores/16 GB RAM, integrated M2 GPU, macOS 27.0.1, built-in 2560 x 1664 display; Rust/Cargo 1.99.0. `Cargo.lock` SHA-256: `c4084ec2080b2c4da29d605573b159b02504f86af481d13c5807e565d419afe7`. The output target was fixed at 1920 x 1080. The offscreen overlay used a 2x logical scale (960 x 540 points), matching the reference Mac windowed capture's physical/logical scale.

Windowed captures use the existing `--capture-policy POLICY --capture-seconds 60` path and acquire/present a window surface. Their frame intervals are sampled after present. Offscreen captures use `--offscreen-capture --capture-policy POLICY --capture-seconds 60`; they create neither a window nor a surface and render the grid and overlay into an RGBA8 GPU texture. They pace from a monotonic clock at a fixed 60 Hz cadence, measure frame-start-to-frame-start intervals, and skip missed cadence slots rather than catch up. Every loop advances one normal simulation slice, uses the same capped dirty-chunk upload scheduler, and submits the same two render passes. GPU queue submission is asynchronous. No render-target readback, device polling, or synchronous GPU wait occurs in the measured loop.

The two interval clocks are intentionally not identical: the windowed interval includes surface acquisition/presentation behavior and display refresh pacing; the offscreen interval includes the fixed-cadence wait and CPU-side frame work. Neither is a GPU-duration measurement. Offscreen results do **not** measure display presentation, vsync, compositor scheduling, or scanout. CPU timings are monotonic wall-clock durations around simulation, upload work, and render encoding/submission and may include OS preemption.

## Accepted paired results

One-minute load before/after is shown for every accepted capture. Each cell is one 60-second run; intervals are sample count, p50/p95/p99/max in milliseconds. `>33.3` counts intervals longer than 33.3 ms. CPU columns are maximum observed simulation, upload, and render-submission CPU time per frame in milliseconds, not percentiles.

| Policy | Mode | Load before/after | Intervals | p50 | p95 | p99 | Max | >33.3 ms | Max sim CPU | Max upload CPU | Max submit CPU |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Bounded FIFO | Windowed | 2.72 / 2.89 | 3,600 | 16.692 | 17.974 | 20.684 | 25.149 | 0 | 5.595 | 9.127 | 1.296 |
| Bounded FIFO | Offscreen | 2.98 / 2.59 | 3,599 | 16.654 | 17.489 | 18.948 | 19.836 | 0 | 8.805 | 8.222 | 2.542 |
| Bounded focus | Windowed | 2.53 / 2.04 | 3,600 | 16.680 | 17.192 | 17.796 | 20.282 | 0 | 4.894 | 4.840 | 0.708 |
| Bounded focus | Offscreen | 2.62 / 2.45 | 3,600 | 16.656 | 17.394 | 18.987 | 20.190 | 0 | 6.234 | 8.762 | 1.634 |
| Traditional | Windowed | 2.37 / 2.75 | 928 | 66.636 | 67.183 | 107.190 | 120.768 | 928 | 108.323 | 4.067 | 0.368 |
| Traditional | Offscreen | 2.68 / 2.54 | 812 | 66.746 | 83.596 | 116.455 | 119.759 | 812 | 108.186 | 4.938 | 0.577 |

For bounded FIFO, the offscreen p99 was 8.4% lower and maximum interval 21.1% lower than this paired windowed run. For bounded focus, offscreen p99 was 6.7% higher while the maxima were nearly equal (20.190 vs 20.282 ms). For traditional, offscreen p99 was 8.6% higher and maximum was 0.8% lower. All three offscreen/windowed pairs had zero bounded-mode intervals over 33.3 ms; every traditional interval exceeded 33.3 ms. Differences are observations from one pair each, not stable ratios or evidence that one mode is generally faster.

The maximum simulation CPU was higher offscreen in both bounded captures (8.805 vs 5.595 ms FIFO; 6.234 vs 4.894 ms focus) and nearly equal in traditional (108.186 vs 108.323 ms). Upload and submit maxima also varied by mode and policy. This demonstrates why offscreen capture is a useful independent timing path, not a replacement or conversion factor for windowed results.

## Game-feel samples

Each capture retained 240 samples per category. Windowed action values use the existing CPU event-to-presented-texture-upload method. Offscreen values are **scripted proxies**: camera changes are timed to render submission, while action latency uses CPU-detected authoritative effects and a chunk upload queue write. They do not confirm GPU completion, visible pixels, presentation, or scanout.

| Policy | Mode | Camera p95 ms | Paint p95 ms | Ignite p95 ms | Detonate p95 ms |
|---|---|---:|---:|---:|---:|
| Bounded FIFO | Windowed | 17.83 | 17.93 | 17.57 | 18.72 |
| Bounded FIFO | Offscreen scripted proxy | 18.00 | 18.29 | 17.94 | 19.23 |
| Bounded focus | Windowed | 17.24 | 17.15 | 17.25 | 17.12 |
| Bounded focus | Offscreen scripted proxy | 19.19 | 19.07 | 19.04 | 19.47 |
| Traditional | Windowed | 66.92 | 68.50 | 66.96 | 135.21 |
| Traditional | Offscreen scripted proxy | 84.30 | 84.02 | 83.72 | 167.15 |

## Rejected attempts

These captures completed, but are excluded because the one-minute load was not below 3 both before and after. Their statistics are retained here as adverse context, not combined with accepted values.

| Policy | Mode | Load before/after | p99 ms | Max ms | Other observation |
|---|---|---:|---:|---:|---|
| Bounded focus | Windowed | 2.38 / 3.60 | 17.972 | 28.765 | Max sim/upload/submit 5.233/4.805/12.384 ms |
| Bounded focus | Windowed | 2.73 / 3.17 | 17.726 | 34.822 | One interval over 33.3 ms; max sim/upload/submit 5.001/4.769/12.384 ms |
| Traditional | Windowed | 2.20 / 3.07 | 107.007 | 122.539 | Every interval over 33.3 ms; max sim CPU 108.210 ms |

## Relation to the prior windowed evidence and limits

The established [native sustained capture v2](native-sustained-v2.md) remains intact and continues to be the three-repetition windowed result. These current-build windowed captures are a separate paired comparison, not a replacement or an extension of its sample count. The new offscreen numbers also remain separate evidence.

This comparison covers one M2 machine, one mixed-overload fixture, one 60-second capture per accepted mode/policy pair, and the current release build. It does not establish repeatability, generalize to other GPUs/backends, measure GPU execution duration, or satisfy every charter acceptance run. Offscreen fixed-cadence intervals cannot substantiate presentation-level responsiveness or display pacing. Use windowed captures for presentation claims and report offscreen measurements under their explicit label and limits.
