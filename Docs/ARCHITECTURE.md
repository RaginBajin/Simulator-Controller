# Simulator Controller - System Architecture

## Overview

**Simulator Controller** is a comprehensive, modular administration and control application for sim racing. It provides AI-powered race assistants, unified hardware control, multi-simulator support, and advanced telemetry analysis tools.

## Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **UI Framework** | AutoHotkey v2 GUI | Cross-application UI components |
| **Main Language** | AutoHotkey v2+ | Application logic, plugins, automation |
| **Backend** | ASP.NET Core 8.0 | Team Server REST API |
| **Native Code** | C++/C# | Low-level telemetry, shared memory readers |
| **Database** | SQLite | Session data, telemetry, settings storage |
| **AI Engine** | RETE Rules + Lua | Assistant decision making |
| **Voice** | Microsoft/Google/OpenAI APIs | Speech recognition and synthesis |
| **Web** | WebView2 | HTML-based UI components |
| **Scripting** | Lua 5.5 | Extended assistant/setup customization |

## High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          USER INTERFACE LAYER                                │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │  Simulator   │  │  Assistant   │  │    Setup     │  │  Database &  │    │
│  │  Controller  │  │    Apps      │  │  Workbench   │  │  Telemetry   │    │
│  │  (Main App)  │  │ (Jona, Cato  │  │   (Garage)   │  │    Tools     │    │
│  │              │  │ Elisa, Aiden)│  │              │  │              │    │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘    │
└─────────┼─────────────────┼─────────────────┼─────────────────┼────────────┘
          │                 │                 │                 │
          └─────────────────┴────────┬────────┴─────────────────┘
                                     │
┌────────────────────────────────────┴────────────────────────────────────────┐
│                           PLUGIN SYSTEM                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │    Core     │  │  Simulator  │  │  Assistant  │  │  Hardware   │        │
│  │   Plugins   │  │   Plugins   │  │   Plugins   │  │   Plugins   │        │
│  │ (System,    │  │ (ACC, AC,   │  │ (Engineer,  │  │ (ButtonBox, │        │
│  │  Core)      │  │ IRC, RF2..) │  │ Strategist.)│  │ StreamDeck) │        │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘        │
└─────────┼────────────────┼────────────────┼────────────────┼───────────────┘
          │                │                │                │
          └────────────────┴───────┬────────┴────────────────┘
                                   │
┌──────────────────────────────────┴──────────────────────────────────────────┐
│                         FRAMEWORK LAYER                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ Application │  │    GUI      │  │   Message   │  │ Collections │        │
│  │  Framework  │  │  Framework  │  │   System    │  │  & Types    │        │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │    Rule     │  │   Speech    │  │  Database   │  │   HTTP &    │        │
│  │   Engine    │  │   I/O       │  │  Extension  │  │   Network   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────────────────────┘
                                   │
