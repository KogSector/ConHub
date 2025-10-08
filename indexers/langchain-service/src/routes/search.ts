import express from 'express';
import { searchContent, searchCode, searchDocuments } from '../services/searchService';
import { logger } from '../utils/logger';

const router = express.Router();

// Universal search across all indexed content
router.post('/universal', async (req: any, res: any) => {
  try {
    const { query, filters, limit = 10 } = req.body;
    
    if (!query) {
      return res.status(400).json({ error: 'Search query is required' });
    }

    logger.info(`Universal search query: ${query}`);
    
    const results = await searchContent(query, filters, limit);
    
    return res.json({
      success: true,
      query,
      results,
      totalCount: results.length
    });
  } catch (error) {
    logger.error('Error in universal search:', error);
    return res.status(500).json({ 
      error: 'Search failed',
      details: error instanceof Error ? error.message : 'Unknown error'
    });
  }
});

// Search specifically in code repositories
router.post('/code', async (req: any, res: any) => {
  try {
    const { query, repositories, language, limit = 10 } = req.body;
    
    if (!query) {
      return res.status(400).json({ error: 'Search query is required' });
    }

    logger.info(`Code search query: ${query}`);
    
    const results = await searchCode(query, { repositories, language }, limit);
    
    return res.json({
      success: true,
      query,
      results,
      totalCount: results.length
    });
  } catch (error) {
    logger.error('Error in code search:', error);
    return res.status(500).json({ 
      error: 'Code search failed',
      details: error instanceof Error ? error.message : 'Unknown error'
    });
  }
});

// Search in documents and files
router.post('/documents', async (req: any, res: any) => {
  try {
    const { query, documentTypes, sources, limit = 10 } = req.body;
    
    if (!query) {
      return res.status(400).json({ error: 'Search query is required' });
    }

    logger.info(`Document search query: ${query}`);
    
    const results = await searchDocuments(query, { documentTypes, sources }, limit);
    
    return res.json({
      success: true,
      query,
      results,
      totalCount: results.length
    });
  } catch (error) {
    logger.error('Error in document search:', error);
    return res.status(500).json({ 
      error: 'Document search failed',
      details: error instanceof Error ? error.message : 'Unknown error'
    });
  }
});

export default router;
