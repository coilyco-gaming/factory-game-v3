# syntax=docker/dockerfile:1.7
# -----------------------------------------------------------------------------
# Stage 1: build the factory_shell Bevy app -> Wasm static bundle via trunk.
# -----------------------------------------------------------------------------
FROM rust:1.97-bookworm AS builder

ARG TRUNK_VERSION=0.21.14
ARG TARGETARCH

# trunk from the upstream prebuilt release (matches the aos dev-base pin), and
# the wasm target for the shell build. No binaryen/wasm-opt in the shell yet -
# index.html pins data-wasm-opt="0", so nothing here can trip the Debian
# browser-side binaryen instantiation failure.
RUN case "${TARGETARCH}" in \
      amd64) trunk_arch=x86_64 ;; \
      arm64) trunk_arch=aarch64 ;; \
      *) echo "unsupported target architecture: ${TARGETARCH}" >&2; exit 1 ;; \
    esac \
 && rustup target add wasm32-unknown-unknown \
 && curl -fsSL "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-${trunk_arch}-unknown-linux-gnu.tar.gz" \
      -o /tmp/trunk.tar.gz \
 && tar -xzf /tmp/trunk.tar.gz -C /usr/local/bin trunk \
 && chmod 0755 /usr/local/bin/trunk \
 && rm /tmp/trunk.tar.gz \
 && trunk --version

WORKDIR /app

# Cache the Bevy dependency graph: manifests plus a shim main first.
COPY Cargo.toml Cargo.lock ./
COPY crates/factory_content/Cargo.toml crates/factory_content/Cargo.toml
COPY crates/factory_sim/Cargo.toml crates/factory_sim/Cargo.toml
COPY crates/factory_cli/Cargo.toml crates/factory_cli/Cargo.toml
COPY crates/factory_shell/Cargo.toml crates/factory_shell/Cargo.toml
RUN set -eux; \
    for crate in factory_content factory_sim; do \
      mkdir -p "crates/${crate}/src"; echo "" > "crates/${crate}/src/lib.rs"; \
    done; \
    for crate in factory_cli factory_shell; do \
      mkdir -p "crates/${crate}/src"; echo "fn main() {}" > "crates/${crate}/src/main.rs"; \
    done; \
    cargo fetch

# Real sources, then the trunk build (release profile via Trunk.toml). Trunk
# must run from the crate dir, not the workspace root.
COPY crates ./crates
RUN touch crates/*/src/*.rs \
 && cd crates/factory_shell \
 && trunk build

# -----------------------------------------------------------------------------
# Stage 2: unprivileged nginx serving the built bundle.
# -----------------------------------------------------------------------------
# Self-contained serving image on the coilyco-bridge/deploy static-site
# precedent: nginx-unprivileged, uid 101, listens on 8080,
# TLS terminated upstream by traefik + cert-manager. Source CI publishes the
# git-sha image. The deploy repo owns rollout.
FROM nginxinc/nginx-unprivileged:1.27-alpine AS runtime

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/crates/factory_shell/dist /usr/share/nginx/html
