import { create } from "zustand"

interface SessionFilterState {
  track: string | null
  car: string | null
  dateStart: string | null
  dateEnd: string | null
  setTrack: (track: string | null) => void
  setCar: (car: string | null) => void
  setDateRange: (start: string | null, end: string | null) => void
  clearFilters: () => void
  hasActiveFilters: () => boolean
}

export const useSessionFilterStore = create<SessionFilterState>((set, get) => ({
  track: null,
  car: null,
  dateStart: null,
  dateEnd: null,
  setTrack: (track) => set({ track }),
  setCar: (car) => set({ car }),
  setDateRange: (dateStart, dateEnd) => set({ dateStart, dateEnd }),
  clearFilters: () => set({ track: null, car: null, dateStart: null, dateEnd: null }),
  hasActiveFilters: () => {
    const s = get()
    return s.track !== null || s.car !== null || s.dateStart !== null || s.dateEnd !== null
  },
}))
