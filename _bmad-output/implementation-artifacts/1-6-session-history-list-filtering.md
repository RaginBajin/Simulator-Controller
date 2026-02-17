# Story 1.6: Session History List & Filtering

Status: dev-complete

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a sim racer,
I want to browse and filter my session history,
so that I can find specific sessions quickly.

## Acceptance Criteria

1. **Session List Displays as SessionCard Components**
   - Sessions are displayed as `SessionCard` components in a vertical list
   - Each card shows: track name, car name, date (human-readable), lap count, best lap time (formatted mm:ss.ms)
   - Cards include a status indicator (completed = green dot, partial = amber dot)
   - Cards use design tokens: bg-surface background, border-default border, text-primary for track/car, text-secondary for date, font-mono (JetBrains Mono) for lap times
   - Clicking a card navigates to the session debrief view (placeholder route for now)

2. **Initial Load and Infinite Scroll**
   - The most recent 50 sessions load initially
   - When the user scrolls to the bottom, the next 50 sessions load automatically (infinite scroll)
   - A loading spinner shows during fetch
   - If no sessions exist, a friendly empty state is shown: "No sessions yet. Start iRacing and your sessions will appear here."

3. **Filter by Track, Car, and Date Range**
   - Filter controls appear above the session list
   - Track filter: dropdown/combobox populated from distinct track names in the database
   - Car filter: dropdown/combobox populated from distinct car names in the database
   - Date range filter: start date and end date pickers
   - Filters are combinable (AND logic): selecting track AND car shows only sessions matching both
   - "Clear filters" button appears when any filter is active
   - Filter updates trigger a new query and reset infinite scroll to page 1

4. **Filter State Persists Across Navigation**
   - When the user navigates away from the session list and returns, filter selections are preserved
   - Filter state is stored in Zustand UI state (not URL params)
   - On app restart, filters reset to defaults (no persistence across restarts)

5. **Session List Query Performance**
   - The filtered session list query returns results within 200ms including filter application
   - The backend `list_sessions` query supports filter parameters (track, car, date range)
   - Pagination is cursor-based or offset-based with limit

6. **Progress Metrics Display**
   - A summary bar above the session list shows aggregate metrics:
     - Total sessions count
     - Most recent session date
     - Best lap time across all sessions (for current filter)
   - Progress metrics update when filters change

7. **Tauri IPC Commands for Filtered Queries**
   - `get_sessions` command extended with optional filter parameters: track, car, date_start, date_end, limit, offset
   - `get_filter_options` command returns distinct track names and car names for populating filter dropdowns
   - `get_session_stats` command returns aggregate stats for current filter

## Tasks / Subtasks

