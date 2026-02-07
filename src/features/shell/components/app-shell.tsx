import { useEffect, useCallback } from "react"
import { Outlet, useLocation } from "react-router-dom"
import { TabBar } from "./tab-bar"
import { ChatBar } from "./chat-bar"
import { useNotificationHandler } from "@/features/notifications"

const pathToPanelId: Record<string, string> = {
  "/summary": "tabpanel-summary",
  "/coaching": "tabpanel-coaching",
  "/telemetry": "tabpanel-telemetry",
}

const pathToTabId: Record<string, string> = {
  "/summary": "tab-summary",
  "/coaching": "tab-coaching",
  "/telemetry": "tab-telemetry",
}

export function AppShell() {
  useNotificationHandler()
  const location = useLocation()
  const matchedPath = Object.keys(pathToPanelId).find((p) =>
    location.pathname.startsWith(p),
  )
  const panelId = matchedPath ? pathToPanelId[matchedPath] : "tabpanel-summary"
  const tabId = matchedPath ? pathToTabId[matchedPath] : "tab-summary"

  const handleEscape = useCallback((e: KeyboardEvent) => {
    if (e.key === "Escape") {
      // Close any active element (modals/popovers) by blurring focus
      if (document.activeElement instanceof HTMLElement) {
        document.activeElement.blur()
      }
    }
  }, [])

  useEffect(() => {
    document.addEventListener("keydown", handleEscape)
    return () => document.removeEventListener("keydown", handleEscape)
  }, [handleEscape])

  return (
    <div className="flex h-screen flex-col bg-base">
      <TabBar />
      <main
        id={panelId}
        role="tabpanel"
        aria-labelledby={tabId}
        tabIndex={-1}
        className="flex-1 overflow-auto bg-surface p-6 outline-none"
      >
        <Outlet />
      </main>
      <ChatBar />
    </div>
  )
}
