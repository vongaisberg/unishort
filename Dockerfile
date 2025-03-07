FROM rustlang/rust:nightly AS build

## cargo package name: customize here or provide via --build-arg
ARG pkg=url-shortener

WORKDIR /build

COPY . .

RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    set -eux; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/$pkg ./main

################################################################################

FROM docker.io/debian:bookworm-slim

## Install postgres library
RUN apt-get update && \
    apt-get install libpq-dev -y && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

## copy the main binary
COPY --from=build /build/main ./

## copy runtime assets which may or may not exist
COPY --from=build /build/Rocket.tom[l] ./static
COPY --from=build /build/stati[c] ./static
COPY --from=build /build/template[s] ./templates

## ensure the container listens globally on port 8080
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=80

CMD ./main
