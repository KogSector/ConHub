export default function Loading() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-background">
      <div className="space-y-4 text-center">
        <div className="w-8 h-8 border-2 border-primary/30 border-t-primary rounded-full animate-spin mx-auto" />
        <p className="text-muted-foreground">Loading ConHub...</p>
        <div className="text-xs text-muted-foreground/60">
          Initializing services and dependencies
        </div>
      </div>
    </div>
  )
}