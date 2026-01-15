# STAGE 1: Build the app

FROM node:lts-alpine AS build-stage

WORKDIR /app

COPY ./client/package*.json ./

# Clean install
RUN npm ci

COPY ./client .

RUN npm run build

COPY ./client/worker.js /app/dist

# STAGE 2: Serve with Nginx

FROM nginx:stable-alpine

COPY --from=build-stage /app/dist /usr/share/nginx/html

CMD ["nginx", "-g", "daemon off;"]
