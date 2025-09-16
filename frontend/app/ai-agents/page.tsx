'use client';

import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Plus, MessageSquare, Bot } from 'lucide-react';

interface AIAgent {
  id: string;
  name: string;
  type: 'github-copilot' | 'amazon-q' | 'custom';
  status: 'connected' | 'disconnected' | 'error';
}

export default function AIAgentsPage() {
  const [agents, setAgents] = useState<AIAgent[]>([]);
  const [query, setQuery] = useState('');
  const [response, setResponse] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    fetchAgents();
    // Register default agents
    registerDefaultAgents();
  }, []);

  const fetchAgents = async () => {
    try {
      const response = await fetch('/api/ai-agents');
      const data = await response.json();
      if (data.success) {
        setAgents(data.agents);
      }
    } catch (error) {
      console.error('Error fetching AI agents:', error);
    }
  };

  const registerDefaultAgents = async () => {
    const defaultAgents = [
      { name: 'GitHub Copilot', type: 'github-copilot', config: {} },
      { name: 'Amazon Q', type: 'amazon-q', config: {} }
    ];

    for (const agent of defaultAgents) {
      try {
        await fetch('/api/ai-agents/register', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(agent)
        });
      } catch (error) {
        console.error('Error registering agent:', error);
      }
    }
    
    fetchAgents();
  };

  const queryAgent = async () => {
    if (!query.trim()) return;
    
    setLoading(true);
    try {
      const response = await fetch('/api/ai-agents/query', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query, includeContext: true })
      });
      
      const data = await response.json();
      if (data.success) {
        setResponse(data.response);
      }
    } catch (error) {
      console.error('Error querying agent:', error);
      setResponse('Error querying AI agent');
    } finally {
      setLoading(false);
    }
  };

  const getAgentIcon = (type: string) => {
    switch (type) {
      case 'github-copilot': return '🐙';
      case 'amazon-q': return '🤖';
      default: return '🔧';
    }
  };

  return (
    <div className="container mx-auto p-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold">AI Agents</h1>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* AI Agents List */}
        <div>
          <h2 className="text-xl font-semibold mb-4">Connected Agents</h2>
          <div className="space-y-4">
            {agents.map((agent) => (
              <Card key={agent.id}>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium flex items-center">
                    <span className="mr-2">{getAgentIcon(agent.type)}</span>
                    {agent.name}
                  </CardTitle>
                  <Badge variant={agent.status === 'connected' ? 'default' : 'secondary'}>
                    {agent.status}
                  </Badge>
                </CardHeader>
                <CardContent>
                  <p className="text-xs text-muted-foreground">
                    Type: {agent.type}
                  </p>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>

        {/* AI Query Interface */}
        <div>
          <h2 className="text-xl font-semibold mb-4">Query AI Agent</h2>
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center">
                <MessageSquare className="w-5 h-5 mr-2" />
                Ask a Question
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <Input
                  placeholder="Ask about your code, docs, or repositories..."
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && queryAgent()}
                />
              </div>
              
              <Button 
                onClick={queryAgent} 
                disabled={!query.trim() || loading}
                className="w-full"
              >
                <Bot className="w-4 h-4 mr-2" />
                {loading ? 'Thinking...' : 'Ask AI'}
              </Button>

              {response && (
                <div className="mt-4">
                  <h3 className="font-medium mb-2">Response:</h3>
                  <Textarea
                    value={response}
                    readOnly
                    className="min-h-[200px]"
                  />
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </div>

      {agents.length === 0 && (
        <div className="text-center py-12">
          <Bot className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
          <p className="text-muted-foreground mb-4">No AI agents connected yet.</p>
          <p className="text-sm text-muted-foreground">
            AI agents will be automatically registered when you start the service.
          </p>
        </div>
      )}
    </div>
  );
}