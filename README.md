# Cascade

A small native 2D cellular sandbox built to be overloaded. The simulation runs a fixed
budget of work credits per update, so piling on explosions, sand, water, and fire makes
effects resolve more slowly instead of stalling the frame.

See [CHARTER.md](CHARTER.md) for goals, invariants, and milestones.

Read the [documentation site](https://akofink.com/cascade-game-demo/).

## Status

The native app starts with a 1024 x 1024 `cascade-sim` world for interactive responsiveness.
Choose a larger validated square world at startup with `--world-size 4096`. At that size the app
preallocates both the normal and tiny-capacity simulation profiles (about 161.5 MiB of sim-owned
CPU arrays combined) and a 16 MiB material texture. Fixture preparation remains incremental.
The headless core includes complete rules, seeded fixtures, bounded and traditional policies,
and a benchmark runner. See the [native performance smoke report](docs/performance.md) for measured
M2 results and limitations.

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
cargo run -p cascade-app
# Full charter-size world (fixed size for this run)
cargo run -p cascade-app -- --world-size 4096
```

Drag with the left mouse button to pan; scroll to zoom. Arrow keys and WASD also pan.
Select a material in the overlay, then Shift-drag to paint (Air erases); each presentation
iteration admits at most 64 brush descriptors. Right-click ignites wood or explosives; Shift-right-click
detonates. Space pauses/resumes, `.` advances one slice, and `r` begins an incremental reset.
The overlay selects fixtures, reports preparation progress, allows cancellation, adjusts credits (25 to 100,000) with an explicit apply-and-restart action, switches between bounded and traditional scheduling by restarting the same fixture, and toggles deferred-cell highlighting. Traditional mode processes the ready frontier captured at update start without the slice credit cap. Hold **DESTROY PERFORMANCE** to prepare mixed overload and
admit a seeded capped disturbance stream; release stops new descriptors but leaves admitted work.
The overlay prominently labels the active scheduler, plots frame intervals beside pending-work
history, and offers a one-click same-fixture restart under the other policy. It also reports
scheduler credits, quanta, pending work, command coalescing/rejection, and upload staleness.

A native smoke check opens a window, incrementally prepares the mixed fixture, uploads the world,
checks a sampled pixel, pans, zooms, resizes/minimizes, then runs matched 120-frame bounded and
traditional disturbance bursts and reports per-policy frame/backlog measurements:

```sh
cargo run -p cascade-app -- --smoke
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

Run the headless benchmark under either policy. Fixture preparation is charged and incremental but excluded from measured slice time. Paired runs use the same versioned fixture, seed, dimensions, capacities, and budget.

```sh
cargo run --release -p cascade-bench -- --fixture mixed-overload --policy bounded --slices 600 --format csv
cargo run --release -p cascade-bench -- --fixture mixed-overload --policy traditional --slices 600 --format json
cargo run --release -p cascade-bench -- --fixture mixed-overload --policy bounded --slices 3600 --disturbances 28800 --disturbances-per-slice 8
```

Fixtures: `quiet-world`, `explosive-lattice`, `sand-release`, `reservoir-breach`, `burning-forest`, `dirty-world-sweep`, `tiny-capacity`, and `mixed-overload`. Options include `--width`, `--height`, and `--budget`. CSV/JSON records include per-slice credits, selection probes, executed quanta, backlog, elapsed nanoseconds, completion, and disturbance admission counters. Traditional runs intentionally exceed the configured credit allowance when their captured ready frontier requires it.

### Documentation site

Install [mdBook 0.5.4](https://github.com/rust-lang/mdBook/releases/tag/v0.5.4), then
run `mdbook build` or `mdbook serve`. The book reads the Markdown files in place.
Add a Markdown page to `SUMMARY.md` to publish it, including future performance
reports and curated benchmark summaries. Use relative links between listed pages;
link source files to `https://github.com/akofink/cascade-game-demo/blob/main/…`.
Pull requests build the book; pushes to `main` deploy it to GitHub Pages.

## License

MIT
