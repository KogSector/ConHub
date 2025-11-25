'use client'

import React, { useState } from 'react'

import { Eye, EyeOff, Mail, Lock, ArrowRight, Sparkles } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import Link from 'next/link'
import { useAuth0 } from '@auth0/auth0-react'
import auth0 from 'auth0-js'



// ----------------------------------------------------------------------
// BRAND LOGOS (Permanent - Do Not Delete)
// These SVGs are hardcoded to ensure you get the exact brand colors.
// ----------------------------------------------------------------------

const GoogleLogo = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
    <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" fill="#4285F4" />
    <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853" />
    <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05" />
    <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335" />
  </svg>
)

const GithubLogo = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor">
    <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
  </svg>
)

const BitbucketLogo = ({ className }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor">
    <path d="M.778 1.213a.768.768 0 00-.768.892l3.182 19.805A1.56 1.56 0 004.73 23.18h14.538a1.56 1.56 0 001.54-1.27L23.99 2.105a.768.768 0 00-.767-.892H.778zM14.66 16.23H9.34L7.59 7.42h8.82l-1.75 8.81z" />
  </svg>
)

// --- Internal Component: SocialLoginButtons ---
interface SocialLoginButtonsProps {
  mode: 'login' | 'register'
  onSocialLogin: (provider: string) => void
  disabled?: boolean
}

function SocialLoginButtons({ mode, onSocialLogin, disabled }: SocialLoginButtonsProps) {
  return (
    <div className="flex flex-col gap-4">
      {/* Google Button - Top */}
      <Button
        type="button"
        onClick={() => onSocialLogin('google-oauth2')}
        disabled={disabled}
        className="w-full bg-white text-black hover:bg-gray-200 border-none h-11 text-base font-normal transition-transform transform hover:scale-[1.02] shadow-lg"
      >
        <GoogleLogo className="mr-3 h-6 w-6" />
        Continue with Google
      </Button>
      
      {/* GitHub Button - Middle - Black Theme */}
      <Button
        type="button"
        onClick={() => onSocialLogin('github')}
        disabled={disabled}
        className="w-full h-11 bg-[#24292F] text-white hover:bg-[#24292F]/90 border-none text-base font-normal transition-transform transform hover:scale-[1.02] shadow-lg"
      >
        <GithubLogo className="mr-3 h-6 w-6" />
        Continue with GitHub
      </Button>

      {/* Bitbucket Button - Bottom - Blue Theme */}
      <Button
        type="button"
        onClick={() => onSocialLogin('bitbucket')}
        disabled={disabled}
        className="w-full h-11 bg-[#0052CC] text-white hover:bg-[#0052CC]/90 border-none text-base font-normal transition-transform transform hover:scale-[1.02] shadow-lg "
      >
        <BitbucketLogo className="mr-3 h-6 w-6" />
        Continue with Bitbucket
      </Button>
    </div>
  )
}

