---
stepsCompleted: ['step-01-init', 'step-02-discovery', 'step-03-success', 'step-04-journeys', 'step-05-domain', 'step-06-innovation', 'step-07-project-type', 'step-08-scoping', 'step-09-functional', 'step-10-nonfunctional', 'step-11-polish']
inputDocuments:
  - README.md
  - Docs/project-documentation/INDEX.md
  - Docs/project-documentation/ARCHITECTURE.md
  - Docs/project-documentation/TECHNOLOGY_STACK.md
  - Docs/REFACTORING_PLAN.md
  - Docs/ARCHITECTURE.md
  - Docs/AI_ASSISTANTS.md
  - Docs/Backlog.md
workflowType: 'prd'
documentCounts:
  briefs: 0
  research: 0
  brainstorming: 0
  projectDocs: 8
classification:
  projectType: Desktop Application (Windows-native)
  domain: Gaming/Entertainment Tools (Sim Racing)
  complexity: CRITICAL
  context: Brownfield Modernization
  migrationStrategy: INCREMENTAL
  uiStrategy: CONDITIONAL (pending profile experiment)
  urgency: MODERATE (quality over speed)
  valueFocus: OUTCOME-DRIVEN
  projectRelationship: Open Source Contribution Proposal
elicitationMethodsUsed:
  - User Persona Focus Group
  - Architecture Decision Records
  - SCAMPER
  - Shark Tank Pitch
  - Feynman Technique
  - Debate Club Showdown
  - Mentor and Apprentice
  - Self-Consistency Validation
  - Party Mode (Multi-Agent Discussion)
  - Cross-Functional War Room
  - Comparative Analysis Matrix
  - Expert Panel Review
  - Genre Mashup
  - Time Traveler Council
  - Reverse Engineering
---

# Product Requirements Document - Simulator-Controller

**Author:** Joe (Contributor/Proposer)
**Original Developer:** SeriousOldMan (Oliver Juwig)
**Repository:** https://github.com/SeriousOldMan/Simulator-Controller
**Date:** 2026-01-26
**Document Type:** Open Source Contribution Proposal + Modernization Roadmap

---

## Executive Summary

Simulator Controller is a 250K-line Windows desktop application providing AI race engineers, strategists, and coaches for sim racing. Despite powerful AI capabilities (RETE rule engine, LLM coaching, voice-first interaction across 11 simulators), an estimated 60% of new users abandon setup before experiencing the core value.

**Problem:** The onboarding experience is dated, complex, and undocumented. Users face 47+ settings, no guided setup, and no path from "installed" to "racing with AI."

**Proposed Solution:** Incremental modernization through open source contribution, starting with configuration profiles and a Quick Start wizard in the existing UI — validated by data before investing in architecture changes.

**Key Differentiator:** This is NOT a rewrite proposal. It preserves working systems (C++ telemetry connectors, RETE engine, voice pipeline) while reducing the barrier to entry. The AI assistants are the competitive moat; the goal is to let more people experience them.

**Approach:** Hybrid MVP — deliver user-visible improvements (profiles, Quick Start) alongside contributor trust-building (tests, documentation) through a Test-Along methodology. Every phase is data-conditional: measure impact before committing to the next investment.

**Discovery:** 15 elicitation methods across 3 sessions produced 42 functional requirements, 31 non-functional requirements, 7 user journeys, and 4 paradigm innovations.

---

## Table of Contents

