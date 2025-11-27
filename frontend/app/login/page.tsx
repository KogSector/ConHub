"use client"
import { useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { LoginForm } from '@/components/auth/LoginForm'
import { isLoginEnabled } from '@/lib/feature-toggles'

export default function LoginPage() {
  const router = useRouter()

  useEffect(() => {
    if (!isLoginEnabled()) {
      router.replace('/dashboard')
    }
  }, [router])

  return <LoginForm />
}