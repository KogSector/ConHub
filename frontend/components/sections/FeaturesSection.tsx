"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  GitBranch,
  Brain,
  Shield,
  Zap,
  Network,
  Code2,
  Lock,
  Workflow,
  Bot,
  Star,
  FileText,
  Globe,
  Database,
  Download,
  Eye,
  Monitor
} from "lucide-react";

const features = [
  {
    icon: Network,
    title: "Multi-Source Connection",
    description: "Connect repositories, documents, and URLs with seamless integration across all your knowledge sources.",
    badge: "Core Feature"
  },
  {
    icon: GitBranch,
    title: "Repository Integration",
    description: "Connect unlimited public and private repositories with seamless OAuth integration.",
    badge: "Git"
  },
  {
    icon: FileText,
    title: "Document Management",
    description: "Upload and sync documentation, specs, and knowledge base articles for complete context.",
    badge: "Docs"
  },
  {
    icon: Globe,
    title: "URL Monitoring",
    description: "Connect external documentation, wikis, and web resources to keep context up-to-date.",
    badge: "Web"
  },
  {
    icon: Bot,
    title: "AI Agent Integration", 
    description: "Connect Amazon Q, GitHub Copilot, Cline, and other AI coding assistants to access full context.",
    badge: "AI Powered"
  },
  {
    icon: Code2,
    title: "Unified Team Context",
    description: "Bridge the gap between technical and non-technical teams with shared understanding of codebase architecture and implementation details.",
    badge: "Team Sync"
  },
  {
    icon: Brain,
    title: "Unified Context",
    description: "AI agents get complete context across repositories, docs, and URLs for comprehensive understanding.",
    badge: "Smart"
  },
  {
    icon: Shield,
    title: "Enterprise Security",
    description: "Row-level security, encrypted data flow, and granular access controls protect your code.",
    badge: "Secure"
  },

  {
    icon: Lock,
    title: "Privacy First",
    description: "Your code stays private. Only connected AI agents can access your authorized repositories.",
    badge: "Private"
  },
  {
    icon: Workflow,
    title: "Seamless Workflow",
    description: "Frictionless integration with your existing development workflow and IDE setup.",
    badge: "Easy"
  },
  {
    icon: Monitor,
    title: "Developer Experience",
    description: "Beautiful, intuitive interface designed by developers, for developers.",
    badge: "UX"
  },
  {
    icon: Zap,
    title: "Smart Routing",
    description: "Intelligent context routing ensures AI agents get the most relevant information from all sources.",
    badge: "Intelligent"
  },
  {
    icon: Star,
    title: "Community Driven",
    description: "Contribute to and benefit from a growing library of community-sourced integrations and tools.",
    badge: "Community"
  },
  {
    icon: Download,
    title: "Exportable Context",
    description: "Easily export and share context snapshots for debugging, collaboration, or analysis.",
    badge: "Utility"
  },
  {
    icon: Eye,
    title: "Audit Trails",
    description: "Keep track of all activities and data access with comprehensive and immutable audit logs.",
    badge: "Security"
  }
];

export const FeaturesSection = () => {
  return (
    <section id="features" className="py-24 bg-background relative overflow-hidden">
      {}
      <div className="absolute inset-0 bg-gradient-to-b from-transparent via-primary/5 to-transparent" />
      <div className="absolute top-1/2 left-1/4 w-64 h-64 bg-accent/10 rounded-full blur-3xl" />
      <div className="absolute bottom-1/4 right-1/4 w-64 h-64 bg-primary/10 rounded-full blur-3xl" />
      
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
        <div className="text-center space-y-6 mb-20 animate-fade-in-up">
          <Badge variant="secondary" className="w-fit mx-auto bg-primary/10 text-primary border-primary/20 hover:bg-primary/20 transition-colors">
            <Star className="w-3 h-3 mr-1" />
            Powerful Features
          </Badge>
          <h2 className="text-3xl md:text-5xl font-bold text-foreground leading-tight">
            Everything you need for
            <br />
            <span className="bg-gradient-to-r from-primary via-primary-glow to-accent bg-clip-text text-transparent">
              unified development
            </span>
          </h2>
          <p className="text-xl text-muted-foreground max-w-3xl mx-auto leading-relaxed">
            Connect repositories, docs, and URLs. Integrate AI agents and supercharge your development workflow with 
            <span className="text-primary font-semibold">unified context across all your sources</span>.
          </p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
          {features.map((feature, index) => (
            <Card 
              key={index} 
              className="bg-card/50 backdrop-blur-sm border-border/50 hover:bg-card hover:border-border hover:shadow-2xl transition-all duration-500 group animate-fade-in-up"
              style={{ animationDelay: `${index * 0.1}s` }}
            >
              <CardHeader className="pb-4">
                <div className="flex items-center justify-between mb-4">
                  <div className="w-14 h-14 bg-gradient-to-br from-primary/10 to-accent/10 rounded-xl flex items-center justify-center group-hover:from-primary/20 group-hover:to-accent/20 transition-all duration-300 group-hover:scale-110">
                    <feature.icon className="w-7 h-7 text-primary group-hover:text-primary-glow transition-colors" />
                  </div>
                  <Badge 
                    variant="outline" 
                    className="text-xs bg-background/50 border-primary/20 text-primary hover:bg-primary/10 transition-colors"
                  >
                    {feature.badge}
                  </Badge>
                </div>
                <CardTitle className="text-xl font-bold text-foreground group-hover:text-primary transition-colors">
                  {feature.title}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-muted-foreground leading-relaxed group-hover:text-foreground/80 transition-colors">
                  {feature.description}
                </p>
              </CardContent>
            </Card>
          ))}
        </div>
        
        {}
        <div className="text-center mt-20 animate-fade-in-up anim-delay-1500">
          <div className="inline-flex items-center gap-2 text-sm text-muted-foreground bg-card/30 backdrop-blur-sm border border-border/50 rounded-full px-6 py-3">
            <Zap className="w-4 h-4 text-primary" />
            <span>And many more features coming soon...</span>
          </div>
        </div>
      </div>
    </section>
  );
};

export default FeaturesSection;