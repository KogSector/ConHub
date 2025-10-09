# Multi-stage build for LangChain Service
FROM node:18-alpine AS base

# Install system dependencies
RUN apk add --no-cache \
    curl \
    git \
    && rm -rf /var/cache/apk/*

WORKDIR /app

# Copy root package files (shared dependencies)
COPY package*.json ./

# Install dependencies from root
RUN npm ci --only=production && npm cache clean --force

# Copy langchain service source
COPY indexers/langchain-service/ ./langchain-service/

# Build the application
RUN cd langchain-service && npm run build

# Create non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S langchain -u 1001 -G nodejs

# Create logs directory
RUN mkdir -p logs && \
    chown -R langchain:nodejs /app

USER langchain

# Expose port
EXPOSE 8003

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8003/health || exit 1

# Start the application
CMD ["node", "langchain-service/dist/index.js"]
