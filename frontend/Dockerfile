# Multi-stage build for Next.js frontend
# Use Node 20 to satisfy engine requirements for some dependencies (e.g., Octokit)
FROM node:20-alpine AS base

# Development stage
FROM base AS development
WORKDIR /app
COPY frontend/package*.json ./
RUN npm install
COPY frontend/ .
EXPOSE 3000
CMD ["npm", "run", "dev"]

# Install dependencies only when needed
FROM base AS deps
RUN apk add --no-cache libc6-compat
WORKDIR /app

# Copy package files only
COPY frontend/package*.json ./
# Install both production and dev dependencies for build time (PostCSS/autoprefixer)
# Use npm ci for reproducible installs based on the lockfile
RUN npm ci

# Rebuild the source code only when needed
FROM base AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY frontend/ .
COPY feature-toggles.json ../

# Build the frontend
RUN npm run build

# Production image, copy all the files and run next
FROM base AS runner
WORKDIR /app

ENV NODE_ENV production

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

# Copy built application
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
COPY --from=builder /app/public ./public

USER nextjs

EXPOSE 3000

ENV PORT 3000
ENV HOSTNAME "0.0.0.0"

CMD ["node", "server.js"]
