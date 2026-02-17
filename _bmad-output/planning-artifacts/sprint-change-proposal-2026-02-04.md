# Sprint Change Proposal

**Date:** 2026-02-04
**Project:** Simulator-Controller (AI Race Team)
**Trigger:** Implementation Readiness Assessment - 34 Issues Identified
**Overall Status:** NOT READY → Requires Course Correction
**Approval Status:** ✅ APPROVED
**Date Approved:** 2026-02-04

---

## Executive Summary

The Implementation Readiness Assessment identified **34 critical issues** across 6 categories that block implementation:

- **10 Missing FRs:** PRD requirements with no corresponding epic/story coverage
- **12 Misaligned FRs:** PRD requirements partially covered but missing key aspects
- **2 PRD↔UX Conflicts:** Direct contradictions requiring product decisions
- **Epic Independence Violations:** Epic 2/3 depend on Epic 4/5 (forbidden)
- **4 Technical Stories:** Developer tasks framed as user stories
- **Greenfield Setup Gaps:** No CI/CD pipeline story

**Proposed Solution:** This document proposes **specific, surgical changes** to epics and stories that resolve all 34 issues while minimizing disruption.

**Impact Summary:**
- **New Stories Added:** 9 stories (8 for Missing FRs + 1 for CI/CD)
- **Story Modifications:** 16 stories (12 Misaligned + 4 Technical reframings)
- **Story Moves/Splits:** 1 story moved (3.7 → 4.7), 1 story split (5.5 → 5.5/5.8/5.9)
- **Product Decisions Required:** 2 (manual trigger conflict + WCAG accessibility conflict)
- **Net Story Count Change:** +11 stories (26 → 37)

**Recommendation:** **Approve with Product Owner decisions on 2 conflicts.** Changes are necessary to achieve implementation readiness.

**Decisions Made:**
- ✅ **Decision 1 (Manual Trigger):** Approved tray-menu trigger approach
- ✅ **Decision 2 (WCAG Accessibility):** Approved Option B (Basic WCAG AA: keyboard + contrast)

---

## Impact Assessment

### Severity Breakdown

| Category | Count | Severity | Implementation Blocking? |
|----------|-------|----------|-------------------------|
| Missing FRs | 10 | 🔴 Critical | Yes - incomplete product |
| Misaligned FRs | 12 | 🟠 High | Yes - incorrect implementation |
| PRD↔UX Conflicts | 2 | 🔴 Critical | Yes - contradictory requirements |
| Epic Independence | 2 violation categories | 🔴 Critical | Yes - untestable epics |
| Technical Stories | 4 | 🟡 Medium | No - but violates user-value principle |
| Greenfield Gaps | 1 | 🟠 High | No - but blocks CI automation |

### Coverage Improvement

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| PRD FRs Covered | 21/43 (49%) | 43/43 (100%) | +51% |
| Story Count | 26 | 37 | +11 |
| Implementation Ready | ❌ NO | ✅ YES (pending decisions) | ✓ |

---

## Detailed Changes

### Category 1: Missing FRs (10 Issues → 8 New Stories)

**Context:** 10 PRD Functional Requirements have no epic/story coverage.

**Proposed New Stories:**

#### **Epic 1: Persistent Data Storage & Session History**

**Story 1.7: Historical Session Import**
**Covers:** FR10 (import historical telemetry from archived session files)

```
As a user
I want to import telemetry from past racing sessions
So that I can analyze historical performance alongside new data

Given I have archived session files (.ibt, .csv, or app export format)
When I import a historical session
Then session data loads into local database
And imported sessions appear in session history
And imported sessions are analyzable like live sessions
```

---

**Story 1.8: Automated Build & Test Pipeline** *(CI/CD Gap)*
**Covers:** Greenfield CI/CD requirement

```
As a developer
I want every code change automatically built and tested
So that integration issues are caught immediately

Given I push code changes to any branch
When the push completes
Then GitHub Actions automatically builds the project for Windows
And all tests run within 5 minutes
And build status shows on pull request
And failed builds block merging
And successful builds produce downloadable artifacts
```

---

#### **Epic 2: Telemetry Capture**

