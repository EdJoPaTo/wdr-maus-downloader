FROM resilio/sync AS rslsync


FROM node:12-stretch
WORKDIR /app
VOLUME /app/tmp
VOLUME /var/lib/resilio-sync

ENV NODE_ENV=production

RUN apt-get update && apt-get -y --no-install-recommends install ffmpeg && rm -rf /var/lib/apt/lists/*

COPY package.json package-lock.json ./
RUN npm ci

COPY --from=rslsync /usr/bin/rslsync /usr/bin/rslsync

COPY . ./
CMD ./docker-run.sh
