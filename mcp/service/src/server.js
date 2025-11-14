import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import rateLimit from 'express-rate-limit';
import dotenv from 'dotenv';
import path from 'path';


dotenv.config({ path: path.resolve(__dirname, '.env') });

import { createServer } from 'http';
import { WebSocketServer } from 'ws';
import winston from 'winston';


import healthRoutes from './routes/health.js';
import mcpRoutes from './routes/mcp.js';
import webhookRoutes from './routes/webhooks.js';
import agentRoutes from './routes/agents.js';
import connectorsRoutes from './routes/connectors.js';


import { McpService } from './services/McpService.js';
import { WebhookService } from './services/WebhookService.js';
import { AgentManager } from './services/AgentManager.js';


const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    winston.format.json()
  ),
  defaultMeta: { service: 'ai-agents-service' },
  transports: [
    new winston.transports.File({ filename: 'logs/error.log', level: 'error' }),
    new winston.transports.File({ filename: 'logs/combined.log' }),
    new winston.transports.Console({
      format: winston.format.combine(
        winston.format.colorize(),
        winston.format.simple()
      )
    })
  ]
});

const app = express();
const server = createServer(app);
const wss = new WebSocketServer({ server });

const PORT = process.env.MCP_SERVICE_PORT || 3004;


app.use(helmet());
app.use(cors({
  origin: process.env.ALLOWED_ORIGINS?.split(',') || ['http://localhost:3000', 'http://localhost:3001'],
  credentials: true
}));


const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, 
  max: 100, 
  message: 'Too many requests from this IP, please try again later.'
});
app.use(limiter);


app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true, limit: '10mb' }));


const mcpService = new McpService(logger);
const webhookService = new WebhookService(logger);
const agentManager = new AgentManager(logger, mcpService, webhookService);


wss.on('connection', (ws, request) => {
  logger.info('New WebSocket connection established', { 
    ip: request.socket.remoteAddress,
    userAgent: request.headers['user-agent']
  });

  ws.on('message', async (message) => {
    try {
      const data = JSON.parse(message.toString());
      await agentManager.handleWebSocketMessage(ws, data);
    } catch (error) {
      logger.error('WebSocket message handling error:', error);
      ws.send(JSON.stringify({
        type: 'error',
        message: 'Invalid message format'
      }));
    }
  });

  ws.on('close', () => {
    logger.info('WebSocket connection closed');
    agentManager.handleWebSocketDisconnection(ws);
  });

  ws.on('error', (error) => {
    logger.error('WebSocket error:', error);
  });
});


app.use((req, res, next) => {
  logger.info('Incoming request', {
    method: req.method,
    url: req.url,
    ip: req.ip,
    userAgent: req.get('User-Agent')
  });
  next();
});


app.use('/api/health', healthRoutes);
app.use('/api/mcp', mcpRoutes(mcpService));
app.use('/api/webhooks', webhookRoutes(webhookService));
app.use('/api/agents', agentRoutes(agentManager));
app.use('/api/connectors', connectorsRoutes(mcpService));


app.use((error, req, res, next) => {
  logger.error('Unhandled error:', error);
  res.status(500).json({
    error: 'Internal server error',
    message: process.env.NODE_ENV === 'development' ? error.message : 'Something went wrong'
  });
});


app.use('*', (req, res) => {
  res.status(404).json({
    error: 'Not found',
    message: `Route ${req.originalUrl} not found`
  });
});


process.on('SIGTERM', async () => {
  logger.info('SIGTERM received, shutting down gracefully');
  await mcpService.cleanup();
  server.close(() => {
    logger.info('Process terminated');
    process.exit(0);
  });
});

process.on('SIGINT', async () => {
  logger.info('SIGINT received, shutting down gracefully');
  await mcpService.cleanup();
  server.close(() => {
    logger.info('Process terminated');
    process.exit(0);
  });
});

server.listen(PORT, () => {
  logger.info(`🔗 ConHub MCP Service running on port ${PORT}`);
  logger.info(`📡 Model Context Protocol server ready`);
  logger.info(`🤖 AI agent connectivity hub operational`);
  logger.info(`🪝 Webhook handlers ready for GitHub Copilot, Amazon Q, and Cline`);
  logger.info(`⚡ WebSocket server ready for real-time communication`);
  
  if (process.env.NODE_ENV === 'development') {
    logger.info('🔧 Development mode - detailed logging enabled');
  }
});

export { app, server, wss, logger };
