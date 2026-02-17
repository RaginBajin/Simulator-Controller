# Simulator Controller - Technology Stack

## Overview

Simulator Controller is a multi-language desktop application combining AutoHotkey for the primary UI/logic, C# for services, and C++ for performance-critical telemetry acquisition.

---

## Primary Languages

### AutoHotkey v2.x

**Purpose:** Core application, UI, plugins, AI assistants

**Statistics:**
- Files: 201
- Lines of code: ~215,692
- File extension: `.ahk`

**Key Features Used:**
- Object-oriented programming (classes, inheritance)
- GUI framework (native Windows controls)
- Hotkeys and hotstrings
- COM automation
- DLL calls (native and .NET)
- IPC (messages, pipes)

**Main Modules:**
- Framework (base classes, utilities)
- Controller (main application loop)
- Plugins (40+ modules)
- Assistants (AI logic)
- Configuration (setup wizards)
- Database (data management)

---

### C# (.NET 8.0)

**Purpose:** Backend services, speech processing, telemetry

**Project Count:** 35 C# projects

**Frameworks:**
- ASP.NET Core 8.0 (Team Server)
- .NET 8.0 Console Apps (providers)
- .NET Framework 4.7.2 (legacy speech)

**Key Libraries:**

| Package | Version | Purpose |
|---------|---------|---------|
| sqlite-net-pcl | 1.9.172 | SQLite ORM |
| SQLitePCLRaw.bundle_green | 2.1.11 | SQLite native |
| LLamaSharp | 0.25.0 | Local LLM inference |
| LLamaSharp.Backend.Cpu | 0.25.0 | CPU inference |
| LLamaSharp.Backend.Cuda11 | 0.24.0 | CUDA 11 GPU |
| LLamaSharp.Backend.Cuda12 | 0.25.0 | CUDA 12 GPU |
| LLamaSharp.Backend.Vulkan | 0.25.0 | Vulkan GPU |
| Microsoft.Speech | - | Windows speech |
| Google.Cloud.Speech.V1 | - | Google speech |
| NAudio | - | Audio processing |
| Newtonsoft.Json | - | JSON serialization |
| Grpc.Core | - | gRPC communication |

---

### C++ (C++17)

**Purpose:** Native shared memory telemetry connectors

**Project Count:** 15 Visual C++ projects

**Target Platform:** Windows x64

**Key Components:**
- Shared memory mapping
- Struct definitions (simulator-specific)
- YAML parsing (iRacing)
- High-frequency polling

**Build Output:**
- DLL connectors for AHK interop
- EXE providers for data acquisition

---

## Runtime Requirements

### Windows Prerequisites

| Component | Version | Purpose |
|-----------|---------|---------|
| Windows | 10/11 | Operating system |
| .NET Runtime | 8.0 | C# applications |
| .NET Framework | 4.7.2+ | Legacy components |
| Visual C++ Redistributable | 2019+ | C++ DLLs |

### Optional Components

| Component | Purpose |
|-----------|---------|
| CUDA 11/12 | GPU acceleration for LLM |
| Vulkan | Alternative GPU compute |
| Google Cloud credentials | Cloud speech services |
| SoX | Audio post-processing |
| NirCmd | Audio volume control |

---

## Data Storage

### SQLite (Team Server)

**Location:** Configurable (local/memory/path)

**ORM:** sqlite-net-pcl with LINQ

**Tables:**
- Accounts
- Tokens
- Teams
- Drivers
- Sessions
- Stints
- Laps
- Data objects (telemetry)

### CSV Files (Local)

**Purpose:** Session and telemetry data

**Locations:**
- `Documents/Simulator Controller/Database/`
- Per-simulator subdirectories
- Per-car/track data

**Format:** Standard CSV with headers

### INI Files (Configuration)

**Count:** 468 INI files

**Locations:**
- `Config/` - Default templates
- `Documents/Simulator Controller/` - User config
- `Resources/` - Application data

**Structure:**
```ini
[Section]
Key=Value
```

### Rule Files (.rules)

**Count:** 118+ rule files

**Purpose:** RETE rule engine definitions

**Location:** `Resources/Rules/` and `Sources/Assistants/Rules/`

### Grammar Files (.grammars.*)

**Count:** 32 grammar files (4 assistants x 8 languages)

**Purpose:** Voice command patterns

