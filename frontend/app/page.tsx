import { Navbar } from "@/components/ui/navbar"
import { HeroSection } from "@/components/sections/HeroSection"
import { LoadingSkeleton, lazyLoad } from "@/components/ui/lazy-loading"
import { Footer } from "@/components/ui/footer"
import { Suspense } from "react"

// Lazy load heavy sections
const FeaturesSection = lazyLoad(
  () => import("@/components/sections/FeaturesSection"),
  () => <LoadingSkeleton height="h-96" className="my-8" />
)

const DocsSection = lazyLoad(
  () => import("@/components/sections/DocsSection"),
  () => <LoadingSkeleton height="h-64" className="my-8" />
)

export default function Home() {
  return (
    <div className="min-h-screen bg-background">
      <Navbar />
      <HeroSection />
      
      {/* Lazy load non-critical sections */}
      <Suspense fallback={<LoadingSkeleton height="h-96" className="my-8" />}>
        <FeaturesSection />
      </Suspense>
      
      <Suspense fallback={<LoadingSkeleton height="h-64" className="my-8" />}>
        <DocsSection />
      </Suspense>
      
      <Footer />
    </div>
  )
}