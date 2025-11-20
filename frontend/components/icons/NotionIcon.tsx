'use client'

import React from 'react'

export default function NotionIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" {...props}>
      <rect x="3" y="3" width="18" height="18" rx="3" fill="#000" />
      <path fill="#fff" d="M8 7h2.6l3.6 7.8V7h1.8v10H13.4L9.8 9.2V17H8V7z"/>
    </svg>
  )
}