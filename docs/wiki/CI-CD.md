# CI/CD

GitHub Actions, many small single-purpose workflows so each can fail and re-run independently. Shared steps are reusable workflows (`workflow_call`, files prefixed `_`) plus the composite action `.github/actions/setup-rust-python` (pinned toolchain, uv, caches). Status: **Implemented** in Milestone 1 unless marked Planned.

## Reusable building blocks

Every workflow file stays at 60 lines or fewer.

| Workflow | Purpose |
|---|---|
| `_rust-test`, `_python-test` | nextest + doc tests (whole workspace, `ksim-py` included via the composite action's `PYO3_PYTHON`); maturin + pytest |
| `_coverage` | llvm-cov (excludes `ksim-py`) + pytest-cov, threshold gate |
| `_build-binaries`, `_build-wheels` | release binaries and wheels |
| `_build-image-docker` | Docker buildx image build, optional multi-arch push |
| `_build-image-podman` | Podman image build, optional push |
| `_image-tags` | namespace, version and tag list for `main`, `release/*` or a `v*` tag |
| `_scan-image` | Trivy scan; fails on HIGH or CRITICAL vulnerabilities that have a fixed version (`ignore-unfixed`), SARIF kept as an artifact |
| `_sign-image` | SBOM (anchore), cosign sign and attest |
| `_verify-image` | `ksim-bench --version` via `docker run` and `podman run` |
| `_bump-version` | version bump |
| `_nightly-pipeline` | per-branch test, bump, build and images, input `branches` (JSON) |

## Push and pull request workflows

| Workflow | Trigger | Content |
|---|---|---|
| `lint.yml` | push, PR | runs `cargo fmt`, `ruff check --fix`, `ruff format`, clippy `-D warnings`, mypy |
| `test.yml` | push, PR | Rust (nextest + doc tests) and Python (maturin + pytest) |
| `coverage.yml` | push, PR | llvm-cov + pytest-cov, threshold gate |
| `security-deps.yml` | push, PR, weekly | cargo-deny, cargo-audit, pip-audit |
| `security-codeql.yml` | push, PR, weekly | CodeQL for Rust and Python |
| `security-secrets.yml` | push, PR | gitleaks |
| `perf.yml` | push, PR | release-mode throughput gates: `ksim-core` engine steps and the `ksim-env` vector environment (`cargo test --release -p <crate> --test perf -- --ignored`) |
| `capi-smoke.yml` | push, PR | Builds `libksim`, generates `ksim.h` with cbindgen, compiles `crates/ksim-capi/tests/c/smoke.c` with `-Wall -Wextra -Werror` and runs one episode through the C ABI |
| `bench-smoke.yml` | push, PR | builds `ksim-bench`, runs `synthetic_steady` for `k8s` and `oracle` (2 seeds), writes the `ksim.analysis` leaderboard to the job summary |
| `container-check.yml` | PR touching container or source files | `_build-image-docker` and `_build-image-podman` (no push) + Trivy scan |

## Nightly chain

1. `nightly-detect.yml` (daily cron + manual): finds `main` and `release/*` branches with commits or merges in the last 24 h, then calls `_nightly-pipeline.yml` if any (tests, bump, build, images per branch).
2. `nightly-bump.yml`: per branch, computes and applies the version with the `ksim_ci` helper (`tools/ci/ksim_ci`), syncing `Cargo.toml` workspace version and `pyproject.toml`.
3. `nightly-build.yml`: binaries and wheels attached to a pre-release.
4. `nightly-images.yml` (Planned/in progress): `_image-tags`, multi-arch push with `_build-image-docker`, `_scan-image`, `_sign-image`, `_verify-image`.

### Version bump rules

- `main`: `X.Y.Z-nightly.YYYYMMDD` (the `-nightly` suffix replaces any previous pre-release suffix, base `X.Y.Z` is unchanged).
- `release/X.Y`: patch bump `X.Y.Z` to `X.Y.(Z+1)`; if the current version is already a pre-release of that patch, the pre-release suffix is dropped instead. The branch `X.Y` must match the version, otherwise the run fails.
- Any other branch is rejected.
- Version strings follow SemVer 2.0.0: prerelease identifiers are validated, numeric prerelease identifiers cannot contain leading zeroes, and build metadata such as `+build.9` is accepted and ignored for bump precedence.

## Release (Planned)

On a `v*` tag: `release-artifacts.yml` (binaries, wheels, GitHub release) and `release-images.yml` (`_image-tags`, tags `X.Y.Z`, `X.Y`, `latest`), reusing the same `_*` workflows.

## Dependency updates

Dependabot/Renovate for Cargo, uv, GitHub Actions and base images. Actions are pinned to the latest stable major or commit SHA.

## Permissions of reusable workflows
Reusable image-build workflows declare no `permissions` of their own: they inherit the caller's token, so `container-check` builds with `contents: read` while the nightly and release image workflows grant `packages: write` for the push. A called workflow that declares more than its caller grants fails at startup. Dependency review is not used because it requires the dependency graph with GitHub Advanced Security; `security-deps` (cargo-deny, cargo-audit, pip-audit) covers vulnerable dependencies instead.
