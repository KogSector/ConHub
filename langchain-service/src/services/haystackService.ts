import axios from 'axios';
import { logger } from '../utils/logger';

export interface HaystackDocument {
  content: string;
  meta?: Record<string, any>;
  id?: string;
}

export interface HaystackSearchResult {
  content: string;
  score?: number;
  meta?: Record<string, any>;
  answer?: string;
  context?: string;
}

export interface HaystackSearchResponse {
  success: boolean;
  query: string;
  results: HaystackSearchResult[];
  total_count: number;
  processing_time?: number;
}

export interface HaystackIndexResponse {
  success: boolean;
  message: string;
  document_count: number;
  index_id?: string;
}

class HaystackService {
  private baseUrl: string;

  constructor() {
    this.baseUrl = process.env.HAYSTACK_SERVICE_URL || 'http://localhost:8001';
  }

  async indexDocuments(documents: HaystackDocument[]): Promise<HaystackIndexResponse> {
    try {
      const response = await axios.post(`${this.baseUrl}/documents`, {
        documents
      });
      
      logger.info(`Successfully indexed ${documents.length} documents in Haystack`);
      return response.data;
    } catch (error) {
      logger.error('Error indexing documents in Haystack:', error);
      throw new Error(`Haystack indexing failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async uploadAndIndexFile(file: Buffer, filename: string, metadata?: Record<string, any>): Promise<HaystackIndexResponse> {
    try {
      const formData = new FormData();
      const blob = new Blob([new Uint8Array(file)]);
      formData.append('file', blob, filename);
      
      if (metadata) {
        formData.append('metadata', JSON.stringify(metadata));
      }

      const response = await axios.post(`${this.baseUrl}/documents/upload`, formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
      });
      
      logger.info(`Successfully uploaded and indexed file: ${filename}`);
      return response.data;
    } catch (error) {
      logger.error('Error uploading file to Haystack:', error);
      throw new Error(`Haystack file upload failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async searchDocuments(
    query: string, 
    topK: number = 10, 
    filters?: Record<string, any>
  ): Promise<HaystackSearchResponse> {
    try {
      const response = await axios.post(`${this.baseUrl}/search`, {
        query,
        top_k: topK,
        filters: filters || {}
      });
      
      logger.info(`Haystack search returned ${response.data.total_count} results for: ${query}`);
      return response.data;
    } catch (error) {
      logger.error('Error searching in Haystack:', error);
      throw new Error(`Haystack search failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async askQuestion(
    question: string, 
    topK: number = 10, 
    filters?: Record<string, any>
  ): Promise<HaystackSearchResponse> {
    try {
      const response = await axios.post(`${this.baseUrl}/ask`, {
        query: question,
        top_k: topK,
        filters: filters || {}
      });
      
      logger.info(`Haystack Q&A returned ${response.data.total_count} answers for: ${question}`);
      return response.data;
    } catch (error) {
      logger.error('Error asking question in Haystack:', error);
      throw new Error(`Haystack Q&A failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async getStats(): Promise<any> {
    try {
      const response = await axios.get(`${this.baseUrl}/stats`);
      return response.data;
    } catch (error) {
      logger.error('Error getting Haystack stats:', error);
      throw new Error(`Haystack stats failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async healthCheck(): Promise<boolean> {
    try {
      const response = await axios.get(`${this.baseUrl}/health`);
      return response.data.status === 'healthy';
    } catch (error) {
      logger.error('Haystack health check failed:', error);
      return false;
    }
  }
}

export const haystackService = new HaystackService();
