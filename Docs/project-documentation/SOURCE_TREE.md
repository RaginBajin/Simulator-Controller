# Simulator Controller - Source Tree Analysis

## Overview

This document provides an exhaustive analysis of the Simulator Controller source code structure, file counts, and organization.

---

## Top-Level Structure

```
Simulator-Controller/
├── Sources/          # Primary source code
├── Resources/        # Runtime assets and data
├── Config/           # Configuration templates
├── Docs/             # User documentation
├── Profiles/         # Startup profiles
├── Utilities/        # Third-party tools
├── LICENSE           # CC BY-NC-SA license
├── README.md         # Project overview
└── VERSION           # Version metadata
```

---

## Sources Directory

### Summary Statistics

| Directory | Files | Lines (est.) | Language |
|-----------|-------|--------------|----------|
| Sources/Assistants | 25 | ~45,000 | AHK |
| Sources/Configuration | 27 | ~35,000 | AHK |
| Sources/Controller | 6 | ~15,000 | AHK |
| Sources/Database | 12 | ~20,000 | AHK |
| Sources/Framework | 30 | ~40,000 | AHK |
| Sources/Garage | 5 | ~8,000 | AHK |
| Sources/Plugins | 40 | ~50,000 | AHK |
| Sources/Special | 90+ | ~30,000 | C#/C++ |
| Sources/Tools | 15 | ~5,000 | AHK |
| **Total** | **~250** | **~248,000** | Mixed |

---

### Sources/Assistants

AI Race Assistant implementations.

```
Assistants/
├── Libraries/
│   ├── DrivingCoach.ahk          # AI Driving Coach (Aiden)
│   ├── RaceAssistant.ahk         # Base class for all assistants
│   ├── RaceEngineer.ahk          # Race Engineer (Jona)
│   ├── RaceReportReader.ahk      # Report parsing
│   ├── RaceReportViewer.ahk      # Report display
│   ├── RaceSpotter.ahk           # Race Spotter (Elisa)
│   ├── RaceStrategist.ahk        # Race Strategist (Cato)
│   ├── Strategy.ahk              # Strategy calculations
│   ├── StrategyViewer.ahk        # Strategy display
│   └── VoiceManager.ahk          # Voice I/O management
├── Actions/                       # Event-triggered rules
│   ├── Position.position_gained.rules
│   ├── Position.position_lost.rules
│   ├── Race Engineer.cancel_pitstop.rules
│   ├── Race Engineer.recalculate_damage_impact.rules
│   ├── Race Spotter.*.rules
│   └── Weather.*.rules
├── Rules/                         # Core AI rules
│   ├── Car Information Retrieval.rules
│   ├── Conversation Actions.rules
│   ├── Driving Coach.rules
│   ├── Lap Information Retrieval.rules
│   ├── Pitstop Computations.rules
│   ├── Pitstop Information Retrieval.rules
│   ├── Race Engineer.rules
│   ├── Race Spotter.rules
│   ├── Race Strategist.rules
│   ├── Session Information Retrieval.rules
│   ├── Standings Computations.rules
│   ├── Statistical Computations.rules
│   ├── Tyre Information Retrieval.rules
│   ├── Utilities.rules
│   ├── Weather Information Retrieval.rules
│   └── Weather Notifications.rules
├── Driving Coach.ahk              # Coach entry point
├── Race Engineer.ahk              # Engineer entry point
├── Race Reports.ahk               # Reports application
├── Race Settings.ahk              # Settings application
├── Race Spotter.ahk               # Spotter entry point
├── Race Strategist.ahk            # Strategist entry point
├── Server Administration.ahk      # Team Server admin
├── Solo Center.ahk                # Solo practice tool
├── Strategy Workbench.ahk         # Strategy planning
└── Team Center.ahk                # Team race coordination
```

---

### Sources/Configuration

Setup wizards and configuration editors.

