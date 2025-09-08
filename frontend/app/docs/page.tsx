import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Navbar } from "@/components/ui/navbar";
import { Footer } from "@/components/ui/footer";
import { 
  BookOpen, 
  Github, 
  Bot, 
  Shield, 
  Zap,
  Play,
  FileText,
  Code,
  Users
} from "lucide-react";

export default function Documentation() {
  const sections = [
    {
      title: "Getting Started",
      icon: Play,
      badge: "Beginner",
      articles: [
        "Quick Setup Guide",
        "Creating Your Account", 
        "Connecting First Repository",
        "Adding Your First AI Agent",
        "Understanding the Dashboard"
      ]
    },
    {
      title: "Repository Management",
      icon: Github,
      badge: "Core",
      articles: [
        "Connecting Public Repositories",
        "Private Repository Access",
        "Organization Permissions",
        "Branch Management",
        "Webhook Configuration"
      ]
    },
    {
      title: "AI Agent Integration",
      icon: Bot,
      badge: "Advanced",
      articles: [
        "GitHub Copilot Setup",
        "Amazon Q Integration",
        "Cline Configuration",
        "Custom Agent Development",
        "Context Optimization"
      ]
    },
    {
      title: "Security & Privacy",
      icon: Shield,
      badge: "Security",
      articles: [
        "Data Encryption",
        "Access Control Policies",
        "Audit Logging",
        "Compliance Standards",
        "Privacy Settings"
      ]
    },
    {
      title: "API Reference",
      icon: Code,
      badge: "Developer",
      articles: [
        "Authentication",
        "Repository Endpoints",
        "Agent Management API",
        "Webhook Events",
        "SDK Documentation"
      ]
    },
    {
      title: "Team Management",
      icon: Users,
      badge: "Enterprise",
      articles: [
        "User Roles & Permissions",
        "Team Organization",
        "SSO Integration",
        "Billing Management",
        "Usage Analytics"
      ]
    }
  ];

  return (
    <div className="min-h-screen bg-background">
      <Navbar />
      
      <main className="pt-16">
        {/* Hero Section */}
        <section className="py-24 bg-gradient-hero">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="text-center space-y-6">
              <Badge variant="secondary" className="w-fit mx-auto">
                <BookOpen className="w-3 h-3 mr-1" />
                Documentation
              </Badge>
              <h1 className="text-4xl md:text-5xl font-bold text-foreground">
                Complete 
                <span className="bg-gradient-primary bg-clip-text text-transparent"> Documentation</span>
              </h1>
              <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
                Everything you need to master GitSync Pro, from basic setup to advanced integrations.
              </p>
              
              {/* Search Bar */}
              <div className="max-w-lg mx-auto mt-8">
                <div className="relative">
                  <input
                    type="text"
                    placeholder="Search documentation..."
                    className="w-full px-4 py-3 bg-card border border-border rounded-lg text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                  />
                  <Button size="sm" className="absolute right-2 top-2">
                    Search
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Quick Links */}
        <section className="py-16 -mt-12">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <Card className="bg-gradient-card border-border shadow-card">
              <CardHeader>
                <CardTitle className="text-center text-xl font-semibold text-foreground">
                  Quick Start
                </CardTitle>
              </CardHeader>
              <CardContent>
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
              </CardContent>
            </Card>
          </div>
        </section>

        {/* Documentation Sections */}
        <section className="py-16">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
              {sections.map((section, index) => (
                <Card key={index} className="bg-card border-border hover:shadow-card transition-all duration-300 group">
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
                    <ul className="space-y-2">
                      {section.articles.map((article, i) => (
                        <li key={i}>
                          <a href="#" className="text-sm text-muted-foreground hover:text-foreground transition-colors block py-1">
                            {article}
                          </a>
                        </li>
                      ))}
                    </ul>
                    <Button variant="ghost" size="sm" className="w-full group-hover:bg-primary/10 transition-colors">
                      View All Articles
                    </Button>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        </section>

        {/* Popular Articles */}
        <section className="py-16 bg-muted/20">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="text-center mb-12">
              <h2 className="text-3xl font-bold text-foreground mb-4">
                Popular Articles
              </h2>
              <p className="text-muted-foreground">
                The most helpful guides from our documentation
              </p>
            </div>
            
            <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
              {[
                {
                  title: "Setting up GitHub OAuth",
                  description: "Step-by-step guide to connecting your GitHub account securely",
                  icon: Github,
                  readTime: "5 min read"
                },
                {
                  title: "Configuring Amazon Q",
                  description: "Complete setup guide for Amazon Q integration with GitSync Pro",
                  icon: Bot,
                  readTime: "8 min read"
                },
                {
                  title: "Security Best Practices",
                  description: "How to keep your repositories and data secure",
                  icon: Shield,
                  readTime: "10 min read"
                },
                {
                  title: "API Authentication",
                  description: "Working with GitSync Pro APIs and authentication tokens",
                  icon: Code,
                  readTime: "6 min read"
                },
                {
                  title: "Context Optimization",
                  description: "Optimizing AI context sharing for better results",
                  icon: Zap,
                  readTime: "7 min read"
                },
                {
                  title: "Team Setup Guide",
                  description: "Setting up GitSync Pro for your development team",
                  icon: Users,
                  readTime: "12 min read"
                }
              ].map((article, index) => (
                <Card key={index} className="bg-card border-border hover:shadow-card transition-all duration-300 cursor-pointer group">
                  <CardContent className="p-6 space-y-4">
                    <div className="flex items-center space-x-3">
                      <div className="w-10 h-10 bg-primary/10 rounded-lg flex items-center justify-center group-hover:bg-primary/20 transition-colors">
                        <article.icon className="w-5 h-5 text-primary" />
                      </div>
                      <Badge variant="secondary" className="text-xs">
                        {article.readTime}
                      </Badge>
                    </div>
                    <div>
                      <h3 className="font-semibold text-foreground group-hover:text-primary transition-colors mb-2">
                        {article.title}
                      </h3>
                      <p className="text-sm text-muted-foreground">
                        {article.description}
                      </p>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        </section>
      </main>
      
      <Footer />
    </div>
  );
}