# Simulator Controller - Technical Documentation

## Overview

**Simulator Controller** is a comprehensive, modular administration and control application for sim racing. This documentation provides detailed technical information about the system architecture, components, and how they interact.

## Documentation Index

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | High-level system architecture, technology stack, directory structure |
| [DATA_MODELS.md](DATA_MODELS.md) | Database schemas, data relationships, entity diagrams |
| [PLUGIN_SYSTEM.md](PLUGIN_SYSTEM.md) | Plugin architecture, lifecycle, types, and development |
| [AI_ASSISTANTS.md](AI_ASSISTANTS.md) | AI assistants (Jona, Cato, Elisa, Aiden), rule engine, voice system |
| [TELEMETRY_PROVIDERS.md](TELEMETRY_PROVIDERS.md) | Simulator connections, shared memory, UDP providers |
| [TEAM_SERVER_API.md](TEAM_SERVER_API.md) | REST API documentation, authentication, endpoints |

## Quick Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           SIMULATOR CONTROLLER                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                      User Interface Layer                            │    │
│  │  • Simulator Controller (Main App)                                   │    │
│  │  • AI Assistants (Jona, Cato, Elisa, Aiden)                         │    │
│  │  • Setup Workbench, Strategy Workbench                               │    │
│  │  • Session Database, Team Center, Solo Center                        │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                     │                                        │
│  ┌─────────────────────────────────┴───────────────────────────────────┐    │
│  │                         Plugin System                                │    │
│  │  • 40+ Plugins (Simulators, Assistants, Hardware)                   │    │
│  │  • Modular, extensible architecture                                  │    │
│  │  • Event-driven communication                                        │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                     │                                        │
│  ┌─────────────────────────────────┴───────────────────────────────────┐    │
│  │                      Framework Layer                                 │    │
│  │  • GUI Framework (Window, Themes, Controls)                         │    │
│  │  • Rule Engine (RETE-based AI decisions)                            │    │
│  │  • Voice System (Recognition, Synthesis)                            │    │
│  │  • Database Extensions, Network, Collections                        │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                     │                                        │
│  ┌─────────────────────────────────┴───────────────────────────────────┐    │
│  │                 Telemetry & Hardware Integration                     │    │
│  │  • Shared Memory Connectors (ACC, AC, RF2, IRC, etc.)               │    │
│  │  • UDP Providers (ACC Broadcasting, PMR)                            │    │
│  │  • Coach/Spotter Components                                          │    │
│  │  • Hardware Controllers (Button Box, Stream Deck)                   │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                     │                                        │
│  ┌─────────────────────────────────┴───────────────────────────────────┐    │
│  │                      Backend Services                                │    │
│  │  • Team Server (ASP.NET Core REST API)                              │    │
│  │  • Voice Server (Speech Recognition/Synthesis)                      │    │
│  │  • LLM Runtime (Driving Coach AI)                                   │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Supported Simulators

| Simulator | Abbreviation | Connection Method |
|-----------|--------------|-------------------|
| Assetto Corsa Competizione | ACC | Shared Memory + UDP |
| Assetto Corsa | AC | Shared Memory |
| Assetto Corsa EVO | ACE | Shared Memory |
| iRacing | IRC | Shared Memory + IBT |
| rFactor 2 | RF2 | Shared Memory (8 buffers) |
| Le Mans Ultimate | LMU | Shared Memory |
| Automobilista 2 | AMS2 | Shared Memory |
| RaceRoom Racing Experience | R3E | Shared Memory |
| Project CARS 2 | PCARS2 | UDP Multicast |
| Project Motor Racing | PMR | UDP |
| Rennsport | RSP | Shared Memory |

## AI Assistants

| Assistant | Persona | Responsibilities |
|-----------|---------|------------------|
| **Race Engineer** | Jona | Pitstop planning, fuel management, tire setup, damage analysis |
| **Race Strategist** | Cato | Race strategy, weather forecasting, position tracking |
| **Race Spotter** | Elisa | Traffic awareness, gap monitoring, opponent tracking |
| **Driving Coach** | Aiden | LLM-powered performance coaching, telemetry analysis |

