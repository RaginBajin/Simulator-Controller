# Simulator Controller - Architecture Documentation

## Executive Summary

Simulator Controller is a comprehensive Windows desktop application for sim racing that provides:
- Modular plugin-based controller automation
- AI-powered race assistants (Engineer, Strategist, Spotter, Coach)
- Voice recognition and synthesis in 8 languages
- Telemetry acquisition from 11 racing simulators
- Team Server for multiplayer coordination

**Version:** 6.8.1.0-dev
**Primary Language:** AutoHotkey v2.x (~215,692 lines)
**Platform:** Windows (requires .NET 8.0 runtime)

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        SIMULATOR CONTROLLER ARCHITECTURE                          │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│                              PRESENTATION LAYER                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │   Button     │  │   Stream     │  │  Dashboard   │  │    Voice     │        │
│  │   Box GUI    │  │   Deck UI    │  │    UIs       │  │  Interface   │        │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘        │
│  [AHK Native]      [C# Plugin]       [AHK Native]       [AHK+C# Bridge]         │
└───────────────────────────────────────┬─────────────────────────────────────────┘
                                        │
┌───────────────────────────────────────┼─────────────────────────────────────────┐
│                              APPLICATION LAYER                                    │
│  ┌────────────────────────────────────┴────────────────────────────────────┐    │
│  │                        SIMULATOR CONTROLLER CORE                         │    │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐        │    │
│  │  │   Plugin   │  │   Mode     │  │  Function  │  │   Event    │        │    │
│  │  │  Manager   │  │  Manager   │  │  Registry  │  │  Dispatch  │        │    │
│  │  └────────────┘  └────────────┘  └────────────┘  └────────────┘        │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                        │                                         │
│  ┌─────────────────────────────────────┼─────────────────────────────────────┐  │
│  │                              AI ASSISTANTS                                 │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐         │  │
│  │  │   Jona     │  │   Cato     │  │   Elisa    │  │   Aiden    │         │  │
│  │  │  Engineer  │  │ Strategist │  │  Spotter   │  │   Coach    │         │  │
│  │  └──────┬─────┘  └──────┬─────┘  └──────┬─────┘  └──────┬─────┘         │  │
│  │         │               │               │               │                │  │
│  │         └───────────────┴───────────────┴───────────────┘                │  │
│  │                              │                                            │  │
│  │                 ┌────────────┴────────────┐                              │  │
│  │                 │    RETE Rule Engine     │                              │  │
│  │                 │  (Forward/Backward)     │                              │  │
│  │                 └─────────────────────────┘                              │  │
│  └───────────────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────┬─────────────────────────────────────────┘
                                        │
┌───────────────────────────────────────┼─────────────────────────────────────────┐
│                              SERVICE LAYER                                        │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐                 │
│  │   Voice    │  │  Telemetry │  │    Team    │  │    LLM     │                 │
│  │  Service   │  │  Service   │  │   Server   │  │  Runtime   │                 │
│  │  [C#/AHK]  │  │  [C#/C++]  │  │  [.NET 8]  │  │  [.NET 8]  │                 │
│  └────────────┘  └────────────┘  └────────────┘  └────────────┘                 │
└───────────────────────────────────────┬─────────────────────────────────────────┘
                                        │
┌───────────────────────────────────────┼─────────────────────────────────────────┐
│                              DATA LAYER                                           │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐                 │
│  │   SQLite   │  │   CSV      │  │    INI     │  │  Shared    │                 │
│  │  Database  │  │  Sessions  │  │   Config   │  │  Memory    │                 │
│  │[Team Server]│  │  [Local]   │  │  [Local]   │  │[Simulators]│                 │
│  └────────────┘  └────────────┘  └────────────┘  └────────────┘                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Directory Structure

```
Simulator-Controller/
├── Sources/                    # Source code (AHK, C#, C++)
│   ├── Assistants/            # AI Race Assistants
│   │   ├── Libraries/         # Assistant base classes
│   │   ├── Rules/             # RETE rule files
│   │   └── Actions/           # Event-triggered rule files
│   ├── Configuration/         # Setup wizards and editors
│   │   └── Libraries/         # Configuration components
│   ├── Controller/            # Core controller application
│   ├── Database/              # Session/telemetry database
│   │   └── Libraries/         # Database access classes
│   ├── Framework/             # Core framework
│   │   └── Extensions/        # Framework extensions (LLM, Speech, etc.)
│   ├── Garage/                # Setup Workbench
│   ├── Plugins/               # All plugin implementations (40+)
│   ├── Special/               # C#/C++ native components
│   │   ├── */                 # Telemetry providers per simulator
│   │   ├── Team Server/       # ASP.NET Core Team Server
│   │   └── LLM Runtime/       # Local LLM inference
│   └── Tools/                 # Development utilities
├── Resources/                  # Runtime resources
│   ├── Actions/               # Event action rules
│   ├── Button Box Images/     # Button box layouts
│   ├── Database/              # Default data
│   ├── Garage/                # Setup rules per car
│   ├── Grammars/              # Voice recognition grammars (8 langs)
│   ├── Icons/                 # Application icons
│   ├── Rules/                 # Core AI rule files
│   ├── Scripts/               # Lua scripts
│   ├── Setup/                 # Installation resources
│   ├── Simulator Data/        # Per-simulator data
│   ├── Stream Deck Images/    # Stream Deck icons
│   └── Translations/          # UI translations (8 langs)
├── Config/                     # Configuration templates
├── Docs/                       # Existing documentation (75 files)
├── Profiles/                   # Startup profiles
└── Utilities/                  # Third-party tools
```

---

## Technology Stack

### Primary Languages

| Language | Version | Lines of Code | Purpose |
|----------|---------|---------------|---------|
| AutoHotkey | v2.x | ~215,692 | Core application, UI, plugins, assistants |
| C# | .NET 8.0 | ~35 projects | Team Server, LLM, Speech, telemetry |
| C++ | C++17 | ~15 projects | Native shared memory connectors |

### Frameworks & Libraries

| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| Team Server | ASP.NET Core | 8.0 | REST API for multiplayer |
| Database ORM | sqlite-net-pcl | 1.9.172 | SQLite access |
| LLM Inference | LLamaSharp | 0.25.0 | Local AI model hosting |
| Speech Recognition | MS Speech / Google | - | Voice input |
| Speech Synthesis | MS Speech / Google | - | Voice output |

### LLM Backends (Driving Coach)

- CPU (LLamaSharp.Backend.Cpu)
- CUDA 11 (LLamaSharp.Backend.Cuda11)
- CUDA 12 (LLamaSharp.Backend.Cuda12)
- Vulkan (LLamaSharp.Backend.Vulkan)

External LLM providers: OpenAI, Mistral, Ollama, GPT4All

---

## Plugin Architecture

### Plugin Categories

| Category | Count | Examples |
|----------|-------|----------|
| Simulator Plugins | 11 | ACC, AC, iRacing, rFactor 2, LMU |
| Assistant Plugins | 4 | Engineer, Strategist, Spotter, Coach |
| Hardware Plugins | 4 | Button Box, Stream Deck, Pedal Calibration |
| Feedback Plugins | 2 | Tactile Feedback, Motion Feedback |
| Configuration Plugins | 10+ | Various setup wizards |

### Plugin Lifecycle

```
┌─────────────────────────────────────────────────────────────────┐
│                      PLUGIN LIFECYCLE                            │
└─────────────────────────────────────────────────────────────────┘

  Registration           Activation              Runtime
  ┌──────────┐          ┌──────────┐          ┌──────────┐
  │  Plugin  │   ───►   │  Plugin  │   ───►   │  Plugin  │
  │ Declared │          │ Enabled  │          │ Running  │
  └──────────┘          └──────────┘          └──────────┘
       │                     │                     │
       │                     │                     │
  ┌────┴────┐           ┌────┴────┐           ┌────┴────┐
  │ Modes   │           │ Actions │           │ Events  │
  │Registered│           │Connected│           │Processed│
  └─────────┘           └─────────┘           └─────────┘
```

### Plugin Base Classes

- `Plugin` - Base class for all plugins
- `ControllerPlugin` - Plugins that interact with hardware
- `SimulatorPlugin` - Plugins for simulation games
- `RaceAssistantPlugin` - Base for AI assistants

---

## AI Assistants Architecture

### Assistant Hierarchy

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI ASSISTANT HIERARCHY                        │
└─────────────────────────────────────────────────────────────────┘

                    ┌─────────────────┐
                    │  RaceAssistant  │ (Base class)
                    │    [AHK]        │
                    └────────┬────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
┌────────┴────────┐ ┌────────┴────────┐ ┌───────┴────────┐
│  RaceEngineer   │ │ RaceStrategist  │ │  RaceSpotter   │
│    (Jona)       │ │    (Cato)       │ │   (Elisa)      │
└────────┬────────┘ └────────┬────────┘ └───────┬────────┘
         │                   │                   │
         └───────────────────┼───────────────────┘
                             │
                    ┌────────┴────────┐
                    │  DrivingCoach   │
                    │    (Aiden)      │
                    │   [LLM-based]   │
                    └─────────────────┘
```

### RETE Rule Engine

The AI assistants use a custom RETE-based rule engine:

- **Forward chaining**: Facts trigger rules
- **Backward chaining**: Goals drive inference
- **Knowledge Base**: Maintains session facts
- **Rule Files**: 18 core rule files + 100+ car-specific

Key Rule Categories:
- Lap Information Retrieval
- Pitstop Computations
- Weather Notifications
- Standings Computations
- Conversation Actions

---

## Telemetry System

### Supported Simulators

| Simulator | Provider Type | Connector Type |
|-----------|---------------|----------------|
| ACC | Shared Memory | C++ DLL |
| AC | Shared Memory | C# DLL |
| iRacing | Shared Memory | C++ DLL |
| rFactor 2 | Shared Memory | C# DLL |
| Le Mans Ultimate | Shared Memory | C# DLL |
| Automobilista 2 | Shared Memory | C++ DLL |
| RaceRoom R3E | Shared Memory | C++ DLL |
| Project CARS 2 | Shared Memory | C++ DLL |
| Project Motor Racing | UDP | C# |
| Assetto Corsa EVO | Shared Memory | C++ DLL |

### Data Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Simulator  │ ──► │  Provider   │ ──► │  Connector  │
│   (Game)    │     │ (.exe C#/C++)│     │ (.dll C#/C++)│
└─────────────┘     └─────────────┘     └─────────────┘
                                              │
                         ┌────────────────────┘
                         ▼
                    ┌─────────────┐
                    │ Controller  │
                    │   Plugin    │
                    │   (AHK)     │
                    └─────────────┘
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
       ┌─────────┐ ┌─────────┐ ┌─────────┐
       │ Engineer│ │Strategist│ │ Spotter │
       └─────────┘ └─────────┘ └─────────┘
```

---

## Team Server

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         TEAM SERVER                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                     REST API Controllers                    │ │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐             │ │
│  │  │  Account   │ │  Session   │ │    Data    │             │ │
│  │  │ Controller │ │ Controller │ │ Controller │             │ │
│  │  └────────────┘ └────────────┘ └────────────┘             │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              │                                   │
│  ┌───────────────────────────┴────────────────────────────────┐ │
│  │                     Service Managers                        │ │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐             │ │
│  │  │   Token    │ │   Team     │ │  Session   │             │ │
│  │  │  Manager   │ │  Manager   │ │  Manager   │             │ │
│  │  └────────────┘ └────────────┘ └────────────┘             │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              │                                   │
│  ┌───────────────────────────┴────────────────────────────────┐ │
│  │                       SQLite ORM                            │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Token Hierarchy

1. **Internal Token** - System operations
2. **Account Token** - User authentication
3. **Session Token** - Team/session management
4. **Data Token** - Telemetry data access

---

## Voice System

### Supported Languages

| Language | Code | Recognition | Synthesis |
|----------|------|-------------|-----------|
| English | en | Yes | Yes |
| German | de | Yes | Yes |
| Spanish | es | Yes | Yes |
| French | fr | Yes | Yes |
| Italian | it | Yes | Yes |
| Portuguese | pt | Yes | Yes |
| Japanese | ja | Yes | Yes |
| Chinese | zh | Yes | Yes |

### Voice Providers

- **Microsoft Speech** - Windows native
- **Google Cloud Speech** - Cloud-based
- **Whisper** - Local transcription server

---

## Configuration System

### File Types

| Type | Count | Purpose |
|------|-------|---------|
| INI Files | 468 | Application configuration |
| CSV Files | - | Session data, telemetry |
| Rules Files | 118+ | AI decision logic |
| Grammar Files | 64+ | Voice command patterns |
| Translation Files | 64+ | UI localization |

### Configuration Hierarchy

1. **Default Config** - `Config/` directory
2. **User Config** - `Documents/Simulator Controller/`
3. **Profile Config** - `Profiles/` directory

---

## Build System

### Build Targets

The `Simulator Tools.targets` file defines:

- **Update procedures** - Version migration scripts
- **Cleanup targets** - Temporary file removal
- **Copy operations** - Binary distribution

### Build Components

| Component | Source | Output |
|-----------|--------|--------|
| AHK Scripts | Sources/*.ahk | Binaries/*.exe |
| C# Projects | Sources/Special/*.csproj | Release/*.dll |
| C++ Projects | Sources/Special/*.vcxproj | x64/Release/*.dll |

---

## Security Considerations

### Known Issues (from REFACTORING_PLAN.md)

1. **Plaintext passwords** in Team Server (needs Argon2id hashing)
2. **No rate limiting** on login endpoints
3. **Missing HTTPS enforcement**

### Authentication Flow

Token-based authentication with:
- 5-minute idle timeout
- Account expiration checks
- Token type validation

---

## Future Architecture (REFACTORING_PLAN.md)

The project has a comprehensive refactoring plan to modernize:

### Proposed Stack

| Layer | Current | Proposed |
|-------|---------|----------|
| Frontend | AHK Native | Electron + React |
| Backend | Mixed | .NET 8 Services |
| Telemetry | C#/C++ | Rust |
| Database | CSV/INI/SQLite | Unified SQLite |
| IPC | File-based | gRPC |

### Timeline

- **Phase 0**: Security hotfixes (Weeks 1-4)
- **Phase 1**: Architecture modernization (Months 1-4)
- **Phase 2**: UX overhaul (Months 2-5)
- **Phase 3**: Data layer consolidation (Months 3-6)
- **Phase 4**: AI assistants modernization (Months 4-8)
- **Phase 5**: Testing & Quality (Ongoing)
- **Phase 6**: Deployment & Distribution (Months 8-10)

---

## References

- [README.md](/README.md) - Project overview
- [REFACTORING_PLAN.md](/Docs/REFACTORING_PLAN.md) - Modernization plan
- [AI_ASSISTANTS.md](/Docs/AI_ASSISTANTS.md) - Assistant architecture
- [PLUGIN_SYSTEM.md](/Docs/PLUGIN_SYSTEM.md) - Plugin details
- [TEAM_SERVER_API.md](/Docs/TEAM_SERVER_API.md) - API documentation
- [TELEMETRY_PROVIDERS.md](/Docs/TELEMETRY_PROVIDERS.md) - Telemetry details
- [DATA_MODELS.md](/Docs/DATA_MODELS.md) - Database schemas
