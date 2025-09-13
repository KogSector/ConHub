import { v4 as uuidv4 } from 'uuid';
import { logger } from '../utils/logger';

interface DataSource {
  id: string;
  type: string;
  name: string;
  status: 'connected' | 'disconnected' | 'syncing' | 'error';
  config: any;
  createdAt: string;
  lastSyncAt?: string;
}

// In-memory store for data sources (replace with database in production)
const dataSources = new Map<string, DataSource>();

export async function connectDataSource(
  type: string,
  credentials: any,
  config: any
): Promise<DataSource> {
  const id = uuidv4();
  
  const dataSource: DataSource = {
    id,
    type,
    name: config.name || `${type}-${id.slice(0, 8)}`,
    status: 'connected',
    config: { ...config, credentials },
    createdAt: new Date().toISOString(),
  };
  
  dataSources.set(id, dataSource);
  
  logger.info(`Connected data source: ${type} (${id})`);
  
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

export async function syncDataSource(sourceId: string): Promise<{ syncId: string }> {
  const dataSource = dataSources.get(sourceId);
  if (!dataSource) {
    throw new Error('Data source not found');
  }
  
  dataSource.status = 'syncing';
  dataSource.lastSyncAt = new Date().toISOString();
  dataSources.set(sourceId, dataSource);
  
  const syncId = uuidv4();
  
  // Simulate async sync process
  setTimeout(() => {
    const ds = dataSources.get(sourceId);
    if (ds) {
      ds.status = 'connected';
      dataSources.set(sourceId, ds);
    }
  }, 5000);
  
  logger.info(`Started sync for data source: ${sourceId}`);
  
  return { syncId };
}