# Implementation Readiness Corrections - Batch Proposal
**Date:** 2026-02-05
**Status:** Proposed Changes (Batch Mode)
**Workflow:** BMAD Correct-Course
**Source:** implementation-readiness-report-2026-02-05.md

## Executive Summary

This document proposes corrections for **6 critical/major implementation blockers** identified in the implementation readiness assessment. All corrections maintain epic independence, preserve user value, and align with PRD governance.

**Readiness Status:** Currently **NOT READY** → Target: **READY FOR IMPLEMENTATION**

**Issues Addressed:**
- 3 Critical Issues (FR traceability, forward dependencies, scope creep)
- 3 Major Issues (missing scaffold, technical stories, blocking conflict)

---

## Critical Issue #1: FR Traceability Mismatch

### Problem Statement

**Finding:** PRD defines 43 Functional Requirements (FR1-FR41 with FR1/FR1a/FR1b split), but `epics-and-stories.md` redefines these same FR numbers with completely different meanings.

**Evidence:**

**PRD Definition (prd.md:601-605):**
```
- FR1: System can automatically detect when iRacing is running and begin
  telemetry capture without user action [MVP] (J1, J2, J3)
- FR1a: System can detect session end conditions (session state change,
  extended off-track timeout, iRacing process exit) and trigger debrief
  generation [MVP] (J1, J2, J3)
- FR1b: User can manually trigger debrief analysis at any point during
  or after a session [MVP] (J1, J3)
```

**Epics Redefinition (epics-and-stories.md - Epic 2 section):**
```
Epic 2 redefines:
- FR1: "System can automatically detect when iRacing is running [MVP]"
- FR1a: "System connects to iRacing shared memory (IRSDK) without user
  configuration [MVP]"
- FR1b: "System maintains IRSDK connection during active sessions and
  gracefully handles disconnects [MVP]"
```

**Impact:**
- Breaks requirement traceability across documents
- Invalidates FR coverage mapping (FR Coverage Map becomes meaningless)
- Creates confusion about which FRs are actually implemented
- Violates single source of truth principle

### Root Cause

Epics document was created independently and attempted to create its own FR numbering scheme instead of referencing the canonical PRD FRs.

### Proposed Correction

**Action:** Remove all FR redefinitions from `epics-and-stories.md`. Reference PRD FRs only.

**Changes Required:**

1. **Delete FR definition sections** from each epic header (Epic 1-5)
2. **Update "Mapped FRs" lines** to reference PRD FRs without redefining them
3. **Update story acceptance criteria** to cite PRD FR numbers for traceability

**Example Before (Epic 2):**
```markdown
**Mapped FRs:**
- FR1: System can automatically detect when iRacing is running [MVP] (J1, J2)
- FR1a: System connects to iRacing shared memory (IRSDK) without user configuration [MVP]
- FR1b: System maintains IRSDK connection during active sessions [MVP]
[... 11 FRs total with full definitions]
```

**Example After (Epic 2):**
```markdown
**Mapped FRs:** FR1, FR1a, FR1b, FR2, FR3, FR4, FR5, FR8, FR9, FR40, FR41 (11 FRs)
**Note:** See PRD for complete FR definitions and acceptance criteria.
```

**Verification:**
- All FR references point to PRD definitions
- FR Coverage Map uses PRD FR numbers
- No duplicate or conflicting FR definitions exist

**Governance:** PRD is the single source of truth for all Functional Requirements.

---

## Critical Issue #2: Forward Epic Dependencies

### Problem Statement

**Finding:** Epic 2 and Epic 3 stories contain acceptance criteria that directly reference UI features from later epics (Epic 4 and Epic 5), creating forward dependencies that violate epic independence.

**Evidence:**

**Story 2.1 (Epic 2) → Epic 5 Dependency (epics-and-stories.md:558):**
```
Then the app displays "Waiting for iRacing" in the system tray
```
- Epic 2 (Telemetry Capture) directly specifies Epic 5 (System Tray) UI implementation

**Story 3.5 (Epic 3) → Epic 4 Dependency (epics-and-stories.md:849):**
```
And a blinking amber cursor ▊ indicates active streaming
```
- Epic 3 (AI Coaching) directly specifies Epic 4 (Frontend UI) streaming presentation

**Impact:**
- Breaks epic independence and sequential development
- Epic 2 cannot be completed without Epic 5 system tray implementation
- Epic 3 cannot be completed without Epic 4 streaming UI implementation
- Violates event-based decoupling architecture

### Architecture Solution (Already Documented)

**Event-Based Decoupling (architecture.md:225-230):**

