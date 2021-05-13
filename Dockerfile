FROM docker.io/library/node:14-alpine AS builder
WORKDIR /build

COPY package.json package-lock.json tsconfig.json ./
RUN npm ci

COPY source source
RUN node_modules/.bin/tsc


FROM docker.io/library/node:14-alpine AS packages
WORKDIR /build
COPY package.json package-lock.json ./
RUN npm ci --production


# ffmpeg versions
# alpine:3.13           4.3.1
# alpine:edge           4.4
# node:14-alpine        4.2.4
# node:14-alpine3.13    4.3.1
# node:16-alpine        4.3.1

FROM docker.io/library/alpine:edge
WORKDIR /app
VOLUME /app/files
VOLUME /app/tmp

ENV NODE_ENV=production

RUN apk --no-cache upgrade \
    && apk --no-cache add ffmpeg nodejs \
    && ffmpeg -version

COPY package.json ./
COPY --from=packages /build/node_modules ./node_modules
COPY --from=builder /build/dist ./

CMD node --unhandled-rejections=strict -r source-map-support/register index.js
