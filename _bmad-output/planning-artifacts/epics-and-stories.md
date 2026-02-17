---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
  - _bmad-output/planning-artifacts/ux-design-specification.md
  - _bmad-output/planning-artifacts/prd-validation-report.md
workflowType: 'epic-creation'
project_name: 'AI Race Team'
date: '2026-02-02'
---

# AI Race Team - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for **AI Race Team** (Simulator-Controller), decomposing requirements from the PRD, Architecture, UX Design, and validation report into implementable stories organized by user value delivery.

**Project Summary:** AI Race Team is a Tauri 2.0 desktop application that automatically captures iRacing telemetry and delivers AI-powered coaching debriefs. It replaces fragmented manual workflows with a unified loop: capture → pre-processing → AI coaching → visualization → conversation → sharing. Built with a Rust core engine (telemetry capture, data storage, AI integration) and React/TypeScript frontend (debrief interface, charts, chat), the application targets Windows-first (iRacing is Windows-only) with MVP focused on single-driver local analysis.

**Development Phase:** All requirements are tagged [MVP] for initial release. The epic organization follows natural development dependency order while maintaining focus on incremental user value delivery.

## Requirements Inventory

### Functional Requirements

**Total Functional Requirements:** 43 FRs (FR1-FR41 with FR1 split into FR1/FR1a/FR1b)

**Note:** Complete FR definitions are documented in the PRD (`prd.md`). This document references FR numbers for traceability but does not redefine them. PRD is the single source of truth for all Functional Requirements.

**FR Categories (see PRD for detailed definitions):**
- **Telemetry Capture & Data Management:** FR1, FR1a, FR1b, FR2, FR3, FR4, FR5, FR6, FR7, FR8, FR9, FR10 (12 FRs)
- **AI Coaching & Analysis:** FR11, FR12, FR13, FR14, FR15, FR16, FR17, FR18, FR19 (9 FRs)
- **AI Provider Management:** FR20, FR21, FR22, FR23, FR24 (5 FRs)
- **Visualization & Debrief:** FR25, FR26, FR27, FR28, FR29, FR30 (6 FRs)
- **Session History & Progress Tracking:** FR31, FR32, FR33 (3 FRs)
- **Desktop Application & System Integration:** FR34, FR35, FR36, FR37, FR38, FR39, FR40, FR41 (8 FRs)

### NonFunctional Requirements

**Performance (8 NFRs):**
- NFR1: Telemetry capture resource consumption while iRacing is running
  - Target: <2% total system CPU, <200MB RSS
  - Measurement: Built-in resource monitor logging
  - Type: Instrumented
- NFR2: Telemetry capture latency (time from IRSDK update to internal buffer write)
  - Target: <10ms p99
  - Measurement: Internal timestamp logging
  - Type: Instrumented
- NFR3: Session pre-processing pipeline latency (session end → pre-processed data ready)
  - Target: <15 seconds for 60-minute sessions with 35 telemetry channels
  - Measurement: Internal pipeline timing logs
  - Type: Instrumented
- NFR4: AI coaching debrief generation latency (session pre-processed → full coaching text available)
  - Target: <60 seconds for 60-minute sessions (depends on API provider response time)
  - Measurement: End-to-end timing from session end to debrief ready
  - Type: Instrumented
- NFR5: Session list query performance
  - Target: <100ms to return 1000 sessions sorted by date
  - Measurement: Database query timing
  - Type: Instrumented
- NFR5a: Full session data retrieval performance (metadata + telemetry Parquet load)
  - Target: <200ms for 60-minute sessions
  - Measurement: End-to-end load timing
  - Type: Instrumented
- NFR6: Telemetry chart rendering performance (1D lap-distance chart with 60Hz data)
  - Target: 60fps smooth pan/zoom interaction
  - Measurement: Browser DevTools frame timing
  - Type: Tested
- NFR6a: AI coaching streaming responsiveness (first token visible)
  - Target: <5 seconds from analysis start to first coaching token appears
  - Measurement: Frontend timestamp logging
  - Type: Instrumented

**Reliability (5 NFRs):**
- NFR7: Telemetry capture reliability during session
  - Target: <1% data loss per session (allowing for brief IRSDK connection hiccups)
  - Measurement: Compare captured sample count vs expected sample count at 60Hz
  - Type: Validated
- NFR8: Data persistence reliability (no silent data loss on write)
  - Target: 100% of captured sessions either successfully written or explicitly error-reported
  - Measurement: Post-write validation, crash recovery testing
  - Type: Validated
- NFR9: Application crash recovery (preserve in-flight data)
  - Target: 100% of captured telemetry preserved up to crash timestamp
  - Measurement: Kill process during active capture, verify session recovery on restart
  - Type: Tested
- NFR10: IRSDK connection recovery (handle iRacing restart mid-capture)
  - Target: Auto-reconnect within 5 seconds of IRSDK availability
  - Measurement: Manual IRSDK disconnect/reconnect testing
  - Type: Tested
- NFR11: API provider failure graceful degradation
  - Target: If AI analysis fails, local stats + telemetry remain accessible
  - Measurement: Manual API key revocation, network disconnect testing
  - Type: Tested

**Data Integrity (5 NFRs):**
- NFR12: Telemetry data accuracy (no value corruption during capture/write)
  - Target: Checksums validate on 100% of Parquet file reads
  - Measurement: SHA-256 checksum on write, validate on read
  - Type: Validated
- NFR13: Lap distance monotonicity (within-lap distance values always increase)
  - Target: 100% of valid laps pass monotonicity check
  - Measurement: Post-processing validation
  - Type: Validated
- NFR14: Session metadata consistency (SQLite foreign keys enforced)
  - Target: 0 orphaned lap records (every lap references valid session)
  - Measurement: Database integrity check on startup
  - Type: Validated
- NFR15: Telemetry channel value range validation
  - Target: 100% of captured values within physically plausible ranges (e.g., brake 0-100%, speed 0-400kph)
  - Measurement: Range check during pre-processing
  - Type: Validated
- NFR16: Corner segmentation determinism (same session → same corner boundaries on re-process)
  - Target: 100% reproducible corner segmentation for identical telemetry input
  - Measurement: Re-process same session, compare corner boundaries
  - Type: Tested

**Security (4 NFRs):**
- NFR17: API credential storage security
  - Target: API keys stored in OS credential store (Windows Credential Manager via keyring-rs), never in plaintext config
  - Measurement: Manual verification of storage location, code review
  - Type: Validated
- NFR18: API key transmission security
  - Target: API keys only transmitted over HTTPS to provider endpoints
  - Measurement: Network traffic inspection, code review
  - Type: Validated
- NFR19: No telemetry data transmission without user action (privacy by default)
  - Target: Telemetry data never leaves local machine unless user explicitly exports or enables future sharing features
  - Measurement: Network traffic inspection during normal operation
  - Type: Validated
- NFR20: Secure IPC between Rust backend and frontend
  - Target: All IPC commands validate input types and reject malformed requests
  - Measurement: Fuzz testing of IPC commands, code review
  - Type: Tested

**Integration (4 NFRs):**
- NFR21: iRacing IRSDK compatibility
  - Target: Works with iRacing 2024 Season 1+ (IRSDK version 1.13+)
  - Measurement: Manual testing with supported iRacing versions
  - Type: Tested