```
Cross-Epic Decoupling: Event-based architecture to maintain epic independence

- Epic 2→5: Rust backend emits domain events (connection_status_changed,
  session_state_changed, debrief_ready) instead of directly calling tray UI.
  Epic 5 subscribes to these events.

- Epic 3→4: AI backend publishes data contracts (coaching_chunk with
  {type, text, metadata}, comparison_data with structured diff) via events.
  Epic 4 UI consumes these events.

This ensures Epic 2 and Epic 3 have zero forward dependencies on Epic 4/5,
maintaining strict epic sequencing.
```

### Proposed Correction

**Action:** Remove UI implementation details from Epic 2/3 stories. Keep event emission requirements. Move UI implementation details to Epic 4/5 stories.

**Changes Required:**

#### Story 2.1 Modifications

**Remove (Line 558):**
```gherkin
Then the app displays "Waiting for iRacing" in the system tray
```

**Keep (Lines 564-565):**
```gherkin
And system emits connection_status_changed event (connected/disconnected/error)
And a Tauri event "capture:irsdk-connected" is emitted
```

**Add to Story 5.1 (System Tray State Management):**
```gherkin
Given the system receives connection_status_changed event with status "disconnected"
When the event is processed
Then system tray displays "Waiting for iRacing" tooltip
And tray icon shows idle state indicator
```

#### Story 3.5 Modifications

**Remove (Line 849):**
```gherkin
And a blinking amber cursor ▊ indicates active streaming
```

**Keep (Line 848):**
```gherkin
And coaching text streams as SSE events (coaching_chunk) with chunk_id sequence
```

**Add to Story 4.2 (Coaching Panel Streaming):**
```gherkin
Given coaching_chunk events are streaming
When tokens are being received
Then display a blinking amber cursor ▊ at the insertion point
And cursor blinks at 1Hz frequency while streaming is active
And cursor is removed when streaming completes
```

**Verification:**
- Epic 2 stories only emit events, no UI references
- Epic 3 stories only emit events, no UI references
- Epic 4/5 stories subscribe to events and implement UI
- Zero forward dependencies in epic sequence

---

## Critical Issue #3: Confidence Scoring Scope Creep

### Problem Statement

**Finding:** Story 3.6 "Confidence Scoring & Grounding" implements confidence indicators, which are explicitly scoped to **Growth (Phase 2)** in the PRD, not MVP.

**Evidence:**

**PRD Phase Classification (prd.md:116, 199):**
```
Growth (Phase 2): ... AI confidence indicators. 13 capabilities.

| AI confidence indicators | Growth | | X | | | |
```

**PRD Explicit Phasing (prd.md:221):**
```
At launch, confidence indicators are heuristic (based on sample size and
data variance). True calibration — validating that high-confidence advice
correlates with actual improvement — requires tracking outcomes over time
and is a Growth-phase capability.
```

**Story 3.6 Implementation (epics-and-stories.md:874-883):**
```gherkin
Then the app computes a confidence score (high/medium/low) based on:
- Data completeness (% of laps with clean telemetry for that corner)
- Consistency of finding (does the pattern hold across multiple laps?)
- Magnitude of delta (is the time loss significant?)

Given an insight has low confidence (<60% data completeness OR inconsistent pattern)
When the insight is displayed
Then it appears with a subtle low-confidence indicator (muted color, optional icon)
And the user can dismiss the insight
And dismissal is logged for future AI improvement
```

**Impact:**
- Adds Growth phase complexity to MVP scope
- Increases MVP development effort and risk
- Confidence indicators depend on historical data (session-over-session comparison)
- Dismissal logging suggests analytics infrastructure not in MVP scope

### Proposed Correction

**Action:** Defer Story 3.6 confidence scoring to Growth phase. Retain only telemetry grounding (FR13) which IS in MVP.

**Changes Required:**

#### Remove Story 3.6 from Epic 3

**Move to Growth Phase Backlog:**
```markdown
### Story 3.6: Confidence Scoring & Grounding [DEFERRED TO GROWTH]

**Rationale:** Confidence indicators are Growth phase per PRD:116, PRD:221.
Requires historical data tracking and outcome validation not in MVP scope.

**Dependencies:** Session history analytics, dismissal tracking, multi-session
comparison infrastructure.
```

#### Add Simplified Story 3.6a to Epic 3 (MVP)

**New Story 3.6a: Telemetry Grounding & Verification:**
```markdown
### Story 3.6a: Telemetry Grounding & Verification

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
And the referenced data is highlighted in the chart
And the user can see the raw data that supports the claim

**Given** historical context is available (multiple sessions on same track)
**When** coaching compares current session to previous sessions (FR16)
**Then** comparison data includes: current_session metrics, reference_session
  metrics, delta values
And the user can see both sessions' telemetry overlaid for visual comparison
```

