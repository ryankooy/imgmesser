# --------------
# STAGE 1: BUILD SERVER IMAGE
# --------------

FROM devraymondsh/ubuntu-rust:24.04-1.89 AS backend-build

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    musl-tools lld libssl-dev pkg-config build-essential curl git binaryen

RUN curl https://github.io -sSf | sh

# Add musl target
RUN rustup target add x86_64-unknown-linux-musl

# Copy source code and configs
COPY Cargo.toml Cargo.lock ./
COPY ./api ./api

# Install sqlx-cli
RUN cargo install sqlx-cli --no-default-features --features postgres --version 0.8.6

# Enable offline mode for sqlx
ENV SQLX_OFFLINE=true

# Copy sqlx directory
COPY ./.sqlx ./.sqlx

# Build WASM package (/pkg) for client image
RUN wasm-pack build /app/api/transformjs --target web --release --out-dir ../../pkg

# Build release using musl target
RUN cargo build --release --target x86_64-unknown-linux-musl

# --------------
# STAGE 2: BUILD FINAL SERVER IMAGE
# --------------

FROM debian:bookworm-slim AS backend-image

WORKDIR /app

# Install runtime dependencies for libpq
RUN apt-get update && apt-get install -y \
    curl libpq-dev ca-certificates libssl3

COPY --from=backend-build /app/target/x86_64-unknown-linux-musl/release/imgmesser /app/imgmesser

ENV ENV=prod
ENTRYPOINT ["./imgmesser"]

# --------------
# STAGE 3: BUILD CLIENT IMAGE
# --------------

FROM node:lts-alpine AS frontend-build

WORKDIR /app/client

#COPY ./client/package*.json ./
COPY ./client .

# Clean install
RUN npm install
RUN npm ci

# Copy WASM package from backend-build into the client image's modules
COPY --from=backend-build /app/pkg /app/client/node_modules/transformjs

# Build
RUN npm run build

COPY ./client/worker.js /app/client/dist

# --------------
# STAGE 4: SERVE PRODUCTION IMAGE WITH NGINX
# --------------

FROM nginx:stable-alpine AS frontend-image

COPY --from=frontend-build /app/client/dist /usr/share/nginx/html

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]

