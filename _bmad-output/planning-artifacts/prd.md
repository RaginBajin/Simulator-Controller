---
stepsCompleted: ['step-01-init', 'step-02-discovery', 'step-03-success', 'step-04-journeys', 'step-05-domain', 'step-06-innovation', 'step-07-project-type', 'step-08-scoping', 'step-09-functional', 'step-10-nonfunctional', 'step-11-polish']
lastPolished: '2026-02-01'
inputDocuments:
  - _bmad-output/planning-artifacts/product-brief.md
  - README.md
  - Docs/AI Driving Coach.md
  - Docs/AI Race Engineer.md
  - Docs/AI Race Spotter.md
  - Docs/AI Race Strategist.md
  - Docs/TELEMETRY_PROVIDERS.md
  - Docs/Session Database.md
  - Docs/Race Reports.md
  - _bmad-output/planning-artifacts/prd-v1-simulator-controller-contribution.md (reference)
workflowType: 'prd'
documentCounts:
  briefs: 1
  research: 0
  brainstorming: 0
  projectDocs: 8
  archivedPRD: 1
classification:
  projectType: Desktop Application + Web Dashboard
  domain: Gaming/Entertainment Tools (Sim Racing)
  complexity: HIGH
  context: Greenfield Development
  platform: Cross-platform (Windows primary)
---

# Product Requirements Document - AI Race Team

**Author:** Joe
**Date:** 2026-02-01

## Executive Summary

**AI Race Team** is a desktop application that automatically captures iRacing telemetry, processes it through a local Rust pipeline, and delivers AI-powered coaching debriefs — replacing fragmented manual workflows with a single unified loop.

**Differentiator:** No single feature is novel. Simulator Controller's Aiden does LLM + telemetry. VRS does visual traces. Garage61 does sharing. The innovation is the unified loop — capture → pre-processing → AI coaching → visualization → conversation → sharing — at zero-config quality. Replicating the full loop at this depth is a multi-year effort.

**Target Users:** Sim racers who want to improve but find traditional telemetry tools (VRS, MoTeC) intimidating or manual workflows (pasting data into ChatGPT) tedious. Primary persona: the obsessed improver who races weekly and wants specific, grounded coaching. Secondary: the newcomer who needs plain-English advice without understanding telemetry graphs.

**MVP Strategy:** Problem-Solving MVP — can it replace Joe's manual Claude Desktop telemetry workflow and be genuinely better? BYOK-only (user provides API key), per-corner text analysis, 1D lap-distance chart, session-over-session comparison. 14 must-have capabilities targeting Journey 1 (happy path) and Journey 2 (onboarding).

**Technology:** Tauri 2.0 (Rust core + web view), SQLite (metadata) + Parquet (telemetry), Windows-first (iRacing is Windows-only).

## Success Criteria

### User Success

**Core Loop Validation (Proven by Prototype)**
Joe's existing telemetry logger + LLM analysis workflow has already validated the fundamental product loop: capture telemetry → structured data → AI analysis → actionable coaching. The product succeeds when this loop is faster, richer, and more automated than the manual Claude Desktop workflow.

- **Zero-config capture**: Telemetry recording starts automatically when iRacing is running — no manual script launch, no file management
- **Actionable insight within 60 seconds**: After session ends, AI produces its first coaching insight (e.g., "You under-braked in every zone — avg 37% pressure, best lap used 90%") without user prompting
- **Visual telemetry + AI annotations**: Brake/throttle traces overlaid on track map with AI callouts — the key gap the prototype revealed
- **Conversational follow-up**: User asks "why was Lap 60 faster?" and gets a grounded answer referencing specific telemetry differences (0.90 vs 0.77 initial brake, 1.50/s vs 0.68/s release rate)
- **Partial lap analysis**: System captures and analyzes ALL data including incomplete laps — validated as containing critical diagnostic data (e.g., throttle oversteer incidents that caused resets)
- **Session-over-session memory**: AI references prior sessions ("Your brake pressure at T1 improved from 37% avg last week to 52% today") — something the prototype cannot do

**Aha Moments:**
1. First time the AI tells you something about your driving you didn't know (under-braking pattern)
2. First time you see your brake trace vs the fast lap and understand *why* visually
3. First time you share a debrief link and a friend says "I need this" *(Growth — requires web sharing infrastructure)*

**Data Ownership:**
- Export any session as Parquet/SQLite — no lock-in, ever
- Share debrief via web link without requiring recipient to install anything
- All data stored locally in open formats

### Business Success

| Timeframe | Metric | Target |
|-----------|--------|--------|
| 3 months | Dogfood validation | Joe uses it for every iRacing session and prefers it over Claude Desktop |
| 6 months | Early adopter traction | 50 beta users from iRacing forums, 70%+ weekly retention |
| 12 months | Product-market fit | 500+ active users, community word-of-mouth as primary growth |
| 12 months | Revenue signal | Freemium conversion rate >5% (free: limited sessions/month, paid: unlimited + advanced AI) |

### Technical Success

**Telemetry Platform:**
- Capture iRacing shared memory at 30-60Hz (validated by prototype at 30Hz/28K+ samples)
- Store as Parquet (columnar, compressed) — sub-second query for any session
- Support typed records: metadata, sample, lap_summary, session_summary (proven format from prototype)
- Track derived metrics: brake application count, trail braking phases, corner-by-corner segmentation

**AI Coaching Layer:**
- Car-specific prompt templates (MX-5 template proven in prototype, extensible to GT3, LMP2, etc.)
- Technique-specific analysis modules (braking, trail braking, throttle application, racing line)
- Per-corner analysis (not just per-lap — the next step beyond the prototype)
- Grounded responses: every AI claim references specific telemetry data points

**Architecture:**
- Rust core processes a full session (60 laps × 60Hz × 15 channels) in <2 seconds
- Tauri desktop app <50MB installed, <200MB RAM while recording
- SQLite + Parquet local storage, no cloud dependency for core features
- Web dashboard reads same data via WASM SQLite

### Measurable Outcomes

| Outcome | Metric | MVP Target | Growth Target |
|---------|--------|------------|---------------|
| Capture reliability | Sessions recorded without data loss | 99% | 99.9% |
| Time to first insight | Seconds from session end to AI coaching | <60s | <30s |
| Coaching accuracy | AI insights rated "useful" by user | >70% | >85% |
| Corner identification | AI correctly identifies problematic corners | >80% | >90% |
| Session comparison | AI references prior session data accurately | >80% | >90% |
| Sharing engagement | Shared debriefs viewed by recipient | N/A (Growth: web sharing) | 50%+ click-through |

## Product Scope

