# Cascade

A small native 2D cellular sandbox built to be overloaded. Its bounded policy runs a fixed
budget of work credits per update, so piling on explosions, sand, water, and fire makes
effects resolve more slowly instead of stalling the frame. A deliberately uncapped traditional
comparison policy processes its full pending frontier and can stall; both behaviors are measured.

See [CHARTER.md](CHARTER.md) for goals, invariants, and milestones.

Read the [documentation site](https://akofink.com/cascade-game-demo/).

## Status

The native app starts with a 1024 x 1024 `cascade-sim` world for interactive responsiveness.
Choose a larger validated square world at startup with `--world-size 4096`. At that size the app
preallocates normal and tiny-capacity simulation worlds (about 193.6 MiB of simulation-owned CPU
arrays combined), a 16 MiB CPU material grid, and a separate 16 MiB GPU texture. Fixture preparation
remains incremental. The headless core includes complete rules, seeded fixtures, bounded and
traditional policies, and a benchmark runner. See the [performance evidence](docs/performance.md)
for corrected headless results and limitations.

## Try it

1. Start the representative full-size release build:

   ```sh
   cargo run --release -p cascade-app -- --world-size 4096
   ```

2. Click **DESTROY PERFORMANCE** once the mixed fixture is visible (or hold `F`). The click latches the disturbance stream so the pointer stays free; a long hold still admits only while the button is down, and releasing a long hold stops new descriptors. A cyan ring is an acknowledgment, not a resolved effect. Gold cells, if deferred highlighting is on, are world work still waiting.
3. Left-drag to paint. The ring appears on the next presented frame. Right-drag pans; a right-click without a drag ignites, and Shift-right-click detonates.
4. Under **Same trigger**, restart FIFO, Focus, or Traditional on the same fixture. Focus services your neighborhood while the rest of the world defers. Traditional mode intentionally processes the complete captured frontier and may visibly stall.

![Cascade's traditional scheduler during the full-size overload comparison](docs/cascade-overload.png)

## Prior art

The mechanisms here are not new: sleeping objects, simulation level of detail, significance-driven
tick rates, capped physics catch-up, and progressive-refinement collision detection all predate this
project. Prior systems defer (capped catch-up), refine (progressive collision detection), or
substitute (cheaper proxy simulations) within one subsystem. Cascade explores making a per-update work
bound a whole-engine contract: player actions can grow pending work without limit, but every
player-caused workload is resumable work in quanta of bounded cost, under one per-slice budget.
Closest references:

- P. M. Hubbard, "Approximating Polyhedra with Spheres for Time-Critical Collision Detection,"
  ACM Transactions on Graphics 15(3), 1996.
- J. Dingliana and C. O'Sullivan, "Graceful Degradation of Collision Handling in Physically Based
  Animation," Computer Graphics Forum 19(3), 2000. [doi:10.1111/1467-8659.00416](https://doi.org/10.1111/1467-8659.00416)
- S. Chenney, "Simulation Level-Of-Detail," Game Developers Conference, 2001.
- Unity [Maximum Allowed Timestep](https://docs.unity3d.com/Manual/class-TimeManager.html) and
  Unreal [Significance Manager](https://dev.epicgames.com/documentation/en-us/unreal-engine/significance-manager).

## Run

```sh
cargo run --release -p cascade-app
# Full charter-size world (fixed size for this run)
cargo run --release -p cascade-app -- --world-size 4096
```

Left-drag paints the selected material (Air erases). Right-drag pans. Scroll zooms toward the cursor, with each wheel event clamped so one tick cannot skip several zoom levels. Hold WASD or the arrow keys to pan smoothly; each presented frame moves a fixed 12 pixels while the key is down. A right-click without a drag ignites wood or explosives; Shift-right-click detonates. Each presentation iteration admits at most 64 brush cells, stamped on press and along the stroke. Space pauses/resumes, `.` advances one slice, and `r` begins an incremental reset. Hold `F` to admit disturbances without using the pointer.
The overlay stays short: policy, the player-action line, **DESTROY PERFORMANCE**, and latency percentiles are visible without scrolling. A short click latches DESTROY PERFORMANCE; a press longer than 280 ms admits only while held and does not latch on release. If the live fixture is already completed mixed overload, DESTROY PERFORMANCE does not prepare it again. Details (fixtures, credits, raw counters) are behind a closed section. Credits still run from 25 to the selected profile allowance, currently 1,000,000, with an explicit apply-and-restart action. The default allowance and cap are read from `profiles/m2-16gb-v3.toml`. Same-trigger buttons restart FIFO, focus, or traditional scheduling. Focus submits the camera viewport and action neighborhoods through admitted simulator commands. The cyan overlay is the simulator's active focus regions. FIFO turns that priority off. The app starts in focus mode so the first actions stay ahead of the deferred world. Traditional mode snapshots every per-cell pending evaluation/blast channel at update start and processes that uncapped frontier; the ready rings are not its authority, two full-world scans are charged each slice, and work generated during the slice waits for the next one. Admitted disturbances are left in place when DESTROY PERFORMANCE stops.
The overlay reports camera/UI feedback latency and paint, ignite, and detonate action-to-first-visible-effect latency as nearest-rank p50/p95. It also plots frame intervals beside pending-work history. Burning wood is drawn as a distinct flame color so ignition is visible before the cell becomes air. Player target cells are exempt from the deferred-work tint so a mark stays readable while neighboring work stays gold.

A native smoke check opens a window, incrementally prepares the mixed fixture, uploads the world,
checks a sampled pixel, pans, zooms, resizes/minimizes, then runs matched 120-frame bounded and
traditional disturbance bursts. During those bursts it also admits a scripted paint/ignite/detonate
stream and a 1 px camera nudge, and prints `SMOKE_FEEL` p50/p95 latencies for bounded FIFO,
traditional, and bounded focus:

```sh
cargo run --release -p cascade-app -- --smoke --world-size 4096 --screenshot docs/cascade-overload.png
```

Headless tests do not open a window or need a GPU:

```sh
cargo test -p cascade-app
```

## Development

Requires Rust via `rustup`; the toolchain is pinned in `rust-toolchain.toml`.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run the headless benchmark under either policy. Fixture preparation is charged and incremental but excluded from measured slice time; its wall time is reported separately. Optional `--warmup-slices` are also timed separately, then the fixture is re-prepared so measurement starts from its descriptor state. Paired runs use the same versioned fixture, seed, dimensions, capacities, and budget. CSV/JSON include completed quanta per slice and cumulative work/wall time.

```sh
cargo run --release -p cascade-bench -- --fixture mixed-overload --policy bounded --slices 600 --warmup-slices 120 --format csv
cargo run --release -p cascade-bench -- --fixture mixed-overload --policy traditional --slices 600 --format json
cargo run --release -p cascade-bench -- --fixture mixed-overload --policy bounded --slices 3600 --disturbances 28800 --disturbances-per-slice 8
# Three-policy full-size player-action latency comparison
cargo +1.99.0 run --release -p cascade-bench -- --player-action-comparison --width 4096 --height 4096 --budget 1000000 --slices 512
```

Fixtures: `quiet-world`, `explosive-lattice`, `sand-release`, `reservoir-breach`, `burning-forest`, `dirty-world-sweep`, `tiny-capacity`, and `mixed-overload`. Options include `--width`, `--height`, and `--budget`. CSV/JSON records include per-slice credits, selection probes, executed quanta, backlog, elapsed nanoseconds, completion, and disturbance admission counters. Traditional runs intentionally exceed the configured credit allowance while scanning the full pending frontier and processing its captured work.

### Documentation site

Install [mdBook 0.5.4](https://github.com/rust-lang/mdBook/releases/tag/v0.5.4), then
run `mdbook build` or `mdbook serve`. The book reads the Markdown files in place.
Add a Markdown page to `SUMMARY.md` to publish it, including the [interactive feature tour](docs/tour.md), future performance reports, and curated benchmark summaries. Build the WASM module with `wasm-pack build crates/web --target web --release --out-dir ../../docs/tour/wasm --no-typescript` before a standalone book build. Use relative links between listed pages;
link source files to `https://github.com/akofink/cascade-game-demo/blob/main/…`.
Pull requests build the book; pushes to `main` deploy it to GitHub Pages. For rule illustrations and browser screenshot regeneration commands, see the relevant docs pages. To capture the live widgets locally, install Node dependencies with `npm ci`, serve the built book, then run `npm run tour:screenshots` with `TOUR_URL` and optionally `CHROME_PATH` set for the local browser.

## License

MIT
