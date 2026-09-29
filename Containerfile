# syntax=docker/dockerfile:1
FROM docker.io/library/rust:1.98.1-slim-bookworm AS builder
ENV RUSTUP_TOOLCHAIN=1.98.1
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p ksim-bench \
    && install -D target/release/ksim-bench /out/ksim-bench

FROM docker.io/library/rust:1.98.1-slim-bookworm AS wheel-builder
ENV RUSTUP_TOOLCHAIN=1.98.1
COPY --from=ghcr.io/astral-sh/uv:0.12.20 /uv /usr/local/bin/uv
ENV UV_PYTHON_INSTALL_DIR=/opt/python
RUN uv python install 3.13
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    uv tool run --from "maturin>=1.15.0,<2" maturin build --release --locked \
        --interpreter "$(uv python find 3.13)" --out /wheels

FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
LABEL org.opencontainers.image.title="ksim" \
      org.opencontainers.image.description="Trace-driven Kubernetes autoscaler simulator" \
      org.opencontainers.image.source="https://github.com/SecCodeSmith/Cubernetes-Simulator-for-autoscaler" \
      org.opencontainers.image.licenses="LicenseRef-SecCodeSmith-Commercial" \
      org.opencontainers.image.vendor="SecCodeSmith"
COPY --from=builder /out/ksim-bench /usr/local/bin/ksim-bench
USER nonroot
CMD ["ksim-bench", "--help"]

FROM docker.io/library/python:3.13-slim-bookworm AS train
LABEL org.opencontainers.image.title="ksim-train" \
      org.opencontainers.image.description="ksim with Python bindings for autoscaler training" \
      org.opencontainers.image.source="https://github.com/SecCodeSmith/Cubernetes-Simulator-for-autoscaler" \
      org.opencontainers.image.licenses="LicenseRef-SecCodeSmith-Commercial" \
      org.opencontainers.image.vendor="SecCodeSmith"
ENV PYTHONUNBUFFERED=1 PIP_NO_CACHE_DIR=1
RUN useradd --create-home --uid 10001 ksim
COPY --from=wheel-builder /wheels /tmp/wheels
RUN pip install "$(ls /tmp/wheels/*.whl)[gym]" && rm -rf /tmp/wheels
COPY --from=builder /out/ksim-bench /usr/local/bin/ksim-bench
USER ksim
WORKDIR /home/ksim
CMD ["ksim-bench", "--help"]
