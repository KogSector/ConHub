FROM node:20-alpine

WORKDIR /app

# Copy package files
COPY mcp/servers/agents/github-copilot/package*.json ./

# Install dependencies
RUN npm install --omit=dev

# Copy application code
COPY mcp/servers/agents/github-copilot/ .

EXPOSE 3008

CMD ["npm", "start"]
