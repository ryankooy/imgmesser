#!/bin/sh

certbot certonly --webroot \
    -w /var/www/certbot -d "${DOMAIN}" --email "${EMAIL}" \
    --agree-tos --no-eff-email --force-renewal

