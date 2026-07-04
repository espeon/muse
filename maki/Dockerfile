# Build configuration
ARG project_name=kyoku

# ── Stage 1: Build the frontend (muse-web) ──
FROM node:22-slim AS web-builder
WORKDIR /web
COPY muse-web/package.json muse-web/pnpm-lock.yaml ./
RUN corepack enable && corepack prepare pnpm@9 --activate && pnpm install --frozen-lockfile
COPY muse-web/ ./
RUN pnpm run build

# ── Stage 2: Build the Rust binary (maki) ──
FROM rust:latest AS builder
ARG project_name

# Create layer for the dependencies, so we don't have to rebuild them later
WORKDIR /usr/src
RUN USER=root cargo new $project_name
WORKDIR /usr/src/$project_name
COPY maki/Cargo.toml maki/Cargo.lock ./
RUN cargo build --release
RUN rm src/*.rs

# Copy the pre-built frontend dist
COPY --from=web-builder /web/dist ../muse-web/dist

# Build the actual source
COPY maki/src ./src
COPY maki/.sqlx ./.sqlx
COPY maki/sqlx-data.json ./sqlx-data.json
COPY maki/migrations ./migrations
RUN touch ./src/main.rs && cargo build --release

# ── Stage 3: Runtime ──
FROM debian:stable-slim
ARG project_name
RUN apt-get update \
    && apt-get install -y ca-certificates libssl-dev libc6 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/$project_name/target/release/$project_name ./app

USER 1000
EXPOSE 3033
CMD ["/app"]
