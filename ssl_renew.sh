#!/bin/bash

DOCKER="/usr/bin/docker"

cd /root/
$DOCKER compose run certbot renew && $COMPOSE kill -s SIGHUP webserver
$DOCKER system prune -af
