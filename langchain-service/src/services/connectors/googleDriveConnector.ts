import { google } from 'googleapis';
import { ConnectorInterface, DataSource } from '../dataSourceService';
import { logger } from '../../utils/logger';

export class GoogleDriveConnector implements ConnectorInterface {
  private drive?: any;
  private docs?: any;

  async validate(credentials: { clientId: string; clientSecret: string; refreshToken: string }): Promise<boolean> {
    try {
      const auth = new google.auth.OAuth2(credentials.clientId, credentials.clientSecret);
      auth.setCredentials({ refresh_token: credentials.refreshToken });
      
      const drive = google.drive({ version: 'v3', auth });
      await drive.about.get({ fields: 'user' });
      return true;
    } catch (error) {
      logger.error('Google Drive credential validation failed:', error);
      return false;
    }
  }

  async connect(credentials: { clientId: string; clientSecret: string; refreshToken: string }, config: any): Promise<boolean> {
    try {
      const auth = new google.auth.OAuth2(credentials.clientId, credentials.clientSecret);
      auth.setCredentials({ refresh_token: credentials.refreshToken });
      
      this.drive = google.drive({ version: 'v3', auth });
      this.docs = google.docs({ version: 'v1', auth });
      
      // Test connection
      await this.drive.about.get({ fields: 'user' });
      return true;
    } catch (error) {
      logger.error('Google Drive connection failed:', error);
      return false;
    }
  }

  async sync(dataSource: DataSource): Promise<{ documents: any[] }> {
    if (!this.drive || !this.docs) {
      throw new Error('Google Drive not connected');
    }

    const documents: any[] = [];
    const { folderIds, includeShared, fileTypes } = dataSource.config;

    // Get files from specified folders or root
    const foldersToSearch = folderIds && folderIds.length > 0 ? folderIds : ['root'];

    for (const folderId of foldersToSearch) {
      try {
        const files = await this.getFilesFromFolder(folderId, includeShared, fileTypes);
        
        for (const file of files) {
          try {
            let content = '';
            
            // Extract content based on file type
            if (file.mimeType === 'application/vnd.google-apps.document') {
              // Google Docs
              const doc = await this.docs.documents.get({ documentId: file.id });
              content = this.extractTextFromGoogleDoc(doc.data);
            } else if (file.mimeType === 'application/vnd.google-apps.presentation') {
              // Google Slides
              const slides = google.slides({ version: 'v1', auth: this.drive.context._options.auth });
              const presentation = await slides.presentations.get({ presentationId: file.id });
              content = this.extractTextFromGoogleSlides(presentation.data);
            } else if (file.mimeType === 'application/vnd.google-apps.spreadsheet') {
              // Google Sheets
              const sheets = google.sheets({ version: 'v4', auth: this.drive.context._options.auth });
              const spreadsheet = await sheets.spreadsheets.get({ spreadsheetId: file.id });
              content = this.extractTextFromGoogleSheets(spreadsheet.data);
            } else if (file.mimeType === 'text/plain' || file.mimeType === 'text/markdown') {
              // Plain text files
              const response = await this.drive.files.get({ fileId: file.id, alt: 'media' });
              content = response.data;
            } else if (file.mimeType === 'application/pdf') {
              // PDF files - would need additional processing
              content = `[PDF File: ${file.name}]`;
            }

            documents.push({
              id: `gdrive-${file.id}`,
              title: file.name,
              content,
              metadata: {
                source: 'google-drive',
                fileId: file.id,
                mimeType: file.mimeType,
                size: file.size,
                createdTime: file.createdTime,
                modifiedTime: file.modifiedTime,
                owners: file.owners?.map((owner: any) => owner.displayName),
                webViewLink: file.webViewLink,
                parents: file.parents
              }
            });
          } catch (error) {
            logger.warn(`Could not process file ${file.name}:`, error);
          }
        }
      } catch (error) {
        logger.error(`Error syncing folder ${folderId}:`, error);
      }
    }

    return { documents };
  }

  private async getFilesFromFolder(
    folderId: string, 
    includeShared: boolean = false, 
    fileTypes: string[] = []
  ): Promise<any[]> {
    if (!this.drive) return [];

    const files: any[] = [];
    let pageToken: string | undefined;

    do {
      try {
        let query = `'${folderId}' in parents and trashed = false`;
        
        if (fileTypes.length > 0) {
          const mimeTypeQuery = fileTypes.map(type => `mimeType='${type}'`).join(' or ');
          query += ` and (${mimeTypeQuery})`;
        }

        const response = await this.drive.files.list({
          q: query,
          fields: 'nextPageToken, files(id, name, mimeType, size, createdTime, modifiedTime, owners, webViewLink, parents)',
          pageSize: 100,
          pageToken,
          includeItemsFromAllDrives: includeShared,
          supportsAllDrives: includeShared
        });

        files.push(...(response.data.files || []));
        pageToken = response.data.nextPageToken;
      } catch (error) {
        logger.error(`Error listing files from folder ${folderId}:`, error);
        break;
      }
    } while (pageToken);

    return files;
  }

  private extractTextFromGoogleDoc(doc: any): string {
    let text = '';
    
    if (doc.body && doc.body.content) {
      for (const element of doc.body.content) {
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

  private extractTextFromGoogleSlides(presentation: any): string {
    let text = '';
    
    if (presentation.slides) {
      for (const slide of presentation.slides) {
        if (slide.pageElements) {
          for (const element of slide.pageElements) {
            if (element.shape && element.shape.text) {
              for (const textElement of element.shape.text.textElements || []) {
                if (textElement.textRun) {
                  text += textElement.textRun.content;
                }
              }
            }
          }
        }
      }
    }
    
    return text;
  }

  private extractTextFromGoogleSheets(spreadsheet: any): string {
    let text = '';
    
    if (spreadsheet.sheets) {
      for (const sheet of spreadsheet.sheets) {
        text += `Sheet: ${sheet.properties?.title}\n`;
        // Note: This is a simplified extraction. For full sheet data, 
        // you'd need to make additional API calls to get cell values
      }
    }
    
    return text;
  }
}