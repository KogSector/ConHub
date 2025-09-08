'use client'

import { useAuth } from "@/hooks/use-auth";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

export const AuthStatus = () => {
  const { user, isAuthenticated, isLoading, login, logout } = useAuth();

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
        <div className="flex items-center justify-between">
          <span>Status:</span>
          <Badge variant={isAuthenticated ? "default" : "secondary"}>
            {isAuthenticated ? "Authenticated" : "Not Authenticated"}
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
            {user.picture && (
              <div className="flex items-center justify-between">
                <span>Avatar:</span>
                <img 
                  src={user.picture} 
                  alt="User avatar" 
                  className="w-8 h-8 rounded-full"
                />
              </div>
            )}
          </div>
        )}
        
        <div className="pt-4">
          {isAuthenticated ? (
            <Button onClick={logout} variant="outline" className="w-full">
              Sign Out
            </Button>
          ) : (
            <Button onClick={login} className="w-full">
              Sign In with Auth0
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
};
