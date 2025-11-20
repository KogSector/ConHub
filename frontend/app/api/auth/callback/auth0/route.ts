import { NextRequest, NextResponse } from 'next/server'

export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams
  const code = searchParams.get('code')
  const error = searchParams.get('error')
  const errorDescription = searchParams.get('error_description')

  // Handle Auth0 errors
  if (error) {
    console.error('Auth0 error:', error, errorDescription)
    return NextResponse.redirect(
      new URL(`/auth/login?error=${encodeURIComponent(errorDescription || error)}`, request.url)
    )
  }

  // Validate authorization code
  if (!code) {
    return NextResponse.redirect(
      new URL('/auth/login?error=Missing authorization code', request.url)
    )
  }

  try {
    // Exchange authorization code for tokens
    const domain = process.env.NEXT_PUBLIC_AUTH0_DOMAIN
    const clientId = process.env.NEXT_PUBLIC_AUTH0_CLIENT_ID
    const clientSecret = process.env.AUTH0_CLIENT_SECRET
    const redirectUri = `${request.nextUrl.origin}/api/auth/callback/auth0`

    const tokenResponse = await fetch(`https://${domain}/oauth/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        grant_type: 'authorization_code',
        client_id: clientId,
        client_secret: clientSecret,
        code,
        redirect_uri: redirectUri,
      }),
    })

    if (!tokenResponse.ok) {
      const errorData = await tokenResponse.json()
      console.error('Token exchange failed:', errorData)
      throw new Error(errorData.error_description || 'Token exchange failed')
    }

    const tokens = await tokenResponse.json()
    const { access_token, id_token, refresh_token, expires_in } = tokens

    // Get user info from Auth0
    const userInfoResponse = await fetch(`https://${domain}/userinfo`, {
      headers: {
        Authorization: `Bearer ${access_token}`,
      },
    })

    if (!userInfoResponse.ok) {
      throw new Error('Failed to fetch user info')
    }

    const userInfo = await userInfoResponse.json()

    // Create or update user in backend
    const authServiceUrl = process.env.NEXT_PUBLIC_AUTH_SERVICE_URL || 'http://localhost:3010'
    const backendResponse = await fetch(`${authServiceUrl}/api/auth/oauth/callback`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        provider: 'auth0',
        provider_user_id: userInfo.sub,
        email: userInfo.email,
        name: userInfo.name || userInfo.email.split('@')[0],
        avatar_url: userInfo.picture,
        access_token,
        refresh_token,
        expires_at: Math.floor(Date.now() / 1000) + expires_in,
        scope: 'openid profile email',
      }),
    })

    if (!backendResponse.ok) {
      const errorData = await backendResponse.json()
      console.error('Backend OAuth callback failed:', errorData)
      throw new Error('Failed to create user session')
    }

    const backendData = await backendResponse.json()

    // Create session and redirect to dashboard
    const response = NextResponse.redirect(new URL('/dashboard', request.url))
    
    // Set session cookies
    const expiresAt = new Date(Date.now() + expires_in * 1000)
    response.cookies.set('conhub_session', JSON.stringify({
      token: access_token,
      refreshToken: refresh_token,
      expiresAt: expiresAt.toISOString(),
      user: {
        id: backendData.user_id,
        email: userInfo.email,
        name: userInfo.name,
        avatar_url: userInfo.picture,
      },
    }), {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      expires: expiresAt,
      path: '/',
    })

    return response
  } catch (error) {
    console.error('Auth0 callback error:', error)
    return NextResponse.redirect(
      new URL(
        `/auth/login?error=${encodeURIComponent(
          error instanceof Error ? error.message : 'Authentication failed'
        )}`,
        request.url
      )
    )
  }
}