**Story 2.3b: Manual Debrief Trigger**
**Covers:** FR1b (manual debrief trigger)

```
As a user
I want to manually trigger debrief analysis anytime
So that I can analyze partial sessions or re-analyze completed sessions

Given a session is active or completed
When I select "Generate Debrief" from system tray menu
Then debrief analysis starts immediately
And debrief generation follows normal pipeline
And manual triggers work for incomplete sessions
```

---

**Story 2.6: Derived Metrics Engine**
**Covers:** FR9 (computed derived metrics)

```
As a user
I want advanced metrics computed automatically
So that I get deeper insights beyond raw telemetry

Given session telemetry is captured
When derived metrics engine processes data
Then brake application count is computed per lap
And trail braking phases are identified and quantified
And tire degradation curves are calculated
And derived metrics are stored with session data
```

---

**Story 2.7: Session Type Capture**
**Covers:** FR40 (detect iRacing session type)

```
As a user
I want session type automatically detected
So that analysis is contextually relevant (practice vs race)

Given iRacing session is active
When telemetry capture starts
Then session type is detected (practice/qualifying/race/warmup)
And session type is stored with session metadata
And session type is visible in UI and used for filtering
```

---

#### **Epic 3: AI Coaching**

**Story 3.8: Car Template System**
**Covers:** FR13 (car-class-specific coaching)

```
As a user
I want coaching tailored to my specific car class
So that advice is relevant to my vehicle's characteristics

Given I'm driving a specific car class (GT3, Formula, etc.)
When debrief analysis runs
Then AI uses car-class-specific coaching templates
And coaching references car-specific techniques (downforce, traction control, etc.)
And templates are configurable per car class
```

---

**Story 3.9: Anomaly & Incident Detection**
**Covers:** FR16 (diagnose incomplete laps)

```
As a user
I want the system to diagnose why laps were incomplete
So that I understand what went wrong (crash, disconnect, reset)

Given a lap ends prematurely
When analysis engine processes the lap
Then incident type is classified (spin/crash/disconnect/manual-reset)
And incident location is identified (track position, corner)
And incident telemetry is flagged for review
And coaching includes incident diagnosis in debrief
```

---

#### **Epic 4: Debrief Interface**

**Story 4.6: Debrief Data Contract**
**Covers:** FR30 (serializable debrief output)

```
As a developer integrating with the system
I want debrief output structured as JSON/serializable data
So that I can programmatically consume and export coaching

Given debrief analysis completes
When debrief data is generated
Then output follows documented JSON schema
And schema includes: insights, recommendations, metrics, metadata
And serialized format is exportable for external tools
```

---

#### **Epic 5: Desktop Shell**

**Story 5.6: Audio Notification System**
**Covers:** FR35 (visual + audio notifications) + FR41 (audio preferences)

```
As a user
I want audio notifications for key events
So that I'm alerted without watching the screen

Given a significant event occurs (session start, debrief ready, error)
When the event triggers
Then system plays audio notification (configurable sound)
And visual notification displays simultaneously
And I can configure audio on/off and volume in settings
And audio preferences persist across sessions
```

---

**Story 5.7: Application Auto-Update**
**Covers:** FR38 (check for and apply updates)

```
As a user
I want the application to update automatically
So that I always have the latest features and fixes

Given a new application version is released
When I launch the application
Then system checks for updates
And update notification displays if available
And I can choose to install now or defer
And update downloads and installs with user confirmation
```

---

### Category 2: Misaligned FRs (12 Issues → 16 Story Modifications)

**Context:** 12 PRD FRs partially covered but missing key acceptance criteria.

**Proposed Acceptance Criteria Additions:**

#### **Epic 1 Modifications**

**Story 1.3: Parquet Telemetry Archive**
**Add for FR6 (local storage separation):**
- `And all data is stored locally on user's machine (no cloud dependency)`
- `And time-series telemetry uses Parquet format for analytical queries`
- `And metadata uses SQLite for structured queries`
- `And storage location is user-configurable with sensible default`

