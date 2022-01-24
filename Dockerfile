FROM docker.io/library/rust:1-bullseye as builder
WORKDIR /build
RUN apt-get update \
    && apt-get upgrade -y \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# cargo needs a dummy src/main.rs to detect bin mode
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

COPY Cargo.toml Cargo.lock ./
RUN cargo fetch --locked
RUN cargo build --release --frozen --offline

# We need to touch our real main.rs file or the cached one will be used.
COPY . ./
RUN touch src/main.rs

RUN cargo build --release --frozen --offline


# ffmpeg versions
# alpine:3.15           4.4.1
# alpine:edge           4.4.1
# node:16-alpine        4.4.1
# node:16-alpine3.15    4.4.1
# debian:bullseye-slim  4.3.3
# debian:bookworm-slim  4.4.1

# Start building the final image
FROM docker.io/library/debian:bookworm-slim
RUN apt-get update \
    && apt-get upgrade -y \
    && apt-get install -y ffmpeg \
    && apt-get clean \
    && ffmpeg -version \
    && rm -rf /var/lib/apt/lists/* /var/cache/* /var/log/*

WORKDIR /app
VOLUME /app

COPY --from=builder /build/target/release/wdr-maus-downloader /usr/bin/

ENTRYPOINT ["wdr-maus-downloader"]
