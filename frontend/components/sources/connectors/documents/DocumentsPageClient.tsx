'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Footer } from "@/components/ui/footer";
import { ArrowLeft, Plus, Settings, ExternalLink, RefreshCw, MoreHorizontal, Trash2, FileText, FolderOpen, Cloud } from "lucide-react";
import Link from "next/link";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/AlertDialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/DropdownMenu";
import { AddDocumentModal } from "@/components/ui/AddDocumentModal";
import { DocumentDetailModal } from "@/components/ui/DocumentDetailModal";
import { apiClient, ApiResponse } from '@/lib/api';

interface DocumentSource {
  id: string;
  name: string;
  type: 'local' | 'google_drive' | 'onedrive' | 'dropbox';
  path: string;
  fileCount: number;
  lastSynced: string;
  status: 'active' | 'inactive' | 'syncing' | 'error';
  size: string;
}

interface DataSource {
  id: string;
  type: string;
  name: string;
  status: 'connected' | 'syncing' | 'error' | 'indexing';
  totalCount?: number;
  indexedCount?: number;
  config?: any;
}

export function DocumentsPageClient() {
  const [documentSources, setDocumentSources] = useState<DocumentSource[]>([]);
  const [dataSources, setDataSources] = useState<DataSource[]>([]);
  const [loading, setLoading] = useState(true);
  const [showConnectDialog, setShowConnectDialog] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [sourceToDelete, setSourceToDelete] = useState<DocumentSource | null>(null);
  const [showDocumentDetail, setShowDocumentDetail] = useState(false);
  const [uploadedDocuments, setUploadedDocuments] = useState<any[]>([]);

  useEffect(() => {
    fetchDocumentSources();
    fetchDataSources();
  }, []);

  const fetchDocumentSources = async () => {
    try {
      // Get uploaded documents from localStorage
      const uploadedDocs = JSON.parse(localStorage.getItem('uploadedDocuments') || '[]');
      
      const sources: DocumentSource[] = [];
      
      // Add uploaded documents as a source if any exist
      if (uploadedDocs.length > 0) {
        const totalSize = uploadedDocs.reduce((sum: number, doc: any) => {
          const sizeMatch = doc.size?.match(/([\d.]+)\s*(\w+)/);
          if (sizeMatch) {
            const value = parseFloat(sizeMatch[1]);
            const unit = sizeMatch[2].toLowerCase();
            const bytes = unit === 'kb' ? value * 1024 : unit === 'mb' ? value * 1024 * 1024 : value;
            return sum + bytes;
          }
          return sum;
        }, 0);
        
        const formatBytes = (bytes: number) => {
          if (bytes === 0) return '0 Bytes';
          const k = 1024;
          const sizes = ['Bytes', 'KB', 'MB', 'GB'];
          const i = Math.floor(Math.log(bytes) / Math.log(k));
          return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        };
        
        sources.push({
          id: 'uploaded-docs',
          name: 'Uploaded Documents',
          type: 'local',
          path: 'Local Storage',
          fileCount: uploadedDocs.length,
          lastSynced: new Date().toISOString(),
          status: 'active',
          size: formatBytes(totalSize)
        });
        
        setUploadedDocuments(uploadedDocs);
      }
      
      setDocumentSources(sources);
    } catch (error) {
      console.error('Error fetching document sources:', error);
    } finally {
      setLoading(false);
    }
  };

  const fetchDataSources = async () => {
    try {
      const response: ApiResponse<{ dataSources: DataSource[] }> = await apiClient.get('/api/data-sources');
      if (response.success) {
        const documentDataSources = response.data.dataSources.filter(
          ds => ds.type === 'documents' || ds.type === 'local_files' || ds.type === 'google_drive' || ds.type === 'onedrive'
        );
        setDataSources(documentDataSources);
      }
    } catch (error) {
      console.error('Error fetching data sources:', error);
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
      case 'connected':
        return 'bg-green-100 text-green-800';
      case 'syncing':
      case 'indexing':
        return 'bg-blue-100 text-blue-800';
      case 'error':
        return 'bg-red-100 text-red-800';
      case 'inactive':
        return 'bg-gray-100 text-gray-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'local':
        return <FolderOpen className="h-4 w-4" />;
      case 'google_drive':
      case 'onedrive':
      case 'dropbox':
        return <Cloud className="h-4 w-4" />;
      default:
        return <FileText className="h-4 w-4" />;
    }
  };

  const handleDeleteSource = (source: DocumentSource) => {
    setSourceToDelete(source);
    setDeleteDialogOpen(true);
  };

  const confirmDelete = async () => {
    if (!sourceToDelete) return;
    
    try {
      if (sourceToDelete.id === 'uploaded-docs') {
        // Clear uploaded documents from localStorage
        localStorage.removeItem('uploadedDocuments');
        fetchDocumentSources();
      }
      setDocumentSources(prev => prev.filter(s => s.id !== sourceToDelete.id));
      setDeleteDialogOpen(false);
      setSourceToDelete(null);
    } catch (error) {
      console.error('Error deleting document source:', error);
    }
  };

  const syncSource = async (sourceId: string) => {
    try {
      // API call to sync the source
      // await apiClient.post(`/api/document-sources/${sourceId}/sync`);
      setDocumentSources(prev => 
        prev.map(s => s.id === sourceId ? { ...s, status: 'syncing' } : s)
      );
    } catch (error) {
      console.error('Error syncing document source:', error);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-gray-900"></div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="container mx-auto px-4 py-8">
        <div className="flex items-center justify-between mb-8">
          <div className="flex items-center space-x-4">
            <Link href="/sources">
              <Button variant="ghost" size="sm">
                <ArrowLeft className="h-4 w-4 mr-2" />
                Back to Data Sources
              </Button>
            </Link>
            <div>
              <h1 className="text-3xl font-bold text-gray-900">Document Sources</h1>
              <p className="text-gray-600 mt-1">Connect and manage your document-based data sources</p>
            </div>
          </div>
          <AddDocumentModal
            open={showConnectDialog}
            onOpenChange={setShowConnectDialog}
            onDocumentAdded={() => {
              fetchDocumentSources();
              fetchDataSources();
            }}
          />
          <Button onClick={() => setShowConnectDialog(true)}>
            <Plus className="h-4 w-4 mr-2" />
            Add Documents
          </Button>
        </div>

        <div className="grid gap-6">
          {documentSources.map((source) => (
            <Card key={source.id} className="hover:shadow-md transition-shadow">
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
                <div className="flex items-center space-x-3">
                  {getTypeIcon(source.type)}
                  <div>
                    <CardTitle className="text-lg">{source.name}</CardTitle>
                    <p className="text-sm text-gray-600">{source.path}</p>
                  </div>
                </div>
                <div className="flex items-center space-x-2">
                  <Badge className={getStatusColor(source.status)}>
                    {source.status}
                  </Badge>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="sm">
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem onClick={() => syncSource(source.id)}>
                        <RefreshCw className="h-4 w-4 mr-2" />
                        Sync Now
                      </DropdownMenuItem>
                      <DropdownMenuItem>
                        <Settings className="h-4 w-4 mr-2" />
                        Settings
                      </DropdownMenuItem>
                      <DropdownMenuSeparator />
                      <DropdownMenuItem 
                        onClick={() => handleDeleteSource(source)}
                        className="text-red-600"
                      >
                        <Trash2 className="h-4 w-4 mr-2" />
                        Remove
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-3 gap-4 text-sm">
                  <div>
                    <p className="text-gray-600">Files</p>
                    <p className="font-medium">{source.fileCount.toLocaleString()}</p>
                  </div>
                  <div>
                    <p className="text-gray-600">Size</p>
                    <p className="font-medium">{source.size}</p>
                  </div>
                  <div>
                    <p className="text-gray-600">Last Synced</p>
                    <p className="font-medium">
                      {new Date(source.lastSynced).toLocaleDateString()}
                    </p>
                  </div>
                </div>
                {source.id === 'uploaded-docs' && source.fileCount > 0 && (
                  <div className="mt-4 pt-4 border-t">
                    <Button 
                      variant="outline" 
                      size="sm" 
                      onClick={() => setShowDocumentDetail(true)}
                      className="w-full"
                    >
                      <FileText className="w-4 h-4 mr-2" />
                      View Documents ({source.fileCount})
                    </Button>
                  </div>
                )}
              </CardContent>
            </Card>
          ))}

          {documentSources.length === 0 && (
            <Card className="text-center py-12">
              <CardContent>
                <FileText className="h-12 w-12 text-gray-400 mx-auto mb-4" />
                <h3 className="text-lg font-medium text-gray-900 mb-2">No document sources connected</h3>
                <p className="text-gray-600 mb-4">
                  Connect your first document source to start indexing files from local storage, Google Drive, OneDrive, or other document repositories.
                </p>
                <Button onClick={() => setShowConnectDialog(true)}>
                  <Plus className="h-4 w-4 mr-2" />
                  Add Documents
                </Button>
              </CardContent>
            </Card>
          )}
        </div>
      </div>

      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Remove Document Source</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to remove "{sourceToDelete?.name}"? This will stop indexing files from this source and remove all associated data.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={confirmDelete} className="bg-red-600 hover:bg-red-700">
              Remove
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <DocumentDetailModal
        open={showDocumentDetail}
        onOpenChange={setShowDocumentDetail}
        documents={uploadedDocuments}
        onDocumentDeleted={() => {
          fetchDocumentSources();
          const updatedDocs = JSON.parse(localStorage.getItem('uploadedDocuments') || '[]');
          setUploadedDocuments(updatedDocs);
        }}
      />

      <Footer />
    </div>
  );
}