"""
Document source connectors for various cloud storage providers and local files.
"""
import os
import json
import uuid
from typing import List, Dict, Any, Optional
from pathlib import Path
import asyncio
import aiofiles
import logging

logger = logging.getLogger(__name__)

class DocumentConnector:
    """Base class for document connectors"""
    
    def __init__(self, connector_id: str, connector_type: str):
        self.connector_id = connector_id
        self.connector_type = connector_type
        self.is_connected = False
        
    async def connect(self, credentials: Dict[str, Any]) -> bool:
        """Connect to the document source"""
        raise NotImplementedError
        
    async def disconnect(self) -> bool:
        """Disconnect from the document source"""
        self.is_connected = False
        return True
        
    async def list_documents(self, folder_path: str = "/") -> List[Dict[str, Any]]:
        """List documents in the specified folder"""
        raise NotImplementedError
        
    async def get_document_content(self, document_id: str) -> str:
        """Get the content of a specific document"""
        raise NotImplementedError
        
    async def sync_documents(self) -> List[Dict[str, Any]]:
        """Sync all documents from the source"""
        raise NotImplementedError

class DropboxConnector(DocumentConnector):
    """Dropbox document connector"""
    
    def __init__(self, connector_id: str):
        super().__init__(connector_id, "dropbox")
        self.access_token = None
        self.folder_path = "/"
        
    async def connect(self, credentials: Dict[str, Any]) -> bool:
        """Connect to Dropbox"""
        try:
            self.access_token = credentials.get("access_token")
            self.folder_path = credentials.get("folder_path", "/")
            
            if not self.access_token:
                raise ValueError("Dropbox access token is required")
                
            # TODO: Implement actual Dropbox API connection
            # For now, simulate successful connection
            self.is_connected = True
            logger.info(f"Connected to Dropbox folder: {self.folder_path}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to connect to Dropbox: {e}")
            return False
            
    async def list_documents(self, folder_path: str = None) -> List[Dict[str, Any]]:
        """List documents in Dropbox folder"""
        if not self.is_connected:
            raise RuntimeError("Not connected to Dropbox")
            
        # TODO: Implement actual Dropbox API calls
        # For now, return mock data
        return [
            {
                "id": f"dropbox-doc-{uuid.uuid4()}",
                "name": "sample-document.txt",
                "path": f"{folder_path or self.folder_path}/sample-document.txt",
                "size": 1024,
                "modified": "2024-01-01T00:00:00Z",
                "type": "text/plain"
            }
        ]
        
    async def get_document_content(self, document_id: str) -> str:
        """Get document content from Dropbox"""
        if not self.is_connected:
            raise RuntimeError("Not connected to Dropbox")
            
        # TODO: Implement actual Dropbox API calls
        return "Sample document content from Dropbox"
        
    async def sync_documents(self) -> List[Dict[str, Any]]:
        """Sync all documents from Dropbox"""
        documents = await self.list_documents()
        synced_docs = []
        
        for doc in documents:
            try:
                content = await self.get_document_content(doc["id"])
                synced_docs.append({
                    **doc,
                    "content": content,
                    "source": "dropbox"
                })
            except Exception as e:
                logger.error(f"Failed to sync document {doc['id']}: {e}")
                
        return synced_docs