**Story 1.4: Session Query Interface**
**Add for FR31 (debrief persistence):**
- `And system stores debrief data (coaching text, insights, recommendations) in SQLite`
- `And debrief data is linked to session_id for retrieval`
- `And past debriefs are retrievable via session query interface`

**Story 1.5: Historical Progress Tracking**
**Add for FR33 (consistency + technique metrics):**
- `And system computes consistency improvement (lap time std deviation trend)`
- `And system computes technique change metrics (braking point variance, apex speed variance)`
- `And progress metrics are queryable by track, car class, and date range`

---

#### **Epic 2 Modifications**

**Story 2.2: Telemetry Channel Mapping**
**Add for FR2 (configurability + environmental context):**
- `And telemetry sample rate is configurable (default: 60Hz, range: 10-120Hz)`
- `And system captures environmental conditions (track temp, air temp, weather)`
- `And system captures session context (session type, car class, track config)`

**Story 2.3: Real-Time Session Processing**
**Add for FR3 (lap boundary detection):**
- `And system detects lap boundaries using iRacing lap distance crossing logic`
- `And lap boundary timestamps are recorded with <50ms precision`
- `And lap summaries are computed immediately upon boundary detection`

**Add for FR4 (incomplete-lap retention):**
- `And system preserves incomplete laps (spins, resets, disconnects) as diagnostic data`
- `And incomplete laps are flagged with completion_status metadata`

**Add for FR5 (gap markers):**
- `And system inserts gap markers on telemetry stream interruptions`
- `And gap markers record start_time, end_time, and reason (disconnect/reset/crash)`
- `And analysis correctly handles gaps without misinterpreting data continuity`

**Story 2.4: Session Lifecycle Management**
**Add for FR1a (session end → debrief trigger):**
- `And when session end is detected, system emits session_completed event with session_id`
- `And session_completed event triggers debrief generation pipeline`

---

#### **Epic 3 Modifications**

**Story 3.1: Provider Setup & Key Management**
**Add for FR21 (pre-session validation):**
- `And system validates API connectivity before each session starts`
- `And validation failure blocks session start with clear error message`
- `And pre-session validation completes within 2 seconds`

**Story 3.2: AI Provider Integration**
**Add for FR22 (provider dispatch explicit):**
- `And system dispatches AI requests to user's configured provider (Claude or OpenAI)`
- `And dispatch routing is determined by provider_type configuration`
- `And dispatch failures log provider name and error details`

**Story 3.3: Debrief Generation Pipeline**
**Add for FR1a (debrief trigger link):**
- `Given a session_completed event is received`
- `When the event contains valid session_id`
- `Then debrief generation begins automatically`

**Story 3.4: Analysis Engine Core**
**Add for FR4 (incomplete-lap analysis):**
- `And analysis engine processes incomplete laps for incident diagnosis`

---

#### **Epic 4 Modifications**

**Story 4.1: Debrief UI Foundation**
**Add for FR27 (per-lap structured stats):**
- `And Telemetry tab displays per-lap summary statistics in table format`
- `And per-lap table shows: lap#, time, sector times, top speed, avg speed, incidents`
- `And table is sortable and filterable`

---

#### **Epic 5 Modifications**

**Story 5.1: System Tray Integration**
**Add for FR24 (provider status visibility):**
- `And system tray icon displays provider connection status (ready/degraded/offline)`
- `And status updates in real-time on connectivity changes`
- `And tooltip shows provider name and last validation time`

---

### Category 3: PRD↔UX Conflicts (2 Issues → 2 Product Decisions Required)

**Context:** Direct contradictions between PRD and UX specification require product owner decisions.

#### **Conflict 1: Manual Debrief Trigger**

**Contradiction:**
- **PRD FR1b:** "User can manually trigger debrief analysis at any point"
- **UX Spec:** "No manual triggers, no import session / run analysis buttons"

**Proposed Resolution:**
- **Primary Flow:** Automatic debrief on session end (UX vision preserved)
- **Secondary Control:** Manual trigger in system tray menu only (satisfies FR1b)
- **UX Spec Update:** Revise "no manual triggers" to "no prominent manual triggers in main UI; minimal trigger in tray for power users"
- **Story Impact:** Story 2.3b scoped to tray menu integration

