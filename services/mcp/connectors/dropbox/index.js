import BaseConnector from '../BaseConnector.js';
import axios from 'axios';
import fs from 'fs/promises';
import path from 'path';

class DropboxConnector extends BaseConnector {
  constructor(config = {}) {
    super(config);
    this.accessToken = null;
    this.apiBaseUrl = 'https://api.dropboxapi.com/2';
    this.contentBaseUrl = 'https://content.dropboxapi.com/2';
  }

  async register(core) {
    try {
      // Register with core MCP service
      await core.registerConnector(this.config.id, {
        name: this.config.name,
        version: this.config.version,
        capabilities: this.config.capabilities,
        metadata: this.config.metadata,
        connector: this
      });

      this.log('info', 'Dropbox connector registered successfully');
      return true;
    } catch (error) {
      this.log('error', 'Failed to register Dropbox connector', { error: error.message });
      throw error;
    }
  }

  async initialize(config = {}) {
    try {
      this.accessToken = config.accessToken || process.env.DROPBOX_ACCESS_TOKEN;
      
      if (!this.accessToken) {
        throw new Error('Dropbox access token is required');
      }

      this.initialized = true;
      this.log('info', 'Dropbox connector initialized');
      return true;
    } catch (error) {
      this.log('error', 'Failed to initialize Dropbox connector', { error: error.message });
      throw error;
    }
  }

  async healthCheck() {
    try {
      if (!this.initialized) {
        return { status: 'unhealthy', message: 'Connector not initialized' };
      }

      // Test API connectivity
      const response = await this.makeApiRequest('/users/get_current_account', {}, 'POST');
      
      return {
        status: 'healthy',
        timestamp: new Date().toISOString(),
        details: {
          authenticated: true,
          apiConnectivity: true,
          accountId: response.account_id
        }
      };
    } catch (error) {
      return {
        status: 'unhealthy',
        message: error.message,
        timestamp: new Date().toISOString()
      };
    }
  }

  async authenticate(credentials) {
    try {
      if (credentials.accessToken) {
        this.accessToken = credentials.accessToken;
        return { success: true };
      }

      throw new Error('Access token is required for Dropbox authentication');
    } catch (error) {
      this.log('error', 'Authentication failed', { error: error.message });
      throw error;
    }
  }

  async fetchData(query) {
    try {
      this.validateInitialized();

      const { type, path: filePath, query: searchQuery, limit = 10 } = query;

      switch (type) {
        case 'file':
          return await this.getFileMetadata(filePath);
        case 'folder':
          return await this.listFolder(filePath, limit);
        case 'search':
          return await this.searchFiles(searchQuery, limit);
        default:
          throw new Error(`Unsupported query type: ${type}`);
      }
    } catch (error) {
      this.log('error', 'Failed to fetch data', { query, error: error.message });
      throw error;
    }
  }

  async search(query, options = {}) {
    try {
      this.validateInitialized();

      const { limit = 10, fileCategories, fileExtensions } = options;
      
      const searchOptions = {
        query: query,
        options: {
          max_results: limit,
          path: options.path || '',
          file_status: 'active'
        }
      };

      if (fileCategories) {
        searchOptions.options.file_categories = fileCategories;
      }

      if (fileExtensions) {
        searchOptions.options.file_extensions = fileExtensions;
      }

      const response = await this.makeApiRequest('/files/search_v2', searchOptions, 'POST');

      return {
        results: response.matches.map(match => ({
          id: match.metadata.metadata.id,
          title: match.metadata.metadata.name,
          type: match.metadata.metadata['.tag'],
          path: match.metadata.metadata.path_display,
          size: match.metadata.metadata.size,
          modifiedTime: match.metadata.metadata.client_modified,
          source: 'dropbox'
        })),
        total: response.matches.length,
        hasMore: response.has_more
      };
    } catch (error) {
      this.log('error', 'Search failed', { query, options, error: error.message });
      throw error;
    }
  }

  async getContext(resourceId, options = {}) {
    try {
      this.validateInitialized();

      // Get file metadata
      const metadata = await this.getFileMetadata(resourceId);
      
      let content = '';
      
      // Download file content if it's a text file and small enough
      if (metadata.size < 1024 * 1024 && this.isTextFile(metadata.name)) { // 1MB limit
        try {
          content = await this.downloadFile(resourceId);
        } catch (error) {
          this.log('warn', 'Failed to download file content', { resourceId, error: error.message });
        }
      }

      return {
        id: resourceId,
        title: metadata.name,
        content: content,
        metadata: {
          size: metadata.size,
          modifiedTime: metadata.client_modified,
          path: metadata.path_display,
          type: metadata['.tag']
        },
        source: 'dropbox'
      };
    } catch (error) {
      this.log('error', 'Failed to get context', { resourceId, error: error.message });
      throw error;
    }
  }

  async cleanup() {
    try {
      this.accessToken = null;
      this.initialized = false;
      
      this.log('info', 'Dropbox connector cleaned up');
    } catch (error) {
      this.log('error', 'Cleanup failed', { error: error.message });
    }
  }

  // Helper methods
  async makeApiRequest(endpoint, data = {}, method = 'POST') {
    const url = `${this.apiBaseUrl}${endpoint}`;
    
    const config = {
      method,
      url,
      headers: {
        'Authorization': `Bearer ${this.accessToken}`,
        'Content-Type': 'application/json'
      }
    };

    if (method === 'POST' && Object.keys(data).length > 0) {
      config.data = data;
    }

    const response = await axios(config);
    return response.data;
  }

  async getFileMetadata(filePath) {
    return await this.makeApiRequest('/files/get_metadata', {
      path: filePath,
      include_media_info: false,
      include_deleted: false,
      include_has_explicit_shared_members: false
    }, 'POST');
  }

  async listFolder(folderPath, limit) {
    const response = await this.makeApiRequest('/files/list_folder', {
      path: folderPath || '',
      recursive: false,
      include_media_info: false,
      include_deleted: false,
      include_has_explicit_shared_members: false,
      include_mounted_folders: true,
      limit: limit
    }, 'POST');

    return response.entries;
  }

  async searchFiles(query, limit) {
    const response = await this.makeApiRequest('/files/search_v2', {
      query: query,
      options: {
        max_results: limit,
        path: '',
        file_status: 'active'
      }
    }, 'POST');

    return response.matches.map(match => match.metadata.metadata);
  }

  async downloadFile(filePath) {
    const url = `${this.contentBaseUrl}/files/download`;
    
    const response = await axios({
      method: 'POST',
      url,
      headers: {
        'Authorization': `Bearer ${this.accessToken}`,
        'Dropbox-API-Arg': JSON.stringify({ path: filePath })
      }
    });

    return response.data;
  }

  isTextFile(filename) {
    const textExtensions = ['.txt', '.md', '.json', '.js', '.ts', '.py', '.java', '.cpp', '.c', '.h', '.css', '.html', '.xml', '.yml', '.yaml'];
    const ext = path.extname(filename).toLowerCase();
    return textExtensions.includes(ext);
  }
}

export default DropboxConnector;