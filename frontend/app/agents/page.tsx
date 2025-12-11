'use client';

import React from 'react';
import Link from 'next/link';
import { Button } from "@/components/ui/button";
import { ProfileAvatar } from "@/components/ui/ProfileAvatar";
import { Footer } from "@/components/ui/footer";
import { ArrowLeft, Bot } from "lucide-react";

export default function AgentsPage() {
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
                <Bot className="w-6 h-6 text-primary" />
                <h1 className="text-2xl font-bold text-foreground">AI Agents</h1>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <ProfileAvatar />
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <AgentCard 
            title="General Agents"
            description="Connect and manage general purpose AI agents"
            href="/agents/general"
          />
          <AgentCard 
            title="GitHub Copilot"
            description="Connect and configure GitHub Copilot integration"
            href="/agents/github-copilot"
          />
          <AgentCard 
            title="Amazon Q"
            description="Connect and configure Amazon Q integration"
            href="/agents/amazon-q"
          />
          <AgentCard 
            title="Cline"
            description="Connect and configure Cline AI agent integration"
            href="/agents/cline"
          />
          <AgentCard 
            title="Cursor"
            description="Connect and configure Cursor AI agent integration"
            href="/agents/cursor"
          />
          <AgentCard 
            title="Custom Agents"
            description="Create and manage custom AI agents with specific rules"
            href="/agents/custom"
          />
        </div>
      </div>
      <Footer />
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