class GoogleDriveConnector(DocumentConnector):
    """Google Drive document connector"""
    
    def __init__(self, connector_id: str):
        super().__init__(connector_id, "google_drive")
        self.credentials = None
        self.folder_id = None
        
    async def connect(self, credentials: Dict[str, Any]) -> bool:
        """Connect to Google Drive"""
        try:
            self.credentials = credentials.get("credentials")
            self.folder_id = credentials.get("folder_id")
            
            if not self.credentials:
                raise ValueError("Google Drive credentials are required")
                
            # TODO: Implement actual Google Drive API connection
            # For now, simulate successful connection
            self.is_connected = True
            logger.info(f"Connected to Google Drive folder: {self.folder_id}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to connect to Google Drive: {e}")
            return False
            
    async def list_documents(self, folder_id: str = None) -> List[Dict[str, Any]]:
        """List documents in Google Drive folder"""
        if not self.is_connected:
            raise RuntimeError("Not connected to Google Drive")
            
        # TODO: Implement actual Google Drive API calls
        return [
            {
                "id": f"gdrive-doc-{uuid.uuid4()}",
                "name": "sample-document.docx",
                "path": f"/{folder_id or self.folder_id}/sample-document.docx",
                "size": 2048,
                "modified": "2024-01-01T00:00:00Z",
                "type": "application/vnd.google-apps.document"
            }
        ]
        
    async def get_document_content(self, document_id: str) -> str:
        """Get document content from Google Drive"""
        if not self.is_connected:
            raise RuntimeError("Not connected to Google Drive")
            
        # TODO: Implement actual Google Drive API calls
        return "Sample document content from Google Drive"
        
    async def sync_documents(self) -> List[Dict[str, Any]]:
        """Sync all documents from Google Drive"""
        documents = await self.list_documents()
        synced_docs = []
        
        for doc in documents:
            try:
                content = await self.get_document_content(doc["id"])
                synced_docs.append({
                    **doc,
                    "content": content,
                    "source": "google_drive"
                })
            except Exception as e:
                logger.error(f"Failed to sync document {doc['id']}: {e}")
                
        return synced_docs

class OneDriveConnector(DocumentConnector):
    """Microsoft OneDrive document connector"""
    
    def __init__(self, connector_id: str):
        super().__init__(connector_id, "onedrive")
        self.access_token = None
        self.folder_path = "/"
        
    async def connect(self, credentials: Dict[str, Any]) -> bool:
        """Connect to OneDrive"""
        try:
            self.access_token = credentials.get("access_token")
            self.folder_path = credentials.get("folder_path", "/")
            
            if not self.access_token:
                raise ValueError("OneDrive access token is required")
                
            # TODO: Implement actual OneDrive API connection
            # For now, simulate successful connection
            self.is_connected = True
            logger.info(f"Connected to OneDrive folder: {self.folder_path}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to connect to OneDrive: {e}")
            return False
            
    async def list_documents(self, folder_path: str = None) -> List[Dict[str, Any]]:
        """List documents in OneDrive folder"""
        if not self.is_connected:
            raise RuntimeError("Not connected to OneDrive")
            
        # TODO: Implement actual OneDrive API calls
        return [
            {
                "id": f"onedrive-doc-{uuid.uuid4()}",
                "name": "sample-document.pdf",
                "path": f"{folder_path or self.folder_path}/sample-document.pdf",
                "size": 4096,
                "modified": "2024-01-01T00:00:00Z",
                "type": "application/pdf"
            }
        ]
        
    async def get_document_content(self, document_id: str) -> str:
        """Get document content from OneDrive"""
        if not self.is_connected:
            raise RuntimeError("Not connected to OneDrive")
            
        # TODO: Implement actual OneDrive API calls
        return "Sample document content from OneDrive"
        
    async def sync_documents(self) -> List[Dict[str, Any]]:
        """Sync all documents from OneDrive"""
        documents = await self.list_documents()
        synced_docs = []
        
        for doc in documents:
            try:
                content = await self.get_document_content(doc["id"])
                synced_docs.append({
                    **doc,
                    "content": content,
                    "source": "onedrive"
                })
            except Exception as e:
                logger.error(f"Failed to sync document {doc['id']}: {e}")
                
        return synced_docs

