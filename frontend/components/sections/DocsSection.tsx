import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { 
  BookOpen, 
  Github, 
  Bot, 
  Settings, 
  Shield, 
  Zap,
  ArrowRight,
  Play,
  FileText
} from "lucide-react";

const docSections = [
  {
    icon: Play,
    title: "Getting Started",
    description: "Quick setup guide to connect your first repository and AI agent in under 5 minutes.",
    topics: ["Account setup", "GitHub OAuth", "First repository", "AI agent connection"],
    badge: "Beginner"
  },
  {
    icon: Github,
    title: "Repository Management",
    description: "Learn how to connect, organize, and manage multiple repositories with advanced permissions.",
    topics: ["Public/Private repos", "Organization access", "Branch management", "Webhooks"],
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
    description: "Customize GitSync Pro to match your workflow with advanced configuration options.",
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
    icon: Zap,
    title: "API Reference",
    description: "Complete API documentation for integrating GitSync Pro into your custom workflows.",
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
            <span className="bg-gradient-primary bg-clip-text text-transparent"> Documentation</span>
          </h2>
          <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
            Everything you need to master GitSync Pro, from basic setup to advanced configurations and API integration.
          </p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6 mb-12">
          {docSections.map((section, index) => (
            <Card key={index} className="bg-card border-border hover:shadow-card transition-all duration-300 group cursor-pointer">
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
              <CardContent className="space-y-4">
                <p className="text-muted-foreground leading-relaxed">
                  {section.description}
                </p>
                <div className="space-y-2">
                  <div className="text-sm font-medium text-foreground">Topics covered:</div>
                  <div className="flex flex-wrap gap-1">
                    {section.topics.map((topic, i) => (
                      <Badge key={i} variant="secondary" className="text-xs">
                        {topic}
                      </Badge>
                    ))}
                  </div>
                </div>
                <Button variant="ghost" size="sm" className="w-full group-hover:bg-primary/10 transition-colors">
                  Read Documentation
                  <ArrowRight className="w-4 h-4 ml-2" />
                </Button>
              </CardContent>
            </Card>
          ))}
        </div>

        {/* Quick Links */}
        <div className="bg-gradient-card rounded-2xl p-8 border border-border">
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
              <Github className="w-6 h-6 text-primary" />
              <span>Connect GitHub</span>
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