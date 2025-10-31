import BaseConnector from '../BaseConnector.js';
import { google } from 'googleapis';
import fs from 'fs/promises';
import path from 'path';

class GoogleDriveConnector extends BaseConnector {
  constructor() {
    super();
    this.drive = null;
    this.docs = null;
    this.auth = null;
    this.config = null;
  }

  async register(core) {
    try {
      // Load connector configuration
      const configPath = path.join(__dirname, 'config.json');
      const configData = await fs.readFile(configPath, 'utf8');
      this.config = JSON.parse(configData);

      // Register with core MCP service
      await core.registerConnector(this.config.id, {
        name: this.config.name,
        version: this.config.version,
        capabilities: this.config.capabilities,
        metadata: this.config.metadata,
        connector: this
      });

      this.log('info', 'Google Drive connector registered successfully');
      return true;
    } catch (error) {
      this.log('error', 'Failed to register Google Drive connector', { error: error.message });
      throw error;
    }
  }

  async initialize(config = {}) {
    try {
      // Initialize Google OAuth2 client
      this.auth = new google.auth.OAuth2(
        config.clientId || process.env.GOOGLE_CLIENT_ID,
        config.clientSecret || process.env.GOOGLE_CLIENT_SECRET,
        config.redirectUri || process.env.GOOGLE_REDIRECT_URI
      );

      // Set credentials if available
      if (config.accessToken || process.env.GOOGLE_ACCESS_TOKEN) {
        this.auth.setCredentials({
          access_token: config.accessToken || process.env.GOOGLE_ACCESS_TOKEN,
          refresh_token: config.refreshToken || process.env.GOOGLE_REFRESH_TOKEN
        });
      }

      // Initialize Google APIs
      this.drive = google.drive({ version: 'v3', auth: this.auth });
      this.docs = google.docs({ version: 'v1', auth: this.auth });

      this.initialized = true;
      this.log('info', 'Google Drive connector initialized');
      return true;
    } catch (error) {
      this.log('error', 'Failed to initialize Google Drive connector', { error: error.message });
      throw error;
    }
  }

