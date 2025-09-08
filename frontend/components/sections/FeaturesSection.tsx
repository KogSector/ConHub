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
  Star
} from "lucide-react";

const features = [
  {
    icon: GitBranch,
    title: "Multi-Repository Connection",
    description: "Connect unlimited public and private repositories with seamless OAuth integration.",
    badge: "Core Feature"
  },
  {
    icon: Brain,
    title: "AI Agent Integration", 
    description: "Connect Amazon Q, GitHub Copilot, Cline, and other AI coding assistants to access full context.",
    badge: "AI Powered"
  },
  {
    icon: Network,
    title: "Cross-Repo Context",
    description: "AI agents get complete context across all connected repositories for better microservices development.",
    badge: "Smart"
  },
  {
    icon: Shield,
    title: "Enterprise Security",
    description: "Row-level security, encrypted data flow, and granular access controls protect your code.",
    badge: "Secure"
  },
  {
    icon: Zap,
    title: "Lightning Fast",
    description: "Optimized backend with advanced data structures for instant repository synchronization.",
    badge: "Performance"
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
    icon: Code2,
    title: "Developer Experience",
    description: "Beautiful, intuitive interface designed by developers, for developers.",
    badge: "UX"
  },
  {
    icon: Bot,
    title: "Smart Routing",
    description: "Intelligent context routing ensures AI agents get the most relevant code and documentation.",
    badge: "Intelligent"
  }
];

export const FeaturesSection = () => {
  return (
    <section id="features" className="py-24 bg-background">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="text-center space-y-4 mb-16">
          <Badge variant="secondary" className="w-fit mx-auto">
            <Star className="w-3 h-3 mr-1" />
            Features
          </Badge>
          <h2 className="text-3xl md:text-4xl font-bold text-foreground">
            Everything you need for 
            <span className="bg-gradient-to-r from-primary to-primary-glow bg-clip-text text-transparent"> unified development</span>
          </h2>
          <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
            Connect your repositories, integrate AI agents, and supercharge your microservices development workflow.
          </p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature, index) => (
            <Card key={index} className="bg-card border-border hover:shadow-card transition-all duration-300 group">
              <CardHeader>
                <div className="flex items-center justify-between mb-2">
                  <div className="w-12 h-12 bg-primary/10 rounded-lg flex items-center justify-center group-hover:bg-primary/20 transition-colors">
                    <feature.icon className="w-6 h-6 text-primary" />
                  </div>
                  <Badge variant="outline" className="text-xs">
                    {feature.badge}
                  </Badge>
                </div>
                <CardTitle className="text-lg font-semibold text-foreground">
                  {feature.title}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-muted-foreground leading-relaxed">
                  {feature.description}
                </p>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </section>
  );
};