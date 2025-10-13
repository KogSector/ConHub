import express from 'express';
import os from 'os';

const router = express.Router();


router.get('/', async (req, res) => {
  try {
    const health = {
      status: 'healthy',
      timestamp: new Date().toISOString(),
      service: 'ai-agents-service',
      version: '1.0.0',
      uptime: process.uptime(),
      environment: process.env.NODE_ENV || 'development',
      system: {
        platform: os.platform(),
        arch: os.arch(),
        nodeVersion: process.version,
        memory: {
          used: Math.round(process.memoryUsage().heapUsed / 1024 / 1024),
          total: Math.round(process.memoryUsage().heapTotal / 1024 / 1024),
          external: Math.round(process.memoryUsage().external / 1024 / 1024),
          rss: Math.round(process.memoryUsage().rss / 1024 / 1024)
        },
        cpu: {
          loadAverage: os.loadavg(),
          cores: os.cpus().length
        }
      },
      dependencies: {
        mcp: 'operational',
        webhooks: 'operational',
        agents: 'operational'
      }
    };

    res.json(health);
  } catch (error) {
    res.status(500).json({
      status: 'unhealthy',
      timestamp: new Date().toISOString(),
      error: error.message
    });
  }
});


router.get('/ready', async (req, res) => {
  try {
    
    const checks = {
      server: true,
      mcp: true,
      webhooks: true,
      agents: true
    };

    const allReady = Object.values(checks).every(check => check === true);

    if (allReady) {
      res.json({
        status: 'ready',
        timestamp: new Date().toISOString(),
        checks
      });
    } else {
      res.status(503).json({
        status: 'not ready',
        timestamp: new Date().toISOString(),
        checks
      });
    }
  } catch (error) {
    res.status(503).json({
      status: 'not ready',
      timestamp: new Date().toISOString(),
      error: error.message
    });
  }
});


router.get('/live', async (req, res) => {
  try {
    res.json({
      status: 'alive',
      timestamp: new Date().toISOString(),
      uptime: process.uptime()
    });
  } catch (error) {
    res.status(500).json({
      status: 'dead',
      timestamp: new Date().toISOString(),
      error: error.message
    });
  }
});


router.get('/detailed', async (req, res) => {
  try {
    const detailed = {
      status: 'healthy',
      timestamp: new Date().toISOString(),
      service: {
        name: 'ai-agents-service',
        version: '1.0.0',
        uptime: process.uptime(),
        environment: process.env.NODE_ENV || 'development'
      },
      system: {
        platform: os.platform(),
        arch: os.arch(),
        hostname: os.hostname(),
        nodeVersion: process.version,
        memory: {
          used: Math.round(process.memoryUsage().heapUsed / 1024 / 1024),
          total: Math.round(process.memoryUsage().heapTotal / 1024 / 1024),
          external: Math.round(process.memoryUsage().external / 1024 / 1024),
          rss: Math.round(process.memoryUsage().rss / 1024 / 1024),
          available: Math.round(os.freemem() / 1024 / 1024),
          totalSystem: Math.round(os.totalmem() / 1024 / 1024)
        },
        cpu: {
          loadAverage: os.loadavg(),
          cores: os.cpus().length,
          usage: process.cpuUsage()
        },
        network: os.networkInterfaces()
      },
      services: {
        mcp: {
          status: 'operational',
          description: 'Model Context Protocol service'
        },
        webhooks: {
          status: 'operational',
          description: 'Webhook processing service'
        },
        agents: {
          status: 'operational',
          description: 'AI agent management service'
        },
        websocket: {
          status: 'operational',
          description: 'WebSocket server for real-time communication'
        }
      },
      configuration: {
        port: process.env.AI_AGENTS_PORT || 3004,
        logLevel: process.env.LOG_LEVEL || 'info',
        nodeEnv: process.env.NODE_ENV || 'development'
      }
    };

    res.json(detailed);
  } catch (error) {
    res.status(500).json({
      status: 'unhealthy',
      timestamp: new Date().toISOString(),
      error: error.message
    });
  }
});

export default router;
