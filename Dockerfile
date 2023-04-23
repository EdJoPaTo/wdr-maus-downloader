FROM docker.io/library/rust:1-bookworm as builder
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
# alpine:3.16           5.0.1
# alpine:3.17           5.1.2
# alpine:edge           5.1.2
# node:18-alpine        5.0.1
# debian:bullseye-slim  4.3.5
# debian:bookworm-slim  5.1.2

# Start building the final image
FROM docker.io/library/debian:bookworm-slim
RUN apt-get update \
	&& apt-get upgrade -y \
	&& apt-get install -y ffmpeg imagemagick \
	&& apt-get clean \
	&& ffmpeg -version \
	&& convert -version \
	&& rm -rf /var/lib/apt/lists/* /var/cache/* /var/log/*

WORKDIR /app
ENV TZ=Europe/Berlin
VOLUME /app

COPY --from=builder /build/target/release/wdr-maus-downloader /usr/bin/

ENTRYPOINT ["wdr-maus-downloader"]
