'use client'

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { 
  BookOpen, 
  Bot, 
  Settings, 
  Shield, 
  ArrowRight,
  Play,
  FileText,
  GitBranch,
  Database,
  Cloud,
  Server,
  Webhook,
  Globe
} from "lucide-react";

const docSections = [
  {
    icon: Play,
    title: "Getting Started",
    description: "Quick setup guide to connect your first sources (repos, docs, URLs) and AI agent in under 5 minutes.",
    topics: ["Account setup", "GitHub OAuth", "First sources", "AI agent connection"],
    badge: "Beginner"
  },
  {
    icon: GitBranch,
    title: "Source Management",
    description: "Learn how to connect, organize, and manage repositories, documents, and URLs with advanced permissions.",
    topics: ["Repositories", "Documents", "URLs", "Access controls"],
    badge: "Core"
  },
  {
    icon: Bot,
    title: "AI Agent Integration",
    description: "Comprehensive guide to connecting and configuring AI coding assistants for optimal performance.",
    topics: ["Amazon Q setup", "GitHub Copilot", "Cline integration", "Custom agents"],
    badge: "Advanced"
  },
  {
    icon: Settings,
    title: "Configuration",
    description: "Customize ConHub to match your workflow with advanced configuration options.",
    topics: ["Context filters", "Access controls", "Webhooks", "API settings"],
    badge: "Config"
  },
  {
    icon: Shield,
    title: "Security & Privacy",
    description: "Understand our security model and learn how to protect your code and maintain privacy.",
    topics: ["Data encryption", "Access controls", "Audit logs", "Compliance"],
    badge: "Security"
  },
  {
    icon: Webhook,
    title: "API Reference",
    description: "Complete API documentation for integrating ConHub into your custom workflows.",
    topics: ["Authentication", "Endpoints", "Webhooks", "SDKs"],
    badge: "Developer"
  }
];

export const DocsSection = () => {
  return (
    <section id="docs" className="py-24 bg-muted/20">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="text-center space-y-4 mb-16">
          <Badge variant="secondary" className="w-fit mx-auto">
            <BookOpen className="w-3 h-3 mr-1" />
            Documentation
          </Badge>
          <h2 className="text-3xl md:text-4xl font-bold text-foreground">
            Complete 
            <span className="bg-gradient-to-r from-primary to-primary-glow bg-clip-text text-transparent"> Documentation</span>
          </h2>
          <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
            Everything you need to master ConHub, from connecting your first sources to advanced AI agent configurations.
          </p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6 mb-12">
          {docSections.map((section, index) => (
            <Card key={index} className="bg-card border-border hover:shadow-card transition-all duration-300 group cursor-pointer flex flex-col h-full">
              <CardHeader>
                <div className="flex items-center justify-between mb-2">
                  <div className="w-12 h-12 bg-primary/10 rounded-lg flex items-center justify-center group-hover:bg-primary/20 transition-colors">
                    <section.icon className="w-6 h-6 text-primary" />
                  </div>
                  <Badge variant="outline" className="text-xs">
                    {section.badge}
                  </Badge>
                </div>
                <CardTitle className="text-lg font-semibold text-foreground group-hover:text-primary transition-colors">
                  {section.title}
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4 flex-grow flex flex-col">
                <p className="text-muted-foreground leading-relaxed">
                  {section.description}
                </p>
                <div className="space-y-2 flex-grow">
                  <div className="text-sm font-medium text-foreground">Topics covered:</div>
                  <div className="flex flex-wrap gap-1">
                    {section.topics.map((topic, i) => (
                      <Badge key={i} variant="secondary" className="text-xs">
                        {topic}
                      </Badge>
                    ))}
                  </div>
                </div>
                <button 
                  className="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium h-9 px-3 w-full bg-transparent border-0 text-white transition-all duration-300 focus-visible:outline-none"
                  style={{
                    color: 'white',
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.boxShadow = '0 0 20px rgba(251, 146, 60, 0.8)';
                    e.currentTarget.style.transform = 'scale(1.05)';
                    e.currentTarget.style.backgroundColor = 'rgba(251, 146, 60, 0.1)';
                    e.currentTarget.style.color = 'white';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.boxShadow = 'none';
                    e.currentTarget.style.transform = 'scale(1)';
                    e.currentTarget.style.backgroundColor = 'transparent';
                    e.currentTarget.style.color = 'white';
                  }}
                >
                  Read Documentation
                  <ArrowRight className="w-4 h-4 ml-2" />
                </button>
              </CardContent>
            </Card>
          ))}
        </div>

        {/* Quick Links */}
        <div className="bg-card rounded-2xl p-8 border border-border">
          <div className="text-center space-y-4 mb-8">
            <h3 className="text-2xl font-bold text-foreground">Quick Access</h3>
            <p className="text-muted-foreground">Jump directly to the most common documentation sections</p>
          </div>
          
          <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-4">
            <Button variant="outline" className="h-auto p-4 flex flex-col items-center space-y-2">
              <Play className="w-6 h-6 text-primary" />
              <span>5-Minute Setup</span>
            </Button>
            <Button variant="outline" className="h-auto p-4 flex flex-col items-center space-y-2">
              <Globe className="w-6 h-6 text-primary" />
              <span>Connect Sources</span>
            </Button>
            <Button variant="outline" className="h-auto p-4 flex flex-col items-center space-y-2">
              <Bot className="w-6 h-6 text-primary" />
              <span>Add AI Agents</span>
            </Button>
            <Button variant="outline" className="h-auto p-4 flex flex-col items-center space-y-2">
              <FileText className="w-6 h-6 text-primary" />
              <span>API Reference</span>
            </Button>
          </div>
        </div>
      </div>
    </section>
  );
};