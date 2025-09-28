import { Navbar } from "@/components/ui/navbar"
import { HeroSection } from "@/components/sections/HeroSection"
import { Footer } from "@/components/ui/footer"

export default function Home() {
  return (
    <div className="min-h-screen bg-background">
      <Navbar />
      <HeroSection />
      
      {/* Lightweight features section */}
      <section className="py-16 px-4">
        <div className="max-w-6xl mx-auto text-center">
          <h2 className="text-3xl font-bold mb-8 bg-gradient-to-r from-primary to-accent bg-clip-text text-transparent">
            ConHub Features
          </h2>
          <div className="grid md:grid-cols-3 gap-8">
            <div className="p-6 rounded-lg border border-border bg-card">
              <h3 className="text-xl font-semibold mb-3">Repository Management</h3>
              <p className="text-muted-foreground">Connect and manage your repositories with ease.</p>
            </div>
            <div className="p-6 rounded-lg border border-border bg-card">
              <h3 className="text-xl font-semibold mb-3">AI Integration</h3>
              <p className="text-muted-foreground">Supercharge your workflow with AI assistance.</p>
            </div>
            <div className="p-6 rounded-lg border border-border bg-card">
              <h3 className="text-xl font-semibold mb-3">Social Connections</h3>
              <p className="text-muted-foreground">Connect with your development community.</p>
            </div>
          </div>
        </div>
      </section>
      
      <Footer />
    </div>
  )
}