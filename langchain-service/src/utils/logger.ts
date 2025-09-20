import winston from 'winston';
import path from 'path';
import fs from 'fs';

// Ensure logs directory exists
const logsDir = path.join(process.cwd(), 'logs');
if (!fs.existsSync(logsDir)) {
  fs.mkdirSync(logsDir, { recursive: true });
}

// Custom format for structured logging
const customFormat = winston.format.combine(
  winston.format.timestamp({
    format: 'YYYY-MM-DD HH:mm:ss.SSS'
  }),
  winston.format.errors({ stack: true }),
  winston.format.printf((info) => {
    const { timestamp, level, message, service, ...meta } = info;
    const metaStr = Object.keys(meta).length ? JSON.stringify(meta, null, 2) : '';
    return `${timestamp} [${level.toUpperCase()}] [${service}] ${message} ${metaStr}`;
  })
);

// JSON format for production
const jsonFormat = winston.format.combine(
  winston.format.timestamp(),
  winston.format.errors({ stack: true }),
  winston.format.json()
);

// Create custom transport classes for filtering
class PerformanceFileTransport extends winston.transports.File {
  constructor(opts: any) {
    super(opts);
  }
  
  log(info: any, callback: any) {
    if (info.category === 'performance') {
      super.log(info, callback);
    } else {
      callback(null, true);
    }
  }
}

class AIOperationFileTransport extends winston.transports.File {
  constructor(opts: any) {
    super(opts);
  }
  
  log(info: any, callback: any) {
    if (info.category === 'ai-operation') {
      super.log(info, callback);
    } else {
      callback(null, true);
    }
  }
}

// Create the logger
export const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || (process.env.NODE_ENV === 'production' ? 'info' : 'debug'),
  format: process.env.NODE_ENV === 'production' ? jsonFormat : customFormat,
  defaultMeta: { 
    service: 'conhub-langchain-service',
    version: process.env.npm_package_version || '1.0.0',
    environment: process.env.NODE_ENV || 'development',
    hostname: require('os').hostname(),
    pid: process.pid
  },
  transports: [
    // Error logs
    new winston.transports.File({ 
      filename: path.join(logsDir, 'error.log'), 
      level: 'error',
      maxsize: 10485760, // 10MB
      maxFiles: 5,
      format: jsonFormat
    }),
    
    // Combined logs
    new winston.transports.File({ 
      filename: path.join(logsDir, 'combined.log'),
      maxsize: 10485760, // 10MB
      maxFiles: 5,
      format: jsonFormat
    }),
    
    // Performance logs
    new PerformanceFileTransport({ 
      filename: path.join(logsDir, 'performance.log'),
      level: 'info',
      maxsize: 10485760, // 10MB
      maxFiles: 5,
      format: jsonFormat
    }),
    
    // AI operation logs
    new AIOperationFileTransport({ 
      filename: path.join(logsDir, 'ai-operations.log'),
      level: 'info',
      maxsize: 10485760, // 10MB
      maxFiles: 5,
      format: jsonFormat
    })
  ],
});

// Console transport for development
if (process.env.NODE_ENV !== 'production') {
  logger.add(new winston.transports.Console({
    format: winston.format.combine(
      winston.format.colorize({ all: true }),
      winston.format.timestamp({ format: 'HH:mm:ss.SSS' }),
      winston.format.printf((info) => {
        const { timestamp, level, message, service, ...meta } = info;
        const metaStr = Object.keys(meta).length ? JSON.stringify(meta, null, 2) : '';
        return `${timestamp} [${level}] [${service}] ${message} ${metaStr}`;
      })
    )
  }));
}

// Performance monitoring class
export class PerformanceLogger {
  private startTimes: Map<string, number> = new Map();
  
  startTimer(operationId: string, operation: string, context?: Record<string, any>) {
    const startTime = performance.now();
    this.startTimes.set(operationId, startTime);
    
    logger.debug('Operation started', {
      operationId,
      operation,
      ...context,
      category: 'performance'
    });
  }
  
  endTimer(operationId: string, operation: string, context?: Record<string, any>) {
    const startTime = this.startTimes.get(operationId);
    if (!startTime) {
      logger.warn('Timer not found for operation', { operationId, operation });
      return 0;
    }
    
    const duration = performance.now() - startTime;
    this.startTimes.delete(operationId);
    
    logger.info('Operation completed', {
      operationId,
      operation,
      duration: Math.round(duration * 100) / 100, // Round to 2 decimal places
      ...context,
      category: 'performance'
    });
    
    // Log slow operations as warnings
    if (duration > 5000) { // 5 seconds
      logger.warn('Slow operation detected', {
        operationId,
        operation,
        duration: Math.round(duration * 100) / 100,
        ...context,
        category: 'performance'
      });
    }
    
    return duration;
  }
  
