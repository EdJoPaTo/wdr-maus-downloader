FROM docker.io/library/rust:1-alpine AS builder
WORKDIR /build
RUN apk upgrade --no-cache \
	&& apk add --no-cache musl-dev

COPY Cargo.toml Cargo.lock ./

# cargo needs a dummy src/lib.rs to compile the dependencies
RUN mkdir -p src \
	&& touch src/lib.rs \
	&& cargo build --release --locked \
	&& rm -rf src

COPY . ./
RUN cargo build --release --locked --offline


FROM docker.io/library/alpine:3 AS final
RUN apk upgrade --no-cache \
	&& apk add --no-cache ffmpeg imagemagick \
	&& ffmpeg -version \
	&& magick -version

WORKDIR /app
ENV TZ=Europe/Berlin
VOLUME /app

COPY --from=builder /build/target/release/wdr-maus-downloader /usr/local/bin/
ENTRYPOINT ["wdr-maus-downloader"]