The authoritative MVP feature set, phase boundaries, and scoping rationale are defined in [Project Scoping & Phased Development](#project-scoping--phased-development). That section reflects intensive scoping analysis including Comparative Analysis Matrix, Time Traveler Council, and 5 Whys Deep Dive. Summary:

- **MVP (Phase 1):** iRacing auto-capture, BYOK AI provider (Claude/OpenAI key), per-corner text analysis, 1D lap-distance chart, session-over-session comparison, conversational follow-up, implicit training recommendations. Windows desktop (Tauri). Single-user, single-machine. 14 must-have capabilities.
- **Growth (Phase 2):** Track map overlay with AI annotations, three-tier provider model (Local + BYOK + Managed), web-shareable debriefs, persistent adaptive training plans, stint analysis, voice coaching, AI confidence indicators. 13 capabilities.
- **Expansion (Phase 3):** ACC/LMU simulator support, multi-driver comparison, team/league features, live race mode (AI crew chief), AI strategist/spotter, generative training plans, predictive lap modeling. 9 capabilities.

## User Journeys

### Journey 1: Joe — The Obsessed Improver (Primary, Happy Path)

**Opening Scene:** It's Thursday night. Joe fires up iRacing for his weekly league race at Lime Rock. He's been stuck in the 58s and knows he's leaving time on the table in the braking zones — his Claude Desktop analysis told him so last week. But tonight, AI Race Team is running silently in the system tray.

**Rising Action:** Joe races 25 laps. He pushes harder into T1, trying to brake later and harder. Some laps click, some don't. After the checkered flag, he closes iRacing and opens the debrief. Within 30 seconds, the AI presents: "Session summary: 25 laps, best 57.8s. You improved initial brake pressure from 37% avg last session to 52% this session. Three corners still show early release — T1, T5, T7."

**Climax:** Joe clicks T5 on the track map. The brake/throttle trace appears overlaid on the corner, with his best lap in green and his average in gray. The AI annotates: "Your best lap trails brake to 25% through apex. Your average releases fully 15 meters before apex. That 15 meters is your 0.4s gap." Joe sees it. He *gets* it. For the first time, the gap isn't a number — it's a visible distance on the track.

**Resolution:** Joe shares the debrief link in his league's Discord. His teammate clicks it, loads his own session from the same race, and overlays both side-by-side. They can see exactly where Joe gains in the braking zone and where his teammate is faster on exit. The conversation shifts from "I don't know why you're faster" to "show me your T5 line."

**Requirements revealed:** Auto-capture, fast post-session analysis, track map with telemetry overlay, per-corner AI annotations, session comparison, shareable web links with comparison capability, system tray background operation, stint-level analysis for endurance races.

### Journey 2: Marcus — The League Newcomer (Secondary, Onboarding)

**Opening Scene:** Marcus just joined an iRacing league after six months of solo practice. He's mid-pack but doesn't know *why* he's slow. He downloaded VRS once, stared at squiggly lines for 20 minutes, and closed it. A league mate shared a debrief link from AI Race Team and Marcus thought "that actually makes sense."

**Rising Action:** Marcus installs the app. No account creation, no cloud setup — it auto-detects his iRacing installation and confirms: "Found iRacing. Ready to record." He runs a 15-minute practice at Spa. After the session, a system tray notification pops: "Debrief ready — Spa, 15 laps." He takes off his VR headset and clicks it 20 minutes later on his monitor. The AI says: "First session captured. Key finding: You're lifting off throttle mid-corner at Eau Rouge and Pouhon. Full throttle commitment through these corners would gain 1.2 seconds."

**Climax:** Marcus asks in the chat: "But won't I spin?" The AI responds: "Based on 11 of your 15 laps with similar entry speed (141-146 mph), the car maintained traction at full throttle. Your 4 partial-throttle laps were actually less stable due to weight transfer disruption." *(Growth vision: the AI will also display a confidence indicator — e.g., "Confidence: high" — grounded in the user's own data distribution.)* Marcus tries it next practice. He doesn't spin. He drops 1.1 seconds.

**Resolution:** A week later, Marcus opens the app and sees his progress: "Spa average: 2:24.1 → 2:22.8 over 3 sessions. Consistency improved from 1.4s std dev to 0.7s." He doesn't understand telemetry graphs. He doesn't need to. The AI tracks his improvement and tells him what to work on next.

**Requirements revealed:** Zero-config onboarding with auto-detection, VR-friendly async workflow, system tray notifications, plain English coaching, car-specific physics knowledge, AI confidence indicators grounded in user's own data, conversational follow-up, practice mode support, session-over-session progress tracking.

### Journey 3: Joe — The Bad Session (Primary, Edge Case/Recovery)

**Opening Scene:** Joe's in a 2-hour endurance race. At lap 40, his internet hiccups and iRacing disconnects for 8 seconds. He reconnects but the telemetry stream had a gap. Later in stint 3, his tires are gone — lap times balloon by 2 seconds. He pits, gets fresh tires, and his stint 4 is strong until he spins at T3 on lap 62 and resets to pits.

**Rising Action:** After the session, the debrief shows stint-level breakdown: Stint 1 (laps 1-18): strong, 58.2 avg. Stint 2 (laps 19-38): consistent, 58.4 avg. Stint 3 (laps 39-52): degradation curve, 58.5 → 60.1 by end. Stint 4 (laps 53-65): fast early, spin at lap 62. A telemetry gap warning: "Laps 40-41: 8.2s gap, data interpolated." The partial spin lap is preserved, not discarded.

**Climax:** Joe clicks the spin lap. The AI shows: "Throttle application reached 100% at mid-corner with 12° of steering lock. This exceeds the MX-5's traction limit at that speed. Your clean laps at T3 apply throttle progressively — 40% at apex, 70% at exit. Lap 62 jumped to 100% at apex." *(Growth vision: the AI will also surface tire degradation analysis — e.g., "Stint 3 degradation: brake temperatures rose 18°C above stint 1 baseline by lap 48" — and optimal pit window recommendations — e.g., "Optimal pit window was lap 45-47" — derived from cross-stint trend modeling.)*

**Resolution:** Joe sees that stint 1 and early stint 4 were his best driving of the session. The AI notes: "Laps 53-61 averaged 57.9s — your fastest sustained pace. Tire management in stint 3 cost approximately 8 seconds over 14 laps. Earlier pit stop would have recovered 5-6 seconds."

**Requirements revealed:** Graceful telemetry gap handling, partial/incomplete lap preservation, spin/incident detection, stint-level analysis, tire degradation tracking, optimal pit window calculation, emotional context (frustration-driven errors).

### Journey 4: Sarah — The Team Manager (Tertiary, Multi-User Future)

**Opening Scene:** Sarah manages a 3-driver endurance team. Each driver uses AI Race Team individually, but Sarah needs to optimize driver assignments for the 6-hour Daytona race.

**Rising Action:** Each driver shares their practice debrief links. Sarah opens the comparison view — three driver profiles side by side. Driver A: fastest raw pace (1:48.2 best), high variance (0.9s std dev). Driver B: most consistent (0.3s std dev), moderate pace (1:48.8). Driver C: best tire conservation (0.1s/lap degradation vs 0.3s/lap for others), slowest raw pace (1:49.1).

**Climax:** The AI suggests stint assignments: "For a 6-hour race with 4 pit stops: Opening double-stint → Driver C (tire conservation reduces pit stops). Middle stints → Driver B (consistency minimizes risk during traffic). Final stint → Driver A (raw pace for the push to the finish). Projected advantage over equal-stint strategy: 12-18 seconds." Sarah reviews the reasoning, adjusts one stint, and locks it in.

**Resolution:** They finish P3 — their best result. The post-race debrief shows each driver's actual performance vs projection. The AI notes: "Driver B exceeded projection by 0.2s/lap in stint 3 — consider extending B's stint allocation in future."

**Requirements revealed:** Multi-driver comparison view, AI-suggested stint assignments, tire degradation metrics per driver, consistency scoring, projection vs actual analysis, team debrief.

### Journey 5: Dave — The League Admin (Tertiary, Future)

**Opening Scene:** Dave runs a 24-driver iRacing league. After Saturday's race at Monza, he has three incident reports in his Discord: "Car #7 brake-checked me into T1," "Car #14 squeezed me off track at Parabolica," and "Lap 1 T1 pileup — who caused it?"

**Rising Action:** Dave opens the race debrief (league edition). He has telemetry from all drivers who use AI Race Team. He clicks "Incident Review" and selects the T1 lap 1 timestamp. The tool reconstructs the first three cars' telemetry: braking points, speeds, lateral positions.

**Climax:** The AI presents: "Car #3 braked 22 meters later than the two preceding races at T1, closing speed was 34 mph higher than the car ahead. Car #7 braked at their normal point. Contact initiated by Car #3's late braking." Dave has telemetry-backed evidence instead of he-said-she-said.

**Resolution:** Dave shares the incident reconstruction link in the protest channel. Both drivers can see the data. The ruling is clear, fair, and accepted. The league's reputation for fair stewarding improves.

**Requirements revealed:** Multi-car telemetry overlay for incident reconstruction, timestamp-based incident lookup, braking point comparison between cars, proximity/contact detection, shareable incident review links, league admin role.

### Journey Requirements Summary

| Capability | Phase | J1 (Joe) | J2 (Marcus) | J3 (Bad Session) | J4 (Sarah) | J5 (Dave) |
|---|---|:---:|:---:|:---:|:---:|:---:|
| Auto-capture telemetry | **MVP** | X | X | X | | |
| Auto-detect iRacing | **MVP** | | X | | | |
| Zero-config setup | **MVP** | | X | | | |
| VR-friendly async workflow | **MVP** | | X | | | |
| System tray notifications | **MVP** | | X | | | |
| Per-corner AI annotations (text) | **MVP** | X | | X | | |
| 1D lap-distance chart | **MVP** | X | | X | | |
| Plain English coaching | **MVP** | | X | | | |
| Conversational follow-up | **MVP** | X | X | | | |
| Session-over-session comparison | **MVP** | X | | | | |
| Progress tracking over time | **MVP** | | X | | | |
| Telemetry gap handling | **MVP** | | | X | | |
| Partial lap preservation | **MVP** | | | X | | |
| Track map + telemetry overlay | Growth | X | | X | | X |
| AI confidence indicators | Growth | | X | | | |
| Shareable web debrief | Growth | X | | | X | X |
| Stint-level analysis | Growth | X | | X | | |
| Spin/incident detection | Growth | | | X | | X |
| Tire degradation tracking | Growth | | | X | X | |
| Optimal pit window calc | Growth | | | X | | |
| Comparison overlay (multi-driver) | Expansion | X | | | X | X |
| AI stint assignment suggestions | Expansion | | | | X | |
| Multi-car incident reconstruction | Expansion | | | | | X |
| League admin role | Expansion | | | | | X |

## Innovation & Novel Patterns

### Detected Innovation Areas

**1. Telemetry-Grounded AI Coaching (Accessible Execution)**
Simulator Controller's Aiden already grounds LLM responses in telemetry JSON — the concept exists. The innovation is making it accessible. Aiden requires Windows, AutoHotkey, 30 minutes of configuration, and speaks through a voice synth during live racing. This product does it post-session with zero config, visual telemetry overlays, conversational follow-up, and shareable web links. The innovation isn't "nobody does this" — it's "nobody does this well enough that normal people use it." No single feature here is defensible in isolation. A competitor could replicate any one piece. The moat is the integrated experience: deep telemetry → intelligent pre-processing → grounded AI → visual annotation → conversational follow-up → shareable output. Replicating the full loop at this quality level is a multi-year effort.

**2. The Unified Loop**
Existing tools fragment the workflow: capture in one tool, analyze in another, share via screenshots, discuss on Discord with no data context. This product unifies capture → analysis → AI coaching → visualization → conversation → sharing into a single seamless loop. The loop is the UX innovation — what hooks users. The moat underneath is data depth and AI coaching quality, which competitors would need to rebuild from scratch.

**3. AI Confidence Grounding**
The AI doesn't just give advice — it shows its work. "Based on 11 of your 15 laps with similar entry speed, confidence: high." This addresses the core trust problem with AI coaching: racers won't follow advice they can't verify. Every insight is traceable to specific telemetry data points. At launch, confidence indicators are heuristic (based on sample size and data variance). True calibration — validating that high-confidence advice correlates with actual improvement — requires tracking outcomes over time and is a Growth-phase capability.

**4. AI-Powered Incident Diagnosis**
Industry tools preserve or discard partial laps. This product goes further: the AI interprets *why* a lap was incomplete. "Throttle oversteer at T4 — 100% throttle mid-corner exceeded traction limit" turns a truncated telemetry trace into a specific, actionable diagnosis. Every anomaly — spins, resets, telemetry gaps, sudden pace drops — becomes a coaching opportunity rather than data noise.

### Market Context & Competitive Landscape

| Innovation | Closest Competitor | Their Approach | Our Differentiation |
|---|---|---|---|
| AI coaching | Simulator Controller (Aiden) | LLM + telemetry JSON, AutoHotkey, voice-during-race | Modern stack, visual, post-session, zero config |
| AI coaching | Coach Dave Delta | Rule-based analysis, data locked | LLM-powered, conversational, open data |
| Unified loop | VRS + Garage61 + Discord | 3+ tools, manual workflow | Single app, zero hand-offs |
| Data ownership | All competitors | Locked or limited export | SQLite + Parquet, fully portable |
| Shareable debriefs | Garage61 (limited) | Static screenshots/links | Interactive web debriefs with AI context |

### Validation Approach

1. **Prototype validation (DONE)**: Joe's telemetry logger + LLM analysis script proved the core AI coaching concept works — structured telemetry → Claude → actionable insights
2. **Dogfood validation (MVP)**: Joe uses it for every iRacing session and prefers it over the Claude Desktop workflow
3. **Peer validation (Growth)**: Share debriefs in league Discord — do teammates ask "how do I get this?"
4. **Retention validation (PMF)**: 70%+ weekly retention among beta users indicates the unified loop is genuinely better than fragmented tools

### Risk Mitigation

Innovation-specific risks. For comprehensive project risks (technical, market, resource), see [Risk Mitigation Strategy](#risk-mitigation-strategy) in the Scoping section.

| Innovation Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Users don't trust AI coaching advice | Medium | Medium | Confidence indicators, telemetry grounding, "show your work" — every insight traceable to data |
| Unified loop replicated by competitor | Medium | High | Depth of telemetry pipeline + AI coaching quality is the real moat; loop UX is the hook, not the lock-in |
| Unified loop is too opinionated | Low | Medium | Each piece works standalone (capture without AI, AI without sharing) |
| "Same as Aiden" perception in sim racing community | Medium | Medium | Lead with zero-config setup, visual output quality, and session memory — things Aiden can't do |

### Business Model (Open Question)

The innovation section focuses on technical novelty but doesn't yet address revenue sustainability. Key open question: what pricing model supports AI-powered coaching without losing money on power users? Options to explore in later steps: tiered AI depth (basic analysis free, deep coaching paid), session packs, flat monthly with aggressive local pre-processing to cap API costs. This needs resolution before launch but does not block MVP development (Joe is the only user during MVP).

## Desktop Application Requirements

### Project-Type Overview

AI Race Team is a **desktop-first application** with a future web companion. The desktop app (Tauri/Rust) handles telemetry capture, local processing, AI coaching, and debrief presentation. A web dashboard for debrief sharing and async review is a Phase 2 capability.

### Platform Support

| Platform | Phase | Status | Notes |
|---|---|---|---|
| Windows 10/11 | **MVP** | Required | iRacing is Windows-only; 95%+ of target users |
| Web browser | **Growth** | Phase 2 | Debrief sharing and viewing — no install needed for recipients |
| macOS | Post-MVP | Optional | iRacing does not run natively; only useful for debrief review |
| Linux | Vision | Optional | Small sim racing community on Linux (Proton/Wine) |

**Windows-first decision:** iRacing's IRSDK only works on Windows via shared memory. Cross-platform is irrelevant for the capture component. The web dashboard (Phase 2) provides cross-platform debrief access without requiring platform porting.

## AI Provider Architecture

The AI layer is designed around a three-tier provider model. **MVP ships BYOK-only.** The Rust trait architecture (`TextGeneration`, `VoiceGeneration`) supports all tiers without architectural debt — adding Local and Managed tiers in Phase 2 is adapter work, not refactoring.

### MVP Provider Model (Phase 1 — BYOK Only)

User provides their own Claude or OpenAI API key. Simple "paste your key" setup.

| Capability | MVP Provider | Notes |
|---|---|---|
| Telemetry analysis | Claude or OpenAI via BYOK key | User's choice at setup |
| Conversational follow-up | Same BYOK key | Same provider as analysis |

### Full Provider Vision (Phase 2+)

| Tier | Description | Phase | Target User |
|---|---|---|---|
| **Local** | User runs their own LLM (Ollama, llama.cpp, LM Studio) or OS-native TTS | Growth | Privacy-conscious, tinkerers, zero-cost users |
| **BYOK** | Bring Your Own Key — user provides API keys for Claude, OpenAI, Gemini, ElevenLabs, etc. | **MVP** | Power users who already have API accounts |
| **Managed** | We route through our APIs — user pays subscription, we handle provider selection and optimization | Growth | Mainstream users who want it to "just work" |

**Phase 2 provider routing by capability:**

| Capability | Local Option | BYOK Option | Managed Option |
|---|---|---|---|
| Telemetry analysis | Ollama, llama.cpp, LM Studio | Claude, OpenAI, Gemini API keys | Our API (routes to best provider) |
| Conversational follow-up | Same local LLM | Same BYOK key | Our API |
| Voice coaching | OS native TTS (Windows SAPI) | ElevenLabs, OpenAI TTS keys | Our API |
| Voice-to-text | OS native STT, Whisper local | OpenAI Whisper API key | Our API |

### Provider UX Design

**MVP Onboarding (2 screens):**
1. **Welcome** — auto-detect iRacing installation, confirm ready state
2. **API Key Setup** — "Paste your Claude or OpenAI API key." Validate connectivity. Done.

**Phase 2 Onboarding (3 screens with three-card selection):**
1. **Welcome** — auto-detect iRacing installation, confirm ready state
2. **AI Setup** — three visual cards per capability, no jargon:
   - **Quick Start** — "We handle the AI for you. Easiest setup." (Managed)
   - **Use My Key** — "I have a Claude/OpenAI API key." (Direct paste)
   - **Run Locally** — "I run my own AI models. (Ollama, etc.)" (Auto-detect)
3. **Ready** — "Go race. We'll analyze your session after."

**Design principles (all phases):**
- No jargon: "Quick Start" not "Managed tier," "Use My Key" not "BYOK"
- Validation on setup AND before each session — never discover broken AI at debrief time
- Never block capture: if any provider is unavailable, telemetry still records

**System tray status indicator:**
- 🟢 AI ready — provider connected
- 🟡 AI degraded — rate limited or slow
- 🔴 AI offline — capture continues, analysis unavailable

**Settings page (power users):**
- Provider selection per capability (text analysis, voice — Phase 2)
- Connection test buttons
- Usage summary (sessions analyzed, estimated tokens)

### Shared Debrief Links — Analytics & Tracking *(Phase 2)*

When a user shares a debrief link (requires web sharing infrastructure):
- **Link open tracking**: Capture who opens the link (IP, user agent, referrer, timestamp)
- **Engagement metrics**: Time on page, sections viewed, scroll depth
- **Identity capture**: If viewer has an account, link to profile; if not, cookie for later identification
- **Ad pixel integration**: Embed tracking pixels (Meta, Google Ads, etc.) on shared debrief pages for retargeting
- **Conversion funnel**: Track shared link → page view → app download → account creation → first session
- **Privacy compliance**: Cookie consent banner on shared pages, GDPR/CCPA compliant opt-in

### System Integration

**iRacing IRSDK (Primary Integration):**
- Shared memory API for real-time telemetry (200-300+ variables at 60Hz)
- Auto-detect iRacing running, begin capture without user action
- Graceful handling of disconnects/reconnects mid-session
- Session state tracking: practice, qualifying, race, warmup
- Version resilience: IRSDK changes between seasons — parser must be versioned

**System Tray Integration:**
- Background operation while racing (minimal resource footprint)
- Notification when debrief is ready
- VR-friendly: no alt-tab required
- Start-with-Windows option for always-on capture

**Local File System:**
- SQLite for session metadata, AI analysis results
- Parquet for raw telemetry time-series
- User-configurable storage location
- No cloud dependency for core functionality

### Update Strategy

| Aspect | Approach |
|---|---|
| Distribution | Direct download from website + GitHub releases |
| Auto-update | Tauri's built-in updater (checks on launch, user-confirmed) |
| Update frequency | Bi-weekly active dev, monthly at stability |
| Breaking changes | SQLite schema auto-migration; Parquet is append-only |
| iRacing season updates | Telemetry parser versioned separately, hotfix-capable |
| Rollback | Previous version available; SQLite migrations reversible |

### Offline Capabilities

| Feature | BYOK *(MVP)* | Local LLM *(Phase 2)* | Managed *(Phase 2)* | No Provider |
|---|---|---|---|---|
| Telemetry capture | Offline | Offline | Offline | Offline |
| Local pre-processing | Offline | Offline | Offline | Offline |
| AI analysis | Internet | Offline | Internet | Stats only |
| Debrief viewing | Offline | Offline | Offline | Offline |
| Conversational follow-up | Internet | Offline | Internet | N/A |
| Web sharing *(Phase 2)* | Internet | Internet | Internet | Internet |
| Session history/trends | Offline | Offline | Offline | Offline |
| Voice coaching *(Phase 2)* | Internet | Offline (OS TTS) | Internet | N/A |

**MVP graceful degradation:** If BYOK provider is unavailable, the app falls back to raw pre-processed stats (corner summaries, lap comparisons, derived metrics). Capture and data review always work regardless of provider status.

**Phase 2 graceful degradation:** Fall back in order: Managed → BYOK → Local → raw pre-processed stats.

### Architecture Decisions

**Provider Architecture:**
- Internal: Rust traits (`TextGeneration`, `VoiceGeneration`) with provider adapters
- MVP adapter: Claude API (BYOK). Single adapter. Simple.
- Phase 2 adapters: Ollama (Local tier), OpenAI, Gemini, ElevenLabs, OS-native TTS — added by demand
- MVP UX: paste API key → validate → ready
- The trait architecture preserves all future provider options without architectural debt

**Desktop Shell:**
- Tauri 2.0 — low footprint, Rust-native, sufficient for thin shell around web view
- Fallback: Rust daemon + localhost web server if Tauri hits blocking limitation
- Test matrix: Windows 10/11 on common sim racing hardware profiles

**Data Storage:**
- SQLite for session metadata, AI analysis results, user configuration (relational)
- Parquet for telemetry time-series from day one (columnar, compressed, analytical-ready)
- Web dashboard reads both via sql.js + arrow-js (WASM)
- Rust storage traits abstract format — swappable if needed

### Prompt & IP Strategy

Prompts are not treated as proprietary IP. Local LLM and BYOK users can see all prompts — accepted and intentional. The moat is the telemetry pipeline, the unified product experience, and curated knowledge bases.

| Asset | Protection Level | Rationale |
|---|---|---|
| Prompts | Open (Local/BYOK visible, Managed server-side) | Transparency builds trust; instructions without structured data are low-value |
| Telemetry pipeline | Proprietary (Rust core engine) | Corner segmentation, anomaly detection, summary generation — real IP |
| Car/track knowledge bases (future) | API-served, not shipped locally | Curated coaching data is competitive intelligence built over time |
| Product experience | Proprietary | Integrated loop is hard to replicate regardless of prompt visibility |

### Implementation Considerations

**Resource Constraints (Sim Racing Rigs):**
- Performance targets defined in NFR1 (CPU/RAM limits with measurement methodology)
- Tauri footprint: 5-10MB installed vs Electron's 150MB+
- No GPU usage by our app during capture — all processing is post-session

**AI Context Management:**
- The Rust pre-processor must aggressively summarize telemetry into structured per-corner statistics before sending to the AI provider. Raw telemetry is never sent to the AI. A 20-minute session (72,000+ samples) reduces to compact structured JSON suitable for a single API call. This is critical for BYOK cost control — users should not burn API credits on verbose payloads.

**Data Volume:**
- 60Hz × 15 channels × 60 min = ~3.2M data points per session
- Parquet compression: ~5-10MB per hour of racing
- SQLite metadata: <1MB per session

**iRacing-Specific:**
- `.ibt` binary telemetry import for offline analysis of past sessions
- Shared memory provides more channels than `.ibt` — live capture preferred
- iRacing subscription ($13/month) means moderate price sensitivity

## Data Privacy & User Data Handling

### Data Classification

| Data Category | Storage | Leaves Device? | Sensitivity |
|---|---|---|---|
| Raw telemetry (brake, throttle, speed, etc.) | Local Parquet files | Never sent raw — pre-processed summaries sent to AI provider | Low (driving data, not personal) |
| Session metadata (track, car, date, lap times) | Local SQLite | Included in AI analysis payloads as context | Low |
| AI analysis results (debriefs, coaching text) | Local SQLite | Generated by AI provider, stored locally | Low |
| API keys (Claude, OpenAI) | OS credential store | Sent with every AI request (standard auth) | High |
| Conversation history (follow-up chat) | Local SQLite + sent per request | Sent to AI provider for context continuity | Medium |
| User preferences and configuration | Local SQLite | Never | Low |

### MVP Data Flow (BYOK)

When the user's session ends, the Rust pre-processor compresses telemetry into structured per-corner statistics. This summary — not raw telemetry — is sent to the user's configured AI provider (Claude or OpenAI) via their own API key. The AI provider's data retention and privacy policies apply to this data. Users should understand that their driving performance summaries are processed by the third-party AI provider they configure.

### Privacy Principles

- **Local-first:** All data stored on the user's machine. No cloud storage for core features.
- **Minimal transmission:** Only pre-processed summaries sent to AI providers, never raw telemetry streams.
- **Transparency:** Users can inspect exactly what data is sent to AI providers (structured JSON visible in logs if enabled).
- **No telemetry home:** The application does not phone home, collect usage analytics, or transmit data to AI Race Team servers at MVP. Usage analytics are a Phase 2 consideration tied to managed tier and web sharing.
- **User controls:** Users can delete any session and its associated AI analysis. Export produces complete local copies.

### Phase 2 Privacy Considerations

When web sharing and managed tier are introduced:
- Shared debrief links require explicit user action — no automatic sharing
- Cookie consent and GDPR/CCPA compliance for shared debrief pages (see FR requirements in Shared Debrief Links section)
- Managed tier data handling policies to be defined before launch
- Ad pixel integration on shared pages requires clear opt-in disclosure

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Philosophy: Problem-Solving MVP**

The fastest path to validated learning: *Can AI Race Team replace Joe's manual Claude Desktop workflow and be genuinely better?* The product succeeds when auto-capture → AI analysis → conversational follow-up is faster, more visual, and more insightful than the manual prototype — without any manual steps.

**The litmus test:** Joe finishes a session, and AI Race Team produces a better debrief than pasting telemetry into Claude Desktop — faster, with per-corner specificity, with session memory.

**Core insight from scoping analysis:** The product is only as good as what the AI says. The Rust pipeline, the Tauri shell, the Parquet storage — those are infrastructure. The *product* is the coaching output. Prompt engineering depth and AI coaching quality are the highest-leverage investments.

### MVP Feature Set (Phase 1)

**Supported Journeys:** Journey 1 (Joe — happy path), Journey 2 (Marcus — onboarding, partially), Journey 3 (Bad session — core telemetry resilience)

**Must-Have Capabilities:**

| # | Capability | Rationale |
|---|---|---|
| 1 | Auto-detect iRacing + auto-capture telemetry | Core loop starts here — zero manual effort |
| 2 | Parquet storage for telemetry, SQLite for metadata | Foundation for everything; day-one format decision |
| 3 | Rust pre-processing (lap detection, per-lap summaries, corner segmentation) | Reduces AI token usage, enables offline stats, feeds per-corner analysis |
| 4 | AI session analysis (post-session debrief) with implicit training recommendations | Core value proposition — every debrief ends with "work on X next" |
| 5 | BYOK support (Claude API key) | Power user path, proven with prototype. MVP-only provider tier |
| 6 | Conversational follow-up about the session | "Why was Lap 60 faster?" — the aha moment |
| 7 | Per-corner AI analysis (text-based) | Core differentiator over prototype — tells you *where* you're losing time, not just *that* you are |
| 8 | 1D lap-distance chart (brake/throttle traces vs lap distance with corner zone labels) | Visual coaching without track-map complexity — simple line chart, high value |
| 9 | Session-over-session comparison | Enables persistent training plans — AI remembers and tracks improvement |
| 10 | System tray background operation | VR-friendly, no alt-tab required |
| 11 | Telemetry gap handling + partial lap preservation | Validated as critical by prototype — partial data is diagnostic |
| 12 | Car-specific prompt templates: MX-5 Cup + GT3 at launch, Formula Vee + LMP2 fast-follow | Coaching quality is the product — wrong advice for the wrong car kills trust |
| 13 | Tauri desktop shell (Windows) | Deployment vehicle |
| 14 | Sharing-ready architecture | Debrief output structured for easy web rendering later — no web infra built yet |

**MVP Provider Model:** BYOK-only. User provides their own Claude or OpenAI API key. Simple "paste your key" setup — no three-card onboarding, no local LLM, no managed routing. The Rust trait architecture (`TextGeneration` trait with provider adapters) preserves all future provider options without architectural debt.

**MVP Sharing Model:** Screenshots. The AI coaching output is designed to be screenshot-friendly for Discord/forum sharing. Debrief data is structured in a sharing-ready format so web viewer is a straightforward addition in Phase 2.

### Post-MVP Features

**Phase 2 — Growth: "The Visual Coaching Platform"**

| # | Capability | Dependency |
|---|---|---|
| 15 | Track map overlay with AI annotations (2D visual) | Corner segmentation from MVP, track geometry data |
| 16 | Three-card provider onboarding UX (text + voice) | Provider architecture from MVP |
| 17 | Local LLM support (Ollama adapter) | Rust TextGeneration trait from MVP |
| 18 | Managed tier with subscription pricing | API routing endpoint, billing integration |
| 19 | Web-shareable debrief links | Sharing-ready architecture from MVP |
| 20 | Shared debrief analytics + ad pixel integration | Web sharing infrastructure |
| 21 | AI confidence indicators (heuristic) | Session history from MVP |
| 22 | Persistent adaptive training plans | Session-over-session comparison from MVP |
| 23 | Stint-level analysis | Lap detection from MVP, stint boundary logic |
| 24 | Spin/incident detection | Anomaly detection algorithm, telemetry gap handling from MVP |
| 25 | Voice coaching (all tiers: OS native, BYOK, Managed) | VoiceGeneration trait, provider adapters |
| 26 | Additional car class templates (expanding beyond launch 4) | Prompt engineering, community feedback |
| 27 | Privacy compliance (GDPR/CCPA) for shared links | Web sharing infrastructure |

**Phase 3 — Expansion: "The AI Race Team"**

| # | Capability | Dependency |
|---|---|---|
| 28 | ACC / LMU simulator support | New telemetry parsers |
| 29 | Multi-driver comparison overlay | Multi-user architecture |
| 30 | Team manager features (Sarah journey) | Multi-user + team model |
| 31 | League admin features (Dave journey) | Multi-car telemetry, incident reconstruction |
| 32 | Race mode (live AI crew chief) | Real-time processing pipeline |
| 33 | AI race strategist (pit strategy, fuel, weather) | Race-state modeling |
| 34 | AI spotter (proximity alerts, blue flags) | Real-time proximity detection |
| 35 | Generative training plans (AI-created practice sessions) | Session history + weakness identification + persistent training |
| 36 | Predictive lap time modeling | Historical data corpus, conditions modeling |

### Training Plan Progression

Training plans are core to the product — this is what the prototype already does implicitly. The progression across phases:

| Phase | Training Plan Capability |
|---|---|
| **MVP** | Implicit — every debrief ends with "work on X next session" based on current analysis |
| **Growth** | Persistent — AI tracks whether you improved on recommended areas, adjusts focus, sequences your development arc |
| **Expansion** | Generative — AI creates targeted practice sessions ("Load Lime Rock, run 10 laps focusing only on T5 entry, target brake point: 85%") |

### Risk Mitigation Strategy

**Technical Risks:**

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Tauri blocking limitation | Low | High | Fallback: Rust daemon + localhost server (ADR-2) |
| iRacing IRSDK complexity | Low | Medium | Prototype already proves 30Hz capture; focus on resilience |
| Corner segmentation accuracy | Medium | Medium | Text-based per-corner in MVP — visual overlay deferred until algorithm is proven |
| AI coaching quality with non-MX-5 cars | High | High | 2 templates at launch (MX-5 + GT3), validate before expanding |

**Market Risks:**

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| "Same as Aiden" perception | Medium | Medium | Lead with zero-config, visual output, session memory |
| Generic AI advice causes churn | High | High | Car-specific templates, grounded responses, quality over features |
| No viral sharing without web links | Medium | Medium | Screenshots serve MVP; sharing-ready architecture unblocks Phase 2 |

**Resource Risks:**

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Single developer (Joe) scope too large | Medium | High | 14 MVP items is aggressive but tractable; no team features in MVP |
| Prompt engineering underinvested | High | High | Explicit MVP priority — coaching quality is the product |
| Per-corner text analysis insufficient without visuals | Low | Medium | 1D distance chart provides 60% of visual value at 5% of track-map effort |

### Scoping Decision Log

Key decisions made during scoping and the reasoning behind them:

| Decision | Rationale | Method |
|---|---|---|
| Per-corner analysis promoted to MVP (text-based) | Core differentiator over prototype; without it the product is "prototype in Rust" | Comparative Analysis Matrix |
| Session-over-session comparison promoted to MVP | Enables persistent training plans; drives retention | Comparative Analysis Matrix + Time Traveler Council |
| Web-shareable debrief demoted to Phase 2 | No audience at MVP; screenshots serve organic sharing; web sharing requires full web product | Comparative Analysis Matrix + 5 Whys |
| Three-card onboarding demoted to Phase 2 | Simple API key input sufficient for dogfooding; polish UX later | Comparative Analysis Matrix |
| Local LLM tier deferred to Phase 2 | No beta users used it; trait architecture preserves option; no cost to waiting | Time Traveler Council + 5 Whys |
| Managed tier demoted to Phase 2 | BYOK covers MVP users; routing layer is a config change via trait architecture | 5 Whys |
| 1D lap-distance chart added to MVP | 60% of track-map visual value at 5% of the effort; emerged from 5 Whys on visual overlay | 5 Whys |
| Multi-car templates: 2 launch + 2 fast-follow | Wrong advice for wrong car kills trust; MX-5 + GT3 covers ~60% of users; prompt work not code work | Time Traveler Council + 5 Whys |
| Training plans split across all phases | Implicit coaching is already MVP; persistent tracking is Growth; generative sessions are Expansion | User feedback (Joe's correction) |
| Prompt engineering depth as explicit MVP priority | "The product is only as good as what the AI says" — coaching quality is highest-leverage investment | Time Traveler Council (Future-Joe) |

## Functional Requirements

This section defines the capability contract for AI Race Team. UX designers design what's listed here. Architects support what's listed here. Epics implement what's listed here. If a capability is not listed, it does not exist in the product.

Each FR states WHAT capability exists, not HOW it's implemented. Each is testable, implementation-agnostic, and independent. FRs are tagged with their target phase (`[MVP]`, `[Growth]`, `[Expansion]`) and the user journeys they serve (`(J1, J2, ...)`). FR34 (old) was merged into FR17; FR42 (old) was merged into FR1b; FR1 was split into FR1/FR1a/FR1b.

### Telemetry Capture & Data Management

- **FR1:** System can automatically detect when iRacing is running and begin telemetry capture without user action `[MVP]` `(J1, J2, J3)`
- **FR1a:** System can detect session end conditions (session state change, extended off-track timeout, iRacing process exit) and trigger debrief generation `[MVP]` `(J1, J2, J3)`
- **FR1b:** User can manually trigger debrief analysis at any point during or after a session, bypassing automatic session-end detection `[MVP]` `(J1, J3)` *[Implementation: System tray menu option for power users; no prominent UI button to preserve automatic-first UX philosophy]*
- **FR2:** System can record driver input telemetry, vehicle dynamics telemetry, tire and brake condition telemetry, environmental conditions, and session context data at configurable sample rates `[MVP]` `(J1, J2, J3)`
- **FR3:** System can detect lap boundaries and compute per-lap summary statistics during capture `[MVP]` `(J1, J2, J3)`
- **FR4:** System can preserve telemetry data from incomplete laps (spins, resets, disconnects) as valid diagnostic data `[MVP]` `(J3)`
- **FR5:** System can detect and gracefully handle telemetry stream interruptions (disconnects, gaps) with gap markers `[MVP]` `(J3)`
- **FR6:** System can store telemetry and session data locally on the user's machine, with time-series data in a format suited for analytical queries and metadata in a format suited for structured queries `[MVP]` `(J1, J2, J3)`
- **FR7:** User can export any session data in open, non-proprietary formats without restrictions or lock-in `[MVP]` `(J1)`
- **FR8:** System can segment telemetry data into corner zones based on lap distance and braking/turning patterns `[MVP]` `(J1, J3)`
- **FR9:** System can compute derived metrics (brake application count, trail braking phases, tire degradation curves) from raw telemetry `[MVP]` `(J1, J3)`
- **FR10:** System can import historical telemetry from archived session files for offline analysis of past sessions `[MVP]` `(J1)`

### AI Coaching & Analysis

- **FR11:** System can generate a structured post-session debrief within 60 seconds of session end `[MVP]` `(J1, J2, J3)`
- **FR12:** System can provide per-corner analysis identifying specific performance gaps with telemetry evidence `[MVP]` `(J1, J3)`
- **FR13:** System can apply car-class-specific coaching knowledge, with coverage expanding over time based on user demand `[MVP]` `(J1, J2, J3)`
- **FR14:** System can ground every coaching insight in specific telemetry data points (lap numbers, pressure values, speeds, distances) `[MVP]` `(J1, J2, J3)`
- **FR15:** User can ask conversational follow-up questions about their session with the AI retaining full telemetry context `[MVP]` `(J1, J2)`
- **FR16:** System can diagnose why an incomplete lap occurred (throttle oversteer, brake lock, off-track) based on telemetry patterns `[MVP]` `(J3)`
- **FR17:** System can compare the current session to previous sessions, identify improvement trends or regressions, and present specific metrics that improved or regressed per corner `[MVP]` `(J1, J2)`
- **FR18:** System can generate implicit training recommendations ("work on X next session") at the end of every debrief `[MVP]` `(J1, J2)`
- **FR19:** System can compare a user's best lap to their average lap within a session, identifying specific differences per corner `[MVP]` `(J1, J3)`

### AI Provider Management

- **FR20:** User can configure their own API credentials for supported third-party AI providers `[MVP]` `(J1, J2)`
- **FR21:** System can validate API key connectivity on setup and before each session `[MVP]` `(J1, J2)`
- **FR22:** System can dispatch AI requests to the user's configured provider `[MVP]` `(J1, J2)`
- **FR23:** System can continue telemetry capture and local pre-processing when no AI provider is available `[MVP]` `(J1, J2, J3)`
- **FR24:** System can display provider connection status (ready, degraded, offline) persistently `[MVP]` `(J1, J2)`

### Visualization & Debrief

- **FR25:** System can visualize telemetry traces (driver inputs, vehicle response) across lap distance with corner zone context `[MVP]` `(J1, J3)`
- **FR26:** User can overlay multiple laps (best vs average, current vs previous session) on the same visualization `[MVP]` `(J1, J3)`
- **FR27:** System can display per-lap summary statistics in a structured debrief view `[MVP]` `(J1, J2, J3)`
- **FR28:** System can display session-level summary statistics (total laps, best/worst/average lap times, consistency metrics) `[MVP]` `(J1, J2, J3)`
- **FR29:** User can select a specific corner or lap to see detailed AI analysis for that segment `[MVP]` `(J1, J3)`
- **FR30:** System can structure debrief output as serializable data suitable for rendering across multiple presentation targets `[MVP]` `(J1)`

### Session History & Progress Tracking

- **FR31:** System can store and retrieve all past session debriefs and telemetry data locally `[MVP]` `(J1, J2)`
- **FR32:** User can browse session history filtered by track, car, and date `[MVP]` `(J1, J2)`
- **FR33:** System can compute progress metrics across sessions (lap time trends, consistency improvement, technique changes) `[MVP]` `(J1, J2)`

### Desktop Application & System Integration

- **FR34:** System can run as a background process accessible via system tray icon `[MVP]` `(J1, J2, J3)`
- **FR35:** System can deliver visual and audio notifications for capture state changes (started, failed) and debrief readiness, supporting VR users who cannot see the system tray `[MVP]` `(J1, J2)`
- **FR36:** User can access the debrief interface — including session summary, per-lap statistics, AI coaching analysis, telemetry visualization, and conversational follow-up — from the system tray notification or icon `[MVP]` `(J1, J2)`
- **FR37:** System can start automatically with Windows (user-configurable) `[MVP]` `(J1)`
- **FR38:** System can check for and apply application updates with user confirmation `[MVP]` `(J1, J2)`
- **FR39:** User can configure storage location for telemetry and session data `[MVP]` `(J1)`
- **FR40:** System can detect iRacing session type (practice, qualifying, race, warmup) `[MVP]` `(J1, J2, J3)`
- **FR41:** User can configure audio notification preferences (on/off, volume) for capture and debrief events `[MVP]` `(J2)`

## Non-Functional Requirements

NFRs define HOW WELL the system performs, not WHAT it does. Only categories relevant to this product are included. Each NFR includes a specific measurement method and is classified as Instrumented (measured in production every session with alerts), Tested (verified in CI/test suite), or Validated (assessed during beta testing with real users).

### Performance

| ID | Requirement | Target | Measurement | Type |
|---|---|---|---|---|
| NFR1 | Telemetry capture resource consumption while iRacing is running | <2% total system CPU (5-second rolling average on a 6-core reference rig: i5-12400/Ryzen 5 5600), <200MB RSS. Parquet flush on background thread. | Built-in resource monitor logging peak CPU per 1-second window and 5-second rolling average, RSS every 10s. CI benchmark on reference hardware profile. Alert on rolling average breach. | Instrumented |
| NFR2 | Local pre-processing time per lap | <10ms per lap | Timer from pre-processing start to complete, logged per session. Test with both 15-lap practice and 120-lap endurance sessions. Catches O(n²) regressions early. | Instrumented |
| NFR3 | Session history browsing and filter responsiveness | <1 second initial load for 500+ sessions; <200ms filter update | Instrumented query time on initial load and on each filter change. Automated test with 500 synthetic sessions. | Instrumented |
| NFR4 | Telemetry visualization rendering | <500ms single-lap (60Hz × 15 channels × 120s max lap), <1s multi-lap overlay (up to 10 laps). Chart interactions (hover, click, zoom) <100ms. | Instrument render time from data-fetch to paint-complete. Interaction latency measured per event. Automated test with max-size session (120 laps × 60Hz × 15 channels). | Instrumented |
| NFR5 | Application cold start to system tray ready | <5 seconds | CI test: process launch to tray-ready event timer | Tested |
| NFR5a | Debrief window open from tray click | <1 second | Instrument time from tray-click event to window-rendered. This is the common-case "launch" — cold start is rare. | Instrumented |
| NFR6 | AI response progressive disclosure | Progressive latency ladder: <2s no indicator; 2-5s loading animation; 5-15s "AI is thinking..." with elapsed timer; 15-30s "View debrief without AI, analysis will appear when complete"; 30s+ "Provider timeout — retry?" | Timer from query submit to first token. UX behavior verified per latency tier in integration test with simulated delays. | Instrumented |
| NFR6a | Progressive debrief display | Local pre-processed stats visible within 2 seconds of session end. AI analysis streams in as available. User never waits for AI to see their data. | Timer from session-end to local-stats-displayed. Verify AI content streams into existing debrief view without page reload. | Instrumented |

### Reliability

| ID | Requirement | Target | Measurement | Type |
|---|---|---|---|---|
| NFR7 | Telemetry data completeness per session | 99%+ samples captured. Any gap >500ms explicitly flagged with timestamp, duration, and lap position in debrief. | Compare expected vs actual samples per session. Gap detection logs every gap >500ms with location context. Alert if completeness below threshold. | Instrumented |
| NFR8 | Telemetry stream interruption detection | <1 second to detect and mark | Timestamp delta between last good sample and gap-marker creation, logged automatically | Instrumented |
| NFR9 | Data protection on application crash | Zero corruption of both metadata storage and telemetry time-series storage | Chaos test: kill process mid-write at various points for BOTH storage formats. Specifically test Parquet mid-write (requires atomic write pattern — write to temp, rename). Periodic CI run. | Tested |
| NFR10 | Graceful degradation on AI provider failure | Capture, pre-processing, and existing debriefs remain fully functional | Integration test: disable provider, run full capture session, verify all non-AI functions work | Tested |
| NFR11 | Recovery from unexpected iRacing shutdown | All telemetry data up to last captured sample preserved, including partial in-progress laps | Kill iRacing process mid-session at various lap-progress points (25%, 50%, 80%). Verify all completed laps AND partial current lap data intact. | Tested |

### Data Integrity

| ID | Requirement | Target | Measurement | Type |
|---|---|---|---|---|
| NFR12 | Telemetry sample timestamp accuracy | Within 1ms of actual capture time | Compare iRacing session clock vs system clock drift in integration tests with known-timestamp data | Tested |
| NFR13 | Lap boundary detection accuracy | 100% for all lap types including pit entry/exit, formation laps, and race restarts | Compare detected boundaries vs iRacing's own lap counter every session. Edge case test suite covering pit stops, penalties, formation laps, and restart scenarios. Zero tolerance — any mismatch is a bug. | Instrumented |
| NFR14 | Derived metric reproducibility | Identical output for identical input | Deterministic CI test: process same telemetry file twice, binary diff output | Tested |
| NFR15 | Stored data corruption detection | All session data checksummed with early detection | Checksum generation on write, validation on read. Background integrity check on application startup for sessions modified since last check. Unit test + startup verification. | Tested |
| NFR16 | Export determinism | Byte-identical output for same session regardless of when exported | CI test: export same session twice at different times, binary diff | Tested |

### Security

| ID | Requirement | Target | Measurement | Type |
|---|---|---|---|---|
| NFR17 | API key storage security | Keys stored using OS-provided credential storage (Windows Credential Manager, macOS Keychain) | Unit test: write key, verify stored in OS credential store not in app files. Verify no plaintext key exists anywhere on disk. Security review checklist. | Tested |
| NFR18 | API key exclusion from logs/reports | Zero appearances in any output | CI test: grep all log output for known test API key patterns during full integration run | Tested |
| NFR19 | Private data exclusion from shared output | Allowlist approach — only explicitly approved data fields included in shared output | Generate debrief export, verify against approved field allowlist. All telemetry metadata fields classified as shareable or private. New fields default to private. | Tested |
| NFR20 | Signed application updates | Reject unsigned or tampered updates | Test: serve unsigned update → verify rejection. Test: serve tampered update → verify rejection. Release checklist. | Tested |

### Integration

| ID | Requirement | Target | Measurement | Type |
|---|---|---|---|---|
| NFR21 | iRacing IRSDK version resilience | Handle version changes without app update. Detect semantic changes to existing variables. | Maintain test .ibt files from multiple seasons, parse all on every CI build. For critical channels (brake, throttle, speed, RPM), validate data ranges against expected bounds and flag anomalies suggesting semantic changes. | Tested |
| NFR22 | AI provider timeout and retry handling | No infinite hangs, graceful failure after retries | Integration test with simulated provider timeouts, verify retry behavior and eventual graceful failure | Tested |
| NFR23 | iRacing process detection speed | <5 seconds from iRacing start | Timer from iRacing process start to detection event in integration test | Tested |
| NFR24 | Historical .ibt file backward compatibility | Support current + previous iRacing seasons | Same test corpus as NFR21 — parse files from each supported season on every CI build | Tested |

### Usability

| ID | Requirement | Target | Measurement | Type |
|---|---|---|---|---|
| NFR25 | First-time setup completion time | <3 minutes (install → configured → ready) | Stopwatch test with 3+ new users during beta. Proxy: count screens/clicks in onboarding flow. | Validated |
| NFR26 | Zero-touch telemetry recording | No user action between iRacing launch and capture start | Automated test: launch iRacing, verify capture started with zero input events from test harness | Tested |
| NFR27 | VR-friendly async debrief workflow | Full flow completable without VR headset interaction | Test: race → notification → debrief view using only monitor mouse clicks. No keyboard, no alt-tab, no headset. | Validated |
| NFR28 | Coaching output quality | Understandable with no telemetry experience AND must not contradict car-class-specific physics | Beta survey: >90% "understood the advice." Flesch-Kincaid grade 8-10. Car-class physics contradiction test: validate output against known car characteristics per FR13. | Validated |

**Categories intentionally excluded:**
- **Scalability** — Single-user desktop app at MVP. No server-side scaling concerns. Revisit when web sharing and managed tier are added in Phase 2.
- **Accessibility (WCAG 2.1 AA - Basic Compliance)** — MVP includes Basic WCAG 2.1 AA compliance: semantic HTML structure, full keyboard navigation support, and contrast ratio compliance (4.5:1 for normal text, 3:1 for large text). Screen reader optimization and ARIA label enhancements are deferred to post-MVP. This baseline ensures accessibility without blocking MVP delivery while reducing future rework costs. Implementation: embedded in all UI stories, no separate accessibility stories required.