class LocalFileConnector(DocumentConnector):
    """Local file system connector"""
    
    def __init__(self, connector_id: str):
        super().__init__(connector_id, "local_files")
        self.upload_path = "./uploads"
        self.allowed_extensions = [".txt", ".md", ".pdf", ".docx", ".py", ".js", ".ts", ".rs", ".go", ".java"]
        
    async def connect(self, credentials: Dict[str, Any]) -> bool:
        """Connect to local file system"""
        try:
            self.upload_path = credentials.get("upload_path", "./uploads")
            self.allowed_extensions = credentials.get("allowed_extensions", self.allowed_extensions)
            
            # Create upload directory if it doesn't exist
            os.makedirs(self.upload_path, exist_ok=True)
            
            self.is_connected = True
            logger.info(f"Connected to local file system: {self.upload_path}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to connect to local file system: {e}")
            return False
            
    async def list_documents(self, folder_path: str = None) -> List[Dict[str, Any]]:
        """List documents in local folder"""
        if not self.is_connected:
            raise RuntimeError("Not connected to local file system")
            
        search_path = Path(folder_path) if folder_path else Path(self.upload_path)
        documents = []
        
        try:
            for file_path in search_path.rglob("*"):
                if file_path.is_file() and file_path.suffix.lower() in self.allowed_extensions:
                    stat = file_path.stat()
                    documents.append({
                        "id": f"local-{uuid.uuid4()}",
                        "name": file_path.name,
                        "path": str(file_path),
                        "size": stat.st_size,
                        "modified": stat.st_mtime,
                        "type": f"text/{file_path.suffix[1:]}" if file_path.suffix else "text/plain"
                    })
        except Exception as e:
            logger.error(f"Failed to list local documents: {e}")
            
        return documents
        
    async def get_document_content(self, file_path: str) -> str:
        """Get document content from local file"""
        if not self.is_connected:
            raise RuntimeError("Not connected to local file system")
            
        try:
            async with aiofiles.open(file_path, 'r', encoding='utf-8') as f:
                return await f.read()
        except Exception as e:
            logger.error(f"Failed to read local file {file_path}: {e}")
            raise
            
    async def save_uploaded_file(self, file_content: bytes, filename: str) -> Dict[str, Any]:
        """Save uploaded file to local storage"""
        if not self.is_connected:
            raise RuntimeError("Not connected to local file system")
            
        file_path = Path(self.upload_path) / filename
        
        try:
            async with aiofiles.open(file_path, 'wb') as f:
                await f.write(file_content)
                
            stat = file_path.stat()
            return {
                "id": f"local-{uuid.uuid4()}",
                "name": filename,
                "path": str(file_path),
                "size": stat.st_size,
                "modified": stat.st_mtime,
                "type": f"text/{file_path.suffix[1:]}" if file_path.suffix else "text/plain"
            }
        except Exception as e:
            logger.error(f"Failed to save uploaded file {filename}: {e}")
            raise
            
    async def sync_documents(self) -> List[Dict[str, Any]]:
        """Sync all local documents"""
        documents = await self.list_documents()
        synced_docs = []
        
        for doc in documents:
            try:
                content = await self.get_document_content(doc["path"])
                synced_docs.append({
                    **doc,
                    "content": content,
                    "source": "local_files"
                })
            except Exception as e:
                logger.error(f"Failed to sync document {doc['path']}: {e}")
                
        return synced_docs

class DocumentConnectorManager:
    """Manager for all document connectors"""
    
    def __init__(self):
        self.connectors: Dict[str, DocumentConnector] = {}
        
    def create_connector(self, connector_type: str) -> DocumentConnector:
        """Create a new document connector"""
        connector_id = f"{connector_type}-{uuid.uuid4()}"
        
        if connector_type == "dropbox":
            connector = DropboxConnector(connector_id)
        elif connector_type == "google_drive":
            connector = GoogleDriveConnector(connector_id)
        elif connector_type == "onedrive":
            connector = OneDriveConnector(connector_id)
        elif connector_type == "local_files":
            connector = LocalFileConnector(connector_id)
        else:
            raise ValueError(f"Unsupported connector type: {connector_type}")
            
        self.connectors[connector_id] = connector
        return connector
        
    def get_connector(self, connector_id: str) -> Optional[DocumentConnector]:
        """Get a connector by ID"""
        return self.connectors.get(connector_id)
        
    def remove_connector(self, connector_id: str) -> bool:
        """Remove a connector"""
        if connector_id in self.connectors:
            connector = self.connectors[connector_id]
            asyncio.create_task(connector.disconnect())
            del self.connectors[connector_id]
            return True
        return False
        
    def list_connectors(self) -> List[Dict[str, Any]]:
        """List all connectors"""
        return [
            {
                "id": connector.connector_id,
                "type": connector.connector_type,
                "connected": connector.is_connected
            }
            for connector in self.connectors.values()
        ]

# Global connector manager instance
connector_manager = DocumentConnectorManager()