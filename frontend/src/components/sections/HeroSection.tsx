import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Github, Zap, Shield, Code } from "lucide-react";
import heroImage from "@/assets/hero-image.jpg";

export const HeroSection = () => {
  const handleGetStarted = () => {
    // For now, show alert about Supabase requirement
    alert("GitHub authentication requires Supabase integration. Please connect Supabase first!");
  };

  const handleViewDocs = () => {
    window.location.href = "/docs";
  };

  return (
    <section className="min-h-screen flex items-center justify-center bg-gradient-hero relative overflow-hidden">
      {/* Background decoration */}
      <div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-transparent to-accent/5" />
      <div className="absolute top-1/4 -left-1/4 w-96 h-96 bg-primary/10 rounded-full blur-3xl" />
      <div className="absolute bottom-1/4 -right-1/4 w-96 h-96 bg-accent/10 rounded-full blur-3xl" />

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-20 relative z-10">
        <div className="grid lg:grid-cols-2 gap-12 items-center">
          {/* Left Column - Content */}
          <div className="space-y-8">
            <Badge variant="secondary" className="w-fit">
              <Zap className="w-3 h-3 mr-1" />
              Now in Beta
            </Badge>
            
            <div className="space-y-4">
              <h1 className="text-4xl md:text-6xl font-bold text-foreground leading-tight">
                Unify Your 
                <span className="bg-gradient-primary bg-clip-text text-transparent"> Repositories</span>
                {" "}with AI
              </h1>
              <p className="text-xl text-muted-foreground leading-relaxed max-w-lg">
                Connect multiple GitHub repositories and let AI agents access complete context across your entire microservices architecture. Code smarter, not harder.
              </p>
            </div>

            <div className="flex items-center gap-4 text-sm text-muted-foreground">
              <div className="flex items-center gap-1">
                <Shield className="w-4 h-4 text-accent" />
                <span>Secure by design</span>
              </div>
              <div className="flex items-center gap-1">
                <Code className="w-4 h-4 text-primary" />
                <span>Multi-repo context</span>
              </div>
              <div className="flex items-center gap-1">
                <Zap className="w-4 h-4 text-primary-glow" />
                <span>AI powered</span>
              </div>
            </div>

            <div className="flex flex-col sm:flex-row gap-4">
              <Button size="lg" className="bg-gradient-primary hover:shadow-primary transition-all duration-300" onClick={handleGetStarted}>
                <Github className="w-5 h-5 mr-2" />
                Get Started with GitHub
              </Button>
              <Button variant="outline" size="lg" onClick={handleViewDocs}>
                View Documentation
              </Button>
            </div>

            <div className="flex items-center gap-8 text-sm text-muted-foreground">
              <div>
                <div className="font-semibold text-foreground">1000+</div>
                <div>Repositories connected</div>
              </div>
              <div>
                <div className="font-semibold text-foreground">50+</div>
                <div>AI agents integrated</div>
              </div>
              <div>
                <div className="font-semibold text-foreground">99.9%</div>
                <div>Uptime guaranteed</div>
              </div>
            </div>
          </div>

          {/* Right Column - Hero Image */}
          <div className="relative">
            <div className="relative rounded-2xl overflow-hidden shadow-glow">
              <img
                src={heroImage}
                alt="GitSync Pro Dashboard showing connected repositories and AI agents"
                className="w-full h-auto object-cover"
              />
              <div className="absolute inset-0 bg-gradient-to-t from-background/20 to-transparent" />
            </div>
            
            {/* Floating elements */}
            <div className="absolute -top-4 -right-4 bg-card border border-border rounded-lg p-3 shadow-card">
              <div className="flex items-center gap-2 text-sm">
                <div className="w-2 h-2 bg-accent rounded-full animate-pulse" />
                <span className="text-muted-foreground">5 AI agents connected</span>
              </div>
            </div>
            
            <div className="absolute -bottom-4 -left-4 bg-card border border-border rounded-lg p-3 shadow-card">
              <div className="flex items-center gap-2 text-sm">
                <Github className="w-4 h-4 text-primary" />
                <span className="text-muted-foreground">12 repos synced</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