```
Configuration/
├── Libraries/
│   ├── ApplicationsStepWizard.ahk
│   ├── AssistantBoosterEditor.ahk
│   ├── AssistantsStepWizard.ahk
│   ├── BasicStepWizard.ahk
│   ├── ButtonBoxPreview.ahk
│   ├── ConfigurationEditor.ahk
│   ├── ConfigurationItemList.ahk
│   ├── ControllerActionsEditor.ahk
│   ├── ControllerEditor.ahk
│   ├── ControllerStepWizard.ahk
│   ├── FormatsEditor.ahk
│   ├── GeneralStepWizard.ahk
│   ├── InstallationStepWizard.ahk
│   ├── ModulesStepWizard.ahk
│   ├── MotionFeedbackStepWizard.ahk
│   ├── PedalCalibrationStepWizard.ahk
│   ├── SettingsEditor.ahk
│   ├── SimulatorsStepWizard.ahk
│   ├── SplashScreenEditor.ahk
│   ├── StreamDeckPreview.ahk
│   ├── SynthesizerEditor.ahk
│   ├── TactileFeedbackStepWizard.ahk
│   ├── TeamManagementPanel.ahk
│   ├── TranslationsEditor.ahk
│   └── TranslatorEditor.ahk
├── Simulator Configuration.ahk    # Low-level config editor
├── Simulator Download.ahk         # Update downloader
├── Simulator Settings.ahk         # Quick settings
├── Simulator Setup.ahk            # Setup wizard
└── Simulator Tools.ahk            # Build/maintenance tools
```

---

### Sources/Controller

Core controller application.

```
Controller/
├── Process Manager.ahk            # Process lifecycle
├── Simulator Controller.ahk       # Main entry point
├── Simulator Startup.ahk          # Startup sequence
├── System Monitor.ahk             # Health monitoring
└── Voice Server.ahk               # Voice I/O server
```

---

### Sources/Database

Session and telemetry database.

```
Database/
├── Libraries/
│   ├── LapsDatabase.ahk           # Lap data storage
│   ├── PressuresEditor.ahk        # Tire pressure editor
│   ├── SessionDatabase.ahk        # Session data storage
│   ├── SessionDatabaseBrowser.ahk # Session browser UI
│   ├── SettingsDatabase.ahk       # Settings storage
│   ├── TelemetryAnalyzer.ahk      # Telemetry analysis
│   ├── TelemetryCollector.ahk     # Data collection
│   ├── TelemetryViewer.ahk        # Telemetry display
│   └── TyresDatabase.ahk          # Tire data storage
├── Database Synchronizer.ahk      # Team data sync
├── Session Database.ahk           # Database main app
└── Track Mapper.ahk               # Track mapping tool
```

---

### Sources/Framework

Core framework libraries.

```
Framework/
├── Extensions/
│   ├── CLR.ahk                    # .NET CLR integration
│   ├── CodeEditor.ahk             # Code editing support
│   ├── Database.ahk               # Database utilities
│   ├── FTP.ahk                    # FTP client
│   ├── GDIP.ahk                   # GDI+ graphics
│   ├── GIFViewer.ahk              # Animated GIF support
│   ├── HTMLViewer.ahk             # HTML rendering
│   ├── HTTP.ahk                   # HTTP client
│   ├── JSON.ahk                   # JSON parsing
│   ├── LLMAgent.ahk               # LLM agent framework
│   ├── LLMBooster.ahk             # Conversation boosting
│   ├── LLMConnector.ahk           # LLM provider connections
│   ├── Math.ahk                   # Mathematical functions
│   ├── Messages.ahk               # IPC messaging
│   ├── RuleEngine.ahk             # RETE rule engine
│   ├── ScriptEngine.ahk           # Lua scripting
│   ├── SpeechRecognizer.ahk       # Voice input
│   ├── SpeechSynthesizer.ahk      # Voice output
│   ├── Task.ahk                   # Async task support
│   └── Translator.ahk             # Machine translation
├── Application.ahk                # Application base class
├── Audio.ahk                      # Audio utilities
├── Collections.ahk                # Data structures
├── Configuration.ahk              # Config management
├── Constants.ahk                  # Global constants
├── Debug.ahk                      # Debug utilities
├── Development.ahk                # Dev mode config
├── Files.ahk                      # File operations
├── Framework.ahk                  # Framework bootstrap
├── GUI.ahk                        # GUI framework
├── Localization.ahk               # I18n support
├── Logging.ahk                    # Logging framework
├── Notifications.ahk              # System notifications
├── Production.ahk                 # Production config
├── Process.ahk                    # Process management
├── Progress.ahk                   # Progress indicators
├── Strings.ahk                    # String utilities
├── Target.ahk                     # Build targets
└── Utils.ahk                      # General utilities
```

---

### Sources/Plugins

All plugin implementations (40 files).

