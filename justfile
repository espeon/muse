# muse justfile

# list available recipes
default:
    @just --list

# --- maki (rust music server) ---

# build maki
maki-build:
    cd maki && cargo build

# run maki (requires .env or env vars set)
maki:
    cd maki && cargo run

# run maki without scanning
maki-serve:
    cd maki && NO_SCAN=1 cargo run

# dry-run scan (no db writes, no server)
maki-dry-run:
    cd maki && DRY_RUN=1 cargo run

# check maki (offline sqlx)
maki-check:
    cd maki && SQLX_OFFLINE=true cargo check

# update sqlx query cache (requires live db)
maki-sqlx-prepare:
    cd maki && cargo sqlx prepare

# run maki db migrations
maki-migrate:
    cd maki && cargo sqlx migrate run

# --- nozomi (next.js frontend) ---

# install nozomi deps
nozomi-install:
    cd nozomi && bun install

# run nozomi dev server (port 3031)
nozomi-dev:
    cd nozomi && bun dev

# build nozomi
nozomi-build:
    cd nozomi && bun build

# lint nozomi
nozomi-lint:
    cd nozomi && bun lint

# --- docker ---

# start full stack (dev)
up:
    docker compose up

# start full stack detached
up-d:
    docker compose up -d

# stop full stack
down:
    docker compose down

# start production stack
up-prod:
    docker compose -f docker-compose.prod.yaml up -d

# --- build & push images ---

# build and push all images as latest
push:
    ./build.sh all

# build and push all images with a tag
push-tag tag:
    ./build.sh all {{tag}}

# build and push a specific service (e.g. just push-service maki)
push-service service:
    ./build.sh {{service}}