  logMetric(metric: string, value: number, unit: string, context?: Record<string, any>) {
    logger.info('Performance metric', {
      metric,
      value,
      unit,
      ...context,
      category: 'performance'
    });
  }
}

// AI operation logging
export class AIOperationLogger {
  logQuery(queryId: string, query: string, agentType: string, context?: Record<string, any>) {
    logger.info('AI query initiated', {
      queryId,
      queryLength: query.length,
      queryPreview: query.substring(0, 100),
      agentType,
      ...context,
      category: 'ai-operation'
    });
  }
  
  logResponse(queryId: string, success: boolean, responseLength?: number, error?: string, context?: Record<string, any>) {
    logger.info('AI query completed', {
      queryId,
      success,
      responseLength,
      error,
      ...context,
      category: 'ai-operation'
    });
  }
  
  logEmbeddingOperation(operation: string, documentCount: number, duration: number, context?: Record<string, any>) {
    logger.info('Embedding operation', {
      operation,
      documentCount,
      duration: Math.round(duration * 100) / 100,
      ...context,
      category: 'ai-operation'
    });
  }
  
  logSearchOperation(query: string, resultCount: number, duration: number, filters?: Record<string, any>) {
    logger.info('Search operation', {
      queryLength: query.length,
      queryPreview: query.substring(0, 50),
      resultCount,
      duration: Math.round(duration * 100) / 100,
      filters,
      category: 'ai-operation'
    });
  }
}

// Request logging middleware enhancement
export function createRequestLogger() {
  return (req: any, res: any, next: any) => {
    const startTime = performance.now();
    const requestId = `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    // Add request ID to request object
    req.requestId = requestId;
    
    logger.info('Request started', {
      requestId,
      method: req.method,
      url: req.url,
      userAgent: req.get('User-Agent'),
      ip: req.ip,
      category: 'request'
    });
    
    // Override res.end to log response
    const originalEnd = res.end;
    res.end = function(chunk: any, encoding: any) {
      const duration = performance.now() - startTime;
      
      logger.info('Request completed', {
        requestId,
        method: req.method,
        url: req.url,
        statusCode: res.statusCode,
        duration: Math.round(duration * 100) / 100,
        contentLength: res.get('Content-Length'),
        category: 'request'
      });
      
      originalEnd.call(this, chunk, encoding);
    };
    
    next();
  };
}

// Error logging enhancement
export function logError(error: Error, context?: Record<string, any>) {
  logger.error('Application error', {
    message: error.message,
    stack: error.stack,
    name: error.name,
    ...context,
    category: 'error'
  });
}

// Health check logging
export function logHealthCheck(component: string, status: 'healthy' | 'unhealthy', details?: Record<string, any>) {
  logger.info('Health check', {
    component,
    status,
    ...details,
    category: 'health'
  });
}

// System metrics logging
export function logSystemMetrics() {
  const usage = process.memoryUsage();
  const cpuUsage = process.cpuUsage();
  
  logger.info('System metrics', {
    memory: {
      rss: Math.round(usage.rss / 1024 / 1024 * 100) / 100, // MB
      heapTotal: Math.round(usage.heapTotal / 1024 / 1024 * 100) / 100,
      heapUsed: Math.round(usage.heapUsed / 1024 / 1024 * 100) / 100,
      external: Math.round(usage.external / 1024 / 1024 * 100) / 100
    },
    cpu: {
      user: cpuUsage.user,
      system: cpuUsage.system
    },
    uptime: process.uptime(),
    category: 'system'
  });
}

// Create instances
export const performanceLogger = new PerformanceLogger();
export const aiLogger = new AIOperationLogger();

// Start periodic system metrics logging
if (process.env.NODE_ENV !== 'test') {
  setInterval(() => {
    logSystemMetrics();
  }, 60000); // Log every minute
}

// Log startup information
logger.info('LangChain service starting', {
  nodeVersion: process.version,
  platform: process.platform,
  arch: process.arch,
  environment: process.env.NODE_ENV,
  port: process.env.PORT || 3001,
  category: 'startup'
});
