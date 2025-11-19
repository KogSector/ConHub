'use client'

import { Button } from '@/components/ui/button'
import { Loader2 } from 'lucide-react'
import { useState } from 'react'

interface SocialLoginButtonsProps {
  mode: 'login' | 'signup'
  onSocialLogin: (provider: string) => Promise<void>
  disabled?: boolean
}

export function SocialLoginButtons({ mode, onSocialLogin, disabled }: SocialLoginButtonsProps) {
  const [loadingProvider, setLoadingProvider] = useState<string | null>(null)

  const handleSocialClick = async (provider: string) => {
    setLoadingProvider(provider)
    try {
      await onSocialLogin(provider)
    } finally {
      setLoadingProvider(null)
    }
  }

  const buttonText = mode === 'login' ? 'Continue with' : 'Continue with'

  return (
    <div className="space-y-3">
      {/* Google Button - White with colorful G */}
      <Button
        type="button"
        variant="outline"
        className="w-full h-11 bg-white hover:bg-gray-50 text-gray-700 border-gray-300 font-medium"
        onClick={() => handleSocialClick('google')}
        disabled={disabled || loadingProvider !== null}
      >
        {loadingProvider === 'google' ? (
          <Loader2 className="mr-2 h-5 w-5 animate-spin" />
        ) : (
          <svg className="mr-2 h-5 w-5" viewBox="0 0 24 24">
            <path
              fill="#4285F4"
              d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
            />
            <path
              fill="#34A853"
              d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
            />
            <path
              fill="#FBBC05"
              d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
            />
            <path
              fill="#EA4335"
              d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
            />
          </svg>
        )}
        {buttonText} Google
      </Button>

      {/* GitHub Button - Black */}
      <Button
        type="button"
        variant="outline"
        className="w-full h-11 bg-[#24292e] hover:bg-[#1b1f23] text-white border-[#24292e] font-medium"
        onClick={() => handleSocialClick('github')}
        disabled={disabled || loadingProvider !== null}
      >
        {loadingProvider === 'github' ? (
          <Loader2 className="mr-2 h-5 w-5 animate-spin" />
        ) : (
          <svg className="mr-2 h-5 w-5 fill-current" viewBox="0 0 24 24">
            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
          </svg>
        )}
        {buttonText} GitHub
      </Button>

      {/* Bitbucket Button - Blue */}
      <Button
        type="button"
        variant="outline"
        className="w-full h-11 bg-[#0052CC] hover:bg-[#0747A6] text-white border-[#0052CC] font-medium"
        onClick={() => handleSocialClick('bitbucket')}
        disabled={disabled || loadingProvider !== null}
      >
        {loadingProvider === 'bitbucket' ? (
          <Loader2 className="mr-2 h-5 w-5 animate-spin" />
        ) : (
          <svg className="mr-2 h-5 w-5 fill-current" viewBox="0 0 24 24">
            <path d="M.778 1.211c-.424 0-.772.347-.772.772 0 .078.012.155.037.229l3.909 15.683c.094.387.439.66.84.66h15.118c.313 0 .591-.207.679-.506l3.905-15.683c.025-.074.037-.151.037-.229 0-.425-.348-.772-.772-.772H.778zm14.493 15.683h-6.266l-1.655-7.183h9.576l-1.655 7.183z" />
          </svg>
        )}
        {buttonText} Bitbucket
      </Button>
    </div>
  )
}
