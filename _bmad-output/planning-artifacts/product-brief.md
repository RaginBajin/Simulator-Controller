# Product Brief: AI Race Team for Sim Racers

**Author:** Joe
**Date:** 2026-02-01
**Status:** Discovery Complete (Party Mode Session)
**Origin:** Strategic pivot from Simulator Controller contribution model to greenfield product

---

## Product Vision

An AI-powered race team application that gives every sim racer a professional-grade AI pit crew — engineer, strategist, spotter, and coach — with full ownership of their data. Built for racers who want to improve, not configure software.

**One-sentence pitch:** "Your AI crew chief that sees what you can't, speaks when it matters, and never locks away your data."

---

## Strategic Context

### Why This Exists

Joe forked Simulator Controller (250K LOC, AutoHotkey-based) to propose open source modernization. After extensive analysis (15 elicitation methods, comprehensive PRD), the strategic conclusion was clear:

- Contributing to the existing codebase carries high burnout risk (70% estimated)
- AutoHotkey testing is near-impossible to do properly
- The maintainer may reject contributions regardless of quality
- The real value isn't the existing code — it's the concept: AI assistants for sim racing

**Decision:** Build a new product from scratch that does the same thing better, in a modern stack, owned entirely by Joe.

### What Already Works (Proof of Concept)

Joe has been using Claude Desktop as an AI racing coach:
- Captures race telemetry data from iRacing
- Feeds it to Claude for analysis
- Gets coaching on technique (trail braking, brake pressure, racing line)
- **What works:** The AI analysis is genuinely useful and actionable
- **What's missing:** No visualization, no real-time capability, no way to share insights, manual process

---

## Target Users

### Primary: Joe (Dogfooding)
- Experienced sim racer, iRacing focused
- Technical enough to provide feedback and iterate
- Wants AI coaching with visual data, not just text

### Secondary: Competitive Sim Racers
- League racers who want data-driven improvement
- Frustrated by existing tools (Coach Dave locks data, Garage61 is limited, VRS is confusing and upsell-heavy)
- Want to own and share their racing data

### Tertiary: Casual Sim Racers
- Want to improve but don't know how to read telemetry
- Need the AI to explain things visually, not just in numbers

---

## Product Modes

### Mode 1: Debrief Mode (MVP — Ships First)
Post-session analysis with rich visualization and AI coaching conversation.

**User flow:**
1. Race in iRacing (telemetry recorded automatically)
2. Open debrief in desktop app
3. AI analyzes session: braking zones, racing line, tire management, pit strategy
4. Visual telemetry traces with AI annotations ("you braked 15m too late here")
5. Conversational follow-up ("explain why my corner exit speed is low at Turn 3")
6. Share session debrief via web link

**Why first:** Already validated with Claude Desktop. No real-time latency pressure. Visual output is the key gap. Standalone value proposition.

### Mode 2: Race Mode (Post-MVP)
Live AI crew chief during racing sessions.

**User flow:**
1. Launch app, start racing
2. AI speaks during the race: pit strategy, tire warnings, gap analysis
3. Voice interaction: ask questions, get immediate answers
4. Post-race transitions into Debrief Mode automatically

**Why second:** Real-time latency constraints are hard. Needs pre-computed insights and queued responses. Requires solving voice I/O during racing (noise, timing, cognitive load).

---

## Technical Architecture

### Stack
- **Backend:** Rust (core engine — telemetry reading, data processing, AI interface)
- **Desktop App:** Tauri (lightweight native shell, same web frontend)
- **Frontend:** React/TypeScript (shared between Tauri desktop and web dashboard)
- **Database:** SQLite (session metadata, AI analysis results, user data)
- **Telemetry Storage:** Parquet files (raw telemetry time-series data)
- **AI:** Claude API (or similar LLM) for coaching analysis

### Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                  Rust Core Engine                     │
│   Telemetry Reader │ Session Store │ AI Interface    │
│   (iRacing IRSDK)  │ (SQLite +     │ (LLM API)      │
│                     │  Parquet)     │                 │
├─────────────────────┴───────────────┴─────────────────┤
│              Local API (HTTP + WebSocket)              │
├──────────────────────┬────────────────────────────────┤
│   Desktop App        │   Web Dashboard                │
│   (Tauri)            │   (Same React codebase)        │
│                      │                                │
│   - Race recording   │   - Debrief review             │
│   - Live debrief     │   - Session sharing            │
│   - Voice I/O        │   - Historical trends          │
│   - System tray      │   - Mobile-friendly            │
└──────────────────────┴────────────────────────────────┘
```

### Data Architecture

```
data/
├── sessions.db                              ← SQLite: metadata, laps, AI results
├── sessions/
│   ├── 2026-02-01_monza_race.parquet        ← Raw telemetry (columnar, compressed)
│   ├── 2026-02-01_monza_race.meta.json      ← Session context
│   └── ...
```

- SQLite is the index, Parquet is the warehouse
- Both are open formats, single files, fully portable
- SQLite can run in browser via WASM (sql.js) for web dashboard
- No server required for local use

### Why Tauri Over Electron
- Uses OS WebView instead of bundling Chromium (5-10MB vs 150MB+)
- Rust backend is already the core engine — natural fit
- Lower RAM footprint matters on sim racing rigs
- Same React frontend code powers both desktop and web

---

## Simulator Support Roadmap

| Priority | Simulator | Telemetry Method | Notes |
|----------|-----------|------------------|-------|
| **V1** | iRacing | IRSDK (shared memory) | Best telemetry API, largest community |
| **V2** | ACC | Shared memory | Strong competitive racing community |
| **V3** | LMU | Shared memory | Growing endurance racing sim |

---

## Competitive Positioning

### "Your Data, Your Way, Anywhere"

| Competitor | Data Access | AI Coaching | Live Racing | Weakness |
|---|---|---|---|---|
| **Coach Dave** | Locked in app | Basic analysis | No | Data hostage |
| **Garage61** | Limited export | None | No | Incomplete data |
| **VRS** | Confusing UI | Human coaches ($$$) | No | Upsell-heavy, poor UX |
| **This Product** | Open + shareable | AI-powered | Future | New entrant |

### Three Differentiation Pillars
1. **AI-native coaching** — LLM-powered analysis, not basic stats or expensive human coaches
2. **Your data is yours** — Open formats (SQLite + Parquet), portable, shareable web links
3. **Live AI crew chief** (future) — Real-time voice interaction during racing

---

## Key Risks

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| LLM latency too high for live mode | Medium | High | Debrief mode first; pre-compute insights for live mode |
| iRacing IRSDK changes between seasons | Medium | Medium | Versioned telemetry parser, recorded session tests |
| AI coaching quality inconsistent | Medium | High | Prompt engineering iteration, telemetry-grounded responses |
| Scope creep beyond MVP | High | High | One sim, one mode, ship it |
| Visualization complexity | Medium | Medium | Start with basic traces, iterate on richness |

---

## MVP Success Criteria

1. Record an iRacing session's telemetry automatically
2. AI produces actionable coaching insights with visual annotations
3. User can have a follow-up conversation about the analysis
4. Session debrief is accessible via web browser (shareable link)
5. Joe uses it for his own racing and finds it more useful than Claude Desktop alone

---

## What This Is NOT

- NOT a rewrite of Simulator Controller
- NOT trying to support 11 simulators at launch
- NOT a configuration-heavy tool with dozens of settings
- NOT a telemetry-only viewer (the AI coaching is the product)
- NOT locked-down data (open formats always)

---

## Origin Artifacts

- **Previous PRD (archived):** `prd-v1-simulator-controller-contribution.md`
  - Contains extensive user persona research, journey mapping, and elicitation findings
  - Some insights (user pain points, competitive analysis, innovation patterns) may inform the new PRD
  - Strategic approach (contribution model) is deprecated
- **Party Mode Session:** 2026-02-01 — Strategic pivot discussion with all BMAD agents
- **Claude Desktop Usage:** Joe's existing AI coaching workflow as proof of concept
