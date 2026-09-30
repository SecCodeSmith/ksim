# Containers

Status: images and pipelines are part of Milestone 1. Docker and Podman are both supported, including rootless Podman.

## Images

| Image | Contents |
|---|---|
| `ghcr.io/seccodesmith/ksim` | minimal runtime (non-root) with `ksim-bench` |
| `ghcr.io/seccodesmith/ksim-train` | Python + `ksim` wheel + gymnasium for training jobs |

Tags: `nightly`, `nightly-YYYYMMDD`, `X.Y.Z`, `latest`. Images carry OCI labels, an SBOM (syft) and provenance attestations, and are signed with cosign.

## Build

One multi-stage `Containerfile` (also available as `Dockerfile`): a Rust builder stage on the pinned latest stable `rust` image, then a distroless or `debian-slim` runtime; a second target `ksim-train`.

```sh
docker build --target runtime -t ksim .
podman build --target runtime -t ksim .
podman build --target train -t ksim-train .
```

## Run

```sh
docker run --rm ghcr.io/seccodesmith/ksim:nightly ksim-bench --version
podman run --rm ghcr.io/seccodesmith/ksim:nightly ksim-bench --version
```

## Compose and Kubernetes YAML

`compose.yaml` works with `docker compose` and `podman compose`; a manifest for `podman kube play` lives in `deploy/`.

## Verify signatures

```sh
cosign verify ghcr.io/seccodesmith/ksim:<tag> \
  --certificate-identity-regexp 'https://github.com/SecCodeSmith/.*' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
```

The identity pattern assumes keyless signing from this repository's workflows.

## Vulnerability policy
The train stage upgrades Debian packages at build time and uninstalls `pip` after installing the wheel, because pip's vendored `msgpack` and `pkg_resources` carry fixable CVEs and pip is not needed at run time. Pull requests scan both images with Trivy and fail on HIGH or CRITICAL findings that already have a fixed version; findings without an upstream fix are reported in the SARIF artifact but do not block, since no rebuild can remove them. `limit-severities-for-sarif` keeps the SARIF (and so the exit code) to HIGH and CRITICAL; without it trivy-action scans every severity for SARIF output and fails on fixable LOW findings too.
