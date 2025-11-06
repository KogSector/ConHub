"use client"
import { useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { ForgotPasswordForm } from '@/components/auth/ForgotPasswordForm'
import { isLoginEnabled } from '@/lib/feature-toggles'

export default function ForgotPasswordPage() {
  const router = useRouter()

  useEffect(() => {
    if (!isLoginEnabled()) {
      router.replace('/dashboard')
    }
  }, [router])

  return <ForgotPasswordForm />
}