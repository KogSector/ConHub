import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import morgan from 'morgan';
import dotenv from 'dotenv';
import { errorHandler, notFound } from './middleware/errorMiddleware';
import { logger, createRequestLogger, logHealthCheck, performanceLogger } from './utils/logger';
import indexingRoutes from './routes/indexing';
import searchRoutes from './routes/search';
import dataSourceRoutes from './routes/dataSources';
import aiAgentRoutes from './routes/aiAgents';
// import authRoutes from './routes/auth'; // Temporarily disabled due to ES module issues
import copilotRoutes from './routes/copilot';
import githubRoutes from './routes/github';

// Load environment variables
dotenv.config();

const app = express();
const PORT = process.env.LANGCHAIN_PORT || 8003;

logger.info('Initializing LangChain service', { port: PORT });

// Middleware
app.use(helmet());
app.use(cors({
  origin: process.env.NODE_ENV === 'production' 
    ? ['https://your-frontend-domain.com'] 
    : ['http://localhost:3000'],
  credentials: true
}));

// Enhanced request logging
app.use(createRequestLogger());

app.use(morgan('combined', {
  stream: {
    write: (message: string) => logger.info(message.trim())
  }
}));
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true }));

// Health check endpoint
app.get('/health', (req: express.Request, res: express.Response) => {
  res.json({ 
    status: 'OK', 
    service: 'ConHub LangChain Service',
    timestamp: new Date().toISOString(),
    version: '1.0.0'
  });
});

// API Routes
app.use('/api/indexing', indexingRoutes);
app.use('/api/search', searchRoutes);
app.use('/api/data-sources', dataSourceRoutes);
app.use('/api/ai-agents', aiAgentRoutes);
// app.use('/api/auth', authRoutes); // Temporarily disabled due to ES module issues
app.use('/api/copilot', copilotRoutes);
app.use('/api/github', githubRoutes);

// Error handling
app.use(notFound);
app.use(errorHandler);

// Start server
app.listen(PORT, () => {
  logger.info(`ConHub LangChain Service running on port ${PORT}`);
  logger.info(`Environment: ${process.env.NODE_ENV || 'development'}`);
});

export default app;
