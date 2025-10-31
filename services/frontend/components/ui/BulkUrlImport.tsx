"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Upload, Loader2, CheckCircle, XCircle } from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { apiClient } from "@/lib/api";

interface BulkUrlImportProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onImportComplete?: () => void;
}

interface ImportResult {
  url: string;
  success: boolean;
  error?: string;
}

export function BulkUrlImport({ open, onOpenChange, onImportComplete }: BulkUrlImportProps) {
  const [urlText, setUrlText] = useState("");
  const [isImporting, setIsImporting] = useState(false);
  const [progress, setProgress] = useState(0);
  const [results, setResults] = useState<ImportResult[]>([]);
  const { toast } = useToast();

  const parseUrls = (text: string): string[] => {
    return text
      .split(/[\n,]/)
      .map(url => url.trim())
      .filter(url => url.length > 0);
  };

  const validateUrl = (url: string): boolean => {
    try {
      new URL(url);
      return true;
    } catch {
      return false;
    }
  };

  const importUrls = async () => {
    const urls = parseUrls(urlText);
    if (urls.length === 0) {
      toast({
        title: "Error",
        description: "Please enter at least one URL",
        variant: "destructive",
      });
      return;
    }

    setIsImporting(true);
    setProgress(0);
    setResults([]);

    const importResults: ImportResult[] = [];

    for (let i = 0; i < urls.length; i++) {
      const url = urls[i];
      
      if (!validateUrl(url)) {
        importResults.push({
          url,
          success: false,
          error: "Invalid URL format"
        });
      } else {
        try {
          const result = await apiClient.createUrl({ url });
          importResults.push({
            url,
            success: !!result?.success,
            error: result?.success ? undefined : result?.error || result?.message || 'Failed',
          });
        } catch (error) {
          importResults.push({
            url,
            success: false,
            error: 'Network error',
          });
        }
      }

      setProgress(((i + 1) / urls.length) * 100);
      setResults([...importResults]);
    }

    setIsImporting(false);
    
    const successCount = importResults.filter(r => r.success).length;
    const failCount = importResults.length - successCount;

    toast({
      title: "Import Complete",
      description: `${successCount} URLs imported successfully${failCount > 0 ? `, ${failCount} failed` : ''}`,
    });

    if (successCount > 0) {
      onImportComplete?.();
    }
  };

  const handleClose = () => {
    setUrlText("");
    setResults([]);
    setProgress(0);
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Bulk URL Import</DialogTitle>
          <DialogDescription>
            Import multiple URLs at once. Enter one URL per line or separate with commas.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="urls">URLs</Label>
            <Textarea
              id="urls"
              placeholder="https://example.com&#10;https://docs.example.com&#10;https://api.example.com"
              value={urlText}
              onChange={(e) => setUrlText(e.target.value)}
              rows={8}
              disabled={isImporting}
            />
            <p className="text-sm text-muted-foreground">
              {parseUrls(urlText).length} URLs detected
            </p>
          </div>

          {isImporting && (
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Importing URLs...</span>
                <span className="text-sm text-muted-foreground">{Math.round(progress)}%</span>
              </div>
              <Progress value={progress} />
            </div>
          )}

          {results.length > 0 && (
            <div className="space-y-2 max-h-40 overflow-y-auto">
              <Label>Import Results</Label>
              <div className="space-y-1">
                {results.map((result, index) => (
                  <div key={index} className="flex items-center gap-2 text-sm">
                    {result.success ? (
                      <CheckCircle className="w-4 h-4 text-green-500" />
                    ) : (
                      <XCircle className="w-4 h-4 text-red-500" />
                    )}
                    <span className="flex-1 truncate">{result.url}</span>
                    {result.error && (
                      <span className="text-red-500 text-xs">{result.error}</span>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button
            type="button"
            variant="outline"
            onClick={handleClose}
            disabled={isImporting}
          >
            {results.length > 0 ? "Close" : "Cancel"}
          </Button>
          {results.length === 0 && (
            <Button onClick={importUrls} disabled={isImporting || !urlText.trim()}>
              {isImporting ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Importing...
                </>
              ) : (
                <>
                  <Upload className="w-4 h-4 mr-2" />
                  Import URLs
                </>
              )}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}