**Mapping:**
- Story 3.6a covers: FR13 (telemetry grounding) and FR16 (session comparison)
- Story 3.6 (deferred) covers: Growth phase confidence indicators
- Lines 885-889 from original Story 3.6 are preserved in Story 3.6a

**Verification:**
- No Growth phase features in MVP epic stories
- FR13 and FR16 (MVP requirements) are still covered
- Confidence scoring deferred until Growth with proper infrastructure

**Alternative (if business decides confidence is critical to MVP):**
Update PRD to explicitly move "AI confidence indicators" from Growth to MVP phase and document the scope increase rationale.

---

## Major Issue #4: Missing Project Scaffold Story

### Problem Statement

**Finding:** Architecture mandates specific foundational project setup (create-tauri-app, Cargo workspace structure, Tailwind configuration), but no Epic 1 story covers this prerequisite work.

**Evidence:**

**Architecture Implementation Sequence (architecture.md:254-256):**
```
Implementation Sequence:
1. Project scaffold (create-tauri-app) → establishes build pipeline
2. Tailwind + shadcn/ui → enables UI development
3. Cargo workspace restructure → enables parallel Rust crate development
```

**Current Epic 1 Stories:**
- Story 1.1: "Desktop Application Runs Locally" — assumes app already exists
- Story 1.2-1.7: Data storage and session management — assumes infrastructure exists
- Story 1.8: "Automated Build & Test Pipeline" — assumes project already scaffolded

**Impact:**
- No story guides initial project creation
- Developers have no acceptance criteria for Cargo workspace structure
- Tailwind + shadcn/ui setup is undocumented
- First story (1.1) has unmet prerequisites

### Proposed Correction

**Action:** Add Story 1.0 "Project Foundation & Build Setup" as the first story in Epic 1.

**Changes Required:**

#### Insert New Story 1.0 Before Story 1.1

```markdown
### Story 1.0: Project Foundation & Build Setup

As a developer,
I want the foundational project structure established,
So that all subsequent development has a consistent build and development environment.

**Acceptance Criteria:**

**Given** starting from an empty repository
**When** the project scaffold is initialized
**Then** Tauri 2.0 project is created using create-tauri-app CLI (v2.6.0+)
And Tauri capabilities are configured with minimal permissions (principle of least privilege)
And the default "Hello Tauri" window renders successfully

**Given** the project scaffold exists
**When** Cargo workspace is configured
**Then** workspace includes 4 crates: src-tauri (binary), telemetry-engine (lib), ai-provider (lib), storage (lib)
And dependency graph is validated: src-tauri depends on all 3 libs, telemetry-engine and ai-provider depend on storage, storage has no dependencies
And `cargo build` completes successfully for all crates
And `cargo test` runs successfully (even if no tests exist yet)

**Given** the frontend structure exists
**When** Tailwind CSS 4.x is configured
**Then** Tailwind processes successfully during build
And PostCSS configuration is validated
And custom design tokens are configured per UX spec (colors: bg-base, bg-surface, text-primary, accent-primary, etc.)
And JetBrains Mono and Inter fonts are loaded

**Given** the UI component system is configured
**When** shadcn/ui is initialized
**Then** components directory is created with owned source (not npm dependency)
And Radix UI primitives are configured
And example component (e.g., Button) renders successfully in dev mode
And dark theme is set as default (no light mode at MVP)

**Given** local development environment is configured
**When** developer runs `npm run tauri dev`
**Then** Vite dev server starts on port 1420
And Tauri window opens with hot-reload enabled
And Rust changes trigger recompilation
And frontend changes trigger hot-reload without full rebuild
And build completes in <30 seconds for incremental changes

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
```

**Renumber Existing Stories:**
- Story 1.1 → Story 1.2 (Desktop Application Runs Locally)
- Story 1.2 → Story 1.3 (Session History Persists Locally)
- ... continue renumbering through Story 1.8 → Story 1.9

**Verification:**
- Story 1.0 is now the first story in Epic 1
- All Architecture Implementation Sequence steps 1-3 are covered
- Story 1.2 (formerly 1.1) now has its prerequisites met
- Build pipeline establishment is explicit and testable

---

## Major Issue #5: Technical-Only Stories Lack User Value

### Problem Statement

**Finding:** Stories 1.8 (Build Pipeline) and 4.6 (Data Contract) are written from developer perspective with no connection to end-user value, violating BMAD's "user stories deliver user value" principle.

**Evidence:**

