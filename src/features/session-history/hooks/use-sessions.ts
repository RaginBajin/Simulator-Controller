import { useInfiniteQuery } from "@tanstack/react-query"
import { getSessions } from "@/lib/tauri"
import { useSessionFilterStore } from "@/state/session-filter-state"

const PAGE_SIZE = 50

export function useInfiniteSessions() {
  const { track, car, dateStart, dateEnd } = useSessionFilterStore()
  const filters = { track, car, dateStart, dateEnd }

  return useInfiniteQuery({
    queryKey: ["sessions", filters],
    queryFn: ({ pageParam = 0 }) => getSessions(filters, PAGE_SIZE, pageParam),
    getNextPageParam: (lastPage, allPages) =>
      lastPage.length === PAGE_SIZE ? allPages.length * PAGE_SIZE : undefined,
    initialPageParam: 0,
  })
}
