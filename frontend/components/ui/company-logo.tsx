"use client";

import Image from "next/image";
import { useState } from "react";

interface CompanyLogoProps {
  company: string;
  className?: string;
  size?: number;
}

export function CompanyLogo({ 
  company, 
  className = "w-5 h-5",
  size = 20 
}: CompanyLogoProps) {
  const [hasError, setHasError] = useState(false);
  const logoPath = `/logos/${company.toLowerCase().replace(/\s+/g, '-')}.png`;
  
  if (hasError) {
    return null; // Will show fallback icon in parent
  }
  
  return (
    <Image
      src={logoPath}
      alt={`${company} logo`}
      width={size}
      height={size}
      className={`object-contain ${className}`}
      onError={() => setHasError(true)}
    />
  );
}