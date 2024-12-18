FROM oven/bun:1 AS base

ARG VITE_ENGINE_HOST
ENV VITE_ENGINE_HOST=${VITE_ENGINE_HOST}

WORKDIR /app
COPY package.json bun.lockb ./
RUN bun install
COPY . .
RUN bun run build

FROM node:20-alpine

RUN npm install -g sirv-cli

WORKDIR /app
COPY --from=base /app/dist ./dist

EXPOSE 3000
ENTRYPOINT [ "sirv", "dist", "--port", "3000", "--host", "0.0.0.0" ]