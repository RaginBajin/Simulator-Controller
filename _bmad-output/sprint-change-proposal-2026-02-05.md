# Sprint Change Proposal
**Date:** 2026-02-05
**Project:** Simulator-Controller
**Trigger:** Implementation Readiness Assessment (2026-02-06)
**Mode:** Batch Analysis
**Status:** Awaiting User Approval

---

## Executive Summary

Implementation readiness assessment identified THREE critical blockers preventing sprint start. After systematic analysis through the correct-course workflow, **all issues are addressable without MVP scope reduction**. Two blockers are trivial documentation fixes; one requires epic restructuring. **Recommendation: Direct Adjustment approach with 1-2 day planning update.**

**Blockers:**
1. ✅ FR6 Documentation Inconsistency (LOW severity - doc bug only)
2. ⚠️  Epic Independence Violations (HIGH severity - architectural)
3. ✅ UX MVP Scope Creep (MEDIUM severity - UX doc alignment)

**Impact to MVP:** NONE - All planned features remain in MVP scope
**Timeline Impact:** 1-2 days for epic restructuring

---

## Issue #1: FR6 Documentation Inconsistency

### Problem Statement

FR6 (local telemetry + metadata storage) appears to be missing from epic coverage based on readiness report, but investigation reveals it IS implemented in Story 1.4. The issue is a documentation bug affecting Epic 1's functional requirement mapping.

**Evidence:**
- FR Coverage Map (epics:287) claims: Epic 1 covers FR6, FR7, FR10, FR31, FR33 (5 FRs)
- Epic 1 header (epics:329) claims: Epic 1 covers FR7, FR10, FR31, FR32, FR33 (5 FRs)
- **ACTUAL coverage per stories:** Epic 1 covers FR6, FR7, FR10, FR31, FR32, FR33 (6 FRs)
- Story 1.4 "Parquet Telemetry Storage" (epics:415-440) explicitly implements FR6:
  - Lines 436-439: "all data is stored locally on user's machine (no cloud dependency) AND time-series telemetry uses Parquet format for analytical queries AND metadata uses SQLite for structured queries"

**Category:** Misunderstanding of original requirements / Documentation inconsistency
**Discovery:** Implementation readiness assessment (2026-02-06) by Codex (PM/SM)

### Epic Impact Assessment

**Current Epic:** Epic 1 (Persistent Data Storage & Session History)
- Can epic be completed as planned? ✅ YES
- Required modifications: Documentation fixes only (header + coverage map)

**Other Epics:** No impact on Epic 2, 3, 4, or 5

**Epic Sequence:** No changes required

### Artifact Conflicts

**PRD:** No conflict - FR6 defined correctly (prd:68)
**Architecture:** No conflict - storage architecture matches FR6
**UX:** No impact

### Recommended Changes

1. **Update Epic 1 header** (epics-and-stories.md:329):
   - Current: `Mapped FRs: FR7, FR10, FR31, FR32, FR33 (5 FRs)`
   - Corrected: `Mapped FRs: FR6, FR7, FR10, FR31, FR32, FR33 (6 FRs)`

2. **Update FR Coverage Map** (epics-and-stories.md:287):
   - Current: `Epic 1: Data Storage | 5 | FR6, FR7, FR10, FR31, FR33 | 8 stories`
   - Corrected: `Epic 1: Data Storage | 6 | FR6, FR7, FR10, FR31, FR32, FR33 | 8 stories`

3. **Update TOTAL FR count** (epics-and-stories.md:292):
   - Current: `TOTAL | 43 | All FRs (FR1-FR41) mapped | 37 stories`
   - Corrected: Verify total still 43 after fixing Epic 1 count

**Effort:** LOW (2 line edits)
**Risk:** LOW (documentation only)
**Timeline:** < 1 hour

---

## Issue #2: Epic Independence Violations

### Problem Statement

Epic execution sequence violates the independence principle where Epic N should not depend on Epic N+1. Current plan has Epic 4 requiring window shell from Epic 5, and Epic 2 requiring system tray from Epic 5. This creates forward dependencies that block independent epic completion and force rigid execution ordering.

