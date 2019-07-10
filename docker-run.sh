#!/usr/bin/env bash

node set-resilio-share.js && \
rslsync --config resilio-config.json && \
node index.js
