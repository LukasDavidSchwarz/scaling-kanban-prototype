FROM node:19-alpine AS base
WORKDIR /frontend
COPY package.json .
RUN npm i
COPY . .

FROM base AS development-watch
CMD ["npm", "run", "dev"]

FROM base AS builder
ARG VITE_API_HOST
ARG VITE_REST_API_PROTOCOL
ARG VITE_WS_API_PROTOCOL
ARG VITE_APP_TITLE

RUN npm run build

FROM nginx:1.24.0-alpine AS production
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /frontend/build /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
