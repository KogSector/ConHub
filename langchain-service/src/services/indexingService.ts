import { Document } from 'langchain/document';
import { RecursiveCharacterTextSplitter } from 'langchain/text_splitter';
import { GithubRepoLoader } from '@langchain/community/document_loaders/web/github';
import { TextLoader } from 'langchain/document_loaders/fs/text';
import { vectorStore } from './vectorStore';
import { logger } from '../utils/logger';
import { v4 as uuidv4 } from 'uuid';

export interface IndexingResult {
  indexId: string;
  status: 'started' | 'processing' | 'completed' | 'failed';
  documentCount?: number;
  error?: string;
}

export interface DocumentToIndex {
  id: string;
  title: string;
  content: string;
  metadata: Record<string, any>;
  sourceId?: string;
  sourceType?: string;
}

// Store for tracking indexing jobs and documents
const indexingJobs = new Map<string, IndexingResult>();
const indexedDocuments = new Map<string, DocumentToIndex>();

// Text splitter for chunking documents
const textSplitter = new RecursiveCharacterTextSplitter({
  chunkSize: 1000,
  chunkOverlap: 200,
});

class IndexingService {
  async indexDocument(doc: DocumentToIndex): Promise<string> {
    const chunks = await textSplitter.splitText(doc.content);
    const documents = chunks.map((chunk, index) => new Document({
      pageContent: chunk,
      metadata: {
        ...doc.metadata,
        documentId: doc.id,
        chunkIndex: index,
        title: doc.title,
        sourceId: doc.sourceId,
        sourceType: doc.sourceType,
        indexedAt: new Date().toISOString()
      }
    }));
    
    await vectorStore.addDocuments(documents);
    indexedDocuments.set(doc.id, doc);
    
    return doc.id;
  }

  async removeDocumentsBySource(sourceId: string): Promise<void> {
    const docsToRemove = Array.from(indexedDocuments.values())
      .filter(doc => doc.sourceId === sourceId);
    
    for (const doc of docsToRemove) {
      indexedDocuments.delete(doc.id);
    }
    
    logger.info(`Removed ${docsToRemove.length} documents for source ${sourceId}`);
  }

  async getDocumentsBySource(sourceId: string): Promise<DocumentToIndex[]> {
    return Array.from(indexedDocuments.values())
      .filter(doc => doc.sourceId === sourceId);
  }
}

export const indexingService = new IndexingService();

export async function indexRepository(
  repoUrl: string, 
  accessToken?: string
): Promise<IndexingResult> {
  const indexId = uuidv4();
  
  const job: IndexingResult = {
    indexId,
    status: 'started'
  };
  
  indexingJobs.set(indexId, job);
  
  try {
    logger.info(`Starting repository indexing: ${repoUrl}`);
    
    // Update status
    job.status = 'processing';
    indexingJobs.set(indexId, job);
    
    // Load documents from GitHub repository
    const loader = new GithubRepoLoader(repoUrl, {
      branch: 'main',
      recursive: true,
      unknown: 'warn',
      accessToken: accessToken
    });
    
    const docs = await loader.load();
    logger.info(`Loaded ${docs.length} documents from repository`);
    
    // Split documents into chunks
    const splitDocs = await textSplitter.splitDocuments(docs);
    logger.info(`Split into ${splitDocs.length} chunks`);
    
    // Add metadata
    const enrichedDocs = splitDocs.map(doc => ({
      ...doc,
      metadata: {
        ...doc.metadata,
        sourceType: 'repository',
        repoUrl,
        indexedAt: new Date().toISOString()
      }
    }));
    
    // Store in vector database
    await vectorStore.addDocuments(enrichedDocs);
    
    // Update job status
    job.status = 'completed';
    job.documentCount = enrichedDocs.length;
    indexingJobs.set(indexId, job);
    
    logger.info(`Successfully indexed repository: ${repoUrl}`);
    
    return job;
  } catch (error) {
    logger.error(`Error indexing repository ${repoUrl}:`, error);
    
    job.status = 'failed';
    job.error = error instanceof Error ? error.message : 'Unknown error';
    indexingJobs.set(indexId, job);
    
    throw error;
  }
}

export async function indexDocument(
  documentUrl: string,
  type: string = 'text',
  metadata: Record<string, any> = {}
): Promise<IndexingResult> {
  const indexId = uuidv4();
  
  const job: IndexingResult = {
    indexId,
    status: 'started'
  };
  
  indexingJobs.set(indexId, job);
  
  try {
    logger.info(`Starting document indexing: ${documentUrl}`);
    
    job.status = 'processing';
    indexingJobs.set(indexId, job);
    
    let loader;
    
    // Choose loader based on document type
    switch (type.toLowerCase()) {
      case 'pdf':
        // PDF support would require additional setup
        throw new Error('PDF support not yet implemented. Please use text files.');
      case 'text':
      case 'txt':
      case 'md':
      case 'js':
      case 'ts':
      case 'py':
      default:
        loader = new TextLoader(documentUrl);
        break;
    }
    
    const docs = await loader.load();
    logger.info(`Loaded ${docs.length} documents`);
    
    // Split documents
    const splitDocs = await textSplitter.splitDocuments(docs);
    
    // Add metadata
    const enrichedDocs = splitDocs.map(doc => ({
      ...doc,
      metadata: {
        ...doc.metadata,
        ...metadata,
        sourceType: 'document',
        documentUrl,
        documentType: type,
        indexedAt: new Date().toISOString()
      }
    }));
    
    // Store in vector database
    await vectorStore.addDocuments(enrichedDocs);
    
    job.status = 'completed';
    job.documentCount = enrichedDocs.length;
    indexingJobs.set(indexId, job);
    
    logger.info(`Successfully indexed document: ${documentUrl}`);
    
    return job;
  } catch (error) {
    logger.error(`Error indexing document ${documentUrl}:`, error);
    
    job.status = 'failed';
    job.error = error instanceof Error ? error.message : 'Unknown error';
    indexingJobs.set(indexId, job);
    
    throw error;
  }
}

export async function getIndexingStatus(indexId: string): Promise<IndexingResult | null> {
  return indexingJobs.get(indexId) || null;
}
