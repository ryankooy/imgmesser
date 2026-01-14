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

if [ "${deploy}" = 'app' ] || [ "${deploy}" = 'all' ]; then
    [ -z "${CLIENT_CONTAINER}" ] && { echo 'CLIENT_CONTAINER variable not set' >&2; exit 1; }
    [ -z "${CLIENT_REPO}" ] && { echo 'CLIENT_REPO variable not set' >&2; exit 1; }
    [ -z "${DOMAIN}" ] && { echo 'DOMAIN variable not set' >&2; exit 1; }
    [ -z "${EMAIL}" ] && { echo 'EMAIL variable not set' >&2; exit 1; }

    if [ "$(docker container inspect -f '{{.State.Running}}' ${CLIENT_CONTAINER})" = 'true' ]; then
        echo 'Stopping and removing the existing client container...'
        docker stop "${CLIENT_CONTAINER}"
        docker rm "${CLIENT_CONTAINER}"
    fi

    echo 'Pulling the latest client image...'
    docker pull "${DOCKER_USER}"/"${CLIENT_REPO}":latest

    echo 'Running the certbot service and creating dummy certificates...'
    docker compose run --rm --entrypoint " \
        mkdir -p /etc/letsencrypt/live/${DOMAIN}; \
        openssl req -x509 -nodes -newkey rsa:4096 -days 1 \
            -keyout /etc/letsencrypt/live/${DOMAIN}/privkey.pem \
            -out /etc/letsencrypt/live/${DOMAIN}/fullchain.pem -subj '/CN=localhost'" \
        certbot

    echo 'Running the client service (and nginx) in a new container...'
    docker compose --profile nginx up --detach

    echo 'Running certbot again separately to create real certificates...'
    docker compose run --rm --entrypoint " \
        certbot certonly --webroot -w /var/www/certbot \
            -d ${DOMAIN} --email ${EMAIL} --agree-tos --no-eff-email --force-renewal" \
        certbot

    echo 'Reloading nginx in the running client container...'
    docker compose exec app nginx -s reload
fi

if [ "${deploy}" = 'api' ] || [ "${deploy}" = 'all' ]; then
    [ -z "${API_CONTAINER}" ] && { echo 'API_CONTAINER variable not set' >&2; exit 1; }
    [ -z "${API_DOCKER_IP}" ] && { echo 'API_DOCKER_IP variable not set' >&2; exit 1; }
    [ -z "${API_REPO}" ] && { echo 'API_REPO variable not set' >&2; exit 1; }
    [ -z "${POSTGRES_USER}" ] && { echo 'POSTGRES_USER variable not set' >&2; exit 1; }
    [ -z "${POSTGRES_PASSWORD}" ] && { echo 'POSTGRES_PASSWORD variable not set' >&2; exit 1; }
    [ -z "${POSTGRES_DB}" ] && { echo 'POSTGRES_DB variable not set' >&2; exit 1; }

    if [ "$(docker container inspect -f '{{.State.Running}}' ${API_CONTAINER})" = 'true' ]; then
        echo 'Stopping and removing the existing API container...'
        docker stop "${API_CONTAINER}"
        docker rm "${API_CONTAINER}"
    fi

    echo 'Pulling the latest API image...'
    docker pull "${DOCKER_USER}"/"${API_REPO}":latest

    export API_DOCKER_IP="${API_DOCKER_IP}"
    export POSTGRES_USER="${POSTGRES_USER}"
    export POSTGRES_PASSWORD="${POSTGRES_PASSWORD}"
    export POSTGRES_DB="${POSTGRES_DB}"

    if [ "${deploy}" = 'api' ]; then
        echo 'Running the API service in a new container...'
        docker compose up --detach api
    elif [ "${deploy}" = 'all' ]; then
        echo 'Running all services...'
        docker compose up --detach
    fi
fi
