import { v4 as uuidv4 } from 'uuid';
import { logger } from '../utils/logger';
import { GitHubConnector } from './connectors/githubConnector';
import { BitBucketConnector } from './connectors/bitbucketConnector';
import { GoogleDriveConnector } from './connectors/googleDriveConnector';
import { NotionConnector } from './connectors/notionConnector';
import { URLConnector } from './connectors/urlConnector';
import { indexingService } from './indexingService';

export interface DataSource {
  id: string;
  type: 'github' | 'bitbucket' | 'google-drive' | 'notion' | 'url' | 'local';
  name: string;
  status: 'connected' | 'disconnected' | 'syncing' | 'error' | 'indexing';
  config: any;
  credentials?: any;
  createdAt: string;
  lastSyncAt?: string;
  indexedCount?: number;
  totalCount?: number;
  error?: string;
}

export interface ConnectorInterface {
  connect(credentials: any, config: any): Promise<boolean>;
  sync(dataSource: DataSource): Promise<{ documents: any[], repositories?: any[], urls?: any[] }>;
  validate(credentials: any): Promise<boolean>;
}

// In-memory store for data sources (replace with database in production)
const dataSources = new Map<string, DataSource>();
const connectors = new Map<string, ConnectorInterface>();

// Initialize connectors
connectors.set('github', new GitHubConnector());
connectors.set('bitbucket', new BitBucketConnector());
connectors.set('google-drive', new GoogleDriveConnector());
connectors.set('notion', new NotionConnector());
connectors.set('url', new URLConnector());

export async function connectDataSource(
  type: string,
  credentials: any,
  config: any
): Promise<DataSource> {
  const connector = connectors.get(type);
  if (!connector) {
    throw new Error(`Unsupported data source type: ${type}`);
  }

  // Validate credentials
  const isValid = await connector.validate(credentials);
  if (!isValid) {
    throw new Error('Invalid credentials provided');
  }

  // Connect to the data source
  const connected = await connector.connect(credentials, config);
  if (!connected) {
    throw new Error('Failed to connect to data source');
  }

  const id = uuidv4();
  const dataSource: DataSource = {
    id,
    type: type as DataSource['type'],
    name: config.name || `${type}-${id.slice(0, 8)}`,
    status: 'connected',
    config,
    credentials,
    createdAt: new Date().toISOString(),
    indexedCount: 0,
    totalCount: 0
  };
  
  dataSources.set(id, dataSource);
  logger.info(`Connected data source: ${type} (${id})`);
  
  // Start initial indexing
  syncDataSource(id).catch(error => {
    logger.error(`Initial sync failed for ${id}:`, error);
    const ds = dataSources.get(id);
    if (ds) {
      ds.status = 'error';
      ds.error = error.message;
      dataSources.set(id, ds);
    }
  });
  
  return dataSource;
}

export async function disconnectDataSource(sourceId: string): Promise<void> {
  const dataSource = dataSources.get(sourceId);
  if (!dataSource) {
    throw new Error('Data source not found');
  }
  
  dataSource.status = 'disconnected';
  dataSources.set(sourceId, dataSource);
  
  logger.info(`Disconnected data source: ${sourceId}`);
}

export async function getDataSources(): Promise<DataSource[]> {
  return Array.from(dataSources.values());
}

export async function getDataSource(sourceId: string): Promise<DataSource | undefined> {
  return dataSources.get(sourceId);
}

export async function syncDataSource(sourceId: string): Promise<{ syncId: string }> {
  const dataSource = dataSources.get(sourceId);
  if (!dataSource) {
    throw new Error('Data source not found');
  }
  
  const connector = connectors.get(dataSource.type);
  if (!connector) {
    throw new Error(`No connector found for type: ${dataSource.type}`);
  }

  dataSource.status = 'syncing';
  dataSource.lastSyncAt = new Date().toISOString();
  dataSources.set(sourceId, dataSource);
  
  const syncId = uuidv4();
  
  // Perform async sync
  (async () => {
    try {
      logger.info(`Starting sync for data source: ${sourceId}`);
      
      // Get data from connector
      const syncResult = await connector.sync(dataSource);
      
      // Update status to indexing
      dataSource.status = 'indexing';
      dataSource.totalCount = syncResult.documents.length;
      dataSources.set(sourceId, dataSource);
      
      // Index documents
      let indexedCount = 0;
      for (const doc of syncResult.documents) {
        await indexingService.indexDocument({
          ...doc,
          sourceId,
          sourceType: dataSource.type
        });
        indexedCount++;
        
        // Update progress
        dataSource.indexedCount = indexedCount;
        dataSources.set(sourceId, dataSource);
      }
      
      // Mark as completed
      dataSource.status = 'connected';
      dataSource.error = undefined;
      dataSources.set(sourceId, dataSource);
      
      logger.info(`Completed sync for data source: ${sourceId} (${indexedCount} documents indexed)`);
    } catch (error) {
      logger.error(`Sync failed for data source ${sourceId}:`, error);
      dataSource.status = 'error';
      dataSource.error = error instanceof Error ? error.message : 'Unknown error';
      dataSources.set(sourceId, dataSource);
    }
  })();
  
  return { syncId };
}

export async function deleteDataSource(sourceId: string): Promise<void> {
  const dataSource = dataSources.get(sourceId);
  if (!dataSource) {
    throw new Error('Data source not found');
  }
  
  // Remove from indexing service
  await indexingService.removeDocumentsBySource(sourceId);
  
  // Remove from memory
  dataSources.delete(sourceId);
  
  logger.info(`Deleted data source: ${sourceId}`);
}

export async function updateDataSource(
  sourceId: string, 
  updates: Partial<Pick<DataSource, 'name' | 'config'>>
): Promise<DataSource> {
  const dataSource = dataSources.get(sourceId);
  if (!dataSource) {
    throw new Error('Data source not found');
  }
  
  Object.assign(dataSource, updates);
  dataSources.set(sourceId, dataSource);
  
  logger.info(`Updated data source: ${sourceId}`);
  return dataSource;
}