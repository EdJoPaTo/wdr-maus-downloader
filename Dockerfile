FROM node:12-stretch
WORKDIR /app
VOLUME /app/tmp
VOLUME /var/lib/resilio-sync

ENV NODE_ENV=production

RUN wget -qO - https://linux-packages.resilio.com/resilio-sync/key.asc | apt-key add - && \
  echo "deb http://linux-packages.resilio.com/resilio-sync/deb resilio-sync non-free" > /etc/apt/sources.list.d/resilio-sync.list

RUN apt-get update && apt-get -y --no-install-recommends install ffmpeg resilio-sync && rm -rf /var/lib/apt/lists/*

COPY package.json package-lock.json ./
RUN npm ci

COPY . ./
CMD ./docker-run.sh
