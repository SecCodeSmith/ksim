# ksim project rules

Trace-driven Kubernetes autoscaler training simulator: Rust core, thin Python/Gymnasium bindings. Plan: `docs/PLAN.md`. Wiki: `docs/wiki/`.

## Rules
- TDD always, test first: red, green, refactor. No production code without a failing test that demands it. Implementers never modify tests.
- SOLID: extend through traits (`Autoscaler`, `TraceSource`, `PerformanceModel`, `RewardFn`, `Metric`), inject via builders, reuse contract tests.
- Clean code: intention-revealing names, small functions. No noise comments; comment only a why, a citation, or an upstream-parity note. No commented-out code.
- Latest stable dependencies only, no pre-releases. Rust pinned in `rust-toolchain.toml`; Python via `uv` lockfile.
- Update the relevant `docs/wiki/` page in the same change as the code.

## Agents (`.claude/agents/`)
- `test-writer` (Haiku 4.5): writes failing tests.
- `implementer` (Sonnet): makes them pass, then refactors; never edits tests.
- `reviewer` (Haiku 4.5): read-only review for SOLID, clean code, noise comments, test-first order.
- The main session orchestrates; agents communicate through messages; at most 5 agents run concurrently, with disjoint file ownership.

## Commands
- Rust tests: `cargo nextest run --workspace` (or `cargo test --workspace`)
- Python tests: `uv run maturin develop && uv run pytest`
- Lint Rust: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings`
- Lint Python: `uv run ruff check . && uv run ruff format --check . && uv run mypy python`
- Dependencies: `cargo deny check`
