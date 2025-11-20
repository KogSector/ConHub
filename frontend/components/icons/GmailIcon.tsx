'use client'

import React from 'react'

export default function GmailIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" {...props}>
      <path fill="#EA4335" d="M2 6.5l10 7 10-7V6a2 2 0 00-2-2H4a2 2 0 00-2 2v.5z"/>
      <path fill="#34A853" d="M2 7.5V19a2 2 0 002 2h3V10L2 7.5z"/>
      <path fill="#FBBC05" d="M22 7.5V19a2 2 0 01-2 2h-3V10l5-2.5z"/>
      <path fill="#4285F4" d="M7 21h10V10l-5 3.5L7 10v11z"/>
    </svg>
  )
}