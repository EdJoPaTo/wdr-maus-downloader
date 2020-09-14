FROM node:14-alpine AS node-builder
WORKDIR /build

COPY package.json package-lock.json tsconfig.json ./
RUN npm ci

COPY source source
RUN npx tsc

RUN rm -rf node_modules && npm ci --production


FROM node:14-alpine

WORKDIR /app
VOLUME /app/files
VOLUME /app/tmp

ENV NODE_ENV=production

RUN apk add --no-cache bash ffmpeg

COPY --from=node-builder /build/node_modules ./node_modules

COPY --from=node-builder /build/dist ./

HEALTHCHECK --interval=25m \
    CMD bash -c '[[ $(find . -maxdepth 1 -name ".last-successful-run" -mmin "-100" -print | wc -l) == "1" ]]'

CMD node --unhandled-rejections=strict index.js
