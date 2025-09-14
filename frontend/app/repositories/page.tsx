import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ProfileAvatar } from "@/components/ui/profile-avatar";
import { Footer } from "@/components/ui/footer";
import { ArrowLeft, GitBranch, Plus, Settings, ExternalLink, Star, GitFork } from "lucide-react";
import Link from "next/link";

export default function RepositoriesPage() {
  const repositories = [
    {
      id: 1,
      name: "ConHub",
      description: "A unified platform for connecting repositories, documents, and AI agents",
      language: "TypeScript",
      stars: 42,
      forks: 8,
      lastUpdated: "2 hours ago",
      status: "active",
      url: "https://github.com/KogSector/ConHub"
    },
    {
      id: 2,
      name: "data-processor",
      description: "High-performance data processing pipeline for ML workflows",
      language: "Python",
      stars: 127,
      forks: 23,
      lastUpdated: "1 day ago",
      status: "active",
      url: "https://github.com/KogSector/data-processor"
    },
    {
      id: 3,
      name: "api-gateway",
      description: "Microservices API gateway with authentication and rate limiting",
      language: "Go",
      stars: 89,
      forks: 15,
      lastUpdated: "3 days ago",
      status: "inactive",
      url: "https://github.com/KogSector/api-gateway"
    }
  ];

  const languageColors = {
    TypeScript: "bg-blue-500",
    Python: "bg-yellow-500",
    Go: "bg-cyan-500",
    JavaScript: "bg-yellow-400",
    Rust: "bg-orange-600"
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
                  <ArrowLeft className="w-4 h-4" />
                </Button>
              </Link>
              <div className="flex items-center space-x-3">
                <GitBranch className="w-6 h-6 text-primary" />
                <h1 className="text-2xl font-bold text-foreground">Repositories</h1>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <Button>
                <Plus className="w-4 h-4 mr-2" />
                Connect Repository
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
                Total Repositories
              </CardTitle>
              <GitBranch className="w-4 h-4 text-primary" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">{repositories.length}</div>
              <p className="text-xs text-muted-foreground">
                +2 from last month
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Active Repos
              </CardTitle>
              <GitBranch className="w-4 h-4 text-green-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">
                {repositories.filter(r => r.status === 'active').length}
              </div>
              <p className="text-xs text-muted-foreground">
                Currently syncing
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Total Stars
              </CardTitle>
              <Star className="w-4 h-4 text-yellow-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">
                {repositories.reduce((sum, repo) => sum + repo.stars, 0)}
              </div>
              <p className="text-xs text-muted-foreground">
                Across all repos
              </p>
            </CardContent>
          </Card>

          <Card className="bg-card border-border">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Total Forks
              </CardTitle>
              <GitFork className="w-4 h-4 text-blue-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-foreground">
                {repositories.reduce((sum, repo) => sum + repo.forks, 0)}
              </div>
              <p className="text-xs text-muted-foreground">
                Community contributions
              </p>
            </CardContent>
          </Card>
        </div>

        {/* Repository List */}
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-xl font-semibold text-foreground">Connected Repositories</h2>
            <Button variant="outline">
              <Settings className="w-4 h-4 mr-2" />
              Manage Connections
            </Button>
          </div>

          <div className="grid gap-4">
            {repositories.map((repo) => (
              <Card key={repo.id} className="bg-card border-border hover:bg-accent/5 transition-colors">
                <div className="flex flex-col px-6 py-4 gap-4">
                  <div className="flex items-center gap-2">
                    <div className="p-2 bg-muted rounded-lg flex-shrink-0">
                      <GitBranch className="w-5 h-5 text-foreground" />
                    </div>
                    <span className="font-semibold text-base text-foreground truncate">{repo.name}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className={`w-2 h-2 rounded-full ${
                      repo.status === 'active' ? 'bg-green-500 shadow-lg shadow-green-500/50' : 'bg-gray-400'
                    }`}></div>
                    <p className="text-xs text-muted-foreground truncate max-w-xs">{repo.description}</p>
                  </div>
                  <div className="flex items-center gap-4 text-xs text-muted-foreground">
                    <div className="flex items-center gap-1">
                      <div className={`w-2 h-2 rounded-full ${languageColors[repo.language as keyof typeof languageColors] || 'bg-gray-500'}`}></div>
                      <span>{repo.language}</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <Star className="w-3 h-3" />
                      <span>{repo.stars}</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <GitFork className="w-3 h-3" />
                      <span>{repo.forks}</span>
                    </div>
                    <span>Updated {repo.lastUpdated}</span>
                  </div>
                  <div className="flex items-center gap-2 ml-auto">
                    <Button variant="outline" size="sm" asChild>
                      <Link href={repo.url} target="_blank" rel="noopener noreferrer">
                        <ExternalLink className="w-4 h-4 mr-1" />
                        View on GitHub
                      </Link>
                    </Button>
                    <Button variant="outline" size="sm">
                      <Settings className="w-4 h-4 mr-1" />
                      Configure
                    </Button>
                  </div>
                </div>
              </Card>
            ))}
          </div>
        </div>
      </div>
      <Footer />
    </div>
  );
}
