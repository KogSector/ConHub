import BaseConnector from '../BaseConnector.js';
import fs from 'fs/promises';
import path from 'path';
import { glob } from 'glob';

class FilesystemConnector extends BaseConnector {
  constructor(config = {}) {
    super(config);
    this.allowedPaths = [];
    this.workspaceRoot = process.env.WORKSPACE_ROOT || process.cwd();
  }

  async register(core) {
    try {
      // Set allowed paths from config or environment
      this.allowedPaths = this.config.settings?.allowedPaths || [this.workspaceRoot];

      // Register with core MCP service
      await core.registerConnector(this.config.id, {
        name: this.config.name,
        version: this.config.version,
        capabilities: this.config.capabilities,
        metadata: this.config.metadata,
        connector: this
      });

      this.log('info', 'Filesystem connector registered successfully');
      return true;
    } catch (error) {
      this.log('error', 'Failed to register Filesystem connector', { error: error.message });
      throw error;
    }
  }

  async initialize(config = {}) {
    try {
      // Override workspace root if provided
      if (config.workspaceRoot) {
        this.workspaceRoot = config.workspaceRoot;
      }

      // Override allowed paths if provided
      if (config.allowedPaths) {
        this.allowedPaths = config.allowedPaths;
      }

      this.initialized = true;
      this.log('info', 'Filesystem connector initialized', { 
        workspaceRoot: this.workspaceRoot,
        allowedPaths: this.allowedPaths 
      });
      return true;
    } catch (error) {
      this.log('error', 'Failed to initialize Filesystem connector', { error: error.message });
      throw error;
    }
  }

