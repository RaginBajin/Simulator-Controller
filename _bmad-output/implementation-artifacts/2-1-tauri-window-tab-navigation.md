# Story 2.1: Tauri Window & Tab Navigation

Status: complete

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a sim racer,
I want a clean desktop window with clear tab navigation,
so that I can easily switch between Summary, Coaching, and Telemetry views.

## Acceptance Criteria

1. **Window Configuration Matches UX Spec**
   - Window defaults to 1200x800px, user-resizable, minimum 800x600px
   - Window title displays "Pitwall"
   - Window uses dark theme (bg-base background)
   - *NOTE: Already implemented in Story 1.2 — verify and confirm, no changes expected*

2. **Tab Bar Navigation Is Fully Functional**
   - Tab bar at top: Summary | Coaching | Telemetry — always visible, always this order
   - Active tab has amber underline (2px, `border-accent-primary`)
   - Inactive tabs are `text-secondary` with hover to `text-primary`
   - Tab underline transition: 150ms ease
   - URL/state updates via React Router when tabs change
   - Persistent chat bar remains visible at the bottom across all tabs
   - *NOTE: Tab bar, routing, chat bar already implemented in Story 1.2 — enhance with transition smoothness and verify*

3. **Keyboard Navigation (WCAG 2.1 AA)**
   - Tab/Shift+Tab cycles through interactive elements in logical order
   - Visible focus rings appear: 2px solid amber (`accent-primary`) with 2px offset
   - Pressing Escape closes any modal or popover (when applicable)
   - Arrow keys navigate between tabs when tab bar has focus
   - Enter/Space activates the focused tab
   - Focus is managed correctly when switching tabs (focus moves to main content area)

4. **System Tray Icon Displays Provider Connection Status**
   - System tray icon is present when the app is running
   - Icon displays connection status: ready (green) / degraded (amber) / offline (gray)
   - Status updates in real-time on connectivity changes
   - Tooltip shows provider name and last validation time
   - *NOTE: Full system tray is Story 2.2 — this AC only requires the initial tray icon setup with static status display. Story 2.2 will add recording states, minimize-to-tray, and dynamic updates*

5. **Tab Transitions Are Smooth**
   - Content area transitions smoothly when switching tabs (no flash/jump)
   - Tab underline slides to active tab (150ms ease CSS transition)
   - No layout shift during tab changes (fixed layout, only content area changes via Outlet)

## Tasks / Subtasks

