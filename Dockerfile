FROM node:14-buster AS node-builder
WORKDIR /build

COPY package.json package-lock.json tsconfig.json ./
RUN npm ci

COPY source source
RUN npx tsc

RUN rm -rf node_modules && npm ci --production


FROM node:14-buster

WORKDIR /app
VOLUME /app/files
VOLUME /app/tmp

ENV NODE_ENV=production

RUN apt-get update && apt-get -y --no-install-recommends install ffmpeg && rm -rf /var/lib/apt/lists/*

COPY --from=node-builder /build/node_modules ./node_modules

COPY --from=node-builder /build/dist ./

CMD node --unhandled-rejections=strict index.js
