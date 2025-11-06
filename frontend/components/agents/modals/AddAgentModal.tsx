"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { Checkbox } from "@/components/ui/checkbox";
import { useToast } from "@/hooks/use-toast";
import { apiClient, CreateAgentRequest, AgentConfig } from "@/lib/api";
import { Card, CardContent } from "@/components/ui/card";
import { Brain, Zap, FileText } from "lucide-react";

interface AddAgentModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onAgentAdded: () => void;
}

const AGENT_TYPES = [
  { value: "openai", label: "OpenAI GPT", icon: Brain, description: "OpenAI's GPT models (GPT-4, GPT-3.5)" },
  { value: "anthropic", label: "Anthropic Claude", icon: Zap, description: "Anthropic's Claude models" },
  { value: "custom", label: "Custom Agent", icon: FileText, description: "Your own AI agent endpoint" },
];

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

export function AddAgentModal({ open, onOpenChange, onAgentAdded }: AddAgentModalProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [step, setStep] = useState(1);
  const [formData, setFormData] = useState({
    name: "",
    agent_type: "",
    endpoint: "",
    api_key: "",
    permissions: [] as string[],
    config: {
      model: "",
      temperature: 0.7,
      max_tokens: 1000,
      timeout: 30,
      custom_instructions: "",
    } as AgentConfig,
  });
  const { toast } = useToast();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);

    try {
      const request: CreateAgentRequest = {
        name: formData.name,
        agent_type: formData.agent_type,
        endpoint: formData.endpoint || undefined,
        api_key: formData.api_key,
        permissions: formData.permissions,
        config: formData.config,
      };

      const result = await apiClient.createAgent(request);
      
      if (result.success) {
        toast({
          title: "Success",
          description: "Agent created successfully",
        });
        onAgentAdded();
        onOpenChange(false);
        resetForm();
      } else {
        toast({
          title: "Error",
          description: result.message || "Failed to create agent",
          variant: "destructive",
        });
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to create agent",
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  };

  const resetForm = () => {
    setStep(1);
    setFormData({
      name: "",
      agent_type: "",
      endpoint: "",
      api_key: "",
      permissions: [],
      config: {
        model: "",
        temperature: 0.7,
        max_tokens: 1000,
        timeout: 30,
        custom_instructions: "",
      },
    });
  };

  const handlePermissionChange = (permission: string, checked: boolean) => {
    setFormData(prev => ({
      ...prev,
      permissions: checked
        ? [...prev.permissions, permission]
        : prev.permissions.filter(p => p !== permission)
    }));
  };

  const canProceedToStep2 = formData.name && formData.agent_type && formData.api_key;
  const canSubmit = canProceedToStep2 && formData.permissions.length > 0;

  return (
    <Dialog open={open} onOpenChange={(open) => { onOpenChange(open); if (!open) resetForm(); }}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Add AI Agent</DialogTitle>
          <DialogDescription>
            Connect a new AI agent to ConHub for automated tasks and assistance.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-6">
          {step === 1 && (
            <div className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="name">Agent Name</Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                  placeholder="e.g., My GitHub Copilot Assistant"
                  required
                />
              </div>

              <div className="space-y-2">
                <Label>Agent Type</Label>
                <div className="grid grid-cols-1 gap-3">
                  {AGENT_TYPES.map((type) => {
                    const Icon = type.icon;
                    return (
                      <Card
                        key={type.value}
                        className={`cursor-pointer transition-colors ${
                          formData.agent_type === type.value
                            ? "border-primary bg-primary/5"
                            : "hover:bg-muted/50"
                        }`}
                        onClick={() => setFormData(prev => ({ ...prev, agent_type: type.value }))}
                      >
                        <CardContent className="p-4">
                          <div className="flex items-center gap-3">
                            <Icon className="w-5 h-5" />
                            <div>
                              <div className="font-medium">{type.label}</div>
                              <div className="text-sm text-muted-foreground">{type.description}</div>
                            </div>
                          </div>
                        </CardContent>
                      </Card>
                    );
                  })}
                </div>
              </div>

              {formData.agent_type === "custom" && (
                <div className="space-y-2">
                  <Label htmlFor="endpoint">API Endpoint</Label>
                  <Input
                    id="endpoint"
                    value={formData.endpoint}
                    onChange={(e) => setFormData(prev => ({ ...prev, endpoint: e.target.value }))}
                    placeholder="https://your-agent-api.com/v1/chat"
                    required
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
                  Your API key will be stored securely and encrypted.
                </p>
              </div>
            </div>
          )}

          {step === 2 && (
            <div className="space-y-4">
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
                        {formData.agent_type === "openai" &&
                          OPENAI_MODELS.map((model) => (
                            <SelectItem key={model} value={model}>
                              {model}
                            </SelectItem>
                          ))}
                        {formData.agent_type === "anthropic" &&
                          ANTHROPIC_MODELS.map((model) => (
                            <SelectItem key={model} value={model}>
                              {model}
                            </SelectItem>
                          ))}
                        {formData.agent_type === "custom" && (
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
                <Label htmlFor="custom_instructions">Custom Instructions (Optional)</Label>
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
            </div>
          )}

          <DialogFooter>
            {step === 1 ? (
              <Button
                type="button"
                onClick={() => setStep(2)}
                disabled={!canProceedToStep2}
              >
                Next: Configure
              </Button>
            ) : (
              <div className="flex gap-2">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => setStep(1)}
                >
                  Back
                </Button>
                <Button type="submit" disabled={!canSubmit || isLoading}>
                  {isLoading ? "Creating..." : "Create Agent"}
                </Button>
              </div>
            )}
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
