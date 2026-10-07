# Metal GPU allocation high-water v1

Status: process-attributed Metal allocation high-water measured; allocations outside the Metal device metric remain unknown.

## Method

Captured a short functional native run with Xcode Instruments' `Metal System Trace` template on the Apple M2 / Metal host. The launched application used the release `cascade-app` binary at the task worktree revision, a 4096² world, 1920×1080 window, `mixed-overload` fixture, and bounded-focus policy. The app's capture duration was 8 seconds; fixture setup, app startup, and shutdown were also present in the trace. The trace itself lasted 13.945 seconds and the app exited successfully. This was a memory diagnostic, not an accepted timing capture: the run was shorter than 60 seconds, ran under Instruments, and did not use the quiet-machine performance gate. No timing values from it are used as acceptance evidence.

The trace's `metal-current-allocated-size` table is documented by Instruments as the app's total allocated Metal device memory (`MTLDevice.currentAllocatedSize`). The table targets the launched process, whose recorded PID was 17074. The maximum reported allocation among 93 events was:

| Metric | Result |
|---|---:|
| App Metal-device allocation high-water | 55,443,456 B (52.875 MiB) |
| Charter GPU memory budget | 128 MiB |
| Measured share of budget | 41.3% |
| Known explicit persistent windowed subtotal | 16,777,376 B |
| Difference above that known subtotal | 38,666,080 B (36.875 MiB) |

Capture command (the report was collected with the task worktree's release binary):

```sh
xcrun xctrace record --template 'Metal System Trace' \
  --output benchmarks/tmp/gpu-memory-mixed-functional.trace \
  --launch -- target/release/cascade-app \
  --capture-policy bounded-focus --fixture mixed-overload \
  --capture-seconds 8 --world-size 4096

xcrun xctrace export --input benchmarks/tmp/gpu-memory-mixed-functional.trace \
  --xpath '/trace-toc/run[@number="1"]/data/table[@schema="metal-current-allocated-size"]' \
  --output benchmarks/tmp/gpu-memory-mixed-table.xml
```

The reusable helper `./scripts/measure_metal_memory.py` performs the trace and reports the high-water. Raw traces and XML are not committed.

## Interpretation and limits

This is a measured high-water for allocations accounted by the app's Metal device, not an estimate derived from the resource descriptors. It captures the aggregate Metal allocations reported for the process, including app-created renderer resources and Metal allocations that the device metric attributes to the app. The excess over the descriptor subtotal is not individually attributable from this aggregate counter.

It is not a complete measurement of all physical GPU residency on the machine. WindowServer-owned surface backing and driver-private or global allocations that are not attributed to the app's `MTLDevice.currentAllocatedSize` are outside this counter and remain unknown. Thus 52.875 MiB is a lower bound on any such wider system-level total, not a claim that the app uses no additional GPU memory. This run measures the windowed path only; offscreen mode's explicit 25,071,776 B persistent subtotal remains an inventory, not a measured offscreen high-water. Results are specific to this M2, OS, renderer, and instrumented run.