**Evidence:**
- Epic 4 Story 4.1 (epics:1009): "**Given** I open a completed session debrief (FR25) **When** the window appears" — Window is defined in Epic 5 Story 5.1 (epics:1177)
- Epic 2 Story 2.5 (epics:708): "**When** the user minimizes AI Race Team to the system tray" — System tray is Epic 5 feature
- Readiness report (lines 243-246): "Epic 4 (Interactive Debrief Visualization) relies on window/tab navigation provided in Epic 5 (Story 5.1). Epic 4 cannot function without Epic 5, which breaks the rule that Epic N cannot depend on Epic N+1."

**Note:** Epics file claims "Epic independence restored via event-based decoupling" (epics:297) but dependencies STILL EXIST in actual story acceptance criteria.

**Category:** Technical limitation discovered during implementation / Architectural issue
**Discovery:** Implementation readiness assessment - Epic Quality Review section

### Epic Impact Assessment

**Affected Epics:**
- Epic 2: Story 2.5 depends on system tray (Epic 5)
- Epic 4: ALL stories depend on window shell from Epic 5 Story 5.1
- Epic 5: Becomes prerequisite for Epic 2 and Epic 4

**Cannot be completed as planned:**
- Epic 2: Story 2.5 references system tray minimization before it exists
- Epic 4: Cannot function without window infrastructure

**Epic Sequence Impact:** HIGH - Epic 5 (or parts of it) must execute BEFORE Epic 2 and Epic 4

### Artifact Conflicts

**PRD:** No conflict - PRD doesn't specify epic execution order
**Architecture:** May need event-based decoupling pattern documentation
**UX:** Minimal impact - UX assumes integrated desktop app but doesn't mandate sequence

### Path Forward Options

**Option A: Re-sequence Epics (Simplest)**
- Move Epic 5 to position 1 or 2 in execution order
- Epic 5 Stories 5.1-5.3 (window shell, system tray, notifications) execute FIRST
- Then Epic 2 and Epic 4 can use these foundations
- Pros: Simplest implementation, clear dependencies
- Cons: Epic 5 contains non-foundation stories (settings, updates) that shouldn't be first

**Option B: Split Epic 5 into Foundation + Integration (Recommended)**
- Create "Epic 1.5: Desktop Foundation" or "Epic 0: Application Shell"
- Move Epic 5 Stories 5.1 (window/tabs), 5.2 (system tray), 5.3 (notifications) to foundation epic
- Execute foundation epic AFTER Epic 1, BEFORE Epic 2
- Remaining Epic 5 stories (settings, updates, auto-launch) stay in position 5
- **New Sequence:** Epic 1 → Epic 1.5 (Foundation) → Epic 2 → Epic 3 → Epic 4 → Epic 5 (Integration)
- Pros: Preserves epic independence principle, clear separation of concerns
- Cons: Adds one more epic to track (but only 3 stories)

**Option C: Event-Based Decoupling (Most Complex)**
- Epic 2 and Epic 4 emit events (e.g., `debrief_ready`, `open_visualization`)
- Epic 5 listens to events and provides UI response
- Epic 2 and Epic 4 don't reference UI directly in acceptance criteria
- Pros: True epic independence, flexible implementation order
- Cons: Requires rewriting multiple story acceptance criteria, more complex architecture

**RECOMMENDATION: Option B - Split Epic 5**

**Rationale:**
1. Maintains epic independence principle correctly
2. Minimal disruption to existing stories (just moves 3 stories to new epic)
3. Clear execution sequence: Foundation → Capture → AI → Visualization → Integration
4. Preserves NFR10 graceful degradation (Epic 2 can work without UI, Epic 4 requires UI)
5. Lower risk than Option C's event architecture refactoring

### Recommended Changes

1. **Create new Epic 1.5: Desktop Application Foundation**
   - **Epic Goal:** Deliver the foundational desktop window and system integration required by all UI epics
   - **User Value:** "The app runs as a native desktop application I can minimize and restore"
   - **Technical Scope:** Tauri window shell, system tray, notifications
   - **Mapped FRs:** FR34 (system tray), FR35 (notifications), FR36 (debrief access from tray)
   - **Stories:** Move from Epic 5:
     - Story 5.1: Tauri Window & Tab Navigation (epics:1177)
     - Story 5.2: System Tray Integration (find in Epic 5)
     - Story 5.3: Desktop Notifications (find in Epic 5)

2. **Update Epic 5 to "Desktop Integration & Settings"**
   - Remove Stories 5.1, 5.2, 5.3 (moved to Epic 1.5)
   - Remaining stories: Settings management, auto-launch, app updates, storage config
   - **Mapped FRs:** FR37 (auto-launch), FR38 (updates), FR39 (storage config), FR41 (audio prefs)

