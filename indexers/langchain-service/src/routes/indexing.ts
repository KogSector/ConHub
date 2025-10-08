import express, { Request, Response } from 'express';
import { indexRepository, indexDocument, getIndexingStatus } from '../services/indexingService';
import { logger } from '../utils/logger';

const router = express.Router();

// Index a GitHub repository
router.post('/repository', async (req: any, res: any) => {
  try {
    const { repoUrl, accessToken } = req.body;
    
    if (!repoUrl) {
      return res.status(400).json({ error: 'Repository URL is required' });
    }

    logger.info(`Starting indexing for repository: ${repoUrl}`);
    
    const result = await indexRepository(repoUrl, accessToken);
    
    return res.json({
      success: true,
      message: 'Repository indexing started',
      indexId: result.indexId
    });
  } catch (error) {
    logger.error('Error indexing repository:', error);
    return res.status(500).json({ 
      error: 'Failed to start repository indexing',
      details: error instanceof Error ? error.message : 'Unknown error'
    });
  }
});

// Index a document or file
router.post('/document', async (req: any, res: any) => {
  try {
    const { documentUrl, type, metadata } = req.body;
    
    if (!documentUrl) {
      return res.status(400).json({ error: 'Document URL is required' });
    }

    logger.info(`Starting indexing for document: ${documentUrl}`);
    
    const result = await indexDocument(documentUrl, type, metadata);
    
    return res.json({
      success: true,
      message: 'Document indexing started',
      indexId: result.indexId
    });
  } catch (error) {
    logger.error('Error indexing document:', error);
    return res.status(500).json({ 
      error: 'Failed to start document indexing',
      details: error instanceof Error ? error.message : 'Unknown error'
    });
  }
});

// Get indexing status
router.get('/status/:indexId', async (req: any, res: any) => {
  try {
    const { indexId } = req.params;
    
    const status = await getIndexingStatus(indexId);
    
    return res.json(status);
  } catch (error) {
    logger.error('Error getting indexing status:', error);
    return res.status(500).json({ 
      error: 'Failed to get indexing status',
      details: error instanceof Error ? error.message : 'Unknown error'
    });
  }
});

export default router;
