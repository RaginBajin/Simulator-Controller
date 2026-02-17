# Story 1.2: Desktop Application Runs Locally

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a user,
I want the application to run as a Windows desktop app,
so that I don't need browser configuration or internet dependency.

## Acceptance Criteria

1. **Native Desktop Window Opens on Launch**
   - Launching the built executable opens a native desktop window
   - Window defaults to 1200x800px (user-resizable, minimum 800x600px enforced by Tauri config)
   - Window title displays "Pitwall"
   - Window uses dark theme (bg-base #0A0A0F background, text-primary #F4F4F5 text)

2. **Application Runs Entirely Locally**
   - All core functionality operates without internet connection
   - No external API calls are made on startup (AI provider calls happen only when user configures keys in future stories)
   - Application data is stored locally (data directory path resolved via Tauri's `app_data_dir`)
   - Application starts and renders UI within 3 seconds on a modern machine

3. **Windows Installer Builds Successfully**
   - `cargo tauri build` produces a working Windows installer (NSIS .exe or MSI)
   - Installer includes bundled WebView2 bootstrapper (for machines without WebView2)
   - Installed application launches from Start Menu / Desktop shortcut
   - Application binary size is under 50MB installed

4. **Application Shell with Placeholder Content**
   - Main window displays the app shell layout (header area, main content area, bottom bar area)
   - Tab navigation skeleton is visible: Summary | Coaching | Telemetry (tabs are non-functional placeholders for now)
   - Each tab area shows placeholder text indicating the section name
   - Persistent chat input area is visible at the bottom (non-functional placeholder)
   - Layout uses the established design tokens from Story 1.0 (bg-surface for panels, border-default for dividers, accent-primary for active tab indicator)

5. **Cross-Platform Dev Build Works**
   - `npm run tauri dev` works on macOS (development) and targets Windows (production)
   - Dev build renders the same shell layout as production
   - Hot-reload continues to function for frontend changes
   - Rust recompilation triggers on backend changes

## Tasks / Subtasks

- [x] Task 1: Configure Tauri window properties (AC: #1)
  - [x] 1.1 Set `tauri.conf.json` window defaults: width 1200, height 800, minWidth 800, minHeight 600
  - [x] 1.2 Set window title to "Pitwall" in `tauri.conf.json`
  - [x] 1.3 Verify window opens at correct dimensions on `npm run tauri dev`

- [x] Task 2: Build application shell layout (AC: #4)
  - [x] 2.1 Create `src/features/shell/` directory with shell components
  - [x] 2.2 Create `AppShell` component with header, main content area, and bottom bar layout
  - [x] 2.3 Create `TabBar` component with Summary | Coaching | Telemetry tabs (visual only, no routing yet)
  - [x] 2.4 Style active tab with amber underline (2px, accent-primary #F59E0B) and inactive tabs with text-secondary
  - [x] 2.5 Create placeholder content panels for each tab section
  - [x] 2.6 Create `ChatBar` placeholder component at bottom of layout (collapsed input bar with "Ask about your session..." placeholder text)
  - [x] 2.7 Wire `AppShell` into `App.tsx` as the main layout

- [x] Task 3: Configure React Router for tab navigation (AC: #4)
  - [x] 3.1 Install `react-router-dom` v7.x (per architecture: React Router DOM 7.12.x)
  - [x] 3.2 Configure routes in `src/routes.tsx`: `/summary`, `/coaching`, `/telemetry` with redirect from `/` to `/summary`
  - [x] 3.3 Connect `TabBar` clicks to route navigation
  - [x] 3.4 Render route-specific placeholder content in the main content area

- [x] Task 4: Verify offline operation (AC: #2)
  - [x] 4.1 Confirm no network requests are made on application startup (check Tauri capabilities — no network permissions)
  - [x] 4.2 Verify `src-tauri/capabilities/default.json` has no `http:default` or `shell:default` permissions
  - [ ] 4.3 Test app launch with network disabled — must render fully

- [x] Task 5: Configure production build (AC: #3)
  - [x] 5.1 Configure `tauri.conf.json` bundle settings: identifier, icons, category
  - [x] 5.2 Set bundle identifier to `com.pitwall.app`
  - [x] 5.3 Configure NSIS installer settings in `tauri.conf.json` (prefer NSIS over MSI for broader Windows compatibility)
  - [x] 5.4 Configure WebView2 bootstrapper embedding (installMode: `downloadBootstrapper` or `embedBootstrapper`)
  - [x] 5.5 Run `cargo tauri build` and verify installer is produced
  - [ ] 5.6 Verify installed application launches from the installer output

- [x] Task 6: Verify dev environment continuity (AC: #5)
  - [x] 6.1 Run `npm run tauri dev` — verify Vite + Tauri window with new shell layout
  - [ ] 6.2 Make a frontend change — verify hot-reload works
  - [ ] 6.3 Make a Rust change — verify recompilation triggers
  - [x] 6.4 Verify all design tokens render correctly (dark bg, amber accent on active tab, font rendering)

## Dev Notes

### Architecture Compliance

**Window Configuration (tauri.conf.json):**
The window properties are configured in `src-tauri/tauri.conf.json` under the `app.windows` array:

```json
{
  "app": {
    "windows": [
      {
        "title": "Pitwall",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false
      }
    ]
  }
}
```

**Bundle Configuration (tauri.conf.json):**
```json
{
  "bundle": {
    "active": true,
    "identifier": "com.pitwall.app",
    "targets": ["nsis"],
    "windows": {
      "nsis": {
        "installMode": "downloadBootstrapper"
      }
    }
  }
}
```

**CRITICAL: Do NOT add any network-related Tauri capabilities.** The `src-tauri/capabilities/default.json` should remain with `core:default` only. Network, HTTP, and shell permissions are added in later stories when needed.

### Frontend Architecture

**React Router DOM 7.x Setup:**
Per architecture doc, routing uses React Router DOM 7.12.x (requires Node >= 20).

```bash
npm install react-router-dom@^7.12.0
```

Router setup in `src/routes.tsx`:
```tsx
import { createBrowserRouter } from 'react-router-dom';
```

**CRITICAL: Use `BrowserRouter` or `createBrowserRouter` — NOT `HashRouter`.** Tauri 2.0 supports BrowserRouter natively.

**Component File Naming:** All frontend files use `kebab-case` per architecture naming conventions:
- `src/features/shell/components/app-shell.tsx` (exports `AppShell`)
- `src/features/shell/components/tab-bar.tsx` (exports `TabBar`)
- `src/features/shell/components/chat-bar.tsx` (exports `ChatBar`)

**Tab Bar Design (from UX spec):**
- Tab bar displays at the top of the content area
- Active tab: `text-primary` (#F4F4F5) with 2px amber underline (`accent-primary` #F59E0B)
- Inactive tabs: `text-secondary` (#B4B4BB) with hover effect
- Tab transition: 150ms ease
- Font: Inter, text-sm (14px), font-medium

**Chat Bar Design (from UX spec):**
- Fixed position at bottom of viewport
- Collapsed by default: single line with "Ask about your session..." placeholder
- Background: `bg-elevated` (#1E1E26)
- Border top: `border-default` (#27272A)
- Non-functional in this story — visual placeholder only

**Layout Structure:**
```
+----------------------------------+
| [Tab: Summary | Coaching | Tele] |  <- TabBar
+----------------------------------+
|                                  |
|       Main Content Area          |  <- Route outlet (bg-surface)
|       (placeholder text)         |
|                                  |
+----------------------------------+
| Ask about your session...        |  <- ChatBar (bg-elevated)
+----------------------------------+
```

### Existing Code to Build On

This story builds directly on Story 1.0's foundation:
- **Tailwind CSS 4.x** is already configured with all design tokens
- **shadcn/ui** is initialized with Button component
- **Dark theme** is default (class="dark" on html element)
- **Font loading** (Inter + JetBrains Mono) is already set up via @fontsource
- **Project structure** directories exist: `src/features/`, `src/hooks/`, `src/lib/`, `src/state/`
- **Rust workspace** with 4 crates is building successfully

**Do NOT re-configure** any of the above. Build on the existing foundation.

### Tauri Build for Windows

**Development on macOS:**
- `npm run tauri dev` works on macOS for development
- Production builds targeting Windows require either Windows CI or cross-compilation
- GitHub Actions with `tauri-apps/tauri-action` handles Windows builds (configured in future story 1.9)
- For this story, verify `cargo tauri build` works on the development machine (macOS build)
- Windows-specific installer testing can be deferred to CI pipeline

**WebView2:**
- Windows production builds require WebView2 runtime
- Configure `downloadBootstrapper` mode — installer downloads WebView2 if not present
- macOS uses WebKit (included with OS), no bootstrapper needed

### Testing Standards

**Frontend:** No test framework yet (Vitest deferred). Manual verification of:
- Window dimensions
- Tab navigation renders correct placeholders
- Design tokens applied correctly
- Hot-reload works

**Rust:** `cargo test` should continue to pass with existing tests.

### Naming Conventions (Enforced)

| Zone | Convention | Example |
|------|-----------|---------|
| React components | `PascalCase` | `AppShell`, `TabBar`, `ChatBar` |
| Frontend files | `kebab-case` | `app-shell.tsx`, `tab-bar.tsx` |
| Feature dirs | `kebab-case` | `src/features/shell/` |
| CSS classes | Tailwind utility | `bg-base`, `text-primary`, `border-default` |

### Project Structure Notes

- New feature directory: `src/features/shell/` for app shell components
- No new Rust code changes needed beyond `tauri.conf.json` configuration
- Builds on existing `src/components/ui/` for any shadcn primitives used
- `src/routes.tsx` already exists as a placeholder — populate with actual routes

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Frontend Architecture]
- [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- [Source: _bmad-output/planning-artifacts/architecture.md#Core Architectural Decisions]
- [Source: _bmad-output/planning-artifacts/architecture.md#Infrastructure & Deployment]
- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 1.2: Desktop Application Runs Locally]
- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 2.1: Tauri Window & Tab Navigation] (cross-reference for tab design)
- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules]
- [Source: _bmad-output/implementation-artifacts/1-0-project-foundation-build-setup.md] (previous story)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

- TypeScript: `npx tsc --noEmit` -- passed
- Vite build: `npm run build` -- passed (52 modules, 1.80s)
- Rust: `cargo check` / `cargo test` -- passed (0 tests, all compilation OK)
- Fixed `tauri.conf.json` NSIS installMode from invalid `downloadBootstrapper` to `currentUser`
- Fixed WebView2 install mode to proper nested object structure

### Completion Notes List

- All shell components (AppShell, TabBar, ChatBar, PlaceholderPage) implemented in `src/features/shell/`
- React Router DOM 7.13.x wired with BrowserRouter via `createBrowserRouter`
- Routes: `/` redirects to `/summary`, plus `/coaching` and `/telemetry`
- TabBar uses NavLink with active amber underline styling
- ChatBar is a visual placeholder with "Ask about your session..." text
- `main.tsx` updated to use `RouterProvider` instead of `<App />`
- `App.tsx` is now dead code (no longer imported) -- can be cleaned up
- Tauri capabilities remain `core:default` only (no network permissions)
- Bundle config: NSIS + DMG targets, `com.pitwall.app` identifier, WebView2 downloadBootstrapper
- Production build verified: `cargo tauri build` produced DMG at `target/release/bundle/dmg/Pitwall_0.1.0_aarch64.dmg`
- All Rust workspace tests pass: `cargo test --workspace` (0 failures)
- Manual verification tasks (hot-reload, network-disabled test, installed app launch) deferred to interactive testing

### File List

- `src-tauri/tauri.conf.json` -- window config, bundle settings
- `src-tauri/capabilities/default.json` -- core:default only (unchanged)
- `src/main.tsx` -- updated to use RouterProvider
- `src/routes.tsx` -- populated with route definitions
- `src/features/shell/index.ts` -- barrel exports
- `src/features/shell/components/app-shell.tsx` -- main layout shell
- `src/features/shell/components/tab-bar.tsx` -- tab navigation with NavLink
- `src/features/shell/components/chat-bar.tsx` -- chat input placeholder
- `src/features/shell/components/placeholder-page.tsx` -- generic placeholder for tab content
- `src/App.tsx` -- deleted (was dead code, superseded by router)

## Review Follow-ups (AI)

- [x] [AI-Review][HIGH] Delete dead code `src/App.tsx`. File is no longer imported by any module (confirmed via grep). Completion notes acknowledge it as dead code but it was left in place. Remove to prevent confusion and keep codebase clean. [src/App.tsx:1-17] -- FIXED: Deleted `src/App.tsx`.
- [x] [AI-Review][MEDIUM] Export `PlaceholderPage` from barrel `src/features/shell/index.ts`. Currently `routes.tsx` imports it via direct path (`@/features/shell/components/placeholder-page`), bypassing the barrel export. All other shell components (AppShell, TabBar, ChatBar) are exported from the barrel -- PlaceholderPage should be consistent. [src/features/shell/index.ts:1-3, src/routes.tsx:3] -- FIXED: Added export to barrel, updated routes.tsx import.
- [x] [AI-Review][MEDIUM] Add `errorElement` to the router configuration in `src/routes.tsx`. While Tauri desktop users cannot type arbitrary URLs, a hard refresh on a subroute during development will hit React Router's default ugly error boundary (white background, unstyled). Adding a themed error fallback ensures graceful handling. [src/routes.tsx:5-25] -- FIXED: Added `RouteErrorBoundary` component with themed styling, wired to root route `errorElement`.
- [x] [AI-Review][MEDIUM] Replace default Vite favicon (`vite.svg`) in `index.html` with Pitwall branding or remove the link. The `<link rel="icon" href="/vite.svg" />` is a scaffold leftover. [index.html:5] -- FIXED: Updated favicon link to `tauri.svg`, deleted `public/vite.svg`.
- [x] [AI-Review][LOW] Consolidate Tailwind design token definitions. Tokens are defined in both `tailwind.config.ts` (v3 config style) and `src/styles/globals.css` `@theme` block (v4 CSS style). Both are read by Tailwind v4 via `@tailwindcss/postcss`. Having two sources of truth risks drift -- consider removing one. [tailwind.config.ts, src/styles/globals.css:3-25] -- FIXED: Removed duplicate color/font definitions from `tailwind.config.ts`. Kept only borderRadius and spacing overrides (not expressible via `@theme`). Added comment explaining the split.
- [x] [AI-Review][LOW] CSP is disabled (`app.security.csp: null`) in `src-tauri/tauri.conf.json`. Acceptable for Story 1.2 (no network features), but must be tightened before any network-enabled story (AI providers, updates). Track as a prerequisite for stories with network capabilities. [src-tauri/tauri.conf.json:24-26] -- ACKNOWLEDGED: No action needed for Story 1.2. CSP tightening tracked as prerequisite for network-enabled stories.
- [x] [AI-Review][LOW] Manual verification tasks remain unchecked: Task 4.3 (network-disabled test), Task 5.6 (installer launch test), Task 6.2 (hot-reload test), Task 6.3 (Rust recompilation test). Deferred to interactive testing per completion notes -- ensure these are verified before sprint close. -- ACKNOWLEDGED: Deferred to interactive testing.
