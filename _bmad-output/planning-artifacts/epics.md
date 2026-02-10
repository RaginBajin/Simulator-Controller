---
stepsCompleted: [1]
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
  - _bmad-output/planning-artifacts/ux-design-specification.md
---

# Simulator-Controller - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for Simulator-Controller (AI Race Team), decomposing the requirements from the PRD, UX Design, and Architecture documents into implementable stories.

## Requirements Inventory

### Functional Requirements

**From PRD (FR1-FR41):**

**Telemetry Capture & Data Integrity (FR1-FR7):**
- FR1: Auto-detect iRacing running via process detection
- FR2: Capture telemetry at 60 Hz from iRacing IRSDK (shared memory)
- FR3: Preserve partial laps and mark telemetry gaps with timestamps
- FR4: Store raw telemetry in Parquet format with gzip compression
- FR5: Compute derived channels (delta speed, cornering force, tire slip)
- FR6: Segment laps into corners using GPS coordinates + track map
- FR7: Store session metadata (track, car, date, weather) in SQLite

**AI Coaching & Analysis (FR8-FR16):**
- FR8: Generate session debrief via BYOK AI (Claude or OpenAI)
- FR9: Stream AI responses progressively (not blocking)
- FR10: Produce per-corner coaching insights with lap references
- FR11: Support conversational follow-up questions with session context
- FR12: Generate "Focus Next Session" recommendation
- FR13: Detect telemetry anomalies and provide coaching
- FR14: Compare session-over-session performance with trend indicators
- FR15: Adapt coaching language to user skill level (plain English vs technical)
- FR16: Detect incidents (spins, crashes, disconnects) and preserve partial laps

**Visualization & UI (FR17-FR27):**
- FR17: Display 1D lap-distance chart (brake/throttle vs distance)
- FR18: Overlay best lap (green) vs average lap (gray) on charts
- FR19: Implement cursor sync across all charts in view
- FR20: Support corner-zoom navigation (click corner → zoom to ±100m context)
- FR21: Show lap picker dropdown (default: best vs avg, select any two laps)
- FR22: Display hero stat cards (best lap, session delta, laps completed, focus area)
- FR23: Provide tab-based navigation (Summary | Coaching | Telemetry)
- FR24: Implement persistent chat input across all tabs
- FR25: Display session history as timeline cards (Strava-inspired)
- FR26: Show stint breakdown cards for endurance sessions (>45 min or >1 pit stop)
- FR27: Display progress line graph (session-over-session best lap trend)

**Session History & Comparison (FR28-FR32):**
- FR28: Store all sessions locally with full telemetry
- FR29: Support session search and filtering (track, car, date range)
- FR30: Compare any two sessions from history
- FR31: Show session-over-session delta with trend arrows (green up / red down)
- FR32: Persist "Focus Next Session" recommendation for returning users

**Desktop Integration (FR33-FR38):**
- FR33: Run in system tray with dynamic status icon (idle/recording/processing/ready/error)
- FR34: Show system tray notification when debrief ready
- FR35: Display amber tray badge for unread debriefs
- FR36: Support left-click tray to open latest debrief
- FR37: Provide right-click tray context menu (quit, settings, session list)
- FR38: Auto-minimize to tray on window close (not quit)

**Data Export & Portability (FR39-FR41):**
- FR39: Export telemetry to CSV format
- FR40: Export session summary with AI coaching to Markdown
- FR41: Support bulk session export with metadata

### Non-Functional Requirements

**From PRD (NFR1-NFR28):**

**Performance (NFR1-NFR6):**
- NFR1: Telemetry capture < 5% CPU impact during racing
- NFR2: Telemetry write latency < 100ms (no blocking during capture)
- NFR3: Session list loads < 200ms for 100 sessions
- NFR4: Debrief Summary tab renders local stats in < 200ms
- NFR5: Telemetry chart renders 10,000 data points in < 500ms
- NFR6: AI coaching streams progressively (first token < 3s, complete < 15s)

