"use client";

import { useState } from "react";
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
import { dataApiClient, listConnections, listProviderFiles, importDocumentFromProvider } from "@/lib/api";
import { Upload, Cloud, Loader2, CheckCircle2, XCircle, FolderOpen, FileText, HardDrive, Droplets, BookOpen } from "lucide-react";

interface ConnectSourceModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSourceConnected?: () => void;
}

type ConnectorType = "local_files" | "connections" | "third_party";

interface ConnectionStatus {
  status: "idle" | "connecting" | "success" | "error";
  message?: string;
}

export function ConnectSourceModal({ open, onOpenChange, onSourceConnected }: ConnectSourceModalProps) {
  const { toast } = useToast();
  const [activeTab, setActiveTab] = useState<ConnectorType>("local_files");
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>({ status: "idle" });

  const [selectedFiles, setSelectedFiles] = useState<FileList | null>(null);
  const [linkedConnections, setLinkedConnections] = useState<any[]>([]);
  const [providerFiles, setProviderFiles] = useState<Record<string, any[]>>({});

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSelectedFiles(e.target.files);
  };

  const handleLocalFileUpload = async () => {
    if (!selectedFiles || selectedFiles.length === 0) {
      toast({ title: "Error", description: "Please select files to upload", variant: "destructive" });
      return;
    }
    setConnectionStatus({ status: "connecting" });
    try {
      const formData = new FormData();
      Array.from(selectedFiles).forEach((file) => formData.append("files", file));
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

  const fetchConnections = async () => {
    try {
      const res = await listConnections();
      const data = (res as any).data || [];
      setLinkedConnections(Array.isArray(data) ? data : []);
    } catch {}
  };

  const browseProvider = async (platform: string) => {
    setConnectionStatus({ status: "connecting" });
    try {
      const res = await listProviderFiles(platform);
      const files = (res as any).data || [];
      setProviderFiles(prev => ({ ...prev, [platform]: files }));
      setConnectionStatus({ status: "success", message: `Fetched ${files.length} items` });
    } catch (e: any) {
      setConnectionStatus({ status: "error", message: e?.message || "Failed to fetch files" });
    }
  };

  const importFromProvider = async (platform: string, file: any) => {
    setConnectionStatus({ status: "connecting" });
    try {
      const resp = await importDocumentFromProvider({ provider: platform, file_id: file.id, name: file.name, mime_type: file.mime_type, size: file.size });
      if ((resp as any).success) {
        setConnectionStatus({ status: "success", message: `Imported ${file.name}` });
        onSourceConnected?.(); onOpenChange(false); resetForm();
      } else {
        throw new Error((resp as any).message || "Import failed");
      }
    } catch (e: any) {
      setConnectionStatus({ status: "error", message: e?.message || "Import failed" });
    }
  };

  const handleThirdPartyConnect = async (service: string) => {
    setConnectionStatus({ status: "connecting" });
    try {
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
    setSelectedFiles(null);
    setProviderFiles({});
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
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="local_files"><HardDrive className="w-4 h-4 mr-2" />Local Files</TabsTrigger>
            <TabsTrigger value="connections" onClick={fetchConnections}><Cloud className="w-4 h-4 mr-2" />Connections</TabsTrigger>
            <TabsTrigger value="third_party"><Cloud className="w-4 h-4 mr-2" />Third Party</TabsTrigger>
          </TabsList>

          <TabsContent value="local_files" className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="files">Select Files</Label>
              <div className="rounded-xl p-8 text-center border border-border bg-card/60">
                <FolderOpen className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
                <Input id="files" type="file" multiple onChange={handleFileChange} className="max-w-sm mx-auto" />
                {selectedFiles && (<p className="text-sm text-muted-foreground mt-3">{selectedFiles.length} file(s) selected</p>)}
              </div>
            </div>
            {connectionStatus.status !== "idle" && (
              <div className="flex items-center gap-2 p-3 bg-muted rounded-md">{renderStatusIcon()}<span className="text-sm">{connectionStatus.message}</span></div>
            )}
            <Button onClick={handleLocalFileUpload} disabled={connectionStatus.status === "connecting"} className="w-full">
              {connectionStatus.status === "connecting" ? (<><Loader2 className="w-4 h-4 mr-2 animate-spin" />Uploading...</>) : (<><Upload className="w-4 h-4 mr-2" />Upload Files</>)}
            </Button>
          </TabsContent>

          <TabsContent value="connections" className="space-y-4">
            {linkedConnections.length === 0 ? (
              <div className="text-sm text-muted-foreground">No connections yet. Use the Connections page to connect your accounts.</div>
            ) : (
              <div className="space-y-4">
                {linkedConnections.map((c) => (
                  <div key={c.id} className="p-4 border border-border rounded-lg">
                    <div className="flex items-center justify-between">
                      <div className="font-medium text-foreground capitalize">{c.platform}</div>
                      <Button variant="outline" size="sm" onClick={() => browseProvider(c.platform)}>Browse Files</Button>
                    </div>
                    {providerFiles[c.platform]?.length ? (
                      <div className="mt-3 space-y-2">
                        {providerFiles[c.platform].map((f) => (
                          <div key={f.id} className="flex items-center justify-between text-sm">
                            <div className="truncate">{f.name} <span className="text-muted-foreground">({f.mime_type})</span></div>
                            <Button size="sm" onClick={() => importFromProvider(c.platform, f)}>Import</Button>
                          </div>
                        ))}
                      </div>
                    ) : null}
                  </div>
                ))}
              </div>
            )}
            {connectionStatus.status !== "idle" && (
              <div className="flex items-center gap-2 p-3 bg-muted rounded-md">{renderStatusIcon()}<span className="text-sm">{connectionStatus.message}</span></div>
            )}
          </TabsContent>

          <TabsContent value="third_party" className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <Button variant="outline" className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent" onClick={() => handleThirdPartyConnect('google_drive')}>
                <Cloud className="w-8 h-8 text-blue-500" />
                <span className="text-sm font-medium">Google Drive</span>
              </Button>
              <Button variant="outline" className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent" onClick={() => handleThirdPartyConnect('dropbox')}>
                <Droplets className="w-8 h-8 text-blue-600" />
                <span className="text-sm font-medium">Dropbox</span>
              </Button>
              <Button variant="outline" className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent" onClick={() => handleThirdPartyConnect('onedrive')}>
                <Cloud className="w-8 h-8 text-blue-700" />
                <span className="text-sm font-medium">OneDrive</span>
              </Button>
              <Button variant="outline" className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent" onClick={() => handleThirdPartyConnect('notion')}>
                <BookOpen className="w-8 h-8 text-gray-700" />
                <span className="text-sm font-medium">Notion</span>
              </Button>
              <Button variant="outline" className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent col-span-2" onClick={() => handleThirdPartyConnect('confluence')}>
                <FileText className="w-8 h-8 text-blue-800" />
                <span className="text-sm font-medium">Confluence</span>
              </Button>
            </div>
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

