"use client";

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Copy, Check } from 'lucide-react';

const ClinePage = () => {
  const [prompt, setPrompt] = useState('');
  const [context, setContext] = useState('');
  const [response, setResponse] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const handleQuery = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const res = await fetch('/api/agents/cline/query', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ prompt, context }),
      });
      if (!res.ok) {
        throw new Error('Failed to fetch response from Cline agent');
      }
      const data = await res.json();
      setResponse(data.response);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An unknown error occurred');
    } finally {
      setIsLoading(false);
    }
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(response);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="container mx-auto p-4">
      <header className="text-center mb-8">
        <h1 className="text-4xl font-bold">Cline Agent</h1>
        <p className="text-lg text-gray-600">Your AI-powered software engineer for complex tasks</p>
      </header>

      <Card className="mb-8">
        <CardHeader>
          <CardTitle>Query Cline</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <Input
              placeholder="Enter your prompt"
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
            />
            <Input
              placeholder="Enter any relevant context (optional)"
              value={context}
              onChange={(e) => setContext(e.target.value)}
            />
            <Button onClick={handleQuery} disabled={isLoading}>
              {isLoading ? 'Loading...' : 'Send Query'}
            </Button>
          </div>
        </CardContent>
      </Card>

      {error && <p className="text-red-500">{error}</p>}

      {response && (
        <Card>
          <CardHeader className="flex justify-between items-center">
            <CardTitle>Response</CardTitle>
            <Button variant="ghost" size="sm" onClick={handleCopy}>
              {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
            </Button>
          </CardHeader>
          <CardContent>
            <pre className="bg-gray-100 p-4 rounded-md overflow-x-auto">
              <code>{response}</code>
            </pre>
          </CardContent>
        </Card>
      )}
    </div>
  );
};

export default ClinePage;
