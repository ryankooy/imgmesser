# STAGE 1: Build the app

FROM node:lts-alpine AS build-stage

WORKDIR /app

COPY package*.json ./

# Clean install
RUN npm ci

COPY . .

RUN npm run build

# STAGE 2: Serve with Nginx

FROM nginx:stable-alpine

COPY --from=build-stage /app/dist /usr/share/nginx/html

EXPOSE 5173

CMD ["nginx", "-g", "daemon off;"]