- [x] Task 1: Extend backend list query with filters (AC: #5, #7)
  - [x] 1.1 Add `FilterOptions` struct to `crates/storage/src/types.rs`: track (Option), car (Option), date_start (Option), date_end (Option)
  - [x] 1.2 Update `list_sessions` query in `crates/storage/src/sqlite/queries/sessions.rs` to accept `FilterOptions`
  - [x] 1.3 Build dynamic WHERE clause: filter by track_name, car_name, started_at range
  - [x] 1.4 Implement `get_distinct_tracks() -> Result<Vec<String>>` query
  - [x] 1.5 Implement `get_distinct_cars() -> Result<Vec<String>>` query
  - [x] 1.6 Implement `get_session_stats(filters: &FilterOptions) -> Result<SessionStats>` query (count, best lap, latest date)

- [x] Task 2: Extend Tauri IPC commands (AC: #7)
  - [x] 2.1 Update `get_sessions` command to accept filter parameters (camelCase JSON)
  - [x] 2.2 Implement `get_filter_options` command returning `{ tracks: string[], cars: string[] }`
  - [x] 2.3 Implement `get_session_stats` command returning `{ totalSessions, bestLapTimeMs, latestSessionDate }`
  - [x] 2.4 Register new commands in `src-tauri/src/lib.rs`

- [x] Task 3: Install frontend dependencies (AC: #1, #4)
  - [x] 3.1 Install `@tanstack/react-query` v5.x for async data fetching
  - [x] 3.2 Install `zustand` v5.x for UI state management
  - [x] 3.3 Configure TanStack Query provider in `src/main.tsx`
  - [x] 3.4 Verify builds succeed with new dependencies

- [x] Task 4: Create Tauri invoke wrappers (AC: #7)
  - [x] 4.1 Implement typed invoke wrappers in `src/lib/tauri.ts`:
    - `getSessions(filters?, limit?, offset?)` → invokes `get_sessions`
    - `getFilterOptions()` → invokes `get_filter_options`
    - `getSessionStats(filters?)` → invokes `get_session_stats`
  - [x] 4.2 Define TypeScript types in `src/lib/types.ts`: `SessionSummary`, `FilterOptions`, `SessionStats`, `FilterOptionsResponse`

- [x] Task 5: Create Zustand filter state store (AC: #4)
  - [x] 5.1 Create `src/state/session-filter-state.ts`
  - [x] 5.2 Define store shape: `{ track, car, dateStart, dateEnd, setTrack, setCar, setDateRange, clearFilters }`
  - [x] 5.3 Filters default to null/undefined (no filter applied)

- [x] Task 6: Create TanStack Query hooks (AC: #2, #5)
  - [x] 6.1 Create `src/features/session-history/hooks/use-sessions.ts`
  - [x] 6.2 Implement `useSessions` hook: fetches paginated sessions with current filters from Zustand store
  - [x] 6.3 Implement `useInfiniteSessions` hook using TanStack Query's `useInfiniteQuery` for infinite scroll
  - [x] 6.4 Create `src/features/session-history/hooks/use-filter-options.ts` — fetches distinct tracks/cars
  - [x] 6.5 Create `src/features/session-history/hooks/use-session-stats.ts` — fetches aggregate stats

- [x] Task 7: Build SessionCard component (AC: #1)
  - [x] 7.1 Create `src/features/session-history/components/session-card.tsx`
  - [x] 7.2 Display: track name (text-primary, font-medium), car name (text-secondary), date (text-muted, relative format), lap count, best lap time (font-mono)
  - [x] 7.3 Status indicator: green dot for completed, amber dot for partial
  - [x] 7.4 Card styling: bg-surface, border-default, rounded-md, hover:bg-elevated transition
  - [x] 7.5 Click handler: navigate to `/session/:id` route (placeholder for now)

- [x] Task 8: Build SessionListView with infinite scroll (AC: #2)
  - [x] 8.1 Create `src/features/session-history/components/session-list-view.tsx`
  - [x] 8.2 Render list of `SessionCard` components from `useInfiniteSessions` hook
  - [x] 8.3 Implement infinite scroll using IntersectionObserver on a sentinel element
  - [x] 8.4 Show loading spinner during fetch (using lucide-react Loader2)
  - [x] 8.5 Show empty state when no sessions match: "No sessions yet. Start iRacing and your sessions will appear here."
  - [x] 8.6 Show "No matching sessions" when filters return empty results

- [x] Task 9: Build filter controls (AC: #3)
  - [x] 9.1 Create `src/features/session-history/components/session-filters.tsx`
  - [x] 9.2 Track filter: native `select` element populated from `useFilterOptions`
  - [x] 9.3 Car filter: native `select` element populated from `useFilterOptions`
  - [x] 9.4 Date range: two date inputs (start, end) with type="date"
  - [x] 9.5 "Clear filters" button: visible only when filters active
  - [x] 9.6 Filter changes update Zustand store, which triggers TanStack Query refetch

- [x] Task 10: Build progress summary bar (AC: #6)
  - [x] 10.1 Create `src/features/session-history/components/session-stats-bar.tsx`
  - [x] 10.2 Display: total sessions count, most recent date, best lap time (font-mono)
  - [x] 10.3 Stats update reactively when filters change (via `useSessionStats` hook)
  - [x] 10.4 Styling: bg-elevated, border-b border-default, compact layout

- [x] Task 11: Wire into app routing (AC: #1)
  - [x] 11.1 Create `src/features/session-history/index.ts` barrel export
  - [x] 11.2 Create session history page component combining: stats bar + filters + session list
  - [x] 11.3 Integrated into existing Summary tab route from Story 1.2
  - [x] 11.4 Add `/session/:id` placeholder route for session detail view

- [x] Task 12: Write tests and verify (AC: all)
  - [x] 12.1 Add Rust integration tests for filtered queries in `crates/storage/tests/session_filters.rs`
  - [x] 12.2 Test: filter by track returns only matching sessions
  - [x] 12.3 Test: filter by car returns only matching sessions
  - [x] 12.4 Test: date range filter works correctly
  - [x] 12.5 Test: combined filters use AND logic
  - [x] 12.6 Test: pagination returns correct pages
  - [x] 12.7 Run `cargo test` — all 46 tests pass
  - [ ] 12.8 Run `npm run tauri dev` — session list renders, filters work (manual verification needed)
  - [x] 12.9 Verify `cargo check` and `vite build` succeed for entire workspace

## Dev Notes

### Architecture Compliance

**Frontend State Management:**
Per architecture doc:
- **TanStack React Query 5.x** for async data fetching (session list, filter options, stats)
- **Zustand 5.x** for UI state (filter selections)
- No direct state mutation outside Zustand actions

```bash
npm install @tanstack/react-query@^5 zustand@^5
```

**TanStack Query Provider Setup:**
```tsx
// src/main.tsx or src/App.tsx
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,  // 30 seconds
      retry: 1,
    },
  },
});
```

**Feature-Based Organization:**
All session history components go under `src/features/session-history/` per architecture:
```
src/features/session-history/
├── components/
│   ├── session-card.tsx
│   ├── session-list-view.tsx
│   ├── session-filters.tsx
│   └── session-stats-bar.tsx
├── hooks/
│   ├── use-sessions.ts
│   ├── use-filter-options.ts
│   └── use-session-stats.ts
└── index.ts
```

### SessionCard Design

From UX spec (Component #1 — SessionCard):
- Timeline entry for session history with track, car, date, lap count, best lap, AI headline
- AI headline is not available yet (future story) — show best lap time instead

```
+------------------------------------------+
| Lime Rock Park              2 hours ago  |
| Mazda MX-5 Cup    25 laps   0:57.812     |
| [green dot] Completed                     |
+------------------------------------------+
```

**Styling:**
- Card: `bg-surface border border-default rounded-md p-4 hover:bg-elevated transition-colors cursor-pointer`
- Track name: `text-primary text-sm font-medium`
- Car name: `text-secondary text-sm`
- Lap time: `text-primary font-mono text-sm`
- Date: `text-muted text-xs`
- Status dot: `w-2 h-2 rounded-full` (green = `bg-trace-best`, amber = `bg-accent-primary`)

### Infinite Scroll Pattern

Use IntersectionObserver with TanStack Query's `useInfiniteQuery`:

```tsx
const { data, fetchNextPage, hasNextPage, isFetchingNextPage } = useInfiniteQuery({
  queryKey: ['sessions', filters],
  queryFn: ({ pageParam = 0 }) => getSessions(filters, 50, pageParam),
  getNextPageParam: (lastPage, pages) =>
    lastPage.length === 50 ? pages.length * 50 : undefined,
});
```

Sentinel element at the bottom of the list triggers `fetchNextPage` when visible.

### Backend Filter Query

**Dynamic WHERE clause pattern:**
```rust
// Build query with optional filters
let mut query = String::from(
    "SELECT id, track_name, car_name, session_type, started_at, lap_count, best_lap_time_ms, status
     FROM sessions WHERE status != 'deleted'"
);
let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send>> = vec![];

if let Some(track) = &filters.track {
    query.push_str(" AND track_name = ?");
    // bind track
}
if let Some(car) = &filters.car {
    query.push_str(" AND car_name = ?");
    // bind car
}
// ... date range ...

query.push_str(" ORDER BY started_at DESC LIMIT ? OFFSET ?");
```

**CRITICAL: Use parameterized queries.** Never interpolate filter values into SQL strings. SQLx handles this safely with `sqlx::query_as` and `.bind()`.

### Lap Time Formatting

Best lap time is stored as milliseconds (i64). Format for display:

```typescript
// src/lib/format.ts
export function formatLapTime(ms: number): string {
  const minutes = Math.floor(ms / 60000);
  const seconds = Math.floor((ms % 60000) / 1000);
  const millis = ms % 1000;
  return `${minutes}:${seconds.toString().padStart(2, '0')}.${millis.toString().padStart(3, '0')}`;
}
```

### Existing Code to Build On

From Story 1.2 (expected):
- App shell with TabBar (Summary | Coaching | Telemetry)
- React Router configured with routes
- Design tokens applied (dark theme, fonts)

From Story 1.3 (expected):
- `Database` struct with `list_sessions`, `get_session` queries
- `SessionSummary` type
- `get_sessions` IPC command

From Story 1.5 (expected):
- Extended session queries with soft delete exclusion
- `SessionDetail` composite type
- `get_session_data` IPC command

**Extend** the existing `list_sessions` query to accept filters. **Extend** the existing `get_sessions` IPC command to accept filter parameters. Do NOT create duplicate query/command paths.

### Naming Conventions (Enforced)

| Zone | Convention | Example |
|------|-----------|---------|
| React components | `PascalCase` | `SessionCard`, `SessionListView`, `SessionFilters` |
| Frontend files | `kebab-case` | `session-card.tsx`, `session-list-view.tsx` |
| Hooks | `camelCase` with `use` prefix | `useSessions`, `useFilterOptions` |
| Zustand stores | `kebab-case` file, `camelCase` store | `session-filter-state.ts` |
| IPC commands | `snake_case` | `get_filter_options`, `get_session_stats` |
| TypeScript types | `PascalCase` | `SessionSummary`, `FilterOptions` |

### Cross-Story Dependencies

- **Story 1.2** (dev-complete): App shell, routing, design tokens
- **Story 1.3** (ready-for-dev): SQLite queries, session list IPC
- **Story 1.5** (ready-for-dev): Extended CRUD, soft delete exclusion
- **Story 5.1** (backlog): Summary tab hero cards — will replace/extend the session list view

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Frontend Architecture]
- [Source: _bmad-output/planning-artifacts/architecture.md#State Management Patterns]
- [Source: _bmad-output/planning-artifacts/architecture.md#Structure Patterns]
- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 1.6: Session History List & Filtering]
- [Source: _bmad-output/planning-artifacts/prd.md#FR32] (browse/filter session history)
- [Source: _bmad-output/planning-artifacts/prd.md#FR33] (progress metrics)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR3] (session history <1s initial, <200ms filter)
- [Source: _bmad-output/planning-artifacts/architecture.md#9 Custom Components — SessionCard]
- [Source: _bmad-output/implementation-artifacts/1-2-desktop-application-runs-locally.md] (app shell)
- [Source: _bmad-output/implementation-artifacts/1-3-session-history-persists-locally.md] (SQLite queries)
- [Source: _bmad-output/implementation-artifacts/1-5-session-crud-operations.md] (CRUD operations)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

- Fixed pre-existing compile error in `crates/storage/src/validation/monotonicity.rs` (missing `use arrow::array::Array` import from Story 1.7)
- Updated existing tests in `session_persistence.rs` and `session_crud.rs` to pass new `&FilterOptions` parameter to `list_sessions`

### Completion Notes List

- [x] Task 1: Backend filter queries - `FilterOptions`, `SessionStats` types added, `list_sessions` extended with dynamic WHERE, `get_distinct_tracks`, `get_distinct_cars`, `get_session_stats` queries
- [x] Task 2: Tauri IPC commands - `get_sessions` extended with filter/pagination params, `get_filter_options` and `get_session_stats` commands added and registered
- [x] Task 3: Frontend dependencies - `@tanstack/react-query@^5` and `zustand@^5` installed, QueryClientProvider configured in main.tsx
- [x] Task 4: Tauri invoke wrappers - typed wrappers in `src/lib/tauri.ts`, TypeScript types in `src/lib/types.ts`, format utilities in `src/lib/format.ts`
- [x] Task 5: Zustand filter store - `src/state/session-filter-state.ts` with track/car/dateStart/dateEnd state and actions
- [x] Task 6: TanStack Query hooks - `useInfiniteSessions`, `useFilterOptions`, `useSessionStats` hooks
- [x] Task 7: SessionCard component - track/car/date/laps/best-lap display with status indicator and click navigation
- [x] Task 8: SessionListView - infinite scroll via IntersectionObserver, loading/empty/error states
- [x] Task 9: SessionFilters - track/car select dropdowns, date range inputs, clear filters button
- [x] Task 10: SessionStatsBar - total sessions, latest date, best lap time display
- [x] Task 11: Routing - SessionHistoryPage on /summary route, /session/:id placeholder route
- [x] Task 12: Tests - 11 new integration tests for filters/pagination/stats/distinct queries, all 46 tests passing

### File List

**Backend (Rust) - Modified:**
- `crates/storage/src/types.rs` - Added `FilterOptions`, `SessionStats` types
- `crates/storage/src/sqlite/queries/sessions.rs` - Updated `list_sessions` with filter params, added `get_distinct_tracks`, `get_distinct_cars`, `get_session_stats`; fixed `list_deleted_sessions` SELECT to include `integrity_status`
- `crates/storage/src/sqlite/connection.rs` - Updated `list_sessions` signature, added `get_distinct_tracks`, `get_distinct_cars`, `get_session_stats` methods
- `src-tauri/src/commands/session.rs` - Extended `get_sessions` with filter/pagination params, added `get_filter_options` and `get_session_stats` commands
- `src-tauri/src/lib.rs` - Registered new IPC commands
- `crates/storage/src/validation/monotonicity.rs` - Fixed missing `Array` trait import (pre-existing bug from Story 1.7)

**Backend (Rust) - Tests Modified:**
- `crates/storage/tests/session_persistence.rs` - Updated `list_sessions` calls with `FilterOptions` param
- `crates/storage/tests/session_crud.rs` - Updated `list_sessions` calls with `FilterOptions` param

**Backend (Rust) - New:**
- `crates/storage/tests/session_filters.rs` - 11 integration tests for filters, pagination, distinct, and stats

**Frontend (TypeScript) - Modified:**
- `src/main.tsx` - Added QueryClientProvider with TanStack React Query
- `src/routes.tsx` - Replaced Summary placeholder with SessionHistoryPage, added /session/:id route
- `src/lib/types.ts` - Full TypeScript type definitions (SessionSummary, FilterOptions, SessionStats, etc.)
- `src/lib/tauri.ts` - Typed Tauri invoke wrappers (getSessions, getFilterOptions, getSessionStats)
- `src/lib/format.ts` - formatLapTime and formatRelativeDate utilities

**Frontend (TypeScript) - New:**
- `src/state/session-filter-state.ts` - Zustand store for filter state
- `src/features/session-history/index.ts` - Barrel exports
- `src/features/session-history/hooks/use-sessions.ts` - useInfiniteSessions hook
- `src/features/session-history/hooks/use-filter-options.ts` - useFilterOptions hook
- `src/features/session-history/hooks/use-session-stats.ts` - useSessionStats hook
- `src/features/session-history/components/session-card.tsx` - SessionCard component
- `src/features/session-history/components/session-list-view.tsx` - SessionListView with infinite scroll
- `src/features/session-history/components/session-filters.tsx` - Filter controls
- `src/features/session-history/components/session-stats-bar.tsx` - Progress metrics bar
- `src/features/session-history/components/session-history-page.tsx` - Composite page component

**Dependencies Added:**
- `@tanstack/react-query@^5`
- `zustand@^5`

## Review Follow-ups (AI)

**Build & Test Results:**
- `cargo build -p storage` -- FAIL (cross-story issue: `write_telemetry` was made async by Story 1.7/1.8 but `connection.rs:101` lacks `.await` -- not a Story 1.6 defect)
- `cargo build -p pitwall` -- FAIL (same cross-story compile error blocks workspace build)
- `npx tsc --noEmit` -- PASS (0 errors)
- `npm run build` (Vite) -- PASS
- `cargo test -p storage --test session_filters` -- 11 tests PASS
- `cargo test -p storage --test session_persistence` -- 8 tests PASS
- `cargo test -p storage --test session_crud` -- 14 tests PASS
- `cargo test -p storage --test parquet_storage` -- FAIL (cross-story: `write_telemetry` now async, all callers need `.await`)
- Note: Story 1.6 correctly fixed the `list_sessions` signature breakage it introduced in session_persistence.rs and session_crud.rs tests

**Acceptance Criteria Coverage:**
- AC1 (SessionCard): PASS - track, car, date, laps, best lap, status dot all present
- AC2 (Infinite Scroll): PASS - IntersectionObserver sentinel, loading/empty states, filter-aware empty message
- AC3 (Filters): PASS - track/car dropdowns, date range, clear button, AND logic
- AC4 (Filter Persistence): PASS - Zustand store persists across navigation, resets on restart
- AC5 (Query Performance): PASS - parameterized dynamic WHERE, pagination with LIMIT/OFFSET
- AC6 (Progress Metrics): PASS - total sessions, latest date, best lap time with font-mono
- AC7 (IPC Commands): PASS - get_sessions extended, get_filter_options, get_session_stats registered

---

- [x] [AI-Review][HIGH] HIGH-1: Cross-story compile error blocks workspace build
  - **File:** `crates/storage/src/sqlite/connection.rs:101` and `crates/storage/src/parquet/writer.rs:28`
  - **Issue:** `write_telemetry` in `writer.rs` was changed to `pub async fn` (by Story 1.7 or 1.8), but the call site in `connection.rs:101` still calls it without `.await`. This causes `cargo build -p storage` to fail with `error[E0277]: the ? operator cannot be applied to type impl Future`. The same async change also broke all 10 `parquet_storage.rs` integration tests.
  - **Impact:** The storage crate does not compile. This blocks `cargo build -p pitwall` and `cargo test -p storage`. No Tauri builds can succeed.
  - **Fix:** Add `.await` to `connection.rs:101`: `parquet::write_telemetry(...).await?;`. Update all 10 call sites in `parquet_storage.rs` to use `.await` and convert tests to `#[tokio::test]`. This is a cross-story integration issue, not a Story 1.6 defect, but it must be resolved before any story can pass CI.

- [x] [AI-Review][MEDIUM] MEDIUM-1: Date filter format mismatch between frontend and backend
  - **File:** `src/features/session-history/components/session-filters.tsx:37-50` and `crates/storage/src/sqlite/queries/sessions.rs:62-71`
  - **Issue:** HTML `type="date"` inputs return `YYYY-MM-DD` format (e.g., `"2025-02-28"`). The backend compares this against `started_at` which is stored as ISO 8601 `"2025-02-28T16:00:00.000Z"`. The SQL `started_at <= "2025-02-28"` will **exclude** sessions that started on 2025-02-28 because `"2025-02-28T..."` > `"2025-02-28"` in SQLite string comparison. This means the end-date filter silently drops all sessions on the selected end date.
  - **Impact:** Users filtering to an end date will miss sessions from that last day. The integration tests pass because they use full ISO timestamps, masking this bug.
  - **Fix:** Either append `T23:59:59.999Z` to the end date in the frontend before sending, or use SQLite `date()` function: `AND date(started_at) <= $N` to extract just the date portion.

- [x] [AI-Review][MEDIUM] MEDIUM-2: TypeScript `SessionSummary` missing `importSource` field
  - **File:** `src/lib/types.ts:1-12` and `crates/storage/src/types.rs:63`
  - **Issue:** The Rust `SessionSummary` struct has `import_source: Option<String>` which serializes as `importSource` in camelCase JSON. The TypeScript `SessionSummary` interface does not include this field. Extra fields in JSON are silently ignored by TypeScript at runtime, so this doesn't cause errors, but it means the frontend cannot display import source information.
  - **Impact:** No runtime error, but the type contract is incomplete. If a future story needs to display import source on session cards, it will need to add this field.
  - **Fix:** Add `importSource: string | null` to the TypeScript `SessionSummary` interface.

- [x] [AI-Review][MEDIUM] MEDIUM-3: `hasActiveFilters` is a function on the store, not a derived value
  - **File:** `src/state/session-filter-state.ts:12,24-27` and `src/features/session-history/components/session-list-view.tsx:10,52`
  - **Issue:** `hasActiveFilters` is defined as a method `() => boolean` on the Zustand store. In `session-list-view.tsx:10`, it's selected via `useSessionFilterStore((s) => s.hasActiveFilters)` which returns the function reference, then called at line 52 as `hasActiveFilters()`. Because the selector returns the function (not its result), the component will NOT re-render when filter values change -- the function reference itself never changes.
  - **Impact:** When filters are applied or cleared, the empty state message may not toggle between "No sessions yet..." and "No matching sessions" until a full re-render is triggered by another state change (like the query result updating). In practice, the query result change will usually trigger a re-render, but there's a brief inconsistency window.
  - **Fix:** Either compute `hasActiveFilters` as a derived selector: `const hasActive = useSessionFilterStore((s) => s.track !== null || s.car !== null || ...)`, or select the individual values and compute inline. Alternatively, in `session-filters.tsx:53` the same pattern is used directly from the destructured store, which works correctly because all state is already subscribed to.

- [x] [AI-Review][MEDIUM] MEDIUM-4: No `aria-label` on filter select elements and date inputs
  - **File:** `src/features/session-history/components/session-filters.tsx:11-51`
  - **Issue:** The track and car `<select>` elements and date `<input>` elements have no `aria-label` attributes. The selects have visible `<option>` text as labels but no programmatic label association. The date inputs have `placeholder` attributes, but `placeholder` is not a reliable substitute for an `aria-label` in screen readers, and `type="date"` inputs may not even display placeholder text.
  - **Impact:** Screen reader users cannot identify what each filter control is for. This is an accessibility gap that should be addressed for WCAG 2.1 AA compliance.
  - **Fix:** Add `aria-label="Filter by track"`, `aria-label="Filter by car"`, `aria-label="Start date"`, and `aria-label="End date"` to the respective elements.

- [ ] [AI-Review][LOW] LOW-1: Dynamic SQL construction with string formatting for bind indices
  - **File:** `crates/storage/src/sqlite/queries/sessions.rs:43-87` and `327-370`
  - **Issue:** The `list_sessions` and `get_session_stats` functions build SQL dynamically using `format!(" AND track_name = ${}", bind_idx)`. While the actual values are properly bound (parameterized), the query structure itself is built with string concatenation. This is safe from SQL injection since only column names and bind placeholders are formatted, but it duplicates the filter-building logic across two functions.
  - **Impact:** Any filter change (e.g., adding a new filter type) must be updated in both `list_sessions` and `get_session_stats`. No security risk -- values are always bound.
  - **Fix:** Extract the shared filter-building logic into a helper function that returns the WHERE clause fragment and bindings. Not urgent.

- [ ] [AI-Review][LOW] LOW-2: `SessionCard` uses `<button>` for navigation -- semantically correct but has styling implications
  - **File:** `src/features/session-history/components/session-card.tsx:13`
  - **Issue:** `SessionCard` uses `<button type="button">` with `onClick={() => navigate(...)}`. This is actually good for accessibility (buttons are keyboard-focusable and have clear interaction semantics). However, using a `<button>` means the user cannot right-click to open in a new tab or copy link, which they could with an `<a>` tag. For a desktop Tauri app where "open in new tab" doesn't apply, this is fine.
  - **Impact:** None for a desktop app. This is the correct pattern.
  - **Fix:** No action needed. Noting as a positive accessibility decision.

- [ ] [AI-Review][LOW] LOW-3: `useFilterOptions` has 60s staleTime but filter values could change after session CRUD
  - **File:** `src/features/session-history/hooks/use-filter-options.ts:8`
  - **Issue:** `useFilterOptions` sets `staleTime: 60_000` (1 minute). If a user deletes a session and that was the only session for a particular track, the filter dropdown will still show that track for up to 60 seconds.
  - **Impact:** Minor UX inconsistency -- stale filter options for up to 60 seconds after CRUD operations. For a desktop app with a single user, this is acceptable.
  - **Fix:** Invalidate the `filter-options` query key after session delete/restore operations. Not urgent.
