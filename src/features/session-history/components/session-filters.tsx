import { useFilterOptions } from "../hooks/use-filter-options"
import { useSessionFilterStore } from "@/state/session-filter-state"

export function SessionFilters() {
  const { data: filterOptions } = useFilterOptions()
  const { track, car, dateStart, dateEnd, setTrack, setCar, setDateRange, clearFilters, hasActiveFilters } =
    useSessionFilterStore()

  return (
    <div className="flex flex-wrap items-center gap-3">
      <select
        value={track ?? ""}
        onChange={(e) => setTrack(e.target.value || null)}
        aria-label="Filter by track"
        className="bg-surface border border-default rounded-md px-3 py-1.5 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-accent-primary"
      >
        <option value="">All Tracks</option>
        {filterOptions?.tracks.map((t) => (
          <option key={t} value={t}>
            {t}
          </option>
        ))}
      </select>

      <select
        value={car ?? ""}
        onChange={(e) => setCar(e.target.value || null)}
        aria-label="Filter by car"
        className="bg-surface border border-default rounded-md px-3 py-1.5 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-accent-primary"
      >
        <option value="">All Cars</option>
        {filterOptions?.cars.map((c) => (
          <option key={c} value={c}>
            {c}
          </option>
        ))}
      </select>

      <input
        type="date"
        value={dateStart ?? ""}
        onChange={(e) => setDateRange(e.target.value || null, dateEnd)}
        aria-label="Start date"
        className="bg-surface border border-default rounded-md px-3 py-1.5 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-accent-primary"
      />

      <input
        type="date"
        value={dateEnd ?? ""}
        onChange={(e) => setDateRange(dateStart, e.target.value || null)}
        aria-label="End date"
        className="bg-surface border border-default rounded-md px-3 py-1.5 text-sm text-text-primary focus:outline-none focus:ring-1 focus:ring-accent-primary"
      />

      {hasActiveFilters() && (
        <button
          type="button"
          onClick={clearFilters}
          className="text-sm text-text-secondary hover:text-text-primary transition-colors"
        >
          Clear filters
        </button>
      )}
    </div>
  )
}
