FROM node:14-alpine AS node-builder
WORKDIR /build

COPY package.json package-lock.json tsconfig.json ./
RUN npm ci

COPY source source
RUN node_modules/.bin/tsc

RUN rm -rf node_modules && npm ci --production


FROM node:14-alpine

WORKDIR /app
VOLUME /app/files
VOLUME /app/tmp

ENV NODE_ENV=production

RUN apk add --no-cache bash ffmpeg

COPY --from=node-builder /build/node_modules ./node_modules

COPY --from=node-builder /build/dist ./

HEALTHCHECK --start-period=2h --interval=3m --retries=100 \
    CMD bash -c '[[ $(find . -maxdepth 1 -name ".last-successful-run" -mmin "-300" -print | wc -l) == "1" ]]'

CMD node --unhandled-rejections=strict index.js