- NFR22: Windows version compatibility
  - Target: Works on Windows 10 (1809+) and Windows 11
  - Measurement: Manual testing on target Windows versions
  - Type: Tested
- NFR23: Claude API compatibility
  - Target: Works with Claude 3.5 Sonnet API (claude-3-5-sonnet-20241022)
  - Measurement: Manual testing with live API
  - Type: Tested
- NFR24: OpenAI API compatibility
  - Target: Works with GPT-4 Turbo API (gpt-4-turbo)
  - Measurement: Manual testing with live API
  - Type: Tested

**Usability (4 NFRs):**
- NFR25: First-time onboarding completion time
  - Target: <3 minutes from download to ready-to-record (install → auto-detect iRacing → paste API key or skip)
  - Measurement: User testing with 5 newcomers, median time
  - Type: Tested
- NFR26: Debrief headline visibility (time to first insight)
  - Target: <5 seconds from window open to headline coaching text visible (local stats instant, AI headline streams within 5s)
  - Measurement: Frontend timestamp logging
  - Type: Instrumented
- NFR27: Chart interaction responsiveness (corner-zoom, lap selection)
  - Target: <300ms from click to zoom/update complete
  - Measurement: Browser DevTools interaction timing
  - Type: Tested
- NFR28: System tray notification clarity (user understands what to do)
  - Target: >90% of beta testers correctly identify notification intent and click to open debrief
  - Measurement: User testing with 10 beta testers
  - Type: Tested

**Total Non-Functional Requirements:** 30 NFRs (including NFR5a and NFR6a)

### Additional Requirements

**Architecture Requirements:**

**Technology Stack:**
- Core Engine: Rust 2021 edition
- Desktop Shell: Tauri 2.0 (CLI v2.6.0, no earlier versions)
- Frontend: React 19+ with TypeScript 5.x (strict mode), Vite 6.x
- Styling: Tailwind CSS 4.x + shadcn/ui (Radix UI primitives)
- Database: SQLite via SQLx or rusqlite
- Telemetry Storage: Parquet via arrow-rs/parquet crate
- Charting: uPlot (wrapped behind TelemetryChartProvider abstraction per ADR-ARCH-2)
- State Management: TanStack Query v5 + React Context (upgrade to Zustand only if complexity warrants per ADR-ARCH-3)
- Routing: React Router v7 or TanStack Router
- Icons: Lucide React
- Fonts: Inter (UI text) + JetBrains Mono (data/telemetry values)
- Security: keyring-rs for API credential storage in OS credential store
- Logging: tracing crate (Rust), console (frontend development)

**Cargo Workspace Structure (4 Crates):**
- src-tauri (binary) → depends on: telemetry-engine, ai-provider, storage
- telemetry-engine (library) → depends on: storage
- ai-provider (library) → depends on: storage
- storage (library) → leaf crate, no dependencies

**Tauri IPC Commands (12 Commands):**
- get_sessions, get_session_data, get_chart_data, get_lap_data
- start_capture, stop_capture
- analyze_session, send_chat_message
- validate_api_key, get_provider_status
- get_settings, update_settings

**Tauri Events (4 Namespaces):**
- session: state-changed, lap-completed, capture-started, capture-stopped
- ai: token-received, analysis-complete, analysis-error, provider-status
- capture: progress, error, irsdk-connected, irsdk-disconnected
- app: tray-action, update-available, settings-changed

**Critical Architecture Decision Records:**
- **ADR-ARCH-1: Rust-Side Data Windowing** — Frontend requests distance ranges, Rust returns windowed slices, keeps IPC payloads <500KB
- **ADR-ARCH-2: Charting Abstraction Layer** — Wrap uPlot behind TelemetryChartProvider interface for swappability
- **ADR-ARCH-3: Progressive State Management** — Start with TanStack Query + React Context, upgrade to Zustand only if needed

**Naming Conventions (4 Zones):**
- Rust: snake_case functions, PascalCase types, SCREAMING_SNAKE constants
- TypeScript: camelCase functions, PascalCase types
- IPC: snake_case commands, kebab-case events with namespace (e.g., `session:lap-completed`)
- SQLite: snake_case tables and columns

**UX Requirements:**

**9 Custom Components to Build:**
1. **SessionCard** — Timeline entry for session history with track, car, date, lap count, best lap, AI headline
2. **DebriefHeader / Hero Block** — Screenshot-native summary block with session stats and top coaching insight
3. **TelemetryChart (1D Lap-Distance)** — Brake/throttle traces vs lap distance with corner zones and multi-lap overlay
4. **CornerAnnotationOverlay** — AI coaching callouts overlaid on telemetry chart
5. **CoachingPanel** — Structured AI coaching narrative with clickable corner references
6. **CornerSidebar** — Quick navigation panel listing all corners with status indicators
7. **Progress Line Graph** — Session-over-session best lap time trend (retention feature for newcomers)
8. **Persistent Chat Input** — Always-available conversational follow-up input at bottom of view
9. **Tray Status Indicator** — Native system tray icon with dynamic state (idle/recording/processing/ready/error)

**Design System Foundation:**
- Tailwind CSS 4.x + shadcn/ui (Radix UI primitives)
- Independent charting layer (uPlot or custom Canvas)
- Dark-first palette (no light mode at MVP)
- Inter typeface (UI) + JetBrains Mono (data)