**Reliability (NFR7-NFR12):**
- NFR7: Preserve partial laps if iRacing crashes or disconnects
- NFR8: Handle telemetry gaps gracefully with inline warnings
- NFR9: Auto-recover from IRSDK connection loss
- NFR10: Degrade gracefully if AI API unavailable (show local stats only)
- NFR11: Never lose captured telemetry data on app crash
- NFR12: Window size and position persisted across sessions

**Data Integrity (NFR13-NFR17):**
- NFR13: Parquet files validated on write (schema enforcement)
- NFR14: SQLite transactions for all metadata writes
- NFR15: Detect and mark telemetry gaps with timestamps + duration
- NFR16: Corner segmentation failures flagged, not discarded
- NFR17: No silent data loss — all errors logged and surfaced to user

**Security (NFR18-NFR20):**
- NFR18: API keys stored in OS credential store (keyring-rs)
- NFR19: Never log API keys or responses in plain text
- NFR20: Telemetry data never leaves local machine without user export

**Integration (NFR21-NFR23):**
- NFR21: Detect iRacing install location automatically (registry scan)
- NFR22: Support Windows 10 and Windows 11 (WebView2 required)
- NFR23: IRSDK shared memory access (Windows-only)

**Usability (NFR24-NFR28):**
- NFR24: First-time setup completes in < 3 minutes (detect iRacing → paste API key → done)
- NFR25: Zero-click session capture (fully automatic when iRacing runs)
- NFR26: Debrief headline visible within 5 seconds of opening Summary tab
- NFR27: All AI coaching insights grounded in specific lap numbers and telemetry values
- NFR28: Screenshot-optimized Summary tab for Discord sharing

### Additional Requirements

**From Architecture Document:**

**AR1: Starter Template**
- Use Tauri create-app script: `sh <(curl https://create.tauri.app/sh)`
- Select React template with TypeScript

**AR2: Data Storage Strategy**
- SQLite (via SQLx) for session metadata, summaries, and AI coaching text
- Parquet files (via arrow2/parquet2 crates) for time-series telemetry data
- File naming: `{session_uuid}.parquet` for telemetry
- Database schema: `snake_case` tables and columns

**AR3: Security Implementation**
- BYOK (Bring Your Own Key) model - users provide own API keys
- OS credential store via keyring-rs crate
- Never store API keys in plain text configuration files
- API keys secured at rest and only accessed via keyring API

**AR4: IPC Architecture**
- Tauri Commands for request/response patterns (e.g., fetch session list, export data)
- Tauri Events for streaming/state updates (e.g., telemetry capture status, AI response streaming)
- Command naming: `snake_case` (e.g., `get_session_list`, `start_capture`)
- Event naming: `domain:action` kebab-case (e.g., `telemetry:capture-started`, `ai:coaching-chunk`)

**AR5: Frontend Stack Requirements**
- React 19.2.x with TypeScript 5.x
- TanStack Query 5.x for async state management
- Zustand 5.x for client-side state
- React Router DOM 7.12.x for navigation
- uPlot 1.6.x for high-performance telemetry charts
- shadcn/ui components with Radix UI primitives
- Tailwind CSS 4.x for styling

**AR6: Testing Requirements**
- Vitest 3.x for unit and integration tests
- React Testing Library 16.x for component tests
- Rust unit tests with `cargo test`
- Test coverage targets: >80% for critical paths (telemetry capture, data integrity)

**AR7: AI Provider Architecture**
- Trait-based provider system (`TextGeneration`, `VoiceGeneration`)
- Provider adapters for Claude (Anthropic API) and OpenAI
- Support for both streaming and non-streaming responses
- Graceful fallback if provider unavailable

**AR8: iRacing Integration**
- Windows-only telemetry capture via IRSDK shared memory
- Auto-detect iRacing process and session state
- 60 Hz capture rate (matches iRacing telemetry frequency)
- Handle session transitions (practice → qualifying → race)

**AR9: Project Structure**
- Frontend: Feature-based organization (`features/`, `components/`, `hooks/`)
- Rust: Crate-based organization (`telemetry-capture`, `ai-coaching`, `storage`)
- Shared types between Rust and TypeScript via `specta` crate
- Clear boundary between capture (Rust) and presentation (React)

