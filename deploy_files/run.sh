#!/bin/bash

if [ "${DEPLOY}" = 'app' ] || [ "${DEPLOY}" = 'all' ]; then
    if [ "$(docker container inspect -f '{{.State.Running}}' ${CLIENT_CONTAINER})" = 'true' ]; then
        echo 'Stopping and removing the existing client container...'
        docker stop "${CLIENT_CONTAINER}"
        docker rm "${CLIENT_CONTAINER}"
    fi

    echo 'Pulling the latest client image...'
    docker pull "${DOCKER_USER}"/${CLIENT_REPO}:latest

    echo 'Running the certbot service and creating dummy certificates...'
    docker compose run --rm --entrypoint " \
        mkdir -p /etc/letsencrypt/live/${DOMAIN}; \
        openssl req -x509 -nodes -newkey rsa:4096 -days 1 \
            -keyout /etc/letsencrypt/live/${DOMAIN}/privkey.pem \
            -out /etc/letsencrypt/live/${DOMAIN}/fullchain.pem -subj '/CN=localhost'" \
        certbot

    echo 'Running the client service (and nginx) in a container...'
    docker compose --profile nginx up --detach

    echo 'Running certbot again separately to create real certificates...'
    docker compose run --rm --entrypoint \
        "certbot certonly --webroot -w /var/www/certbot -d ${DOMAIN} --email ${EMAIL} --agree-tos --no-eff-email --force-renewal" \
        certbot

    echo 'Reloading nginx in the running client container...'
    docker compose exec app nginx -s reload
fi

if [ "${DEPLOY}" = 'api' ] || [ "${DEPLOY}" = 'all' ]; then
    if [ "$(docker container inspect -f '{{.State.Running}}' ${API_CONTAINER})" = 'true' ]; then
        echo 'Stopping and removing the existing API container...'
        docker stop ${API_CONTAINER}
        docker rm ${API_CONTAINER}
    fi

    echo 'Pulling the latest API image...'
    docker pull ${DOCKER_USER}/${API_REPO}:latest

    if [ "${DEPLOY}" = 'api' ]; then
        echo 'Running the API service in a container...'
        POSTGRES_USER=${POSTGRES_USER} \
            POSTGRESS_PASSWORD=${POSTGRES_PASSWORD} \
            POSTGRES_DB=${POSTGRES_DB} \
            docker compose up --detach api
    fi
fi

if [ "${DEPLOY}" = 'all' ]; then
    echo 'Running all services...'
    POSTGRES_USER=${POSTGRES_USER} \
        POSTGRESS_PASSWORD=${POSTGRES_PASSWORD} \
        POSTGRES_DB=${POSTGRES_DB} \
        docker compose up --detach
fi
