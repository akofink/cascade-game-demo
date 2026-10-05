# Agent Guidance

- `CHARTER.md` is the source of truth for scope, invariants, and milestones. Do not expand v0.1 past it.
- Work on a task branch in a worktree under `~/dev/worktrees/cascade-game-demo/<slug>/`, open a PR to `main`, and merge (squash) once CI passes. `main` is protected; never push to it directly.
- Before pushing, run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- Headless tests must not need a GPU or window. Keep `crates/sim` free of graphics, windowing, and wall-clock dependencies.
- Keep code easy to follow. Comment invariants and non-obvious semantics only. Keep docs to `README.md`, `CHARTER.md`, and `docs/{architecture,rules,performance}.md`.
- Use conventional commit titles. Keep commits GPG-signed.
- Do not commit raw benchmark captures; commit small summaries under `benchmarks/results/`.
