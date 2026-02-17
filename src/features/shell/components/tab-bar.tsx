import { useRef, useCallback, useEffect } from "react"
import { useNavigate, useLocation } from "react-router-dom"

const tabs = [
  { label: "Summary", path: "/summary", id: "tab-summary", panelId: "tabpanel-summary" },
  { label: "Coaching", path: "/coaching", id: "tab-coaching", panelId: "tabpanel-coaching" },
  { label: "Telemetry", path: "/telemetry", id: "tab-telemetry", panelId: "tabpanel-telemetry" },
] as const

export function TabBar() {
  const navigate = useNavigate()
  const location = useLocation()
  const tabRefs = useRef<(HTMLButtonElement | null)[]>([])

  const activeIndex = tabs.findIndex((tab) => location.pathname.startsWith(tab.path))
  const resolvedIndex = activeIndex === -1 ? 0 : activeIndex

  const activateTab = useCallback(
    (index: number) => {
      navigate(tabs[index].path)
      tabRefs.current[index]?.focus()
    },
    [navigate],
  )

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      let nextIndex: number | null = null

      switch (e.key) {
        case "ArrowRight":
          nextIndex = (resolvedIndex + 1) % tabs.length
          break
        case "ArrowLeft":
          nextIndex = (resolvedIndex - 1 + tabs.length) % tabs.length
          break
        case "Home":
          nextIndex = 0
          break
        case "End":
          nextIndex = tabs.length - 1
          break
        default:
          return
      }

      e.preventDefault()
      activateTab(nextIndex)
    },
    [resolvedIndex, activateTab],
  )

  // Move focus to the tab panel after a tab switch triggered by click
  useEffect(() => {
    const panel = document.getElementById(tabs[resolvedIndex]?.panelId)
    if (panel && !tabRefs.current.some((ref) => ref === document.activeElement)) {
      panel.focus({ preventScroll: true })
    }
  }, [resolvedIndex])

  return (
    <div
      role="tablist"
      aria-label="Main navigation"
      className="flex border-b border-border-default bg-surface"
      onKeyDown={handleKeyDown}
    >
      {tabs.map((tab, index) => {
        const isActive = index === resolvedIndex
        return (
          <button
            key={tab.path}
            ref={(el) => { tabRefs.current[index] = el }}
            role="tab"
            id={tab.id}
            aria-selected={isActive}
            aria-controls={isActive ? tab.panelId : undefined}
            tabIndex={isActive ? 0 : -1}
            onClick={() => activateTab(index)}
            className={[
              "px-4 py-3 text-sm font-medium transition-colors duration-150 cursor-pointer",
              isActive
                ? "text-text-primary border-b-2 border-accent-primary"
                : "text-text-secondary hover:text-text-primary",
            ].join(" ")}
          >
            {tab.label}
          </button>
        )
      })}
    </div>
  )
}