- [x] Task 1: Verify and lock down existing shell (AC: #1, #2)
  - [x] 1.1 Confirm `tauri.conf.json` window settings match spec (1200x800, min 800x600, title "Pitwall") — verified, matches spec
  - [x] 1.2 Confirm `TabBar` component matches UX spec (Summary | Coaching | Telemetry, amber underline) — verified
  - [x] 1.3 Confirm `ChatBar` persists across all tabs — verified via AppShell layout
  - [x] 1.4 Add smooth 150ms transition to tab underline if not already present — already present (`transition-colors duration-150`)

- [x] Task 2: Implement keyboard navigation (AC: #3)
  - [x] 2.1 Add global focus-visible styles in `globals.css`: 2px solid amber outline with 2px offset
  - [x] 2.2 Add `role="tablist"` to tab bar `<nav>`, `role="tab"` to each button, `role="tabpanel"` to main content area
  - [x] 2.3 Add `aria-selected`, `aria-controls`, `aria-labelledby`, and `id` attributes to tabs and panels
  - [x] 2.4 Implement arrow key navigation within tab bar (Left/Right to cycle tabs, Home/End for first/last)
  - [x] 2.5 Add `tabIndex` management: active tab gets `tabIndex={0}`, inactive tabs get `tabIndex={-1}`
  - [x] 2.6 On tab change, manage focus: move focus to the content area (`role="tabpanel"`) after tab activation
  - [x] 2.7 Add Escape key handler at shell level for closing modals/popovers (blurs active element)

- [x] Task 3: Set up initial system tray icon (AC: #4)
  - [x] 3.1 Add `tray-icon` feature to Cargo.toml and `core:event:default` permission to capabilities
  - [x] 3.2 Create `src-tauri/src/tray.rs` with tray icon setup function
  - [x] 3.3 Set initial tray icon to gray (idle/offline state) with tooltip "Pitwall — Waiting for iRacing"
  - [x] 3.4 Register tray module and setup in `lib.rs` during Tauri app build
  - [x] 3.5 Add tray icon assets: gray circle RGBA for idle state in `src-tauri/icons/tray/`
  - [x] 3.6 Emit `tray:status-changed` event from backend on initial setup
  - [x] 3.7 *NOTE: Did NOT implement minimize-to-tray, recording states, or click-to-open — that's Story 2.2*

- [x] Task 4: Smooth tab transitions (AC: #5)
  - [x] 4.1 Verify no layout shift occurs during tab switches — confirmed (fixed `h-screen flex flex-col` layout)
  - [x] 4.2 Confirm tab underline has CSS transition applied — confirmed (`transition-colors duration-150`)
  - [x] 4.3 No fade transition needed — content swap is instant with no flash/jump due to fixed layout

- [x] Task 5: Build verification and testing (AC: all)
  - [x] 5.1 Run `cargo build` — passed (1 warning from notifications.rs, not Story 2.1 code)
  - [x] 5.2 Run `cargo test` — all 77 tests passed
  - [x] 5.3 Run `npm run build` — passed clean
  - [x] 5.4 Run `npm run typecheck` — passed clean
  - [ ] 5.5 Manual verification: `cargo tauri dev` opens window, tabs navigate, keyboard works — requires manual testing

## Dev Notes

### What's Already Built (from Story 1.2)

The shell infrastructure is largely complete. Story 1.2 built:
- `src/features/shell/components/app-shell.tsx` — Main layout: TabBar (top) + Outlet (center) + ChatBar (bottom)
- `src/features/shell/components/tab-bar.tsx` — NavLink tabs with amber underline, `transition-colors duration-150`
- `src/features/shell/components/chat-bar.tsx` — Placeholder chat input bar
- `src/features/shell/components/placeholder-page.tsx` — Generic placeholder for tab sections
- `src/routes.tsx` — React Router with `/summary`, `/coaching`, `/telemetry`, `/session/:id`
- `src-tauri/tauri.conf.json` — Window 1200x800, min 800x600, NSIS+DMG bundle

**This story's primary NEW work is:**
1. Keyboard navigation and ARIA compliance (Task 2 — most substantial)
2. Initial system tray icon setup (Task 3 — new Rust code)
3. Transition polish (Task 4 — minor CSS)

### Architecture Patterns to Follow

- **IPC Events:** Use `domain:action` kebab-case naming. Tray events should be `tray:status-changed`
- **Error Contract:** `{ code, message, details?, retryable? }` for any IPC errors
- **State Management:** Use Zustand for UI state (e.g., `ui-state.ts` for modal/focus management). TanStack Query for server state
- **Component System:** shadcn/ui + Radix primitives. Use existing design tokens from `globals.css`
- **File Structure:** New tray code goes in `src-tauri/src/tray.rs`. Frontend tray integration (if any) goes in `src/features/tray/`

### Design Tokens (from globals.css)

- `bg-base: #0A0A0F` — Application background
- `bg-surface: #141419` — Cards, panels
- `bg-elevated: #1E1E26` — Dropdowns, tooltips
- `text-primary: #F4F4F5` — Headings, primary content
- `text-secondary: #B4B4BB` — Metadata, labels
- `accent-primary: #F59E0B` — Active tab, focus rings, brand emphasis
- `accent-hover: #D97706` — Hover states
- `border-default: #27272A` — Card edges, dividers

### Focus Ring Spec (WCAG 2.1 AA)

```css
/* Add to globals.css */
*:focus-visible {
  outline: 2px solid var(--color-accent-primary);
  outline-offset: 2px;
}
```

### Tauri System Tray API (v2)

Tauri 2.0 uses `tauri::tray::TrayIconBuilder`. Key imports:
```rust
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::image::Image;
```

Register in `lib.rs` builder chain:
```rust
.setup(|app| {
    // ... existing database setup ...
    tray::setup_tray(app)?;
    Ok(())
})
```

### Previous Story Learnings (from Epic 1)

- All shell components use barrel exports via `index.ts` — maintain this pattern
- `PlaceholderPage` is already exported from shell — keep for Coaching/Telemetry tabs
- React Router uses `createBrowserRouter` + `RouterProvider` — NOT HashRouter
- `main.tsx` wraps with `QueryClientProvider` — keep that intact
- Cross-story compile issues happened in Epic 1 when shared files were modified — be careful with `lib.rs` changes

### Project Structure Notes

- Shell components: `src/features/shell/components/`
- Tray backend: `src-tauri/src/tray.rs` (new file)
- Tray frontend: `src/features/tray/` (new directory, if frontend integration needed)
- Tray icons: `src-tauri/icons/tray/` (new directory for tray-specific icons)
- Global styles: `src/styles/globals.css` (add focus-visible styles here)
- UI state: `src/state/ui-state.ts` (new file for modal/focus state management)

### References

- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Epic 2 Story 2.1]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Tab Navigation]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#System Tray Icon State System]
- [Source: _bmad-output/planning-artifacts/architecture.md#Frontend Architecture]
- [Source: _bmad-output/planning-artifacts/architecture.md#Project Structure]
- [Source: _bmad-output/implementation-artifacts/1-2-desktop-application-runs-locally.md]

## Dev Agent Record

### Agent Model Used
Claude Opus 4.6 (claude-opus-4-6) as dev-1

### Debug Log References
- Initial `cargo build` failed due to Tauri 2.10.2 API differences (`Image::from_bytes` not available, `TrayIconBuilder::new` takes no args, `Emitter` trait not in scope). Fixed by using `Image::new` with raw RGBA bytes and correct imports.
- `tokio` crate missing for dev-2's `notifications.rs` -- added `tokio = { version = "1", features = ["time"] }` to unblock shared build.

### Completion Notes List
- Task 1: All shell components verified as matching spec from Story 1.2. No changes needed.
- Task 2: Rewrote `tab-bar.tsx` from NavLink-based to button-based tabs with full ARIA tablist pattern (role, aria-selected, aria-controls, tabIndex management, arrow/Home/End key navigation). Updated `app-shell.tsx` with tabpanel role, dynamic id/aria-labelledby, and Escape key handler.
- Task 3: Created `tray.rs` with idle tray icon setup using raw RGBA bytes. Dev-2 subsequently expanded this file significantly for Story 2.2 (TrayManager, multiple states, context menu, click handling). Both implementations coexist correctly.
- Task 4: Verified smooth transitions -- `transition-colors duration-150` already present, fixed layout prevents layout shift.
- Task 5: All build checks pass (cargo build, cargo test 77/77, npm run build, npm run typecheck).

### Change Log
1. `src/styles/globals.css` -- Added global `*:focus-visible` styles (2px solid amber, 2px offset) for WCAG 2.1 AA compliance
2. `src/features/shell/components/tab-bar.tsx` -- Rewrote from NavLink to button-based tabs with full ARIA tablist pattern, keyboard navigation (Arrow keys, Home/End), tabIndex management
3. `src/features/shell/components/app-shell.tsx` -- Added tabpanel role with dynamic id/aria-labelledby, Escape key handler, outline-none for focus ring suppression on panel
4. `src-tauri/src/tray.rs` -- Created initial tray icon setup (idle state, gray circle, tooltip)
5. `src-tauri/src/lib.rs` -- Added `mod tray;` declaration and `tray::setup_tray()` call in setup block
6. `src-tauri/Cargo.toml` -- Added `tokio` dependency (needed by notifications.rs from dev-2)
7. `src-tauri/capabilities/default.json` -- Added `core:event:default` permission for backend event emission
8. `src-tauri/icons/tray/tray-idle.rgba` -- 32x32 gray circle raw RGBA tray icon asset
9. `src-tauri/icons/tray/tray-idle.png` -- 32x32 gray circle PNG tray icon asset

### File List
- `src/styles/globals.css`
- `src/features/shell/components/tab-bar.tsx`
- `src/features/shell/components/app-shell.tsx`
- `src-tauri/src/tray.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/capabilities/default.json`
- `src-tauri/icons/tray/tray-idle.rgba`
- `src-tauri/icons/tray/tray-idle.png`

## AI Review (reviewer-1)

### [AI-Review][MEDIUM] nav element with role="tablist" creates conflicting semantics
**File:** `src/features/shell/components/tab-bar.tsx:62-66`
The `<nav>` element has an implicit `navigation` landmark role. Adding `role="tablist"` overrides it, which creates semantic confusion for screen readers. The WAI-ARIA Authoring Practices recommend using a `<div>` for the tablist container. Change `<nav role="tablist" ...>` to `<div role="tablist" ...>`.

### [AI-Review][MEDIUM] aria-controls references non-existent panel IDs for inactive tabs
**File:** `src/features/shell/components/tab-bar.tsx:77` and `src/features/shell/components/app-shell.tsx:46-48`
Each tab button has `aria-controls` pointing to its corresponding panel ID (e.g., `tabpanel-coaching`), but the `<main>` element in AppShell only ever has a single `id` set to the *active* panel's ID. When tab "Summary" is active, the DOM has `id="tabpanel-summary"` on `<main>`, meaning `aria-controls="tabpanel-coaching"` on the Coaching tab points to a non-existent element. Assistive technology may struggle with this. Two options: (1) only set `aria-controls` on the active tab, or (2) keep all three panel IDs valid by wrapping each route's content in its own div with the correct panel ID (though this would require restructuring how Outlet works).

### [AI-Review][MEDIUM] Mutex unwrap() calls in TrayManager can panic on poisoned locks
**File:** `src-tauri/src/tray.rs:141,147,151,198,219`
All `Mutex::lock().unwrap()` calls will panic if a thread panics while holding the lock. While unlikely in a single-threaded desktop app context, the idiomatic approach is `lock().unwrap_or_else(|e| e.into_inner())` or explicit error handling to be resilient against poisoned mutexes.

### [AI-Review][MEDIUM] Event payloads missing architecture-mandated fields (type, timestamp, version)
**File:** `src-tauri/src/events.rs:20-26` and `src-tauri/src/tray.rs:172,292,309`
The architecture doc specifies that event payloads should include `type`, `timestamp`, and `version` fields (`architecture.md` "Event System Patterns"). The `TrayStatusPayload` struct and the `tray:navigate-to-debrief` event do not include these fields. This is a pattern consistency issue that should be addressed project-wide, but noted here since these are the first events being emitted.

### [AI-Review][LOW] Story file lists tray-idle.rgba/tray-idle.png but code uses idle.png
**File:** Story change log items 8-9 vs `src-tauri/src/tray.rs:225`
The story's change log references `tray-idle.rgba` and `tray-idle.png` as created assets, but the actual code uses `idle.png` (as well as `recording.png`, `ready.png`, `error.png` from dev-2's Story 2.2 work). The `tray-idle.*` files still exist on disk but are unused. Minor doc discrepancy; the unused files (`tray-idle.rgba`, `tray-idle.png`) should be cleaned up.

### [AI-Review][LOW] Focus-to-panel useEffect fires on initial mount
**File:** `src/features/shell/components/tab-bar.tsx:54-59`
The `useEffect` that moves focus to the tab panel runs whenever `resolvedIndex` changes, including on initial page load. This could cause the page to focus on the main content area immediately, skipping over the tab bar. In practice this is unlikely to cause user-visible issues since the guard condition checks if a tab ref is focused, but the intent could be clearer with an explicit "did the user trigger this" check.

### [AI-Review][LOW] Global focus-visible styles may need additional contrast in some contexts
**File:** `src/styles/globals.css:28-31`
The `*:focus-visible` rule applies a 2px amber outline universally. This meets WCAG 2.1 AA for most dark backgrounds in the app (amber on #0A0A0F has high contrast). However, on any element with an amber/yellow background, the focus ring would be invisible. This is not currently an issue given the dark theme, but worth noting for future reference if lighter surfaces are introduced.

### Summary
- **HIGH:** 0 issues
- **MEDIUM:** 4 issues (nav/tablist semantics, aria-controls dangling references, Mutex unwrap, missing event payload fields)
- **LOW:** 3 issues (unused icon assets, focus-on-mount timing, focus ring contrast edge case)
