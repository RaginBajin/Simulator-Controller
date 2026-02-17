import { useEffect } from "react"
import { listen } from "@tauri-apps/api/event"
import { create } from "zustand"
import type { TrayStateKey, TrayStatusDetails, TrayStatusPayload } from "@/lib/types"

interface TrayStatusStore {
  state: TrayStateKey
  details: TrayStatusDetails | undefined
  setState: (state: TrayStateKey, details?: TrayStatusDetails) => void
}

export const useTrayStore = create<TrayStatusStore>((set) => ({
  state: "idle",
  details: undefined,
  setState: (state, details) => set({ state, details }),
}))

/**
 * Hook that listens for `tray:status-changed` events from the Rust backend
 * and updates the Zustand tray status store.
 *
 * Mount this once near the app root (e.g., in AppShell).
 */
export function useTrayStatus() {
  const setState = useTrayStore((s) => s.setState)

  useEffect(() => {
    const unlisten = listen<TrayStatusPayload>("tray:status-changed", (event) => {
      setState(event.payload.state, event.payload.details)
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [setState])

  return useTrayStore()
}