3. **Update Epic execution sequence in epics-and-stories.md**
   - Line 303-315: Rewrite Epic List with new sequence
   - Insert Epic 1.5 between Epic 1 and Epic 2
   - Update epic numbers if needed (or use 1.5 numbering)

4. **Update FR Coverage Map** (epics:287)
   - Add row for Epic 1.5 with FR34, FR35, FR36 (3 FRs, 3 stories)
   - Update Epic 5 row to remove FR34, FR35, FR36 (remaining: FR37, FR38, FR39, FR41 = 4 FRs)

5. **Verify Epic 2 and Epic 4 dependencies are satisfied**
   - Epic 2 Story 2.5 (line 708): System tray now available from Epic 1.5
   - Epic 4 Story 4.1 (line 1009): Window shell now available from Epic 1.5

**Effort:** MEDIUM-HIGH (Epic restructuring, story moves, documentation updates)
**Risk:** MEDIUM (Careful mapping of stories to new epic, FR reallocation)
**Timeline:** 1-2 days for restructuring and validation

---

## Issue #3: UX MVP Scope Creep

### Problem Statement

UX specification includes Growth-phase features (confidence indicators, advanced incident classification with user override, pit stop annotations) in MVP journey flows. The PRD explicitly places these in Growth phase, but UX treats them as MVP features. This creates scope creep risk and could delay MVP delivery if teams build Growth features prematurely.

**Evidence:**
- Readiness report (line 231): "UX includes AI confidence indicators and dismissal logging in core Journey 1 flow, but PRD defines confidence indicators as Growth phase"
- Readiness report (line 232): "UX flow for bad sessions includes incident classification with user override and pit stop annotations; PRD places incident detection and stint-level analysis in Growth"
- PRD line 199: "AI confidence indicators | Growth"
- PRD line 221: "At launch, confidence indicators are heuristic... True calibration... is a Growth-phase capability"
- PRD lines 201-204: Stint-level analysis, tire degradation, pit windows all marked Growth

**IMPORTANT:** Epics-and-stories.md is CORRECT:
- Story 3.6 (epics:906): "Confidence Scoring UI Indicators [DEFERRED TO GROWTH]" with explicit rationale
- Story 3.9 (epics:971): Basic incident detection (FR16 MVP scope only)

**Issue is UX document only, NOT in epic planning.**

**Category:** Misunderstanding of original requirements / Phase boundary confusion
**Discovery:** Implementation readiness assessment - UX Alignment section

### Epic Impact Assessment

**Affected Epics:**
- Epic 3: CORRECT - Story 3.6 properly deferred
- Epic 4: Need to verify no Growth UI elements in visualization stories

**Epics can be completed as planned:** ✅ YES - No epic changes required

**Epic Sequence:** No changes required

### Artifact Conflicts

**PRD:** Internal contradiction resolved:
- FR16 (basic incident diagnosis): MVP ✅
- Advanced incident analysis (stint-level, pit strategy, user override): Growth ✅
- Confidence indicators: Growth ✅

**Architecture:** No impact - Growth features not yet architected
**UX:** PRIMARY ISSUE - Growth features appear in MVP journeys

### Recommended Changes

1. **Review UX Design Specification for Growth features in MVP journeys**
   - Journey 1 (Happy Path): Identify confidence indicator UI elements
   - Journey 3 (Bad Session): Identify advanced incident classification, pit stop annotations

2. **Mark Growth features with [DEFERRED TO GROWTH] annotations in UX**
   - Add annotation blocks similar to epics Story 3.6 pattern
   - Example format:
     ```markdown
     ### AI Confidence Indicators [DEFERRED TO GROWTH]
     **Rationale:** Confidence indicators are Growth phase per PRD:116, PRD:221
     ```

3. **Create MVP-only UX flow variants**
   - Journey 1 MVP: Remove confidence indicator UI, keep core coaching
   - Journey 3 MVP: Basic incident type only (spin/crash/disconnect), no user override, no pit strategy

4. **Document Growth feature placeholders**
   - Note WHERE Growth features will appear in UI
   - Specify Growth features don't block MVP delivery
   - Ensure UI architecture accommodates future Growth additions

