import { vectorStore } from './vectorStore';
import { logger } from '../utils/logger';

export interface SearchResult {
  id: string;
  content: string;
  metadata: Record<string, any>;
  score?: number;
  source: string;
  type: 'code' | 'document' | 'web';
}

export async function searchContent(
  query: string,
  filters: Record<string, any> = {},
  limit: number = 10
): Promise<SearchResult[]> {
  try {
    logger.info(`Searching content with query: ${query}`);
    
    const results = await vectorStore.similaritySearchWithScore(query, limit, filters);
    
    return results.map(([doc, score], index) => ({
      id: `result_${index}`,
      content: doc.pageContent,
      metadata: doc.metadata,
      score,
      source: doc.metadata.source || 'unknown',
      type: determineContentType(doc.metadata)
    }));
  } catch (error) {
    logger.error('Error in content search:', error);
    throw error;
  }
}

export async function searchCode(
  query: string,
  filters: { repositories?: string[], language?: string } = {},
  limit: number = 10
): Promise<SearchResult[]> {
  try {
    logger.info(`Searching code with query: ${query}`);
    
    // Build filter for code-specific search
    const codeFilter: Record<string, any> = {
      sourceType: 'repository',
      ...filters
    };
    
    if (filters.language) {
      codeFilter.language = filters.language;
    }
    
    if (filters.repositories && filters.repositories.length > 0) {
      codeFilter.repoUrl = { $in: filters.repositories };
    }
    
    const results = await vectorStore.similaritySearchWithScore(query, limit, codeFilter);
    
    return results.map(([doc, score], index) => ({
      id: `code_${index}`,
      content: doc.pageContent,
      metadata: doc.metadata,
      score,
      source: doc.metadata.repoUrl || doc.metadata.source || 'unknown',
      type: 'code' as const
    }));
  } catch (error) {
    logger.error('Error in code search:', error);
    throw error;
  }
}

export async function searchDocuments(
  query: string,
  filters: { documentTypes?: string[], sources?: string[] } = {},
  limit: number = 10
): Promise<SearchResult[]> {
  try {
    logger.info(`Searching documents with query: ${query}`);
    
    // Build filter for document-specific search
    const docFilter: Record<string, any> = {
      sourceType: 'document'
    };
    
    if (filters.documentTypes && filters.documentTypes.length > 0) {
      docFilter.documentType = { $in: filters.documentTypes };
    }
    
    if (filters.sources && filters.sources.length > 0) {
      docFilter.source = { $in: filters.sources };
    }
    
    const results = await vectorStore.similaritySearchWithScore(query, limit, docFilter);
    
    return results.map(([doc, score], index) => ({
      id: `doc_${index}`,
      content: doc.pageContent,
      metadata: doc.metadata,
      score,
      source: doc.metadata.documentUrl || doc.metadata.source || 'unknown',
      type: 'document' as const
    }));
  } catch (error) {
    logger.error('Error in document search:', error);
    throw error;
  }
}

function determineContentType(metadata: Record<string, any>): 'code' | 'document' | 'web' {
  if (metadata.sourceType === 'repository') {
    return 'code';
  }
  
  if (metadata.sourceType === 'document') {
    return 'document';
  }
  
  if (metadata.sourceType === 'web') {
    return 'web';
  }
  
  // Default based on file extension or other indicators
  const source = metadata.source || '';
  if (source.includes('github.com') || source.includes('.git')) {
    return 'code';
  }
  
  return 'document';
}
