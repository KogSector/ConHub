import axios from 'axios';
import * as cheerio from 'cheerio';
import { ConnectorInterface, DataSource } from '../dataSourceService';
import { logger } from '../../utils/logger';

export class URLConnector implements ConnectorInterface {
  async validate(credentials: any): Promise<boolean> {
    // URL connector doesn't require credentials
    return true;
  }

  async connect(credentials: any, config: any): Promise<boolean> {
    // Test connectivity by trying to fetch one URL
    if (config.urls && config.urls.length > 0) {
      try {
        await axios.get(config.urls[0], { timeout: 10000 });
        return true;
      } catch (error) {
        logger.error('URL connectivity test failed:', error);
        return false;
      }
    }
    return true;
  }

  async sync(dataSource: DataSource): Promise<{ documents: any[], urls: any[] }> {
    const documents: any[] = [];
    const urls: any[] = [];
    const { urls: urlList, crawlDepth = 0, allowedDomains = [] } = dataSource.config;

    for (const url of urlList || []) {
      try {
        const crawledUrls = await this.crawlUrl(url, crawlDepth, allowedDomains);
        
        for (const crawledUrl of crawledUrls) {
          const content = await this.extractContent(crawledUrl.url);
          
          if (content) {
            documents.push({
              id: `url-${Buffer.from(crawledUrl.url).toString('base64')}`,
              title: content.title || crawledUrl.url,
              content: content.text,
              metadata: {
                source: 'url',
                url: crawledUrl.url,
                title: content.title,
                description: content.description,
                keywords: content.keywords,
                author: content.author,
                publishedDate: content.publishedDate,
                lastModified: content.lastModified,
                depth: crawledUrl.depth,
                contentType: content.contentType,
                wordCount: content.wordCount
              }
            });

            urls.push(crawledUrl);
          }
        }
      } catch (error) {
        logger.error(`Error syncing URL ${url}:`, error);
      }
    }

    return { documents, urls };
  }

  private async crawlUrl(
    startUrl: string, 
    maxDepth: number = 0, 
    allowedDomains: string[] = []
  ): Promise<{ url: string; depth: number }[]> {
    const visited = new Set<string>();
    const toVisit: { url: string; depth: number }[] = [{ url: startUrl, depth: 0 }];
    const results: { url: string; depth: number }[] = [];

    while (toVisit.length > 0) {
      const { url, depth } = toVisit.shift()!;
      
      if (visited.has(url) || depth > maxDepth) {
        continue;
      }

      visited.add(url);
      results.push({ url, depth });

      if (depth < maxDepth) {
        try {
          const links = await this.extractLinks(url);
          
          for (const link of links) {
            if (!visited.has(link) && this.isAllowedDomain(link, allowedDomains)) {
              toVisit.push({ url: link, depth: depth + 1 });
            }
          }
        } catch (error) {
          logger.warn(`Could not extract links from ${url}:`, error);
        }
      }
    }

    return results;
  }

  private async extractLinks(url: string): Promise<string[]> {
    try {
      const response = await axios.get(url, { 
        timeout: 10000,
        headers: {
          'User-Agent': 'ConHub-Crawler/1.0'
        }
      });

      const $ = cheerio.load(response.data);
      const links: string[] = [];
      const baseUrl = new URL(url);

      $('a[href]').each((_: any, element: any) => {
        const href = $(element).attr('href');
        if (href) {
          try {
            const absoluteUrl = new URL(href, baseUrl).toString();
            if (absoluteUrl.startsWith('http')) {
              links.push(absoluteUrl);
            }
          } catch (error) {
            // Invalid URL, skip
          }
        }
      });

      return [...new Set(links)]; // Remove duplicates
    } catch (error) {
      logger.error(`Error extracting links from ${url}:`, error);
      return [];
    }
  }

  private async extractContent(url: string): Promise<{
    title?: string;
    text: string;
    description?: string;
    keywords?: string[];
    author?: string;
    publishedDate?: string;
    lastModified?: string;
    contentType?: string;
    wordCount: number;
  } | null> {
    try {
      const response = await axios.get(url, { 
        timeout: 15000,
        headers: {
          'User-Agent': 'ConHub-Crawler/1.0'
        }
      });

      const contentType = response.headers['content-type'] || '';
      
      // Handle different content types
      if (contentType.includes('application/json')) {
        return {
          text: JSON.stringify(response.data, null, 2),
          contentType: 'application/json',
          wordCount: JSON.stringify(response.data).split(/\s+/).length
        };
      }

      if (contentType.includes('text/plain')) {
        return {
          text: response.data,
          contentType: 'text/plain',
          wordCount: response.data.split(/\s+/).length
        };
      }

      if (!contentType.includes('text/html')) {
        return null; // Skip non-text content
      }

      const $ = cheerio.load(response.data);

      // Remove script and style elements
      $('script, style, nav, footer, aside, .advertisement, .ads').remove();

      // Extract metadata
      const title = $('title').text().trim() || 
                   $('meta[property="og:title"]').attr('content') || 
                   $('h1').first().text().trim();

      const description = $('meta[name="description"]').attr('content') || 
                         $('meta[property="og:description"]').attr('content');

      const keywords = $('meta[name="keywords"]').attr('content')?.split(',').map((k: string) => k.trim());

      const author = $('meta[name="author"]').attr('content') || 
                    $('meta[property="article:author"]').attr('content');

      const publishedDate = $('meta[property="article:published_time"]').attr('content') || 
                           $('meta[name="date"]').attr('content');

      const lastModified = $('meta[property="article:modified_time"]').attr('content') || 
                          $('meta[name="last-modified"]').attr('content');

      // Extract main content
      let text = '';
      
      // Try to find main content areas
      const contentSelectors = [
        'main',
        'article',
        '.content',
        '.post-content',
        '.entry-content',
        '.article-content',
        '#content',
        '.main-content'
      ];

      let contentFound = false;
      for (const selector of contentSelectors) {
        const content = $(selector);
        if (content.length > 0) {
          text = content.text().trim();
          contentFound = true;
          break;
        }
      }

      // Fallback to body if no main content found
      if (!contentFound) {
        text = $('body').text().trim();
      }

      // Clean up text
      text = text.replace(/\s+/g, ' ').trim();

      const wordCount = text.split(/\s+/).filter((word: string) => word.length > 0).length;

      return {
        title,
        text,
        description,
        keywords,
        author,
        publishedDate,
        lastModified,
        contentType: 'text/html',
        wordCount
      };

    } catch (error) {
      logger.error(`Error extracting content from ${url}:`, error);
      return null;
    }
  }

  private isAllowedDomain(url: string, allowedDomains: string[]): boolean {
    if (allowedDomains.length === 0) {
      return true; // No restrictions
    }

    try {
      const domain = new URL(url).hostname;
      return allowedDomains.some(allowed => 
        domain === allowed || domain.endsWith('.' + allowed)
      );
    } catch (error) {
      return false;
    }
  }
}