FROM node:12-stretch AS node-builder
WORKDIR /build

COPY package.json package-lock.json ./
RUN npm ci --production


FROM resilio/sync AS rslsync


FROM node:12-stretch
WORKDIR /app
VOLUME /app/tmp
VOLUME /var/lib/resilio-sync

ENV NODE_ENV=production

RUN apt-get update && apt-get -y --no-install-recommends install ffmpeg && rm -rf /var/lib/apt/lists/*

COPY --from=rslsync /usr/bin/rslsync /usr/bin/rslsync
COPY --from=node-builder /build/node_modules ./node_modules
COPY source ./

CMD [ "/usr/local/bin/node", "index.js" ]
