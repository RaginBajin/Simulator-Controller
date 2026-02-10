import { Outlet } from "react-router-dom"
import { TabBar } from "./tab-bar"
import { ChatBar } from "./chat-bar"

export function AppShell() {
  return (
    <div className="flex h-screen flex-col bg-base">
      <TabBar />
      <main className="flex-1 overflow-auto bg-surface p-6">
        <Outlet />
      </main>
      <ChatBar />
    </div>
  )
}
