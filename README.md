# Cascade

A small native 2D cellular sandbox built to be overloaded. The simulation runs a fixed
budget of work credits per update, so piling on explosions, sand, water, and fire makes
effects resolve more slowly instead of stalling the frame.

See [CHARTER.md](CHARTER.md) for goals, invariants, and milestones.

## Status

The headless simulation core is on main. The native shell renders a placeholder
material grid and is not driven by the simulation yet.

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
  Unreal [Significance Manager](https://dev.epicgames.com/documentation/en-us/unreal-engine/significance-manager-in-unreal-engine).

## Run

```sh
cargo run -p cascade-app
```

Drag with the left mouse button to pan. Scroll to zoom. Arrow keys and WASD also pan.
The overlay graphs actual frame intervals and shows upload backlog. **DESTROY PERFORMANCE**
is visible and not wired yet.

A native smoke check opens a window, pans, zooms, resizes, and exits:

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

## License

MIT
