"use client"
import { Suspense, useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { ResetPasswordForm } from '@/components/auth/ResetPasswordForm'
import { isLoginEnabled } from '@/lib/feature-toggles'

export default function ResetPasswordPage() {
  const router = useRouter()

  useEffect(() => {
    if (!isLoginEnabled()) {
      router.replace('/dashboard')
    }
  }, [router])

  return (
    <Suspense fallback={
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-indigo-900 to-slate-900">
        <div className="w-8 h-8 border-2 border-indigo-400/30 border-t-indigo-400 rounded-full animate-spin"></div>
      </div>
    }>
      <ResetPasswordForm />
    </Suspense>
  )
}