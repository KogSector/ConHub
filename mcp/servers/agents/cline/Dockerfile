FROM node:20-alpine

WORKDIR /app

# Copy package files
COPY mcp/servers/agents/cline/package*.json ./

# Install dependencies
RUN npm install --omit=dev

# Copy application code
COPY mcp/servers/agents/cline/ .

EXPOSE 3010

CMD ["npm", "start"]
