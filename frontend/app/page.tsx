"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/contexts/auth-context";
import { isLoginEnabled } from "@/lib/feature-toggles";
import { FeaturesSection } from "@/components/sections/FeaturesSection";
import { Footer } from "@/components/ui/footer"
import { HeroSection } from "@/components/sections/HeroSection"
import { Navbar } from "@/components/ui/navbar"

export default function Home() {
  const { isAuthenticated, isLoading } = useAuth();
  const router = useRouter();
  const loginEnabled = isLoginEnabled();

  useEffect(() => {
    // Only redirect authenticated users to dashboard when login is enabled
    if (!isLoading && isAuthenticated && loginEnabled) {
      router.push('/dashboard');
    }
  }, [isAuthenticated, isLoading, loginEnabled, router]);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  // Avoid rendering the landing page only when login is enabled and user is authenticated
  if (isAuthenticated && loginEnabled) {
    return null; 
  }

  return (
    <div className="min-h-screen bg-background">
      <Navbar />
      <HeroSection />
      <FeaturesSection />
      <Footer />
    </div>
  );
}