**AR10: Build & Deployment**
- Tauri 2.0 native build system
- Windows installer (.msi) with WebView2 bootstrapper
- Desktop icon and start menu shortcuts
- System tray integration configured in `tauri.conf.json`

**From UX Design Document:**

**UXR1: Visual Design System**
- Dark-first color palette (no light mode at MVP)
- Amber accent color (#F59E0B) for interactive elements
- Inter font for UI text, JetBrains Mono for telemetry values
- Dual-density spacing: relaxed for coaching, compact for data
- Border radius tokens: sm (4px), md (6px), lg (8px)

**UXR2: Component System**
- Tailwind CSS 4.x + shadcn/ui for base components
- Custom components: Session Card, Debrief Header, Telemetry Chart, Corner Sidebar, Coaching Panel, Persistent Chat Input, Progress Line Graph, Tray Status Indicator, Corner Annotation Overlay
- All components follow amber-for-interaction, green/red-for-data pattern

**UXR3: Accessibility Requirements** (MVP Scope)
- Full keyboard navigation (Tab, Enter, Escape, Arrow keys)
- Visible focus indicators with 2px amber ring
- WCAG AA contrast compliance (4.5:1 for normal text, 3:1 for large)
- Semantic HTML structure with proper heading hierarchy
- Respect `prefers-reduced-motion` for animations
- Screen reader optimization deferred to post-MVP

**UXR4: User Journey Support**
- Two-phase loading: local stats instant (<200ms), AI coaching progressive (3-15s)
- System tray states: idle (gray), recording (green), processing (amber pulse), ready (amber + badge), error (red)
- Tray notification format: "[Car] @ [Track] — Debrief ready"
- Onboarding: 3-step max (detect iRacing → paste API key or skip → done)
- Local-only mode for users without API key (lap time graph, basic stats, coaching teaser)

**UXR5: Responsive Design**
- Minimum window size: 800×600px (enforced by Tauri)
- Container queries for component-level responsiveness
- Breakpoints: compact (<600px), standard (600-1000px), wide (>1000px)
- Cross-platform WebView support: macOS WebKit, Windows WebView2

**UXR6: Interaction Patterns**
- Corner names clickable everywhere → navigate to Telemetry tab centered on corner
- Lap picker default: best vs average, power users select any two laps
- Chat input collapsible, defaults to open for first 5 sessions
- Session history preserves scroll position and filters when returning
- Tab transitions with amber underline slide animation (150ms ease)

**UXR7: Data Visualization Patterns**
- Crosshair cursor on charts with tooltip showing exact values
- Corner-zoom: click corner → zoom to ±100m range with 300ms ease-out transition
- Lap time format: `M:SS.mmm` (e.g., `1:32.456`)
- Delta times: `+0.234` (red) or `-0.156` (green) with sign always shown
- Chart color coding: current lap (amber), comparison lap (blue), best reference (purple dashed), danger zones (red shaded)

**UXR8: Notification & Feedback**
- One tray notification per completed session (never more)
- Zero notifications during active racing (iRacing fullscreen detected)
- Toast stack max 2 visible, newest on top, auto-dismiss after 4s (success) or persist (errors)
- AI streaming feedback: word-by-word with blinking amber cursor, skeleton lines before first token
- Error feedback: inline alert below trigger element with red left-border, includes what went wrong + what to do next

### FR Coverage Map

This map will be populated during epic design to ensure every functional requirement is addressed by at least one story.

| Requirement | Epic(s) | Story ID(s) | Status |
|-------------|---------|-------------|--------|
| FR1-FR41    | TBD     | TBD         | TBD    |
| NFR1-NFR28  | TBD     | TBD         | TBD    |
| AR1-AR10    | TBD     | TBD         | TBD    |
| UXR1-UXR8   | TBD     | TBD         | TBD    |

## Epic List

To be populated in Step 02 after user confirmation of requirements extraction.