  async healthCheck() {
    try {
      if (!this.initialized) {
        return { status: 'unhealthy', message: 'Connector not initialized' };
      }

      // Test API connectivity
      await this.drive.about.get({ fields: 'user' });
      
      return {
        status: 'healthy',
        timestamp: new Date().toISOString(),
        details: {
          authenticated: true,
          apiConnectivity: true
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
      if (credentials.code) {
        // Exchange authorization code for tokens
        const { tokens } = await this.auth.getToken(credentials.code);
        this.auth.setCredentials(tokens);
        
        return {
          success: true,
          tokens: {
            accessToken: tokens.access_token,
            refreshToken: tokens.refresh_token,
            expiryDate: tokens.expiry_date
          }
        };
      }

      if (credentials.accessToken) {
        // Use provided access token
        this.auth.setCredentials({
          access_token: credentials.accessToken,
          refresh_token: credentials.refreshToken
        });
        
        return { success: true };
      }

      throw new Error('Invalid credentials provided');
    } catch (error) {
      this.log('error', 'Authentication failed', { error: error.message });
      throw error;
    }
  }

  async fetchData(query) {
    try {
      this.validateInitialized();

      const { type, fileId, folderId, searchQuery, limit = 10 } = query;

      switch (type) {
        case 'file':
          return await this.getFile(fileId);
        case 'folder':
          return await this.getFolderContents(folderId, limit);
        case 'search':
          return await this.searchFiles(searchQuery, limit);
        case 'recent':
          return await this.getRecentFiles(limit);
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

      const { limit = 10, mimeType, modifiedTime } = options;
      
      let searchQuery = `name contains '${query}' and trashed=false`;
      
      if (mimeType) {
        searchQuery += ` and mimeType='${mimeType}'`;
      }
      
      if (modifiedTime) {
        searchQuery += ` and modifiedTime > '${modifiedTime}'`;
      }

      const response = await this.drive.files.list({
        q: searchQuery,
        pageSize: limit,
        fields: 'files(id,name,mimeType,size,modifiedTime,webViewLink,thumbnailLink)',
        orderBy: 'modifiedTime desc'
      });

      return {
        results: response.data.files.map(file => ({
          id: file.id,
          title: file.name,
          type: this.getMimeTypeCategory(file.mimeType),
          mimeType: file.mimeType,
          size: file.size,
          modifiedTime: file.modifiedTime,
          url: file.webViewLink,
          thumbnail: file.thumbnailLink,
          source: 'google-drive'
        })),
        total: response.data.files.length,
        hasMore: response.data.files.length === limit
      };
    } catch (error) {
      this.log('error', 'Search failed', { query, options, error: error.message });
      throw error;
    }
  }

  async getContext(resourceId, options = {}) {
    try {
      this.validateInitialized();

      const file = await this.drive.files.get({
        fileId: resourceId,
        fields: 'id,name,mimeType,size,modifiedTime,webViewLink,parents'
      });

      let content = '';
      
      // Extract content based on file type
      if (file.data.mimeType === 'application/vnd.google-apps.document') {
        const doc = await this.docs.documents.get({ documentId: resourceId });
        content = this.extractTextFromDocument(doc.data);
      } else if (file.data.mimeType.startsWith('text/')) {
        const response = await this.drive.files.get({
          fileId: resourceId,
          alt: 'media'
        });
        content = response.data;
      }

      return {
        id: resourceId,
        title: file.data.name,
        content: content,
        metadata: {
          mimeType: file.data.mimeType,
          size: file.data.size,
          modifiedTime: file.data.modifiedTime,
          url: file.data.webViewLink,
          parents: file.data.parents
        },
        source: 'google-drive'
      };
    } catch (error) {
      this.log('error', 'Failed to get context', { resourceId, error: error.message });
      throw error;
    }
  }

  async cleanup() {
    try {
      if (this.auth) {
        this.auth.revokeCredentials();
      }
      
      this.drive = null;
      this.docs = null;
      this.auth = null;
      this.initialized = false;
      
      this.log('info', 'Google Drive connector cleaned up');
    } catch (error) {
      this.log('error', 'Cleanup failed', { error: error.message });
    }
  }

  // Helper methods
  async getFile(fileId) {
    const response = await this.drive.files.get({
      fileId: fileId,
      fields: 'id,name,mimeType,size,modifiedTime,webViewLink,thumbnailLink'
    });

    return {
      id: response.data.id,
      name: response.data.name,
      mimeType: response.data.mimeType,
      size: response.data.size,
      modifiedTime: response.data.modifiedTime,
      url: response.data.webViewLink,
      thumbnail: response.data.thumbnailLink
    };
  }

  async getFolderContents(folderId, limit) {
    const response = await this.drive.files.list({
      q: `'${folderId}' in parents and trashed=false`,
      pageSize: limit,
      fields: 'files(id,name,mimeType,size,modifiedTime,webViewLink)',
      orderBy: 'modifiedTime desc'
    });

    return response.data.files;
  }

  async searchFiles(searchQuery, limit) {
    const response = await this.drive.files.list({
      q: `name contains '${searchQuery}' and trashed=false`,
      pageSize: limit,
      fields: 'files(id,name,mimeType,size,modifiedTime,webViewLink)',
      orderBy: 'modifiedTime desc'
    });

    return response.data.files;
  }

  async getRecentFiles(limit) {
    const response = await this.drive.files.list({
      q: 'trashed=false',
      pageSize: limit,
      fields: 'files(id,name,mimeType,size,modifiedTime,webViewLink)',
      orderBy: 'modifiedTime desc'
    });

    return response.data.files;
  }

  extractTextFromDocument(document) {
    let text = '';
    
    if (document.body && document.body.content) {
      for (const element of document.body.content) {
        if (element.paragraph) {
          for (const textElement of element.paragraph.elements || []) {
            if (textElement.textRun) {
              text += textElement.textRun.content;
            }
          }
        }
      }
    }
    
    return text;
  }

  getMimeTypeCategory(mimeType) {
    if (mimeType.includes('document')) return 'document';
    if (mimeType.includes('spreadsheet')) return 'spreadsheet';
    if (mimeType.includes('presentation')) return 'presentation';
    if (mimeType.includes('pdf')) return 'pdf';
    if (mimeType.includes('text')) return 'text';
    if (mimeType.includes('image')) return 'image';
    return 'file';
  }
}

export default GoogleDriveConnector;