  async healthCheck() {
    try {
      if (!this.initialized) {
        return { status: 'unhealthy', message: 'Connector not initialized' };
      }

      // Test filesystem access
      await fs.access(this.workspaceRoot);
      
      return {
        status: 'healthy',
        timestamp: new Date().toISOString(),
        details: {
          workspaceRoot: this.workspaceRoot,
          allowedPaths: this.allowedPaths,
          accessible: true
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
    // Filesystem connector doesn't require authentication
    return { success: true };
  }

  async fetchData(query) {
    try {
      this.validateInitialized();

      const { type, path: filePath, recursive = false, limit = 100 } = query;

      switch (type) {
        case 'file':
          return await this.getFileInfo(filePath);
        case 'directory':
          return await this.listDirectory(filePath, recursive, limit);
        case 'content':
          return await this.readFile(filePath);
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

      const { 
        limit = 50, 
        fileExtensions, 
        includeContent = false,
        searchPath = this.workspaceRoot 
      } = options;
      
      // Validate search path
      const resolvedPath = this.resolvePath(searchPath);
      this.validatePath(resolvedPath);

      // Build glob pattern
      let pattern = `**/*${query}*`;
      if (fileExtensions && fileExtensions.length > 0) {
        if (fileExtensions.length === 1) {
          pattern = `**/*${query}*.${fileExtensions[0]}`;
        } else {
          pattern = `**/*${query}*.{${fileExtensions.join(',')}}`;
        }
      }

      const files = await glob(pattern, {
        cwd: resolvedPath,
        absolute: true,
        nodir: true,
        ignore: ['**/node_modules/**', '**/.git/**', '**/target/**', '**/build/**']
      });

      const results = [];
      const limitedFiles = files.slice(0, limit);

      for (const file of limitedFiles) {
        try {
          const stats = await fs.stat(file);
          const relativePath = path.relative(this.workspaceRoot, file);
          
          const result = {
            id: file,
            title: path.basename(file),
            type: 'file',
            path: relativePath,
            size: stats.size,
            modifiedTime: stats.mtime.toISOString(),
            source: 'filesystem'
          };

          if (includeContent && this.isTextFile(file) && stats.size < 1024 * 1024) { // 1MB limit
            try {
              result.content = await fs.readFile(file, 'utf8');
            } catch (error) {
              this.log('warn', 'Failed to read file content', { file, error: error.message });
            }
          }

          results.push(result);
        } catch (error) {
          this.log('warn', 'Failed to get file stats', { file, error: error.message });
        }
      }

      return {
        results,
        total: results.length,
        hasMore: files.length > limit
      };
    } catch (error) {
      this.log('error', 'Search failed', { query, options, error: error.message });
      throw error;
    }
  }

  async getContext(resourceId, options = {}) {
    try {
      this.validateInitialized();

      const resolvedPath = this.resolvePath(resourceId);
      this.validatePath(resolvedPath);

      const stats = await fs.stat(resolvedPath);
      
      if (stats.isDirectory()) {
        // Return directory listing
        const files = await this.listDirectory(resolvedPath, false, 50);
        return {
          id: resourceId,
          title: path.basename(resolvedPath),
          content: `Directory containing ${files.length} items`,
          metadata: {
            type: 'directory',
            size: files.length,
            modifiedTime: stats.mtime.toISOString(),
            path: path.relative(this.workspaceRoot, resolvedPath)
          },
          source: 'filesystem'
        };
      }

      // Read file content
      let content = '';
      if (this.isTextFile(resolvedPath) && stats.size < 10 * 1024 * 1024) { // 10MB limit
        content = await fs.readFile(resolvedPath, 'utf8');
      } else {
        content = `Binary file (${this.formatFileSize(stats.size)})`;
      }

      return {
        id: resourceId,
        title: path.basename(resolvedPath),
        content: content,
        metadata: {
          type: 'file',
          size: stats.size,
          modifiedTime: stats.mtime.toISOString(),
          path: path.relative(this.workspaceRoot, resolvedPath),
          extension: path.extname(resolvedPath)
        },
        source: 'filesystem'
      };
    } catch (error) {
      this.log('error', 'Failed to get context', { resourceId, error: error.message });
      throw error;
    }
  }

  async cleanup() {
    try {
      this.initialized = false;
      this.log('info', 'Filesystem connector cleaned up');
    } catch (error) {
      this.log('error', 'Cleanup failed', { error: error.message });
    }
  }

  // Helper methods
  resolvePath(inputPath) {
    if (path.isAbsolute(inputPath)) {
      return inputPath;
    }
    return path.resolve(this.workspaceRoot, inputPath);
  }

  validatePath(targetPath) {
    const normalizedPath = path.normalize(targetPath);
    
    // Check if path is within allowed paths
    const isAllowed = this.allowedPaths.some(allowedPath => {
      const normalizedAllowed = path.normalize(allowedPath);
      return normalizedPath.startsWith(normalizedAllowed);
    });

    if (!isAllowed) {
      throw new Error(`Access denied: Path '${targetPath}' is not within allowed paths`);
    }
  }

  async getFileInfo(filePath) {
    const resolvedPath = this.resolvePath(filePath);
    this.validatePath(resolvedPath);

    const stats = await fs.stat(resolvedPath);
    
    return {
      path: filePath,
      name: path.basename(resolvedPath),
      size: stats.size,
      type: stats.isDirectory() ? 'directory' : 'file',
      modifiedTime: stats.mtime.toISOString(),
      createdTime: stats.birthtime.toISOString(),
      extension: path.extname(resolvedPath)
    };
  }

  async listDirectory(dirPath, recursive = false, limit = 100) {
    const resolvedPath = this.resolvePath(dirPath);
    this.validatePath(resolvedPath);

    const items = [];
    
    if (recursive) {
      const pattern = '**/*';
      const files = await glob(pattern, {
        cwd: resolvedPath,
        absolute: true,
        ignore: ['**/node_modules/**', '**/.git/**']
      });
      
      for (const file of files.slice(0, limit)) {
        try {
          const stats = await fs.stat(file);
          items.push({
            path: path.relative(resolvedPath, file),
            name: path.basename(file),
            size: stats.size,
            type: stats.isDirectory() ? 'directory' : 'file',
            modifiedTime: stats.mtime.toISOString()
          });
        } catch (error) {
          this.log('warn', 'Failed to stat file', { file, error: error.message });
        }
      }
    } else {
      const entries = await fs.readdir(resolvedPath, { withFileTypes: true });
      
      for (const entry of entries.slice(0, limit)) {
        try {
          const fullPath = path.join(resolvedPath, entry.name);
          const stats = await fs.stat(fullPath);
          
          items.push({
            path: entry.name,
            name: entry.name,
            size: stats.size,
            type: entry.isDirectory() ? 'directory' : 'file',
            modifiedTime: stats.mtime.toISOString()
          });
        } catch (error) {
          this.log('warn', 'Failed to stat entry', { entry: entry.name, error: error.message });
        }
      }
    }

    return items;
  }

  async readFile(filePath) {
    const resolvedPath = this.resolvePath(filePath);
    this.validatePath(resolvedPath);

    const stats = await fs.stat(resolvedPath);
    
    if (stats.isDirectory()) {
      throw new Error('Cannot read directory as file');
    }

    if (stats.size > 10 * 1024 * 1024) { // 10MB limit
      throw new Error('File too large to read');
    }

    if (this.isTextFile(resolvedPath)) {
      return await fs.readFile(resolvedPath, 'utf8');
    } else {
      return await fs.readFile(resolvedPath);
    }
  }

  isTextFile(filePath) {
    const textExtensions = [
      '.txt', '.md', '.json', '.js', '.ts', '.jsx', '.tsx', '.py', '.java', 
      '.cpp', '.c', '.h', '.css', '.html', '.xml', '.yml', '.yaml', '.toml',
      '.ini', '.cfg', '.conf', '.log', '.sql', '.sh', '.bat', '.ps1'
    ];
    const ext = path.extname(filePath).toLowerCase();
    return textExtensions.includes(ext);
  }

  formatFileSize(bytes) {
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    if (bytes === 0) return '0 Bytes';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
  }
}

export default FilesystemConnector;