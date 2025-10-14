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
    
    return (
      <div className={`${className} bg-primary/10 border border-primary/20 rounded-md flex items-center justify-center text-xs font-semibold text-primary`}>
        {company.charAt(0).toUpperCase()}
      </div>
    );
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