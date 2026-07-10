# SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
# SPDX-License-Identifier: Apache-2.0

# Conformance-fixture rebuild container (Rust).
#
# Build:
#     docker build -t yamlsigil-conformance-rebuild-rs conformance/rebuild-rs
#
# Run (from the repository root, with conformance/ mounted at /work):
#     docker run --rm \
#         -v "$(pwd)/conformance:/work" \
#         yamlsigil-conformance-rebuild-rs
#
# The entrypoint runs as UID 1000 and writes regenerated fixtures into
# each subdirectory of /work. Re-running the container MUST produce
# bit-identical output to what's already shipped; any diff is a
# generator / spec drift to investigate.

FROM rust:1.95.0-trixie AS builder

# Non-root build user; owns /src so cargo can write target/.
RUN useradd --uid 1000 --create-home --shell /bin/bash builder \
    && install -d -o builder -g builder /src

WORKDIR /src
COPY --chown=builder:builder Cargo.toml Cargo.lock /src/
COPY --chown=builder:builder xtask /src/xtask
COPY --chown=builder:builder vendor /src/vendor
COPY --chown=builder:builder src /src/src

USER builder

# --locked aborts if Cargo.lock would be regenerated, which is what
# we want for reproducibility. `-p` builds just the rebuild_all bin
# (skips the xtask dev tool).
RUN cargo build --release --locked -p yamlsigil-conformance-rebuild

# --- Runtime stage --------------------------------------------------
# The release binary is statically-enough linked for Debian Trixie's
# libc (Rust links libstd statically by default). Use a minimal
# Trixie runtime image so the dynamic loader, libc, and libgcc match.

FROM debian:trixie-slim

# CA certificates for any future net-fetching steps; ca-certificates is
# tiny and keeps the image generally useful.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --uid 1000 --create-home --shell /bin/bash builder

COPY --from=builder --chown=builder:builder \
    /src/target/release/rebuild_all /usr/local/bin/rebuild_all

USER builder
WORKDIR /work

# CONFORMANCE_ROOT defaults to /work; honored verbatim if overridden.
ENTRYPOINT ["/usr/local/bin/rebuild_all"]
