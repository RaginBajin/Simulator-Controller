import { invoke } from "@tauri-apps/api/core"
import type { FilterOptions, FilterOptionsResponse, SessionStats, SessionSummary } from "./types"

/** Normalize a date-only string (YYYY-MM-DD) to an ISO start-of-day timestamp. */
function normalizeDateStart(d: string | null | undefined): string | null {
  if (!d) return null
  // Already an ISO timestamp
  if (d.includes("T")) return d
  return `${d}T00:00:00.000Z`
}

/** Normalize a date-only string (YYYY-MM-DD) to an ISO end-of-day timestamp. */
function normalizeDateEnd(d: string | null | undefined): string | null {
  if (!d) return null
  if (d.includes("T")) return d
  return `${d}T23:59:59.999Z`
}

export async function getSessions(
  filters?: Partial<FilterOptions>,
  limit?: number,
  offset?: number,
): Promise<SessionSummary[]> {
  return invoke("get_sessions", {
    track: filters?.track ?? null,
    car: filters?.car ?? null,
    dateStart: normalizeDateStart(filters?.dateStart),
    dateEnd: normalizeDateEnd(filters?.dateEnd),
    limit: limit ?? 50,
    offset: offset ?? 0,
  })
}

export async function getFilterOptions(): Promise<FilterOptionsResponse> {
  return invoke("get_filter_options")
}

export async function getSessionStats(
  filters?: Partial<FilterOptions>,
): Promise<SessionStats> {
  return invoke("get_session_stats", {
    track: filters?.track ?? null,
    car: filters?.car ?? null,
    dateStart: normalizeDateStart(filters?.dateStart),
    dateEnd: normalizeDateEnd(filters?.dateEnd),
  })
}
