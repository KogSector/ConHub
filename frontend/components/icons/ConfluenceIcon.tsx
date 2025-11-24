import React from 'react'

export default function ConfluenceIcon({ className = 'w-6 h-6' }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" className={className} xmlns="http://www.w3.org/2000/svg" fill="none">
      <path d="M3.5 6.5c0-.828.672-1.5 1.5-1.5h6.25c2.9 0 5.25 2.35 5.25 5.25 0 1.794-.9 3.387-2.27 4.35l-3.23 2.29a1.5 1.5 0 01-2.17-.45L6.49 9.78a1.5 1.5 0 01.45-2.17L9.5 6.5H5c-.828 0-1.5-.672-1.5-1.5z" fill="#0052CC"/>
      <path d="M20.5 17.5c0 .828-.672 1.5-1.5 1.5h-6.25a5.25 5.25 0 01-5.25-5.25c0-1.794.9-3.387 2.27-4.35l3.23-2.29a1.5 1.5 0 012.17.45l2.12 4.96a1.5 1.5 0 01-.45 2.17L14.5 17.5H19c.828 0 1.5.672 1.5 1.5z" fill="#2684FF"/>
    </svg>
  )
}