# =============================================================================
# GPAI web — Next.js 15 multi-stage Dockerfile
#
# Build context MUST be the monorepo root because:
#   * pnpm-workspace.yaml lives at repo root
#   * apps/web/package.json depends on workspace packages (`packages/*`
#     and `gen/ts`) which must be on disk so pnpm can symlink them into
#     apps/web/node_modules/@gpai/*.
# =============================================================================

# -------- Deps stage --------
FROM node:20-alpine AS deps

WORKDIR /repo

RUN corepack enable && corepack prepare pnpm@9.15.0 --activate

# Copy workspace manifests and the actual contents of every workspace
# package apps/web depends on. `pnpm install --filter @gpai/web...`
# symlinks these into apps/web/node_modules/@gpai/*.
COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY apps/web/package.json       apps/web/
COPY gen/ts                      gen/ts/
COPY packages                    packages

RUN pnpm install --frozen-lockfile --filter @gpai/web...

# -------- Builder stage --------
FROM node:20-alpine AS builder

WORKDIR /repo

RUN corepack enable && corepack prepare pnpm@9.15.0 --activate

# Bring the deps tree (already-installed node_modules + workspace pkgs)
# in from the deps stage.
COPY --from=deps /repo/node_modules           ./node_modules
COPY --from=deps /repo/apps/web/node_modules  ./apps/web/node_modules
COPY --from=deps /repo/gen/ts                 ./gen/ts
COPY --from=deps /repo/packages               ./packages

# Now the rest of the web app source (after deps so they cache cleanly).
COPY apps/web apps/web
COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./

ENV NEXT_TELEMETRY_DISABLED=1

RUN pnpm --filter @gpai/web build

# -------- Runner stage --------
FROM node:20-alpine AS runner

WORKDIR /repo

ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
ENV PORT=3000
ENV HOSTNAME=0.0.0.0

RUN addgroup --system --gid 1001 nodejs \
 && adduser  --system --uid 1001 --ingroup nodejs --no-create-home nextjs

COPY --from=builder /repo/apps/web/.next         apps/web/.next
COPY --from=builder /repo/apps/web/package.json  apps/web/package.json
COPY --from=builder /repo/node_modules           node_modules

USER nextjs

EXPOSE 3000

CMD ["pnpm", "--filter", "@gpai/web", "start"]