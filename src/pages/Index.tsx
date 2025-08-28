import { Navbar } from "@/components/ui/navbar";
import { HeroSection } from "@/components/sections/HeroSection";
import { FeaturesSection } from "@/components/sections/FeaturesSection";
import { DocsSection } from "@/components/sections/DocsSection";
import { Footer } from "@/components/ui/footer";

const Index = () => {
  return (
    <div className="min-h-screen bg-background">
      <Navbar />
      <HeroSection />
      <FeaturesSection />
      <DocsSection />
      <Footer />
    </div>
  );
};

export default Index;
