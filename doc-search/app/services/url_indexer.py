"""
URL indexing service for crawling and indexing web content.
"""
import asyncio
import aiohttp
import uuid
from typing import List, Dict, Any, Optional
from urllib.parse import urljoin, urlparse
from bs4 import BeautifulSoup
import logging

logger = logging.getLogger(__name__)

class URLIndexer:
    """Service for crawling and indexing web content"""
    
    def __init__(self):
        self.session = None
        self.indexed_urls = set()
        
    async def __aenter__(self):
        self.session = aiohttp.ClientSession(
            timeout=aiohttp.ClientTimeout(total=30),
            headers={
                'User-Agent': 'ConHub-URLIndexer/1.0 (Web Content Indexer)'
            }
        )
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()
            
    async def index_url(self, url: str, max_depth: int = 2, allowed_domains: List[str] = None) -> List[Dict[str, Any]]:
        """Index a URL and optionally crawl related pages"""
        if not self.session:
            raise RuntimeError("URLIndexer must be used as async context manager")
            
        indexed_documents = []
        urls_to_process = [(url, 0)]  # (url, depth)
        processed_urls = set()
        
        while urls_to_process:
            current_url, depth = urls_to_process.pop(0)
            
            if current_url in processed_urls or depth > max_depth:
                continue
                
            if allowed_domains and not self._is_allowed_domain(current_url, allowed_domains):
                continue
                
            try:
                document = await self._crawl_single_url(current_url)
                if document:
                    indexed_documents.append(document)
                    processed_urls.add(current_url)
                    
                    # Extract links for further crawling if within depth limit
                    if depth < max_depth:
                        links = await self._extract_links(current_url, document["raw_html"])
                        for link in links:
                            if link not in processed_urls:
                                urls_to_process.append((link, depth + 1))
                                
            except Exception as e:
                logger.error(f"Failed to crawl {current_url}: {e}")
                continue
                
        return indexed_documents
        
    async def _crawl_single_url(self, url: str) -> Optional[Dict[str, Any]]:
        """Crawl a single URL and extract content"""
        try:
            async with self.session.get(url) as response:
                if response.status != 200:
                    logger.warning(f"HTTP {response.status} for {url}")
                    return None
                    
                content_type = response.headers.get('content-type', '').lower()
                if 'text/html' not in content_type:
                    logger.info(f"Skipping non-HTML content: {url}")
                    return None
                    
                html_content = await response.text()
                
                # Parse HTML content
                soup = BeautifulSoup(html_content, 'html.parser')
                
                # Extract metadata
                title = self._extract_title(soup)
                description = self._extract_description(soup)
                text_content = self._extract_text_content(soup)
                
                # Create document
                document = {
                    "id": f"url-{uuid.uuid4()}",
                    "url": url,
                    "title": title,
                    "description": description,
                    "content": text_content,
                    "raw_html": html_content,
                    "content_type": "text/html",
                    "size": len(html_content),
                    "indexed_at": asyncio.get_event_loop().time(),
                    "source_type": "url"
                }
                
                logger.info(f"Successfully crawled: {url} ({len(text_content)} chars)")
                return document
                
        except Exception as e:
            logger.error(f"Error crawling {url}: {e}")
            return None
            
    def _extract_title(self, soup: BeautifulSoup) -> str:
        """Extract page title"""
        title_tag = soup.find('title')
        if title_tag:
            return title_tag.get_text().strip()
            
        # Fallback to h1
        h1_tag = soup.find('h1')
        if h1_tag:
            return h1_tag.get_text().strip()
            
        return "Untitled"
        
    def _extract_description(self, soup: BeautifulSoup) -> Optional[str]:
        """Extract page description from meta tags"""
        # Try meta description
        meta_desc = soup.find('meta', attrs={'name': 'description'})
        if meta_desc and meta_desc.get('content'):
            return meta_desc['content'].strip()
            
        # Try Open Graph description
        og_desc = soup.find('meta', attrs={'property': 'og:description'})
        if og_desc and og_desc.get('content'):
            return og_desc['content'].strip()
            
        return None
        
    def _extract_text_content(self, soup: BeautifulSoup) -> str:
        """Extract clean text content from HTML"""
        # Remove script and style elements
        for script in soup(["script", "style", "nav", "footer", "header"]):
            script.decompose()
            
        # Get text content
        text = soup.get_text()
        
        # Clean up whitespace
        lines = (line.strip() for line in text.splitlines())
        chunks = (phrase.strip() for line in lines for phrase in line.split("  "))
        text = ' '.join(chunk for chunk in chunks if chunk)
        
        return text
        
    async def _extract_links(self, base_url: str, html_content: str) -> List[str]:
        """Extract links from HTML content"""
        soup = BeautifulSoup(html_content, 'html.parser')
        links = []
        
        for link in soup.find_all('a', href=True):
            href = link['href']
            absolute_url = urljoin(_url, href)
            
            # Filter out non-HTTP links
            parsed = urlparse(absolute_url)
            if parsed.scheme in ('http', 'https'):
                links.append(absolute_url)
                
        return list(set(links))  # Remove duplicates
        
    def _is_allowed_domain(self, url: str, allowed_domains: List[str]) -> bool:
        """Check if URL domain is in allowed domains list"""
        parsed = urlparse(url)
        domain = parsed.netloc.lower()
        
        for allowed in allowed_domains:
            if domain == allowed.lower() or domain.endswith(f'.{allowed.lower()}'):
                return True
                
        return False

