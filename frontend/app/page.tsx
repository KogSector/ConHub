"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useAuth0 } from "@auth0/auth0-react";
import { isLoginEnabled } from "@/lib/feature-toggles";
import { FeaturesSection } from "@/components/sections/FeaturesSection";
import { Footer } from "@/components/ui/footer";
import { HeroSection } from "@/components/sections/HeroSection";
import { Navbar } from "@/components/ui/navbar";

export default function Home() {
  // 👇 Get the ERROR object too
  const { isAuthenticated, isLoading, user, error } = useAuth0();
  const router = useRouter();
  const loginEnabled = isLoginEnabled();

  // 👇 DEBUG: Print status to console
  console.log("AUTH DEBUG:", { isLoading, isAuthenticated, error, user });

  useEffect(() => {
    const syncAndRedirect = async () => {
      if (!isLoading && isAuthenticated && loginEnabled && user) {
        try {
          const authServiceUrl = process.env.NEXT_PUBLIC_AUTH_SERVICE_URL || 'http://localhost:3010';
          await fetch(`${authServiceUrl}/api/auth/auth0/exchange`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              auth0_id: user.sub,
              email: user.email,
              name: user.name,
              avatar_url: user.picture
            })
          });
        } catch (e) {
          console.error("Sync failed", e);
        }
        router.push('/dashboard');
      }
    };
    syncAndRedirect();
  }, [isAuthenticated, isLoading, loginEnabled, router, user]);

  // 🔴 1. IF THERE IS AN ERROR, SHOW IT (Don't spin forever)
  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-screen text-red-600 gap-4">
        <h2 className="text-2xl font-bold">Auth0 Error</h2>
        <p>{error.message}</p>
        <p className="text-sm text-gray-500">Check console for details</p>
      </div>
    );
  }

  // 🟡 2. SHOW SPINNER ONLY IF NO ERROR
  if (isLoading) {
     return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  // 🟢 3. RENDER PAGE
  if (isAuthenticated && loginEnabled) {
    return null; 
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
