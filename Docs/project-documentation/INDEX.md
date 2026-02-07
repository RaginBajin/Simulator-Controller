# Simulator Controller - Project Documentation Index

**Generated:** 2026-01-26
**Version:** 6.8.1.0-dev
**Scan Level:** Exhaustive

---

## Project Summary

| Attribute | Value |
|-----------|-------|
| **Project Name** | Simulator Controller |
| **Type** | Desktop Application |
| **Platform** | Windows |
| **Primary Language** | AutoHotkey v2.x |
| **Lines of Code** | ~250,000 |
| **Repository Type** | Monolith |
| **License** | CC BY-NC-SA |

---

## Quick Links

### Generated Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | System architecture, component diagrams, technology overview |
| [SOURCE_TREE.md](./SOURCE_TREE.md) | Complete source tree analysis with file counts |
| [TECHNOLOGY_STACK.md](./TECHNOLOGY_STACK.md) | Languages, frameworks, libraries, protocols |

### Existing Documentation (in /Docs)

| Document | Description |
|----------|-------------|
| [AI_ASSISTANTS.md](/Docs/AI_ASSISTANTS.md) | AI assistant architecture, rule engine, voice system |
| [DATA_MODELS.md](/Docs/DATA_MODELS.md) | Database schemas, data structures |
| [PLUGIN_SYSTEM.md](/Docs/PLUGIN_SYSTEM.md) | Plugin architecture, lifecycle, types |
| [TELEMETRY_PROVIDERS.md](/Docs/TELEMETRY_PROVIDERS.md) | Telemetry acquisition, simulator integrations |
| [TEAM_SERVER_API.md](/Docs/TEAM_SERVER_API.md) | REST API documentation |
| [REFACTORING_PLAN.md](/Docs/REFACTORING_PLAN.md) | Future modernization roadmap |

---

## Key Statistics

### Code Distribution

```
AutoHotkey   ████████████████████████████████  86% (215,692 LOC)
C#           █████░░░░░░░░░░░░░░░░░░░░░░░░░░░  10% (~25,000 LOC)
C++          ███░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   4% (~10,000 LOC)
```

### File Counts

| Type | Count |
|------|-------|
| AutoHotkey (.ahk) | 201 |
| C# Projects (.csproj) | 35 |
| C++ Projects (.vcxproj) | 15 |
| Rule Files (.rules) | 118+ |
| INI Config Files | 468 |
| Documentation (.md) | 75+ |

### Supported Simulators

1. Assetto Corsa Competizione (ACC)
2. Assetto Corsa (AC)
3. Assetto Corsa EVO (ACE)
4. iRacing (IRC)
5. rFactor 2 (RF2)
6. Le Mans Ultimate (LMU)
7. Automobilista 2 (AMS2)
8. RaceRoom Racing Experience (R3E)
9. Project CARS 2 (PCARS2)
10. Project Motor Racing (PMR)
11. Rennsport (RSP)

### Supported Languages (Voice)

1. English (en)
2. German (de)
3. Spanish (es)
4. French (fr)
5. Italian (it)
6. Portuguese (pt)
7. Japanese (ja)
8. Chinese (zh)

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                    USER INTERFACE LAYER                       │
│  Button Box • Stream Deck • Dashboard • Voice Interface       │
└──────────────────────────────────────────────────────────────┘
                              │
┌──────────────────────────────────────────────────────────────┐
│                    APPLICATION LAYER                          │
│  Controller Core • Plugin System • AI Assistants • Rule Engine│
└──────────────────────────────────────────────────────────────┘
                              │
┌──────────────────────────────────────────────────────────────┐
│                    SERVICE LAYER                              │
│  Voice Service • Telemetry Service • Team Server • LLM Runtime│
└──────────────────────────────────────────────────────────────┘
                              │
┌──────────────────────────────────────────────────────────────┐
│                    DATA LAYER                                 │
│  SQLite • CSV Sessions • INI Config • Shared Memory           │
└──────────────────────────────────────────────────────────────┘
```

---

## Key Components

### AI Assistants

| Name | Role | Technology |
|------|------|------------|
| Jona | Race Engineer | Rule Engine |
| Cato | Race Strategist | Rule Engine |
| Elisa | Race Spotter | Rule Engine |
| Aiden | Driving Coach | LLM-based |

### Plugin System

| Category | Count | Examples |
|----------|-------|----------|
| Simulator | 11 | ACC, iRacing, RF2 |
| Assistant | 4 | Engineer, Strategist |
| Hardware | 4 | Button Box, Stream Deck |
| Feedback | 2 | Tactile, Motion |
| Config | 10+ | Various wizards |

### Telemetry Providers

| Protocol | Simulators |
|----------|------------|
| Shared Memory | ACC, AC, IRC, RF2, AMS2, R3E |
| UDP | ACC Broadcast, PMR |

---

## Development Environment

### Required Tools

- AutoHotkey v2.x
- Visual Studio 2019+ (for C#/C++)
- .NET 8.0 SDK
- Windows 10/11

### Build Process

1. Compile C#/C++ projects in Visual Studio
2. Run `Simulator Tools.ahk` to copy binaries
3. Compile AHK scripts with Ahk2Exe

---

## Future Roadmap

Based on [REFACTORING_PLAN.md](/Docs/REFACTORING_PLAN.md):

### Phase 0: Quick Wins (Weeks 1-4)
- Security hotfixes (password hashing)
- Logging infrastructure
- Basic onboarding improvements

### Phase 1: Architecture Modernization (Months 1-4)
- New tech stack (Electron + React + .NET 8)
- Project restructuring
- gRPC for service communication

### Phase 2-6: Full Modernization (Months 2-12)
- UX overhaul
- Data layer consolidation
- AI assistants modernization
- Testing & deployment

---

## Contact & Community

- **GitHub:** [SeriousOldMan/Simulator-Controller](https://github.com/SeriousOldMan/Simulator-Controller)
- **Discord:** [Community Server](https://discord.gg/5N8JrNr48H)
- **Wiki:** [Documentation](https://github.com/SeriousOldMan/Simulator-Controller/wiki)
- **Patreon:** [Support](https://www.patreon.com/simulatorcontroller)

---

## Workflow Status

| Step | Status | Description |
|------|--------|-------------|
| 1 | Completed | Project classification |
| 2 | Completed | Documentation discovery |
| 3 | Completed | Technology stack analysis |
| 4 | Completed | Conditional analysis |
| 5 | Completed | Source tree analysis |
| 6 | Completed | Architecture documentation |
| 7 | Completed | Master index generation |
