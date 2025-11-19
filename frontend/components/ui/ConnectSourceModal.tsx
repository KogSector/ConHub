"use client";

import { useState, useRef } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useToast } from "@/hooks/use-toast";
import { dataApiClient, listConnections, unwrapResponse } from "@/lib/api";
import { Upload, Cloud, Loader2, CheckCircle2, XCircle, FolderOpen, FileText, HardDrive, Droplets, BookOpen } from "lucide-react";

interface ConnectSourceModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSourceConnected?: () => void;
}

type ConnectorType = "local_files" | "third_party";

interface ConnectionStatus {
  status: "idle" | "connecting" | "success" | "error";
  message?: string;
}

export function ConnectSourceModal({ open, onOpenChange, onSourceConnected }: ConnectSourceModalProps) {
  const { toast } = useToast();
  const [activeTab, setActiveTab] = useState<ConnectorType>("local_files");
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>({ status: "idle" });
  const [needsSocialConnect, setNeedsSocialConnect] = useState<string | null>(null);
  const [checkingConnection, setCheckingConnection] = useState(false);

  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || []);
    let combined = [...selectedFiles, ...files];
    if (combined.length > 5) {
      toast({ title: "Limit reached", description: "You can upload up to 5 files", variant: "destructive" });
    }
    combined = combined.slice(0, 5);
    setSelectedFiles(combined);
    e.target.value = "";
  };

  const handleLocalFileUpload = async () => {
    if (!selectedFiles || selectedFiles.length === 0) {
      toast({ title: "Error", description: "Please select files to upload", variant: "destructive" });
      return;
    }
    setConnectionStatus({ status: "connecting" });
    try {
      const formData = new FormData();
      selectedFiles.forEach((file) => formData.append("files", file));
      const result = await dataApiClient.postForm<{ success: boolean; message?: string }>("/api/data/documents/upload", formData);
      if ((result as any).success) {
        setConnectionStatus({ status: "success", message: `Successfully uploaded ${selectedFiles.length} file(s)` });
        toast({ title: "Success", description: `Uploaded ${selectedFiles.length} file(s) successfully` });
        setTimeout(() => { onSourceConnected?.(); onOpenChange(false); resetForm(); }, 1500);
      } else {
        throw new Error((result as any).message || "Upload failed");
      }
    } catch (error) {
      setConnectionStatus({ status: "error", message: "Failed to upload files" });
      toast({ title: "Error", description: "Failed to upload files", variant: "destructive" });
    }
  };

  

  const ensureProviderConnected = async (platform: string): Promise<boolean> => {
    setCheckingConnection(true);
    try {
      const resp = await listConnections();
      const list = unwrapResponse<Array<{ platform: string; is_active: boolean }>>(resp) || [];
      const connected = list.some((c) => c.platform === platform && c.is_active);
      if (!connected) {
        setNeedsSocialConnect(platform);
      } else {
        setNeedsSocialConnect(null);
      }
      return connected;
    } catch {
      return false;
    } finally {
      setCheckingConnection(false);
    }
  };

  const handleThirdPartyConnect = async (service: string) => {
    setConnectionStatus({ status: "connecting" });
    try {
      if (service === 'google_drive' || service === 'dropbox') {
        const connected = await ensureProviderConnected(service);
        if (!connected) {
          setConnectionStatus({ status: "error", message: `Please connect ${service === 'google_drive' ? 'Google Drive' : 'Dropbox'} in Social Connections first` });
          return;
        }
      }
      await new Promise((resolve) => setTimeout(resolve, 1000));
      setConnectionStatus({ status: "success", message: `Successfully connected to ${service}` });
      toast({ title: "Success", description: `Connected to ${service} successfully` });
      setTimeout(() => { onSourceConnected?.(); onOpenChange(false); resetForm(); }, 1000);
    } catch {
      setConnectionStatus({ status: "error", message: `Failed to connect to ${service}` });
      toast({ title: "Error", description: `Failed to connect to ${service}`, variant: "destructive" });
    }
  };

  const resetForm = () => {
    setActiveTab("local_files");
    setConnectionStatus({ status: "idle" });
    setSelectedFiles([]);
  };

  const renderStatusIcon = () => {
    switch (connectionStatus.status) {
      case "connecting": return <Loader2 className="w-5 h-5 animate-spin text-blue-500" />;
      case "success": return <CheckCircle2 className="w-5 h-5 text-green-500" />;
      case "error": return <XCircle className="w-5 h-5 text-red-500" />;
      default: return null;
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[720px]">
        <DialogHeader>
          <DialogTitle>Connect Data Source</DialogTitle>
          <DialogDescription>Connect to repositories, cloud storage, or upload local files</DialogDescription>
        </DialogHeader>

        <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as ConnectorType)} className="mt-4">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="local_files"><HardDrive className="w-4 h-4 mr-2" />Local Files</TabsTrigger>
            <TabsTrigger value="third_party"><Cloud className="w-4 h-4 mr-2" />Third Party</TabsTrigger>
          </TabsList>

          <TabsContent value="local_files" className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="files">Select Files (max 5)</Label>
              <div className="rounded-xl p-10 text-center border border-border bg-card/60 hover:bg-card transition-colors">
                <FolderOpen className="w-12 h-12 mx-auto text-primary mb-6" />
                <input
                  id="files"
                  ref={fileInputRef}
                  type="file"
                  multiple
                  title="Select files to upload"
                  aria-label="Select files to upload"
                  onChange={handleFileChange}
                  className="hidden"
                />
                <Button variant="outline" className="mx-auto" onClick={() => fileInputRef.current?.click()}>Choose Files</Button>
                <div className="mt-3 text-sm text-muted-foreground">
                  {selectedFiles.length > 0 ? `${selectedFiles.length} file(s) selected` : 'No files chosen'}
                </div>
                {selectedFiles.length > 0 && (
                  <div className="mt-4 max-h-36 overflow-auto text-left mx-auto inline-block">
                    {selectedFiles.map((f, i) => (
                      <div key={i} className="text-xs text-muted-foreground">{f.name}</div>
                    ))}
                  </div>
                )}
              </div>
            </div>
            {connectionStatus.status !== "idle" && (
              <div className="flex items-center gap-2 p-3 bg-muted rounded-md">{renderStatusIcon()}<span className="text-sm">{connectionStatus.message}</span></div>
            )}
            <Button onClick={handleLocalFileUpload} disabled={connectionStatus.status === "connecting" || selectedFiles.length === 0} className="w-full">
              {connectionStatus.status === "connecting" ? (<><Loader2 className="w-4 h-4 mr-2 animate-spin" />Uploading...</>) : (<><Upload className="w-4 h-4 mr-2" />Upload Selected ({selectedFiles.length || 0})</>)}
            </Button>
          </TabsContent>

          

          <TabsContent value="third_party" className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <Button variant="outline" title="Connect Google Drive" aria-label="Connect Google Drive" className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent" onClick={() => handleThirdPartyConnect('google_drive')}>
                <Cloud className="w-8 h-8 text-blue-500" />
                <span className="text-sm font-medium">Google Drive</span>
              </Button>
              <Button variant="outline" title="Connect Dropbox" aria-label="Connect Dropbox" className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent" onClick={() => handleThirdPartyConnect('dropbox')}>
                <Droplets className="w-8 h-8 text-blue-600" />
                <span className="text-sm font-medium">Dropbox</span>
              </Button>
              <Button variant="outline" title="Connect OneDrive" aria-label="Connect OneDrive" className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent" onClick={() => handleThirdPartyConnect('onedrive')}>
                <Cloud className="w-8 h-8 text-blue-700" />
                <span className="text-sm font-medium">OneDrive</span>
              </Button>
              <Button variant="outline" title="Connect Notion" aria-label="Connect Notion" className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent" onClick={() => handleThirdPartyConnect('notion')}>
                <BookOpen className="w-8 h-8 text-gray-700" />
                <span className="text-sm font-medium">Notion</span>
              </Button>
              <Button variant="outline" title="Connect Confluence" aria-label="Connect Confluence" className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent col-span-2" onClick={() => handleThirdPartyConnect('confluence')}>
                <FileText className="w-8 h-8 text-blue-800" />
                <span className="text-sm font-medium">Confluence</span>
              </Button>
            </div>
            {needsSocialConnect && (
              <div className="p-3 bg-yellow-50 border border-yellow-200 rounded-md">
                <p className="text-sm text-yellow-800">
                  {`You need to connect ${needsSocialConnect === 'google_drive' ? 'Google Drive' : 'Dropbox'} before proceeding.`}
                </p>
                <div className="mt-2">
                  <Button
                    title="Go to Social Connections"
                    aria-label="Go to Social Connections"
                    className="bg-blue-600 hover:bg-blue-700 text-white"
                    onClick={() => { window.location.href = '/dashboard/social'; }}
                    disabled={checkingConnection}
                  >
                    Go to Social Connections
                  </Button>
                </div>
              </div>
            )}
            {connectionStatus.status !== "idle" && (
              <div className="flex items-center gap-2 p-3 bg-muted rounded-md">{renderStatusIcon()}<span className="text-sm">{connectionStatus.message}</span></div>
            )}
          </TabsContent>
        </Tabs>

        <div className="mt-4 text-xs text-muted-foreground">
          <p>
            <strong>Local Files:</strong> Upload documents directly from your computer.<br/>
            <strong>Third Party:</strong> Connect to cloud storage and document platforms for seamless access.
          </p>
        </div>
      </DialogContent>
    </Dialog>
  );
}
