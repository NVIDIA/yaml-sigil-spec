# SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
# SPDX-License-Identifier: Apache-2.0

# Conformance-fixture rebuild container (Rust).
#
# Build from the repository root so the image can include the repository
# license and complete third-party notice record:
#     docker build -f conformance/rebuild-rs/Dockerfile \
#         -t yamlsigil-conformance-rebuild-rs .
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

FROM docker.io/library/rust:1.95.0-trixie AS builder

# Non-root build user; owns /src so cargo can write target/.
RUN useradd --uid 1000 --create-home --shell /bin/bash builder \
    && install -d -o builder -g builder /src

WORKDIR /src
COPY --chown=builder:builder \
    conformance/rebuild-rs/Cargo.toml \
    conformance/rebuild-rs/Cargo.lock \
    /src/
COPY --chown=builder:builder \
    conformance/rebuild-rs/pinned-dir /src/pinned-dir
COPY --chown=builder:builder conformance/rebuild-rs/xtask /src/xtask
COPY --chown=builder:builder conformance/rebuild-rs/vendor /src/vendor
COPY --chown=builder:builder conformance/rebuild-rs/src /src/src

USER builder

# --locked aborts if Cargo.lock would be regenerated, which is what
# we want for reproducibility. `-p` builds just the rebuild_all bin
# (skips the xtask dev tool).
RUN cargo build --release --locked -p yamlsigil-conformance-rebuild

# Collect the license, copyright, and notice files supplied by every registry
# crate Cargo downloaded for the locked build. This uses the upstream files
# verbatim and introduces no notice-generation dependency.
RUN set -eu; \
    notice_root=/src/notices/cargo; \
    install -d "${notice_root}"; \
    for crate_dir in /usr/local/cargo/registry/src/*/*; do \
        test -f "${crate_dir}/Cargo.toml" || continue; \
        crate_name="${crate_dir##*/}"; \
        crate_notice_dir="${notice_root}/${crate_name}"; \
        install -d "${crate_notice_dir}"; \
        copied=0; \
        for notice in \
            "${crate_dir}"/LICENSE* \
            "${crate_dir}"/COPYRIGHT* \
            "${crate_dir}"/NOTICE*; do \
            test -f "${notice}" || continue; \
            cp "${notice}" "${crate_notice_dir}/"; \
            copied=1; \
        done; \
        test "${copied}" -eq 1 || rmdir "${crate_notice_dir}"; \
    done

# The final binary statically links the Rust standard library. Carry the
# exact library notice bundle supplied by the pinned Rust toolchain into
# the runtime image.
RUN install -d /src/notices/rust \
    && cp \
        "$(rustc --print sysroot)/share/doc/rust/COPYRIGHT-library.html" \
        /src/notices/rust/

# --- Runtime stage --------------------------------------------------
# The release binary is statically-enough linked for Debian Trixie's
# libc (Rust links libstd statically by default). Use a minimal
# Trixie runtime image so the dynamic loader, libc, and libgcc match.

FROM docker.io/library/debian:trixie-slim

# CA certificates for any future net-fetching steps; ca-certificates is
# tiny and keeps the image generally useful.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --uid 1000 --create-home --shell /bin/bash builder

COPY --from=builder --chown=builder:builder \
    /src/target/release/rebuild_all /usr/local/bin/rebuild_all
COPY LICENSE THIRD_PARTY_NOTICES.md \
    /usr/share/doc/yamlsigil-conformance-rebuild/
COPY --from=builder /src/notices/cargo \
    /usr/share/doc/yamlsigil-conformance-rebuild/cargo/
COPY --from=builder /src/notices/rust/COPYRIGHT-library.html \
    /usr/share/doc/yamlsigil-conformance-rebuild/rust/

USER builder
WORKDIR /work

# CONFORMANCE_ROOT defaults to /work; honored verbatim if overridden.
ENTRYPOINT ["/usr/local/bin/rebuild_all"]
