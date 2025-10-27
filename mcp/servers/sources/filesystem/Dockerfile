FROM node:18-alpine

WORKDIR /app

# Copy package files
COPY mcp/servers/sources/filesystem/package*.json ./

# Install dependencies
RUN npm install --omit=dev

# Copy application code
COPY mcp/servers/sources/filesystem/ .

EXPOSE 3007

CMD ["node", "server.js"]
