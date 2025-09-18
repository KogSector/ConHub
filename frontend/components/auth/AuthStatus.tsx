'use client'

import { useAuth } from "@/hooks/use-auth";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { isLoginEnabled } from "@/lib/feature-toggles";

export const AuthStatus = () => {
  const { user, isAuthenticated, isLoading, logout } = useAuth();
  const loginEnabled = isLoginEnabled();

  const handleLogin = () => {
    // Redirect to login page
    window.location.href = '/auth/login';
  };

  if (isLoading) {
    return (
      <Card className="w-full max-w-md mx-auto">
        <CardContent className="p-6">
          <div className="text-center">Loading authentication status...</div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="w-full max-w-md mx-auto">
      <CardHeader>
        <CardTitle>Authentication Status</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {!loginEnabled && (
          <div className="p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
            <p className="text-sm text-yellow-800">Development Mode: Authentication bypassed</p>
          </div>
        )}
        
        <div className="flex items-center justify-between">
          <span>Status:</span>
          <Badge variant={isAuthenticated ? "default" : "secondary"}>
            {isAuthenticated ? (loginEnabled ? "Authenticated" : "Dev Mode") : "Not Authenticated"}
          </Badge>
        </div>
        
        {isAuthenticated && user && (
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span>Name:</span>
              <span className="text-sm">{user.name || 'N/A'}</span>
            </div>
            <div className="flex items-center justify-between">
              <span>Email:</span>
              <span className="text-sm">{user.email || 'N/A'}</span>
            </div>
            {user.avatar_url && (
              <div className="flex items-center justify-between">
                <span>Avatar:</span>
                <img 
                  src={user.avatar_url} 
                  alt="User avatar" 
                  className="w-8 h-8 rounded-full"
                />
              </div>
            )}
            <div className="flex items-center justify-between">
              <span>Role:</span>
              <Badge variant="outline">{user.role}</Badge>
            </div>
            <div className="flex items-center justify-between">
              <span>Organization:</span>
              <span className="text-sm">{user.organization || 'N/A'}</span>
            </div>
          </div>
        )}
        
        <div className="pt-4">
          {loginEnabled ? (
            isAuthenticated ? (
              <Button onClick={logout} variant="outline" className="w-full">
                Sign Out
              </Button>
            ) : (
              <Button onClick={handleLogin} className="w-full">
                Sign In
              </Button>
            )
          ) : (
            <Button disabled className="w-full">
              Authentication Disabled
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
};
