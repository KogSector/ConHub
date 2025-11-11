'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ProfileAvatar } from "@/components/ui/ProfileAvatar";
import { Footer } from "@/components/ui/footer";
import { ConnectSourceModal } from "@/components/ui/ConnectSourceModal";
import { ArrowLeft, FileText, Plus, Upload, Cloud, HardDrive, RefreshCw, Trash2, Download } from "lucide-react";
import Link from "next/link";

interface DocumentSource {
  id: string;
  name: string;
  type: 'local_files' | 'google_drive' | 'dropbox' | 'onedrive' | 'notion' | 'confluence';
  status: 'connected' | 'syncing' | 'error' | 'disconnected';
  documentCount: number;
  lastSync?: string;
  size?: string;
}

export default function DocumentsPage() {
  const [sources, setSources] = useState<DocumentSource[]>([]);
  const [showConnectModal, setShowConnectModal] = useState(false);
  const [loading, setLoading] = useState(true);

  // Sample data for demonstration
  const sampleSources: DocumentSource[] = [
    {
      id: "1",
      name: "Local Files",
      type: "local_files",
      status: "connected",
      documentCount: 12,
      lastSync: "2 hours ago",
      size: "2.4 MB"
    },
    {
      id: "2", 
      name: "Google Drive",
      type: "google_drive",
      status: "syncing",
      documentCount: 45,
      lastSync: "5 minutes ago",
      size: "15.2 MB"
    }
  ];

  useEffect(() => {
    // Simulate loading
    setTimeout(() => {
      setSources(sampleSources);
      setLoading(false);
    }, 1000);
  }, []);

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'local_files': return <HardDrive className="w-5 h-5 text-gray-600" />;
      case 'google_drive': return <Cloud className="w-5 h-5 text-blue-500" />;
      case 'dropbox': return <Cloud className="w-5 h-5 text-blue-600" />;
      case 'onedrive': return <Cloud className="w-5 h-5 text-blue-700" />;
      case 'notion': return <FileText className="w-5 h-5 text-gray-700" />;
      case 'confluence': return <FileText className="w-5 h-5 text-blue-800" />;
      default: return <FileText className="w-5 h-5 text-gray-500" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'connected': return 'bg-green-500 shadow-lg shadow-green-500/50';
      case 'syncing': return 'bg-blue-500 shadow-lg shadow-blue-500/50';
      case 'error': return 'bg-red-500 shadow-lg shadow-red-500/50';
      default: return 'bg-gray-400';
    }
  };

  const getTypeName = (type: string) => {
    switch (type) {
      case 'local_files': return 'Local Files';
      case 'google_drive': return 'Google Drive';
      case 'dropbox': return 'Dropbox';
      case 'onedrive': return 'OneDrive';
      case 'notion': return 'Notion';
      case 'confluence': return 'Confluence';
      default: return type;
    }
  };

  const handleSourceConnected = () => {
    // Refresh sources list
    setLoading(true);
    setTimeout(() => {
      setSources(sampleSources);
      setLoading(false);
    }, 1000);
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center">
          <RefreshCw className="w-8 h-8 animate-spin mx-auto mb-4" />
          <p className="text-muted-foreground">Loading document sources...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <div className="border-b border-border bg-card/50 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-20">
            <div className="flex items-center space-x-4">
              <Link href="/dashboard">
                <Button variant="ghost" size="sm">
                  <ArrowLeft className="w-4 h-4" />
                </Button>
              </Link>
              <div className="flex items-center space-x-3">
                <FileText className="w-6 h-6 text-primary" />
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
        {/* Stats Cards */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Total Sources
              </CardTitle>
              <Cloud className="w-4 h-4 text-primary" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">{sources.length}</div>
              <p className="text-xs text-muted-foreground">
                Connected sources
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Total Documents
              </CardTitle>
              <FileText className="w-4 h-4 text-green-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">
                {sources.reduce((sum, source) => sum + source.documentCount, 0)}
              </div>
              <p className="text-xs text-muted-foreground">
                Indexed documents
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Active Sources
              </CardTitle>
              <Cloud className="w-4 h-4 text-blue-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">
                {sources.filter(s => s.status === 'connected').length}
              </div>
              <p className="text-xs text-muted-foreground">
                Currently active
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Total Size
              </CardTitle>
              <HardDrive className="w-4 h-4 text-purple-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">17.6 MB</div>
              <p className="text-xs text-muted-foreground">
                Storage used
              </p>
            </CardContent>
          </Card>
        </div>

        {/* Action Bar */}
        <div className="flex justify-between items-center mb-6">
          <div>
            <h2 className="text-xl font-semibold text-foreground">Document Sources</h2>
            <p className="text-sm text-muted-foreground">Connect and manage your document sources</p>
          </div>
          <div className="flex gap-3">
            <Button variant="outline" onClick={() => handleSourceConnected()}>
              <RefreshCw className="w-4 h-4 mr-2" />
              Refresh
            </Button>
            <Button onClick={() => setShowConnectModal(true)}>
              <Plus className="w-4 h-4 mr-2" />
              Connect Source
            </Button>
          </div>
        </div>

        {/* Sources List */}
        <div className="space-y-4">
          {sources.length === 0 ? (
            <Card className="bg-muted/50 border-dashed border-muted-foreground/25">
              <CardContent className="flex flex-col items-center justify-center py-12">
                <FileText className="w-12 h-12 text-muted-foreground mb-4" />
                <h3 className="text-lg font-medium text-foreground mb-2">No document sources connected</h3>
                <p className="text-muted-foreground text-center mb-6 max-w-md">
                  Connect your first document source to start indexing and accessing your documents with AI.
                </p>
                <Button onClick={() => setShowConnectModal(true)}>
                  <Plus className="w-4 h-4 mr-2" />
                  Connect Your First Source
                </Button>
              </CardContent>
            </Card>
          ) : (
            <div className="grid gap-4">
              {sources.map((source) => (
                <Card key={source.id} className="bg-card border-border hover:bg-accent/5 transition-colors">
                  <div className="flex flex-col px-6 py-4 gap-4">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        {getTypeIcon(source.type)}
                        <div>
                          <h3 className="font-semibold text-foreground">{source.name}</h3>
                          <p className="text-sm text-muted-foreground">{getTypeName(source.type)}</p>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <div className={`w-2 h-2 rounded-full ${getStatusColor(source.status)}`}></div>
                        <Badge variant="outline" className="text-xs">
                          {source.status}
                        </Badge>
                      </div>
                    </div>

                    <div className="flex items-center gap-6 text-sm text-muted-foreground">
                      <span>{source.documentCount} documents</span>
                      {source.size && <span>{source.size}</span>}
                      {source.lastSync && <span>Last sync: {source.lastSync}</span>}
                    </div>

                    <div className="flex items-center gap-2">
                      <Button variant="outline" size="sm">
                        <RefreshCw className={`w-4 h-4 mr-1 ${source.status === 'syncing' ? 'animate-spin' : ''}`} />
                        {source.status === 'syncing' ? 'Syncing...' : 'Sync'}
                      </Button>
                      <Button variant="outline" size="sm">
                        <Download className="w-4 h-4 mr-1" />
                        Export
                      </Button>
                      <Button variant="outline" size="sm" className="text-red-500 hover:text-red-700">
                        <Trash2 className="w-4 h-4 mr-1" />
                        Disconnect
                      </Button>
                    </div>
                  </div>
                </Card>
              ))}
            </div>
          )}
        </div>
      </div>

      <Footer />

      <ConnectSourceModal
        open={showConnectModal}
        onOpenChange={setShowConnectModal}
        onSourceConnected={handleSourceConnected}
      />
    </div>
  );
}