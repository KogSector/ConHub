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
import { apiClient } from "@/lib/api";
import {
  Upload,
  Github,
  Cloud,
  Loader2,
  CheckCircle2,
  XCircle,
  FolderOpen,
  FileText,
  HardDrive,
  Droplets,
  BookOpen
} from "lucide-react";

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

interface ConnectorResponse {
  success: boolean;
  error?: string;
  account?: {
    status: string;
    credentials?: {
      auth_url?: string;
    };
  };
}

export function ConnectSourceModal({
  open,
  onOpenChange,
  onSourceConnected,
}: ConnectSourceModalProps) {
  const { toast } = useToast();
  const [activeTab, setActiveTab] = useState<ConnectorType>("local_files");
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>({
    status: "idle",
  });

  // GitHub credentials
  const [githubClientId, setGithubClientId] = useState("");
  const [githubClientSecret, setGithubClientSecret] = useState("");

  // Google Drive credentials
  const [googleClientId, setGoogleClientId] = useState("");
  const [googleClientSecret, setGoogleClientSecret] = useState("");

  // File upload
  const [selectedFiles, setSelectedFiles] = useState<FileList | null>(null);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSelectedFiles(e.target.files);
  };

  const handleLocalFileUpload = async () => {
    if (!selectedFiles || selectedFiles.length === 0) {
      toast({
        title: "Error",
        description: "Please select files to upload",
        variant: "destructive",
      });
      return;
    }

    setConnectionStatus({ status: "connecting" });

    try {
      // TODO: Implement file upload to local_file connector
      // For now, just simulate the upload
      await new Promise((resolve) => setTimeout(resolve, 1500));

      setConnectionStatus({
        status: "success",
        message: `Successfully uploaded ${selectedFiles.length} file(s)`,
      });

      toast({
        title: "Success",
        description: `Uploaded ${selectedFiles.length} file(s) successfully`,
      });

      setTimeout(() => {
        onSourceConnected?.();
        onOpenChange(false);
        resetForm();
      }, 2000);
    } catch (error) {
      console.error("Upload error:", error);
      setConnectionStatus({
        status: "error",
        message: "Failed to upload files",
      });
      toast({
        title: "Error",
        description: "Failed to upload files",
        variant: "destructive",
      });
    }
  };

  const handleGitHubConnect = async () => {
    if (!githubClientId || !githubClientSecret) {
      toast({
        title: "Error",
        description: "Please provide GitHub OAuth credentials",
        variant: "destructive",
      });
      return;
    }

    setConnectionStatus({ status: "connecting" });

    try {
      const result = await apiClient.post<ConnectorResponse>("/api/connectors/connect", {
        connector_type: "github",
        account_name: "GitHub Account",
        credentials: {
          client_id: githubClientId,
          client_secret: githubClientSecret,
        },
        settings: {},
      });

      if (result.success && result.account) {
        // Check if OAuth is required
        if (result.account.status === "pending_auth" && result.account.credentials?.auth_url) {
          setConnectionStatus({
            status: "success",
            message: "Redirecting to GitHub for authorization...",
          });

          // Open OAuth URL
          window.open(result.account.credentials.auth_url, "_blank");

          toast({
            title: "Authorization Required",
            description: "Please complete the GitHub authorization in the opened window",
          });
        } else {
          setConnectionStatus({
            status: "success",
            message: "Successfully connected to GitHub",
          });

          toast({
            title: "Success",
            description: "Connected to GitHub successfully",
          });

          setTimeout(() => {
            onSourceConnected?.();
            onOpenChange(false);
            resetForm();
          }, 2000);
        }
      } else {
        throw new Error(result.error || "Failed to connect");
      }
    } catch (error: any) {
      console.error("GitHub connection error:", error);
      setConnectionStatus({
        status: "error",
        message: error.message || "Failed to connect to GitHub",
      });
      toast({
        title: "Error",
        description: error.message || "Failed to connect to GitHub",
        variant: "destructive",
      });
    }
  };

  const handleGoogleDriveConnect = async () => {
    if (!googleClientId || !googleClientSecret) {
      toast({
        title: "Error",
        description: "Please provide Google OAuth credentials",
        variant: "destructive",
      });
      return;
    }

    setConnectionStatus({ status: "connecting" });

    try {
      const result = await apiClient.post<ConnectorResponse>("/api/connectors/connect", {
        connector_type: "google_drive",
        account_name: "Google Drive",
        credentials: {
          client_id: googleClientId,
          client_secret: googleClientSecret,
        },
        settings: {},
      });

      if (result.success && result.account) {
        // Check if OAuth is required
        if (result.account.status === "pending_auth" && result.account.credentials?.auth_url) {
          setConnectionStatus({
            status: "success",
            message: "Redirecting to Google for authorization...",
          });

          // Open OAuth URL
          window.open(result.account.credentials.auth_url, "_blank");

          toast({
            title: "Authorization Required",
            description: "Please complete the Google authorization in the opened window",
          });
        } else {
          setConnectionStatus({
            status: "success",
            message: "Successfully connected to Google Drive",
          });

          toast({
            title: "Success",
            description: "Connected to Google Drive successfully",
          });

          setTimeout(() => {
            onSourceConnected?.();
            onOpenChange(false);
            resetForm();
          }, 2000);
        }
      } else {
        throw new Error(result.error || "Failed to connect");
      }
    } catch (error: any) {
      console.error("Google Drive connection error:", error);
      setConnectionStatus({
        status: "error",
        message: error.message || "Failed to connect to Google Drive",
      });
      toast({
        title: "Error",
        description: error.message || "Failed to connect to Google Drive",
        variant: "destructive",
      });
    }
  };

  const handleThirdPartyConnect = async (service: string) => {
    setConnectionStatus({ status: "connecting" });

    try {
      // Simulate OAuth flow for third-party services
      await new Promise((resolve) => setTimeout(resolve, 1500));

      setConnectionStatus({
        status: "success",
        message: `Successfully connected to ${service.charAt(0).toUpperCase() + service.slice(1)}`,
      });

      toast({
        title: "Success",
        description: `Connected to ${service.charAt(0).toUpperCase() + service.slice(1)} successfully`,
      });

      setTimeout(() => {
        onSourceConnected?.();
        onOpenChange(false);
        resetForm();
      }, 2000);
    } catch (error) {
      console.error(`${service} connection error:`, error);
      setConnectionStatus({
        status: "error",
        message: `Failed to connect to ${service}`,
      });
      toast({
        title: "Error",
        description: `Failed to connect to ${service}`,
        variant: "destructive",
      });
    }
  };

  const resetForm = () => {
    setActiveTab("local_files");
    setConnectionStatus({ status: "idle" });
    setGithubClientId("");
    setGithubClientSecret("");
    setGoogleClientId("");
    setGoogleClientSecret("");
    setSelectedFiles(null);
  };

  const renderStatusIcon = () => {
    switch (connectionStatus.status) {
      case "connecting":
        return <Loader2 className="w-5 h-5 animate-spin text-blue-500" />;
      case "success":
        return <CheckCircle2 className="w-5 h-5 text-green-500" />;
      case "error":
        return <XCircle className="w-5 h-5 text-red-500" />;
      default:
        return null;
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Connect Data Source</DialogTitle>
          <DialogDescription>
            Connect to your repositories, file storage, or upload local files
          </DialogDescription>
        </DialogHeader>

        <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as ConnectorType)} className="mt-4">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="local_files">
              <HardDrive className="w-4 h-4 mr-2" />
              Local Files
            </TabsTrigger>
            <TabsTrigger value="third_party">
              <Cloud className="w-4 h-4 mr-2" />
              Third Party
            </TabsTrigger>
          </TabsList>

          <TabsContent value="local_files" className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="files">Select Files</Label>
              <div className="border-2 border-dashed border-border rounded-lg p-8 text-center">
                <FolderOpen className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
                <Input
                  id="files"
                  type="file"
                  multiple
                  onChange={handleFileChange}
                  className="max-w-xs mx-auto"
                />
                {selectedFiles && (
                  <p className="text-sm text-muted-foreground mt-2">
                    {selectedFiles.length} file(s) selected
                  </p>
                )}
              </div>
            </div>

            {connectionStatus.status !== "idle" && (
              <div className="flex items-center gap-2 p-3 bg-muted rounded-md">
                {renderStatusIcon()}
                <span className="text-sm">{connectionStatus.message}</span>
              </div>
            )}

            <Button
              onClick={handleLocalFileUpload}
              disabled={connectionStatus.status === "connecting"}
              className="w-full"
            >
              {connectionStatus.status === "connecting" ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Uploading...
                </>
              ) : (
                <>
                  <Upload className="w-4 h-4 mr-2" />
                  Upload Files
                </>
              )}
            </Button>
          </TabsContent>

          <TabsContent value="third_party" className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <Button
                variant="outline"
                className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent"
                onClick={() => handleThirdPartyConnect('google_drive')}
              >
                <Cloud className="w-8 h-8 text-blue-500" />
                <span className="text-sm font-medium">Google Drive</span>
              </Button>

              <Button
                variant="outline"
                className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent"
                onClick={() => handleThirdPartyConnect('dropbox')}
              >
                <Droplets className="w-8 h-8 text-blue-600" />
                <span className="text-sm font-medium">Dropbox</span>
              </Button>

              <Button
                variant="outline"
                className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent"
                onClick={() => handleThirdPartyConnect('onedrive')}
              >
                <Cloud className="w-8 h-8 text-blue-700" />
                <span className="text-sm font-medium">OneDrive</span>
              </Button>

              <Button
                variant="outline"
                className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent"
                onClick={() => handleThirdPartyConnect('notion')}
              >
                <BookOpen className="w-8 h-8 text-gray-700" />
                <span className="text-sm font-medium">Notion</span>
              </Button>

              <Button
                variant="outline"
                className="h-24 flex flex-col items-center justify-center space-y-2 hover:bg-accent col-span-2"
                onClick={() => handleThirdPartyConnect('confluence')}
              >
                <FileText className="w-8 h-8 text-blue-800" />
                <span className="text-sm font-medium">Confluence</span>
              </Button>
            </div>

            {connectionStatus.status !== "idle" && (
              <div className="flex items-center gap-2 p-3 bg-muted rounded-md">
                {renderStatusIcon()}
                <span className="text-sm">{connectionStatus.message}</span>
              </div>
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
