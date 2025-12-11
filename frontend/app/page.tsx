"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { FeaturesSection } from "@/components/sections/FeaturesSection";
import { Footer } from "@/components/ui/footer";
import { HeroSection } from "@/components/sections/HeroSection";
import { Navbar } from "@/components/ui/navbar";
import { useAuth } from "@/hooks/use-auth";

export default function Home() {
  const { isAuthenticated, isLoading } = useAuth();
  const router = useRouter();

  useEffect(() => {
    // Redirect authenticated users to dashboard
    if (!isLoading && isAuthenticated) {
      router.replace("/dashboard");
    }
  }, [isAuthenticated, isLoading, router]);

  // Show loading spinner while checking auth or redirecting
  if (isLoading || isAuthenticated) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      <Navbar showUserMenu={false} />
      <HeroSection />
      <FeaturesSection />
      <Footer />
    </div>
  );
}
