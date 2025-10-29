import express from 'express';

const router = express.Router();

export default function connectorsRoutes(mcpService) {
  // Get all available connectors
  router.get('/', async (req, res) => {
    try {
      const connectors = mcpService.getConnectors();
      res.json({
        success: true,
        data: connectors,
        count: connectors.length
      });
    } catch (error) {
      res.status(500).json({
        success: false,
        error: 'Failed to retrieve connectors',
        message: error.message
      });
    }
  });

  // Get specific connector details
  router.get('/:id', async (req, res) => {
    try {
      const { id } = req.params;
      const connector = mcpService.getConnector(id);
      
      if (!connector) {
        return res.status(404).json({
          success: false,
          error: 'Connector not found',
          message: `Connector with ID '${id}' does not exist`
        });
      }

      res.json({
        success: true,
        data: {
          id,
          name: connector.name,
          version: connector.version,
          capabilities: connector.capabilities,
          metadata: connector.metadata,
          status: connector.connector ? 'active' : 'inactive'
        }
      });
    } catch (error) {
      res.status(500).json({
        success: false,
        error: 'Failed to retrieve connector',
        message: error.message
      });
    }
  });

  // Get connector health status
  router.get('/:id/health', async (req, res) => {
    try {
      const { id } = req.params;
      const health = await mcpService.getConnectorHealth(id);
      
      res.json({
        success: true,
        data: {
          connectorId: id,
          ...health
        }
      });
    } catch (error) {
      res.status(500).json({
        success: false,
        error: 'Failed to check connector health',
        message: error.message
      });
    }
  });

  // Get health status for all connectors
  router.get('/health/all', async (req, res) => {
    try {
      const healthStatus = await mcpService.getAllConnectorHealth();
      
      res.json({
        success: true,
        data: healthStatus
      });
    } catch (error) {
      res.status(500).json({
        success: false,
        error: 'Failed to check connector health',
        message: error.message
      });
    }
  });

  // Search across all connectors
  router.post('/search', async (req, res) => {
    try {
      const { query, options = {} } = req.body;
      
      if (!query) {
        return res.status(400).json({
          success: false,
          error: 'Missing required parameter',
          message: 'Query parameter is required'
        });
      }

      const results = await mcpService.searchConnectors(query, options);
      
      res.json({
        success: true,
        data: results
      });
    } catch (error) {
      res.status(500).json({
        success: false,
        error: 'Search failed',
        message: error.message
      });
    }
  });

  // Fetch data from specific connector
  router.post('/:id/fetch', async (req, res) => {
    try {
      const { id } = req.params;
      const { query } = req.body;
      
      if (!query) {
        return res.status(400).json({
          success: false,
          error: 'Missing required parameter',
          message: 'Query parameter is required'
        });
      }

      const data = await mcpService.fetchConnectorData(id, query);
      
      res.json({
        success: true,
        data: {
          connectorId: id,
          query,
          result: data
        }
      });
    } catch (error) {
      res.status(500).json({
        success: false,
        error: 'Failed to fetch data',
        message: error.message
      });
    }
  });

  // Get context from specific connector
  router.get('/:id/context/:resourceId', async (req, res) => {
    try {
      const { id, resourceId } = req.params;
      const options = req.query;
      
      const context = await mcpService.getConnectorContext(id, resourceId, options);
      
      res.json({
        success: true,
        data: {
          connectorId: id,
          resourceId,
          context
        }
      });
    } catch (error) {
      res.status(500).json({
        success: false,
        error: 'Failed to get context',
        message: error.message
      });
    }
  });

  // Search within specific connector
  router.post('/:id/search', async (req, res) => {
    try {
      const { id } = req.params;
      const { query, options = {} } = req.body;
      
      if (!query) {
        return res.status(400).json({
          success: false,
          error: 'Missing required parameter',
          message: 'Query parameter is required'
        });
      }

      const connector = mcpService.getConnector(id);
      
      if (!connector || !connector.connector) {
        return res.status(404).json({
          success: false,
          error: 'Connector not found',
          message: `Connector with ID '${id}' does not exist or is not active`
        });
      }

      if (typeof connector.connector.search !== 'function') {
        return res.status(400).json({
          success: false,
          error: 'Search not supported',
          message: `Connector '${id}' does not support search functionality`
        });
      }

      const results = await connector.connector.search(query, options);
      
      res.json({
        success: true,
        data: {
          connectorId: id,
          connectorName: connector.name,
          query,
          ...results
        }
      });
    } catch (error) {
      res.status(500).json({
        success: false,
        error: 'Search failed',
        message: error.message
      });
    }
  });

  return router;
}