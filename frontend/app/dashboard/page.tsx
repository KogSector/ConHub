"use client";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ProfileAvatar } from "@/components/ui/profile-avatar";
import { Footer } from "@/components/ui/footer";
import { AuthGuard } from "@/components/auth/auth-guard";
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
  BookOpen,
  Users
} from "lucide-react";

export default function Dashboard() {
  return (
    <AuthGuard>
      <div className="min-h-screen bg-background">
      {}
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
        {}
        <div className="grid lg:grid-cols-2 gap-12 mb-8">
          {}
          <div>
            <h2 className="text-2xl font-semibold text-foreground mb-6">Quick Actions</h2>
            <div className="grid grid-cols-2 gap-6 mt-7">
                <Link href="/repositories" className="group">
                  <div className="relative transform transition-all duration-300 hover:scale-105 hover:-translate-y-2">
                    <div className="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-600 rounded-2xl blur-xl opacity-25 group-hover:opacity-40 transition-opacity duration-300"></div>
                    <div className="relative bg-gradient-to-br from-blue-500 to-blue-600 hover:from-blue-600 hover:to-purple-600 text-white rounded-2xl p-6 shadow-2xl border border-blue-400/20 backdrop-blur-sm transition-all duration-300">
                      <div className="flex flex-col items-center space-y-3">
                        <div className="p-3 bg-white/20 rounded-xl backdrop-blur-sm">
                          <GitBranch className="w-7 h-7" />
                        </div>
                        <span className="font-semibold text-center leading-tight">Connect Repository</span>
                      </div>
                      <div className="absolute top-2 right-2 w-2 h-2 bg-white/30 rounded-full"></div>
                      <div className="absolute bottom-2 left-2 w-1 h-1 bg-white/20 rounded-full"></div>
                    </div>
                  </div>
                </Link>
                
                <Link href="/dashboard/documents" className="group">
                  <div className="relative transform transition-all duration-300 hover:scale-105 hover:-translate-y-2">
                    <div className="absolute inset-0 bg-gradient-to-r from-emerald-500 to-teal-600 rounded-2xl blur-xl opacity-25 group-hover:opacity-40 transition-opacity duration-300"></div>
                    <div className="relative bg-gradient-to-br from-emerald-500 to-emerald-600 hover:from-emerald-600 hover:to-teal-600 text-white rounded-2xl p-6 shadow-2xl border border-emerald-400/20 backdrop-blur-sm transition-all duration-300">
                      <div className="flex flex-col items-center space-y-3">
                        <div className="p-3 bg-white/20 rounded-xl backdrop-blur-sm">
                          <FileText className="w-7 h-7" />
                        </div>
                        <span className="font-semibold text-center leading-tight">Add Documents</span>
                      </div>
                      <div className="absolute top-2 right-2 w-2 h-2 bg-white/30 rounded-full"></div>
                      <div className="absolute bottom-2 left-2 w-1 h-1 bg-white/20 rounded-full"></div>
                    </div>
                  </div>
                </Link>
                
                <Link href="/dashboard/urls" className="group">
                  <div className="relative transform transition-all duration-300 hover:scale-105 hover:-translate-y-2">
                    <div className="absolute inset-0 bg-gradient-to-r from-orange-500 to-red-600 rounded-2xl blur-xl opacity-25 group-hover:opacity-40 transition-opacity duration-300"></div>
                    <div className="relative bg-gradient-to-br from-orange-500 to-orange-600 hover:from-orange-600 hover:to-red-600 text-white rounded-2xl p-6 shadow-2xl border border-orange-400/20 backdrop-blur-sm transition-all duration-300">
                      <div className="flex flex-col items-center space-y-3">
                        <div className="p-3 bg-white/20 rounded-xl backdrop-blur-sm">
                          <LinkIcon className="w-7 h-7" />
                        </div>
                        <span className="font-semibold text-center leading-tight">Add URLs</span>
                      </div>
                      <div className="absolute top-2 right-2 w-2 h-2 bg-white/30 rounded-full"></div>
                      <div className="absolute bottom-2 left-2 w-1 h-1 bg-white/20 rounded-full"></div>
                    </div>
                  </div>
                </Link>
                
                <Link href="/dashboard/social" className="group">
                  <div className="relative transform transition-all duration-300 hover:scale-105 hover:-translate-y-2">
                    <div className="absolute inset-0 bg-gradient-to-r from-indigo-500 to-purple-600 rounded-2xl blur-xl opacity-25 group-hover:opacity-40 transition-opacity duration-300"></div>
                    <div className="relative bg-gradient-to-br from-indigo-500 to-indigo-600 hover:from-indigo-600 hover:to-purple-600 text-white rounded-2xl p-6 shadow-2xl border border-indigo-400/20 backdrop-blur-sm transition-all duration-300">
                      <div className="flex flex-col items-center space-y-3">
                        <div className="p-3 bg-white/20 rounded-xl backdrop-blur-sm">
                          <Users className="w-7 h-7" />
                        </div>
                        <span className="font-semibold text-center leading-tight">Social Connections</span>
                      </div>
                      <div className="absolute top-2 right-2 w-2 h-2 bg-white/30 rounded-full"></div>
                      <div className="absolute bottom-2 left-2 w-1 h-1 bg-white/20 rounded-full"></div>
                    </div>
                  </div>
                </Link>
                
                <Link href="/agents" className="group">
                  <div className="relative transform transition-all duration-300 hover:scale-105 hover:-translate-y-2">
                    <div className="absolute inset-0 bg-gradient-to-r from-purple-500 to-pink-600 rounded-2xl blur-xl opacity-25 group-hover:opacity-40 transition-opacity duration-300"></div>
                    <div className="relative bg-gradient-to-br from-purple-500 to-purple-600 hover:from-purple-600 hover:to-pink-600 text-white rounded-2xl p-6 shadow-2xl border border-purple-400/20 backdrop-blur-sm transition-all duration-300">
                      <div className="flex flex-col items-center space-y-3">
                        <div className="p-3 bg-white/20 rounded-xl backdrop-blur-sm">
                          <Bot className="w-7 h-7" />
                        </div>
                        <span className="font-semibold text-center leading-tight">Manage AI Agents</span>
                      </div>
                      <div className="absolute top-2 right-2 w-2 h-2 bg-white/30 rounded-full"></div>
                      <div className="absolute bottom-2 left-2 w-1 h-1 bg-white/20 rounded-full"></div>
                    </div>
                  </div>
                </Link>
                
                <div className="group cursor-pointer">
                  <div className="relative transform transition-all duration-300 hover:scale-105 hover:-translate-y-2">
                    <div className="absolute inset-0 bg-gradient-to-r from-cyan-500 to-blue-600 rounded-2xl blur-xl opacity-25 group-hover:opacity-40 transition-opacity duration-300"></div>
                    <div className="relative bg-gradient-to-br from-cyan-500 to-cyan-600 hover:from-cyan-600 hover:to-blue-600 text-white rounded-2xl p-6 shadow-2xl border border-cyan-400/20 backdrop-blur-sm transition-all duration-300">
                      <div className="flex flex-col items-center space-y-3">
                        <div className="p-3 bg-white/20 rounded-xl backdrop-blur-sm">
                          <Network className="w-7 h-7" />
                        </div>
                        <span className="font-semibold text-center leading-tight">Configure RAG</span>
                      </div>
                      <div className="absolute top-2 right-2 w-2 h-2 bg-white/30 rounded-full"></div>
                      <div className="absolute bottom-2 left-2 w-1 h-1 bg-white/20 rounded-full"></div>
                    </div>
                  </div>
                </div>
                
                <Link href="/docs" className="group">
                  <div className="relative transform transition-all duration-300 hover:scale-105 hover:-translate-y-2">
                    <div className="absolute inset-0 bg-gradient-to-r from-indigo-500 to-purple-600 rounded-2xl blur-xl opacity-25 group-hover:opacity-40 transition-opacity duration-300"></div>
                    <div className="relative bg-gradient-to-br from-indigo-500 to-indigo-600 hover:from-indigo-600 hover:to-purple-600 text-white rounded-2xl p-6 shadow-2xl border border-indigo-400/20 backdrop-blur-sm transition-all duration-300">
                      <div className="flex flex-col items-center space-y-3">
                        <div className="p-3 bg-white/20 rounded-xl backdrop-blur-sm">
                          <BookOpen className="w-7 h-7" />
                        </div>
                        <span className="font-semibold text-center leading-tight">View Documentation</span>
                      </div>
                      <div className="absolute top-2 right-2 w-2 h-2 bg-white/30 rounded-full"></div>
                      <div className="absolute bottom-2 left-2 w-1 h-1 bg-white/20 rounded-full"></div>
                    </div>
                  </div>
                </Link>
            </div>
          </div>

          {}
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
                    <LinkIcon className="w-4 h-4 text-primary" />
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
          {}
          <Card className="bg-card border-border">
            <CardHeader>
              <div className="flex justify-between items-center">
                <CardTitle className="text-lg font-semibold text-foreground">
                  Connected Sources
                </CardTitle>
                <Link href="/dashboard/urls">
                  <Button variant="outline" size="sm">
                    <Plus className="w-4 h-4 mr-2" />
                    Add URL
                  </Button>
                </Link>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              {}
              {[
                { name: "frontend-app", type: "repository", status: "active", private: false, icon: GitBranch },
                { name: "API Documentation", type: "document", status: "active", private: false, icon: FileText },
                { name: "Confluence Wiki", type: "url", status: "syncing", private: true, icon: LinkIcon },
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

          {}
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
              {}
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

          {}
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
      
        <Footer />
      </div>
    </AuthGuard>
  );
}
