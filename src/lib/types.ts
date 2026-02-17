export interface SessionSummary {
  id: string
  trackName: string
  carName: string
  sessionType: string
  startedAt: string
  endedAt: string | null
  lapCount: number
  bestLapTimeMs: number | null
  status: string
  integrityStatus: string | null
  importSource: string | null
}

export interface FilterOptions {
  track: string | null
  car: string | null
  dateStart: string | null
  dateEnd: string | null
}

export interface FilterOptionsResponse {
  tracks: string[]
  cars: string[]
}

export interface SessionStats {
  totalSessions: number
  bestLapTimeMs: number | null
  latestSessionDate: string | null
}

export interface SessionDetail {
  session: Session
  laps: LapSummary[]
  debrief: AiDebrief | null
  hasTelemetry: boolean
}

export interface Session {
  id: string
  trackName: string
  carName: string
  sessionType: string
  startedAt: string
  endedAt: string | null
  lapCount: number
  bestLapTimeMs: number | null
  status: string
  telemetryChecksum: string | null
  telemetryPath: string | null
  createdAt: string
  updatedAt: string
  deletedAt: string | null
  previousStatus: string | null
  integrityStatus: string | null
  integrityDetails: string | null
  integrityValidatedAt: string | null
}

export interface LapSummary {
  id: string
  sessionId: string
  lapNumber: number
  lapTimeMs: number
  isValid: boolean
  completionStatus: string
  createdAt: string
}

export interface AiDebrief {
  id: string
  sessionId: string
  coachingText: string | null
  insightsJson: string | null
  recommendationsJson: string | null
  providerName: string | null
  modelName: string | null
  createdAt: string
  updatedAt: string
}

export type TrayStateKey = "idle" | "recording" | "ready" | "error"

export interface TrayStatusDetails {
  track?: string
  lap?: number
  sessionId?: string
  lapCount?: number
  bestTime?: string
  errorMessage?: string
}

export interface TrayStatusPayload {
  type: string
  timestamp: string
  version: string
  state: TrayStateKey
  details?: TrayStatusDetails
}
