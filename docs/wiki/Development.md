# Development

## Test-driven development

TDD is always test first: red, green, refactor.

1. Red: write a failing test that demands the behavior; confirm it fails for the right reason.
2. Green: write the minimum production code that passes.
3. Refactor with tests green.

Git history shows `test:` commits before the implementation commits for each feature. Baselines start from tests ported from upstream Kubernetes; metrics from hand-computed examples in the papers. CI gates on test failure and coverage regression (target 90 % for core, autoscalers, metrics).

## SOLID mapped to traits

- Single responsibility: scheduler, performance model, metrics and trace IO are separate modules/crates.
- Open/closed: new behavior implements `Autoscaler`, `TraceSource`, `PerformanceModel`, `RewardFn`, `Metric`, `SchedulerPlugin`.
- Liskov: all implementations pass the shared contract test macros.
- Interface segregation: `ReplicaRecommender`, `ResourceRecommender`, `NodeProvisioner` instead of one fat trait.
- Dependency inversion: the engine takes traits through `SimulationBuilder`; tests inject fakes.

## Clean code, no noise comments

Intention-revealing names and small functions. Comments only where they add what code cannot: a why, a formula source or citation, an upstream-parity note. No comments restating code, no commented-out code, no TODO without an issue link. Enforced by review, `clippy::pedantic` (selected lints) and `ruff`.

## Dependency policy

Latest stable versions only, no pre-releases, yanked or unmaintained crates. Rust toolchain pinned in `rust-toolchain.toml`; Python managed with `uv` and a lockfile. `cargo deny`, `cargo outdated` and Dependabot keep versions current.

## Multi-agent workflow

Defined in `.claude/agents/` and coordinated by the main Claude Code session (the orchestrator).

| Role | Model | Job |
|---|---|---|
| test-writer | Haiku 4.5 | writes failing tests first from the wiki spec or upstream reference |
| implementer | Sonnet | makes tests pass minimally, then refactors; never modifies tests |
| reviewer | Haiku 4.5 | read-only review: SOLID, clean code, noise comments, test-first ordering |
| orchestrator | main session | splits work by file ownership, dispatches, integrates |

Rules: agents communicate via messages through the orchestrator; at most 5 agents run concurrently; parallel implementers own disjoint files; implementers never modify tests (a needed test change is a separate, justified `test:` commit).

## Commands

```sh
cargo nextest run --workspace        # or cargo test --workspace
uv run maturin develop && uv run pytest
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings
uv run ruff check . && uv run mypy python
cargo deny check
```
