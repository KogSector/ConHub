import { Client } from '@notionhq/client';
import { ConnectorInterface, DataSource } from '../dataSourceService';
import { logger } from '../../utils/logger';

export class NotionConnector implements ConnectorInterface {
  private notion?: Client;

  async validate(credentials: { apiKey: string }): Promise<boolean> {
    try {
      const notion = new Client({ auth: credentials.apiKey });
      await notion.users.me({});
      return true;
    } catch (error) {
      logger.error('Notion credential validation failed:', error);
      return false;
    }
  }

  async connect(credentials: { apiKey: string }, config: any): Promise<boolean> {
    try {
      this.notion = new Client({ auth: credentials.apiKey });
      await this.notion.users.me({});
      return true;
    } catch (error) {
      logger.error('Notion connection failed:', error);
      return false;
    }
  }

  async sync(dataSource: DataSource): Promise<{ documents: any[] }> {
    if (!this.notion) {
      throw new Error('Notion not connected');
    }

    const documents: any[] = [];
    const { databaseIds, pageIds, includeSubpages } = dataSource.config;

    // Sync databases
    if (databaseIds && databaseIds.length > 0) {
      for (const databaseId of databaseIds) {
        try {
          const database = await this.notion.databases.retrieve({ database_id: databaseId });
          const pages = await this.getDatabasePages(databaseId);
          
          for (const page of pages) {
            const content = await this.getPageContent(page.id);
            
            documents.push({
              id: `notion-page-${page.id}`,
              title: this.getPageTitle(page),
              content,
              metadata: {
                source: 'notion',
                type: 'database_page',
                pageId: page.id,
                databaseId,
                databaseTitle: this.getDatabaseTitle(database),
                createdTime: page.created_time,
                lastEditedTime: page.last_edited_time,
                url: page.url,
                properties: page.properties
              }
            });
          }
        } catch (error) {
          logger.error(`Error syncing database ${databaseId}:`, error);
        }
      }
    }

    // Sync individual pages
    if (pageIds && pageIds.length > 0) {
      for (const pageId of pageIds) {
        try {
          const page = await this.notion.pages.retrieve({ page_id: pageId });
          const content = await this.getPageContent(pageId);
          
          documents.push({
            id: `notion-page-${pageId}`,
            title: this.getPageTitle(page),
            content,
            metadata: {
              source: 'notion',
              type: 'page',
              pageId,
              createdTime: page.created_time,
              lastEditedTime: page.last_edited_time,
              url: page.url
            }
          });

          // Get subpages if requested
          if (includeSubpages) {
            const subpages = await this.getSubpages(pageId);
            for (const subpage of subpages) {
              const subpageContent = await this.getPageContent(subpage.id);
              
              documents.push({
                id: `notion-page-${subpage.id}`,
                title: this.getPageTitle(subpage),
                content: subpageContent,
                metadata: {
                  source: 'notion',
                  type: 'subpage',
                  pageId: subpage.id,
                  parentPageId: pageId,
                  createdTime: subpage.created_time,
                  lastEditedTime: subpage.last_edited_time,
                  url: subpage.url
                }
              });
            }
          }
        } catch (error) {
          logger.error(`Error syncing page ${pageId}:`, error);
        }
      }
    }

    return { documents };
  }

  private async getDatabasePages(databaseId: string): Promise<any[]> {
    if (!this.notion) return [];

    const pages: any[] = [];
    let cursor: string | undefined;

    do {
      try {
        const response = await this.notion.databases.query({
          database_id: databaseId,
          start_cursor: cursor,
          page_size: 100
        });

        pages.push(...response.results);
        cursor = response.next_cursor || undefined;
      } catch (error) {
        logger.error(`Error querying database ${databaseId}:`, error);
        break;
      }
    } while (cursor);

    return pages;
  }

  private async getSubpages(pageId: string): Promise<any[]> {
    if (!this.notion) return [];

    try {
      const response = await this.notion.blocks.children.list({
        block_id: pageId,
        page_size: 100
      });

      return response.results.filter((block: any) => block.type === 'child_page');
    } catch (error) {
      logger.error(`Error getting subpages for ${pageId}:`, error);
      return [];
    }
  }

  private async getPageContent(pageId: string): Promise<string> {
    if (!this.notion) return '';

    try {
      const blocks = await this.getPageBlocks(pageId);
      return this.blocksToText(blocks);
    } catch (error) {
      logger.error(`Error getting content for page ${pageId}:`, error);
      return '';
    }
  }

  private async getPageBlocks(pageId: string): Promise<any[]> {
    if (!this.notion) return [];

    const blocks: any[] = [];
    let cursor: string | undefined;

    do {
      try {
        const response = await this.notion.blocks.children.list({
          block_id: pageId,
          start_cursor: cursor,
          page_size: 100
        });

        blocks.push(...response.results);
        cursor = response.next_cursor || undefined;
      } catch (error) {
        logger.error(`Error getting blocks for page ${pageId}:`, error);
        break;
      }
    } while (cursor);

    return blocks;
  }

  private blocksToText(blocks: any[]): string {
    let text = '';

    for (const block of blocks) {
      switch (block.type) {
        case 'paragraph':
          text += this.richTextToPlainText(block.paragraph?.rich_text || []) + '\n';
          break;
        case 'heading_1':
          text += '# ' + this.richTextToPlainText(block.heading_1?.rich_text || []) + '\n';
          break;
        case 'heading_2':
          text += '## ' + this.richTextToPlainText(block.heading_2?.rich_text || []) + '\n';
          break;
        case 'heading_3':
          text += '### ' + this.richTextToPlainText(block.heading_3?.rich_text || []) + '\n';
          break;
        case 'bulleted_list_item':
          text += '• ' + this.richTextToPlainText(block.bulleted_list_item?.rich_text || []) + '\n';
          break;
        case 'numbered_list_item':
          text += '1. ' + this.richTextToPlainText(block.numbered_list_item?.rich_text || []) + '\n';
          break;
        case 'to_do':
          const checked = block.to_do?.checked ? '[x]' : '[ ]';
          text += `${checked} ${this.richTextToPlainText(block.to_do?.rich_text || [])}\n`;
          break;
        case 'code':
          text += '```\n' + this.richTextToPlainText(block.code?.rich_text || []) + '\n```\n';
          break;
        case 'quote':
          text += '> ' + this.richTextToPlainText(block.quote?.rich_text || []) + '\n';
          break;
      }
    }

    return text;
  }

  private richTextToPlainText(richText: any[]): string {
    return richText.map(text => text.plain_text || '').join('');
  }

  private getPageTitle(page: any): string {
    if (page.properties) {
      // For database pages, find the title property
      for (const [key, property] of Object.entries(page.properties)) {
        if ((property as any).type === 'title') {
          return this.richTextToPlainText((property as any).title || []);
        }
      }
    }
    
    // For regular pages, use the page title
    if (page.title) {
      return this.richTextToPlainText(page.title);
    }
    
    return 'Untitled';
  }

  private getDatabaseTitle(database: any): string {
    if (database.title) {
      return this.richTextToPlainText(database.title);
    }
    return 'Untitled Database';
  }
}