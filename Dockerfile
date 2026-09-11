ARG RUST_VERSION=1.98.1
ARG APP_NAME=cthulhu

# Build stage with alpine
FROM rust:${RUST_VERSION}-alpine AS build

ARG APP_NAME

WORKDIR /app

RUN apk add --no-cache \
    clang \
    lld \
    musl-dev \
    git

RUN rustup target add x86_64-unknown-linux-musl

RUN --mount=type=bind,source=src,target=src,readonly \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml,readonly \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock,readonly \
    --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build \
        --locked \
        --release \
        --target x86_64-unknown-linux-musl \
    && cp "target/x86_64-unknown-linux-musl/release/${APP_NAME}" /bin/server


# Run stage with wolfi
FROM cgr.dev/chainguard/static:latest

WORKDIR /app

COPY --from=build /bin/server /bin/server
COPY conf.yml /app/conf.yml

USER nonroot:nonroot

ENTRYPOINT ["/bin/server"]