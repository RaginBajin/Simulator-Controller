import { useNavigate } from "react-router-dom"
import type { SessionSummary } from "@/lib/types"
import { formatLapTime, formatRelativeDate } from "@/lib/format"

interface SessionCardProps {
  session: SessionSummary
}

export function SessionCard({ session }: SessionCardProps) {
  const navigate = useNavigate()

  return (
    <button
      type="button"
      className="w-full text-left bg-surface border border-default rounded-md p-4 hover:bg-elevated transition-colors cursor-pointer"
      onClick={() => navigate(`/session/${session.id}`)}
    >
      <div className="flex items-start justify-between">
        <span className="text-text-primary text-sm font-medium">{session.trackName}</span>
        <span className="text-text-muted text-xs">{formatRelativeDate(session.startedAt)}</span>
      </div>
      <div className="flex items-center justify-between mt-1">
        <span className="text-text-secondary text-sm">{session.carName}</span>
        <div className="flex items-center gap-3 text-sm">
          <span className="text-text-secondary">{session.lapCount} laps</span>
          {session.bestLapTimeMs != null && (
            <span className="text-text-primary font-mono">{formatLapTime(session.bestLapTimeMs)}</span>
          )}
        </div>
      </div>
      <div className="flex items-center gap-1.5 mt-2">
        <span
          className={`w-2 h-2 rounded-full ${
            session.status === "completed" ? "bg-trace-best" : "bg-accent-primary"
          }`}
        />
        <span className="text-text-muted text-xs capitalize">{session.status}</span>
      </div>
    </button>
  )
}
