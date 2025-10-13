'use client'

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Navbar } from "@/components/ui/navbar";
import { Footer } from "@/components/ui/footer";
import { useAuth } from "@/hooks/use-auth";
import Link from "next/link";
import { 
  BookOpen, 
  Bot, 
  Shield, 
  Play,
  FileText,
  Code,
  Users,
  GitBranch,
  Target,
  ArrowRight
} from "lucide-react";

export default function Documentation() {
  const { isAuthenticated, isLoading } = useAuth();

  const handleDashboardClick = () => {
    if (isAuthenticated) {
      window.location.href = '/dashboard';
    } else {
      
      
      window.location.href = '/';
    }
  };

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
      icon: GitBranch,
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
      
      <main className="pt-20">
        {}
        <section className="py-24 bg-background">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="text-center space-y-6">
              <Badge variant="secondary" className="w-fit mx-auto">
                <BookOpen className="w-3 h-3 mr-1" />
                Documentation
              </Badge>
              <h1 className="text-4xl md:text-5xl font-bold text-foreground">
                Complete 
                <span className="bg-gradient-to-r from-primary to-primary-glow bg-clip-text text-transparent"> Documentation</span>
              </h1>
              <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
                Everything you need to master ConHub, from basic setup to advanced integrations.
              </p>
              
              {}
              <div className="flex justify-center mt-6">
                <Button 
                  onClick={handleDashboardClick}
                  disabled={isLoading}
                  className="bg-primary hover:bg-primary/90 text-primary-foreground"
                >
                  {isLoading ? (
                    "Loading..."
                  ) : isAuthenticated ? (
                    <>
                      Go to Dashboard
                      <ArrowRight className="w-4 h-4 ml-2" />
                    </>
                  ) : (
                    <>
                      Get Started
                      <ArrowRight className="w-4 h-4 ml-2" />
                    </>
                  )}
                </Button>
              </div>
              
              {}
              <div className="max-w-lg mx-auto mt-8">
                <div className="relative">
                  <input
                    type="text"
                    placeholder="Search documentation..."
                    className="w-full px-4 py-3 bg-card border border-border rounded-lg text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                  />
                  <Button size="sm" className="absolute right-2 top-1/2 -translate-y-1/2">
                    Search
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </section>

        {}
        <section className="py-16 -mt-12">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <Card className="bg-card border-border shadow-card">
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
                    <GitBranch className="w-6 h-6 text-primary" />
                    <span>Connect Repository</span>
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

        {}
        <section className="py-16">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex flex-col gap-6">
              {sections.map((section, index) => (
                <Card key={index} className="bg-card border-border hover:shadow-card transition-all duration-300 group relative">
                  <div className="flex flex-col px-6 py-4 gap-4 overflow-hidden">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <div className="w-12 h-12 bg-primary/10 rounded-lg flex items-center justify-center group-hover:bg-primary/20 transition-colors">
                          <section.icon className="w-6 h-6 text-primary" />
                        </div>
                        <CardTitle className="text-lg font-semibold text-foreground group-hover:text-primary transition-colors">
                          {section.title}
                        </CardTitle>
                      </div>
                      <div className="flex items-center gap-2">
                        <div className="w-2 h-2 rounded-full bg-green-500 shadow-lg shadow-green-500/50"></div>
                        <Badge variant="outline" className="text-xs">
                          {section.badge}
                        </Badge>
                      </div>
                    </div>
                    <ul className="space-y-2">
                      {section.articles.map((article, i) => (
                        <li key={i}>
                          <a href="#" className="text-sm text-muted-foreground hover:text-foreground transition-colors block py-1">
                            {article}
                          </a>
                        </li>
                      ))}
                    </ul>
                    <div className="relative px-4 py-2 -mx-4 -my-2">
                      <button 
                        className="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium w-full h-10 px-4 py-2 text-white border border-orange-500/30 bg-transparent hover:text-white hover:bg-orange-500/10 hover:border-orange-500/50 hover:shadow-[0px_0px_12px_rgba(251,146,60,0.6)] hover:scale-105 transition-all duration-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-orange-500 focus-visible:ring-offset-2"
                      >
                        View All Articles
                      </button>
                    </div>
                  </div>
                </Card>
              ))}
            </div>
          </div>
        </section>

        {}
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
                  title: "Setting up Repository OAuth",
                  description: "Step-by-step guide to connecting your repositories securely",
                  icon: GitBranch,
                  readTime: "5 min read"
                },
                {
                  title: "Configuring Amazon Q",
                  description: "Complete setup guide for Amazon Q integration with ConHub",
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
                  description: "Working with ConHub APIs and authentication tokens",
                  icon: Code,
                  readTime: "6 min read"
                },
                {
                  title: "Context Optimization",
                  description: "Optimizing AI context sharing for better results",
                  icon: Target,
                  readTime: "7 min read"
                },
                {
                  title: "Team Setup Guide",
                  description: "Setting up ConHub for your development team",
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
