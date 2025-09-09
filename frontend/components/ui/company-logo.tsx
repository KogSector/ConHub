"use client";

import Image from "next/image";
import { useState } from "react";
import { 
  Github, 
  FileText, 
  HardDrive, 
  Globe 
} from "lucide-react";

interface CompanyLogoProps {
  company: string;
  className?: string;
  size?: number;
  fallbackIconName?: string;
}

const iconMap = {
  "github": Github,
  "bitbucket": Github,
  "google-drive": HardDrive,
  "dropbox": HardDrive,
  "confluence": FileText,
  "notion": FileText,
  "web-crawler": Globe
};

export function CompanyLogo({ 
  company, 
  className = "w-5 h-5",
  size = 20,
  fallbackIconName
}: CompanyLogoProps) {
  const [hasError, setHasError] = useState(false);
  const logoPath = `/logos/${company.toLowerCase().replace(/\s+/g, '-')}.png`;
  
  if (hasError && fallbackIconName) {
    const FallbackIcon = iconMap[fallbackIconName as keyof typeof iconMap];
    if (FallbackIcon) {
      return <FallbackIcon className={`${className} text-foreground`} />;
    }
  }
  
  if (hasError) {
    return null; // No fallback icon provided
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