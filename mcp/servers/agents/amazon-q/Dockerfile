FROM node:20-alpine

WORKDIR /app

# Copy package files
COPY mcp/servers/agents/amazon-q/package*.json ./

# Install dependencies
RUN npm install --omit=dev

# Copy application code
COPY mcp/servers/agents/amazon-q/ .

EXPOSE 3009

CMD ["npm", "start"]
