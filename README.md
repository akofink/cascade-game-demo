# Cascade

A small native 2D cellular sandbox built to be overloaded. The simulation runs a fixed
budget of work credits per update, so piling on explosions, sand, water, and fire makes
effects resolve more slowly instead of stalling the frame.

See [CHARTER.md](CHARTER.md) for goals, invariants, and milestones.

## Status

Milestone 1 (headless foundation) in progress.

## Development

Requires Rust via `rustup`; the toolchain is pinned in `rust-toolchain.toml`.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## License

MIT
