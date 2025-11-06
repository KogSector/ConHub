'use client';

import React from 'react';

interface BitbucketIconProps extends React.SVGProps<SVGSVGElement> {}

export function BitbucketIcon(props: BitbucketIconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden="true"
      {...props}
    >
      {/* Simplified Bitbucket-like mark used in LoginForm for consistency */}
      <path d="M2 3.5a.5.5 0 0 0-.5.5v.5L4 20.5l.5.5h15l.5-.5L22.5 4.5V4a.5.5 0 0 0-.5-.5H2zm13.5 10.5h-7L7 8h10l-1.5 6z" />
    </svg>
  );
}

export default BitbucketIcon;