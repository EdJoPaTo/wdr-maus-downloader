FROM docker.io/library/node:16-alpine AS builder
WORKDIR /build

COPY package.json package-lock.json tsconfig.json ./
RUN npm ci

COPY source source
RUN node_modules/.bin/tsc


FROM docker.io/library/node:16-alpine AS packages
WORKDIR /build
COPY package.json package-lock.json ./
RUN npm ci --production


# ffmpeg versions
# alpine:3.15           4.4.1
# alpine:edge           4.4.1
# node:16-alpine        4.4.1
# node:16-alpine3.15    4.4.1

FROM docker.io/library/node:16-alpine
ENV NODE_ENV=production
RUN apk --no-cache upgrade \
    && apk --no-cache add ffmpeg \
    && ffmpeg -version \
    && node --version

WORKDIR /app
VOLUME /app/files
VOLUME /app/tmp

COPY package.json ./
COPY --from=packages /build/node_modules ./node_modules
COPY --from=builder /build/dist ./

CMD node --enable-source-maps index.js