┌──────────────────────────────────┴──────────────────────────────────────────┐
│                    SIMULATION & HARDWARE INTEGRATION                         │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    Telemetry Providers                               │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐            │    │
│  │  │ ACC SHM  │  │ IRC SHM  │  │ RF2 SHM  │  │ UDP      │            │    │
│  │  │ Provider │  │ Provider │  │ Provider │  │ Provider │            │    │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘            │    │
│  └───────┼─────────────┼─────────────┼─────────────┼───────────────────┘    │
│          │             │             │             │                        │
│  ┌───────┴─────────────┴─────────────┴─────────────┴───────────────────┐    │
│  │                 Shared Memory / UDP Connectors (C++/C#)              │    │
│  └──────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
                                   │
┌──────────────────────────────────┴──────────────────────────────────────────┐
│                         BACKEND SERVICES                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────┐  ┌─────────────────────────┐                   │
│  │    Team Server          │  │    Voice Processing      │                   │
│  │    (ASP.NET Core)       │  │                          │                   │
│  │  ┌───────────────────┐  │  │  ┌──────────────────┐   │                   │
│  │  │ REST Controllers  │  │  │  │ Speech Recognizer│   │                   │
│  │  │ - Account         │  │  │  │ (MS/Google)      │   │                   │
│  │  │ - Session         │  │  │  └──────────────────┘   │                   │
│  │  │ - Team/Driver     │  │  │  ┌──────────────────┐   │                   │
│  │  │ - Data            │  │  │  │ Speech Synth     │   │                   │
│  │  └───────────────────┘  │  │  │ (TTS)            │   │                   │
│  │  ┌───────────────────┐  │  │  └──────────────────┘   │                   │
│  │  │ SQLite Database   │  │  │  ┌──────────────────┐   │                   │
│  │  └───────────────────┘  │  │  │ LLM Runtime      │   │                   │
│  └─────────────────────────┘  │  │ (Whisper/Ollama) │   │                   │
│                               │  └──────────────────┘   │                   │
│                               └─────────────────────────┘                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                   │
┌──────────────────────────────────┴──────────────────────────────────────────┐
│                      AI ASSISTANT CORE                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    RETE-Based Rule Engine                            │    │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐         │    │
│  │  │ Race Engineer  │  │ Race Strategist│  │ Race Spotter   │         │    │
│  │  │    Rules       │  │    Rules       │  │    Rules       │         │    │
│  │  └────────────────┘  └────────────────┘  └────────────────┘         │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    Voice Recognition & Commands                      │    │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐         │    │
│  │  │ Grammar-based  │  │ Conversation   │  │ Custom Voice   │         │    │
│  │  │ Patterns (8+   │  │ Flows          │  │ Commands       │         │    │
│  │  │ languages)     │  │                │  │                │         │    │
│  │  └────────────────┘  └────────────────┘  └────────────────┘         │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
                                   │
┌──────────────────────────────────┴──────────────────────────────────────────┐
│                      DATA & CONFIGURATION LAYER                              │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐       │
│  │   Configuration   │  │    Data Stores    │  │    Resources      │       │
│  │   System          │  │                   │  │                   │       │
│  │  ┌─────────────┐  │  │  ┌─────────────┐  │  │  ┌─────────────┐  │       │
│  │  │ INI-based   │  │  │  │ Session DB  │  │  │  │ Car/Track   │  │       │
│  │  │ Config      │  │  │  │ (Laps,      │  │  │  │ Definitions │  │       │
│  │  │             │  │  │  │ Telemetry)  │  │  │  │             │  │       │
│  │  └─────────────┘  │  │  └─────────────┘  │  │  └─────────────┘  │       │
│  │  ┌─────────────┐  │  │  ┌─────────────┐  │  │  ┌─────────────┐  │       │
│  │  │ Setup       │  │  │  │ Tyres DB    │  │  │  │ Rule Files  │  │       │
│  │  │ Wizard      │  │  │  │ (Pressures) │  │  │  │             │  │       │
│  │  └─────────────┘  │  │  └─────────────┘  │  │  └─────────────┘  │       │
│  │  ┌─────────────┐  │  │  ┌─────────────┐  │  │  ┌─────────────┐  │       │
│  │  │ Settings    │  │  │  │ Settings DB │  │  │  │ Translations│  │       │
│  │  │ Editor      │  │  │  │ (Strategy)  │  │  │  │ (8+ langs)  │  │       │
│  │  └─────────────┘  │  │  └─────────────┘  │  │  └─────────────┘  │       │
│  └───────────────────┘  └───────────────────┘  └───────────────────┘       │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Supported Racing Simulators

| Simulator | Abbreviation | Connection Method |
|-----------|--------------|-------------------|
| Assetto Corsa Competizione | ACC | Shared Memory + UDP |
| Assetto Corsa | AC | Shared Memory |
| Assetto Corsa EVO | ACE | Shared Memory |
| iRacing | IRC | Shared Memory + IBT Files |
| rFactor 2 | RF2 | Shared Memory (8 buffers) |
| Le Mans Ultimate | LMU | Shared Memory |
| Automobilista 2 | AMS2 | Shared Memory |
| RaceRoom Racing Experience | R3E | Shared Memory |
| Project CARS 2 | PCARS2 | UDP Multicast |
| Project Motor Racing | PMR | UDP |
| Rennsport | RSP | Shared Memory |

## Key Applications

### Primary Applications

| Application | Purpose |
|-------------|---------|
| **Simulator Controller** | Main control center - manages plugins, hardware, launches |
| **Simulator Tools** | Configuration, build tool, setup wizard |
| **Simulator Startup** | Pre-start initialization and environment setup |
| **System Monitor** | Real-time health monitoring and debugging |
| **Voice Server** | Central voice processing hub |

### AI Assistant Applications

| Application | Persona | Purpose |
|-------------|---------|---------|
| **Race Engineer** | Jona | Pitstop planning, damage analysis, fuel management |
| **Race Strategist** | Cato | Race strategy, weather forecasting, position tracking |
| **Race Spotter** | Elisa | Traffic awareness, gap monitoring, blue flags |
| **Driving Coach** | Aiden | LLM-powered performance coaching |

### Workbench Applications

| Application | Purpose |
|-------------|---------|
| **Setup Workbench** | AI-driven car setup recommendations |
| **Strategy Workbench** | Fuel/tyre strategy calculation |
| **Team Center** | Multiplayer team race coordination |
| **Solo Center** | Practice session management |
| **Session Database** | Telemetry data browser and analysis |
| **Race Reports** | Performance analysis and reporting |

## Directory Structure

```
/Sources/
├── Controller/                 # Main application core
│   ├── Simulator Controller.ahk
│   ├── Process Manager.ahk
│   ├── Simulator Startup.ahk
│   ├── System Monitor.ahk
│   └── Voice Server.ahk
│
├── Framework/                  # Core library/foundation
│   ├── Application.ahk
│   ├── GUI.ahk
│   ├── Framework.ahk
│   ├── Collections.ahk
│   ├── Types.ahk
│   └── Extensions/
│       ├── RuleEngine.ahk
│       ├── SpeechRecognizer.ahk
│       ├── SpeechSynthesizer.ahk
│       ├── Database.ahk
│       └── HTTP.ahk
│
├── Plugins/                    # Plugin ecosystem (40+)
│   ├── Core Plugin.ahk
│   ├── System Plugin.ahk
│   ├── Button Box Plugin.ahk
│   ├── Stream Deck Plugin.ahk
│   ├── *Simulator*Plugin.ahk   # ACC, AC, IRC, RF2, etc.
│   ├── Race Engineer Plugin.ahk
│   ├── Race Strategist Plugin.ahk
│   ├── Race Spotter Plugin.ahk
│   └── Libraries/
│       ├── SimulatorPlugin.ahk
│       ├── SimulatorProvider.ahk
│       └── RaceAssistantPlugin.ahk
│
├── Assistants/                 # AI assistant applications
│   ├── Driving Coach.ahk
│   ├── Race Engineer.ahk
│   ├── Race Strategist.ahk
│   ├── Race Spotter.ahk
│   ├── Libraries/
│   ├── Rules/                  # RETE rule definitions
│   ├── Grammars/               # Voice recognition patterns
│   └── Instructions/           # LLM instruction templates
│
├── Database/                   # Database management
│   ├── Session Database.ahk
│   └── Libraries/
│       ├── SessionDatabase.ahk
│       ├── TelemetryCollector.ahk
│       ├── TelemetryAnalyzer.ahk
│       ├── TyresDatabase.ahk
│       └── LapsDatabase.ahk
│
├── Garage/                     # Setup & telemetry tools
│   ├── Setup Workbench.ahk
│   ├── Libraries/
│   ├── Rules/
│   └── Definitions/
│
├── Configuration/              # Setup & configuration tools
│   └── Simulator Tools.ahk
│
├── Special/                    # Backend services & native modules
│   ├── Team Server/            # .NET ASP Core server
│   ├── SimHub Plugin/          # .NET plugin
│   ├── Stream Deck Plugin/     # .NET plugin
│   ├── *SHM Coach/Spotter/     # Simulator-specific (C#/C++)
│   ├── *SHM Connector/         # Telemetry connectors
│   ├── Speech Recognizers/
│   ├── Speech Synthesizers/
│   ├── LLM Runtime/
│   └── Whisper Server/
│
└── Tests/                      # Testing framework
    ├── AHKUnit/
    └── Test Scripts/
```

## Code Statistics

| Language | Files | Lines | Purpose |
|----------|-------|-------|---------|
| **AutoHotkey** | 201 | ~107,000 | Main application, plugins, UI |
| **C#** | 140 | ~30,000 | Backend services, telemetry providers |
| **C++** | 68 | ~15,000 | Shared memory connectors |
| **Rules** | 22 | ~4,300 | AI decision logic |
| **Grammars** | 70+ | ~3,000 | Voice recognition |
| **Total** | ~500 | ~160,000+ | - |
