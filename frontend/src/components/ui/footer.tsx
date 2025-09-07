import { Button } from "@/components/ui/button";
import { Github, Twitter, Mail, Heart } from "lucide-react";

export const Footer = () => {
  return (
    <footer className="bg-card border-t border-border">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="grid md:grid-cols-4 gap-8">
          {/* Brand */}
          <div className="space-y-4">
            <div className="flex items-center space-x-2">
              <div className="w-8 h-8 bg-gradient-primary rounded-lg flex items-center justify-center">
                <Github className="w-5 h-5 text-primary-foreground" />
              </div>
              <span className="text-xl font-bold text-foreground">GitSync Pro</span>
            </div>
            <p className="text-muted-foreground max-w-xs">
              Unify your repositories with AI for better microservices development.
            </p>
            <div className="flex space-x-3">
              <Button variant="ghost" size="sm">
                <Github className="w-4 h-4" />
              </Button>
              <Button variant="ghost" size="sm">
                <Twitter className="w-4 h-4" />
              </Button>
              <Button variant="ghost" size="sm">
                <Mail className="w-4 h-4" />
              </Button>
            </div>
          </div>

          {/* Product */}
          <div className="space-y-4">
            <h3 className="font-semibold text-foreground">Product</h3>
            <div className="space-y-2 text-sm">
              <a href="#features" className="block text-muted-foreground hover:text-foreground transition-colors">
                Features
              </a>
              <a href="#docs" className="block text-muted-foreground hover:text-foreground transition-colors">
                Documentation
              </a>
              <a href="#pricing" className="block text-muted-foreground hover:text-foreground transition-colors">
                Pricing
              </a>
              <a href="#changelog" className="block text-muted-foreground hover:text-foreground transition-colors">
                Changelog
              </a>
            </div>
          </div>

          {/* Developers */}
          <div className="space-y-4">
            <h3 className="font-semibold text-foreground">Developers</h3>
            <div className="space-y-2 text-sm">
              <a href="#api" className="block text-muted-foreground hover:text-foreground transition-colors">
                API Reference
              </a>
              <a href="#sdk" className="block text-muted-foreground hover:text-foreground transition-colors">
                SDKs
              </a>
              <a href="#webhooks" className="block text-muted-foreground hover:text-foreground transition-colors">
                Webhooks
              </a>
              <a href="#examples" className="block text-muted-foreground hover:text-foreground transition-colors">
                Examples
              </a>
            </div>
          </div>

          {/* Company */}
          <div className="space-y-4">
            <h3 className="font-semibold text-foreground">Company</h3>
            <div className="space-y-2 text-sm">
              <a href="#about" className="block text-muted-foreground hover:text-foreground transition-colors">
                About
              </a>
              <a href="#blog" className="block text-muted-foreground hover:text-foreground transition-colors">
                Blog
              </a>
              <a href="#careers" className="block text-muted-foreground hover:text-foreground transition-colors">
                Careers
              </a>
              <a href="#contact" className="block text-muted-foreground hover:text-foreground transition-colors">
                Contact
              </a>
            </div>
          </div>
        </div>

        <div className="border-t border-border mt-8 pt-8 flex flex-col sm:flex-row justify-between items-center space-y-4 sm:space-y-0">
          <div className="text-sm text-muted-foreground">
            © 2024 GitSync Pro. All rights reserved.
          </div>
          <div className="flex items-center space-x-1 text-sm text-muted-foreground">
            <span>Built with</span>
            <Heart className="w-4 h-4 text-red-500 fill-current" />
            <span>for developers</span>
          </div>
        </div>
      </div>
    </footer>
  );
};