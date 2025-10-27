FROM node:18-alpine

WORKDIR /app

# Copy package files
COPY mcp/servers/sources/google-drive/package*.json ./

# Install dependencies
RUN npm install --omit=dev

# Copy application code
COPY mcp/servers/sources/google-drive/ .

EXPOSE 3005

CMD ["node", "server.js"]