1. [Project Classification](#1-project-classification)
2. [Discovery Insights](#2-discovery-insights)
3. [Priority Stack](#3-priority-stack)
4. [Author's Context](#4-authors-context)
5. [Success Criteria](#5-success-criteria)
6. [Product Scope](#6-product-scope)
7. [User Journeys](#7-user-journeys)
8. [Journey Requirements Summary](#8-journey-requirements-summary)
9. [Innovation & Novel Patterns](#9-innovation--novel-patterns)
10. [Desktop Application Specific Requirements](#10-desktop-application-specific-requirements)
11. [Project Scoping & Phased Development](#11-project-scoping--phased-development)
12. [Functional Requirements](#12-functional-requirements)
13. [Non-Functional Requirements](#13-non-functional-requirements)

---

## 1. Project Classification

### 1.1 Core Classification

| Attribute | Value | Rationale |
|-----------|-------|-----------|
| **Project Type** | Desktop Application | Windows-native sim racing companion |
| **Domain** | Gaming/Entertainment Tools | Sim racing ecosystem software |
| **Complexity** | CRITICAL | 250K LOC, 4 languages, 11 simulators, 4 AI assistants |
| **Context** | Brownfield Modernization | Existing mature codebase requiring evolution |
| **Platform** | Windows-only | Sim racing ecosystem is Windows-centric |

### 1.2 Strategic Classification

| Attribute | Value | Rationale |
|-----------|-------|-----------|
| **Migration Strategy** | INCREMENTAL | Preserve working systems, evolve carefully |
| **UI Strategy** | CONDITIONAL | Depends on profile experiment results |
| **Urgency** | MODERATE | No competitive pressure; quality over speed |
| **Value Focus** | OUTCOME-DRIVEN | "Reduce abandonment, amplify AI" |
| **Project Relationship** | Contribution Proposal | Fork → PR model with original maintainer |

### 1.3 Technical Snapshot

```
AutoHotkey   ████████████████████████████████  86% (~215K LOC)
C#           █████░░░░░░░░░░░░░░░░░░░░░░░░░░░  10% (~25K LOC)
C++          ███░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   4% (~10K LOC)
```

- **11 Simulator Integrations:** ACC, AC, ACE, iRacing, RF2, LMU, AMS2, R3E, PCARS2, PMR, RSP
- **4 AI Assistants:** Jona (Engineer), Cato (Strategist), Elisa (Spotter), Aiden (Coach)
- **8 Languages:** EN, DE, ES, FR, IT, PT, JA, ZH
- **Voice System:** Microsoft Speech, Google Cloud, Whisper
- **Team Server:** ASP.NET Core REST API

---

## 2. Discovery Insights

### 2.1 User Persona Findings

Five representative personas analyzed:

| Persona | Type | Key Need | Pain Point |
|---------|------|----------|------------|
| **Marcus** | Casual (60%) | Quick setup, basic features | Overwhelmed by complexity |
| **Elena** | Competitive (25%) | Precise data, reliability | Setup time before league races |
| **Takeshi** | Endurance (10%) | Team features, AI strategies | Multi-stint reliability |
| **Derek** | Hardware Enthusiast (3%) | Button box, motion rig | Integration complexity |
| **Sofia** | Content Creator (2%) | OBS integration, overlays | Stream-friendly UI |

**Core Insight:** The majority (60%) are casual users who need simpler onboarding, but the value proposition lies in features used by serious racers. Progressive disclosure is essential.

### 2.2 Architecture Decision Records

| Decision | Chosen Approach | Rejected Alternative | Rationale |
|----------|-----------------|---------------------|-----------|
| **Migration** | Incremental (WebView2 → Tauri) | Big-bang Electron rewrite | Lower risk, continuous value |
| **Telemetry** | Keep C++ connectors | Rewrite in C# | Working code, not user pain |
| **API Strategy** | REST + OpenAPI + WebSocket | gRPC everywhere | Tooling, debugging, adoption |
| **First Experiment** | Profiles in existing UI | New framework first | Test hypothesis before investment |

**Architecture Philosophy:** "Optimize for the journey, not just the destination."

### 2.3 Innovation Opportunities (SCAMPER)

| Innovation | Type | Impact | Effort |
|------------|------|--------|--------|
| **Configuration Profiles** | Combine | HIGH | LOW |
| **Zero-Config Launch** | Eliminate | HIGH | MEDIUM |
| **Proactive AI** | Adapt | MEDIUM | LOW |
| **Post-Session Reports** | Put to other use | MEDIUM | LOW |
| **Progressive Disclosure** | Rearrange | HIGH | MEDIUM |

**Key Insight:** Many quick wins don't require architecture changes.

### 2.4 Investment Validation (Shark Tank)

**Strategic Reframe:** "Reduce abandonment and amplify AI" — not "modernize technology"

**Measurement Requirements:**
- Instrument BEFORE major investment
- Validate profile hypothesis before UI rewrite
- Track: Setup completion, feature adoption, session frequency

**ROI Insight:** Configuration profiles might work in EXISTING UI — test before rebuilding.

### 2.5 Simplification Analysis (Feynman)

**Validated Concepts:**
- Why modernize: "App is too hard to use"
- Incremental approach: "Fix one room at a time"
- AI value: "Having a crew chief who learns"

**Gaps Identified:**
- Measurement specification needed
- Profile design specification needed
- Proactive AI specification needed
- Prioritization framework needed

### 2.6 Strategic Debate (Debate Club)

**Motion:** "UI modernization should begin immediately"

| Position | Advocate | Key Argument |
|----------|----------|--------------|
| **Modernist** | Nova | Technical debt compounds; longer delay = harder migration |
| **Pragmatist** | Max | No data proves UI is the problem; profiles first |

**Verdict:** Max wins 4-2. *"Profiles are the experiment. UI modernization is conditional on data."*

**Synthesis:** Ship profiles → Measure impact → THEN decide on UI framework.

### 2.7 Hidden Assumptions (Mentor & Apprentice)

| Assumption | Reality Check | Action |
|------------|---------------|--------|
| All 47+ settings needed | Likely not — audit required | Settings usage audit |
| Users want 4 AI assistants | Adds complexity — progressive disclosure | Feature usage audit |
| 250K LOC all necessary | May contain cruft | Code audit (lower priority) |
| Urgency is high | No competitive pressure | Quality over speed |

### 2.8 Cross-Validation (Self-Consistency)

**Strongly Validated (4-5 lenses agree):**
- Onboarding is the critical bottleneck
- Incremental migration is correct strategy
- Profiles should precede UI modernization
- AI assistants are the competitive moat
- Audio experience > Visual UI
- Instrumentation must happen

**Gap Discovered:** Testing infrastructure not mentioned in prior analysis — critical for contribution model.

**Reframe:** "Simple Mode" → "Progressive Disclosure" (preserves moat while reducing complexity)

### 2.9 Contribution Model (Party Mode)

**Project Relationship Reframe:**
- This is NOT an internal roadmap
- This IS a contribution proposal + pitch document
- Author (Joe) is a user who experienced onboarding pain firsthand
- Goal: Partner with original maintainer (SeriousOldMan) on modernization

**Trust-Building Strategy:**
1. Start with tests and documentation (low-risk, high-value)
2. Small, focused PRs that demonstrate competence
3. Prove reliability before proposing architecture changes
4. Frame criticism as "offering help," not "pointing out problems"

**Test-Along Approach:**
- Add tests WITH every change, not as upfront infrastructure
- Every PR includes tests for touched code
- Builds test coverage incrementally while delivering value

---

## 3. Priority Stack

Based on all elicitation methods, the validated priority order:

| Priority | Initiative | Rationale |
|----------|------------|-----------|
| **1** | Testing Infrastructure (Test-Along) | Foundation for safe changes + builds contributor trust |
| **2** | Settings Usage Audit | Data before decisions |
| **3** | Configuration Profiles | Test onboarding hypothesis in existing UI |
| **4** | Instrumentation & Measurement | Validate impact before investment |
| **5** | Security Fundamentals | Password hashing, token management |
| **6** | Reliability & Stability | Zero crashes, graceful degradation |
| **7** | Progressive Disclosure (AI) | Start with Jona, unlock others |
| **8** | AI Amplification | Proactive insights, post-session reports |
| **9** | Feature Usage Audit | Identify what to deprecate |
| **10** | Modern UX | CONDITIONAL on profile experiment data |

---

## 4. Author's Context

**Important Background:** The author of this PRD (Joe) is NOT the original developer of Simulator Controller. He is a user who:

- Struggled significantly with initial setup and configuration
- Found the UI dated and unhelpful for understanding the system
- Experienced the onboarding pain that this document aims to address
- Decided to contribute back to the project by proposing modernization

This firsthand experience as a frustrated user provides valuable primary research for understanding the problems this modernization aims to solve.

**Collaboration Goal:** Work with the original developer (SeriousOldMan/Oliver Juwig) through the open source contribution model — fork, improve, submit PRs — to evolve this valuable software together.

---

## 5. Success Criteria

### 5.1 User Success

**Onboarding Success (Primary)**
- New user can complete basic setup in **< 15 minutes** (vs current 60+ min estimated)
- Configuration profiles enable "1-click to racing" for common setups
- First AI interaction (Jona) occurs within **first session**
- Setup completion rate: Target **> 80%** (current estimated < 40%)

**Daily Use Success**
- Zero-config launch: Simulator auto-detected, assistant auto-connects
- Proactive AI insights delivered before user asks
- Post-session report received within 2 minutes of session end
- Crash/hang rate: **< 0.1%** of sessions

**Emotional Success Moments**
- "Aha!" moment: First time Jona gives pit strategy advice that works
- Delight: Seeing improvement trends in Aiden's coaching reports
- Relief: Configuration saved as profile, never redo setup
- Empowerment: Understanding when to unlock additional AI assistants

### 5.2 Business Success (Community/Contribution Model)

**3-Month Success**
- Contribution proposal accepted by SeriousOldMan
- First PR merged (likely tests/docs)
- Test coverage increased by **+15%** in touched areas
- At least 3 contributor-friendly improvements merged

**12-Month Success**
- Configuration profiles feature shipped and adopted
- Setup completion rate measurably improved (instrumentation working)
- 2+ additional contributors attracted by improved DX
- Roadmap for UI modernization informed by real data

**Community Success**
- Discord engagement: Reduced "setup help" questions by **50%**
- GitHub: Contributor-friendly labels, clear contribution guide
- Patreon: Modest growth from improved user retention

### 5.3 Technical Success

**Quality Metrics**
- Test coverage: **> 60%** for modified code (Test-Along approach)
- All tests passing before any PR merge
- No regressions in existing functionality
- C++ connectors remain untouched (working code)

**Architecture Metrics**
- WebView2 components (if introduced) isolated and reversible
- REST API documented with OpenAPI spec
- Instrumentation capturing setup funnel and feature adoption

**Maintainability Metrics**
- New contributors can build project in **< 30 minutes**
- Clear separation between UI and business logic where modernized
- Comprehensive docs for each modified subsystem

### 5.4 Measurable Outcomes

| Metric | Baseline (Est.) | Target | Timeline |
|--------|-----------------|--------|----------|
| Setup completion rate | ~40% | >80% | 6 months |
| Time to first race | 60+ min | <15 min | 6 months |
| Test coverage (touched code) | <10% | >60% | Ongoing |
| Crash rate per session | Unknown | <0.1% | 6 months |
| Discord setup questions | ~15/week | <8/week | 6 months |
| Active contributors | 1 | 3+ | 12 months |

---

## 6. Product Scope

> *Note: This section provides the high-level scope boundaries. See [Section 11](#11-project-scoping--phased-development) for detailed phased roadmap with dependencies, risk mitigation, and conditional triggers.*

### 6.1 MVP - Minimum Viable Product (0-3 months)

**Must-Have for Contribution Acceptance:**
1. ✅ Test infrastructure (Test-Along framework)
2. ✅ Settings usage audit (data collection)
3. ✅ Basic instrumentation (setup funnel)
4. ✅ Security fundamentals (password hashing)
5. ✅ Contribution documentation (CONTRIBUTING.md)

**Proves the Hypothesis:**
- Configuration profiles in existing AHK UI
- "Quick Start" mode with single profile selection
- Basic telemetry to measure adoption

### 6.2 Growth Features (Post-MVP, 3-6 months)

**Conditional on Profile Success:**
- Progressive disclosure for AI assistants (start with Jona)
- Proactive AI insights (speaks without being asked)
- Post-session reports (automatic value delivery)
- Zero-config auto-detection

**Requires Data Before Decision:**
- UI framework modernization (WebView2/Tauri)
- Settings audit results inform simplification

### 6.3 Vision (Future, 6-12+ months)

**Full Modernization (If Data Supports):**
- Modern UI shell (Tauri/WebView2)
- React-based configuration wizard
- Streaming voice with visual feedback
- Team Server dashboard overhaul

**Community Growth:**
- Plugin marketplace for community extensions
- Configuration profile sharing
- Sim-specific quick-start presets

---

## 7. User Journeys

### 7.1 Journey 1: Marcus — The Casual Racer's First Week

**Persona:** Marcus, 34, software developer. Races ACC after work to unwind. Has a basic Logitech G29 setup. Heard about Simulator Controller from a YouTube video promising "AI crew chief."

**Opening Scene:**
Marcus downloads the installer after a long Monday. He's excited but tired. He just wants to hear Jona tell him when to pit. The installer finishes... and he's staring at a window with 15 tabs, 47 settings, and terminology he doesn't understand. "Telemetry Provider?" "Assistant Booster?" What's a "Motion Feedback Controller?"

He clicks around randomly. Nothing seems connected to his simulator. There's no "just make it work" button. After 45 minutes of confusion — trying random settings, breaking his configuration, unable to undo — he still hasn't heard Jona say a word. His wife calls him for dinner. He quits the app and doesn't open it again for three weeks.

**Rising Action (With Modernization):**
Three weeks later, a friend tells Marcus about the new "Quick Start" feature. He re-launches. This time:
- "Select your simulator: ACC ✓" — one click
- "Select your wheel: Logitech G29 ✓" — auto-detected
- "Enable AI Race Engineer (Jona): ✓" — single checkbox
- Profile saved: "Marcus's ACC Setup"

Total time: 4 minutes.

**Climax:**
First race. Lap 3. His tires are going off. He's never sure when to pit. Then:
> *"Marcus, tire wear is at 72%. Based on your pace, I recommend pitting in two laps. Box, box."*

Marcus grins. This is what he wanted. He pits, emerges in P4, and finishes the race feeling like a real driver with a real crew.

**Resolution:**
Marcus races twice a week now. He's never touched the "advanced" settings. Jona just... works. Three months later, he notices there are other AI assistants. He's curious about Cato the Strategist. He unlocks it — and discovers a whole new layer of strategy he didn't know existed.

**Requirements Revealed:**
- Quick Start wizard with profile selection
- Simulator auto-detection
- Hardware auto-detection
- Progressive AI disclosure (start with Jona)
- Simple mode that hides complexity
- Profile persistence
- Confusion recovery ("Reset to defaults" + "Undo last changes")

---

### 7.2 Journey 2: Elena — League Night Pressure

**Persona:** Elena, 28, marketing manager. Competes in a Thursday night iRacing league. Every position matters for championship points. She's skilled — P3 in standings — but paranoid about software failures.

**Opening Scene:**
It's 7:45 PM Thursday. League race starts at 8:00. Elena's routine: launch iRacing, launch Simulator Controller, verify connection, test voice, check strategy. Tonight, something's wrong. The telemetry light is red. "Connection failed."

Her heart rate spikes. 12 minutes to fix this. She restarts the app. Still red. Restarts iRacing. Still red. She's sweating. Last week, someone in her league lost championship position because their pit strategy software crashed mid-race. She cannot let that happen.

**Rising Action (With Modernization):**
The new Simulator Controller has a diagnostic panel:
- **Status: iRacing** — ❌ Shared memory not found
- **Suggestion:** "iRacing may need to be restarted with admin privileges"

She follows the suggestion. Restarts iRacing as admin. Green checkmark appears.

> *"Elena, I've got you. Telemetry connection established. 340 cars ahead, P12 qualifying position confirmed. Let's go get them."* — Cato

7:58 PM. Two minutes to spare.

**Climax:**
Lap 28 of 45. She's P4, hunting P3 for championship points. The leader pits early. Cato speaks:
> *"Elena, leader pitted. Based on tire degradation models, I recommend extending 3 laps to undercut P3. Do you want me to calculate the gap?"*

She extends. The math works. She emerges in P3 with fresher tires. She hunts down P2 and finishes second.

**Resolution:**
Post-race, Elena opens her session report. She sees the tire model, pit strategy comparison, and the moment Cato's recommendation gained her the position. She screenshots it and posts in her league Discord: "This is why I use Simulator Controller."

**Post-Incident Recovery (Addition from Time Traveler Council):**
Two weeks later, Elena's iRacing crashes mid-race due to a VR glitch. She force-quits everything. When she relaunches Simulator Controller, instead of silence, she sees: "Session interrupted. 23 laps recorded. Strategy data preserved. Would you like to review?" Relief washes over her.

**Requirements Revealed:**
- Diagnostic panel with clear error states
- Actionable error recovery suggestions
- Pre-race connection verification
- Reliability above all (zero crashes)
- Post-session reports with decision replay
- Strategy confidence indicators
- Post-incident reassurance messaging

---

### 7.3 Journey 3: Joe — The Frustrated User Becomes Contributor

**Persona:** Joe, developer, sim racing enthusiast. Downloaded Simulator Controller hoping for AI assistance. Spent three frustrating evenings trying to get it working. Found bugs, confusing UI, and no documentation for contributors.

**Opening Scene:**
Joe stares at the Simulator Controller window. He's been at this for two hours. The Voice Recognition test says "Working" but no assistant responds when he speaks. The logs mention "grammar files" but he doesn't know where they are. The Discord has 50 people asking the same "how do I make the AI talk?" question.

He thinks: *"This software could be incredible. But it's so hard to use that people give up before experiencing the magic."*

He was angry at this software, but he could see the vision. The AI crew chief concept was brilliant. He wanted to SAVE it, not just fix it. He opens GitHub. He clicks "Fork."

**Rising Action:**
Joe starts exploring the codebase. 250,000 lines across three languages. No tests he can find. He wants to improve the first-time setup wizard, but:
- No CONTRIBUTING.md
- No architecture documentation
- No test suite to validate changes
- AutoHotkey v2 — a language he's never used

He decides to start small: add tests for one component, document what he learns.

**Climax:**
After two weeks, Joe opens his first PR: "Add unit tests for Configuration Profile Manager + documentation."

The maintainer, Oliver (SeriousOldMan), responds within a day:
> *"Thank you for this contribution! The tests look good. One question about the profile validation logic..."*

A conversation begins. Trust builds.

**Resolution:**
Six months later, Joe has contributed:
- Test infrastructure (Test-Along framework)
- Configuration profiles feature
- Setup instrumentation
- 15+ merged PRs

The Discord now has a "Quick Start" pinned post linking to Joe's documentation. New users complete setup in under 15 minutes. Joe's frustration became the foundation for making Simulator Controller accessible to thousands of casual racers like Marcus.

**Requirements Revealed:**
- CONTRIBUTING.md with clear guidelines
- Test infrastructure for safe contributions
- Architecture documentation
- Small, focused PR workflow
- Code review responsiveness
- Contributor recognition
- First contribution wizard (auto-generate test stubs)

---

### 7.4 Journey 4: SeriousOldMan — Receiving a Modernization Proposal

**Persona:** Oliver (SeriousOldMan), original developer of Simulator Controller. Solo maintainer for 6+ years. Passionate about sim racing and AI assistants. Receives many feature requests but few quality contributions.

**Opening Scene:**
Oliver wakes up to a GitHub notification: "New Issue: Comprehensive Modernization Proposal + PRD."

He sighs. He's seen these before. Usually someone saying "rewrite everything in Rust" or "just use Electron" with no understanding of the constraints. He clicks, expecting disappointment.

**Rising Action:**
This one is different. The proposal starts with:
> *"I'm a user who struggled with setup. I spent three evenings unable to get Jona working. Rather than complain, I want to help fix this — starting with tests and documentation."*

The PRD includes:
- Deep analysis of the existing architecture (they actually read the code)
- Respect for working systems (C++ connectors stay)
- Incremental approach (no big-bang rewrite)
- Specific, measurable goals
- Author's own user experience as research

Oliver reads the priority stack: "Testing Infrastructure first." That's exactly what he's wanted but never had time for.

**Climax:**
The first PR arrives: "Add Test-Along framework + 15 tests for Profile Manager."

Oliver reviews it. The code is clean. The tests actually pass. The documentation explains WHY each test exists. This person understands the codebase.

He merges it.

**Resolution:**
Over the following months, Oliver and Joe collaborate:
- Joe handles testing, documentation, and UI experiments
- Oliver focuses on AI and telemetry improvements
- Pull requests become conversations
- The project gains three new active contributors

Oliver posts in Discord: *"Thanks to Joe and our new contributors, the next release includes a Quick Start wizard. First-time setup now takes under 15 minutes."*

**Requirements Revealed:**
- Contribution model that respects maintainer time
- Trust-building through small wins
- Clear communication in PRs
- Documentation-first approach
- Collaborative, not demanding, tone

---

### 7.5 Journey 5: Derek — The Hardware Integration Expert

**Persona:** Derek, 42, retired IT engineer. Converted his garage into a sim racing cockpit with motion platform, button boxes, bass shakers, and custom Arduino controllers. He knows Windows inside-out.

**Opening Scene:**
Derek's rig is complex: 6DOF motion platform, 4 button boxes, Stream Deck, wind simulator, bass shakers. Each device needs to talk to Simulator Controller. Currently, he has 47 settings configured manually, 3 AHK scripts he wrote himself, and a 20-page Notion document explaining his setup.

A Windows update breaks something. His motion platform stops syncing. He needs to debug.

**Rising Action (Current State):**
Derek opens logs. They're scattered across 5 folders. He searches for "motion" — 847 results, mostly noise. He opens the configuration — which of the 47 settings controls motion timing? He remembers setting it up two years ago but not which menu.

Four hours later, he finds the issue: a COM port changed. He fixes it and updates his Notion document to page 21.

**Rising Action (With Modernization):**
Derek saves his entire setup as a profile: "Derek's Garage Cockpit v2.3."

When the Windows update breaks COM ports, he:
1. Opens diagnostics — sees "Motion Platform: COM3 not found, expected COM5"
2. Fixes COM port assignment
3. Profile auto-updates with new port
4. Zero Notion pages needed

**Climax:**
Derek creates a second profile: "Travel Rig" for the portable setup he takes to sim racing events. One click switches between garage cockpit and travel rig. All 47 settings, all device mappings, all button assignments — swapped instantly.

**Resolution:**
Derek shares his "Garage Cockpit" profile in Discord. Three other motion platform users download it and report: "Saved me hours of configuration." Derek becomes the unofficial "motion rig expert" in the community, helping others with hardware integration.

**Requirements Revealed:**
- Profile export/import
- Device mapping persistence
- Diagnostic logging with actionable messages
- Multiple profile support
- Community profile sharing
- Hardware change detection
- Profile repair wizard

---

### 7.6 Journey 6: The Returning Racer (Time Traveler Council Addition)

**Persona:** Marcus (from Journey 1), 6 months later. Life got busy — new job, family obligations. He hasn't raced since spring.

**Opening Scene:**
It's a quiet Saturday. Marcus remembers how much he enjoyed racing with Jona. He launches Simulator Controller for the first time in 6 months.

His profile — "Marcus's ACC Setup" — loads... but something's wrong:
- ACC updated its API while he was away
- Windows reassigned his wheel to a different USB port
- The voice recognition model was updated in a patch

The old software would have just failed silently. Marcus would have spent an hour debugging or given up entirely.

**Rising Action (With Modernization):**
Instead, Marcus sees: **"Profile Health Check"**

> "Welcome back, Marcus! 3 issues found with your profile:
> ❌ ACC API v2.3 → v2.5 (compatibility update needed)
> ❌ Logitech G29 moved from USB3 to USB4
> ❌ Voice model updated — redownload required
>
> [Fix All] [Review Each]"

He clicks "Fix All." 45 seconds later:

> "✅ All issues resolved. Profile updated to v2.4. Ready to race!"

**Climax:**
First race back. Lap 1. Jona speaks:

> *"Welcome back, Marcus. It's been a while. Your tire pressures are set for the old weather model — I've adjusted them for today's conditions. Let's have a good one."*

Marcus smiles. The software remembered him. It adapted. It felt like coming home.

**Resolution:**
Marcus is back to racing twice a week. He tells a coworker about Simulator Controller. When they struggle with setup, Marcus says: "Just use Quick Start. And don't worry if you take a break — it remembers you."

**Requirements Revealed:**
- Profile health check on launch
- Automatic compatibility detection
- Graceful handling of external changes
- One-click repair for common issues
- "Welcome back" re-engagement flow
- Version migration for profiles

---

### 7.7 Journey 7: Trust Graduation — Marcus's First League Race (Time Traveler Council Addition)

**Persona:** Marcus (from Journey 1), 2 months into using Simulator Controller. His friend Elena invited him to join her Thursday league.

**Opening Scene:**
Marcus is nervous. This isn't casual hot-lapping anymore. This is competition. Points matter. His setup has worked fine for solo races, but what if something goes wrong when it counts?

He's heard horror stories: software crashes mid-race, telemetry disconnects, voice recognition failing at critical moments. He trusts Simulator Controller for fun — but does he trust it for *this*?

**Rising Action:**
Thursday, 7:30 PM. 30 minutes to race start. Marcus opens Simulator Controller and sees something new:

**Pre-Race Checklist**

> "League Race Detected: ACC Thursday Championship
>
> ✅ Telemetry: Connected (47ms latency — excellent)
> ✅ Voice Recognition: Active (last test: 2 min ago)
> ✅ AI Assistant: Jona ready
> ✅ Strategy: Loaded (2-stop, Medium → Hard)
> ✅ Profile: Marcus's ACC Setup v2.4
>
> **All systems ready. You've got this, Marcus.**"

He exhales. The software knows this matters. It checked everything for him.

**Climax:**
Lap 15 of 30. Marcus is P8, his best position ever in competition. His hands are sweating. Jona speaks:

> *"Marcus, you're doing great. Tire temps are stable. P7 is 1.2 seconds ahead and struggling with rear grip. You've got pace on him. Stay focused."*

He catches P7. Then P6. He finishes P5 — his best league result ever.

**Resolution:**
Post-race, Elena messages him: "P5! Not bad for a rookie!" Marcus replies: "Jona kept me calm. I actually trusted the strategy."

Marcus is no longer a casual user. He's a competitive racer who relies on Simulator Controller for every league race. The trust graduation is complete.

**Requirements Revealed:**
- Pre-race checklist for competitive events
- League/competition detection
- System status confidence indicators
- Latency monitoring and display
- Encouraging but professional messaging
- Trust-building through transparency

---

## 8. Journey Requirements Summary

### 8.1 Capability Matrix

| Capability | Marcus | Elena | Joe | Oliver | Derek | Return | Trust |
|------------|--------|-------|-----|--------|-------|--------|-------|
| Quick Start wizard | ✅ | | | | | | |
| Auto-detection | ✅ | | | | | | |
| Progressive disclosure | ✅ | | | | | | |
| Configuration profiles | ✅ | | | | ✅ | ✅ | |
| Confusion recovery | ✅ | | | | | | |
| Diagnostic panel | | ✅ | | | ✅ | | |
| Pre-race checklist | | ✅ | | | | | ✅ |
| Post-incident reassurance | | ✅ | | | | | |
| Post-session reports | | ✅ | | | | | |
| Test infrastructure | | | ✅ | ✅ | | | |
| CONTRIBUTING.md | | | ✅ | ✅ | | | |
| First contribution wizard | | | ✅ | | | | |
| Multi-profile support | | | | | ✅ | | |
| Profile sharing | | | | | ✅ | | |
| Profile repair wizard | | | | | ✅ | ✅ | |
| Profile health check | | | | | | ✅ | |
| Trust-building UI | | | | | | | ✅ |

### 8.2 Journey-to-Feature Traceability

| Feature | Primary Journey | Supporting Journeys |
|---------|-----------------|---------------------|
| Quick Start Wizard | Marcus (J1) | Trust Graduation (J7) |
| Configuration Profiles | Marcus (J1) | Derek (J5), Returning (J6) |
| Diagnostic Panel | Elena (J2) | Derek (J5) |
| Pre-Race Checklist | Trust Graduation (J7) | Elena (J2) |
| Test Infrastructure | Joe (J3) | Oliver (J4) |
| Profile Health Check | Returning (J6) | Derek (J5) |
| Post-Session Reports | Elena (J2) | — |
| Progressive AI Disclosure | Marcus (J1) | — |

---

## 9. Innovation & Novel Patterns

### 9.1 Existing Product Innovation (Foundation)

Simulator Controller already contains significant technical innovation:

| Innovation | Description | Status |
|------------|-------------|--------|
| **AI Race Engineer (Jona)** | Voice-interactive AI providing real-time pit strategy, tire advice | Exists |
| **RETE Rule Engine** | Forward/backward chaining AI for real-time racing decisions | Exists |
| **LLM Coaching (Aiden)** | Modern LLM integrated with driving telemetry for personalized coaching | Exists |
| **Multi-Sim Unified AI** | Same AI assistants working across 11 different simulators | Exists |
| **Voice-First Interaction** | Natural language commands during racing without taking hands off wheel | Exists |

### 9.2 New Paradigm Innovations (Modernization Focus)

Through reverse engineering from ideal user outcomes, four paradigm shifts were identified:

#### Innovation 1: Invisible Setup

| Aspect | Detail |
|--------|--------|
| **Current Paradigm** | "Configure everything, then use" |
| **New Paradigm** | "Use immediately, we configure around you" |
| **Concrete Feature** | Background Discovery Service |
| **How It Works** | During splash screen: scan for simulators, detect hardware, create provisional profile |
| **User Experience** | "I see you have ACC and a G29. Ready to race?" → Single click → Racing |

#### Innovation 2: Anticipatory AI

| Aspect | Detail |
|--------|--------|
| **Current Paradigm** | "Ask and receive" — user must request information |
| **New Paradigm** | "AI anticipates and delivers at the right moment" |
| **Concrete Feature** | Proactive Insight Engine |
| **How It Works** | Rule-based triggers for "moments that matter" + cognitive load detection |
| **User Experience** | Jona speaks unprompted: "Gap to P4 closing. Push now or you'll be defending." |

#### Innovation 3: Performance Journey Tracking

| Aspect | Detail |
|--------|--------|
| **Current Paradigm** | "Each session is isolated" — no longitudinal view |
| **New Paradigm** | "Every session contributes to your racing story" |
| **Concrete Feature** | Racing Progress System |
| **How It Works** | Longitudinal telemetry database, skill gap analysis, automated improvement detection |
| **User Experience** | "Your lap times at Monza improved 2.3s. Biggest opportunity: Turn 1 entry speed." |

#### Innovation 4: Profiles as Knowledge Transfer

| Aspect | Detail |
|--------|--------|
| **Current Paradigm** | "Configuration is personal" — not shareable |
| **New Paradigm** | "Configuration is shareable knowledge" |
| **Concrete Feature** | Profile Ecosystem |
| **How It Works** | Portable versioned format, community repository, compatibility validation |
| **User Experience** | Derek exports "Garage Cockpit" profile, friend imports it, motion rig works immediately |

### 9.3 Innovation Validation Approach

| Innovation | Validation Method | Success Metric |
|------------|-------------------|----------------|
| Invisible Setup | A/B test: wizard vs auto-detect | >90% completion in <2 minutes |
| Anticipatory AI | User survey + telemetry analysis | "Right moment" rating >80% |
| Performance Journey | Retention cohort analysis | +20% return rate vs control |
| Profiles as Knowledge | Community adoption metrics | 100+ shared profiles in Year 1 |

### 9.4 Innovation Risk Mitigation

| Innovation | Risk | Mitigation Strategy |
|------------|------|---------------------|
| **Invisible Setup** | Wrong auto-detection frustrates users | "Customize" escape hatch always visible; show what was detected |
| **Anticipatory AI** | Speaks at wrong moment, distracts driver | Cognitive load detection (straights vs corners); user verbosity control |
| **Performance Journey** | Storage bloat from telemetry history | Aggregate old data, retain insights only; configurable retention |
| **Profiles as Knowledge** | Incompatible device references | Validation on import; clear compatibility warnings; graceful degradation |

### 9.5 Innovation-to-Journey Connection

| Innovation | Primary Journey | How It Transforms Experience |
|------------|-----------------|------------------------------|
| Invisible Setup | Marcus (J1) | 90-second download-to-racing instead of 60+ minutes |
| Anticipatory AI | Elena (J2) | AI provides strategy without being asked during race pressure |
| Performance Journey | Marcus (J1), Trust (J7) | Users see tangible improvement, builds commitment |
| Profiles as Knowledge | Derek (J5), Returning (J6) | Complex configs become shareable, recoverable |

---

## 10. Desktop Application Specific Requirements

### 10.1 Platform Strategy

| Requirement | Decision | Rationale |
|-------------|----------|-----------|
| **Target OS** | Windows 10 1809+ (64-bit only) | Sim racing ecosystem is Windows-centric; WebView2 requires 1809+ |
| **Cross-Platform** | Windows-only | No macOS/Linux—sim racing is 95%+ Windows |
| **Runtime Dependencies** | .NET 8, VC++ 2019 Redist, WebView2 | WebView2 for modern UI panels; auto-installer recommended |
| **Architecture** | x64 only | Drop x86 support; sim racing hardware is universally 64-bit |

**Expert Warning:** WebView2 Runtime is NOT pre-installed on older Windows 10 builds. Auto-detect and prompt for installation during first launch.

### 10.2 Installation & Updates

| Requirement | Decision | Rationale |
|-------------|----------|-----------|
| **Distribution** | ZIP primary (no installer overhead) | Scored 3.75; 80% of users comfortable with ZIP |
| **MSI Installer** | Conditional (Phase 2+) | Only if data shows >10% requesting installer |
| **Update Mechanism** | In-app update with race-aware timing | Never interrupt: startup check only, defer during active sim |
| **Data Migration** | First-launch wizard for v5→v6 profiles | Explicit user confirmation before modifying settings |
| **Portable Mode** | Full support | Run from USB drive with no registry/AppData writes |

**Key Decision:** Versioned migration with rollback capability. If profile conversion fails, preserve original and create backup.

### 10.3 System Integration

| Requirement | Decision | Rationale |
|-------------|----------|-----------|
| **Shared Memory** | Keep existing C++ connectors | Working code; no user-facing pain from current implementation |
| **COM Ports** | Auto-detect with change notifications | Hardware changes should be surfaced, not silently fail |
| **Credential Storage** | Windows Credential Manager + visibility option | Secure but not opaque; show "●●●●●" with reveal toggle |
| **System Tray** | Minimize to tray with status indicator | Green=connected, Yellow=degraded, Red=disconnected |
| **File Associations** | .scp (profile), .scr (replay) | Optional; prompt during install |

**Expert Requirement:** Use WASAPI exclusive mode for sub-100ms voice latency during racing.

### 10.4 Offline & Resource Management

| Requirement | Decision | Rationale |
|-------------|----------|-----------|
| **Offline Mode** | Full offline-first architecture | Racing happens without internet; all core features work offline |
| **Cloud Sync** | Optional, user-initiated only | Race-aware: never sync during active session |
| **Storage Location** | User-configurable; default Documents | Respect portable mode; never hard-code paths |
| **Memory Footprint** | <200MB base, <500MB with AI active | Profile for racing rigs with limited RAM |
| **Background CPU** | <2% when not actively processing | Sim racing needs every CPU cycle |

### 10.5 Expert Panel Requirements

#### Windows Platform (Dr. Helena Park)
- REQ-WIN-01: WebView2 runtime detection and graceful installation prompting
- REQ-WIN-02: DPI-aware UI for 4K/ultrawide monitors (common in sim racing)
- REQ-WIN-03: Multi-monitor support with window position persistence
- REQ-WIN-04: Admin privilege detection and guidance (some sims require elevated rights)

#### Real-Time Performance (Raj Krishnamurthy)
- REQ-RT-01: <16ms telemetry polling (match 60Hz minimum refresh)
- REQ-RT-02: WASAPI exclusive mode for voice latency <100ms
- REQ-RT-03: Dedicated thread for shared memory access (never block UI)
- REQ-RT-04: Graceful degradation during high CPU load (reduce AI verbosity, not crash)

#### Voice & AI (Dr. Yuki Tanaka)
- REQ-VOICE-01: Voice latency <500ms from trigger to first syllable
- REQ-VOICE-02: Voice activity detection with racing noise filtering
- REQ-VOICE-03: Confidence scoring for speech recognition (show user what AI heard)

#### Open Source Community (Nadia Okonkwo)
- REQ-OSS-01: Contributor-first decisions; maintainer sustainability
- REQ-OSS-02: Public roadmap aligned with community feedback
- REQ-OSS-03: Issue triage within 48 hours for new contributors

**Expert Warnings:**
1. ⚠️ WebView2 may conflict with some VR runtimes—test thoroughly
2. ⚠️ SAPI voice engines vary wildly in quality; test across Windows versions
3. ⚠️ Shared memory access patterns differ per sim—don't unify connector architecture
4. ⚠️ Community governance needs thought BEFORE scaling contributors

### 10.6 Innovation Opportunities (Genre Mashup Discoveries)

Cross-domain analysis revealed 7 innovations applicable to sim racing:

| Innovation | Source Domain | Effort | Impact | Description |
|------------|---------------|--------|--------|-------------|
| **Pre-Race Mental Prep** | Calm (Meditation) | LOW | MEDIUM | 2-minute guided breathing before race; reduce anxiety, improve focus |
| **Post-Race Debrief** | Aviation (Flight Training) | LOW | MEDIUM | Structured "what went well / what to improve" flow post-session |
| **Season Wrapped** | Spotify (Social Music) | MEDIUM | HIGH | End-of-month/year summary: total laps, best moments, improvement trends |
| **Daily Driving Drills** | Duolingo (Language Learning) | MEDIUM | HIGH | 10-minute targeted practice: "Today: Trail braking at Monza T1" |
| **Sector Leaderboards** | Strava (Fitness) | MEDIUM | MEDIUM | Friends/community comparison on specific track sectors |
| **Driver DNA Profile** | Spotify (Social Music) | HIGH | HIGH | "You're an aggressive late-braker with smooth exits" personality profile |
| **Driver Certifications** | Aviation (Flight Training) | MEDIUM | HIGH | Progressive skill badges: "Wet Weather Certified", "Endurance Ready" |

**Recommended Top 3 for MVP/Phase 2:**
1. **Pre-Race Mental Prep** (LOW effort, immediate value)
2. **Post-Race Debrief** (LOW effort, builds habit)
3. **Season Wrapped** (MEDIUM effort, HIGH engagement)

---

## 11. Project Scoping & Phased Development

> *Note: This section expands on [Section 6 (Product Scope)](#6-product-scope) with detailed feature-level phasing, conditional triggers, dependency maps, and risk mitigation strategies.*

### 11.1 MVP Strategy & Philosophy

**MVP Approach:** Hybrid MVP (Trust + Value in Parallel)

**Rationale:** The open source contribution model requires both trust-building (tests, docs) AND visible user impact (profiles, Quick Start) to succeed. Delivering one without the other risks either rejection (no trust) or irrelevance (no user value).

**Resource Requirements:**
- Primary contributor: Joe (part-time, evenings/weekends)
- Collaboration: SeriousOldMan (code review, merge authority)
- Community: Discord feedback, testing volunteers

### 11.2 MVP Feature Set (Phase 1: Foundation)

**Core User Journeys Supported:**
- Marcus (J1): Quick Start + Profile → First race with Jona
- Joe (J3): Test-Along + CONTRIBUTING.md → First merged PR
- Oliver (J4): Receives quality contributions that respect his time

**Must-Have Capabilities:**

| # | Capability | Journey | Success Criterion |
|---|-----------|---------|-------------------|
| 1 | Test-Along framework | J3, J4 | >60% coverage on touched code |
| 2 | Configuration Profiles (in existing AHK UI) | J1, J5 | Save/load/switch profiles |
| 3 | Quick Start wizard | J1 | <15 min first setup |
| 4 | Basic instrumentation | All | Setup funnel tracking |
| 5 | Security fundamentals | All | Password hashing, token rotation |
| 6 | CONTRIBUTING.md + Architecture docs | J3, J4 | Build in <30 min |
| 7 | Settings usage audit | J1 | Data on which settings are used |
| 8 | Simulator auto-detection | J1 | Detect installed sims on launch |

**Cannot Be Deferred:**
- Tests (foundation for all changes)
- Profiles (validates core hypothesis)
- Instrumentation (need data before Phase 2 decisions)

**Can Be Manual Initially:**
- Profile sharing (Discord/file exchange before in-app)
- Hardware detection (manual selection OK for MVP)

### 11.3 Post-MVP Features (Phase 2: Growth)

**Conditional on Profile Success Data:**

| # | Capability | Trigger Condition | Journey |
|---|-----------|-------------------|---------|
| 1 | Progressive AI disclosure | Profiles adopted by >50% of new users | J1, J7 |
| 2 | Proactive AI insights | Instrumentation shows AI usage patterns | J2, J7 |
| 3 | Post-session reports | Session completion data available | J2 |
| 4 | Diagnostic panel | Support ticket patterns analyzed | J2, J5 |
| 5 | Profile health check | Return user data from instrumentation | J6 |
| 6 | Pre-race checklist | League/competition detection feasible | J7 |
| 7 | Pre-Race Mental Prep | Low effort, immediate value add | Genre Mashup |
| 8 | Post-Race Debrief | Low effort, builds session habit | Genre Mashup |
| 9 | Zero-config auto-detection | Hardware detection reliability >95% | J1, J5 |
| 10 | Profile export/import | Community demand validated | J5 |

**Additional Journeys Supported:**
- Elena (J2): Competitive reliability + diagnostics
- Derek (J5): Multi-profile + hardware management
- Returning Racer (J6): Profile health + welcome back

### 11.4 Expansion Features (Phase 3: Vision)

**Requires Phase 2 Data + Architecture Decisions:**

| # | Capability | Prerequisite |
|---|-----------|--------------|
| 1 | Modern UI shell (WebView2/Tauri) | Profile experiment data supports UI investment |
| 2 | React-based configuration wizard | WebView2 integration validated |
| 3 | Season Wrapped | Performance Journey data foundation built |
| 4 | Daily Driving Drills | Longitudinal telemetry tracking available |
| 5 | Driver DNA Profile | Enough session data for personality analysis |
| 6 | Driver Certifications | Skill tracking system in place |
| 7 | Community profile marketplace | Profile sharing adopted by community |
| 8 | Sector Leaderboards | Community features infrastructure ready |
| 9 | Team Server dashboard overhaul | REST API modernized with OpenAPI |
| 10 | Streaming voice with visual feedback | WebView2 UI available for visualization |

### 11.5 Risk Mitigation Strategy

**Technical Risks:**

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| AutoHotkey testing is hard | HIGH | HIGH | Research AHK test frameworks early; worst case, test at integration level |
| WebView2 conflicts with VR runtimes | MEDIUM | HIGH | Test thoroughly before committing; have fallback plan |
| Profile migration breaks existing configs | MEDIUM | HIGH | Versioned profiles with rollback; never modify originals |
| Performance impact from instrumentation | LOW | MEDIUM | Lightweight telemetry; disable in race mode |

**Contribution Model Risks:**

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Maintainer rejects proposal | MEDIUM | CRITICAL | Start with tests/docs (hard to say no); build relationship first |
| Scope creep from community requests | HIGH | MEDIUM | PRD defines boundaries; politely defer out-of-scope requests |
| Contributor burnout (Joe is part-time) | MEDIUM | HIGH | Realistic phase lengths; no time pressure; quality over speed |
| Incompatible architectural visions | LOW | HIGH | Discuss architecture BEFORE implementing; respect maintainer authority |

**Market/Community Risks:**

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Users don't adopt profiles | MEDIUM | HIGH | Instrumentation validates; pivot to alternative onboarding approach |
| Community governance issues | LOW | MEDIUM | Define governance before scaling contributors |
| Simulator API changes break connectors | MEDIUM | MEDIUM | Keep C++ connectors isolated; don't unify architecture |

### 11.6 Phase Dependency Map

```
Phase 1 (Foundation)
├── Test-Along Framework ────────────────────────┐
├── Configuration Profiles ──────────────────────┤
├── Quick Start Wizard ──────────────────────────┤
├── Instrumentation ─────────────────────────────┤
├── Security Fundamentals ───────────────────────┤
└── CONTRIBUTING.md + Docs ──────────────────────┤
                                                  │
Phase 2 (Growth) ← Conditional on Phase 1 Data ──┘
├── Progressive AI Disclosure ───────────────────┐
├── Proactive AI Insights ───────────────────────┤
├── Post-Session Reports ────────────────────────┤
├── Diagnostic Panel ────────────────────────────┤
├── Profile Health Check ────────────────────────┤
├── Pre-Race Checklist ──────────────────────────┤
├── Pre-Race Mental Prep ────────────────────────┤
├── Post-Race Debrief ──────────────────────────┤
└── Profile Export/Import ───────────────────────┤
                                                  │
Phase 3 (Vision) ← Conditional on Phase 2 Data ──┘
├── Modern UI Shell (WebView2/Tauri)
├── Season Wrapped
├── Driver DNA Profile
├── Community Profile Marketplace
└── Team Server Dashboard Overhaul
```

---

## 12. Functional Requirements

### 12.1 Onboarding & Setup

- **FR1:** New users can complete initial setup by selecting their simulator, hardware, and preferred AI assistant from a guided wizard
- **FR2:** The system can auto-detect installed simulators on the user's machine
- **FR3:** The system can auto-detect connected hardware controllers (wheels, button boxes)
- **FR4:** Users can select a pre-built configuration profile matching their simulator and hardware combination
- **FR5:** Users can skip advanced settings and begin racing with sensible defaults
- **FR6:** Users can return to setup at any time to modify their configuration
- **FR7:** The system can display a progress indicator during first-time setup showing remaining steps

### 12.2 Configuration Profiles

- **FR8:** Users can save their current configuration as a named profile
- **FR9:** Users can load a previously saved profile to restore all settings
- **FR10:** Users can switch between multiple saved profiles
- **FR11:** Users can export a profile to a portable file format
- **FR12:** Users can import a profile from an external file
- **FR13:** The system can validate imported profiles for compatibility with current hardware/software
- **FR14:** The system can detect profile health issues on launch (changed hardware, updated APIs, moved devices)
- **FR15:** Users can repair detected profile issues through guided resolution
- **FR16:** The system can preserve original profile data before migration or repair operations

### 12.3 AI Assistant Interaction

- **FR17:** Users can interact with AI assistants (Jona, Cato, Elisa, Aiden) via voice commands during racing sessions
- **FR18:** The system can deliver proactive AI insights at contextually appropriate moments without user prompting
- **FR19:** Users can control AI assistant verbosity level (minimal, standard, detailed)
- **FR20:** The system can progressively introduce AI assistants (starting with Jona, unlocking others based on usage)
- **FR21:** Users can manually enable or disable individual AI assistants
- **FR22:** The system can display confidence indicators for AI recommendations
- **FR23:** The system can generate post-session summary reports with key decisions and outcomes

### 12.4 Session Management & Telemetry

- **FR24:** The system can connect to simulator telemetry automatically when a supported simulator is running
- **FR25:** The system can display connection status for all active telemetry sources
- **FR26:** The system can preserve session data if the simulator or application terminates unexpectedly
- **FR27:** Users can review historical session data including laps, stints, and strategy decisions
- **FR28:** The system can track performance trends across sessions (lap times, consistency, improvement areas)
- **FR29:** Users can view a pre-race system checklist confirming all connections and readiness

### 12.5 Diagnostics & Reliability

- **FR30:** The system can display a diagnostic panel showing status of all subsystems (telemetry, voice, AI, hardware)
- **FR31:** The system can provide actionable error messages with specific recovery suggestions
- **FR32:** Users can reset individual subsystems without restarting the entire application
- **FR33:** The system can operate fully offline with all core features available without internet
- **FR34:** The system can defer non-critical operations (updates, sync) during active racing sessions

### 12.6 Contributor & Developer Experience

- **FR35:** Contributors can build the project from source following documented instructions
- **FR36:** Contributors can run automated tests to validate their changes
- **FR37:** Contributors can generate test stubs for components they're modifying
- **FR38:** The system can collect anonymized usage telemetry (with opt-in consent) for feature adoption analysis
- **FR39:** The system can track setup funnel completion rates for onboarding optimization

### 12.7 Security & Data Management

- **FR40:** The system can store credentials securely using the OS credential manager
- **FR41:** Users can view which credentials are stored (masked) and revoke them
- **FR42:** The system can manage authentication tokens with automatic rotation for Team Server connections

---

## 13. Non-Functional Requirements

### 13.1 Performance

- **NFR1:** Telemetry polling completes within 16ms (60Hz minimum refresh rate)
- **NFR2:** Voice response latency <500ms from trigger to first audible syllable
- **NFR3:** Application base memory footprint <200MB; <500MB with AI assistants active
- **NFR4:** Background CPU usage <2% when not actively processing telemetry or voice
- **NFR5:** Profile switching completes within 2 seconds including all device remapping
- **NFR6:** Setup wizard loads and is interactive within 3 seconds of launch
- **NFR7:** Shared memory connectors introduce <1ms latency over direct access

### 13.2 Reliability

- **NFR8:** Application crash rate <0.1% of racing sessions
- **NFR9:** Session data preserved if application or simulator terminates unexpectedly
- **NFR10:** All core features (telemetry, voice, AI) function without internet connectivity
- **NFR11:** System degrades gracefully under high CPU load (reduce AI verbosity rather than crash)
- **NFR12:** Profile operations never corrupt or destroy existing configuration data
- **NFR13:** Updates and non-critical sync never execute during an active racing session
- **NFR14:** System recovers to functional state within 10 seconds of any subsystem failure

### 13.3 Security

- **NFR15:** All stored passwords use bcrypt or equivalent adaptive hashing (never plaintext or reversible encryption)
- **NFR16:** Team Server authentication tokens rotate automatically; maximum token lifetime 24 hours
- **NFR17:** Credentials stored in Windows Credential Manager, not in plaintext config files
- **NFR18:** API communication with Team Server uses TLS 1.2+ exclusively
- **NFR19:** Instrumentation telemetry anonymized; no PII transmitted without explicit opt-in consent

### 13.4 Compatibility

- **NFR20:** Supports Windows 10 version 1809 and all subsequent Windows 10/11 releases
- **NFR21:** x64 architecture only (x86 not required)
- **NFR22:** All 11 simulator telemetry connectors maintain backward compatibility with previous connector API versions
- **NFR23:** WebView2 components (if introduced) detect runtime availability and provide installation guidance if missing
- **NFR24:** Application does not conflict with VR runtimes (SteamVR, Oculus, WMR)
- **NFR25:** Profile format includes version metadata to enable forward migration across application updates

### 13.5 Maintainability

- **NFR26:** Test coverage exceeds 60% for all modified code (Test-Along approach)
- **NFR27:** New contributor can build project from source within 30 minutes following documented instructions
- **NFR28:** All public API endpoints documented with OpenAPI specification
- **NFR29:** Architecture decisions recorded in ADRs for significant changes
- **NFR30:** Code follows consistent style conventions documented in CONTRIBUTING.md
- **NFR31:** Each PR includes tests for all touched code; CI passes before merge
