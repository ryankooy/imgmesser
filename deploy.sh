#!/bin/bash

# Purpose: Deploy containers to the EC2 instance

[ ! -f .env.deploy ] && { echo '.env.deploy not found' >&2; exit 1; }
source ./.env.deploy

API_REPO="${PROJECT}-${API_SERVICE}"
API_CONTAINER="${API_REPO}-1"
CLIENT_REPO="${PROJECT}-${CLIENT_SERVICE}"
CLIENT_CONTAINER="${CLIENT_REPO}-1"

function usage() {
    cat <<'MSG'
Usage: deploy.sh [-h] [-c] [-s api|app]
    -s <service>    Docker service name; optional, and if not
                    included, all services are deployed
    -c              Copy config files to EC2 instance (optional)
    -h              Show options and exit
MSG
    [ "$1" = 1 ] && exit 1 || exit 0
}

function die() {
    [ "$1" -ne 0 ] && { printf "ERROR: %s\n" "$2" >&2; exit 1; }
}

copy_cfgs=false
deploy_api=false
deploy_client=false
deploy_all=false

while getopts "s:ch" opt; do
    case "${opt}" in
        s) svc="${OPTARG}";;
        c) copy_cfgs=true;;
        h) usage;;
        \?) usage 1;;
    esac
    [[ "${OPTARG}" = -* ]] && usage 1
done

if [ -z "${svc}" ]; then
    deploy_all=true
elif [ "${svc}" = "${CLIENT_SERVICE}" ]; then
    deploy_client=true
elif [ "${svc}" = "${API_SERVICE}" ]; then
    deploy_api=true
else
    die 1 'Unknown service specified'
fi

# Build image(s) using `compose.yaml`
if [ "${deploy_all}" = true ]; then
    echo 'Building images'
    docker compose build
else
    echo 'Building image'
    docker compose build "${svc}"
fi
die "$?" 'Failed to build image(s)'

# Tag and push the API image
if [ "${deploy_api}" = true ] || [ "${deploy_all}" = true ]; then
    echo 'Tagging and pushing API image'
    docker tag "${API_REPO}":latest "${DOCKER_USER}"/"${API_REPO}":latest
    docker push "${DOCKER_USER}"/"${API_REPO}":latest
fi

# Tag and push the client app image
if [ "${deploy_client}" = true ] || [ "${deploy_all}" = true ]; then
    echo 'Tagging and pushing client app image'
    docker tag "${CLIENT_REPO}":latest "${DOCKER_USER}"/"${CLIENT_REPO}":latest
    docker push "${DOCKER_USER}"/"${CLIENT_REPO}":latest
fi

# Copy config files to the EC2 instance
if [ "${copy_cfgs}" = true ]; then
    echo 'Copying files to server'
    scp .env.prod compose.run.yaml custom_nginx.conf "${EC2_INSTANCE_ALIAS}":~
fi

# On the EC2 Instance, remove old containers, pull fresh
# images, and run new containers
echo 'Deploying to server'
if [ "${deploy_all}" = true ]; then
    ssh "${EC2_INSTANCE_ALIAS}" " \
        docker stop ${CLIENT_CONTAINER}; \
        docker stop ${API_CONTAINER}; \
        docker rm ${CLIENT_CONTAINER}; \
        docker rm ${API_CONTAINER}; \
        docker pull ${DOCKER_USER}/${CLIENT_REPO}:latest; \
        docker pull ${DOCKER_USER}/${API_REPO}:latest; \
        source .env.prod && \
        docker compose --file compose.run.yaml up --detach; \
        rm .env.prod"
elif [ "${deploy_api}" = true ]; then
    ssh "${EC2_INSTANCE_ALIAS}" " \
        docker stop ${API_CONTAINER}; \
        docker rm ${API_CONTAINER}; \
        docker pull "${DOCKER_USER}"/${API_REPO}:latest && \
        source .env.prod && \
        docker compose --file compose.run.yaml up --detach ${API_SERVICE}; \
        rm .env.prod"
elif [ "${deploy_client}" = true ]; then
    ssh "${EC2_INSTANCE_ALIAS}" " \
        docker stop ${CLIENT_CONTAINER}; \
        docker rm ${CLIENT_CONTAINER}; \
        docker pull "${DOCKER_USER}"/${CLIENT_REPO}:latest && \
        source .env.prod && \
        docker compose --file compose.run.yaml up --detach ${CLIENT_SERVICE}; \
        rm .env.prod"
fi