**Decision:** ✅ **APPROVED** - Tray-menu trigger approach approved (2026-02-04)

**Implementation Actions:**
- Story 2.3b: Manual Debrief Trigger (system tray menu integration)
- Update UX spec language: "No prominent UI triggers; power-user tray option available"
- PRD FR1b satisfied via tray menu control

---

#### **Conflict 2: WCAG Accessibility Requirement**

**Contradiction:**
- **PRD Non-Functional Requirements:** "Explicitly excludes formal accessibility targets at MVP"
- **UX Spec:** "WCAG 2.1 AA compliance mandatory (keyboard nav, screen reader, contrast)"
- **Architecture:** "Assumes WCAG AA compliance"

**Resolution Options:**

**Option A: Exclude WCAG AA** (follows PRD)
- Remove WCAG from UX spec and Architecture
- Document as post-MVP technical debt
- **Risk:** Significant rework cost later

**Option B: Basic WCAG AA** (compromise) ⭐ **RECOMMENDED**
- Include: Semantic HTML + keyboard nav + contrast compliance
- Defer: Screen reader optimization to post-MVP
- Update PRD to "Basic WCAG 2.1 AA: keyboard + contrast only"
- **Cost:** Minimal (good practice anyway)
- **Benefit:** Reduces post-MVP rework risk

**Option C: Full WCAG AA** (follows UX/Architecture)
- Full keyboard navigation + ARIA labels + screen reader support
- Update PRD to include WCAG AA
- **Cost:** +2-3 stories worth of effort

**Decision:** ✅ **APPROVED - Option B** (Basic WCAG AA) (2026-02-04)

**Implementation Actions:**
- Update PRD Non-Functional Requirements: "Basic WCAG 2.1 AA: keyboard navigation + contrast compliance"
- Update UX Spec: "Screen reader support deferred to post-MVP"
- Update Architecture: Align with Basic WCAG AA scope
- All UI stories to include: semantic HTML, keyboard navigation, contrast validation
- No additional stories required (embedded in existing UI stories)

---

### Category 4: Epic Independence Violations (2 Violation Categories → Event-Based Decoupling)

**Context:** Epic 2/3 stories reference Epic 4/5 UI components, violating independence principle.

#### **Violation Fix 1: Epic 2 → Epic 5 Dependencies**

**Problem:** Epic 2 stories reference system tray and notifications (Epic 5)

**Stories Affected:** 2.1, 2.4, 2.5

**Fix:** Replace UI references with event emissions

**Story 2.1: IRSDK Connection Manager**
- ❌ Remove: "And system tray shows connected/disconnected status"
- ✅ Add: "And system emits connection_status_changed event (connected/disconnected/error)"

**Story 2.4: Session Lifecycle Management**
- ❌ Remove: "And system tray icon shows recording/processing state"
- ✅ Add: "And system emits session_state_changed event (idle/recording/processing/completed)"

**Story 2.5: Background Processing Optimization**
- ❌ Remove: "And system displays notification when debrief is ready"
- ✅ Add: "And system emits debrief_ready event with session_id"

**Epic 5 Impact:**
**Story 5.1** subscribes to events and updates tray accordingly

---

#### **Violation Fix 2: Epic 3 → Epic 4 Dependencies**

**Problem:** Epic 3 stories reference debrief UI components (Epic 4)

**Stories Affected:** 3.5, 3.6, 3.7

**Story 3.5: Progressive Streaming Display**
- ❌ Remove: "And coaching text streams word-by-word into Summary tab"
- ✅ Replace: "And coaching text streams as SSE events (coaching_chunk) with chunk_id sequence"
- **Epic 4 Impact:** Story 4.1 subscribes to coaching_chunk events

**Story 3.6: Historical Context & Comparison**
- ❌ Remove: "And historical context enables Telemetry tab comparison"
- ✅ Replace: "And historical context provides comparison_data structure (current_session, reference_session, delta_metrics)"
- **Epic 4 Impact:** Story 4.3 consumes comparison_data

