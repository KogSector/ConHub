import { Server } from '@modelcontextprotocol/server-filesystem';
import http from 'http';

const PORT = process.env.MCP_FILESYSTEM_PORT || 3007;

// Configuration from environment variables
const allowedPaths = process.env.FILE_SYSTEM_ALLOWED_PATHS
  ? process.env.FILE_SYSTEM_ALLOWED_PATHS.split(',').map(p => p.trim())
  : ['/workspace/data', '/workspace/uploads'];

const config = {
  allowedDirectories: allowedPaths,
  allowSymlinks: false, // Security: Prevent symlink traversal attacks
  allowHidden: false, // Don't allow access to hidden files
};

console.log('Filesystem MCP Server starting with configuration:');
console.log('Allowed paths:', allowedPaths);

// Initialize MCP server
const mcpServer = new Server(config);

// Create HTTP server to handle MCP requests
const server = http.createServer(async (req, res) => {
  // Enable CORS for backend communication
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');

  if (req.method === 'OPTIONS') {
    res.writeHead(200);
    res.end();
    return;
  }

  // Health check endpoint
  if (req.url === '/health' && req.method === 'GET') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({
      status: 'healthy',
      service: 'filesystem-mcp',
      allowedPaths: allowedPaths
    }));
    return;
  }

  // Handle MCP JSON-RPC requests
  if (req.method === 'POST') {
    let body = '';
    req.on('data', chunk => body += chunk);
    req.on('end', async () => {
      try {
        const request = JSON.parse(body);
        const response = await mcpServer.handleRequest(request);

        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(response));
      } catch (error) {
        console.error('Error handling MCP request:', error);
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({
          jsonrpc: '2.0',
          error: { code: -32603, message: 'Internal error', data: error.message },
          id: null
        }));
      }
    });
  } else {
    res.writeHead(404, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'Not found' }));
  }
});

server.listen(PORT, () => {
  console.log(`Filesystem MCP Server listening on port ${PORT}`);
  console.log(`Health check: http://localhost:${PORT}/health`);
});

// Graceful shutdown
process.on('SIGTERM', () => {
  console.log('Received SIGTERM, shutting down gracefully...');
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
});
