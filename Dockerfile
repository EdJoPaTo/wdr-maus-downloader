FROM node:12-buster AS node-builder
WORKDIR /build

COPY package.json package-lock.json tsconfig.json ./
RUN npm ci

COPY source source
RUN npx tsc

RUN rm -rf node_modules && npm ci --production


FROM resilio/sync AS rslsync


FROM node:12-buster
WORKDIR /app
VOLUME /app/tmp
VOLUME /var/lib/resilio-sync

ENV NODE_ENV=production

RUN apt-get update && apt-get -y --no-install-recommends install ffmpeg && rm -rf /var/lib/apt/lists/*

COPY --from=rslsync /usr/bin/rslsync /usr/bin/rslsync
COPY --from=node-builder /build/node_modules ./node_modules

COPY resilio-config.json ./
COPY --from=node-builder /build/dist ./

CMD node index.js