## Technology Stack

| Layer | Technology |
|-------|-----------|
| Main Application | AutoHotkey v2 |
| Backend Server | ASP.NET Core 8.0 |
| Native Connectors | C++/C# |
| Database | SQLite |
| AI Engine | RETE Rules + Lua |
| Voice | Microsoft/Google/OpenAI APIs |
| LLM | Claude, OpenAI, Ollama |

## Code Statistics

| Language | Files | Lines |
|----------|-------|-------|
| AutoHotkey | 201 | ~107,000 |
| C# | 140 | ~30,000 |
| C++ | 68 | ~15,000 |
| Rules | 22 | ~4,300 |
| Grammars | 70+ | ~3,000 |
| **Total** | ~500 | ~160,000+ |

## Key Directory Structure

```
/Sources/
├── Controller/          # Main application core
├── Framework/           # Core libraries and extensions
├── Plugins/             # Plugin ecosystem (40+)
├── Assistants/          # AI assistant applications
├── Database/            # Session and telemetry database
├── Garage/              # Setup workbench tools
├── Configuration/       # Configuration tools
├── Special/             # Backend services (Team Server, etc.)
└── Tests/               # Test framework
```

## Getting Started with the Documentation

1. **Start with [ARCHITECTURE.md](ARCHITECTURE.md)** to understand the overall system design and component relationships.

2. **Read [PLUGIN_SYSTEM.md](PLUGIN_SYSTEM.md)** to understand how the modular plugin architecture works and how different components communicate.

3. **Explore [AI_ASSISTANTS.md](AI_ASSISTANTS.md)** to learn about the AI-powered race assistants, the rule engine, and voice interaction.

4. **Review [TELEMETRY_PROVIDERS.md](TELEMETRY_PROVIDERS.md)** to understand how the system connects to different racing simulators and acquires telemetry data.

5. **Check [DATA_MODELS.md](DATA_MODELS.md)** for detailed information about database schemas and data relationships.

6. **Reference [TEAM_SERVER_API.md](TEAM_SERVER_API.md)** for the REST API documentation used for multiplayer coordination and data sharing.

## Key Concepts

### Plugin Architecture
The system uses a modular plugin architecture where functionality is encapsulated in plugins that can be independently loaded, configured, and extended. Plugins communicate through events and messages.

### Rule Engine
AI assistants use a RETE-based rule engine for decision-making. Rules are defined in `.rules` files and operate on a knowledge base of facts about the current race state.

### Voice Interaction
Assistants support natural language voice interaction through grammar-based speech recognition (8+ languages supported) and text-to-speech synthesis.

### Telemetry Pipeline
Simulator data flows through connectors (C++/C#) → providers → main application → assistants. This pipeline normalizes data from different simulators into a common format.

### Team Server
A .NET Core REST API provides multiplayer coordination, session management, and data sharing capabilities with token-based authentication.

## Data Flow Summary

```
Racing Simulator
       │
       ▼
Shared Memory / UDP ──► Connector (DLL) ──► Provider (EXE)
       │
       ▼
Main Application (Simulator Controller)
       │
       ├──► Plugin System (Configure, Actions)
       │
       ├──► AI Assistants (Analysis, Recommendations)
       │         │
       │         └──► Rule Engine (Decisions)
       │         │
       │         └──► Voice System (Communication)
       │
       ├──► Session Database (Telemetry Storage)
       │
       └──► Team Server (Multiplayer Sync)
```

## Contributing

When working with this codebase:

1. Follow the existing plugin architecture for new features
2. Use the rule engine for AI decision logic
3. Support multi-language voice grammars
4. Maintain the normalized telemetry format
5. Use the database framework for persistence
6. Follow the established IPC patterns for inter-process communication

## License

See the main repository LICENSE file for licensing information.