**Effort:** LOW (UX documentation annotations)
**Risk:** LOW (Documentation only, no implementation impact)
**Timeline:** < 1 day

---

## Consolidated Action Plan

### Phase 1: Documentation Fixes (< 1 day)

1. ✅ **Fix FR6 mapping** (Issue #1)
   - Update epics-and-stories.md lines 287, 329
   - Verify total FR count

2. ✅ **Mark UX Growth features** (Issue #3)
   - Review ux-design-specification.md for Growth elements
   - Add [DEFERRED TO GROWTH] annotations
   - Create MVP-only UX flow variants

### Phase 2: Epic Restructuring (1-2 days)

3. ⚠️  **Split Epic 5 and create Epic 1.5** (Issue #2)
   - Extract Stories 5.1, 5.2, 5.3 from Epic 5
   - Create Epic 1.5: Desktop Application Foundation
   - Update Epic 5 to "Desktop Integration & Settings"
   - Rewrite Epic List with new sequence
   - Update FR Coverage Map with new epic row
   - Verify all epic dependencies satisfied

### Phase 3: Validation (< 1 day)

4. ✅ **Run implementation readiness check again**
   - Verify FR6 now shows as covered
   - Verify Epic 1.5 → Epic 2/4 dependency chain is valid
   - Verify UX aligned with PRD phase boundaries
   - Confirm project status: READY

---

## PRD MVP Impact

**MVP Scope:** ✅ NO CHANGES
**Core Goals:** ✅ PRESERVED
**Features Deferred:** NONE (Growth features were already correctly scoped in epics)
**Timeline:** +1-2 days for epic restructuring
**Risk:** LOW - All changes are planning/documentation level

**MVP Remains Achievable:** YES - All 43 FRs, all 5 core epics (now 6 with Epic 1.5), all user value delivered

---

## Agent Handoff Plan

### Responsible Agents/Roles

**Phase 1 (Documentation Fixes):**
- **Product Manager / Scrum Master (Codex):** Execute FR mapping fix, UX annotations
- **Estimated Time:** 4 hours
- **Deliverables:**
  - Updated epics-and-stories.md with corrected FR mappings
  - Updated ux-design-specification.md with Growth annotations

**Phase 2 (Epic Restructuring):**
- **Product Manager / Architect:** Design Epic 1.5 structure, FR reallocation
- **Scrum Master:** Move stories, update epic metadata, verify dependencies
- **Estimated Time:** 8-12 hours
- **Deliverables:**
  - New Epic 1.5 section in epics-and-stories.md
  - Updated Epic 5 section with reduced scope
  - Updated Epic List and FR Coverage Map
  - Verification checklist showing no forward dependencies

**Phase 3 (Validation):**
- **Product Manager:** Re-run implementation readiness assessment
- **Team:** Final review and sign-off
- **Estimated Time:** 2-4 hours
- **Deliverables:**
  - Updated implementation readiness report showing READY status

### Success Criteria

- [ ] FR6 correctly mapped in both coverage map and Epic 1 header
- [ ] Epic 1.5 created with 3 stories (window, tray, notifications)
- [ ] Epic 5 reduced to 6 stories (settings, updates, auto-launch, storage config, audio prefs)
- [ ] Epic sequence updated: 1 → 1.5 → 2 → 3 → 4 → 5
- [ ] UX Growth features annotated with [DEFERRED TO GROWTH]
- [ ] No epic N depends on epic N+1
- [ ] Implementation readiness status: READY

---

## Approval Required

**User Decision Point:** Approve this Sprint Change Proposal?

**If APPROVED:**
- Proceed with Phase 1 documentation fixes immediately
- Begin Phase 2 epic restructuring
- Target: Implementation readiness READY status within 2 days

**If CHANGES REQUESTED:**
- Specify which approach to modify (Issue #1, #2, or #3)
- Provide alternative direction
- Revise proposal accordingly

**Questions to Consider:**
1. Do you prefer Option B (Split Epic 5) or want to explore Option A (Re-sequence) or Option C (Event-based)?
2. Should Epic 1.5 be numbered differently (e.g., Epic 0, or renumber all epics)?
3. Any concerns about the 1-2 day timeline for epic restructuring?

---

**END OF SPRINT CHANGE PROPOSAL**

*Generated by: Correct-Course Workflow (Batch Mode)*
*Date: 2026-02-05*
*Status: Awaiting User Approval*
