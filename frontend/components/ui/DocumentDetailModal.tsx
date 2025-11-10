"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { FileText, Calendar, Tag, HardDrive, Trash2 } from "lucide-react";

interface Document {
  id: string;
  name: string;
  source: string;
  doc_type: string;
  size: string;
  tags: string[];
  created_at: string;
  status: string;
}

interface DocumentDetailModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  documents: Document[];
  onDocumentDeleted?: () => void;
}

export function DocumentDetailModal({ 
  open, 
  onOpenChange, 
  documents, 
  onDocumentDeleted 
}: DocumentDetailModalProps) {
  const [selectedDoc, setSelectedDoc] = useState<string | null>(null);

  const handleDeleteDocument = (docId: string) => {
    const updatedDocs = documents.filter(doc => doc.id !== docId);
    localStorage.setItem('uploadedDocuments', JSON.stringify(updatedDocs));
    onDocumentDeleted?.();
    if (updatedDocs.length === 0) {
      onOpenChange(false);
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[700px] max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FileText className="w-5 h-5" />
            Document Collection ({documents.length} files)
          </DialogTitle>
          <DialogDescription>
            View and manage your uploaded documents
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 max-h-[60vh] overflow-y-auto">
          {documents.map((doc) => (
            <div 
              key={doc.id} 
              className="border rounded-lg p-4 hover:bg-accent/5 transition-colors"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-2">
                    <FileText className="w-4 h-4 text-muted-foreground flex-shrink-0" />
                    <h3 className="font-medium truncate">{doc.name}</h3>
                    <Badge variant="outline" className="text-xs">
                      {doc.doc_type}
                    </Badge>
                  </div>
                  
                  <div className="grid grid-cols-2 gap-4 text-sm text-muted-foreground mb-3">
                    <div className="flex items-center gap-1">
                      <HardDrive className="w-3 h-3" />
                      <span>{doc.source}</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <Calendar className="w-3 h-3" />
                      <span>{formatDate(doc.created_at)}</span>
                    </div>
                  </div>

                  <div className="flex items-center gap-2 mb-2">
                    <span className="text-xs text-muted-foreground">Size:</span>
                    <span className="text-xs font-medium">{doc.size}</span>
                  </div>

                  {doc.tags && doc.tags.length > 0 && (
                    <div className="flex items-center gap-2 flex-wrap">
                      <Tag className="w-3 h-3 text-muted-foreground" />
                      {doc.tags.map((tag, index) => (
                        <Badge key={index} variant="secondary" className="text-xs">
                          {tag}
                        </Badge>
                      ))}
                    </div>
                  )}
                </div>

                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => handleDeleteDocument(doc.id)}
                  className="text-red-500 hover:text-red-700 hover:bg-red-50"
                >
                  <Trash2 className="w-4 h-4" />
                </Button>
              </div>
            </div>
          ))}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Close
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}