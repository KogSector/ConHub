import { OpenAIEmbeddings } from '@langchain/openai';
import { HuggingFaceTransformersEmbeddings } from '@langchain/community/embeddings/hf_transformers';
import { MemoryVectorStore } from 'langchain/vectorstores/memory';
import { QdrantVectorStore } from '@langchain/qdrant';
import { PineconeStore } from '@langchain/pinecone';
import { Document } from 'langchain/document';
import { logger } from '../utils/logger';

// Initialize embeddings with fallback to local model
let embeddings: OpenAIEmbeddings | HuggingFaceTransformersEmbeddings;

try {
  if (process.env.OPENAI_API_KEY && process.env.OPENAI_API_KEY !== 'sk-your-openai-api-key-here') {
    embeddings = new OpenAIEmbeddings({
      openAIApiKey: process.env.OPENAI_API_KEY,
    });
    logger.info('Using OpenAI embeddings');
  } else {
    // Fallback to local HuggingFace embeddings
    embeddings = new HuggingFaceTransformersEmbeddings({
      modelName: "Xenova/all-MiniLM-L6-v2",
    });
    logger.info('Using local HuggingFace embeddings');
  }
} catch (error) {
  logger.warn('Failed to initialize embeddings, falling back to local model:', error);
  embeddings = new HuggingFaceTransformersEmbeddings({
    modelName: "Xenova/all-MiniLM-L6-v2",
  });
}

// Vector store instance (will be initialized based on configuration)
let vectorStoreInstance: QdrantVectorStore | PineconeStore | MemoryVectorStore | null = null;

async function initializeVectorStore() {
  if (vectorStoreInstance) {
    return vectorStoreInstance;
  }

  try {
    // Try Qdrant first (if configured)
    if (process.env.QDRANT_URL) {
      logger.info('Initializing Qdrant vector store');
      
      const { QdrantClient } = await import('@qdrant/js-client-rest');
      
      const client = new QdrantClient({
        url: process.env.QDRANT_URL,
        apiKey: process.env.QDRANT_API_KEY,
      });

      vectorStoreInstance = await QdrantVectorStore.fromExistingCollection(
        embeddings,
        {
          client,
          collectionName: 'conhub-documents',
        }
      );
      
      logger.info('Successfully initialized Qdrant vector store');
      return vectorStoreInstance;
    }
    
    // Fallback to Pinecone (if configured)
    if (process.env.PINECONE_API_KEY) {
      logger.info('Initializing Pinecone vector store');
      
      const { Pinecone } = await import('@pinecone-database/pinecone');
      
      const pinecone = new Pinecone({
        apiKey: process.env.PINECONE_API_KEY,
      });

      const index = pinecone.Index(process.env.PINECONE_INDEX_NAME || 'conhub-index');

      vectorStoreInstance = await PineconeStore.fromExistingIndex(embeddings, {
        pineconeIndex: index as any, // Type assertion to handle version compatibility
      });
      
      logger.info('Successfully initialized Pinecone vector store');
      return vectorStoreInstance;
    }
    
    // Fallback to in-memory vector store for development
    logger.info('No external vector store configured, using in-memory vector store');
    vectorStoreInstance = new MemoryVectorStore(embeddings);
    logger.info('Successfully initialized in-memory vector store');
    return vectorStoreInstance;
    
  } catch (error) {
    logger.error('Failed to initialize vector store, falling back to in-memory:', error);
    vectorStoreInstance = new MemoryVectorStore(embeddings);
    return vectorStoreInstance;
  }
}

export const vectorStore = {
  async addDocuments(documents: Document[]) {
    const store = await initializeVectorStore();
    if (!store) {
      throw new Error('Vector store not initialized');
    }
    return store.addDocuments(documents);
  },

  async similaritySearch(query: string, k: number = 10, filter?: Record<string, any>) {
    const store = await initializeVectorStore();
    if (!store) {
      throw new Error('Vector store not initialized');
    }
    // Different vector stores handle filters differently
    if (store instanceof MemoryVectorStore) {
      return store.similaritySearch(query, k);
    }
    return store.similaritySearch(query, k, filter as any);
  },

  async similaritySearchWithScore(query: string, k: number = 10, filter?: Record<string, any>) {
    const store = await initializeVectorStore();
    if (!store) {
      throw new Error('Vector store not initialized');
    }
    // Different vector stores handle filters differently
    if (store instanceof MemoryVectorStore) {
      return store.similaritySearchWithScore(query, k);
    }
    return store.similaritySearchWithScore(query, k, filter as any);
  },

  async delete(ids: string[]) {
    const store = await initializeVectorStore();
    if (!store) {
      throw new Error('Vector store not initialized');
    }
    // Implementation depends on the vector store
    // This is a placeholder - actual implementation may vary
    logger.warn('Delete operation not implemented for current vector store');
  }
};
