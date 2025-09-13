import express from 'express';
import { connectDataSource, disconnectDataSource, getDataSources, syncDataSource } from '../services/dataSourceService';
import { logger } from '../utils/logger';

const router = express.Router();

// Get all connected data sources
router.get('/', async (req, res) => {
  try {
    const dataSources = await getDataSources();
    
    res.json({
      success: true,
      dataSources
    });
  } catch (error) {
    logger.error('Error getting data sources:', error);
    res.status(500).json({ 
      error: 'Failed to get data sources',
      details: error instanceof Error ? error.message : 'Unknown error'
    });
  }
});

// Connect a new data source
router.post('/connect', async (req: any, res: any) => {
  try {
    const { type, credentials, config } = req.body;
    
    if (!type) {
      return res.status(400).json({ error: 'Data source type is required' });
    }

    logger.info(`Connecting data source: ${type}`);
    
    const result = await connectDataSource(type, credentials, config);
    
    return res.json({
      success: true,
      message: 'Data source connected successfully',
      dataSource: result
    });
  } catch (error) {
    logger.error('Error connecting data source:', error);
    return res.status(500).json({ 
      error: 'Failed to connect data source',
      details: error instanceof Error ? error.message : 'Unknown error'
    });
  }
});

// Disconnect a data source
router.delete('/:sourceId', async (req, res) => {
  try {
    const { sourceId } = req.params;
    
    logger.info(`Disconnecting data source: ${sourceId}`);
    
    await disconnectDataSource(sourceId);
    
    res.json({
      success: true,
      message: 'Data source disconnected successfully'
    });
  } catch (error) {
    logger.error('Error disconnecting data source:', error);
    res.status(500).json({ 
      error: 'Failed to disconnect data source',
      details: error instanceof Error ? error.message : 'Unknown error'
    });
  }
});

// Sync a data source (re-index)
router.post('/:sourceId/sync', async (req, res) => {
  try {
    const { sourceId } = req.params;
    
    logger.info(`Syncing data source: ${sourceId}`);
    
    const result = await syncDataSource(sourceId);
    
    res.json({
      success: true,
      message: 'Data source sync started',
      syncId: result.syncId
    });
  } catch (error) {
    logger.error('Error syncing data source:', error);
    res.status(500).json({ 
      error: 'Failed to sync data source',
      details: error instanceof Error ? error.message : 'Unknown error'
    });
  }
});

export default router;
