import { FeaturesSection } from "@/components/sections/FeaturesSection";
import { Footer } from "@/components/ui/footer"
import { HeroSection } from "@/components/sections/HeroSection"
import { Navbar } from "@/components/ui/navbar"

export default function Home() {
  return (
    <div className="min-h-screen bg-background">
      <Navbar />
      <HeroSection />
      <FeaturesSection />
      <Footer />
    </div>
  );
}