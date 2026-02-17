import { useQuery } from "@tanstack/react-query"
import { getSessionStats } from "@/lib/tauri"
import { useSessionFilterStore } from "@/state/session-filter-state"

export function useSessionStats() {
  const { track, car, dateStart, dateEnd } = useSessionFilterStore()
  const filters = { track, car, dateStart, dateEnd }

  return useQuery({
    queryKey: ["session-stats", filters],
    queryFn: () => getSessionStats(filters),
  })
}
