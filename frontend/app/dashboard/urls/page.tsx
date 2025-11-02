"use client";

import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { AddUrlModal } from "@/components/ui/AddUrlModal";
import { BulkUrlImport } from "@/components/ui/BulkUrlImport";
import { ProfileAvatar } from "@/components/ui/ProfileAvatar";
import { Footer } from "@/components/ui/footer";
import { useToast } from "@/hooks/use-toast";
import { apiClient, ApiResponse } from "@/lib/api";
import { UrlRecord } from "@/hooks/use-urls";
import Link from "next/link";
import { 
  Plus, 
  Globe, 
  ExternalLink, 
  Trash2, 
  ArrowLeft,
  Calendar,
  Tag,
  Upload
} from "lucide-react";

export default function UrlsPage() {
  const [urls, setUrls] = useState<UrlRecord[]>([]);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isBulkImportOpen, setIsBulkImportOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const { toast } = useToast();

  const fetchUrls = async () => {
    try {
      const result = await apiClient.getUrls() as ApiResponse<UrlRecord[]>;
      if (result.success) {
        setUrls(result.data || []);
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to fetch URLs",
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  };

  const deleteUrl = async (id: string) => {
    try {
      const result = await apiClient.deleteUrl(id);
      if (result.success) {
        setUrls(prev => prev.filter(url => url.id !== id));
        toast({
          title: "Success",
          description: "URL deleted successfully",
        });
      } else {
        toast({
          title: "Error",
          description: result.message || "Failed to delete URL",
          variant: "destructive",
        });
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to delete URL",
        variant: "destructive",
      });
    }
  };

  useEffect(() => {
    fetchUrls();
  }, []);

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
              <h1 className="text-2xl font-bold text-foreground">URL Management</h1>
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
              <CardTitle className="text-sm font-medium">Total URLs</CardTitle>
              <Globe className="w-4 h-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{urls.length}</div>
            </CardContent>
          </Card>
          
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Active URLs</CardTitle>
              <Globe className="w-4 h-4 text-green-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {urls.filter(url => url.status === 'active').length}
              </div>
            </CardContent>
          </Card>
          
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Total Tags</CardTitle>
              <Tag className="w-4 h-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {new Set(urls.flatMap(url => url.tags)).size}
              </div>
            </CardContent>
          </Card>
        </div>

        {}
        <div className="flex justify-between items-center mb-6">
          <div>
            <h2 className="text-xl font-semibold text-foreground">Your URLs</h2>
            <p className="text-sm text-muted-foreground">Add and organize your URL sources for AI context</p>
          </div>
          <div className="flex gap-3">
            <Button variant="outline" onClick={() => setIsBulkImportOpen(true)}>
              <Upload className="w-4 h-4 mr-2" />
              Bulk Import
            </Button>
            <Button onClick={() => setIsModalOpen(true)}>
              <Plus className="w-4 h-4 mr-2" />
              Add URL
            </Button>
          </div>
        </div>

        {}
        <Card>
          <CardHeader>
            <CardTitle>URL Collection</CardTitle>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="flex items-center justify-center py-8">
                <div className="text-muted-foreground">Loading URLs...</div>
              </div>
            ) : urls.length === 0 ? (
              <div className="text-center py-8">
                <Globe className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
                <h3 className="text-lg font-medium text-foreground mb-2">No URLs added yet</h3>
                <p className="text-muted-foreground mb-4">
                  Start by adding your first URL to the knowledge base.
                </p>
                <Button onClick={() => setIsModalOpen(true)}>
                  <Plus className="w-4 h-4 mr-2" />
                  Add Your First URL
                </Button>
              </div>
            ) : (
              <div className="space-y-4">
                {urls.map((url) => (
                  <div
                    key={url.id}
                    className="flex items-start justify-between p-4 border border-border rounded-lg hover:bg-muted/50 transition-colors"
                  >
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-2">
                        <Globe className="w-4 h-4 text-primary flex-shrink-0" />
                        <h3 className="font-medium text-foreground truncate">
                          {url.title}
                        </h3>
                        <Badge variant="outline" className="text-xs">
                          {url.status}
                        </Badge>
                      </div>
                      
                      <a
                        href={url.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-sm text-primary hover:underline flex items-center gap-1 mb-2"
                      >
                        {url.url}
                        <ExternalLink className="w-3 h-3" />
                      </a>
                      
                      {url.description && (
                        <p className="text-sm text-muted-foreground mb-2">
                          {url.description}
                        </p>
                      )}
                      
                      <div className="flex items-center gap-4 text-xs text-muted-foreground">
                        <div className="flex items-center gap-1">
                          <Calendar className="w-3 h-3" />
                          {new Date(url.created_at).toLocaleDateString()}
                        </div>
                        {url.tags.length > 0 && (
                          <div className="flex items-center gap-1">
                            <Tag className="w-3 h-3" />
                            <div className="flex gap-1">
                              {url.tags.map((tag) => (
                                <Badge key={tag} variant="secondary" className="text-xs">
                                  {tag}
                                </Badge>
                              ))}
                            </div>
                          </div>
                        )}
                      </div>
                    </div>
                    
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => deleteUrl(url.id)}
                      className="text-red-500 hover:text-red-700 hover:bg-red-50"
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <AddUrlModal
        open={isModalOpen}
        onOpenChange={setIsModalOpen}
        onUrlAdded={fetchUrls}
      />
      
      <BulkUrlImport
        open={isBulkImportOpen}
        onOpenChange={setIsBulkImportOpen}
        onImportComplete={fetchUrls}
      />
      
      <Footer />
    </div>
  );
}