#!/bin/bash
set -euo pipefail

DOCKER="/usr/bin/docker"
PROJECT_DIR="/root"

cd "$PROJECT_DIR"
$DOCKER compose run --rm certbot renew
$DOCKER compose kill -s SIGHUP webserver
$DOCKER system prune -af