**Story 1.8 (epics-and-stories.md:518-532):**
```markdown
As a developer,
I want every code change automatically built and tested,
So that integration issues are caught immediately.
```
- Written for developer persona, not sim racer user
- Describes technical infrastructure, not user-facing value

**Story 4.6 (epics-and-stories.md:1069-1082):**
```markdown
As a developer integrating with the system,
I want debrief output structured as JSON/serializable data,
So that I can programmatically consume and export coaching.
```
- Written for developer/integrator persona
- Describes data contract, not user benefit

**Impact:**
- Stories don't connect technical work to user outcomes
- Unclear why these features matter to end users
- Risk of implementing technical solutions without validating user need

### Proposed Correction

**Action:** Reframe stories with user value connection OR mark as technical enablers with explicit story linkage.

**Changes Required:**

#### Option A: Reframe Story 1.8 with User Value

```markdown
### Story 1.8: Reliable Updates Without Breaking Changes

As a sim racer,
I want confidence that app updates won't break my workflow,
So that I can install new versions safely without fear of losing functionality.

**Acceptance Criteria:**

**Given** I'm using a stable version of the app
**When** a new version is released
**Then** I can trust that core functionality (capture, debrief, history) still works
And breaking changes are caught before release
And I can see what was tested before installation

**Technical Implementation (CI/CD Pipeline):**

**Given** code changes are pushed to any branch
**When** the push completes
**Then** GitHub Actions automatically builds the project for Windows
And all tests run within 5 minutes
And build status shows on pull request
And failed builds block merging
And successful builds produce downloadable artifacts
And test coverage report is generated
```

**User Value Connection:** Users get reliable software updates without fear of breakage.

#### Option B: Mark as Technical Enabler

```markdown
### Story 1.8: Automated Build & Test Pipeline [TECHNICAL ENABLER]

**Purpose:** Enable reliable delivery of all Epic 1-5 user-facing stories.

**Enabled Stories:** All 37 user stories depend on this build infrastructure.

As a developer,
I want every code change automatically built and tested,
So that user-facing stories ship without integration regressions.

[... rest of acceptance criteria unchanged ...]

**User Impact:** Prevents bugs from reaching users by catching integration issues early.
```

#### Option A: Reframe Story 4.6 with User Value (Growth Phase)

```markdown
### Story 4.6: Export Debrief for External Analysis [GROWTH PHASE]

As a sim racer,
I want to export my debrief data to other tools,
So that I can use third-party analysis software or share structured data with teammates.

**Acceptance Criteria:**

**Given** I have completed a debrief
**When** I click "Export Debrief"
**Then** a JSON file downloads with structured debrief data
And the export includes: insights, recommendations, metrics, session metadata
And I can import this data into spreadsheet tools (Excel, Google Sheets)
And I can share the file with teammates for collaborative analysis

**Technical Implementation (Data Contract):**

**Given** debrief analysis completes
**When** debrief data is serialized
**Then** output follows documented JSON schema
And schema version is included for compatibility tracking
And file format is human-readable (pretty-printed JSON)
```

**User Value Connection:** Users can integrate with their own workflows and tools.

#### Option B: Move Story 4.6 to Growth or Remove

**Recommendation:** Story 4.6 export functionality is Growth phase. Defer unless user export is MVP requirement.

If deferred:
```markdown
### Story 4.6: Export Debrief for External Analysis [DEFERRED TO GROWTH]

**Rationale:** Data export is a Growth phase power-user feature. MVP focuses on
in-app debrief experience. JSON data contract exists internally for IPC but
user-facing export is not MVP critical path.

**Dependencies:** API stabilization, schema versioning, user testing of export formats.
```

**Verification:**
- All user stories connect to end-user value
- Technical enablers are explicitly marked and linked to enabled stories
- Growth phase features are properly deferred with rationale

---

## Major Issue #6: Story 3.1 Blocks Capture on AI Failure

### Problem Statement

**Finding:** Story 3.1 validates API connectivity "before each session starts" and "validation failure blocks session start," which violates the graceful degradation pattern and creates an Epic 2 (Telemetry) dependency on Epic 3 (AI).

**Evidence:**

**Story 3.1 Problematic Criteria (epics-and-stories.md:759-761):**
```gherkin
And system validates API connectivity before each session starts
And validation failure blocks session start with clear error message
And pre-session validation completes within 2 seconds
```

**PRD Graceful Degradation Requirement (prd.md:327, 389, NFR10):**
```
🔴 AI offline — capture continues, analysis unavailable

MVP graceful degradation: If BYOK provider is unavailable, the app falls
back to raw pre-processed stats (corner summaries, lap comparisons, derived
metrics). Capture and data review always work regardless of provider status.

NFR10: Graceful degradation on AI provider failure | Capture, pre-processing,
and existing debriefs remain fully functional
```

