FROM docker.io/library/node:14 AS builder
WORKDIR /build

COPY package.json package-lock.json tsconfig.json ./
RUN npm ci

COPY source source
RUN node_modules/.bin/tsc

RUN rm -rf node_modules && npm ci --production


FROM docker.io/bitnami/node:14-prod
WORKDIR /app
VOLUME /app/files
VOLUME /app/tmp

ENV NODE_ENV=production

RUN install_packages ffmpeg

COPY --from=builder /build/node_modules ./node_modules
COPY --from=builder /build/dist ./

HEALTHCHECK --start-period=2h --interval=3m --retries=100 \
    CMD bash -c '[[ $(find . -maxdepth 1 -name ".last-successful-run" -mmin "-300" -print | wc -l) == "1" ]]'

CMD node --unhandled-rejections=strict -r source-map-support/register index.js
