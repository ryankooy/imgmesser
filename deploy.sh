#!/bin/bash

# Purpose: Builds and deploys images to the EC2 instance

[ ! -f .env.deploy ] && { echo '.env.deploy not found' >&2; exit 1; }
source ./.env.deploy

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

api_repo="${PROJECT}-api"
api_container="${api_repo}-1"
client_repo="${PROJECT}-app"
client_container="${client_repo}-1"
copy_cfgs=false
deploy_api=false
deploy_client=false
deploy_all=false
svc=all

while getopts "s:ch" opt; do
    case "${opt}" in
        s) svc="${OPTARG}";;
        c) copy_cfgs=true;;
        h) usage;;
        \?) usage 1;;
    esac
    [[ "${OPTARG}" = -* ]] && usage 1
done

case "${svc}" in
    all) deploy_all=true;;
    app) deploy_client=true;;
    api) deploy_api=true;;
    *) die 1 'Unknown service specified';;
esac

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
    docker tag "${api_repo}":latest "${DOCKER_USER}"/"${api_repo}":latest
    docker push "${DOCKER_USER}"/"${api_repo}":latest
fi

# Tag and push the client app image
if [ "${deploy_client}" = true ] || [ "${deploy_all}" = true ]; then
    echo 'Tagging and pushing client app image'
    docker tag "${client_repo}":latest "${DOCKER_USER}"/"${client_repo}":latest
    docker push "${DOCKER_USER}"/"${client_repo}":latest
fi

# Copy config files to the EC2 instance
if [ "${copy_cfgs}" = true ]; then
    echo 'Copying files to server'
    scp -r deploy_files/* "${EC2_INSTANCE_ALIAS}":~
    ssh "${EC2_INSTANCE_ALIAS}" chmod 700 run.sh
fi

# On the EC2 Instance, remove old containers, pull fresh
# images, and run new containers
echo 'Deploying to server'
ssh "${EC2_INSTANCE_ALIAS}" " \
    API_CONTAINER=${api_container} API_DOCKER_IP=${API_DOCKER_IP} API_REPO=${api_repo} \
        CLIENT_CONTAINER=${client_container} CLIENT_REPO=${client_repo} \
        DOCKER_USER=${DOCKER_USER} DOMAIN=${DOMAIN} EMAIL=${EMAIL} \
        POSTGRES_USER=${POSTGRES_USER} POSTGRES_PASSWORD=${POSTGRES_PASSWORD} \
        POSTGRES_DB=${POSTGRES_DB} PROJECT=${PROJECT} \
        ./run.sh ${svc} > last_run.log"
