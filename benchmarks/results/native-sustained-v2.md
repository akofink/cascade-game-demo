# Native sustained capture v2: focus-membership cache

Status: three valid 60-second captures per policy; post-cache source; MacBook Air M2; 4096² world; 1920x1080 presenting surface.

## Method and gate

Each run used `caffeinate -dimsu target/release/cascade-app --world-size 4096 --capture-policy POLICY --capture-seconds 60`. The fresh process prepared `mixed-overload-v1`, warmed the renderer outside measurement, then captured 60 seconds while DESTROY PERFORMANCE disturbance admission was held. Every run printed `SMOKE_CAPTURE_STARTED` with render `1920x1080`, produced `SMOKE_RESULT ok`, measured 3,600 intervals for each bounded policy (traditional's slower cadence yields about 926 frames), and reported zero interval-buffer drops. Window presentation remained active; no occlusion/outdated abort occurred in the retained runs. One-minute load was below 3 before and after every retained capture.

Three additional launch attempts were excluded because the one-minute load exceeded 3 at the end (start/end: traditional 2.36/3.12, FIFO 1.94/3.05, FIFO 2.58/3.60, focus 2.35/3.36). They are not included in the distributions. After the gate failures, accepted captures were completed as load allowed; consequently the accepted run chronology did not preserve a strict alternating policy order. Exact retained start/end one-minute loads and capture chronology:

| Start EDT | Policy | Load before/after |
|---|---|---:|
| 18:46 | Bounded focus | 1.91 / 2.65 |
| 18:47 | Bounded FIFO | 2.47 / 2.75 |
| 18:48 | Traditional | 2.61 / 2.48 |
| 18:57 | Bounded focus | 2.31 / 2.59 |
| 18:58 | Bounded FIFO | 2.46 / 2.38 |
| 19:03 | Traditional | 1.84 / 2.84 |
| 19:06 | Traditional | 1.97 / 2.72 |
| 19:11 | Bounded FIFO | 2.36 / 2.81 |
| 19:13 | Bounded focus | 2.22 / 2.09 |

Reference machine: MacBook Air (Mac14,2), M2/8 CPU cores/16 GB RAM, integrated M2 GPU, macOS 27.0.1, Rust/Cargo 1.99.0, built-in 2560x1664 display; presentation surface 1920x1080. The load gate does not guarantee absence of OS/driver interference.

## Frame intervals and simulation CPU

| Rep | Policy | Intervals | p50 ms | p95 ms | p99 ms | max ms | >33.3 ms | max simulation CPU ms |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Bounded focus | 3,600 | 16.681 | 17.185 | 17.654 | 20.117 | 0 | 4.876 |
| 2 | Bounded focus | 3,600 | 16.677 | 17.197 | 17.709 | 20.626 | 0 | 4.947 |
| 3 | Bounded focus | 3,600 | 16.682 | 17.181 | 17.701 | 20.235 | 0 | 5.062 |
| 1 | Bounded FIFO | 3,600 | 16.694 | 17.865 | 20.621 | 26.856 | 0 | 5.113 |
| 2 | Bounded FIFO | 3,600 | 16.694 | 17.875 | 20.506 | 25.266 | 0 | 5.171 |
| 3 | Bounded FIFO | 3,600 | 16.691 | 17.817 | 20.572 | 25.158 | 0 | 5.600 |
| 1 | Traditional | 930 | 66.624 | 67.123 | 107.011 | 120.618 | 930 | 108.256 |
| 2 | Traditional | 927 | 66.632 | 67.201 | 106.942 | 120.712 | 927 | 108.132 |
| 3 | Traditional | 926 | 66.635 | 67.345 | 106.999 | 120.904 | 926 | 108.271 |

Bounded policies had zero intervals over 33.3 ms among 21,600 valid intervals. The earlier 34.342 ms focus interval was not reproduced. Bounded-focus frame p99 met 20 ms in all three captures; FIFO frame p99 remained above 20 ms in all three (20.506–20.621 ms). Both bounded policies still had maximum simulation CPU above the 4 ms target: focus 4.876–5.062 ms; FIFO 5.113–5.600 ms. Thus the cache improves but does not close the bounded slice maximum gap. Traditional remains intentionally expensive and had every interval above 33.3 ms.

## Work composition and presentation attribution

The capture reports counts for the slice with maximum simulation CPU. Focus maxima executed 33,831 / 34,459 / 33,425 evaluations, about 6,810 recovery quanta, one to three commands, and 45,452–45,460 selection probes. FIFO maxima executed 35,290–35,291 evaluations, about 23,527–23,532 recoveries, one command, and 58,824–58,832 selections. The same 1,000,000-credit allowance is charged in bounded mode. Evaluation plus selection dominates these worst slices; action-record tracking is disabled in the native app. Per-chunk focus memoization lowers repeated region checks without changing scheduling or credit allowance.

Upload CPU maxima were 4.764–4.852 ms in focus and 8.865–9.382 ms in FIFO, while maximum upload backlog remained zero. No bounded frame exceeded 33.3 ms, so upload peaks did not correspond to long presentation intervals in these captures. Traditional's worst frame (about 120.7–120.9 ms) coincided with about 102.3 ms simulation CPU, about 2.2 ms upload, and below 0.43 ms submit. It performed 33,554,432 recovery/frontier-scan quanta per such slice: two mandatory full scans of the 4096² world. Action history, recovery cursor, cache/memory, upload, and presentation effects are therefore distinguishable: action tracking is disabled; the cursor work is counted; focus-cache headless/native results improve; upload peaks have no backlog/slow-frame association; and traditional stalls are explained by the required full scan.

## Game-feel p95

Each capture window reported 240 samples per interaction/camera category. Across runs, bounded-focus camera and action p95s were approximately 17.2 ms; bounded-FIFO camera/paint/ignite/detonate p95s were approximately 17.7/18.0/17.6/18.5 ms. Traditional camera/paint/ignite/detonate p95s were approximately 67/68/67/134 ms. Bounded feedback remains under the 50 ms target.

## Before/after and remaining gaps

The prior valid native suite reported bounded-focus simulation maxima 5.976–6.227 ms and p99 frame intervals 18.132–18.224 ms; FIFO maxima 4.903–5.536 ms and frame p99 20.758–20.773 ms. Post-cache focus maxima are 4.876–5.062 ms and frame p99 17.654–17.709 ms. FIFO is not routed through focus membership and remains about 5.1–5.6 ms max / 20.506–20.621 ms p99. No credit allowance reduction was made, so scheduler throughput was not reduced. The headless v4 calibration reports unchanged work quanta/slice and the measured p99 improvements; its 8 burning-forest >4 ms outliers remain disclosed.

**Upload-fix follow-up, 2026-10-07:** The R8 chunk uploader previously supplied a 32-byte row pitch to `Queue::write_texture`. In wgpu-core 30.0.1 this selected its chunked staging path: each 32x32 chunk required 32 row copies into an 8,192-byte, 256-byte-pitch staging allocation for a 1,024-byte logical payload. The conversion also searched up to 64 action-acknowledgment targets for every texel. Up to 256 planned writes plus 16 separate priority writes could exceed the 256-copy cap. The fix uses a chunk-local 1,024-cell acknowledgment mask, 256-byte row-pitched texture data to select the contiguous staging-write path, and a 240 queued plus 16 priority upload reservation. The 1M simulation credit allowance was retained.

Three accepted 60-second windowed captures on `dc237d2`, each with load below 3 before and after and no occlusion, report:

| Rep | Load before/after | Intervals | p50 ms | p95 ms | p99 ms | max ms | >33.3 ms | max sim CPU ms | max upload CPU ms | max chunks | payload bytes | staging bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 2.27 / 1.92 | 3,600 | 16.679 | 17.335 | 18.096 | 20.023 | 0 | 5.287 | 3.468 | 242 | 247,808 | 1,928,256 |
| 2 | 1.94 / 2.85 | 3,600 | 16.685 | 17.288 | 17.944 | 21.365 | 0 | 5.188 | 3.729 | 242 | 247,808 | 1,928,256 |
| 3 | 2.76 / 2.89 | 3,600 | 16.682 | 17.318 | 18.000 | 20.468 | 0 | 5.410 | 3.538 | 242 | 247,808 | 1,928,256 |

The upload-CPU peak frame used 132, 132, and 128 writes across reps 1-3, with logical payloads of 135,168, 135,168, and 131,072 bytes; staging sizes of 1,051,776, 1,051,776, and 1,019,904 bytes; and row padding of 916,608, 916,608, and 888,832 bytes. Versus the prior valid FIFO captures in v2, p99 fell from 20.506-20.621 ms to 17.944-18.096 ms, and maximum upload CPU fell from 8.865-9.382 ms to 3.468-3.729 ms. All three post-fix windowed p99s meet the 20 ms target for mixed-overload FIFO; the all-fixture acceptance remains incomplete. The maximum simulation CPU remains above 4 ms. Rep 1 executed 35,290 evaluations, 3 blasts, 23,531 recoveries, 1 command, and 58,832 selections. At the current costs (24/24/4/12/1 credits respectively), this is exactly 1,000,000 credits. Rep 2 executed 35,291 evaluations, 2 blasts, 23,532 recoveries, 1 command, and 58,828 selections, also exactly 1,000,000 credits. Rep 3 executed 35,290 evaluations, 4 blasts, 23,527 recoveries, 1 command, and 58,824 selections, also exactly 1,000,000 credits. This is aggregate budget saturation, not a credit-limit violation. Lowering credits would reduce useful work throughput, so the profile was retained. One current-head offscreen capture failed the post-run load gate (2.29/3.44) and is excluded; one earlier accepted offscreen capture on the upload-fix branch is documented in [offscreen comparison v1](native-offscreen-v1.md).

**Remaining:** observed bounded simulation CPU maxima remain above 4 ms (current FIFO 5.188-5.410 ms, existing focus 4.876-5.062 ms, headless burning-forest maximum 9.767 ms). The headless p99 values remain below 4 ms, though 8/5,400 burning-forest samples exceeded 4 ms. The full 60-second all-fixture acceptance is incomplete. The one prior 34.342 ms bounded-focus interval did not recur in the original 21,600 bounded intervals, but remains historical evidence. These are empirical observations on one machine, not wall-clock guarantees.
