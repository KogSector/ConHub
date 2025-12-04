 'use client'

 import { Button } from "@/components/ui/button";
 import { Menu, X, LogOut, User } from "lucide-react";
 import { useState } from "react";
 import Link from "next/link";
 // 🆕 1. Import the router hook for navigation
 import { useRouter } from "next/navigation"; 

 import { useAuth0 } from "@auth0/auth0-react";

 type NavbarProps = {
   showUserMenu?: boolean;
 };

 export const Navbar = ({ showUserMenu = true }: NavbarProps) => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  
  // 🆕 2. Initialize the router
  const router = useRouter();

  // We keep loginWithRedirect here just in case, but we won't use it in handleLogin anymore
  const { isAuthenticated, isLoading, loginWithRedirect, logout, user } = useAuth0();

  const getInitial = (s?: string) => {
    const v = (s || '').trim()
    return v ? v.charAt(0).toUpperCase() : 'U'
  }

  const handleLogin = () => {
    // 🆕 3. CHANGED: Navigate to your custom login page instead of Auth0 default
    router.push("/auth/login");
  };

  const handleLogout = () => {
    logout({ logoutParams: { returnTo: window.location.origin } });
  };

  return (
    <nav className="fixed top-0 left-0 right-0 z-50 bg-background/80 backdrop-blur-sm border-b border-border">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center h-20">
          
          <Link href="/" className="flex items-center space-x-2">
            <span className="text-3xl md:text-4xl font-bold font-orbitron bg-gradient-to-r from-primary via-primary-glow to-accent bg-clip-text text-transparent">ConHub</span>
          </Link>

          {/* --- DESKTOP MENU --- */}
          <div className="hidden md:flex items-center space-x-8">
            <Link href="/#features" className="text-muted-foreground hover:text-foreground transition-colors">
              Features
            </Link>
            <Link href="/docs" className="text-muted-foreground hover:text-foreground transition-colors">
              Documentation
            </Link>
            <Link href="/pricing" className="text-muted-foreground hover:text-foreground transition-colors">
              Pricing
            </Link>

            {showUserMenu && (
              isLoading ? (
                <Button variant="outline" size="sm" disabled>
                  Loading...
                </Button>
              ) : isAuthenticated && user ? (
                // STATE A: LOGGED IN (Show Profile)
                <div className="flex items-center gap-4">
                  <span className="text-sm font-medium text-foreground/80">
                    {user.name || user.email}
                  </span>
                  <div className="flex items-center gap-2">
                    <div className="h-8 w-8 rounded-full border border-border bg-muted flex items-center justify-center">
                      <span className="text-sm font-semibold">{getInitial(user.name || user.email)}</span>
                    </div>
                    <Button variant="ghost" size="sm" onClick={handleLogout} title="Log Out">
                      <LogOut className="w-4 h-4 text-muted-foreground hover:text-destructive" />
                    </Button>
                  </div>
                </div>
              ) : null
            )}
          </div>

          {/* --- MOBILE TOGGLE --- */}
          <div className="md:hidden">
            <Button
              variant="ghost" 
              size="sm"
              onClick={() => setIsMenuOpen(!isMenuOpen)}
            >
              {isMenuOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
            </Button>
          </div>
        </div>

        {/* --- MOBILE MENU --- */}
        {isMenuOpen && (
          <div className="md:hidden py-4 border-t border-border">
            <div className="flex flex-col space-y-3">
              <Link href="/#features" className="text-muted-foreground hover:text-foreground transition-colors">
                Features
              </Link>
              <Link href="/docs" className="text-muted-foreground hover:text-foreground transition-colors">
                Documentation
              </Link>
              <Link href="/pricing" className="text-muted-foreground hover:text-foreground transition-colors">
                Pricing
              </Link>

              <div className="pt-2 border-t border-border/50">
                {showUserMenu ? (
                  isLoading ? (
                    <Button variant="outline" size="sm" className="w-full" disabled>
                      Loading...
                    </Button>
                  ) : isAuthenticated && user ? (
                    // MOBILE LOGGED IN STATE
                    <div className="flex flex-col gap-3">
                      <div className="flex items-center gap-3 px-2">
                        <div className="h-8 w-8 rounded-full border border-border bg-muted flex items-center justify-center">
                          <span className="text-sm font-semibold">{getInitial(user.name || user.email)}</span>
                        </div>
                        <span className="text-sm font-medium">{user.name || user.email}</span>
                      </div>
                      <Button variant="ghost" size="sm" className="w-full justify-start text-destructive" onClick={handleLogout}>
                        <LogOut className="w-4 h-4 mr-2" /> Log Out
                      </Button>
                    </div>
                  ) : (
                    // MOBILE LOGGED OUT STATE
                    <Button variant="outline" size="sm" className="w-full" onClick={handleLogin}>
                      Sign In
                    </Button>
                  )
                ) : null}
              </div>
            </div>
          </div>
        )}
      </div>
    </nav>
  );
};
