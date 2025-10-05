'use client';

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Textarea } from '@/components/ui/textarea';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Bot, Github, Zap, Code, MessageSquare } from 'lucide-react';

interface AIAgent {
  id: string;
  name: string;
  description: string;
  icon: React.ReactNode;
  capabilities: string[];
  endpoint: string;
  status: 'connected' | 'disconnected' | 'connecting';
}

const aiAgents: AIAgent[] = [
  {
    id: 'github_copilot',
    name: 'GitHub Copilot',
    description: 'AI pair programmer for code assistance and completion',
    icon: <Github className="h-6 w-6" />,
    capabilities: ['Code Completion', 'Code Explanation', 'Bug Fixing', 'Code Review'],
    endpoint: '/api/agents/github-copilot/query',
    status: 'disconnected'
  },
  {
    id: 'amazon_q',
    name: 'Amazon Q',
    description: 'AWS AI assistant for development and cloud operations',
    icon: <Bot className="h-6 w-6" />,
    capabilities: ['AWS Guidance', 'Code Assistance', 'Cloud Architecture', 'Best Practices'],
    endpoint: '/api/agents/amazon-q/query',
    status: 'disconnected'
  },
  {
    id: 'cline',
    name: 'Cline',
    description: 'AI-powered software engineer for complex tasks',
    icon: <Code className="h-6 w-6" />,
    capabilities: ['Code Generation', 'Refactoring', 'Project Scaffolding', 'Debugging'],
    endpoint: '/api/agents/cline/query',
    status: 'disconnected'
  },
  {
    id: 'cursor_ide',
    name: 'Cursor IDE',
    description: 'AI-powered IDE with advanced code assistance',
    icon: <Zap className="h-6 w-6" />,
    capabilities: ['Code Completion', 'Code Generation', 'Refactoring', 'Debugging'],
    endpoint: '/api/agents/cursor/query',
    status: 'disconnected'
  }
];

export default function AIAgentsPage() {
  const [selectedAgent, setSelectedAgent] = useState<AIAgent>(aiAgents[0]);
  const [prompt, setPrompt] = useState('');
  const [context, setContext] = useState('');
  const [response, setResponse] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const handleQuery = async () => {
    if (!prompt.trim()) return;

    setIsLoading(true);
    try {
      const res = await fetch(selectedAgent.endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ prompt, context: context || undefined }),
      });

      const data = await res.json();
      setResponse(data.response || data.error || 'No response received');
    } catch (error) {
      setResponse(`Error: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex items-center space-x-2">
        <MessageSquare className="h-8 w-8" />
        <h1 className="text-3xl font-bold">AI Agents</h1>
      </div>
      
      <p className="text-muted-foreground">
        Connect and interact with various AI agents to enhance your development workflow.
      </p>

      <Tabs defaultValue="agents" className="space-y-6">
        <TabsList>
          <TabsTrigger value="agents">Available Agents</TabsTrigger>
          <TabsTrigger value="chat">Chat Interface</TabsTrigger>
        </TabsList>

        <TabsContent value="agents" className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {aiAgents.map((agent) => (
              <Card key={agent.id} className="cursor-pointer hover:shadow-md transition-shadow"
                    onClick={() => setSelectedAgent(agent)}>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-2">
                      {agent.icon}
                      <CardTitle>{agent.name}</CardTitle>
                    </div>
                    <Badge variant={agent.status === 'connected' ? 'default' : 'secondary'}>
                      {agent.status}
                    </Badge>
                  </div>
                  <CardDescription>{agent.description}</CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2">
                    <Label className="text-sm font-medium">Capabilities:</Label>
                    <div className="flex flex-wrap gap-1">
                      {agent.capabilities.map((capability) => (
                        <Badge key={capability} variant="outline" className="text-xs">
                          {capability}
                        </Badge>
                      ))}
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        <TabsContent value="chat" className="space-y-4">
          <Card>
            <CardHeader>
              <div className="flex items-center space-x-2">
                {selectedAgent.icon}
                <CardTitle>Chat with {selectedAgent.name}</CardTitle>
              </div>
              <CardDescription>{selectedAgent.description}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="prompt">Prompt</Label>
                <Textarea
                  id="prompt"
                  placeholder="Enter your question or request..."
                  value={prompt}
                  onChange={(e) => setPrompt(e.target.value)}
                  rows={3}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="context">Context (Optional)</Label>
                <Textarea
                  id="context"
                  placeholder="Provide additional context for better responses..."
                  value={context}
                  onChange={(e) => setContext(e.target.value)}
                  rows={2}
                />
              </div>

              <Button onClick={handleQuery} disabled={isLoading || !prompt.trim()}>
                {isLoading ? 'Processing...' : 'Send Query'}
              </Button>

              {response && (
                <div className="space-y-2">
                  <Label>Response</Label>
                  <div className="p-4 bg-muted rounded-lg">
                    <pre className="whitespace-pre-wrap text-sm">{response}</pre>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}