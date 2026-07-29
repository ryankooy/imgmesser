#!/bin/bash

# Purpose: Builds and deploys images to the EC2 instance

[ ! -f .env.deploy ] && { echo '.env.deploy not found' >&2; exit 1; }
source ./.env.deploy

function usage() {
    cat <<'MSG'
Usage: manual_deploy.sh [-h] [-c] [-d] [-s api|app]
    -s <service>    Docker service name; if not included,
                    all services are deployed
    -c              Copy config files to EC2 instance
    -d              Deploy services without building or pushing images
    -h              Show options and exit
MSG
    [ "$1" = 1 ] && exit 1 || exit 0
}

function die() {
    [ "$1" -ne 0 ] && { printf "ERROR: %s\n" "$2" >&2; exit 1; }
}

backend_repo="${PROJECT}-api"
frontend_repo="${PROJECT}-app"
copy_cfgs=false
deploy_backend=false
deploy_frontend=false
deploy_all=false
no_build=false
svc=all

while getopts "s:cdh" opt; do
    case "${opt}" in
        s) svc="${OPTARG}";;
        c) copy_cfgs=true;;
        d) no_build=true;;
        h) usage;;
        \?) usage 1;;
    esac
    [[ "${OPTARG}" = -* ]] && usage 1
done

if [ "${no_build}" = false ]; then
    case "${svc}" in
        all) deploy_all=true;;
        app) deploy_frontend=true;;
        api) deploy_backend=true;;
        *) die 1 'Unknown service specified';;
    esac

    echo 'Building images...'

    # Build, tag, and push the backend image
    if [ "${deploy_backend}" = true ] || [ "${deploy_all}" = true ]; then
        docker build --target backend-image -t "${backend_repo}" .
        die "$?" 'Failed to build backend image'

        echo 'Tagging and pushing backend image...'
        docker tag "${backend_repo}":latest "${DOCKER_USER}"/"${backend_repo}":latest
        docker push "${DOCKER_USER}"/"${backend_repo}":latest
    fi

    # Build, tag, and push the frontend image
    if [ "${deploy_frontend}" = true ] || [ "${deploy_all}" = true ]; then
        docker build --target frontend-image -t "${frontend_repo}" .
        die "$?" 'Failed to build frontend image'

        echo 'Tagging and pushing frontend app image...'
        docker tag "${frontend_repo}":latest "${DOCKER_USER}"/"${frontend_repo}":latest
        docker push "${DOCKER_USER}"/"${frontend_repo}":latest
    fi
fi

# Copy config files to the EC2 instance
if [ "${copy_cfgs}" = true ]; then
    echo 'Copying files to server...'
    scp -r deploy_files/* "${EC2_INSTANCE_ALIAS}":~
    ssh "${EC2_INSTANCE_ALIAS}" chmod 700 run.sh
fi

# On the EC2 Instance, pull fresh images and run new containers
echo 'Deploying to server...'
ssh "${EC2_INSTANCE_ALIAS}" " \
    API_DOCKER_IP=${API_DOCKER_IP} BACKEND_REPO=${backend_repo} FRONTEND_REPO=${frontend_repo} \
        DOCKER_USER=${DOCKER_USER} DOMAIN=${DOMAIN} EMAIL=${EMAIL} \
        POSTGRES_USER=${POSTGRES_USER} POSTGRES_PASSWORD=${POSTGRES_PASSWORD} \
        POSTGRES_DB=${POSTGRES_DB} PROJECT=${PROJECT} \
        ./run.sh ${svc} > last_run.log"
