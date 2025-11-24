import React from 'react'

export default function CustomAppsIcon({ className = 'w-6 h-6' }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" className={className} xmlns="http://www.w3.org/2000/svg" fill="none">
      <rect x="3" y="3" width="7" height="7" rx="2" fill="#6EE7B7"/>
      <rect x="14" y="3" width="7" height="7" rx="2" fill="#93C5FD"/>
      <rect x="3" y="14" width="7" height="7" rx="2" fill="#FBBF24"/>
      <rect x="14" y="14" width="7" height="7" rx="2" fill="#F472B6"/>
    </svg>
  )
}