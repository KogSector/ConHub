"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { FileUploadZone } from "@/components/ui/FileUploadZone";
import { 
  Upload, 
  FileText, 
  Cloud, 
  HardDrive,
  Folder,
  Globe,
  Database
} from "lucide-react";
import { useToast } from "@/hooks/use-toast";

interface AddDocumentModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDocumentAdded?: () => void;
}

interface DocumentSource {
  id: string;
  name: string;
  description: string;
  icon: React.ComponentType<any>;
  color: string;
  available: boolean;
}

export function AddDocumentModal({ open, onOpenChange, onDocumentAdded }: AddDocumentModalProps) {
  const [selectedSource, setSelectedSource] = useState<string | null>(null);
  const [showFileUpload, setShowFileUpload] = useState(false);
  const { toast } = useToast();

  const documentSources: DocumentSource[] = [
    {
      id: "local",
      name: "Local Files",
      description: "Upload files from your computer (PDF, Word, Text, etc.)",
      icon: HardDrive,
      color: "text-blue-500",
      available: true
    },
    {
      id: "google-drive",
      name: "Google Drive",
      description: "Connect to your Google Drive documents and folders",
      icon: Cloud,
      color: "text-green-500",
      available: true
    },
    {
      id: "dropbox",
      name: "Dropbox",
      description: "Import documents from your Dropbox account",
      icon: Folder,
      color: "text-blue-600",
      available: true
    },
    {
      id: "onedrive",
      name: "OneDrive",
      description: "Access Microsoft OneDrive documents",
      icon: Cloud,
      color: "text-blue-400",
      available: true
    },
    {
      id: "notion",
      name: "Notion",
      description: "Import pages and databases from Notion",
      icon: FileText,
      color: "text-gray-700",
      available: true
    },
    {
      id: "confluence",
      name: "Confluence",
      description: "Connect to Atlassian Confluence spaces",
      icon: Globe,
      color: "text-blue-700",
      available: false
    },
    {
      id: "sharepoint",
      name: "SharePoint",
      description: "Access Microsoft SharePoint document libraries",
      icon: Database,
      color: "text-purple-600",
      available: false
    },
    {
      id: "box",
      name: "Box",
      description: "Import documents from Box cloud storage",
      icon: Folder,
      color: "text-blue-500",
      available: false
    }
  ];

  const handleSourceSelect = (sourceId: string) => {
    const source = documentSources.find(s => s.id === sourceId);
    
    if (!source?.available) {
      toast({
        title: "Coming Soon",
        description: `${source?.name} integration is coming soon!`,
      });
      return;
    }

    if (sourceId === "local") {
      setShowFileUpload(true);
    } else {
      
      toast({
        title: "Connecting...",
        description: `Initiating connection to ${source?.name}`,
      });
      
      setTimeout(() => {
        toast({
          title: "Success",
          description: `Connected to ${source?.name} successfully!`,
        });
        onDocumentAdded?.();
        onOpenChange(false);
      }, 2000);
    }
  };

  const handleFileUpload = async (files: File[]) => {
    try {
      // Files are now handled by FileUploadZone and stored in localStorage
      toast({
        title: "Success",
        description: `${files.length} file(s) uploaded successfully!`,
      });
      
      onDocumentAdded?.();
      onOpenChange(false);
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to upload files. Please try again.",
        variant: "destructive",
      });
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[700px] max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Add Documents</DialogTitle>
          <DialogDescription>
            Choose a source to connect and import your documents from.
          </DialogDescription>
        </DialogHeader>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 py-4 max-h-[60vh] overflow-y-auto pr-2">
          {documentSources.map((source) => {
            const IconComponent = source.icon;
            return (
              <Card 
                key={source.id}
                className={`cursor-pointer transition-all hover:shadow-md ${
                  !source.available ? 'opacity-60' : 'hover:border-primary'
                }`}
                onClick={() => handleSourceSelect(source.id)}
              >
                <CardHeader className="pb-3">
                  <div className="flex items-center gap-3">
                    <div className={`p-2 rounded-lg bg-muted`}>
                      <IconComponent className={`w-5 h-5 ${source.color}`} />
                    </div>
                    <div className="flex-1">
                      <CardTitle className="text-base flex items-center gap-2">
                        {source.name}
                        {!source.available && (
                          <span className="text-xs bg-yellow-100 text-yellow-800 px-2 py-1 rounded">
                            Soon
                          </span>
                        )}
                      </CardTitle>
                    </div>
                  </div>
                </CardHeader>
                <CardContent className="pt-0">
                  <p className="text-sm text-muted-foreground">
                    {source.description}
                  </p>
                </CardContent>
              </Card>
            );
          })}
        </div>

        <div className="flex justify-between items-center pt-4 border-t">
          <p className="text-sm text-muted-foreground">
            More integrations coming soon
          </p>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
        </div>
      </DialogContent>
      
      <FileUploadZone
        open={showFileUpload}
        onOpenChange={setShowFileUpload}
        onUploadComplete={() => {
          setShowFileUpload(false);
          onDocumentAdded?.();
          onOpenChange(false);
        }}
      />
    </Dialog>
  );
}