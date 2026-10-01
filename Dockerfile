# Same Debian release as the runtime stage below, so the binary never links
# against a newer glibc than the one it runs on.
FROM docker.io/library/rust:1.98-bookworm AS build

## cargo package name: customize here or provide via --build-arg
ARG pkg=url-shortener

WORKDIR /build

COPY . .

RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    set -eux; \
    cargo build --release --locked; \
    objcopy --compress-debug-sections target/release/$pkg ./main

################################################################################

FROM docker.io/debian:bookworm-slim

## Postgres client library; diesel links against it
RUN apt-get update && \
    apt-get install -y --no-install-recommends libpq5 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=build /build/main ./
COPY --from=build /build/static ./static
COPY --from=build /build/templates ./templates

## Unprivileged port, so the app does not need root to bind it. The uid
## matches runAsUser in sun-gitops.
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8080
EXPOSE 8080
USER 1000:1000

## Exec form: main runs as PID 1 and receives SIGTERM directly
CMD ["./main"]
