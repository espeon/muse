# use the official Bun image
# see all versions at https://hub.docker.com/r/oven/bun/tags
FROM oven/bun:1 AS base
WORKDIR /usr/src/app

# install dependencies into temp directory
# this will cache them and speed up future builds
FROM base AS install
RUN mkdir -p /temp/dev
COPY package.json bun.lockb /temp/dev/
RUN cd /temp/dev && bun install --frozen-lockfile

# install with --production (exclude devDependencies)
RUN mkdir -p /temp/prod
COPY package.json bun.lockb /temp/prod/
RUN cd /temp/prod && bun install --frozen-lockfile --production

# copy node_modules from temp directory
# then copy all (non-ignored) project files into the image
FROM base AS builder
WORKDIR /app
COPY --from=install /temp/dev/node_modules node_modules
COPY . .

# tests & build
RUN bun test
RUN bun run build

# copy production dependencies and source code into final image
FROM base AS runner
WORKDIR /app

ENV NODE_ENV=production

# our user is bun:nextjs
RUN addgroup --system --gid 1001 bun
RUN adduser --system --uid 1001 nextjs

COPY --from=builder /app/public ./public

# Set the correct permission for prerender cache
RUN mkdir .next
RUN chown nextjs:bun .next


# Automatically leverage output traces to reduce image size
# https://nextjs.org/docs/advanced-features/output-file-tracing
COPY --from=builder --chown=nextjs:bun /app/.next/standalone ./
COPY --from=builder --chown=nextjs:bun /app/.next/static ./.next/static

# Remove environment configs
RUN rm -rf .env*

# run the app
USER nextjs
EXPOSE 3031/tcp
ENV PORT=3031
ENTRYPOINT [ "node", "server.js" ]
