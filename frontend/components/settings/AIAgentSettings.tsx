import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { 
  Bot, 
  Plus, 
  Settings, 
  Trash2,
  Key,
  Activity,
  Shield,
  Zap,
  Brain,
  Code
} from "lucide-react";

export function AIAgentSettings() {
  const connectedAgents = [
    { 
      id: 1, 
      name: "GitHub Copilot", 
      status: "connected", 
      requests: "1,247",
      permissions: ["read", "write"],
      lastUsed: "2 minutes ago",
      apiKey: "gh_***********1234"
    },
    { 
      id: 2, 
      name: "Amazon Q", 
      status: "connected", 
      requests: "892",
      permissions: ["read"],
      lastUsed: "1 hour ago",
      apiKey: "aq_***********5678"
    },
    { 
      id: 3, 
      name: "Cline", 
      status: "connected", 
      requests: "634",
      permissions: ["read", "write"],
      lastUsed: "5 minutes ago",
      apiKey: "cl_***********9012"
    },
    { 
      id: 4, 
      name: "Custom Agent", 
      status: "pending", 
      requests: "0",
      permissions: [],
      lastUsed: "Never",
      apiKey: "Not configured"
    },
  ];

  return (
    <div className="space-y-6">
      {/* Add New Agent */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Plus className="w-5 h-5" />
            Connect New AI Agent
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="agentType">Agent Type</Label>
              <Select>
                <SelectTrigger>
                  <SelectValue placeholder="Select an AI agent" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="github-copilot">GitHub Copilot</SelectItem>
                  <SelectItem value="amazon-q">Amazon Q</SelectItem>
                  <SelectItem value="anthropic-claude">Anthropic Claude</SelectItem>
                  <SelectItem value="openai-gpt">OpenAI GPT</SelectItem>
                  <SelectItem value="custom">Custom Agent</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="agentName">Agent Name</Label>
              <Input 
                id="agentName" 
                placeholder="My Custom Agent" 
              />
            </div>
          </div>
          
          <div className="space-y-2">
            <Label htmlFor="apiKey">API Key</Label>
            <Input 
              id="apiKey" 
              type="password"
              placeholder="Enter your API key"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="endpoint">Custom Endpoint (Optional)</Label>
            <Input 
              id="endpoint" 
              placeholder="https://api.your-agent.com/v1"
            />
          </div>

          <div className="space-y-2">
            <Label>Permissions</Label>
            <div className="flex flex-wrap gap-4">
              <div className="flex items-center space-x-2">
                <Switch id="read-perm" />
                <Label htmlFor="read-perm">Read access to repositories</Label>
              </div>
              <div className="flex items-center space-x-2">
                <Switch id="write-perm" />
                <Label htmlFor="write-perm">Write access to repositories</Label>
              </div>
              <div className="flex items-center space-x-2">
                <Switch id="exec-perm" />
                <Label htmlFor="exec-perm">Execute commands</Label>
              </div>
            </div>
          </div>

          <Button className="w-full md:w-auto">
            <Plus className="w-4 h-4 mr-2" />
            Connect Agent
          </Button>
        </CardContent>
      </Card>

      {/* Connected Agents */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Bot className="w-5 h-5" />
            Connected AI Agents ({connectedAgents.length})
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {connectedAgents.map((agent) => (
            <div key={agent.id} className="p-4 rounded-lg border border-border bg-muted/20">
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-3">
                  <Bot className="w-5 h-5 text-accent" />
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-foreground">{agent.name}</span>
                      <Badge 
                        variant={agent.status === "connected" ? "default" : "secondary"}
                        className="text-xs"
                      >
                        {agent.status}
                      </Badge>
                    </div>
                    <div className="text-sm text-muted-foreground">
                      {agent.requests} requests today • Last used: {agent.lastUsed}
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <Button variant="ghost" size="sm">
                    <Activity className="w-4 h-4" />
                  </Button>
                  <Button variant="ghost" size="sm">
                    <Settings className="w-4 h-4" />
                  </Button>
                  <Button variant="ghost" size="sm" className="text-destructive">
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>

              {/* Agent Details */}
              <div className="grid gap-4 md:grid-cols-2 pt-3 border-t border-border">
                <div className="space-y-2">
                  <Label className="text-sm flex items-center gap-2">
                    <Key className="w-4 h-4" />
                    API Key
                  </Label>
                  <div className="flex items-center gap-2">
                    <Input 
                      type="password" 
                      value={agent.apiKey} 
                      readOnly 
                      className="h-8 text-xs"
                    />
                    <Button variant="outline" size="sm">
                      Update
                    </Button>
                  </div>
                </div>
                <div className="space-y-2">
                  <Label className="text-sm flex items-center gap-2">
                    <Shield className="w-4 h-4" />
                    Permissions
                  </Label>
                  <div className="flex flex-wrap gap-1">
                    {agent.permissions.map((perm) => (
                      <Badge key={perm} variant="outline" className="text-xs">
                        {perm}
                      </Badge>
                    ))}
                    {agent.permissions.length === 0 && (
                      <Badge variant="outline" className="text-xs text-muted-foreground">
                        No permissions
                      </Badge>
                    )}
                  </div>
                </div>
              </div>

              {/* Agent-specific settings */}
              <div className="mt-4 pt-3 border-t border-border">
                <div className="grid gap-4 md:grid-cols-3">
                  <div className="flex items-center justify-between">
                    <Label className="text-sm">Auto-suggestions</Label>
                    <Switch defaultChecked={agent.status === "connected"} />
                  </div>
                  <div className="flex items-center justify-between">
                    <Label className="text-sm">Context sharing</Label>
                    <Switch defaultChecked={true} />
                  </div>
                  <div className="flex items-center justify-between">
                    <Label className="text-sm">Real-time assistance</Label>
                    <Switch defaultChecked={agent.status === "connected"} />
                  </div>
                </div>
              </div>
            </div>
          ))}
        </CardContent>
      </Card>

      {/* Global AI Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Brain className="w-5 h-5" />
            Global AI Settings
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Context Window Size</Label>
              <p className="text-sm text-muted-foreground">
                Maximum number of lines to include in AI context
              </p>
            </div>
            <Select defaultValue="1000">
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="500">500 lines</SelectItem>
                <SelectItem value="1000">1000 lines</SelectItem>
                <SelectItem value="2000">2000 lines</SelectItem>
                <SelectItem value="5000">5000 lines</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Auto-completion</Label>
              <p className="text-sm text-muted-foreground">
                Enable AI-powered code completion across all agents
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Smart suggestions</Label>
              <p className="text-sm text-muted-foreground">
                Show contextual suggestions based on repository content
              </p>
            </div>
            <Switch defaultChecked />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Rate limiting</Label>
              <p className="text-sm text-muted-foreground">
                Limit requests per minute to prevent API quota exhaustion
              </p>
            </div>
            <Select defaultValue="60">
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="30">30/min</SelectItem>
                <SelectItem value="60">60/min</SelectItem>
                <SelectItem value="120">120/min</SelectItem>
                <SelectItem value="unlimited">Unlimited</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardContent>
      </Card>

      {/* Custom Prompts */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Code className="w-5 h-5" />
            Custom System Prompts
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="systemPrompt">Default System Prompt</Label>
            <Textarea 
              id="systemPrompt" 
              placeholder="Enter your custom system prompt for AI agents..."
              defaultValue="You are a helpful AI assistant specialized in software development. Provide clear, concise, and accurate responses."
              className="resize-none min-h-[100px]"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="codePrompt">Code Review Prompt</Label>
            <Textarea 
              id="codePrompt" 
              placeholder="Enter prompt for code review tasks..."
              defaultValue="Review this code for potential issues, suggest improvements, and ensure it follows best practices."
              className="resize-none min-h-[80px]"
            />
          </div>
        </CardContent>
      </Card>

      {/* Save Changes */}
      <div className="flex justify-end gap-4">
        <Button variant="outline">Cancel</Button>
        <Button>Save Changes</Button>
      </div>
    </div>
  );
}
