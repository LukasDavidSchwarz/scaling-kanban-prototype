FROM node:19-alpine AS base
WORKDIR /frontend
COPY package.json .
COPY package-lock.json .
RUN npm i
COPY . .

FROM base AS development-watch
CMD ["npm", "run", "dev"]

FROM base AS builder
ARG VITE_API_HOST
ARG VITE_PORT
ARG VITE_REST_API_PROTOCOL
ARG VITE_WS_API_PROTOCOL
ARG VITE_APP_TITLE

RUN npm run build

FROM nginx:1.24.0-alpine AS production
ARG FRONTEND_PORT

COPY nginx.conf /etc/nginx/templates/default.conf.template
COPY --from=builder /frontend/build /usr/share/nginx/html
EXPOSE "$FRONTEND_PORT"
CMD ["nginx", "-g", "daemon off;"]
