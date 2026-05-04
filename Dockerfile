# Stage 1: Build the Rust binary
FROM rust:1.94.1-bookworm AS rust-builder

WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml rustfmt.toml ./
COPY crates/ crates/
COPY xtask/ xtask/

RUN cargo build --release --locked --bin know-now

# Stage 2: Build the dashboard
FROM node:22-bookworm-slim AS web-builder

RUN corepack enable && corepack prepare pnpm@10.33.2 --activate

WORKDIR /build/web
COPY web/package.json web/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY web/ .
RUN pnpm build

# Stage 3: Runtime
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 1000 knownow \
    && useradd --uid 1000 --gid knownow --shell /bin/bash --create-home knownow

COPY --from=rust-builder /build/target/release/know-now /usr/local/bin/know-now
COPY --from=web-builder /build/web/dist /opt/know-now/dashboard

WORKDIR /workspace
RUN mkdir -p /workspace/.knownow && chown -R knownow:knownow /workspace

USER knownow

EXPOSE 3827

HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -sf http://127.0.0.1:3827/__health || exit 1

ENTRYPOINT ["know-now"]
CMD ["serve", "--host", "127.0.0.1", "--port", "3827"]