**Story 3.7: Persistent Chat Interface**
- **Move:** Story 3.7 → **Story 4.7 (new)**
- **Epic 3 Scope:** Keep only chat API endpoint (Question → Answer)
- **Epic 4 Scope:** Chat UI, message history, input handling

---

**Validation:** After fixes, Epic 2 and Epic 3 are fully testable without Epic 4/5 UI

---

### Category 5: Technical Stories (4 Issues → User Value Reframing)

**Context:** Stories framed as developer tasks rather than user outcomes.

#### **Story 1.1: Project Initialization & Tauri Scaffold**

**Current (Technical):** "Set up Tauri 2.0 workspace, Tailwind, shadcn, fonts"

**Reframed (User Value):** **"Desktop Application Runs Locally"**

```
As a user
I want the application to run as a Windows desktop app
So that I don't need browser configuration or internet dependency

Given the application is installed
When I launch the executable
Then a native desktop window opens
And the application runs entirely on my local machine
And no internet connection is required for core functionality
```

---

#### **Story 1.2: SQLite Session Database Schema**

**Current (Technical):** "Create SQLite schema for sessions, laps, metadata"

**Reframed (User Value):** **"Session History Persists Locally"**

```
As a user
I want all my racing sessions saved permanently on my computer
So that I can review past performance anytime without data loss

Given I have completed multiple racing sessions
When I close and reopen the application
Then all previous session data is still available
And session metadata loads within 100ms
And no data is lost between application restarts
```

---

#### **Story 3.2: AI Provider Integration**

**Current (Technical):** "Integrate Claude API + OpenAI API with streaming, retries"

**Reframed (User Value):** **"AI Coaching Uses My Preferred Provider"**

```
As a user
I want to use my own AI provider account (Claude or OpenAI)
So that I control costs and data privacy

Given I have configured my API key for Claude or OpenAI
When debrief analysis runs
Then coaching uses my selected provider
And my API key is stored securely in OS credential store
And I can switch providers without losing functionality
And provider errors show clear, actionable messages
```

---

#### **Story 5.5: IPC Command Catalog (Partial)**

**Current (Technical):** "Define 12 IPC commands, quit handling, crash recovery"

**Reframed (User Value):** **Split into 3 stories:**

**Story 5.5: Application State Syncs Seamlessly**
```
As a user
I want the UI to always reflect current system state
So that I know exactly what's happening (recording, processing, ready)

Given the backend state changes (e.g., starts recording)
When the state change occurs
Then the UI updates within 100ms
And state transitions are smooth without UI freezing
And error states display actionable messages
```

**Story 5.8: Safe Quit Without Data Loss** *(new)*
```
As a user
I want to quit the application anytime without losing data
So that I can close the app confidently

Given I quit during active recording or processing
When I click quit or close window
Then in-progress work saves before exit
And application quits within 2 seconds
And next launch shows no data corruption
```

**Story 5.9: Crash Recovery Preserves Sessions** *(new)*
```
As a user
I want my session data preserved even if the app crashes
So that unexpected errors don't lose my racing data

Given the application crashes during recording
When I restart the application
Then partial session data is recovered
And I see a recovery prompt with session details
And recovered data is usable for analysis
```

---

### Category 6: Greenfield Setup Gap (1 Issue → 1 New Story)

**Covered above in Category 1:** Story 1.8 (Automated Build & Test Pipeline)

---

## Implementation Impact Summary

### Story Count Changes

| Epic | Before | After | Change | Notes |
|------|--------|-------|--------|-------|
| Epic 1 | 6 | 8 | +2 | Added: 1.7 (Historical Import), 1.8 (CI/CD) |
| Epic 2 | 5 | 7 | +2 | Added: 2.6 (Derived Metrics), 2.7 (Session Type), 2.3b (Manual Trigger) |
| Epic 3 | 6 | 7 | +1 | Added: 3.8 (Car Templates), 3.9 (Incident Detection); Moved: 3.7 → Epic 4 |
| Epic 4 | 5 | 6 | +1 | Added: 4.6 (Debrief Data Contract), 4.7 (moved from 3.7) |
| Epic 5 | 5 | 9 | +4 | Added: 5.6 (Audio), 5.7 (Auto-Update); Split: 5.5 → 5.5/5.8/5.9 |
| **Total** | **27** | **37** | **+10** | Net increase |