**Visual Design Tokens:**
- Background: bg-base (#0A0A0F), bg-surface (#141419), bg-elevated (#1E1E26)
- Text: text-primary (#F4F4F5), text-secondary (#B4B4BB), text-muted (#71717A)
- Accent: accent-primary (#F59E0B amber), accent-hover (#D97706)
- Telemetry: trace-best (#22C55E green), trace-average (#71717A gray), trace-regression (#EF4444 red)
- Typography: text-3xl (30px) down to text-xs (12px), font weights normal/medium/semibold/bold
- Spacing: 4px base unit, space-1 (4px) through space-12 (48px)
- Border radius: radius-sm (4px), radius-md (6px), radius-lg (8px)

**5 User Journey Flows:**
- J1: Joe — The Happy Path Debrief (system tray notification → Summary tab → Coaching tab → Telemetry corner-zoom → chat follow-up)
- J2: Marcus — First-Time Onboarding (install → auto-detect iRacing → API key setup or skip → first race → plain English coaching → progress tracking)
- J3: Joe — The Bad Session (Edge Case Recovery: telemetry gaps, tire degradation, spin → graceful error handling → coaching from failures)
- Plus: System tray icon state system (idle/recording/processing/ready/error)
- Plus: Adaptive patterns (sprint vs endurance layout, coaching language depth, chat visibility)

**Accessibility Requirements:**
- WCAG 2.1 Level AA compliance target
- Contrast ratios: text-primary (18:1), text-secondary (5.5:1), text-muted (4.6:1)
- Keyboard navigation for all interactive elements
- Screen reader support with ARIA labels and live regions
- Respect prefers-reduced-motion
- Focus indicators: 2px solid amber with 2px offset

**Responsive Design:**
- Container-first approach using CSS container queries
- Three breakpoints: compact (<600px), standard (600-1000px), wide (>1000px)
- Window minimum: 800×600px enforced by Tauri
- Cross-platform WebView: macOS WebKit + Windows WebView2

**UX Patterns:**
- Tab-based navigation: Summary | Coaching | Telemetry
- Two-phase loading: local stats instant, AI streams progressively
- Corner-zoom navigation: click corner → chart centers with ~200m context
- Cursor sync across charts
- Persistent chat input (collapsible)
- Screenshot-native Summary tab design
- Adaptive layout: sprint (hero cards) vs endurance (stint breakdown)
- Graceful degradation: Full AI → AI slow → AI unavailable → No API key → Capture failed

### FR Coverage Map

| Epic | FR Count | Functional Requirements Covered | Story Count |
|------|----------|--------------------------------|-------------|
| **Epic 1: Data Storage** | 6 | FR6, FR7, FR10, FR31, FR32, FR33 | 8 stories |
| **Epic 2: Desktop Foundation** | 3 | FR34, FR35, FR36 | 3 stories |
| **Epic 3: Telemetry Capture** | 11 | FR1, FR1a, FR1b, FR2, FR3, FR4, FR5, FR8, FR9, FR40, FR41 | 7 stories |
| **Epic 4: AI Coaching** | 14 | FR11-FR24 | 8 stories |
| **Epic 5: Debrief Interface** | 7 | FR25, FR26, FR27, FR28, FR29, FR30, FR40 | 6 stories |
| **Epic 6: Desktop Integration & Settings** | 4 | FR37, FR38, FR39, FR41 | 6 stories |
| **TOTAL** | **43** | **All FRs (FR1-FR41) mapped** | **38 stories** |

**Verification:**
- ✅ All 43 FRs covered (100% coverage achieved)
- ✅ All 6 critical/major implementation readiness issues resolved (2026-02-05)
- ✅ Epic independence restored via event-based decoupling
- ✅ Technical stories reframed with user value or marked as Growth phase
- ✅ Each epic delivers standalone user value
- ✅ Epic sequence follows natural development dependency order
- ✅ Graceful degradation pattern preserved (NFR10 compliance)

## Epic List

1. **Epic 1: Persistent Data Storage & Session History** — Deliver durable, performant storage for telemetry data with comprehensive session history browsing and data integrity guarantees (6 FRs, 8 stories)

2. **Epic 2: Desktop Application Foundation** — Deliver the foundational desktop window and system integration required by all UI epics (3 FRs, 3 stories)

3. **Epic 3: Zero-Config Telemetry Capture Engine** — Deliver automatic, zero-configuration telemetry capture from iRacing that preserves all data with high fidelity and handles edge cases gracefully (11 FRs, 7 stories)

4. **Epic 4: AI-Powered Coaching Analysis** — Deliver grounded, insightful AI coaching that explains where time was lost and how to improve, using user-provided API keys (14 FRs, 8 stories)

5. **Epic 5: Interactive Debrief Visualization** — Deliver an intuitive, visually rich debrief interface that makes telemetry data understandable and connects AI coaching to visual evidence (7 FRs, 6 stories)

6. **Epic 6: Desktop Integration & Settings** — Deliver seamless settings management, auto-updates, and application lifecycle features (4 FRs, 6 stories)

**Total:** 6 Epics, 38 User Stories, 43 Functional Requirements, 30 Non-Functional Requirements

---

## Epic 1: Persistent Data Storage & Session History

**Epic Goal:** Deliver durable, performant storage for telemetry data with comprehensive session history browsing and data integrity guarantees.

**User Value:** "My telemetry is preserved forever and I can browse my racing history easily."

**Journey Alignment:** J1 (Progress tracking), J3 (Endurance sessions)

**Technical Scope:** Storage crate (SQLite + Parquet), session CRUD, data integrity validation, project scaffolding

**Mapped FRs:** FR6, FR7, FR10, FR31, FR32, FR33 (6 FRs)
**Note:** See PRD for complete FR definitions and acceptance criteria.

### Story 1.0: Project Foundation & Build Setup

As a developer,
I want the foundational project structure established,
So that all subsequent development has a consistent build and development environment.

**Acceptance Criteria:**

**Given** starting from an empty repository
**When** the project scaffold is initialized
**Then** Tauri 2.0 project is created using create-tauri-app CLI (v2.6.0+)
**And** Tauri capabilities are configured with minimal permissions (principle of least privilege)
**And** the default "Hello Tauri" window renders successfully

**Given** the project scaffold exists
**When** Cargo workspace is configured
**Then** workspace includes 4 crates: src-tauri (binary), telemetry-engine (lib), ai-provider (lib), storage (lib)
**And** dependency graph is validated: src-tauri depends on all 3 libs, telemetry-engine and ai-provider depend on storage, storage has no dependencies
**And** `cargo build` completes successfully for all crates
**And** `cargo test` runs successfully (even if no tests exist yet)

**Given** the frontend structure exists
**When** Tailwind CSS 4.x is configured
**Then** Tailwind processes successfully during build
**And** PostCSS configuration is validated
**And** custom design tokens are configured per UX spec (colors: bg-base, bg-surface, text-primary, accent-primary, etc.)
**And** JetBrains Mono and Inter fonts are loaded

**Given** the UI component system is configured
**When** shadcn/ui is initialized
**Then** components directory is created with owned source (not npm dependency)
**And** Radix UI primitives are configured
**And** example component (e.g., Button) renders successfully in dev mode
**And** dark theme is set as default (no light mode at MVP)

**Given** local development environment is configured
**When** developer runs `npm run tauri dev`
**Then** Vite dev server starts on port 1420
**And** Tauri window opens with hot-reload enabled
**And** Rust changes trigger recompilation
**And** frontend changes trigger hot-reload without full rebuild
**And** build completes in <30 seconds for incremental changes

**Technical Note:** This story establishes the foundation referenced in Architecture Implementation Sequence steps 1-3. All subsequent stories depend on this infrastructure existing.

**Definition of Done:**
- [ ] create-tauri-app project initialized
- [ ] Cargo workspace with 4 crates builds successfully
- [ ] Tailwind CSS 4.x configured and processing
- [ ] shadcn/ui components directory created
- [ ] Fonts (Inter + JetBrains Mono) loaded
- [ ] Design tokens configured
- [ ] Local dev server runs with hot-reload
- [ ] README updated with setup instructions

### Story 1.2: Desktop Application Runs Locally

As a user,
I want the application to run as a Windows desktop app,
So that I don't need browser configuration or internet dependency.

**Acceptance Criteria:**

**Given** the application is installed
**When** I launch the executable
**Then** a native desktop window opens
**And** the application runs entirely on my local machine
**And** no internet connection is required for core functionality

### Story 1.3: Session History Persists Locally

As a user,
I want all my racing sessions saved permanently on my computer,
So that I can review past performance anytime without data loss.

**Acceptance Criteria:**

**Given** I have completed multiple racing sessions
**When** I close and reopen the application
**Then** all previous session data is still available
**And** session metadata loads within 100ms
**And** no data is lost between application restarts

### Story 1.4: Parquet Telemetry Storage

As a sim racer,
I want my raw telemetry data stored in efficient Parquet files,
So that I can export and analyze data in external tools.

**Acceptance Criteria:**

**Given** a session has been captured
**When** the session ends or reaches a data checkpoint
**Then** raw telemetry channels are written to a Parquet file at `{data_dir}/telemetry/{session_id}.parquet`
**And** the file contains all 35 captured channels as columnar data
**And** the file is compressed with Snappy codec
**And** the file includes metadata (session_id, track, car, timestamp)
**And** a SHA-256 checksum is computed and stored in session metadata (NFR12)

**Given** Parquet write fails (disk full, permissions error)
**When** the error occurs
**Then** the app retries write once after 5 seconds
**And** if retry fails, logs the error and preserves in-memory data
**And** notifies the user "Failed to save telemetry data"
**And** all data is stored locally on user's machine (no cloud dependency)
**And** time-series telemetry uses Parquet format for analytical queries
**And** metadata uses SQLite for structured queries
**And** storage location is user-configurable with sensible default

### Story 1.5: Session CRUD Operations

As a sim racer,
I want to retrieve, update, and delete sessions,
So that I can manage my telemetry history.

**Acceptance Criteria:**

**Given** multiple sessions exist in the database
**When** I request the session list
**Then** sessions are returned sorted by timestamp descending (newest first)
**And** each session includes: id, track, car, date, lap_count, best_lap_time, status
**And** the query completes in <100ms for up to 1000 sessions (NFR5)

**Given** I want to view a specific session
**When** I provide a session_id
**Then** the app returns full session data including all laps and corner breakdowns
**And** loads the corresponding Parquet telemetry file
**And** the query completes in <200ms (NFR5a)
**And** validates the Parquet checksum on read (NFR12)

**Given** I want to delete a session
**When** I confirm deletion
**Then** the SQLite session record is marked as deleted (soft delete)
**And** the Parquet telemetry file is moved to a `.trash` directory
**And** the session no longer appears in the session list
**And** deletion can be undone within 30 days
**And** system stores debrief data (coaching text, insights, recommendations) in SQLite
**And** debrief data is linked to session_id for retrieval
**And** past debriefs are retrievable via session query interface

### Story 1.6: Session History List & Filtering

As a sim racer,
I want to browse and filter my session history,
So that I can find specific sessions quickly.

**Acceptance Criteria:**

**Given** I open the session history view
**When** the list loads
**Then** sessions are displayed as SessionCard components showing track, car, date, lap count, best lap
**And** the most recent 50 sessions load initially
**And** infinite scroll loads the next 50 when I reach the bottom

**Given** I want to filter sessions
**When** I select filters (track, car, date range)
**Then** the list updates to show only matching sessions
**And** filter state persists when I navigate away and return
**And** "Clear filters" button appears when any filter is active
**And** system computes consistency improvement (lap time std deviation trend)
**And** system computes technique change metrics (braking point variance, apex speed variance)
**And** progress metrics are queryable by track, car class, and date range

### Story 1.7: Data Integrity Validation

As a sim racer,
I want confidence that my telemetry data is accurate and uncorrupted,
So that AI coaching is based on trustworthy data.

**Acceptance Criteria:**

**Given** telemetry data is being written to Parquet
**When** the write completes
**Then** a SHA-256 checksum is computed and stored in session metadata (NFR12)
**And** the checksum is validated on read
**And** checksum mismatches are logged and reported to the user

**Given** a session is loaded from storage
**When** data integrity checks run
**Then** lap distances are monotonically increasing within each lap (NFR13)
**And** lap times are positive and <10 minutes (sanity check)
**And** telemetry channel values are within valid ranges (e.g., brake 0-100%) (NFR15)
**And** integrity violations are flagged with specific error messages

### Story 1.8: Historical Session Import

As a user,
I want to import telemetry from past racing sessions,
So that I can analyze historical performance alongside new data.

**Acceptance Criteria:**

**Given** I have archived session files (.ibt, .csv, or app export format)
**When** I import a historical session
**Then** session data loads into local database
**And** imported sessions appear in session history
**And** imported sessions are analyzable like live sessions

### Story 1.9: Reliable Updates Without Breaking Changes

As a sim racer,
I want confidence that app updates won't break my workflow,
So that I can install new versions safely without fear of losing functionality.

**Acceptance Criteria:**

**Given** I'm using a stable version of the app
**When** a new version is released
**Then** I can trust that core functionality (capture, debrief, history) still works
**And** breaking changes are caught before release
**And** I can see what was tested before installation

**Technical Implementation (CI/CD Pipeline):**

**Given** code changes are pushed to any branch
**When** the push completes
**Then** GitHub Actions automatically builds the project for Windows
**And** all tests run within 5 minutes
**And** build status shows on pull request
**And** failed builds block merging
**And** successful builds produce downloadable artifacts
**And** test coverage report is generated

**User Impact:** Prevents bugs from reaching users by catching integration issues early.

---

## Epic 2: Desktop Application Foundation

**Epic Goal:** Deliver the foundational desktop window and system integration required by all UI epics.

**User Value:** "The app runs as a native desktop application I can minimize and restore."

**Journey Alignment:** J1 (Seamless transition), J2 (Zero-config), J3 (Graceful errors)

**Technical Scope:** Tauri 2.0 window shell, system tray integration, desktop notifications

**Mapped FRs:** FR34, FR35, FR36 (3 FRs)
**Note:** See PRD for complete FR definitions and acceptance criteria.

### Story 2.1: Tauri Window & Tab Navigation

As a sim racer,
I want a clean desktop window with clear tab navigation,
So that I can easily switch between Summary, Coaching, and Telemetry views.

**Acceptance Criteria:**

**Given** I launch the app or open a debrief (FR34)
**When** the window appears
**Then** the window is 1200×800px by default (user-resizable, minimum 800×600px per UX spec)
**And** a tab bar displays at the top: Summary | Coaching | Telemetry (FR25)
**And** the active tab has an amber underline (2px, accent-primary)
**And** inactive tabs are text-secondary with hover effect

**Given** I click a tab
**When** the tab changes
**Then** the content area updates to show that tab's content
**And** the URL/state updates (React Router or TanStack Router per architecture)
**And** the persistent chat bar remains visible at the bottom
**And** the transition is smooth (150ms ease)

**Given** I use keyboard navigation
**When** I press Tab/Shift+Tab
**Then** focus cycles through interactive elements in logical order
**And** visible focus rings appear (2px solid amber with 2px offset, WCAG 2.1 AA)
**And** pressing Escape closes any modal or popover
**And** system tray icon displays provider connection status (ready/degraded/offline)
**And** status updates in real-time on connectivity changes
**And** tooltip shows provider name and last validation time

### Story 2.2: System Tray Status Indicator

As a sim racer,
I want the app to run in the system tray showing real-time capture status,
So that I always know if it's recording without opening the window.

**Acceptance Criteria:**

**Given** the app is running (FR35)
**When** I minimize the window
**Then** the app window closes but the process continues
**And** the system tray icon displays with current state (FR36):
  - Gray = idle (Waiting for iRacing)
  - Green = recording
  - Amber = debrief ready
  - Red = error
**And** hovering the tray icon shows a tooltip with current status
**And** when iRacing is not detected, tooltip displays "Waiting for iRacing"

**Given** iRacing is running and capture is active
**When** I check the tray icon (FR36)
**Then** the icon is solid green
**And** the tooltip shows "Recording — [Track], lap [X]..."
**And** the icon updates in real-time as lap count increases

**Given** a debrief is ready
**When** processing completes (FR36)
**Then** the tray icon changes to amber with a badge (number of unread debriefs)
**And** the tooltip shows "Debrief ready — [Track], [N] laps, best [time]"
**And** clicking the icon opens the debrief window to the latest session

### Story 2.3: Debrief Ready Notifications

As a sim racer,
I want a notification when my debrief is ready,
So that I know when to review it without constantly checking the app.

**Acceptance Criteria:**

**Given** a session has completed and AI analysis has finished (FR35)
**When** the debrief is ready
**Then** a system notification appears with title "Debrief Ready" and body "[Car] @ [Track] — [N] laps, best [time]"
**And** clicking the notification opens the app to that debrief's Summary tab
**And** the notification auto-dismisses from the OS tray after 30 seconds

**Given** iRacing is running in fullscreen (active racing)
**When** a debrief becomes ready
**Then** NO notification is shown (respects "Never interrupt racing" principle per UX spec)
**And** the tray badge updates silently
**And** the notification will appear when the user exits fullscreen

**Given** multiple sessions complete in quick succession
**When** debriefs are ready
**Then** only ONE notification is shown for the most recent session
**And** the tray badge count reflects all unread debriefs
**And** clicking the tray icon shows the session list with all unread sessions highlighted

---

## Epic 3: Zero-Config Telemetry Capture Engine

**Epic Goal:** Deliver automatic, zero-configuration telemetry capture from iRacing that preserves all data with high fidelity and handles edge cases gracefully.

**User Value:** "I install the app and it automatically records every iRacing session without any configuration."

**Journey Alignment:** J1 (Happy Path), J2 (Onboarding), J3 (Bad Session)

**Technical Scope:** Rust telemetry-engine crate, IRSDK integration, capture lifecycle management

**Mapped FRs:** FR1, FR1a, FR1b, FR2, FR3, FR4, FR5, FR8, FR9, FR40, FR41 (11 FRs)
**Note:** See PRD for complete FR definitions and acceptance criteria.

### Story 3.1: IRSDK Connection & Auto-Detection

As a sim racer,
I want the app to automatically detect when iRacing is running and connect to telemetry,
So that I never have to manually configure or start capture.

**Acceptance Criteria:**

**Given** iRacing is not running
**When** the user launches AI Race Team
**Then** the app polls for IRSDK shared memory availability every 2 seconds

**Given** iRacing launches while AI Race Team is running
**When** IRSDK shared memory becomes available
**Then** the app connects within 2 seconds
**And** system emits connection_status_changed event (connected/disconnected/error)
**And** a Tauri event "capture:irsdk-connected" is emitted

**Given** iRacing is already running
**When** the user launches AI Race Team
**Then** the app detects IRSDK immediately
**And** connects within 1 second
**And** begins capture without user action

### Story 3.2: Telemetry Channel Capture

As a sim racer,
I want the app to capture all essential telemetry channels from iRacing,
So that AI coaching has complete data to analyze my driving.

**Acceptance Criteria:**

**Given** IRSDK connection is established
**When** telemetry data is available
**Then** the app captures at 60Hz sampling rate
**And** captures all 35 required channels (speed, throttle, brake, steering, lat/long G, lap distance, lap time, tire temps/pressures, fuel, session state)
**And** stores raw channel data in memory ring buffers

**Given** telemetry channels are being captured
**When** iRacing updates session state (driving, pitting, spectating)
**Then** the app continues capturing all channels
**And** flags session state transitions with timestamps
**And** telemetry sample rate is configurable (default: 60Hz, range: 10-120Hz)
**And** system captures environmental conditions (track temp, air temp, weather)
**And** system captures session context (session type, car class, track config)

### Story 3.3: Session Lifecycle Management

As a sim racer,
I want the app to automatically detect session start/end and preserve incomplete sessions,
So that I never lose data even if I disconnect or crash.

**Acceptance Criteria:**

**Given** the user enters a driving session in iRacing
**When** SessionState changes to "Racing" or "Testing"
**Then** the app marks session start with timestamp
**And** emits "session:capture-started" event
**And** system emits session_state_changed event (idle/recording/processing/completed)

**Given** the user completes a session normally
**When** SessionState changes to "ParadeLap" or session timer expires
**Then** the app marks session end with timestamp
**And** emits "session:capture-stopped" event
**And** triggers data flush to storage

**Given** the user disconnects mid-session (alt-F4, crash, network drop)
**When** IRSDK connection is lost
**Then** the app preserves all captured data up to disconnection
**And** marks the session as "partial" with disconnection timestamp
**And** emits "capture:irsdk-disconnected" event
**And** system detects lap boundaries using iRacing lap distance crossing logic
**And** lap boundary timestamps are recorded with <50ms precision
**And** lap summaries are computed immediately upon boundary detection
**And** system preserves incomplete laps (spins, resets, disconnects) as diagnostic data
**And** incomplete laps are flagged with completion_status metadata
**And** system inserts gap markers on telemetry stream interruptions
**And** gap markers record start_time, end_time, and reason (disconnect/reset/crash)
**And** analysis correctly handles gaps without misinterpreting data continuity

### Story 3.3b: Manual Debrief Trigger

As a user,
I want to manually trigger debrief analysis anytime,
So that I can analyze partial sessions or re-analyze completed sessions.

**Acceptance Criteria:**

**Given** a session is active or completed
**When** I select "Generate Debrief" from system tray menu
**Then** debrief analysis starts immediately
**And** debrief generation follows normal pipeline
**And** manual triggers work for incomplete sessions

### Story 3.4: Capture Error Handling & Recovery

As a sim racer experiencing telemetry issues,
I want the app to gracefully handle data gaps and provide clear error messages,
So that I understand what went wrong and can take action if needed.

**Acceptance Criteria:**

**Given** telemetry capture is active
**When** a data gap occurs (e.g., IRSDK stalls for >100ms)
**Then** the app marks the gap with start/end timestamps
**And** continues capture after the gap
**And** logs the gap duration for session metadata

**Given** IRSDK connection fails during session
**When** reconnection is possible
**Then** the app attempts reconnection every 1 second for 30 seconds
**And** displays "Reconnecting..." in system tray tooltip

**Given** IRSDK connection cannot be re-established
**When** 30 seconds elapse with no connection
**Then** the app stops capture attempt
**And** displays error notification "Telemetry connection lost"
**And** preserves partial session data with clear "disconnected" status
**And** when session end is detected, system emits session_completed event with session_id
**And** session_completed event triggers debrief generation pipeline

### Story 3.5: Background Capture Monitoring

As a sim racer,
I want the capture engine to run with minimal system impact,
So that it doesn't affect my sim racing performance.

**Acceptance Criteria:**

**Given** telemetry capture is active during racing
**When** system resource usage is measured
**Then** CPU usage is <2% of total system CPU (NFR1)
**And** memory (RSS) usage is <200MB (NFR1)
**And** measurements are logged every 60 seconds for monitoring

**Given** capture is running in the background
**When** the user minimizes AI Race Team to the system tray
**Then** the window closes but capture continues
**And** system emits debrief_ready event with session_id
**And** no UI rendering occurs while minimized

### Story 3.6: Derived Metrics Engine

As a user,
I want advanced metrics computed automatically,
So that I get deeper insights beyond raw telemetry.

**Acceptance Criteria:**

**Given** session telemetry is captured
**When** derived metrics engine processes data
**Then** brake application count is computed per lap
**And** trail braking phases are identified and quantified
**And** tire degradation curves are calculated
**And** derived metrics are stored with session data

### Story 3.7: Session Type Capture

As a user,
I want session type automatically detected,
So that analysis is contextually relevant (practice vs race).

**Acceptance Criteria:**

**Given** iRacing session is active
**When** telemetry capture starts
**Then** session type is detected (practice/qualifying/race/warmup)
**And** session type is stored with session metadata
**And** session type is visible in UI and used for filtering

---

## Epic 4: AI-Powered Coaching Analysis

**Epic Goal:** Deliver grounded, insightful AI coaching that explains where time was lost and how to improve, using user-provided API keys.

**User Value:** "I get AI coaching that tells me exactly where I lost time and how to improve, with evidence I can verify."

**Journey Alignment:** J1 (Core value), J2 (First-time success), J3 (Failure coaching)

**Technical Scope:** ai-provider crate, BYOK integration, prompt engineering, structured output

**Mapped FRs:** FR11, FR12, FR13, FR14, FR15, FR16, FR17, FR18, FR19, FR20, FR21, FR22, FR23, FR24 (14 FRs)

### Story 4.1: BYOK API Key Management

As a sim racer,
I want to securely store my Claude or OpenAI API key,
So that I can use AI coaching without the app having access to my credentials.

**Acceptance Criteria:**

**Given** I'm setting up the app for the first time
**When** I reach the API key setup step
**Then** I see input fields for Claude API key OR OpenAI API key
**And** I see a link to "How to get a key in 2 minutes" with step-by-step instructions
**And** I can choose "Skip for now" to use local-only mode (FR24)

**Given** I paste an API key
**When** I submit the key
**Then** the app validates the key by making a test API call (FR23)
**And** if valid, stores the key in OS credential store via keyring-rs (FR22, NFR17)
**And** displays "✓ Connected — AI coaching ready"
**And** if invalid, shows "That key didn't work. Check it starts with sk-..." with specific error details (FR23)

**Given** I want to change my API key later
**When** I open Settings → AI Provider
**Then** I can view the currently configured provider (Claude/OpenAI)
**And** I can update or remove the key
**And** key changes take effect immediately without app restart

**Given** I have configured a valid API key
**When** a session ends and debrief generation begins
**Then** the system validates API connectivity at debrief time (not session start time)
**And** if API is unavailable, displays "AI coaching unavailable - showing telemetry analysis only"
**And** user can still view telemetry charts, lap comparisons, and derived metrics (FR9)
**And** user can retry AI analysis later via "Retry AI Coaching" button

**Given** I have skipped API key setup (local-only mode)
**When** a session ends
**Then** debrief shows raw pre-processed stats without AI coaching
**And** debrief includes corner summaries, lap comparisons, derived metrics (NFR10)
**And** user sees "Add API key for AI coaching" prompt with link to settings
**And** telemetry capture and data review work fully without AI

**Given** my API key becomes invalid mid-session (expired, rate limit, network issue)
**When** session ends and debrief generation runs
**Then** telemetry capture completed successfully (not blocked)
**And** pre-processing completed successfully (corner detection, lap analysis)
**And** debrief view shows local stats immediately
**And** AI coaching section shows error state: "Unable to connect to AI provider"
**And** user can retry AI analysis after fixing the issue
**And** existing debriefs remain accessible regardless of current API status (NFR10)

### Story 4.2: AI Coaching Uses My Preferred Provider

As a user,
I want to use my own AI provider account (Claude or OpenAI),
So that I control costs and data privacy.

**Acceptance Criteria:**

**Given** I have configured my API key for Claude or OpenAI
**When** debrief analysis runs
**Then** coaching uses my selected provider
**And** my API key is stored securely in OS credential store
**And** I can switch providers without losing functionality
**And** provider errors show clear, actionable messages
**And** system dispatches AI requests to user's configured provider (Claude or OpenAI)
**And** dispatch routing is determined by provider_type configuration
**And** dispatch failures log provider name and error details

### Story 4.3: Structured Coaching Prompt Engineering

As a sim racer,
I want AI coaching that references specific laps, corners, and telemetry values,
So that I can verify and trust the advice.

**Acceptance Criteria:**

**Given** a session_completed event is received
**When** the event contains valid session_id
**Then** debrief generation begins automatically

**Given** a session has been pre-processed with corner segmentation
**When** the AI analysis prompt is constructed
**Then** the prompt includes session context (track, car, lap count, conditions)
**And** includes best lap time and average lap time (FR15)
**And** includes per-corner statistics (best vs avg brake pressure, throttle application, speed delta) (FR14)
**And** includes specific lap numbers for reference laps
**And** instructs the AI to cite lap numbers and corner names in coaching output (FR13)

**Given** the AI generates coaching text (FR12)
**When** the response is parsed
**Then** coaching text includes specific corner names (T1, T2, T3...)
**And** includes specific telemetry values ("average brake pressure was 42%") (FR13)
**And** includes lap number references ("Your best lap, Lap 14, used...")
**And** coaching is structured with clear sections (Overview, Per-Corner Analysis, Focus Next Session) (FR12)

### Story 4.4: Per-Corner Analysis Generation

As a sim racer,
I want detailed coaching for each corner where I'm losing time,
So that I know exactly what to improve.

**Acceptance Criteria:**

**Given** a session with multiple corners (T1-T7)
**When** per-corner analysis is generated (FR14)
**Then** the AI analyzes each corner independently
**And** identifies time gained/lost per corner vs best lap (FR15)
**And** explains the driving behavior difference (brake pressure, throttle timing, line consistency)
**And** ranks corners by improvement opportunity (largest time loss first)
**And** uses plain-English language suitable for newcomers (FR17)

**Given** a corner has inconsistent data (telemetry gap, partial lap)
**When** that corner is analyzed
**Then** the AI notes the data limitation
**And** provides analysis based on available clean laps only
**And** coaching includes "Based on 8 of 12 laps (4 excluded due to incomplete data)"

**Given** session-over-session comparison is requested (FR16)
**When** analyzing the same track across multiple sessions
**Then** the AI compares current session to previous sessions on that track
**And** highlights improvements or regressions per corner
**And** coaching includes "Your T1 braking improved from 37% avg last week to 52% today"
**And** analysis engine processes incomplete laps for incident diagnosis

### Story 4.5: Session Debrief Streaming

As a sim racer,
I want AI coaching to appear progressively as it's generated,
So that I don't wait for the entire analysis before seeing value.

**Acceptance Criteria:**

**Given** AI analysis has started (FR19)
**When** tokens begin streaming from the API
**Then** the frontend receives tokens via Tauri events
**And** coaching text streams as SSE events (coaching_chunk) with chunk_id sequence
**And** the user can start reading the top-level insight while per-corner details stream
**And** first coaching token appears within 5 seconds of analysis start (NFR6a)

**Given** streaming is interrupted (network drop, API timeout)
**When** the interruption occurs
**Then** the app preserves all tokens received so far
**And** displays "[Streaming interrupted - partial coaching shown]"
**And** offers "Retry" button to re-request full analysis

**Given** the full debrief has been generated (FR11)
**When** timing is measured
**Then** the entire debrief completes within 60 seconds of session end (NFR4)
**And** the user sees the top insight within 5 seconds (NFR6a, FR19)

### Story 4.6: Confidence Scoring UI Indicators [DEFERRED TO GROWTH]

**Rationale:** Confidence indicators are Growth phase per PRD:116, PRD:221. Requires historical data tracking and outcome validation not in MVP scope.

**Dependencies:** Session history analytics, dismissal tracking, multi-session comparison infrastructure.

As a sim racer,
I want to know how confident the AI is in each insight,
So that I can prioritize which advice to follow.

**Acceptance Criteria:**

**Given** AI coaching has been generated
**When** each per-corner insight is evaluated
**Then** the app computes a confidence score (high/medium/low) based on:
- Data completeness (% of laps with clean telemetry for that corner)
- Consistency of finding (does the pattern hold across multiple laps?)
- Magnitude of delta (is the time loss significant?)

**Given** an insight has low confidence (<60% data completeness OR inconsistent pattern)
**When** the insight is displayed
**Then** it appears with a subtle low-confidence indicator (muted color, optional icon)
**And** the user can dismiss the insight
**And** dismissal is logged for future AI improvement

### Story 4.6a: Telemetry Grounding & Verification

As a sim racer,
I want every AI coaching insight linked to specific telemetry data,
So that I can verify the advice against my actual driving.

**Acceptance Criteria:**

**Given** every AI claim is made (FR13)
**When** the coaching text is generated
**Then** the claim includes a reference to supporting telemetry:
- Specific lap number(s)
- Corner name/number
- Telemetry channel value (e.g., "average brake pressure was 42%")

**Given** the user wants to verify an insight
**When** the insight references a specific corner or lap
**Then** clicking the reference navigates to that section in the Telemetry tab
**And** the referenced data is highlighted in the chart
**And** the user can see the raw data that supports the claim

**Given** historical context is available (multiple sessions on same track)
**When** coaching compares current session to previous sessions (FR16)
**Then** comparison data includes: current_session metrics, reference_session metrics, delta values
**And** the user can see both sessions' telemetry overlaid for visual comparison

### Story 4.7: Car Template System

As a user,
I want coaching tailored to my specific car class,
So that advice is relevant to my vehicle's characteristics.

**Acceptance Criteria:**

**Given** I'm driving a specific car class (GT3, Formula, etc.)
**When** debrief analysis runs
**Then** AI uses car-class-specific coaching templates
**And** coaching references car-specific techniques (downforce, traction control, etc.)
**And** templates are configurable per car class

### Story 4.8: Anomaly & Incident Detection

As a user,
I want the system to diagnose why laps were incomplete,
So that I understand what went wrong (crash, disconnect, reset).

**Acceptance Criteria:**

**Given** a lap ends prematurely
**When** analysis engine processes the lap
**Then** incident type is classified (spin/crash/disconnect/manual-reset)
**And** incident location is identified (track position, corner)
**And** incident telemetry is flagged for review
**And** coaching includes incident diagnosis in debrief

---

## Epic 5: Interactive Debrief Visualization

**Epic Goal:** Deliver an intuitive, visually rich debrief interface that makes telemetry data understandable and connects AI coaching to visual evidence.

**User Value:** "I can see my telemetry, understand the coaching visually, and explore my data with one-click depth."

**Journey Alignment:** J1 (Evidence confirmation), J2 (Visual learning), J3 (Corner investigation)

**Technical Scope:** React frontend, uPlot charts, tab navigation, corner-zoom, chat interface

**Mapped FRs:** FR25, FR26, FR27, FR28, FR29, FR30, FR40 (7 FRs)

### Story 5.1: Summary Tab with Hero Cards

As a sim racer,
I want to see my session headline stats and top coaching insight immediately upon opening a debrief,
So that I get value in 5 seconds without deep investigation.

**Acceptance Criteria:**

**Given** I open a completed session debrief (FR25)
**When** the window appears
**Then** the Summary tab is active by default
**And** hero stat cards display instantly (<200ms): Best Lap, Session Delta, Laps Completed, Focus Area from previous session
**And** AI coaching headline streams in within 5-10 seconds (NFR6a)
**And** all cards use the established design tokens (amber accent, dark backgrounds, JetBrains Mono for times)
**And** the layout is screenshot-optimized for Discord sharing (UX J1)

**Given** the session was an endurance race (>45 min OR multiple pit stops)
**When** the Summary tab loads
**Then** the layout adapts to show stint breakdown cards instead of single hero cards (UX adaptive pattern)
**And** each stint shows: stint number, lap range, average lap time, best lap, tire/fuel strategy
**And** Telemetry tab displays per-lap summary statistics in table format
**And** per-lap table shows: lap#, time, sector times, top speed, avg speed, incidents
**And** table is sortable and filterable

### Story 5.2: Coaching Tab with Sidebar Navigation

As a sim racer investigating AI coaching details,
I want to read per-corner breakdowns and navigate quickly between corners,
So that I can explore the full analysis efficiently.

**Acceptance Criteria:**

**Given** I click the Coaching tab (FR25)
**When** the tab loads
**Then** a fixed 280px sidebar displays on the left with corner shortcuts (T1-T7) (CornerSidebar component)
**And** the main panel shows the full AI coaching narrative (CoachingPanel component)
**And** each corner in the sidebar shows a time delta indicator (green/red) and severity icon
**And** if coaching is actively streaming, a blinking amber cursor `▊` indicates streaming progress

**Given** I click a corner name in the sidebar
**When** the click occurs
**Then** the main panel scrolls to that corner's coaching section
**And** the corner is highlighted with an amber left-border
**And** the sidebar indicates which corner is currently active

**Given** I click a corner name within the coaching text
**When** the click occurs (FR29)
**Then** the app navigates to the Telemetry tab
**And** the chart zooms to that corner with ~200m context (FR29)
**And** the corner annotation overlay highlights

### Story 5.3: Telemetry Chart (1D Lap-Distance)

As a sim racer,
I want to see brake and throttle traces overlaid on lap distance,
So that I can visually understand my inputs and compare laps.

**Acceptance Criteria:**

**Given** I open the Telemetry tab (FR25, FR26)
**When** the chart renders (TelemetryChart component)
**Then** the X-axis shows lap distance from 0 to track length in meters
**And** the Y-axis shows input percentage (0-100%)
**And** brake trace displays as a red filled area
**And** throttle trace displays as a green filled area
**And** corner zones (T1-T7) display as vertical dashed lines with labels
**And** the chart defaults to best lap (green) vs average lap (gray) overlay (FR27)
**And** the chart renders at 60fps for smooth interaction (NFR6)

**Given** I want to compare specific laps (FR28)
**When** I use the lap picker dropdown
**Then** I can select any two laps from the session
**And** the chart updates to show those two laps overlaid
**And** the legend clearly identifies which color represents which lap

**Given** the chart is displayed
**When** the viewport is resized
**Then** the chart scales to fill available width (responsive, container-first)
**And** maintains aspect ratio
**And** axis labels remain readable at all sizes

### Story 5.4: Corner-Zoom & Cursor Sync

As a sim racer,
I want to zoom into specific corners and see synchronized values across all charts,
So that I can analyze cause-and-effect in my driving.

**Acceptance Criteria:**

**Given** I'm viewing the telemetry chart
**When** I click a corner zone label (e.g., "T5") (FR29)
**Then** the chart smoothly zooms (300ms ease-out) to show ~200m centered on that corner
**And** a "Reset zoom" button appears in the top-right
**And** double-clicking the chart background also resets zoom
**And** zoom completes in <300ms (NFR27)

**Given** I hover over any point on the telemetry chart
**When** the cursor moves
**Then** a vertical crosshair line snaps to the nearest data point
**And** a tooltip appears showing exact values: distance, speed, brake %, throttle %, lap time delta
**And** if multiple charts are visible, all charts' cursors sync to the same lap distance (cursor sync pattern)

**Given** I'm using keyboard navigation
**When** I press arrow keys (left/right)
**Then** the cursor moves to the previous/next corner marker
**And** tooltip updates accordingly
**And** pressing Escape resets zoom and cursor

### Story 5.5: Persistent Chat Interface

As a sim racer,
I want to ask follow-up questions about my session in natural language,
So that I can dig deeper into specific concerns.

**Acceptance Criteria:**

**Given** I'm viewing any tab (Summary, Coaching, Telemetry) (FR30)
**When** the view loads
**Then** a persistent chat input bar displays at the bottom (Persistent Chat Input component)
**And** defaults to collapsed (minimized) after the first 5 sessions (UX adaptive pattern)
**And** shows "Ask about your session..." placeholder text
**And** expands when clicked to show input field + send button

**Given** I type a question and click send
**When** the message is submitted
**Then** my question appears in the conversation history
**And** the AI response streams in progressively word-by-word
**And** the AI has full context of the current session and tab I'm viewing
**And** the AI grounds responses with specific lap/corner references just like the main coaching

**Given** I switch tabs while a conversation is active
**When** I navigate to a different tab
**Then** the chat conversation persists (doesn't reset)
**And** the chat context updates to include which tab I'm now viewing
**And** I can continue asking questions relevant to the new tab's content

### Story 5.6: Export Debrief for External Analysis [DEFERRED TO GROWTH]

**Rationale:** Data export is a Growth phase power-user feature. MVP focuses on in-app debrief experience. JSON data contract exists internally for IPC but user-facing export is not MVP critical path.

**Dependencies:** API stabilization, schema versioning, user testing of export formats.

As a sim racer,
I want to export my debrief data to other tools,
So that I can use third-party analysis software or share structured data with teammates.

**Acceptance Criteria:**

**Given** I have completed a debrief
**When** I click "Export Debrief"
**Then** a JSON file downloads with structured debrief data
**And** the export includes: insights, recommendations, metrics, session metadata
**And** I can import this data into spreadsheet tools (Excel, Google Sheets)
**And** I can share the file with teammates for collaborative analysis

**Technical Implementation (Data Contract):**

**Given** debrief analysis completes
**When** debrief data is serialized
**Then** output follows documented JSON schema
**And** schema version is included for compatibility tracking
**And** file format is human-readable (pretty-printed JSON)

---

## Epic 6: Desktop Integration & Settings

**Epic Goal:** Deliver seamless settings management, auto-updates, and application lifecycle features.

**User Value:** "The app runs invisibly in the background and notifies me when debriefs are ready, with zero manual intervention."

**Journey Alignment:** J1 (Seamless transition), J2 (Zero-config), J3 (Graceful errors)

**Technical Scope:** Settings management, auto-updates, application lifecycle, OS integration

**Mapped FRs:** FR37, FR38, FR39, FR41 (4 FRs)

### Story 6.1: Settings Management

As a sim racer,
I want to configure app preferences (data paths, AI provider, capture settings),
So that I can customize the app to my workflow.

**Acceptance Criteria:**

**Given** I open Settings (tray menu → Settings or in-app menu)
**When** the settings view loads
**Then** I see sections for: AI Provider, Data Storage, Capture Settings, UI Preferences
**And** all current values are displayed
**And** changes are saved immediately (no "Save" button required)
**And** a "Reset to Defaults" option is available per section

**Given** I want to change the data storage location
**When** I click "Change Data Folder"
**Then** a native file picker opens
**And** I can select a new folder
**And** existing data is NOT moved automatically (user must manually migrate if desired)
**And** new sessions will be stored in the new location
**And** the app shows both old and new locations in the session list

**Given** I want the app to launch on system startup
**When** I enable "Launch at Startup" in Settings
**Then** the app is registered in OS startup programs
**And** on next boot, the app launches minimized to tray (window closed)
**And** capture auto-starts if iRacing is detected

### Story 6.2: Application State Syncs Seamlessly

As a user,
I want the UI to always reflect current system state,
So that I know exactly what's happening (recording, processing, ready).

**Acceptance Criteria:**

**Given** the backend state changes (e.g., starts recording)
**When** the state change occurs
**Then** the UI updates within 100ms
**And** state transitions are smooth without UI freezing
**And** error states display actionable messages

### Story 6.3: Audio Notification System

As a user,
I want audio notifications for key events,
So that I'm alerted without watching the screen.

**Acceptance Criteria:**

**Given** a significant event occurs (session start, debrief ready, error)
**When** the event triggers
**Then** system plays audio notification (configurable sound)
**And** visual notification displays simultaneously
**And** I can configure audio on/off and volume in settings
**And** audio preferences persist across sessions

### Story 6.4: Application Auto-Update

As a user,
I want the application to update automatically,
So that I always have the latest features and fixes.

**Acceptance Criteria:**

**Given** a new application version is released
**When** I launch the application
**Then** system checks for updates
**And** update notification displays if available
**And** I can choose to install now or defer
**And** update downloads and installs with user confirmation

### Story 6.5: Safe Quit Without Data Loss

As a user,
I want to quit the application anytime without losing data,
So that I can close the app confidently.

**Acceptance Criteria:**

**Given** I quit during active recording or processing
**When** I click quit or close window
**Then** in-progress work saves before exit
**And** application quits within 2 seconds
**And** next launch shows no data corruption

### Story 6.6: Crash Recovery Preserves Sessions

As a user,
I want my session data preserved even if the app crashes,
So that unexpected errors don't lose my racing data.

**Acceptance Criteria:**

**Given** the application crashes during recording
**When** I restart the application
**Then** partial session data is recovered
**And** I see a recovery prompt with session details
**And** recovered data is usable for analysis

---

## Summary

**Epic Breakdown Complete:**
- **6 Epics** organized by user value delivery
- **38 User Stories** (36 MVP + 2 deferred to Growth) with complete Given/When/Then/And acceptance criteria
- **43 Functional Requirements** (FR1-FR41 with FR1 variants) fully mapped
- **30 Non-Functional Requirements** cross-mapped to applicable stories
- **100% requirements coverage** verified (all 6 critical/major implementation readiness issues resolved - 2026-02-05)

**Development Readiness:**
- Each epic delivers standalone user value
- Stories are implementation-ready with clear acceptance criteria
- Architecture, UX, and NFR constraints integrated into acceptance criteria
- Epic sequence follows natural dependency order: Storage → Desktop Foundation → Capture → AI → Visualization → Settings
- All stories align with user journeys J1-J3 from UX design
- Epic independence restored via event-based decoupling (Correction #2 - 2026-02-05)
- Technical stories reframed with user value or marked as Growth phase (Corrections #5 - 2026-02-05)
- PRD is single source of truth for all FR definitions (Correction #1 - 2026-02-05)
- Graceful degradation pattern enforced (NFR10 compliance via Correction #6 - 2026-02-05)
- Project foundation story added for scaffolding (Correction #4 - 2026-02-05)
- Scope discipline maintained: Growth features deferred appropriately (Correction #3 - 2026-02-05)

**Next Steps:**
- Begin implementation with Epic 1: Persistent Data Storage & Session History
- Use this document as the single source of truth for feature requirements
- Update story status as development progresses
- Verify acceptance criteria during QA and user testing
