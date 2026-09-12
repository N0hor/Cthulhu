# Build stage with alpine
FROM rust:1.98.1-alpine@sha256:1716b3aa042d735f4566d14dc54e8037de9d69556e2d5dd58131d93a613d173d AS build

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
    && cp "target/x86_64-unknown-linux-musl/release/cthulhu" /bin/server


# Run stage with wolfi
FROM cgr.dev/chainguard/static@sha256:207a5673ab31ed83332e54ae33d0f1de4adb5984bd93b8309789889e7bf30ba6

WORKDIR /app

COPY --from=build /bin/server /bin/server

USER nonroot:nonroot

ENTRYPOINT ["/bin/server"]