**Impact:**
- Violates NFR10 (graceful degradation requirement)
- Creates backward dependency: Epic 3 (AI) blocks Epic 2 (Capture)
- Users cannot capture telemetry if AI is temporarily unavailable
- Contradicts "zero-config" and "local-first" design principles

### Root Cause

Story conflates API key setup (one-time configuration) with per-session validation (ongoing runtime check). Per-session validation should not block capture.

### Proposed Correction

**Action:** Modify Story 3.1 to validate API key at configuration time only. Remove per-session validation that blocks capture. Add graceful degradation behavior.

**Changes Required:**

#### Remove Problematic Lines from Story 3.1

**Delete (Lines 759-761):**
```gherkin
And system validates API connectivity before each session starts
And validation failure blocks session start with clear error message
And pre-session validation completes within 2 seconds
```

#### Add Graceful Degradation Criteria to Story 3.1

**Add after line 752:**
```gherkin
**Given** I have configured a valid API key
**When** a session ends and debrief generation begins
**Then** the system validates API connectivity at debrief time (not session start time)
And if API is unavailable, displays "AI coaching unavailable - showing telemetry analysis only"
And user can still view telemetry charts, lap comparisons, and derived metrics (FR9)
And user can retry AI analysis later via "Retry AI Coaching" button

**Given** I have skipped API key setup (local-only mode)
**When** a session ends
**Then** debrief shows raw pre-processed stats without AI coaching
And debrief includes corner summaries, lap comparisons, derived metrics (NFR10)
And user sees "Add API key for AI coaching" prompt with link to settings
And telemetry capture and data review work fully without AI
```

#### Verification Acceptance Criteria

**Add to Story 3.1:**
```gherkin
**Given** my API key becomes invalid mid-session (expired, rate limit, network issue)
**When** session ends and debrief generation runs
**Then** telemetry capture completed successfully (not blocked)
And pre-processing completed successfully (corner detection, lap analysis)
And debrief view shows local stats immediately
And AI coaching section shows error state: "Unable to connect to AI provider"
And user can retry AI analysis after fixing the issue
And existing debriefs remain accessible regardless of current API status (NFR10)
```

**Testing Note:** Integration test should disable AI provider, run full capture session, verify all non-AI functions work per NFR10.

**Verification:**
- No session-start blocking on AI validation
- Telemetry capture is independent of AI provider status
- Graceful degradation to local stats when AI unavailable
- NFR10 compliance verified via integration test

---

## Implementation Checklist

**Before applying these corrections:**
- [ ] Review all 6 corrections with product owner / stakeholder
- [ ] Confirm scope decisions (especially Story 3.6 deferral to Growth)
- [ ] Validate that PRD remains single source of truth for FRs
- [ ] Verify epic independence is maintained (no forward dependencies)
- [ ] Confirm graceful degradation pattern is preserved

**After applying corrections:**
- [ ] Update epics-and-stories.md with all 6 corrections
- [ ] Validate FR Coverage Map references PRD FRs correctly
- [ ] Verify all stories have clear user value or enabler marking
- [ ] Run readiness assessment again to confirm READY status
- [ ] Update sprint planning to include Story 1.0 as first task

---

## Risk Assessment

| Correction | Risk | Mitigation |
|------------|------|------------|
| #1: Remove FR redefinitions | Documentation confusion during transition | Update all references simultaneously, clear commit message |
| #2: Remove forward dependencies | UI details might be lost | Explicitly add UI criteria to Epic 4/5 stories before removing from Epic 2/3 |
| #3: Defer confidence scoring | Stakeholder expectation mismatch | Confirm Growth deferral with product owner before implementing |
| #4: Add Story 1.0 scaffold | Story renumbering affects references | Use find/replace to update all "Story 1.X" references in documentation |
| #5: Reframe technical stories | Acceptance criteria become longer | Use "Technical Implementation" subsections to separate user value from details |
| #6: Remove AI blocking | Perception that AI is "optional" | Emphasize AI as core value prop, but with graceful degradation for reliability |

---

## Next Steps

1. **Review & Approval:** Product owner reviews this correction proposal
2. **Apply Changes:** Update epics-and-stories.md with approved corrections
3. **Validation:** Run implementation readiness assessment again
4. **Sprint Planning:** Begin Epic 1 development starting with Story 1.0

**Expected Outcome:** Implementation readiness status changes from **NOT READY** to **READY FOR IMPLEMENTATION**.

---

**Document Status:** Proposed corrections in batch mode, pending approval.
