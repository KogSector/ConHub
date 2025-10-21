"use client";

import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { Checkbox } from "@/components/ui/checkbox";
import { useToast } from "@/hooks/use-toast";
import { apiClient, AgentRecord, UpdateAgentRequest, AgentConfig } from "@/lib/api";
import { Badge } from "@/components/ui/badge";

interface EditAgentModalProps {
  agent: AgentRecord;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onAgentUpdated: () => void;
}

const PERMISSIONS = [
  { value: "read", label: "Read Access", description: "Allow agent to read data" },
  { value: "write", label: "Write Access", description: "Allow agent to modify data" },
  { value: "context", label: "Context Access", description: "Provide unified context to agent" },
  { value: "repositories", label: "Repository Access", description: "Access to connected repositories" },
  { value: "documents", label: "Document Access", description: "Access to uploaded documents" },
  { value: "urls", label: "URL Access", description: "Access to saved URLs" },
];

const OPENAI_MODELS = [
  "gpt-4",
  "gpt-4-turbo",
  "gpt-3.5-turbo",
];

const ANTHROPIC_MODELS = [
  "claude-3-opus-20240229",
  "claude-3-sonnet-20240229",
  "claude-3-haiku-20240307",
];

const STATUS_OPTIONS = [
  { value: "Connected", label: "Connected" },
  { value: "Pending", label: "Pending" },
  { value: "Error", label: "Error" },
  { value: "Inactive", label: "Inactive" },
];