```
Plugins/
├── Simulator Plugins (11)
│   ├── AC Plugin.ahk              # Assetto Corsa
│   ├── ACC Plugin.ahk             # Assetto Corsa Competizione
│   ├── ACE Plugin.ahk             # Assetto Corsa EVO
│   ├── AMS2 Plugin.ahk            # Automobilista 2
│   ├── IRC Plugin.ahk             # iRacing
│   ├── LMU Plugin.ahk             # Le Mans Ultimate
│   ├── PCARS2 Plugin.ahk          # Project CARS 2
│   ├── PMR Plugin.ahk             # Project Motor Racing
│   ├── R3E Plugin.ahk             # RaceRoom R3E
│   ├── RF2 Plugin.ahk             # rFactor 2
│   ├── RSP Plugin.ahk             # Rennsport
│   └── RST Plugin.ahk             # RST Racing
│
├── Assistant Plugins (4)
│   ├── Driving Coach Plugin.ahk
│   ├── Race Engineer Plugin.ahk
│   ├── Race Spotter Plugin.ahk
│   └── Race Strategist Plugin.ahk
│
├── Hardware Plugins (4)
│   ├── Button Box Plugin.ahk
│   ├── Stream Deck Plugin.ahk
│   ├── Pedal Calibration Plugin.ahk
│   └── Motion Feedback Plugin.ahk
│
├── Feedback Plugins (2)
│   ├── Tactile Feedback Plugin.ahk
│   └── Motion Feedback Plugin.ahk
│
├── Configuration Plugins (10+)
│   ├── Applications Configuration Plugin.ahk
│   ├── Chat Messages Configuration Plugin.ahk
│   ├── Controller Configuration Plugin.ahk
│   ├── Driving Coach Configuration Plugin.ahk
│   ├── Launchpad Configuration Plugin.ahk
│   ├── Plugins Configuration Plugin.ahk
│   ├── Race Engineer Configuration Plugin.ahk
│   ├── Race Spotter Configuration Plugin.ahk
│   ├── Race Strategist Configuration Plugin.ahk
│   ├── Team Server Configuration Plugin.ahk
│   └── Voice Control Configuration Plugin.ahk
│
├── System Plugins
│   ├── Core Plugin.ahk            # Core functionality
│   ├── System Plugin.ahk          # System management
│   ├── Integration Plugin.ahk     # External integrations
│   └── Team Server Plugin.ahk     # Team Server client
│
├── Voice Plugin
│   └── Voice Control Plugin.ahk   # Voice commands
│
└── Meta Plugins
    ├── Configuration Plugins.ahk  # Plugin aggregator
    ├── Controller Plugins.ahk     # Plugin aggregator
    └── Simulator Providers.ahk    # Provider aggregator
```

---

### Sources/Special

Native C#/C++ components (90+ files across 40+ projects).

```
Special/
├── Telemetry Providers/
│   ├── ACC SHM Provider/          # ACC shared memory (C++)
│   ├── AC SHM Provider/           # AC shared memory (C#)
│   ├── AMS2 SHM Provider/         # AMS2 shared memory (C++)
│   ├── IRC SHM Provider/          # iRacing shared memory (C++)
│   ├── R3E SHM Provider/          # R3E shared memory (C++)
│   ├── RF2 SHM Provider/          # rFactor 2 shared memory (C#)
│   ├── PMR UDP Provider/          # PMR UDP telemetry (C#)
│   └── ACC UDP Provider/          # ACC broadcasting (C#)
│
├── Telemetry Connectors/
│   ├── ACC SHM Connector/         # ACC connector DLL (C++)
│   ├── AC SHM Connector/          # AC connector DLL (C#)
│   ├── AMS2 SHM Connector/        # AMS2 connector DLL (C++)
│   ├── IRC SHM Connector/         # iRacing connector DLL (C++)
│   ├── R3E SHM Connector/         # R3E connector DLL (C++)
│   ├── RF2 SHM Connector/         # rFactor 2 connector DLL (C#)
│   └── PMR UDP Connector/         # PMR connector DLL (C#)
│
├── Spotter Providers/
│   ├── ACC SHM Spotter/           # ACC spotter data (C++)
│   ├── AC SHM Spotter/            # AC spotter data (C#)
│   ├── AMS2 SHM Spotter/          # AMS2 spotter data (C++)
│   ├── IRC SHM Spotter/           # iRacing spotter data (C++)
│   ├── R3E SHM Spotter/           # R3E spotter data (C++)
│   ├── RF2 SHM Spotter/           # rFactor 2 spotter data (C#)
│   └── PMR UDP Spotter/           # PMR spotter data (C#)
│
├── Coach Providers/
│   ├── ACC SHM Coach/             # ACC coach telemetry (C++)
│   ├── AC SHM Coach/              # AC coach telemetry (C#)
│   ├── AMS2 SHM Coach/            # AMS2 coach telemetry (C++)
│   ├── IRC SHM Coach/             # iRacing coach telemetry (C++)
│   ├── R3E SHM Coach/             # R3E coach telemetry (C++)
│   ├── RF2 SHM Coach/             # rFactor 2 coach telemetry (C#)
│   └── PMR UDP Coach/             # PMR coach telemetry (C#)
│
├── Speech Services/
│   ├── Microsoft Speech Recognizer/  # MS voice input (C#)
│   ├── Microsoft Speech Synthesizer/ # MS voice output (C#)
│   ├── Google Speech Recognizer/     # Google voice input (C#)
│   ├── Google Speech Synthesizer/    # Google voice output (C#)
│   └── Whisper Server/               # Whisper transcription (C#)
│
├── Infrastructure/
│   ├── Team Server/               # ASP.NET Core 8.0 API
│   ├── LLM Runtime/               # LLamaSharp inference
│   ├── Audio Capture/             # Audio recording (C#)
│   ├── WebView2/                  # Web rendering (C#)
│   └── Second Monitor Reader/     # Monitor capture (C#)
│
├── External Integrations/
│   ├── Stream Deck Plugin/        # Stream Deck connector
│   ├── SimHub Plugin/             # SimHub integration
│   └── iRacing IBT Reader/        # iRacing telemetry files
│
└── Solution Files (*.sln)
    └── Each project has its own solution file
```

