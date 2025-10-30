"use client"

import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/ScrollArea';
import { useToast } from '@/hooks/use-toast';
import { apiClient, unwrapResponse } from '@/lib/api';
import { 
  Github, 
  Users, 
  GitBranch, 
  Star, 
  Bot,
  Activity,
  Building2,
  Search,
  Code,
  BookOpen
} from 'lucide-react';

interface Repository {
  id: number;
  name: string;
  full_name: string;
  private: boolean;
  html_url: string;
  description?: string;
  language?: string;
  stargazers_count: number;
  forks_count: number;
  updated_at: string;
  topics: string[];
}

interface Organization {
  login: string;
  id: number;
  avatar_url: string;
  description?: string;
  name?: string;
}

interface CopilotUsage {
  total_seats: number;
  seats: Array<{
    assignee: {
      login: string;
      id: number;
      type: string;
    };
    last_activity_at?: string;
    last_activity_editor?: string;
    created_at: string;
  }>;
}

export default function GitHubCopilotDashboard() {
  const [githubToken, setGithubToken] = useState('');
  const [isConnected, setIsConnected] = useState(false);
  const [currentUser, setCurrentUser] = useState<any>(null);
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [organizations, setOrganizations] = useState<Organization[]>([]);
  const [copilotUsage, setCopilotUsage] = useState<CopilotUsage | null>(null);
  const [selectedOrg, setSelectedOrg] = useState<string>('');
  const [searchQuery, setSearchQuery] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const { toast } = useToast();

  
  const connectToGitHub = async () => {
    if (!githubToken.trim()) {
      toast({
        title: 'Error',
        description: 'Please enter a GitHub token',
        variant: 'destructive',
      });
      return;
    }

    setIsLoading(true);
    try {
  const resp = await apiClient.get('/api/github/user', { Authorization: `Bearer ${githubToken}` });
  const data = unwrapResponse<{ success?: boolean; data?: any }>(resp) ?? resp;
  setCurrentUser((data && data.data) || data || null);
  setIsConnected(true);
      toast({
        title: 'Success',
        description: `Connected to GitHub as ${data.data.login}`,
      });

      
      await Promise.all([
        loadRepositories(),
        loadOrganizations(),
      ]);
    } catch (error: any) {
      toast({
        title: 'Error',
        description: error.message,
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  
  const loadRepositories = async () => {
    try {
  const resp = await apiClient.get('/api/github/repositories', { Authorization: `Bearer ${githubToken}` });
  const data = unwrapResponse<{ success?: boolean; data?: Repository[] }>(resp) ?? resp;
  setRepositories((data && data.data) || data || []);
    } catch (error) {
      console.error('Failed to load repositories:', error);
    }
  };

  
  const loadOrganizations = async () => {
    try {
  const resp = await apiClient.get('/api/github/organizations', { Authorization: `Bearer ${githubToken}` });
  const data = unwrapResponse<{ success?: boolean; data?: Organization[] }>(resp) ?? resp;
  setOrganizations((data && data.data) || data || []);
    } catch (error) {
      console.error('Failed to load organizations:', error);
    }
  };

  
  const loadCopilotUsage = async (org: string) => {
    try {
      const resp = await apiClient.get(`/api/copilot/seats/${org}`, { Authorization: `Bearer ${githubToken}` });
      const data = unwrapResponse<{ success?: boolean; data?: CopilotUsage }>(resp) ?? resp;
      if (data && data.data) {
        setCopilotUsage(data.data);
      } else {
        toast({
          title: 'Warning',
          description: 'Unable to load Copilot usage. You may need additional permissions.',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Failed to load Copilot usage:', error);
    }
  };

  
  const searchRepositories = async () => {
    if (!searchQuery.trim()) return;

    setIsLoading(true);
    try {
  const resp = await apiClient.get(`/api/github/search/repositories?q=${encodeURIComponent(searchQuery)}`, { Authorization: `Bearer ${githubToken}` });
  const data = unwrapResponse<{ success?: boolean; data?: { items?: Repository[] } }>(resp) ?? resp;
  setRepositories(((data && data.data && data.data.items) || (data && (data as Repository[])) || []) as Repository[]);
    } catch (error) {
      toast({
        title: 'Error',
        description: 'Failed to search repositories',
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  
  const handleOrganizationSelect = async (org: string) => {
    setSelectedOrg(org);
    if (org) {
      await loadCopilotUsage(org);
    }
  };

  
  const disconnect = () => {
    setGithubToken('');
    setIsConnected(false);
    setCurrentUser(null);
    setRepositories([]);
    setOrganizations([]);
    setCopilotUsage(null);
    setSelectedOrg('');
  };

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold flex items-center gap-2">
            <Github className="h-8 w-8" />
            GitHub + Copilot Integration
          </h1>
          <p className="text-muted-foreground">
            Connect to GitHub and manage Copilot access across your repositories and organizations
          </p>
        </div>
        {isConnected && (
          <Button variant="outline" onClick={disconnect}>
            Disconnect
          </Button>
        )}
      </div>

      {!isConnected ? (
        <Card>
          <CardHeader>
            <CardTitle>Connect to GitHub</CardTitle>
            <CardDescription>
              Enter your GitHub personal access token to connect
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="github-token">GitHub Personal Access Token</Label>
              <Input
                id="github-token"
                type="password"
                placeholder="ghp_..."
                value={githubToken}
                onChange={(e) => setGithubToken(e.target.value)}
              />
              <p className="text-sm text-muted-foreground">
                Token needs scopes: repo, read:org, admin:org (for Copilot management)
              </p>
            </div>
            <Button onClick={connectToGitHub} disabled={isLoading} className="w-full">
              {isLoading ? 'Connecting...' : 'Connect to GitHub'}
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-6">
          {}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <img
                  src={currentUser?.avatar_url}
                  alt={currentUser?.login}
                  className="w-8 h-8 rounded-full"
                />
                Welcome, {currentUser?.name || currentUser?.login}
              </CardTitle>
              <CardDescription>
                {currentUser?.bio && <p>{currentUser.bio}</p>}
                <div className="flex items-center gap-4 mt-2">
                  <span className="flex items-center gap-1">
                    <Users className="h-4 w-4" />
                    {currentUser?.followers} followers
                  </span>
                  <span className="flex items-center gap-1">
                    <GitBranch className="h-4 w-4" />
                    {currentUser?.public_repos} repositories
                  </span>
                </div>
              </CardDescription>
            </CardHeader>
          </Card>

          <Tabs defaultValue="repositories" className="space-y-4">
            <TabsList className="grid w-full grid-cols-4">
              <TabsTrigger value="repositories">Repositories</TabsTrigger>
              <TabsTrigger value="organizations">Organizations</TabsTrigger>
              <TabsTrigger value="copilot">Copilot Usage</TabsTrigger>
              <TabsTrigger value="search">Search</TabsTrigger>
            </TabsList>

            <TabsContent value="repositories" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <BookOpen className="h-5 w-5" />
                    Your Repositories ({repositories.length})
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <ScrollArea className="h-96">
                    <div className="space-y-4">
                      {repositories.map((repo) => (
                        <div key={repo.id} className="border rounded-lg p-4 space-y-2">
                          <div className="flex items-center justify-between">
                            <h3 className="font-semibold flex items-center gap-2">
                              <GitBranch className="h-4 w-4" />
                              {repo.name}
                              {repo.private && <Badge variant="secondary">Private</Badge>}
                            </h3>
                            <div className="flex items-center gap-2">
                              <span className="flex items-center gap-1 text-sm">
                                <Star className="h-3 w-3" />
                                {repo.stargazers_count}
                              </span>
                              <span className="flex items-center gap-1 text-sm">
                                <GitBranch className="h-3 w-3" />
                                {repo.forks_count}
                              </span>
                            </div>
                          </div>
                          {repo.description && (
                            <p className="text-sm text-muted-foreground">{repo.description}</p>
                          )}
                          <div className="flex items-center gap-2 flex-wrap">
                            {repo.language && (
                              <Badge variant="outline">{repo.language}</Badge>
                            )}
                            {repo.topics.map((topic) => (
                              <Badge key={topic} variant="secondary" className="text-xs">
                                {topic}
                              </Badge>
                            ))}
                          </div>
                          <p className="text-xs text-muted-foreground">
                            Updated {new Date(repo.updated_at).toLocaleDateString()}
                          </p>
                        </div>
                      ))}
                    </div>
                  </ScrollArea>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="organizations" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Building2 className="h-5 w-5" />
                    Organizations ({organizations.length})
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {organizations.map((org) => (
                      <div
                        key={org.id}
                        className="border rounded-lg p-4 cursor-pointer hover:bg-accent transition-colors"
                        onClick={() => handleOrganizationSelect(org.login)}
                      >
                        <div className="flex items-center gap-3">
                          <img
                            src={org.avatar_url}
                            alt={org.login}
                            className="w-10 h-10 rounded-full"
                          />
                          <div>
                            <h3 className="font-semibold">{org.name || org.login}</h3>
                            {org.description && (
                              <p className="text-sm text-muted-foreground">{org.description}</p>
                            )}
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="copilot" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Bot className="h-5 w-5" />
                    GitHub Copilot Usage
                  </CardTitle>
                  <CardDescription>
                    Select an organization to view Copilot usage and manage seats
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  {selectedOrg && copilotUsage ? (
                    <div className="space-y-4">
                      <div className="flex items-center justify-between">
                        <h3 className="text-lg font-semibold">
                          Organization: {selectedOrg}
                        </h3>
                        <Badge variant="outline">
                          {copilotUsage.total_seats} Total Seats
                        </Badge>
                      </div>
                      
                      <ScrollArea className="h-64">
                        <div className="space-y-2">
                          {copilotUsage.seats.map((seat, index) => (
                            <div key={index} className="border rounded p-3 flex items-center justify-between">
                              <div className="flex items-center gap-2">
                                <Users className="h-4 w-4" />
                                <span className="font-medium">{seat.assignee.login}</span>
                                <Badge variant="secondary">{seat.assignee.type}</Badge>
                              </div>
                              <div className="text-sm text-muted-foreground">
                                {seat.last_activity_at ? (
                                  <span className="flex items-center gap-1">
                                    <Activity className="h-3 w-3" />
                                    Last active: {new Date(seat.last_activity_at).toLocaleDateString()}
                                    {seat.last_activity_editor && ` (${seat.last_activity_editor})`}
                                  </span>
                                ) : (
                                  'No recent activity'
                                )}
                              </div>
                            </div>
                          ))}
                        </div>
                      </ScrollArea>
                    </div>
                  ) : (
                    <div className="text-center py-8">
                      <Bot className="h-16 w-16 mx-auto text-muted-foreground mb-4" />
                      <p className="text-muted-foreground">
                        Select an organization from the Organizations tab to view Copilot usage
                      </p>
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="search" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Search className="h-5 w-5" />
                    Search Repositories
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex gap-2">
                    <Input
                      placeholder="Search repositories..."
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      onKeyPress={(e) => e.key === 'Enter' && searchRepositories()}
                    />
                    <Button onClick={searchRepositories} disabled={isLoading}>
                      {isLoading ? 'Searching...' : 'Search'}
                    </Button>
                  </div>
                  
                  {repositories.length > 0 && (
                    <ScrollArea className="h-96">
                      <div className="space-y-4">
                        {repositories.map((repo) => (
                          <div key={repo.id} className="border rounded-lg p-4 space-y-2">
                            <div className="flex items-center justify-between">
                              <h3 className="font-semibold flex items-center gap-2">
                                <Code className="h-4 w-4" />
                                {repo.full_name}
                                {repo.private && <Badge variant="secondary">Private</Badge>}
                              </h3>
                              <Button variant="outline" size="sm" asChild>
                                <a href={repo.html_url} target="_blank" rel="noopener noreferrer">
                                  View on GitHub
                                </a>
                              </Button>
                            </div>
                            {repo.description && (
                              <p className="text-sm text-muted-foreground">{repo.description}</p>
                            )}
                            <div className="flex items-center gap-4 text-sm">
                              <span className="flex items-center gap-1">
                                <Star className="h-3 w-3" />
                                {repo.stargazers_count}
                              </span>
                              <span className="flex items-center gap-1">
                                <GitBranch className="h-3 w-3" />
                                {repo.forks_count}
                              </span>
                              {repo.language && (
                                <Badge variant="outline">{repo.language}</Badge>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    </ScrollArea>
                  )}
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>
        </div>
      )}
    </div>
  );
}