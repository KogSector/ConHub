'use client'

import React from 'react'

export default function SlackIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" {...props}>
      <path fill="#36C5F0" d="M7.5 0A2.5 2.5 0 005 2.5V6h2.5A2.5 2.5 0 0010 3.5V1A1 1 0 009 0H7.5z"/>
      <path fill="#2EB67D" d="M0 7.5A2.5 2.5 0 012.5 5H6v2.5A2.5 2.5 0 013.5 10H1A1 1 0 010 9V7.5z"/>
      <path fill="#E01E5A" d="M16.5 24A2.5 2.5 0 0019 21.5V18h-2.5A2.5 2.5 0 0014 20.5V23a1 1 0 001 1h1.5z"/>
      <path fill="#ECB22E" d="M24 16.5A2.5 2.5 0 0021.5 19H18v-2.5A2.5 2.5 0 0020.5 14H23a1 1 0 011 1v1.5z"/>
    </svg>
  )
}