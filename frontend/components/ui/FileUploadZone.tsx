"use client";

import { useState, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Upload, File as FileIcon, X, CheckCircle, AlertCircle } from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { apiClient } from "@/lib/api";

interface FileUploadZoneProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onUploadComplete?: () => void;
}

interface UploadFile {
  file: File;
  id: string;
  progress: number;
  status: 'pending' | 'uploading' | 'success' | 'error';
  error?: string;
}

export function FileUploadZone({ open, onOpenChange, onUploadComplete }: FileUploadZoneProps) {
  const [files, setFiles] = useState<UploadFile[]>([]);
  const [isDragging, setIsDragging] = useState(false);
  const { toast } = useToast();

  const acceptedTypes = ['.pdf', '.doc', '.docx', '.txt', '.md', '.rtf', '.ppt', '.pptx'];
  const maxFileSize = 10 * 1024 * 1024; 

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    
    const droppedFiles = Array.from(e.dataTransfer.files);
    addFiles(droppedFiles);
  }, []);

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = Array.from(e.target.files || []);
    addFiles(selectedFiles);
  };

  const addFiles = (newFiles: File[]) => {
    const validFiles = newFiles.filter(file => {
      const extension = '.' + file.name.split('.').pop()?.toLowerCase();
      if (!acceptedTypes.includes(extension)) {
        toast({
          title: "Invalid file type",
          description: `${file.name} is not a supported file type`,
          variant: "destructive",
        });
        return false;
      }
      if (file.size > maxFileSize) {
        toast({
          title: "File too large",
          description: `${file.name} exceeds the 10MB limit`,
          variant: "destructive",
        });
        return false;
      }
      return true;
    });

    const uploadFiles: UploadFile[] = validFiles.map(file => ({
      file,
      id: Math.random().toString(36).substr(2, 9),
      progress: 0,
      status: 'pending'
    }));

    setFiles(prev => [...prev, ...uploadFiles]);
  };

  const removeFile = (id: string) => {
    setFiles(prev => prev.filter(f => f.id !== id));
  };

  const uploadFiles = async () => {
    const pendingFiles = files.filter(f => f.status === 'pending');
    
    for (const uploadFile of pendingFiles) {
      setFiles(prev => prev.map(f => 
        f.id === uploadFile.id ? { ...f, status: 'uploading' } : f
      ));

      try {
        
        const fileExtension = uploadFile.file.name.split('.').pop()?.toLowerCase() || '';
        const docType = getDocumentType(fileExtension);
        const fileSize = formatFileSize(uploadFile.file.size);
        
        // Use shared ApiClient so base URL / env is respected and avoid hardcoded ports
        const apiResp = await apiClient.createDocument({
          name: uploadFile.file.name,
          source: 'Local Upload',
          doc_type: docType,
          size: fileSize,
          tags: [fileExtension, 'uploaded'],
        });

        if (!apiResp || !apiResp.success) throw new Error(apiResp?.error || 'Upload failed');

        
        for (let progress = 0; progress <= 100; progress += 20) {
          await new Promise(resolve => setTimeout(resolve, 50));
          setFiles(prev => prev.map(f => 
            f.id === uploadFile.id ? { ...f, progress } : f
          ));
        }

        setFiles(prev => prev.map(f => 
          f.id === uploadFile.id ? { ...f, status: 'success', progress: 100 } : f
        ));
      } catch (error) {
        setFiles(prev => prev.map(f => 
          f.id === uploadFile.id ? { 
            ...f, 
            status: 'error', 
            error: 'Upload failed' 
          } : f
        ));
      }
    }

    const successCount = files.filter(f => f.status === 'success').length;
    if (successCount > 0) {
      toast({
        title: "Upload Complete",
        description: `${successCount} file(s) uploaded successfully`,
      });
      onUploadComplete?.();
    }
  };

  const getDocumentType = (extension: string): string => {
    const typeMap: Record<string, string> = {
      'pdf': 'PDF',
      'doc': 'Word Document',
      'docx': 'Word Document',
      'txt': 'Text File',
      'md': 'Markdown',
      'rtf': 'Rich Text',
      'ppt': 'PowerPoint',
      'pptx': 'PowerPoint'
    };
    return typeMap[extension] || 'Document';
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const getStatusIcon = (status: UploadFile['status']) => {
    switch (status) {
      case 'success':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'error':
        return <AlertCircle className="w-4 h-4 text-red-500" />;
    default:
      return <FileIcon className="w-4 h-4 text-muted-foreground" />;
    }
  };

  const handleClose = () => {
    setFiles([]);
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Upload Documents</DialogTitle>
          <DialogDescription>
            Drag and drop files or click to browse. Supported formats: PDF, Word, Text, Markdown, PowerPoint
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {}
          <div
            className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors ${
              isDragging 
                ? 'border-primary bg-primary/5' 
                : 'border-muted-foreground/25 hover:border-primary/50'
            }`}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
          >
            <Upload className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
            <div className="space-y-2">
              <p className="text-lg font-medium">Drop files here</p>
              <p className="text-sm text-muted-foreground">
                or{" "}
                <Button
                  variant="link"
                  className="p-0 h-auto"
                  onClick={() => document.getElementById('file-input')?.click()}
                >
                  browse files
                </Button>
              </p>
              <p className="text-xs text-muted-foreground">
                Maximum file size: 10MB
              </p>
            </div>
            <input
              id="file-input"
              type="file"
              multiple
              accept={acceptedTypes.join(',')}
              onChange={handleFileSelect}
              className="hidden"
            />
          </div>

          {}
          {files.length > 0 && (
            <div className="space-y-2 max-h-60 overflow-y-auto">
              <h4 className="font-medium">Selected Files</h4>
              {files.map((uploadFile) => (
                <div key={uploadFile.id} className="flex items-center gap-3 p-3 border rounded-lg">
                  {getStatusIcon(uploadFile.status)}
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium truncate">{uploadFile.file.name}</p>
                    <div className="flex items-center gap-2 text-xs text-muted-foreground">
                      <span>{formatFileSize(uploadFile.file.size)}</span>
                      <Badge variant="outline" className="text-xs">
                        {uploadFile.file.type || 'Unknown'}
                      </Badge>
                    </div>
                    {uploadFile.status === 'uploading' && (
                      <Progress value={uploadFile.progress} className="mt-2 h-1" />
                    )}
                    {uploadFile.error && (
                      <p className="text-xs text-red-500 mt-1">{uploadFile.error}</p>
                    )}
                  </div>
                  {uploadFile.status === 'pending' && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => removeFile(uploadFile.id)}
                    >
                      <X className="w-4 h-4" />
                    </Button>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={handleClose}>
            Cancel
          </Button>
          <Button 
            onClick={uploadFiles}
            disabled={files.length === 0 || files.every(f => f.status !== 'pending')}
          >
            Upload {files.filter(f => f.status === 'pending').length} Files
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}