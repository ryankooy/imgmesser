#!/bin/bash

# Purpose: Pulls fresh images and replaces running containers
# Note: Environment variable assignments are required before running this script

if [ -z "$1" ]; then
    deploy=all
else
    deploy="$1"
fi

[ -z "${DOCKER_USER}" ] && { echo 'DOCKER_USER variable not set' >&2; exit 1; }
[ -z "${PROJECT}" ] && { echo 'DOCKER_USER variable not set' >&2; exit 1; }
export DOCKER_USER="${DOCKER_USER}" PROJECT="${PROJECT}"

if docker network inspect "${PROJECT}"-nw >/dev/null 2>&1; then
    echo 'Network exists'
else
    echo 'Creating network...'
    docker network create --driver bridge \
        --subnet 172.20.0.0/16 --gateway 172.20.0.1 \
        "${PROJECT}"-nw
fi

if [ "${deploy}" = 'app' ] || [ "${deploy}" = 'all' ]; then
    [ -z "${FRONTEND_REPO}" ] && { echo 'FRONTEND_REPO variable not set' >&2; exit 1; }
    [ -z "${DOMAIN}" ] && { echo 'DOMAIN variable not set' >&2; exit 1; }
    [ -z "${EMAIL}" ] && { echo 'EMAIL variable not set' >&2; exit 1; }

    echo 'Pulling the latest frontend image...'
    docker compose pull app

    echo 'Running the certbot service and creating dummy certificates...'
    docker compose run --rm --entrypoint " \
        mkdir -p /etc/letsencrypt/live/${DOMAIN}; \
        openssl req -x509 -nodes -newkey rsa:4096 -days 1 \
            -keyout /etc/letsencrypt/live/${DOMAIN}/privkey.pem \
            -out /etc/letsencrypt/live/${DOMAIN}/fullchain.pem -subj '/CN=localhost'" \
        certbot

    echo 'Running the frontend service (and nginx) in a new container...'
    docker compose --profile nginx up --detach

    echo 'Running certbot again separately to create real certificates...'
    docker compose run --rm --entrypoint " \
        certbot certonly --webroot -w /var/www/certbot \
            -d ${DOMAIN} --email ${EMAIL} --agree-tos --no-eff-email --force-renewal" \
        certbot

    echo 'Reloading nginx in the running frontend container...'
    docker compose exec app nginx -s reload
fi

if [ "${deploy}" = 'api' ] || [ "${deploy}" = 'all' ]; then
    [ -z "${API_DOCKER_IP}" ] && { echo 'API_DOCKER_IP variable not set' >&2; exit 1; }
    [ -z "${BACKEND_REPO}" ] && { echo 'BACKEND_REPO variable not set' >&2; exit 1; }
    [ -z "${POSTGRES_USER}" ] && { echo 'POSTGRES_USER variable not set' >&2; exit 1; }
    [ -z "${POSTGRES_PASSWORD}" ] && { echo 'POSTGRES_PASSWORD variable not set' >&2; exit 1; }
    [ -z "${POSTGRES_DB}" ] && { echo 'POSTGRES_DB variable not set' >&2; exit 1; }

    echo 'Pulling the latest backend image...'
    docker compose pull api

    export API_DOCKER_IP="${API_DOCKER_IP}"
    export POSTGRES_USER="${POSTGRES_USER}"
    export POSTGRES_PASSWORD="${POSTGRES_PASSWORD}"
    export POSTGRES_DB="${POSTGRES_DB}"

    if [ "${deploy}" = 'api' ]; then
        echo 'Running the backend service in a new container...'
        docker compose up --detach api
    elif [ "${deploy}" = 'all' ]; then
        echo 'Running all services...'
        docker compose up --detach

        # Prune stale image layers
        docker image prune -f
    fi
fi
