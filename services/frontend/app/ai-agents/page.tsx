import React from 'react';
import Link from 'next/link';

export default function AIAgentsPage() {
  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-3xl font-bold mb-6">AI Agents</h1>
      
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <AgentCard 
          title="General Agents"
          description="Connect and manage general purpose AI agents"
          href="/ai-agents/general"
        />
        <AgentCard 
          title="GitHub Copilot"
          description="Connect and configure GitHub Copilot integration"
          href="/ai-agents/github-copilot"
        />
        <AgentCard 
          title="Amazon Q"
          description="Connect and configure Amazon Q integration"
          href="/ai-agents/amazon-q"
        />
        <AgentCard 
          title="Custom Agents"
          description="Create and manage custom AI agents with specific rules"
          href="/ai-agents/custom"
        />
      </div>
    </div>
  );
}

function AgentCard({ title, description, href }: { title: string; description: string; href: string }) {
  return (
    <Link href={href}>
      <div className="border rounded-lg p-6 hover:shadow-md transition-shadow cursor-pointer">
        <h2 className="text-xl font-semibold mb-2">{title}</h2>
        <p className="text-gray-600">{description}</p>
      </div>
    </Link>
  );
}