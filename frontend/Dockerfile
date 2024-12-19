FROM oven/bun:1 AS base

ENV NODE_ENV=production

WORKDIR /app
COPY package.json bun.lockb ./
RUN bun install

ARG VITE_ENGINE_HOST
ENV VITE_ENGINE_HOST=${VITE_ENGINE_HOST}

COPY . .
RUN bun run build

FROM node:20-alpine

RUN npm install -g sirv-cli

WORKDIR /app
COPY --from=base /app/dist ./dist

EXPOSE 3000
ENTRYPOINT [ "sirv", "dist", "--port", "3000", "--host", "0.0.0.0" ]