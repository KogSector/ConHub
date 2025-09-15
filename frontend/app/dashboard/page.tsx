"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ProfileAvatar } from "@/components/ui/profile-avatar";
import { Footer } from "@/components/ui/footer";
import { AddUrlModal } from "@/components/ui/add-url-modal";
import Link from "next/link";
import { 
  Bot, 
  Plus, 
  Settings,
  Activity,
  Code,
  Network,
  Shield,
  GitBranch,
  FileText,
  Link as LinkIcon,
  Globe,
  BookOpen
} from "lucide-react";

export default function Dashboard() {
  const [isAddUrlModalOpen, setIsAddUrlModalOpen] = useState(false);

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <div className="border-b border-border bg-card/50 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-20">
            <div className="flex items-center space-x-3">
              <h1 className="text-3xl md:text-4xl font-bold font-orbitron bg-gradient-to-r from-primary via-primary-glow to-accent bg-clip-text text-transparent">ConHub</h1>
            </div>
            <div className="flex items-center gap-4">
              <ProfileAvatar />
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Main Dashboard Layout - 2 Columns */}
        <div className="grid lg:grid-cols-2 gap-12 mb-8">
          {/* Left Column - Quick Actions */}
          <div>
            <h2 className="text-2xl font-semibold text-foreground mb-6">Quick Actions</h2>
            <div className="grid grid-cols-2 gap-4 mt-7">
                <Link href="/repositories">
                  <Button size="lg" className="px-6 py-6 h-auto flex flex-col items-center space-y-2 hover:bg-green-600 transition-colors w-full">
                    <GitBranch className="w-6 h-6" />
                    <span>Connect Repository</span>
                  </Button>
                </Link>
                <Link href="/dashboard/documents">
                  <Button size="lg" className="px-6 py-6 h-auto flex flex-col items-center space-y-2 hover:bg-green-600 transition-colors w-full">
                    <FileText className="w-6 h-6" />
                    <span>Add Documents</span>
                  </Button>
                </Link>
                <Link href="/dashboard/urls">
                  <Button size="lg" className="px-6 py-6 h-auto flex flex-col items-center space-y-2 hover:bg-green-600 transition-colors w-full">
                    <LinkIcon className="w-6 h-6" />
                    <span>Add URLs</span>
                  </Button>
                </Link>
                <Link href="/ai-agents">
                  <Button size="lg" className="px-6 py-6 h-auto flex flex-col items-center space-y-2 hover:bg-green-600 transition-colors w-full">
                    <Bot className="w-6 h-6" />
                    <span>Manage AI Agents</span>
                  </Button>
                </Link>
                <Button size="lg" className="px-6 py-6 h-auto flex flex-col items-center space-y-2 hover:bg-green-600 transition-colors">
                  <Network className="w-6 h-6" />
                  <span>Configure RAG</span>
                </Button>
                <Link href="/docs">
                  <Button size="lg" className="px-6 py-6 h-auto flex flex-col items-center space-y-2 hover:bg-green-600 transition-colors w-full">
                    <BookOpen className="w-6 h-6" />
                    <span>View Documentation</span>
                  </Button>
                </Link>
            </div>
          </div>

          {/* Right Column - Stats Overview */}
          <div>
            <h2 className="text-2xl font-semibold text-foreground mb-6">Overview</h2>
            <div className="grid grid-cols-2 gap-4">
                <Card className="bg-card border-border">
                  <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle className="text-sm font-medium text-muted-foreground">
                      Repositories
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
                      Documents
                    </CardTitle>
                    <FileText className="w-4 h-4 text-primary" />
                  </CardHeader>
                  <CardContent>
                    <div className="text-2xl font-bold text-foreground">47</div>
                    <p className="text-xs text-muted-foreground">
                      +8 from last week
                    </p>
                  </CardContent>
                </Card>

                <Card className="bg-card border-border">
                  <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle className="text-sm font-medium text-muted-foreground">
                      URLs
                    </CardTitle>
                    <Globe className="w-4 h-4 text-primary" />
                  </CardHeader>
                  <CardContent>
                    <div className="text-2xl font-bold text-foreground">23</div>
                    <p className="text-xs text-muted-foreground">
                      +5 from last week
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
          </div>
        </div>

        <div className="grid lg:grid-cols-3 gap-8">
          {/* Connected Sources */}
          <Card className="bg-card border-border">
            <CardHeader>
              <div className="flex justify-between items-center">
                <CardTitle className="text-lg font-semibold text-foreground">
                  Connected Sources
                </CardTitle>
                <Button variant="outline" size="sm" onClick={() => setIsAddUrlModalOpen(true)}>
                  <Plus className="w-4 h-4 mr-2" />
                  Add URL
                </Button>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Source Items */}
              {[
                { name: "frontend-app", type: "repository", status: "active", private: false, icon: GitBranch },
                { name: "API Documentation", type: "document", status: "active", private: false, icon: FileText },
                { name: "Confluence Wiki", type: "url", status: "syncing", private: true, icon: Globe },
                { name: "user-service", type: "repository", status: "active", private: true, icon: GitBranch },
              ].map((source, index) => {
                const IconComponent = source.icon;
                return (
                  <div key={index} className="flex flex-col p-3 rounded-lg bg-muted/20 border border-border gap-2">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center space-x-3">
                        <IconComponent className="w-5 h-5 text-primary" />
                        <span className="font-medium text-foreground">{source.name}</span>
                      </div>
                      <div className="flex items-center space-x-2">
                        {source.private && <Badge variant="secondary" className="text-xs">Private</Badge>}
                        <Badge variant="outline" className="text-xs capitalize">{source.type}</Badge>
                      </div>
                    </div>
                    <div className="flex items-center space-x-2">
                      <div className={`w-2 h-2 rounded-full ${
                        source.status === 'active' || source.status === 'syncing' ? 'bg-green-500 shadow-lg shadow-green-500/50' : 'bg-gray-400'
                      }`}></div>
                      <div className="text-sm text-muted-foreground">
                        Status: {source.status}
                      </div>
                    </div>
                    <Button variant="ghost" size="sm" className="ml-auto">
                      <Settings className="w-4 h-4" />
                    </Button>
                  </div>
                );
              })}
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
                <div key={index} className="flex flex-col p-3 rounded-lg bg-muted/20 border border-border gap-2">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                      <Bot className="w-5 h-5 text-accent" />
                      <span className="font-medium text-foreground">{agent.name}</span>
                    </div>
                    <div className={`w-2 h-2 rounded-full ${
                      agent.status === 'connected' ? 'bg-blue-500 shadow-lg shadow-blue-500/50' : 'bg-gray-400'
                    }`}></div>
                  </div>
                  <div className="text-sm text-muted-foreground">
                    {agent.requests} requests today
                  </div>
                  <Button variant="ghost" size="sm" className="ml-auto">
                    <Settings className="w-4 h-4" />
                  </Button>
                </div>
              ))}
            </CardContent>
          </Card>

          {/* Recent Activity */}
          <Card className="bg-card border-border">
            <CardHeader>
              <CardTitle className="text-lg font-semibold text-foreground">
                Recent Activity
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {[
                { action: "Added API docs", source: "Swagger UI", time: "2 hours ago", type: "document" },
                { action: "Synced repository", source: "payment-service", time: "4 hours ago", type: "repository" },
                { action: "Connected URL", source: "Team Wiki", time: "1 day ago", type: "url" },
                { action: "Agent query", source: "GitHub Copilot", time: "2 days ago", type: "agent" },
              ].map((activity, index) => {
                const getIcon = (type: string) => {
                  switch (type) {
                    case 'document': return FileText;
                    case 'repository': return GitBranch;
                    case 'url': return Globe;
                    case 'agent': return Bot;
                    default: return Activity;
                  }
                };
                const IconComponent = getIcon(activity.type);
                return (
                  <div key={index} className="flex items-center space-x-3 p-3 rounded-lg bg-muted/20 border border-border">
                    <IconComponent className="w-4 h-4 text-primary" />
                    <div className="flex-1">
                      <div className="text-sm font-medium text-foreground">{activity.action}</div>
                      <div className="text-xs text-muted-foreground">{activity.source} • {activity.time}</div>
                    </div>
                  </div>
                );
              })}
            </CardContent>
          </Card>
        </div>
      </div>
      
      <AddUrlModal
        open={isAddUrlModalOpen}
        onOpenChange={setIsAddUrlModalOpen}
      />
      
      <Footer />
    </div>
  );
}
