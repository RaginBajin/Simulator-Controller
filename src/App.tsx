import { Button } from "@/components/ui/button"

function App() {
  return (
    <main className="flex min-h-screen items-center justify-center bg-base">
      <div className="text-center space-y-4">
        <h1 className="text-2xl font-sans text-text-primary">Pitwall</h1>
        <p className="text-text-secondary">Your AI Race Engineer</p>
        <Button className="bg-accent-primary hover:bg-accent-hover text-base">
          Get Started
        </Button>
      </div>
    </main>
  )
}

export default App