**Location:** `Resources/Grammars/`

---

## AI/ML Components

### RETE Rule Engine

**Implementation:** Custom AHK implementation

**Features:**
- Forward chaining (fact → rule → action)
- Backward chaining (goal → rule → subgoal)
- Working memory (facts)
- Production memory (rules)
- Pattern matching

**Rule File Format:**
```
[Condition]
Fact = Value

[Action]
Call = FunctionName
Set = Fact, NewValue
```

### LLM Integration

**Local Inference (LLamaSharp):**
- Supports GGUF models
- Multiple backend options (CPU/CUDA/Vulkan)
- Used by Driving Coach

**Cloud Providers:**
- OpenAI (GPT-3.5/4)
- Mistral AI
- Azure OpenAI

**Local Alternatives:**
- Ollama
- GPT4All

---

## Voice System

### Speech Recognition

**Microsoft Speech:**
- Windows native (SAPI 5.4)
- Supports 8+ languages
- Grammar-based recognition

**Google Cloud Speech:**
- Cloud-based
- Streaming recognition
- Higher accuracy

**Whisper (Local):**
- OpenAI Whisper model
- Self-hosted server
- Offline capable

### Speech Synthesis

**Microsoft Speech:**
- Windows native voices
- Multiple languages/voices

**Google Cloud TTS:**
- High-quality neural voices
- Many language options

**Audio Post-Processing:**
- SoX for radio effect
- ffmpeg for format conversion

---

## Communication Protocols

### IPC (Inter-Process Communication)

**AHK Messages:**
- Windows messaging (SendMessage/PostMessage)
- Custom message types
- Used for controller coordination

**File-Based:**
- Temporary files for data exchange
- Lock files for synchronization

### REST API (Team Server)

**Framework:** ASP.NET Core 8.0

**Endpoints:**
- `/api/login` - Authentication
- `/api/account` - User management
- `/api/team` - Team management
- `/api/session` - Session management
- `/api/data` - Telemetry data

**Authentication:** Token-based (custom implementation)

### Shared Memory (Telemetry)

**Simulators Using Shared Memory:**
- ACC, AC, ACE
- iRacing
- rFactor 2, Le Mans Ultimate
- AMS2, PCARS2
- RaceRoom R3E

**Pattern:**
1. Simulator writes to shared memory
2. Provider (C#/C++) reads memory
3. Connector DLL exposes to AHK
4. Plugin processes data

### UDP (Telemetry)

**Simulators Using UDP:**
- ACC (broadcasting)
- Project Motor Racing

**Pattern:**
1. Simulator broadcasts UDP
2. Provider receives packets
3. Data parsed and forwarded

---

## Build System

### AHK Compilation

**Compiler:** Ahk2Exe (AutoHotkey compiler)

**Process:**
1. Preprocess includes
2. Apply conditional compilation
3. Embed resources
4. Sign executable (optional)

### C# Compilation

**Build Tool:** MSBuild / dotnet CLI

**Targets:**
- Debug
- Release

**Output:** DLL/EXE in Release folder

### C++ Compilation

**IDE:** Visual Studio 2019+

**Targets:**
- Win32 (x86)
- x64

**Output:** DLL/EXE in Release folder

### Build Orchestration

**Tool:** Simulator Tools (AHK)

**Configuration:** `Config/Simulator Tools.targets`

**Tasks:**
- Clean binaries
- Copy compiled files
- Update version numbers
- Package releases

---

## Deployment

### Distribution Method

**Primary:** ZIP archive download

**Locations:**
- Amazon S3
- Dropbox
- Custom file server

### Auto-Update

**Mechanism:**
1. Check VERSION file online
2. Compare with local version
3. Download update package
4. Apply update via Simulator Tools

### Installation

**Process:**
1. Download installer or ZIP
2. Run Simulator Tools
3. Configure simulators
4. Set up controllers

---

## Version Control

### Git Repository

**Hosting:** GitHub

**Branches:**
- main (stable)
- Development (active work)

### Version Scheme

**Format:** `Major.Minor.Patch.Build-type`

**Example:** `6.8.1.0-dev`

**Types:**
- `release` - Stable
- `dev` - Development

### Release Artifacts

**Components:**
- Media (splash screens)
- Configuration (setup data)
- Utilities (tools and dependencies)