class URLIndexingService:
    """Service for managing URL indexing operations"""
    
    def __init__(self):
        self.indexing_jobs = {}
        
    async def start_url_indexing(self, urls: List[str], config: Dict[str, Any] = None) -> str:
        """Start indexing URLs in the background"""
        job_id = f"url-job-{uuid.uuid4()}"
        config = config or {}
        
        # Start background task
        task = asyncio.create_task(self._index_urls_background(job_id, urls, config))
        
        self.indexing_jobs[job_id] = {
            "id": job_id,
            "status": "running",
            "urls": urls,
            "config": config,
            "task": task,
            "results": [],
            "errors": []
        }
        
        return job_id
        
    async def _index_urls_background(self, job_id: str, urls: List[str], config: Dict[str, Any]):
        """Background task for indexing URLs"""
        job = self.indexing_jobs[job_id]
        
        try:
            async with URLIndexer() as indexer:
                for url in urls:
                    try:
                        max_depth = config.get("crawl_depth", 1)
                        allowed_domains = config.get("allowed_domains", [])
                        
                        documents = await indexer.index_url(url, max_depth, allowed_domains)
                        job["results"].extend(documents)
                        
                        logger.info(f"Indexed {len(documents)} documents from {url}")
                        
                    except Exception as e:
                        error_msg = f"Failed to index {url}: {e}"
                        job["errors"].append(error_msg)
                        logger.error(error_msg)
                        
            job["status"] = "completed"
            
        except Exception as e:
            job["status"] = "failed"
            job["errors"].append(f"Job failed: {e}")
            logger.error(f"URL indexing job {job_id} failed: {e}")
            
    def get_job_status(self, job_id: str) -> Optional[Dict[str, Any]]:
        """Get the status of an indexing job"""
        job = self.indexing_jobs.get(job_id)
        if not job:
            return None
            
        return {
            "id": job["id"],
            "status": job["status"],
            "urls_count": len(job["urls"]),
            "documents_indexed": len(job["results"]),
            "errors_count": len(job["errors"]),
            "errors": job["errors"][-5:]  # Last 5 errors
        }
        
    def get_job_results(self, job_id: str) -> Optional[List[Dict[str, Any]]]:
        """Get the results of a completed indexing job"""
        job = self.indexing_jobs.get(job_id)
        if not job:
            return None
            
        return job["results"]
        
    def list_jobs(self) -> List[Dict[str, Any]]:
        """List all indexing jobs"""
        return [
            {
                "id": job["id"],
                "status": job["status"],
                "urls_count": len(job["urls"]),
                "documents_indexed": len(job["results"]),
                "errors_count": len(job["errors"])
            }
            for job in self.indexing_jobs.values()
        ]

# Global URL indexing service instance
url_indexing_service = URLIndexingService()