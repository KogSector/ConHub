'use client';

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Bot, Cloud, Shield, Zap } from 'lucide-react';

export default function AmazonQPage() {
  const [prompt, setPrompt] = useState('');
  const [context, setContext] = useState('');
  const [response, setResponse] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const handleQuery = async () => {
    if (!prompt.trim()) return;

    setIsLoading(true);
    try {
      const res = await fetch('/api/agents/amazon-q/query', {
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
        <Bot className="h-8 w-8 text-orange-500" />
        <h1 className="text-3xl font-bold">Amazon Q</h1>
        <Badge variant="secondary">AWS AI Assistant</Badge>
      </div>
      
      <p className="text-muted-foreground">
        AWS AI assistant for development and cloud operations. Get expert guidance on AWS services, 
        best practices, and cloud architecture.
      </p>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center space-x-2">
              <Cloud className="h-5 w-5 text-blue-500" />
              <CardTitle className="text-lg">AWS Guidance</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Expert advice on AWS services, configurations, and best practices.
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center space-x-2">
              <Shield className="h-5 w-5 text-green-500" />
              <CardTitle className="text-lg">Security</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Security recommendations and compliance guidance for AWS workloads.
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center space-x-2">
              <Zap className="h-5 w-5 text-yellow-500" />
              <CardTitle className="text-lg">Optimization</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Performance and cost optimization strategies for your AWS infrastructure.
            </p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Chat with Amazon Q</CardTitle>
          <CardDescription>
            Ask questions about AWS services, get architecture advice, or troubleshoot issues.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="prompt">Your Question</Label>
            <Textarea
              id="prompt"
              placeholder="e.g., How do I set up a secure VPC for my web application?"
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              rows={3}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="context">AWS Context (Optional)</Label>
            <Textarea
              id="context"
              placeholder="Provide details about your AWS environment, services used, or specific requirements..."
              value={context}
              onChange={(e) => setContext(e.target.value)}
              rows={2}
            />
          </div>

          <Button onClick={handleQuery} disabled={isLoading || !prompt.trim()}>
            {isLoading ? 'Consulting Amazon Q...' : 'Ask Amazon Q'}
          </Button>

          {response && (
            <div className="space-y-2">
              <Label>Amazon Q Response</Label>
              <div className="p-4 bg-muted rounded-lg border-l-4 border-orange-500">
                <pre className="whitespace-pre-wrap text-sm">{response}</pre>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Example Queries</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
            {[
              "How do I secure my S3 bucket?",
              "What's the best EC2 instance type for my workload?",
              "How to set up auto-scaling for my application?",
              "Best practices for Lambda function optimization",
              "How to implement disaster recovery on AWS?",
              "Cost optimization strategies for RDS"
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