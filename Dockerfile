FROM docker.io/library/rust:1-bookworm AS builder
WORKDIR /build
RUN apt-get update \
	&& apt-get upgrade -y \
	&& apt-get clean \
	&& rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./

# cargo needs a dummy src/lib.rs to compile the dependencies
RUN mkdir -p src \
	&& touch src/lib.rs \
	&& cargo fetch --locked \
	&& cargo build --release --offline \
	&& rm -rf src

COPY . ./
RUN cargo build --release --frozen --offline


# ffmpeg versions
# alpine:3.17           5.1.3
# alpine:3.18           6.0
# alpine:edge           6.0
# debian:bookworm-slim  5.1.3
# debian:trixie-slim    6.0
# debian:sid-slim       6.0

# Start building the final image
FROM docker.io/library/debian:bookworm-slim AS final
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

COPY --from=builder /build/target/release/wdr-maus-downloader /usr/local/bin/
ENTRYPOINT ["wdr-maus-downloader"]