### Modification Summary

| Change Type | Count | Stories Affected |
|-------------|-------|-----------------|
| New Stories | 10 | 1.7, 1.8, 2.3b, 2.6, 2.7, 3.8, 3.9, 4.6, 5.6, 5.7 |
| AC Additions | 16 | 1.3, 1.4, 1.5, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3, 3.4, 4.1, 5.1 |
| Reframings | 4 | 1.1, 1.2, 3.2, 5.5 |
| Story Moves | 1 | 3.7 → 4.7 |
| Story Splits | 1 | 5.5 → 5.5/5.8/5.9 |
| Epic Independence Fixes | 6 | 2.1, 2.4, 2.5, 3.5, 3.6, 3.7 |

---

## Risk Assessment

### Risks Mitigated by These Changes

| Risk | Before | After |
|------|--------|-------|
| Incomplete product (missing FRs) | 🔴 High | ✅ Resolved |
| Incorrect implementation (misaligned FRs) | 🔴 High | ✅ Resolved |
| Contradictory requirements | 🔴 High | ✅ Resolved (decisions approved) |
| Untestable epics (forward dependencies) | 🔴 High | ✅ Resolved |
| Poor user-value framing | 🟡 Medium | ✅ Resolved |
| No CI automation | 🟠 High | ✅ Resolved |

### New Risks Introduced

| Risk | Severity | Mitigation | Status |
|------|----------|------------|--------|
| Increased story count (+10) | 🟡 Medium | Stories are small, focused; net effort aligns with PRD scope | Active |
| Product decisions required (2) | 🟠 High | Decisions are binary (approve/reject options); block resolved immediately upon decision | ✅ Resolved (2026-02-04) |
| Refactoring existing stories | 🟡 Medium | Changes are additive (AC additions) or clarifying (reframings); minimal disruption | Active |

---

## Recommendation

### Approval Status: ✅ **APPROVED** (2026-02-04)

**Rationale:**
1. **Implementation blocking issues resolved:** All 34 issues addressed with specific, actionable changes
2. **Minimal disruption:** Most changes are additive (new stories, new ACs) rather than destructive
3. **Product integrity restored:** PRD-Epic-UX alignment achieved
4. **Best practices enforced:** Epic independence, user value framing, CI/CD automation

### Approved Decisions (2)

**Decision 1: Manual Trigger Conflict** ✅
- ✅ **APPROVED:** Tray-menu manual trigger (satisfies FR1b)
- Implementation: Story 2.3b scoped to system tray menu integration

**Decision 2: WCAG Accessibility** ✅
- ✅ **APPROVED:** Option B (Basic WCAG AA: keyboard + contrast only)
- Implementation: Update PRD, UX Spec, and Architecture to align with Basic WCAG AA scope
- No additional stories required (embedded in existing UI stories)

### Next Steps (Post-Approval)

✅ **Proposal Approved:** Ready for implementation

**Required Actions:**

1. **Update Epics & Stories Document** (`epics-and-stories.md`)
   - Add 10 new stories (8 Missing FRs + 2 split from Story 5.5)
   - Add acceptance criteria to 16 existing stories
   - Reframe 4 technical stories with user value
   - Move Story 3.7 to Story 4.7
   - Apply Epic independence fixes (event-based decoupling)

2. **Update PRD** (`prd.md`)
   - Update Non-Functional Requirements: "Basic WCAG 2.1 AA: keyboard navigation + contrast compliance"
   - Confirm FR1b satisfied via tray-menu trigger approach

3. **Update UX Spec** (`ux-design-specification.md`)
   - Revise manual trigger language: "No prominent UI triggers; power-user tray option available"
   - Update accessibility scope: "Screen reader support deferred to post-MVP"

4. **Update Architecture** (`architecture.md`)
   - Align WCAG scope with Basic WCAG AA (keyboard + contrast)
   - Document event-based decoupling patterns (Epic 2→5, Epic 3→4)

5. **Re-run Implementation Readiness Assessment**
   - Validate all 34 issues resolved
   - Confirm 100% FR coverage
   - Verify Epic independence restored

