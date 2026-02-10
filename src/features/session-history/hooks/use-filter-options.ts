import { useQuery } from "@tanstack/react-query"
import { getFilterOptions } from "@/lib/tauri"

export function useFilterOptions() {
  return useQuery({
    queryKey: ["filter-options"],
    queryFn: getFilterOptions,
    staleTime: 60_000,
  })
}
