# Build configuration
ARG project_name=kyoku
# Fill in name of crate^ here

# Set up rust build environment
FROM rust:latest AS builder
ARG project_name

# Create layer for the dependencies, so we don't have to rebuild them later
WORKDIR /usr/src
RUN USER=root cargo new $project_name
WORKDIR /usr/src/$project_name
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
RUN rm src/*.rs

# Build the actual source
COPY src ./src
COPY .sqlx ./.sqlx
COPY sqlx-data.json ./sqlx-data.json
COPY migrations ./migrations
RUN touch ./src/main.rs && cargo build --release

# second stage.
FROM debian:stable
ARG project_name
RUN apt-get update \
    && apt-get install -y ca-certificates libssl-dev libc6 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/$project_name/target/release/$project_name ./app
USER 1000
EXPOSE 3000
CMD ["/app"]