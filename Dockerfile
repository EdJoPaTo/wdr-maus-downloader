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


FROM docker.io/library/fedora
WORKDIR /app
VOLUME /app/files
VOLUME /app/tmp

ENV NODE_ENV=production

RUN dnf install -y https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-34.noarch.rpm \
    && dnf install -y ffmpeg nodejs \
    && dnf clean all  \
    && rm -rf /var/cache/yum  \
    && rm -rf /var/lib/yum/* /var/log/yum.log

COPY package.json ./
COPY --from=packages /build/node_modules ./node_modules
COPY --from=builder /build/dist ./

CMD node --unhandled-rejections=strict -r source-map-support/register index.js
