#!/bin/sh

mkdir -p /etc/letsencrypt/live/"${DOMAIN}"

openssl req -x509 -nodes -newkey rsa:4096 -days 1 \
    -keyout /etc/letsencrypt/live/"${DOMAIN}"/privkey.pem \
    -out /etc/letsencrypt/live/"${DOMAIN}"/fullchain.pem \
    -subj '/CN=localhost'