---

## Resources Directory

```
Resources/
├── Actions/               # 10 event action rule files
├── Button Box Images/     # Button box layout images
├── Charts/                # Chart templates
├── Database/              # Default database data
├── Garage/
│   ├── Rules/            # Setup rules per simulator
│   │   └── Cars/         # 100+ car-specific rules
│   └── Setup Workbench.ini
├── Grammars/             # Voice grammars (8 languages)
│   ├── Choices.{lang}    # Choice patterns
│   ├── Conversation.{lang}
│   ├── Driving Coach.grammars.{lang}
│   ├── Race Engineer.grammars.{lang}
│   ├── Race Spotter.grammars.{lang}
│   └── Race Strategist.grammars.{lang}
├── Icons/                # Application icons
├── Instructions/         # Assistant instructions
├── Rules/                # 18 core rule files
├── Screen Images/        # UI images
├── Scripts/              # Lua scripts
├── Setup/                # Installation resources
│   ├── Installer/
│   └── Windows Runtimes/
├── Simulator Data/       # Per-simulator data files
├── Sounds/               # Audio files
├── Splash Media/         # Startup animations
├── Strategy/             # Strategy templates
├── Stream Deck Images/   # Stream Deck icons
├── Templates/            # Code templates
└── Translations/         # 64+ translation files
    ├── Languages.csv     # Language metadata
    ├── Settings.{lang}   # Settings translations
    └── *.{lang}          # Various UI translations
```

---

## File Type Statistics

| Extension | Count | Purpose |
|-----------|-------|---------|
| .ahk | 201 | AutoHotkey source |
| .rules | 118+ | Rule engine definitions |
| .grammars.* | 32 | Voice command grammars |
| .cs | ~150 | C# source |
| .cpp | ~35 | C++ source |
| .csproj | 35 | C# project files |
| .vcxproj | 15 | C++ project files |
| .sln | 40+ | Visual Studio solutions |
| .ini | 468 | Configuration files |
| .md | 75+ | Documentation |
| .csv | varies | Data files |
| .png/.jpg | varies | Images |

---

## Entry Points

| Application | Source File | Purpose |
|-------------|-------------|---------|
| Simulator Controller | Controller/Simulator Controller.ahk | Main app |
| Simulator Setup | Configuration/Simulator Setup.ahk | Setup wizard |
| Simulator Configuration | Configuration/Simulator Configuration.ahk | Low-level config |
| Simulator Tools | Configuration/Simulator Tools.ahk | Build tools |
| Race Engineer | Assistants/Race Engineer.ahk | Engineer assistant |
| Race Strategist | Assistants/Race Strategist.ahk | Strategist assistant |
| Race Spotter | Assistants/Race Spotter.ahk | Spotter assistant |
| Driving Coach | Assistants/Driving Coach.ahk | Coach assistant |
| Session Database | Database/Session Database.ahk | Database browser |
| Strategy Workbench | Assistants/Strategy Workbench.ahk | Strategy tool |
| Solo Center | Assistants/Solo Center.ahk | Practice tool |
| Team Center | Assistants/Team Center.ahk | Team coordination |
| Team Server | Special/Team Server/Team Server.csproj | REST API server |
