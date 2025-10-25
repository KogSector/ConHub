# Use Node.js 18 Alpine for smaller image size
FROM node:18-alpine

# Set working directory
WORKDIR /app

# Install system dependencies
RUN apk add --no-cache \
    git \
    curl \
    && rm -rf /var/cache/apk/*

# Copy root package files (shared dependencies)
COPY package*.json ./

# Install dependencies from root
RUN npm ci --only=production && npm cache clean --force

# Copy MCP service source code
COPY mcp/src/ ./mcp/src/
COPY mcp/package.json ./mcp/

# Create logs directory
RUN mkdir -p logs

# Create non-root user for security
RUN addgroup -g 1001 -S nodejs && \
    adduser -S conhub -u 1001 -G nodejs

# Change ownership of app directory
RUN chown -R conhub:nodejs /app

# Switch to non-root user
USER conhub

# Expose port
EXPOSE 3004

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3004/api/health || exit 1

# Start the application
CMD ["node", "mcp/src/server.js"]
