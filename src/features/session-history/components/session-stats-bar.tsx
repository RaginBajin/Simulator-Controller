import { useSessionStats } from "../hooks/use-session-stats"
import { formatLapTime, formatRelativeDate } from "@/lib/format"

export function SessionStatsBar() {
  const { data: stats } = useSessionStats()

  if (!stats) return null

  return (
    <div className="bg-elevated border-b border-default px-4 py-3 flex items-center gap-6 text-sm">
      <div>
        <span className="text-text-muted">Sessions:</span>{" "}
        <span className="text-text-primary font-medium">{stats.totalSessions}</span>
      </div>
      {stats.latestSessionDate && (
        <div>
          <span className="text-text-muted">Latest:</span>{" "}
          <span className="text-text-primary">{formatRelativeDate(stats.latestSessionDate)}</span>
        </div>
      )}
      {stats.bestLapTimeMs != null && (
        <div>
          <span className="text-text-muted">Best Lap:</span>{" "}
          <span className="text-text-primary font-mono">{formatLapTime(stats.bestLapTimeMs)}</span>
        </div>
      )}
    </div>
  )
}
