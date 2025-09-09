import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ProfileAvatar } from "@/components/ui/profile-avatar";
import { Footer } from "@/components/ui/footer";
import { 
  Bot, 
  Plus, 
  Settings, 
  Activity,
  Code,
  Network,
  Shield,
  GitBranch
} from "lucide-react";

export default function Dashboard() {
  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <div className="border-b border-border bg-card/50 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-20">
            <div className="flex items-center space-x-3">
              <h1 className="text-3xl md:text-4xl font-bold font-orbitron bg-gradient-to-r from-primary via-primary-glow to-accent bg-clip-text text-transparent">ConHub</h1>
            </div>
            <div className="flex items-center">
              <ProfileAvatar />
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Quick Actions */}
        <div className="mb-8 text-center">
          <h2 className="text-2xl font-semibold text-foreground mb-6">Quick Actions</h2>
          <div className="max-w-2xl mx-auto">
            <div className="grid grid-cols-2 gap-4">
              <Button size="lg" className="px-6 py-4 h-auto flex flex-col items-center space-y-2 hover:bg-green-600 transition-colors">
                <GitBranch className="w-6 h-6" />
                <span>Connect Repository</span>
              </Button>
              <Button size="lg" className="px-6 py-4 h-auto flex flex-col items-center space-y-2 hover:bg-green-600 transition-colors">
                <Bot className="w-6 h-6" />
                <span>Add AI Agents</span>
              </Button>
              <Button size="lg" className="px-6 py-4 h-auto flex flex-col items-center space-y-2 hover:bg-green-600 transition-colors">
                <Code className="w-6 h-6" />
                <span>Upload Documents</span>
              </Button>
              <Button size="lg" className="px-6 py-4 h-auto flex flex-col items-center space-y-2 hover:bg-green-600 transition-colors">
                <Network className="w-6 h-6" />
                <span>Configure RAG</span>
              </Button>
            </div>
          </div>
        </div>

        {/* Stats Overview */}
        <div className="grid md:grid-cols-4 gap-6 mb-8">
          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Connected Repositories
              </CardTitle>
              <GitBranch className="w-4 h-4 text-primary" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">12</div>
              <p className="text-xs text-muted-foreground">
                +2 from last week
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                AI Agents
              </CardTitle>
              <Bot className="w-4 h-4 text-accent" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">5</div>
              <p className="text-xs text-muted-foreground">
                All active
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Context Requests
              </CardTitle>
              <Activity className="w-4 h-4 text-primary-glow" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">1,247</div>
              <p className="text-xs text-muted-foreground">
                +12% from yesterday
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Security Score
              </CardTitle>
              <Shield className="w-4 h-4 text-accent" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">98%</div>
              <p className="text-xs text-muted-foreground">
                Excellent
              </p>
            </CardContent>
          </Card>
        </div>

        <div className="grid lg:grid-cols-2 gap-8">
          {/* Connected Repositories */}
          <Card className="bg-card border-border">
            <CardHeader>
              <div className="flex justify-between items-center">
                <CardTitle className="text-lg font-semibold text-foreground">
                  Connected Repositories
                </CardTitle>
                <Button variant="outline" size="sm">
                  <Plus className="w-4 h-4 mr-2" />
                  Add Repository
                </Button>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Repository Items */}
              {[
                { name: "frontend-app", status: "active", private: false },
                { name: "api-gateway", status: "active", private: true },
                { name: "user-service", status: "syncing", private: true },
                { name: "payment-service", status: "active", private: true },
              ].map((repo, index) => (
                <div key={index} className="flex items-center justify-between p-3 rounded-lg bg-muted/20 border border-border">
                  <div className="flex items-center space-x-3">
                    <Code className="w-5 h-5 text-primary" />
                    <div>
                      <div className="flex items-center space-x-2">
                        <span className="font-medium text-foreground">{repo.name}</span>
                        {repo.private && <Badge variant="secondary" className="text-xs">Private</Badge>}
                      </div>
                      <div className="text-sm text-muted-foreground">
                        Status: {repo.status}
                      </div>
                    </div>
                  </div>
                  <Button variant="ghost" size="sm">
                    <Settings className="w-4 h-4" />
                  </Button>
                </div>
              ))}
            </CardContent>
          </Card>

          {/* AI Agents */}
          <Card className="bg-card border-border">
            <CardHeader>
              <div className="flex justify-between items-center">
                <CardTitle className="text-lg font-semibold text-foreground">
                  AI Agents
                </CardTitle>
                <Button variant="outline" size="sm">
                  <Plus className="w-4 h-4 mr-2" />
                  Connect Agent
                </Button>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* AI Agent Items */}
              {[
                { name: "GitHub Copilot", status: "connected", requests: "1,247" },
                { name: "Amazon Q", status: "connected", requests: "892" },
                { name: "Cline", status: "connected", requests: "634" },
                { name: "Custom Agent", status: "pending", requests: "0" },
              ].map((agent, index) => (
                <div key={index} className="flex items-center justify-between p-3 rounded-lg bg-muted/20 border border-border">
                  <div className="flex items-center space-x-3">
                    <Bot className="w-5 h-5 text-accent" />
                    <div>
                      <div className="flex items-center space-x-2">
                        <span className="font-medium text-foreground">{agent.name}</span>
                        <Badge 
                          variant={agent.status === "connected" ? "default" : "secondary"}
                          className="text-xs"
                        >
                          {agent.status}
                        </Badge>
                      </div>
                      <div className="text-sm text-muted-foreground">
                        {agent.requests} requests today
                      </div>
                    </div>
                  </div>
                  <Button variant="ghost" size="sm">
                    <Settings className="w-4 h-4" />
                  </Button>
                </div>
              ))}
            </CardContent>
          </Card>
        </div>
      </div>
      <Footer />
    </div>
  );
}
