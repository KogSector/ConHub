'use client';

import { Auth0Provider } from '@auth0/auth0-react';
import React from 'react';

const Auth0ProviderWithNavigate = ({ children }: { children: React.ReactNode }) => {
  const redirectUri = typeof window !== 'undefined' ? window.location.origin : 'http://localhost:3000';
  const domain = process.env.NEXT_PUBLIC_AUTH0_DOMAIN;
  const clientId = process.env.NEXT_PUBLIC_AUTH0_CLIENT_ID;
  const audience = process.env.NEXT_PUBLIC_AUTH0_AUDIENCE;

  // Safety check - render children anyway to avoid blocking the app
  if (!(domain && clientId)) {
    if (process.env.NODE_ENV === 'development') {
      console.warn("⚠️ Auth0 Config Missing - running in unauthenticated mode");
    }
    return <>{children}</>;
  }

  return (
    <Auth0Provider
      domain={domain}
      clientId={clientId}
      authorizationParams={{
        redirect_uri: redirectUri, 
        audience: audience,
      }}
      cacheLocation="localstorage"
      useRefreshTokens={true}
    >
      {children}
    </Auth0Provider>
  );
};

export default Auth0ProviderWithNavigate;