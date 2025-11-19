"use client";

import { useState, useEffect, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { AddDocumentModal } from "@/components/ui/AddDocumentModal";
import { ConnectSourceModal } from "@/components/ui/ConnectSourceModal";
import { ProfileAvatar } from "@/components/ui/ProfileAvatar";
import { Footer } from "@/components/ui/footer";
import { useToast } from "@/hooks/use-toast";
import { dataApiClient } from "@/lib/api";
import Link from "next/link";
import { 
  Plus, 
  FileText, 
  Download, 
  Trash2, 
  ArrowLeft,
  Calendar,
  Tag,
  File,
  FolderOpen
} from "lucide-react";

interface DocumentRecord {
  id: string;
  user_id: string;
  name: string;
  doc_type: string;
  source: string;
  size: string;
  tags: string[];
  created_at: string;
  updated_at: string;
  status: string;
}

export default function DocumentsPage() {
  const [documents, setDocuments] = useState<DocumentRecord[]>([]);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isConnectModalOpen, setIsConnectModalOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const { toast } = useToast();

  const fetchDocuments = useCallback(async () => {
    try {
      const result = await dataApiClient.getDocuments();
      if (result.success) {
        
        setDocuments(Array.isArray(result.data) ? result.data : result.data?.data || []);
      } else {
        toast({
          title: "Error",
          description: result.message || "Failed to fetch documents",
          variant: "destructive",
        });
      }
    } catch (error) {
      console.error("Error fetching documents:", error);
      toast({
        title: "Error",
        description: "Failed to fetch documents",
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  }, [toast]);

  const deleteDocument = async (id: string) => {
    try {
      const result = await dataApiClient.deleteDocument(id);
      if (result.success) {
        setDocuments(prev => prev.filter(doc => doc.id !== id));
        toast({
          title: "Success",
          description: "Document deleted successfully",
        });
      } else {
        toast({
          title: "Error",
          description: result.message || "Failed to delete document",
          variant: "destructive",
        });
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to delete document",
        variant: "destructive",
      });
    }
  };

  useEffect(() => {
    const init = async () => {
      // Check backend health first to avoid JSON parse errors when API returns HTML or is down
      const healthy = await dataApiClient.checkBackendHealth();
      if (healthy) {
        await fetchDocuments();
      } else {
        toast({
          title: "Backend Unavailable",
          description: "Could not reach the API. Please start backend services or try again later.",
          variant: "destructive",
        });
        setIsLoading(false);
      }
    };
    init();
  }, [fetchDocuments, toast]);

  return (
    <div className="min-h-screen bg-background">
      {}
      <div className="border-b border-border bg-card/50 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-20">
            <div className="flex items-center space-x-4">
              <Link href="/dashboard">
                <Button variant="ghost" size="sm">
                  <ArrowLeft className="w-4 h-4" />
                </Button>
              </Link>
              <div className="h-6 w-px bg-border" />
              <div className="flex items-center gap-2">
                <FileText className="w-5 h-5 text-primary" />
                <h1 className="text-2xl font-bold text-foreground">Documents</h1>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <ProfileAvatar />
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Total Documents</CardTitle>
              <FileText className="w-4 h-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{documents.length}</div>
            </CardContent>
          </Card>
          
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Processed</CardTitle>
              <FileText className="w-4 h-4 text-green-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {documents.filter(doc => doc.status === 'processed').length}
              </div>
            </CardContent>
          </Card>
          
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Total Size</CardTitle>
              <FolderOpen className="w-4 h-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">4.2 MB</div>
            </CardContent>
          </Card>
        </div>

        {}
        <div className="flex justify-between items-center mb-6">
          <div>
            <h2 className="text-xl font-semibold text-foreground">Your Documents</h2>
            <p className="text-sm text-muted-foreground">Connect and manage documents from various sources</p>
          </div>
          <div className="flex gap-3">
            <Button onClick={() => setIsConnectModalOpen(true)}>
              <Plus className="w-4 h-4 mr-2" />
              Connect Source
            </Button>
          </div>
        </div>

        {}
        <Card>
          <CardHeader>
            <CardTitle>Document Collection</CardTitle>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="flex items-center justify-center py-8">
                <div className="text-muted-foreground">Loading documents...</div>
              </div>
            ) : documents.length === 0 ? (
              <div className="text-center py-8">
                <FileText className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
                <h3 className="text-lg font-medium text-foreground mb-2">No documents added yet</h3>
                <p className="text-muted-foreground mb-8">
                  Start by connecting your first document source.
                </p>
                <Button onClick={() => setIsConnectModalOpen(true)}>
                  <Plus className="w-4 h-4 mr-2" />
                  Connect Your First Source
                </Button>
              </div>
            ) : (
              <div className="space-y-4">
                {documents.map((doc) => (
                  <div
                    key={doc.id}
                    className="flex items-start justify-between p-4 border border-border rounded-lg hover:bg-muted/50 transition-colors"
                  >
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-2">
                        <File className="w-4 h-4 text-primary flex-shrink-0" />
                        <h3 className="font-medium text-foreground truncate">
                          {doc.name}
                        </h3>
                        <Badge variant="outline" className="text-xs">
                          {doc.status}
                        </Badge>
                      </div>
                      
                      <div className="flex items-center gap-4 text-xs text-muted-foreground mb-2">
                        <span>{doc.doc_type}</span>
                        <span>{doc.source}</span>
                        <span>{doc.size}</span>
                      </div>
                      
                      <div className="flex items-center gap-4 text-xs text-muted-foreground">
                        <div className="flex items-center gap-1">
                          <Calendar className="w-3 h-3" />
                          {new Date(doc.created_at).toLocaleDateString()}
                        </div>
                        {doc.tags.length > 0 && (
                          <div className="flex items-center gap-1">
                            <Tag className="w-3 h-3" />
                            <div className="flex gap-1">
                              {doc.tags.map((tag) => (
                                <Badge key={tag} variant="secondary" className="text-xs">
                                  {tag}
                                </Badge>
                              ))}
                            </div>
                          </div>
                        )}
                      </div>
                    </div>
                    
                    <div className="flex gap-2">
                      <Button variant="ghost" size="sm">
                        <Download className="w-4 h-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => deleteDocument(doc.id)}
                        className="text-red-500 hover:text-red-700 hover:bg-red-50"
                      >
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <AddDocumentModal
        open={isModalOpen}
        onOpenChange={setIsModalOpen}
        onDocumentAdded={fetchDocuments}
      />

      <ConnectSourceModal
        open={isConnectModalOpen}
        onOpenChange={setIsConnectModalOpen}
        onSourceConnected={fetchDocuments}
      />
      
      <Footer />
    </div>
  );
}
