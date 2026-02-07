import { useEffect, useRef } from "react"
import { useInfiniteSessions } from "../hooks/use-sessions"
import { useSessionFilterStore } from "@/state/session-filter-state"
import { SessionCard } from "./session-card"
import { Loader2 } from "lucide-react"

export function SessionListView() {
  const { data, fetchNextPage, hasNextPage, isFetchingNextPage, isLoading, isError } =
    useInfiniteSessions()
  const hasActiveFilters = useSessionFilterStore(
    (s) => s.track !== null || s.car !== null || s.dateStart !== null || s.dateEnd !== null,
  )
  const sentinelRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const sentinel = sentinelRef.current
    if (!sentinel) return

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasNextPage && !isFetchingNextPage) {
          fetchNextPage()
        }
      },
      { threshold: 0.1 },
    )

    observer.observe(sentinel)
    return () => observer.disconnect()
  }, [fetchNextPage, hasNextPage, isFetchingNextPage])

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-6 w-6 animate-spin text-text-muted" />
      </div>
    )
  }

  if (isError) {
    return (
      <div className="text-center py-12">
        <p className="text-sm text-text-secondary">Failed to load sessions. Please try again.</p>
      </div>
    )
  }

  const sessions = data?.pages.flatMap((page) => page) ?? []

  if (sessions.length === 0) {
    return (
      <div className="text-center py-12">
        <p className="text-sm text-text-secondary">
          {hasActiveFilters
            ? "No matching sessions"
            : "No sessions yet. Start iRacing and your sessions will appear here."}
        </p>
      </div>
    )
  }

  return (
    <div className="space-y-2">
      {sessions.map((session) => (
        <SessionCard key={session.id} session={session} />
      ))}
      <div ref={sentinelRef} className="h-4" />
      {isFetchingNextPage && (
        <div className="flex items-center justify-center py-4">
          <Loader2 className="h-5 w-5 animate-spin text-text-muted" />
        </div>
      )}
    </div>
  )
}
