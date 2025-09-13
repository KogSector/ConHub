import { vectorStore } from './vectorStore.js';
import { logger } from '../utils/logger.js';

interface SearchResult {
  id: string;
  content: string;
  metadata: any;
  score: number;
  source: string;
}

export async function searchContent(
  query: string,
  filters?: any,
  limit: number = 10
): Promise<SearchResult[]> {
  try {
    logger.info(`Searching content: ${query}`);
    
    const results = await vectorStore.similaritySearchWithScore(query, limit, filters);
    
    return results.map(([doc, score], index) => ({
      id: `result-${index}`,
      content: doc.pageContent,
      metadata: doc.metadata,
      score,
      source: doc.metadata.sourceType || 'unknown'
    }));
  } catch (error) {
    logger.error('Error in content search:', error);
    throw error;
  }
}

export async function searchCode(
  query: string,
  filters?: { repositories?: string[]; language?: string },
  limit: number = 10
): Promise<SearchResult[]> {
  try {
    logger.info(`Searching code: ${query}`);
    
    const searchFilters = {
      sourceType: 'repository',
      ...filters
    };
    
    const results = await vectorStore.similaritySearchWithScore(query, limit, searchFilters);
    
    return results.map(([doc, score], index) => ({
      id: `code-${index}`,
      content: doc.pageContent,
      metadata: doc.metadata,
      score,
      source: 'code'
    }));
  } catch (error) {
    logger.error('Error in code search:', error);
    throw error;
  }
}

export async function searchDocuments(
  query: string,
  filters?: { documentTypes?: string[]; sources?: string[] },
  limit: number = 10
): Promise<SearchResult[]> {
  try {
    logger.info(`Searching documents: ${query}`);
    
    const searchFilters = {
      sourceType: 'document',
      ...filters
    };
    
    const results = await vectorStore.similaritySearchWithScore(query, limit, searchFilters);
    
    return results.map(([doc, score], index) => ({
      id: `doc-${index}`,
      content: doc.pageContent,
      metadata: doc.metadata,
      score,
      source: 'document'
    }));
  } catch (error) {
    logger.error('Error in document search:', error);
    throw error;
  }
}