export function EditAgentModal({ agent, open, onOpenChange, onAgentUpdated }: EditAgentModalProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [formData, setFormData] = useState({
    name: "",
    endpoint: "",
    api_key: "",
    permissions: [] as string[],
    status: "Connected" as AgentRecord['status'],
    config: {
      model: "",
      temperature: 0.7,
      max_tokens: 1000,
      timeout: 30,
      custom_instructions: "",
    } as AgentConfig,
  });
  const { toast } = useToast();

  useEffect(() => {
    if (agent && open) {
      setFormData({
        name: agent.name,
        endpoint: agent.endpoint || "",
        api_key: agent.api_key,
        permissions: agent.permissions,
        status: agent.status,
        config: agent.config,
      });
    }
  }, [agent, open]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);

    try {
      const request: UpdateAgentRequest = {
        name: formData.name !== agent.name ? formData.name : undefined,
        endpoint: formData.endpoint !== agent.endpoint ? formData.endpoint || undefined : undefined,
        api_key: formData.api_key !== agent.api_key ? formData.api_key : undefined,
        permissions: JSON.stringify(formData.permissions) !== JSON.stringify(agent.permissions) 
          ? formData.permissions : undefined,
        status: formData.status !== agent.status ? formData.status : undefined,
        config: JSON.stringify(formData.config) !== JSON.stringify(agent.config) 
          ? formData.config : undefined,
      };

      
      const cleanRequest = Object.fromEntries(
        Object.entries(request).filter(([_, value]) => value !== undefined)
      ) as UpdateAgentRequest;

      if (Object.keys(cleanRequest).length === 0) {
        toast({
          title: "No Changes",
          description: "No changes were made to the agent",
        });
        onOpenChange(false);
        return;
      }

      const result = await apiClient.updateAgent(agent.id, cleanRequest);
      
      if (result.success) {
        toast({
          title: "Success",
          description: "Agent updated successfully",
        });
        onAgentUpdated();
        onOpenChange(false);
      } else {
        toast({
          title: "Error",
          description: result.message || "Failed to update agent",
          variant: "destructive",
        });
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to update agent",
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handlePermissionChange = (permission: string, checked: boolean) => {
    setFormData(prev => ({
      ...prev,
      permissions: checked
        ? [...prev.permissions, permission]
        : prev.permissions.filter(p => p !== permission)
    }));
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Edit Agent: {agent.name}</DialogTitle>
          <DialogDescription>
            Update agent configuration and permissions.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-6">
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="name">Agent Name</Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                  required
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="status">Status</Label>
                <Select
                  value={formData.status}
                  onValueChange={(value) =>
                    setFormData(prev => ({ ...prev, status: value as AgentRecord['status'] }))
                  }
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {STATUS_OPTIONS.map((status) => (
                      <SelectItem key={status.value} value={status.value}>
                        {status.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>

            {agent.agent_type === "custom" && (
              <div className="space-y-2">
                <Label htmlFor="endpoint">API Endpoint</Label>
                <Input
                  id="endpoint"
                  value={formData.endpoint}
                  onChange={(e) => setFormData(prev => ({ ...prev, endpoint: e.target.value }))}
                  placeholder="https://your-agent-api.com/v1/chat"
                />
              </div>
            )}

            <div className="space-y-2">
              <Label htmlFor="api_key">API Key</Label>
              <Input
                id="api_key"
                type="password"
                value={formData.api_key}
                onChange={(e) => setFormData(prev => ({ ...prev, api_key: e.target.value }))}
                placeholder="sk-..."
                required
              />
              <p className="text-xs text-muted-foreground">
                Leave unchanged to keep current API key.
              </p>
            </div>

            <div className="space-y-2">
              <Label>Model Configuration</Label>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="model">Model</Label>
                  <Select
                    value={formData.config.model}
                    onValueChange={(value) =>
                      setFormData(prev => ({
                        ...prev,
                        config: { ...prev.config, model: value }
                      }))
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select model" />
                    </SelectTrigger>
                    <SelectContent>
                      {agent.agent_type === "openai" &&
                        OPENAI_MODELS.map((model) => (
                          <SelectItem key={model} value={model}>
                            {model}
                          </SelectItem>
                        ))}
                      {agent.agent_type === "anthropic" &&
                        ANTHROPIC_MODELS.map((model) => (
                          <SelectItem key={model} value={model}>
                            {model}
                          </SelectItem>
                        ))}
                      {agent.agent_type === "custom" && (
                        <SelectItem value="custom">Custom Model</SelectItem>
                      )}
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="temperature">Temperature</Label>
                  <Input
                    id="temperature"
                    type="number"
                    min="0"
                    max="2"
                    step="0.1"
                    value={formData.config.temperature}
                    onChange={(e) =>
                      setFormData(prev => ({
                        ...prev,
                        config: { ...prev.config, temperature: parseFloat(e.target.value) }
                      }))
                    }
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="max_tokens">Max Tokens</Label>
                  <Input
                    id="max_tokens"
                    type="number"
                    min="1"
                    max="4000"
                    value={formData.config.max_tokens}
                    onChange={(e) =>
                      setFormData(prev => ({
                        ...prev,
                        config: { ...prev.config, max_tokens: parseInt(e.target.value) }
                      }))
                    }
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="timeout">Timeout (seconds)</Label>
                  <Input
                    id="timeout"
                    type="number"
                    min="5"
                    max="300"
                    value={formData.config.timeout}
                    onChange={(e) =>
                      setFormData(prev => ({
                        ...prev,
                        config: { ...prev.config, timeout: parseInt(e.target.value) }
                      }))
                    }
                  />
                </div>
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="custom_instructions">Custom Instructions</Label>
              <Textarea
                id="custom_instructions"
                value={formData.config.custom_instructions}
                onChange={(e) =>
                  setFormData(prev => ({
                    ...prev,
                    config: { ...prev.config, custom_instructions: e.target.value }
                  }))
                }
                placeholder="Additional instructions for the agent..."
                rows={3}
              />
            </div>

            <div className="space-y-2">
              <Label>Permissions</Label>
              <p className="text-sm text-muted-foreground">
                Select what data and actions this agent can access.
              </p>
              <div className="grid grid-cols-2 gap-2">
                {PERMISSIONS.map((permission) => (
                  <div key={permission.value} className="flex items-center space-x-2">
                    <Checkbox
                      id={permission.value}
                      checked={formData.permissions.includes(permission.value)}
                      onCheckedChange={(checked) =>
                        handlePermissionChange(permission.value, checked as boolean)
                      }
                    />
                    <Label htmlFor={permission.value} className="text-sm">
                      {permission.label}
                    </Label>
                  </div>
                ))}
              </div>
            </div>

            {}
            <div className="space-y-2">
              <Label>Usage Statistics</Label>
              <div className="grid grid-cols-3 gap-4">
                <div className="text-center p-3 bg-muted rounded-lg">
                  <div className="text-2xl font-bold">{agent.usage_stats.total_requests}</div>
                  <div className="text-sm text-muted-foreground">Total Requests</div>
                </div>
                <div className="text-center p-3 bg-muted rounded-lg">
                  <div className="text-2xl font-bold">{agent.usage_stats.total_tokens.toLocaleString()}</div>
                  <div className="text-sm text-muted-foreground">Total Tokens</div>
                </div>
                <div className="text-center p-3 bg-muted rounded-lg">
                  <div className="text-2xl font-bold">
                    {agent.usage_stats.avg_response_time ? 
                      `${agent.usage_stats.avg_response_time.toFixed(0)}ms` : 
                      'N/A'
                    }
                  </div>
                  <div className="text-sm text-muted-foreground">Avg Response</div>
                </div>
              </div>
            </div>
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={isLoading}>
              {isLoading ? "Updating..." : "Update Agent"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}