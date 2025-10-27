FROM node:18-alpine

# Install git for GitHub dependencies
RUN apk add --no-cache git

WORKDIR /app

# Copy package files
COPY mcp/servers/sources/dropbox/package*.json ./

# Install dependencies
RUN npm install --omit=dev

# Copy application code
COPY mcp/servers/sources/dropbox/ .

EXPOSE 3006

CMD ["node", "server.js"]
