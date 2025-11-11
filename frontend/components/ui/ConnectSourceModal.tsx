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
  FolderOpen
} from "lucide-react";

interface ConnectSourceModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSourceConnected?: () => void;
}

type ConnectorType = "local_file" | "github" | "google_drive";

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
  const [activeTab, setActiveTab] = useState<ConnectorType>("local_file");
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

  const resetForm = () => {
    setActiveTab("local_file");
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
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="local_file">
              <Upload className="w-4 h-4 mr-2" />
              Local Files
            </TabsTrigger>
            <TabsTrigger value="github">
              <Github className="w-4 h-4 mr-2" />
              GitHub
            </TabsTrigger>
            <TabsTrigger value="google_drive">
              <Cloud className="w-4 h-4 mr-2" />
              Google Drive
            </TabsTrigger>
          </TabsList>

          {/* Local File Upload */}
          <TabsContent value="local_file" className="space-y-4">
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

          {/* GitHub Connection */}
          <TabsContent value="github" className="space-y-4">
            <div className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="github-client-id">GitHub Client ID</Label>
                <Input
                  id="github-client-id"
                  placeholder="Your GitHub OAuth Client ID"
                  value={githubClientId}
                  onChange={(e) => setGithubClientId(e.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="github-client-secret">GitHub Client Secret</Label>
                <Input
                  id="github-client-secret"
                  type="password"
                  placeholder="Your GitHub OAuth Client Secret"
                  value={githubClientSecret}
                  onChange={(e) => setGithubClientSecret(e.target.value)}
                />
              </div>
            </div>

            {connectionStatus.status !== "idle" && (
              <div className="flex items-center gap-2 p-3 bg-muted rounded-md">
                {renderStatusIcon()}
                <span className="text-sm">{connectionStatus.message}</span>
              </div>
            )}

            <Button
              onClick={handleGitHubConnect}
              disabled={connectionStatus.status === "connecting"}
              className="w-full"
            >
              {connectionStatus.status === "connecting" ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Connecting...
                </>
              ) : (
                <>
                  <Github className="w-4 h-4 mr-2" />
                  Connect GitHub
                </>
              )}
            </Button>
          </TabsContent>

          {/* Google Drive Connection */}
          <TabsContent value="google_drive" className="space-y-4">
            <div className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="google-client-id">Google Client ID</Label>
                <Input
                  id="google-client-id"
                  placeholder="Your Google OAuth Client ID"
                  value={googleClientId}
                  onChange={(e) => setGoogleClientId(e.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="google-client-secret">Google Client Secret</Label>
                <Input
                  id="google-client-secret"
                  type="password"
                  placeholder="Your Google OAuth Client Secret"
                  value={googleClientSecret}
                  onChange={(e) => setGoogleClientSecret(e.target.value)}
                />
              </div>
            </div>

            {connectionStatus.status !== "idle" && (
              <div className="flex items-center gap-2 p-3 bg-muted rounded-md">
                {renderStatusIcon()}
                <span className="text-sm">{connectionStatus.message}</span>
              </div>
            )}

            <Button
              onClick={handleGoogleDriveConnect}
              disabled={connectionStatus.status === "connecting"}
              className="w-full"
            >
              {connectionStatus.status === "connecting" ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Connecting...
                </>
              ) : (
                <>
                  <Cloud className="w-4 h-4 mr-2" />
                  Connect Google Drive
                </>
              )}
            </Button>
          </TabsContent>
        </Tabs>

        <div className="mt-4 text-xs text-muted-foreground">
          <p>
            Note: OAuth credentials can be obtained from{" "}
            <a
              href="https://github.com/settings/developers"
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary underline"
            >
              GitHub Developer Settings
            </a>{" "}
            or{" "}
            <a
              href="https://console.cloud.google.com/"
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary underline"
            >
              Google Cloud Console
            </a>
            .
          </p>
        </div>
      </DialogContent>
    </Dialog>
  );
}
