# ffmpeg versions
# alpine:3.13           4.3.1
# alpine:3.14           4.4
# alpine:edge           4.4
# node:14-alpine        4.2.4
# node:14-alpine3.13    4.3.1
# node:14-alpine3.14    4.4
# node:16-alpine        4.3.1
# node:16-alpine3.14    4.4
# deno:alpine           4.3.1

FROM docker.io/denoland/deno:alpine
ENV NODE_ENV=production
RUN apk --no-cache upgrade \
    && apk --no-cache add ffmpeg \
    && ffmpeg -version

WORKDIR /app
VOLUME /app/files
VOLUME /app/tmp

COPY source source
RUN deno cache source/index.ts

CMD /bin/deno run \
    --allow-env \
    --allow-net \
    --allow-read \
    --allow-write \
    source/index.ts