export function LoginForm() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [error, setError] = useState('')
  const [isLoading, setIsLoading] = useState(false)

  // 👇 3. Get the login function from the React SDK
  const { loginWithRedirect } = useAuth0()

  // Helper for Email/Password Login (Custom UI)
  const getAuthClient = () => {
    return new auth0.WebAuth({
      domain: process.env.NEXT_PUBLIC_AUTH0_DOMAIN || '',
      clientID: process.env.NEXT_PUBLIC_AUTH0_CLIENT_ID || '',
      redirectUri: typeof window !== 'undefined' ? window.location.origin : '', 
      responseType: 'token id_token',
      scope: 'openid profile email'
    })
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setIsLoading(true);

    const webAuth = getAuthClient();

    // Custom Form Logic (Legacy Method)
    webAuth.login({
      realm: 'Username-Password-Authentication', 
      username: email,
      password: password,
    }, (err: any) => {
      setIsLoading(false);
      if (err) {
        console.error("Auth0 Login Error:", err);
        setError(err.description || 'Invalid email or password.');
      }
    });
  };

  const handleSocialLogin = async (provider: string) => {
    setError('')
    setIsLoading(true)
    
    try {
      // 👇 4. Use the React SDK for Social Login
      // This allows specific providers (like 'github') but keeps the Navbar synced.
      await loginWithRedirect({
        authorizationParams: {
          connection: provider, // 'github' or 'google-oauth2'
        }
      });
    } catch (err) {
      setIsLoading(false)
      setError(`Failed to sign in with ${provider}`)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 relative overflow-hidden">
      {/* Background Elements */}
      <div className="absolute inset-0 bg-grid-pattern opacity-5"></div>
      <div className="absolute top-0 left-1/4 w-96 h-96 bg-purple-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-blob"></div>
      <div className="absolute top-0 right-1/4 w-96 h-96 bg-blue-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-blob animation-delay-2000"></div>
      <div className="absolute bottom-8 left-1/3 w-96 h-96 bg-pink-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-blob animation-delay-4000"></div>
      
      <div className="relative z-10 w-full max-w-md mx-4">
        {/* Logo */}
        <div className="text-center mb-8 mt-10">
          <Link href="/" className="inline-flex items-center space-x-2 group">
            <Sparkles className="w-8 h-8 text-purple-400 group-hover:text-purple-300 transition-colors" />
            <span className="text-4xl font-bold font-orbitron bg-gradient-to-r from-purple-400 via-pink-400 to-blue-400 bg-clip-text text-transparent">
              ConHub
            </span>
          </Link>
        </div>

        <Card className="backdrop-blur-xl bg-white/5 border border-white/10 shadow-2xl">
          <CardHeader className="space-y-1 text-center pb-6">
            <CardTitle className="text-3xl font-bold text-white">Welcome back</CardTitle>
            <CardDescription className="text-gray-300">
              Sign in to continue your journey
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <form onSubmit={handleSubmit} className="space-y-6">
              {error && (
                <div className="p-4 text-sm text-red-300 bg-red-500/10 border border-red-500/20 rounded-lg backdrop-blur-sm">
                  {error}
                </div>
              )}
              
              <div className="space-y-2">
                <Label htmlFor="email" className="text-gray-200 font-medium">Email Address</Label>
                <div className="relative group">
                  <Mail className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-5 w-5 group-focus-within:text-purple-400 transition-colors" />
                  <Input
                    id="email"
                    type="email"
                    placeholder="Enter your email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    className="pl-11 bg-white/5 border-white/10 text-white placeholder:text-gray-400 focus:border-purple-400 focus:ring-purple-400/20 h-12"
                    required
                  />
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="password" className="text-gray-200 font-medium">Password</Label>
                <div className="relative group">
                  <Lock className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-5 w-5 group-focus-within:text-purple-400 transition-colors" />
                  <Input
                    id="password"
                    type={showPassword ? 'text' : 'password'}
                    placeholder="Enter your password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="pl-11 pr-12 bg-white/5 border-white/10 text-white placeholder:text-gray-400 focus:border-purple-400 focus:ring-purple-400/20 h-12"
                    required
                  />
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="absolute right-0 top-0 h-full px-3 text-gray-400 hover:text-purple-400 hover:bg-transparent"
                    onClick={() => setShowPassword(!showPassword)}
                  >
                    {showPassword ? (
                      <EyeOff className="h-5 w-5" />
                    ) : (
                      <Eye className="h-5 w-5" />
                    )}
                  </Button>
                </div>
              </div>

              {/* Main Submit Button (Standard height for form balance) */}
              <Button
                type="submit"
                className="w-full h-12 bg-gradient-to-r from-purple-500 to-pink-500 hover:from-purple-600 hover:to-pink-600 text-white font-semibold rounded-lg transition-all duration-300 transform hover:scale-[1.02] shadow-lg hover:shadow-purple-500/25 group mt-8"
                disabled={isLoading}
              >
                {isLoading ? (
                  <div className="flex items-center space-x-2">
                    <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                    <span>Signing in...</span>
                  </div>
                ) : (
                  <div className="flex items-center justify-center space-x-2">
                    <span>Sign in</span>
                    <ArrowRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
                  </div>
                )}
              </Button>

              <div className="text-center">
                <p className="text-gray-300">
                  Don&apos;t have an account?{' '}
                  <Link 
                    href="/auth/register" 
                    className="font-semibold text-purple-400 hover:text-purple-300 transition-colors hover:underline"
                  >
                    Sign up
                  </Link>
                </p>
                <p className="text-gray-400 text-sm mt-2">
                  <Link 
                    href="/auth/forgot-password" 
                    className="hover:text-purple-400 transition-colors hover:underline"
                  >
                    Forgot your password?
                  </Link>
                </p>
              </div>
            </form>

            {/* Social Login Section */}
            <div className="relative my-6">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-white/10"></div>
              </div>
              <div className="relative flex justify-center text-sm">
                <span className="px-4 bg-slate-900/50 text-gray-400">Or</span>
              </div>
            </div>

            <SocialLoginButtons 
              mode="login" 
              onSocialLogin={handleSocialLogin}
              disabled={isLoading}
            />
          </CardContent>
        </Card>

        <div className="text-center mt-8">
          <p className="text-gray-400 text-sm">
            Secure authentication powered by ConHub
          </p>
        </div>
      </div>
    </div>
  )
}

// Add default export for the preview environment
export default LoginForm