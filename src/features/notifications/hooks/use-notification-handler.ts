import { useEffect } from "react"
import { listen } from "@tauri-apps/api/event"
import { useNavigate } from "react-router-dom"

/**
 * Listens for tray-driven debrief navigation events from the Rust backend
 * and navigates the app to the corresponding session detail page.
 *
 * When a user clicks the tray icon while a debrief is ready (Story 2.2),
 * `tray:navigate-to-debrief` is emitted with the session_id as payload.
 *
 * Should be registered once in AppShell for app-wide listening.
 */
export function useNotificationHandler() {
  const navigate = useNavigate()

  useEffect(() => {
    const unlisten = listen<string>(
      "tray:navigate-to-debrief",
      (event) => {
        const sessionId = event.payload
        if (sessionId) {
          navigate(`/session/${sessionId}`)
        }
      },
    )

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [navigate])
}
