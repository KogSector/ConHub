'use client';

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Zap, Code, Sparkles, RefreshCw } from 'lucide-react';

export default function CursorPage() {
  const [prompt, setPrompt] = useState('');
  const [context, setContext] = useState('');
  const [response, setResponse] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const handleQuery = async () => {
    if (!prompt.trim()) return;

    setIsLoading(true);
    try {
      const res = await fetch('/api/agents/cursor/query', {
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
        <Zap className="h-8 w-8 text-purple-500" />
        <h1 className="text-3xl font-bold">Cursor IDE</h1>
        <Badge variant="secondary">AI-Powered IDE</Badge>
      </div>
      
      <p className="text-muted-foreground">
        AI-powered IDE with advanced code assistance. Get intelligent code completions, 
        refactoring suggestions, and debugging help.
      </p>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center space-x-2">
              <Code className="h-5 w-5 text-blue-500" />
              <CardTitle className="text-lg">Code Generation</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Generate high-quality code from natural language descriptions.
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center space-x-2">
              <RefreshCw className="h-5 w-5 text-green-500" />
              <CardTitle className="text-lg">Refactoring</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Intelligent code refactoring and optimization suggestions.
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center space-x-2">
              <Sparkles className="h-5 w-5 text-yellow-500" />
              <CardTitle className="text-lg">Smart Completion</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Context-aware code completions and intelligent suggestions.
            </p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Chat with Cursor IDE</CardTitle>
          <CardDescription>
            Get code assistance, refactoring suggestions, or generate code from descriptions.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="prompt">Your Request</Label>
            <Textarea
              id="prompt"
              placeholder="e.g., Create a React component for a user profile card"
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              rows={3}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="context">Code Context (Optional)</Label>
            <Textarea
              id="context"
              placeholder="Paste existing code or provide additional context..."
              value={context}
              onChange={(e) => setContext(e.target.value)}
              rows={2}
            />
          </div>

          <Button onClick={handleQuery} disabled={isLoading || !prompt.trim()}>
            {isLoading ? 'Processing with Cursor...' : 'Ask Cursor IDE'}
          </Button>

          {response && (
            <div className="space-y-2">
              <Label>Cursor IDE Response</Label>
              <div className="p-4 bg-muted rounded-lg border-l-4 border-purple-500">
                <pre className="whitespace-pre-wrap text-sm">{response}</pre>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Example Requests</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
            {[
              "Create a TypeScript interface for user data",
              "Refactor this function to use async/await",
              "Generate a REST API endpoint for user management",
              "Create a responsive navbar component",
              "Write unit tests for this function",
              "Optimize this database query"
            ].map((example) => (
              <Button
                key={example}
                variant="outline"
                size="sm"
                className="justify-start text-left h-auto p-3"
                onClick={() => setPrompt(example)}
              >
                {example}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}