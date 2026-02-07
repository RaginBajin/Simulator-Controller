import { SessionStatsBar } from "./session-stats-bar"
import { SessionFilters } from "./session-filters"
import { SessionListView } from "./session-list-view"

export function SessionHistoryPage() {
  return (
    <div className="flex flex-col h-full">
      <SessionStatsBar />
      <div className="px-4 py-3">
        <SessionFilters />
      </div>
      <div className="flex-1 overflow-y-auto px-4 pb-4">
        <SessionListView />
      </div>
    </div>
  )
}
