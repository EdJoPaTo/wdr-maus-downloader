#!/usr/bin/env bash
set -e

export TELEGRAM_API_HASH=
export TELEGRAM_API_ID=
export TELEGRAM_LOCAL=true

podman run -p 8081:8081 --env TELEGRAM_API_ID --env TELEGRAM_API_HASH --env TELEGRAM_LOCAL docker.io/tdlight/tdlightbotapi