6. **Proceed to Sprint Planning**
   - Use corrected epic/story set (37 stories)
   - Prioritize Story 1.8 (CI/CD) early in Sprint 1

---

## Appendix: Change Checklist

### Category 1: Missing FRs (10 → 8 Stories)
- [ ] Add Story 1.7: Historical Session Import (FR10)
- [ ] Add Story 1.8: Automated Build & Test Pipeline (CI/CD)
- [ ] Add Story 2.3b: Manual Debrief Trigger (FR1b)
- [ ] Add Story 2.6: Derived Metrics Engine (FR9)
- [ ] Add Story 2.7: Session Type Capture (FR40)
- [ ] Add Story 3.8: Car Template System (FR13)
- [ ] Add Story 3.9: Anomaly & Incident Detection (FR16)
- [ ] Add Story 4.6: Debrief Data Contract (FR30)
- [ ] Add Story 5.6: Audio Notification System (FR35 + FR41)
- [ ] Add Story 5.7: Application Auto-Update (FR38)

### Category 2: Misaligned FRs (12 → 16 Modifications)
- [ ] Story 1.3: Add AC for FR6 (local storage separation)
- [ ] Story 1.4: Add AC for FR31 (debrief persistence)
- [ ] Story 1.5: Add AC for FR33 (consistency + technique metrics)
- [ ] Story 2.2: Add AC for FR2 (configurability + environmental context)
- [ ] Story 2.3: Add AC for FR3 (lap boundary detection)
- [ ] Story 2.3: Add AC for FR4 (incomplete-lap retention)
- [ ] Story 2.3: Add AC for FR5 (gap markers)
- [ ] Story 2.4: Add AC for FR1a (session end → debrief trigger)
- [ ] Story 3.1: Add AC for FR21 (pre-session validation)
- [ ] Story 3.2: Add AC for FR22 (provider dispatch explicit)
- [ ] Story 3.3: Add AC for FR1a (debrief trigger link)
- [ ] Story 3.4: Add AC for FR4 (incomplete-lap analysis)
- [ ] Story 4.1: Add AC for FR27 (per-lap structured stats)
- [ ] Story 5.1: Add AC for FR24 (provider status visibility)

### Category 3: PRD↔UX Conflicts (2 Decisions)
- [x] Decision 1: Manual trigger (approve tray-menu OR revise PRD) - **APPROVED: Tray-menu trigger**
- [x] Decision 2: WCAG accessibility (Option A / B / C) - **APPROVED: Option B (Basic WCAG AA)**

### Category 4: Epic Independence (6 Story Fixes)
- [ ] Story 2.1: Replace tray UI with connection_status_changed event
- [ ] Story 2.4: Replace tray UI with session_state_changed event
- [ ] Story 2.5: Replace notification UI with debrief_ready event
- [ ] Story 3.5: Replace Summary tab with coaching_chunk SSE events
- [ ] Story 3.6: Replace Telemetry tab with comparison_data contract
- [ ] Story 3.7: Move to Story 4.7 (Epic 3 keeps API only)

### Category 5: Technical Stories (4 Reframings + 2 Splits)
- [ ] Story 1.1: Reframe as "Desktop Application Runs Locally"
- [ ] Story 1.2: Reframe as "Session History Persists Locally"
- [ ] Story 3.2: Reframe as "AI Coaching Uses My Preferred Provider"
- [ ] Story 5.5: Reframe as "Application State Syncs Seamlessly"
- [ ] Add Story 5.8: Safe Quit Without Data Loss (split from 5.5)
- [ ] Add Story 5.9: Crash Recovery Preserves Sessions (split from 5.5)

### Category 6: Greenfield Gap
- [ ] Covered by Story 1.8 (CI/CD)

---

**Total Checklist Items:** 40 changes
**Product Owner Decisions Required:** 2 ✅ **BOTH APPROVED**

---

**Prepared By:** Course Correction Workflow (BMAD Method)
**Review Status:** ✅ **APPROVED**
**Approval Signature:** Joe (Product Owner)
**Date Approved:** 2026-02-04
