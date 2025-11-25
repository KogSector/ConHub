'use client';

import { Auth0Provider } from '@auth0/auth0-react';
import React from 'react';

const Auth0ProviderWithNavigate = ({ children }: { children: React.ReactNode }) => {
  const redirectUri = 'http://localhost:3000'; 
  const domain = process.env.NEXT_PUBLIC_AUTH0_DOMAIN;
  const clientId = process.env.NEXT_PUBLIC_AUTH0_CLIENT_ID;
  const audience = process.env.NEXT_PUBLIC_AUTH0_AUDIENCE;

  console.log("Auth0 Debug:", { domain, clientId, redirectUri });

  // Safety check
  if (!(domain && clientId)) {
    console.error("❌ Auth0 Config Missing! Check .env.local");
    return null;
  }

  return (
    <Auth0Provider
      domain={domain}
      clientId={clientId}
      authorizationParams={{
        redirect_uri: redirectUri, // This forces ALL buttons to use localhost:3000
        //audience: audience,
      }}
    >
      {children}
    </Auth0Provider>
  );
};

export default Auth0ProviderWithNavigate;