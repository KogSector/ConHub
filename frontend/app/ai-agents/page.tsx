import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ProfileAvatar } from "@/components/ui/profile-avatar";
import { Footer } from "@/components/ui/footer";
import { ArrowLeft, Bot, Plus, Settings, Play, Pause, Activity, Zap, Brain, FileText, Shield } from "lucide-react";
import Link from "next/link";

export default function AIAgentsPage() {
  const agents = [
    {
      id: 1,
      name: "CodeReviewer Pro",
      description: "Advanced code review agent that analyzes pull requests and suggests improvements",
      type: "Code Analysis",
      status: "active",
      lastRun: "5 minutes ago",
      totalRuns: 1247,
      successRate: 98.5,
      model: "GPT-4",
      repositories: ["ConHub", "data-processor"]
    },
    {
      id: 2,
      name: "Documentation Assistant",
      description: "Automatically generates and updates documentation based on code changes",
      type: "Documentation",
      status: "active",
      lastRun: "2 hours ago",
      totalRuns: 856,
      successRate: 96.2,
      model: "Claude-3",
      repositories: ["ConHub", "api-gateway"]
    },
    {
      id: 3,
      name: "Bug Hunter",
      description: "Identifies potential bugs and security vulnerabilities in codebases",
      type: "Security",
      status: "paused",
      lastRun: "1 day ago",
      totalRuns: 432,
      successRate: 94.8,
      model: "GPT-4",
      repositories: ["api-gateway"]
    },
    {
      id: 4,
      name: "Performance Optimizer",
      description: "Analyzes code performance and suggests optimization strategies",
      type: "Performance",
      status: "inactive",
      lastRun: "3 days ago",
      totalRuns: 234,
      successRate: 92.1,
      model: "Claude-3",
      repositories: ["data-processor"]
    }
  ];

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'Code Analysis': return 'bg-green-500 shadow-lg shadow-green-500/50';
      case 'Documentation': return 'bg-green-500 shadow-lg shadow-green-500/50';
      case 'Security': return 'bg-green-500 shadow-lg shadow-green-500/50';
      case 'Performance': return 'bg-green-500 shadow-lg shadow-green-500/50';
      default: return 'bg-gray-400';
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'Code Analysis': return Brain;
      case 'Documentation': return FileText;
      case 'Security': return Shield;
      case 'Performance': return Zap;
      default: return Bot;
    }
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <div className="border-b border-border bg-card/50 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-20">
            <div className="flex items-center space-x-4">
              <Link href="/dashboard">
                <Button variant="ghost" size="sm">
                  <ArrowLeft className="w-4 h-4 mr-2" />
                  Back to Dashboard
                </Button>
              </Link>
              <div className="flex items-center space-x-3">
                <Bot className="w-6 h-6 text-primary" />
                <h1 className="text-2xl font-bold text-foreground">AI Agents</h1>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <Button>
                <Plus className="w-4 h-4 mr-2" />
                Create Agent
              </Button>
              <ProfileAvatar />
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Stats Overview */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Total Agents
              </CardTitle>
              <Bot className="w-4 h-4 text-primary" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">{agents.length}</div>
              <p className="text-xs text-muted-foreground">
                +1 from last week
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Active Agents
              </CardTitle>
              <Activity className="w-4 h-4 text-green-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">
                {agents.filter(a => a.status === 'active').length}
              </div>
              <p className="text-xs text-muted-foreground">
                Currently running
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Total Runs
              </CardTitle>
              <Play className="w-4 h-4 text-blue-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">
                {agents.reduce((sum, agent) => sum + agent.totalRuns, 0).toLocaleString()}
              </div>
              <p className="text-xs text-muted-foreground">
                This month
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Avg Success Rate
              </CardTitle>
              <Zap className="w-4 h-4 text-yellow-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">
                {(agents.reduce((sum, agent) => sum + agent.successRate, 0) / agents.length).toFixed(1)}%
              </div>
              <p className="text-xs text-muted-foreground">
                Across all agents
              </p>
            </CardContent>
          </Card>
        </div>

        {/* Agents List */}
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-xl font-semibold text-foreground">Your AI Agents</h2>
            <Button variant="outline">
              <Settings className="w-4 h-4 mr-2" />
              Global Settings
            </Button>
          </div>

          <div className="grid gap-4">
            {agents.map((agent) => {
              const TypeIcon = getTypeIcon(agent.type);
              return (
                <Card key={agent.id} className="bg-card border-border hover:bg-accent/5 transition-colors">
                  <div className="flex flex-col px-6 py-4 gap-4">
                    <div className="flex items-center gap-2">
                      <div className="p-2 bg-muted rounded-lg flex-shrink-0">
                        <TypeIcon className="w-5 h-5 text-foreground" />
                      </div>
                      <span className="font-semibold text-base text-foreground truncate">{agent.name}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <div className={`w-2 h-2 rounded-full ${
                        agent.status === 'active' ? 'bg-blue-500 shadow-lg shadow-blue-500/50' : 'bg-gray-400'
                      }`}></div>
                      <p className="text-xs text-muted-foreground truncate max-w-xs">{agent.description}</p>
                    </div>
                    <div className="flex items-center gap-2">
                      <div className={`w-2 h-2 rounded-full ${getTypeColor(agent.type)}`}></div>
                      <span className="text-xs text-muted-foreground">{agent.totalRuns.toLocaleString()} requests today</span>
                      <span className="text-xs text-muted-foreground">Model: {agent.model}</span>
                    </div>
                    <Button variant="ghost" size="icon" className="ml-auto"><Settings className="w-4 h-4" /></Button>
                  </div>
                </Card>
              );
            })}
          </div>
        </div>
      </div>
      <Footer />
    </div>
  );
}
