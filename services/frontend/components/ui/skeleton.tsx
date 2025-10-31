import { cn } from "@/lib/utils"
import { isHeavyModeEnabled } from "@/lib/feature-toggles";

function Skeleton({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  const heavyMode = isHeavyModeEnabled();

  return (
    <div
      className={cn(
        "rounded-md",
        heavyMode 
          ? "animate-shine bg-[linear-gradient(110deg,transparent_25%,hsl(var(--primary-glow))_50%,transparent_75%)] bg-[length:200%_100%]" 
          : "animate-pulse bg-muted",
        className
      )}
      {...props}
    />
  );
}

export { Skeleton }
