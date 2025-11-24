import React from 'react'

export default function JiraIcon({ className = 'w-6 h-6' }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" className={className} xmlns="http://www.w3.org/2000/svg" fill="none">
      <path d="M2 11.999c0-5.523 4.477-10 10-10 2.651 0 5.07 1.03 6.879 2.711l-4.243 4.243a6 6 0 10-.001 8.487l4.244 4.244A9.953 9.953 0 0112 22c-5.523 0-10-4.477-10-10z" fill="#2684FF"/>
      <path d="M14.828 9.172L12 12l2.828 2.828a4 4 0 10-5.657 0l2.829-2.828-2.829-2.828a4 4 0 105.657 0z" fill="#0052CC"/>
    </svg>
  )
}