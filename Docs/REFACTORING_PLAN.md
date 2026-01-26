# Simulator Controller - Refactoring Master Plan

## Executive Summary

This document outlines a complete refactoring strategy to transform Simulator Controller from a complex, AHK-based application into a modern, maintainable, and user-friendly platform. The plan is organized into phases that can be executed incrementally while maintaining functionality for existing users.

**Primary Goals:**
- Dramatically simplify the first-run experience
- Modernize the technology stack for maintainability
- Fix critical security issues
- Improve performance and stability
- Create a foundation for community contributions

**Estimated Timeline:** 12-18 months for full implementation (can ship incrementally)

---

## Phase 0: Quick Wins (Weeks 1-4)

Before major refactoring, address critical issues that provide immediate value.

### 0.1 Security Hotfixes

**Priority: CRITICAL**

```csharp
// BEFORE (current)
public class Account : ModelObject {
    public string Password { get; set; }  // Plaintext!
}

// AFTER
public class Account : ModelObject {
    public string PasswordHash { get; set; }
    public string PasswordSalt { get; set; }

    public void SetPassword(string password) {
        PasswordSalt = GenerateSalt();
        PasswordHash = HashPassword(password, PasswordSalt);
    }

    public bool VerifyPassword(string password) {
        return HashPassword(password, PasswordSalt) == PasswordHash;
    }

    private static string HashPassword(string password, string salt) {
        using var argon2 = new Argon2id(Encoding.UTF8.GetBytes(password));
        argon2.Salt = Convert.FromBase64String(salt);
        argon2.Iterations = 4;
        argon2.MemorySize = 65536;
        argon2.DegreeOfParallelism = 2;
        return Convert.ToBase64String(argon2.GetBytes(32));
    }
}
```

**Tasks:**
- [ ] Implement password hashing with Argon2id
- [ ] Add migration script for existing accounts (force password reset)
- [ ] Add rate limiting to login endpoint (5 attempts per minute)
- [ ] Add HTTPS requirement documentation
- [ ] Audit all endpoints for authentication bypass

### 0.2 Logging Infrastructure

```csharp
// Add structured logging throughout
public class SessionManager {
    private readonly ILogger<SessionManager> _logger;

    public async Task<Session> CreateSession(CreateSessionRequest request) {
        _logger.LogInformation("Creating session {SessionName} for team {TeamId}",
            request.Name, request.TeamId);

        try {
            var session = await _repository.CreateAsync(request);
            _logger.LogInformation("Session {SessionId} created successfully", session.Id);
            return session;
        }
        catch (Exception ex) {
            _logger.LogError(ex, "Failed to create session {SessionName}", request.Name);
            throw;
        }
    }
}
```

**Tasks:**
- [ ] Add Serilog with structured logging
- [ ] Create log aggregation (file + optional cloud sink)
- [ ] Add correlation IDs for request tracing
- [ ] Create debug mode that captures verbose telemetry

### 0.3 Basic Onboarding Improvements

**Current Problem:** User launches app → sees complex wizard with 15+ steps → gives up

**Quick Fix:** Add a "Quick Start" mode

```
┌─────────────────────────────────────────────────────────────────┐
│                    Welcome to Simulator Controller               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  How would you like to get started?                              │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  🚀 Quick Start (Recommended)                            │    │
│  │  Auto-detect your simulator and get racing in 2 minutes  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  ⚙️ Custom Setup                                         │    │
│  │  Configure every detail (for power users)                │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  📁 Import Configuration                                 │    │
│  │  Load settings from a file or previous installation      │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Tasks:**
- [ ] Create Quick Start flow that auto-detects installed simulators
- [ ] Provide sensible defaults for all settings
- [ ] Delay advanced configuration until user needs it
- [ ] Add "Reset to Defaults" button everywhere

---

## Phase 1: Architecture Modernization (Months 1-4)

### 1.1 New Technology Stack

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         NEW ARCHITECTURE                                     │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                            Frontend                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         Electron + React                             │   │
│  │  • TypeScript for type safety                                        │   │
│  │  • Tailwind CSS for consistent styling                               │   │
│  │  • Zustand for state management                                      │   │
│  │  • React Query for server state                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                              IPC / REST API
                                     │
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Backend Services                                     │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐                   │
│  │   Core        │  │   Telemetry   │  │   Assistant   │                   │
│  │   Service     │  │   Service     │  │   Service     │                   │
│  │   (.NET 8)    │  │   (.NET 8)    │  │   (.NET 8)    │                   │
│  └───────────────┘  └───────────────┘  └───────────────┘                   │
│                                                                              │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐                   │
│  │   Voice       │  │   Team        │  │   Hardware    │                   │
│  │   Service     │  │   Server      │  │   Service     │                   │
│  │   (.NET 8)    │  │   (.NET 8)    │  │   (.NET 8)    │                   │
│  └───────────────┘  └───────────────┘  └───────────────┘                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                              Shared Libraries
                                     │
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Native Layer                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │              Rust-based Telemetry Connectors                         │   │
│  │  • Memory-safe shared memory access                                  │   │
│  │  • Cross-platform UDP handling                                       │   │
│  │  • Versioned structure definitions                                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Data Layer                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    SQLite (via EF Core)                              │   │
│  │  • Single database for all data                                      │   │
│  │  • Proper migrations                                                 │   │
│  │  • Full-text search for telemetry                                    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Project Structure

```
simulator-controller/
├── apps/
│   ├── desktop/                    # Electron app
│   │   ├── src/
│   │   │   ├── main/              # Electron main process
│   │   │   ├── renderer/          # React frontend
│   │   │   │   ├── components/
│   │   │   │   ├── pages/
│   │   │   │   ├── hooks/
│   │   │   │   ├── stores/
│   │   │   │   └── styles/
│   │   │   └── preload/
│   │   └── package.json
│   │
│   └── mobile/                     # Future: companion app
│
├── services/
│   ├── SimController.Core/         # Core business logic
│   ├── SimController.Telemetry/    # Telemetry acquisition
│   ├── SimController.Assistants/   # AI assistants
│   ├── SimController.Voice/        # Voice recognition/synthesis
│   ├── SimController.TeamServer/   # Multiplayer coordination
│   └── SimController.Hardware/     # Hardware integration
│
├── libs/
│   ├── connectors/                 # Rust telemetry connectors
│   │   ├── acc-connector/
│   │   ├── irc-connector/
│   │   ├── rf2-connector/
│   │   └── shared/
│   │
│   ├── SimController.Shared/       # Shared .NET code
│   ├── SimController.RuleEngine/   # RETE rule engine (ported)
│   └── SimController.Database/     # EF Core data layer
│
├── tools/
│   ├── config-migrator/            # Migrate old configs
│   └── telemetry-analyzer/         # Standalone analysis tool
│
├── tests/
│   ├── unit/
│   ├── integration/
│   └── e2e/
│
└── docs/
```

### 1.3 Service Communication

Replace file-based IPC with proper inter-process communication:

```csharp
// Define service contracts
public interface ITelemetryService {
    IAsyncEnumerable<TelemetryFrame> StreamTelemetry(
        string simulator,
        CancellationToken ct);

    Task<TelemetrySnapshot> GetCurrentState(string simulator);
}

public interface IAssistantService {
    Task<AssistantResponse> ProcessEvent(AssistantEvent evt);
    Task<PitstopPlan> PlanPitstop(PitstopRequest request);
    IAsyncEnumerable<AssistantMessage> StreamConversation(
        ConversationContext context,
        CancellationToken ct);
}

// Use gRPC for efficient binary communication
service TelemetryService {
    rpc StreamTelemetry(TelemetryRequest) returns (stream TelemetryFrame);
    rpc GetCurrentState(StateRequest) returns (TelemetrySnapshot);
}

service AssistantService {
    rpc ProcessEvent(AssistantEvent) returns (AssistantResponse);
    rpc PlanPitstop(PitstopRequest) returns (PitstopPlan);
    rpc StreamConversation(ConversationContext) returns (stream AssistantMessage);
}
```

### 1.4 Rust-Based Telemetry Connectors

Replace fragile C++/C# connectors with memory-safe Rust:

```rust
// libs/connectors/shared/src/lib.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("Simulator not running")]
    SimulatorNotRunning,

    #[error("Shared memory version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },

    #[error("Failed to map shared memory: {0}")]
    MappingFailed(#[from] std::io::Error),
}

pub trait SimulatorConnector: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_running(&self) -> bool;
    fn connect(&mut self) -> Result<(), ConnectorError>;
    fn disconnect(&mut self);
    fn read_telemetry(&self) -> Result<TelemetryFrame, ConnectorError>;
    fn read_standings(&self) -> Result<StandingsData, ConnectorError>;
}

// libs/connectors/acc-connector/src/lib.rs
pub struct AccConnector {
    physics: Option<MappedMemory<AccPhysics>>,
    graphics: Option<MappedMemory<AccGraphics>>,
    statics: Option<MappedMemory<AccStatic>>,
}

impl AccConnector {
    // Versioned structure with magic number validation
    const EXPECTED_VERSION: u32 = 0x0108;  // ACC 1.8
    const MAGIC_NUMBER: u32 = 0x41434350;  // "ACCP"

    fn validate_structure(&self) -> Result<(), ConnectorError> {
        if let Some(ref physics) = self.physics {
            let header = physics.read_header()?;
            if header.magic != Self::MAGIC_NUMBER {
                return Err(ConnectorError::SimulatorNotRunning);
            }
            if header.version != Self::EXPECTED_VERSION {
                return Err(ConnectorError::VersionMismatch {
                    expected: Self::EXPECTED_VERSION,
                    actual: header.version,
                });
            }
        }
        Ok(())
    }
}

impl SimulatorConnector for AccConnector {
    fn read_telemetry(&self) -> Result<TelemetryFrame, ConnectorError> {
        self.validate_structure()?;

        let physics = self.physics.as_ref()
            .ok_or(ConnectorError::SimulatorNotRunning)?
            .read()?;

        let graphics = self.graphics.as_ref()
            .ok_or(ConnectorError::SimulatorNotRunning)?
            .read()?;

        Ok(TelemetryFrame {
            timestamp: Instant::now(),
            throttle: physics.gas,
            brake: physics.brake,
            steering: physics.steer_angle,
            speed_kmh: physics.speed_kmh,
            rpm: physics.rpms as u32,
            gear: physics.gear,
            fuel_remaining: physics.fuel,
            // ... normalize all fields
        })
    }
}
```

---

## Phase 2: User Experience Overhaul (Months 2-5)

### 2.1 New Onboarding Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ONBOARDING FLOW                                      │
└─────────────────────────────────────────────────────────────────────────────┘

Step 1: Welcome
┌─────────────────────────────────────────────────────────────────┐
│  👋 Welcome to Simulator Controller!                             │
│                                                                  │
│  Let's get you racing in under 2 minutes.                        │
│                                                                  │
│  [Get Started →]                                                 │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
Step 2: Detect Simulators (Automatic)
┌─────────────────────────────────────────────────────────────────┐
│  🔍 Scanning for installed simulators...                         │
│                                                                  │
│  ✅ Assetto Corsa Competizione (Steam)                          │
│  ✅ iRacing                                                      │
│  ⬚ rFactor 2 (not installed)                                    │
│  ⬚ Automobilista 2 (not installed)                              │
│                                                                  │
│  [Continue with detected simulators →]                          │
│                                                                  │
│  [+ Add simulator manually]                                      │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
Step 3: Choose Your Experience
┌─────────────────────────────────────────────────────────────────┐
│  🎯 What would you like to use?                                  │
│                                                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ ☑️ Race Engineer (Jona)                                    │  │
│  │    Pitstop planning, fuel management, tire setup           │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ ☑️ Race Strategist (Cato)                                  │  │
│  │    Race strategy, weather forecasting                      │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ ☐ Race Spotter (Elisa)                                     │  │
│  │    Traffic alerts, gap tracking                            │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ ☐ Driving Coach (Aiden)                                    │  │
│  │    AI-powered performance coaching                         │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                  │
│  [Continue →]                                                    │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
Step 4: Voice Setup (Optional)
┌─────────────────────────────────────────────────────────────────┐
│  🎤 Voice Control Setup                                          │
│                                                                  │
│  Would you like to talk to your assistants?                      │
│                                                                  │
│  [🎤 Test Microphone]  "Hey Jona, can you hear me?"             │
│                                                                  │
│  ✅ Microphone detected: Blue Yeti                               │
│  ✅ Voice recognition working                                    │
│                                                                  │
│  [Enable Voice Control]  [Skip for now]                         │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
Step 5: Done!
┌─────────────────────────────────────────────────────────────────┐
│  🎉 You're all set!                                              │
│                                                                  │
│  Start your simulator and Simulator Controller will              │
│  automatically connect.                                          │
│                                                                  │
│  Quick Tips:                                                     │
│  • Say "Hey Jona" to talk to your race engineer                 │
│  • Press F10 to open quick settings                             │
│  • Check the tray icon for status                               │
│                                                                  │
│  [Launch ACC Now]  [Open Dashboard]                             │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 New Dashboard Design

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Simulator Controller                              ⚙️ Settings  👤 Profile  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─ Status ──────────────────────────────────────────────────────────────┐  │
│  │                                                                        │  │
│  │  🟢 Ready to connect                     No simulator running          │  │
│  │                                                                        │  │
│  │  Quick Launch:  [ACC]  [iRacing]  [rFactor 2]                         │  │
│  │                                                                        │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌─ Assistants ──────────────────────────────────────────────────────────┐  │
│  │                                                                        │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │  │
│  │  │  👨‍🔧 Jona    │  │  📊 Cato    │  │  👁️ Elisa   │  │  🏎️ Aiden │ │  │
│  │  │  Engineer    │  │  Strategist  │  │  Spotter     │  │  Coach     │ │  │
│  │  │  ──────────  │  │  ──────────  │  │  ──────────  │  │  ────────  │ │  │
│  │  │  ✅ Active   │  │  ✅ Active   │  │  ⬚ Inactive │  │  ⬚ Inactive│ │  │
│  │  │  [Configure] │  │  [Configure] │  │  [Enable]    │  │  [Enable]  │ │  │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘ │  │
│  │                                                                        │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌─ Recent Sessions ─────────────────────────────────────────────────────┐  │
│  │                                                                        │  │
│  │  📅 Today                                                              │  │
│  │  └─ ACC • Spa • Ferrari 296 GT3 • 45 laps • Best: 2:18.432           │  │
│  │                                                                        │  │
│  │  📅 Yesterday                                                          │  │
│  │  └─ iRacing • Daytona • Porsche 911 • 28 laps • Best: 1:42.891       │  │
│  │  └─ ACC • Monza • BMW M4 GT3 • 62 laps • Best: 1:47.234              │  │
│  │                                                                        │  │
│  │  [View All Sessions →]                                                 │  │
│  │                                                                        │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌─ Hardware ────────────────────────────────────────────────────────────┐  │
│  │                                                                        │  │
│  │  🎮 No hardware configured    [+ Add Button Box]  [+ Add Stream Deck] │  │
│  │                                                                        │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.3 In-Session Overlay

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     IN-SESSION OVERLAY (Compact)                             │
└─────────────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────┐
│  LAP 12/45        P3        +2.3s  │
├────────────────────────────────────┤
│  Fuel: 42.3L (8 laps)              │
│  Tires: 67% │ 65% │ 71% │ 69%     │
│  Next Pit: Lap 18-22               │
├────────────────────────────────────┤
│  💬 Jona: "Box this lap for        │
│     dry tires. Rain stopping."     │
│                                    │
│  [OK] [Delay 1 Lap] [Ignore]       │
└────────────────────────────────────┘

// Minimized state (corner of screen)
┌─────────────────────┐
│ P3 │ Fuel: 8 laps │ │
└─────────────────────┘
```

### 2.4 Settings Organization

**Current:** 15+ wizard pages, nested INI files, overwhelming

**New:** Contextual settings that appear when relevant

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Settings                                                           🔍 Search │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─ Categories ─────┐  ┌─ General ───────────────────────────────────────┐ │
│  │                   │  │                                                 │ │
│  │  ▶ General       │  │  Language                                       │ │
│  │    Simulators    │  │  ┌─────────────────────────────────────────┐    │ │
│  │    Assistants    │  │  │ English                              ▼ │    │ │
│  │    Voice         │  │  └─────────────────────────────────────────┘    │ │
│  │    Hardware      │  │                                                 │ │
│  │    Database      │  │  Theme                                          │ │
│  │    Team Server   │  │  ○ Light  ● Dark  ○ System                      │ │
│  │    Advanced      │  │                                                 │ │
│  │                   │  │  Start with Windows                            │ │
│  │                   │  │  [✓] Launch on startup                         │ │
│  │                   │  │  [✓] Start minimized                           │ │
│  │                   │  │                                                 │ │
│  │                   │  │  Units                                          │ │
│  │                   │  │  Temperature: ○ Celsius  ● Fahrenheit          │ │
│  │                   │  │  Pressure:    ● PSI      ○ Bar                 │ │
│  │                   │  │                                                 │ │
│  └───────────────────┘  └─────────────────────────────────────────────────┘ │
│                                                                              │
│                         [Reset to Defaults]              [Save Changes]      │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 3: Data Layer Consolidation (Months 3-6)

### 3.1 Unified Database Schema

Replace CSV files and INI configs with a proper SQLite database:

```sql
-- Database: simulator_controller.db

-- User & Configuration
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    settings JSON  -- User preferences as JSON
);

-- Simulators & Sessions
CREATE TABLE simulators (
    id INTEGER PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,  -- 'ACC', 'IRC', 'RF2'
    name TEXT NOT NULL,
    install_path TEXT,
    detected_at DATETIME,
    settings JSON
);

CREATE TABLE sessions (
    id INTEGER PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    simulator_id INTEGER REFERENCES simulators(id),
    car_id INTEGER REFERENCES cars(id),
    track_id INTEGER REFERENCES tracks(id),
    session_type TEXT,  -- 'practice', 'qualify', 'race'
    started_at DATETIME,
    ended_at DATETIME,
    total_laps INTEGER,
    best_lap_ms INTEGER,
    weather_conditions JSON,
    notes TEXT
);

CREATE TABLE laps (
    id INTEGER PRIMARY KEY,
    session_id INTEGER REFERENCES sessions(id),
    lap_number INTEGER,
    lap_time_ms INTEGER,
    sector_1_ms INTEGER,
    sector_2_ms INTEGER,
    sector_3_ms INTEGER,
    is_valid BOOLEAN,
    fuel_used REAL,
    tire_wear JSON,  -- {fl: 0.95, fr: 0.94, rl: 0.96, rr: 0.95}
    recorded_at DATETIME
);

-- Telemetry (time-series optimized)
CREATE TABLE telemetry_frames (
    id INTEGER PRIMARY KEY,
    lap_id INTEGER REFERENCES laps(id),
    distance_meters REAL,
    timestamp_ms INTEGER,
    throttle REAL,
    brake REAL,
    steering REAL,
    speed_kmh REAL,
    rpm INTEGER,
    gear INTEGER,
    lat_g REAL,
    long_g REAL,
    position_x REAL,
    position_y REAL
);

CREATE INDEX idx_telemetry_lap ON telemetry_frames(lap_id);
CREATE INDEX idx_telemetry_distance ON telemetry_frames(lap_id, distance_meters);

-- Cars & Tracks
CREATE TABLE cars (
    id INTEGER PRIMARY KEY,
    simulator_id INTEGER REFERENCES simulators(id),
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    class TEXT,
    manufacturer TEXT,
    UNIQUE(simulator_id, code)
);

CREATE TABLE tracks (
    id INTEGER PRIMARY KEY,
    simulator_id INTEGER REFERENCES simulators(id),
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    country TEXT,
    length_meters INTEGER,
    UNIQUE(simulator_id, code)
);

-- Tire Pressure Data
CREATE TABLE tire_pressures (
    id INTEGER PRIMARY KEY,
    car_id INTEGER REFERENCES cars(id),
    track_id INTEGER REFERENCES tracks(id),
    weather TEXT,  -- 'dry', 'wet', 'mixed'
    air_temp_c INTEGER,
    track_temp_c INTEGER,
    compound TEXT,
    pressure_cold_fl REAL,
    pressure_cold_fr REAL,
    pressure_cold_rl REAL,
    pressure_cold_rr REAL,
    pressure_hot_fl REAL,
    pressure_hot_fr REAL,
    pressure_hot_rl REAL,
    pressure_hot_rr REAL,
    created_at DATETIME,
    source TEXT  -- 'user', 'community', 'ai_recommended'
);

-- Setup Data
CREATE TABLE setups (
    id INTEGER PRIMARY KEY,
    car_id INTEGER REFERENCES cars(id),
    track_id INTEGER REFERENCES tracks(id),
    name TEXT,
    weather TEXT,
    setup_data JSON,  -- Full setup as JSON
    notes TEXT,
    created_at DATETIME,
    is_baseline BOOLEAN DEFAULT FALSE
);

-- Assistant Conversation History
CREATE TABLE conversations (
    id INTEGER PRIMARY KEY,
    session_id INTEGER REFERENCES sessions(id),
    assistant TEXT,  -- 'jona', 'cato', 'elisa', 'aiden'
    timestamp DATETIME,
    speaker TEXT,  -- 'user', 'assistant'
    message TEXT,
    context JSON
);

-- Full-text search
CREATE VIRTUAL TABLE sessions_fts USING fts5(
    notes,
    content='sessions',
    content_rowid='id'
);
```

### 3.2 Migration Tool

```typescript
// tools/config-migrator/src/index.ts

interface MigrationResult {
  sessionsImported: number;
  lapsImported: number;
  setupsImported: number;
  pressuresImported: number;
  errors: string[];
}

async function migrateFromLegacy(legacyPath: string): Promise<MigrationResult> {
  const result: MigrationResult = {
    sessionsImported: 0,
    lapsImported: 0,
    setupsImported: 0,
    pressuresImported: 0,
    errors: []
  };

  // Migrate INI configuration
  const configFiles = await glob(`${legacyPath}/**/*.ini`);
  for (const file of configFiles) {
    try {
      await migrateIniConfig(file);
    } catch (e) {
      result.errors.push(`Failed to migrate ${file}: ${e.message}`);
    }
  }

  // Migrate CSV data files
  const csvFiles = await glob(`${legacyPath}/**/*.csv`);
  for (const file of csvFiles) {
    try {
      const imported = await migrateCsvData(file);
      result.lapsImported += imported.laps;
      result.pressuresImported += imported.pressures;
    } catch (e) {
      result.errors.push(`Failed to migrate ${file}: ${e.message}`);
    }
  }

  return result;
}
```

---

## Phase 4: AI Assistants Modernization (Months 4-8)

### 4.1 Rule Engine Port to .NET

Port the AHK rule engine to C# with better tooling:

```csharp
// libs/SimController.RuleEngine/RuleEngine.cs

public class RuleEngine {
    private readonly KnowledgeBase _knowledgeBase;
    private readonly List<Rule> _rules;
    private readonly ILogger<RuleEngine> _logger;

    public async Task<IEnumerable<RuleResult>> ProduceAsync(
        string trigger,
        CancellationToken ct = default)
    {
        _knowledgeBase.SetFact(trigger, true);

        var results = new List<RuleResult>();
        var matchingRules = _rules
            .Where(r => r.Matches(_knowledgeBase))
            .OrderByDescending(r => r.Priority);

        foreach (var rule in matchingRules) {
            if (ct.IsCancellationRequested) break;

            _logger.LogDebug("Executing rule: {RuleName}", rule.Name);

            var result = await rule.ExecuteAsync(_knowledgeBase, ct);
            results.Add(result);

            if (result.StopProcessing) break;
        }

        return results;
    }
}

// Rule definition with fluent API
public class PitstopFuelRule : Rule {
    public PitstopFuelRule() {
        Name = "Calculate Fuel Target";
        Priority = 100;

        When(kb => kb.HasFact("Lap"))
            .And(kb => !kb.HasFact("Fuel.Amount.Target"))
            .And(kb => kb.GetFact<double>("Fuel.Remaining") <
                       kb.GetFact<double>("Fuel.Required"));

        Then(async (kb, ct) => {
            var avgConsumption = kb.GetFact<double>("Fuel.AvgConsumption");
            var remainingLaps = kb.GetFact<int>("Session.RemainingLaps");
            var currentFuel = kb.GetFact<double>("Fuel.Remaining");

            var required = avgConsumption * remainingLaps;
            var refillAmount = Math.Max(0, required - currentFuel);

            kb.SetFact("Fuel.Amount.Target", refillAmount);

            return new RuleResult {
                FactsModified = new[] { "Fuel.Amount.Target" },
                Message = $"Calculated fuel target: {refillAmount:F1}L"
            };
        });
    }
}
```

### 4.2 Modern Voice System

Replace grammar-based recognition with hybrid approach:

```csharp
public class VoiceService : IVoiceService {
    private readonly ISpeechRecognizer _recognizer;
    private readonly ISpeechSynthesizer _synthesizer;
    private readonly ILlmService _llm;

    public async Task<VoiceIntent> RecognizeIntentAsync(
        AudioStream audio,
        CancellationToken ct)
    {
        // Step 1: Speech to text
        var transcript = await _recognizer.TranscribeAsync(audio, ct);

        // Step 2: Intent classification (fast, local model)
        var intent = await ClassifyIntentAsync(transcript, ct);

        // Step 3: If unclear, use LLM for disambiguation
        if (intent.Confidence < 0.7) {
            intent = await _llm.ClassifyIntentAsync(transcript, ct);
        }

        return intent;
    }

    private async Task<VoiceIntent> ClassifyIntentAsync(
        string transcript,
        CancellationToken ct)
    {
        // Local intent classification
        var intents = new Dictionary<string, string[]> {
            ["pitstop_plan"] = new[] { "plan", "pit", "stop", "box" },
            ["fuel_check"] = new[] { "fuel", "gas", "remaining" },
            ["tire_status"] = new[] { "tire", "tyre", "wear", "pressure" },
            ["weather"] = new[] { "weather", "rain", "forecast" },
            ["position"] = new[] { "position", "gap", "ahead", "behind" },
        };

        var words = transcript.ToLower().Split(' ');
        var scores = intents.ToDictionary(
            kv => kv.Key,
            kv => kv.Value.Count(w => words.Contains(w)) / (double)kv.Value.Length
        );

        var best = scores.OrderByDescending(kv => kv.Value).First();

        return new VoiceIntent {
            Name = best.Key,
            Confidence = best.Value,
            RawTranscript = transcript
        };
    }
}
```

### 4.3 LLM Integration for All Assistants

Extend LLM support beyond just the Driving Coach:

```csharp
public class AssistantLlmService {
    private readonly ILlmClient _llm;
    private readonly IPromptTemplateService _templates;

    public async Task<string> GenerateResponseAsync(
        AssistantContext context,
        string userMessage,
        CancellationToken ct)
    {
        var systemPrompt = await _templates.GetSystemPromptAsync(
            context.AssistantType,
            context.Simulator,
            context.SessionType
        );

        var messages = new List<ChatMessage> {
            new SystemMessage(systemPrompt),
            new SystemMessage(FormatTelemetryContext(context.Telemetry)),
        };

        // Add conversation history
        foreach (var msg in context.ConversationHistory.TakeLast(10)) {
            messages.Add(msg.Speaker == "user"
                ? new UserMessage(msg.Text)
                : new AssistantMessage(msg.Text));
        }

        messages.Add(new UserMessage(userMessage));

        var response = await _llm.ChatAsync(messages, new ChatOptions {
            MaxTokens = 150,  // Keep responses concise
            Temperature = 0.7,
        }, ct);

        return response.Content;
    }

    private string FormatTelemetryContext(TelemetrySnapshot telemetry) {
        return $"""
            Current session state:
            - Position: P{telemetry.Position} of {telemetry.TotalCars}
            - Lap: {telemetry.CurrentLap} of {telemetry.TotalLaps}
            - Fuel: {telemetry.FuelRemaining:F1}L ({telemetry.FuelLapsRemaining:F1} laps)
            - Tire wear: FL:{telemetry.TireWear.FL:P0} FR:{telemetry.TireWear.FR:P0} RL:{telemetry.TireWear.RL:P0} RR:{telemetry.TireWear.RR:P0}
            - Gap ahead: {telemetry.GapAhead:F1}s
            - Gap behind: {telemetry.GapBehind:F1}s
            - Weather: {telemetry.Weather}, {telemetry.AirTemp}°C air, {telemetry.TrackTemp}°C track
            - Weather forecast: {telemetry.WeatherForecast10Min} in 10min, {telemetry.WeatherForecast30Min} in 30min
            """;
    }
}
```

---

## Phase 5: Testing & Quality (Ongoing)

### 5.1 Test Strategy

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           TEST PYRAMID                                       │
└─────────────────────────────────────────────────────────────────────────────┘

                        ┌─────────────┐
                        │    E2E      │  10%
                        │   Tests     │  Playwright/Selenium
                        └──────┬──────┘
                               │
                    ┌──────────┴──────────┐
                    │   Integration       │  20%
                    │      Tests          │  Service + DB
                    └──────────┬──────────┘
                               │
            ┌──────────────────┴──────────────────┐
            │           Unit Tests                │  70%
            │   Rule Engine, Services, Utils      │
            └─────────────────────────────────────┘
```

### 5.2 Unit Test Examples

```csharp
// tests/unit/SimController.RuleEngine.Tests/FuelCalculationTests.cs

public class FuelCalculationTests {
    [Theory]
    [InlineData(2.5, 10, 15.0, 10.0)]  // Need 25L, have 15L, refill 10L
    [InlineData(2.0, 5, 20.0, 0.0)]    // Need 10L, have 20L, no refill
    [InlineData(3.0, 20, 10.0, 50.0)]  // Need 60L, have 10L, refill 50L
    public async Task CalculateFuelTarget_ReturnsCorrectAmount(
        double avgConsumption,
        int remainingLaps,
        double currentFuel,
        double expectedRefill)
    {
        // Arrange
        var kb = new KnowledgeBase();
        kb.SetFact("Fuel.AvgConsumption", avgConsumption);
        kb.SetFact("Session.RemainingLaps", remainingLaps);
        kb.SetFact("Fuel.Remaining", currentFuel);
        kb.SetFact("Lap", true);

        var rule = new PitstopFuelRule();
        var engine = new RuleEngine(new[] { rule });

        // Act
        await engine.ProduceAsync("Lap");

        // Assert
        var target = kb.GetFact<double>("Fuel.Amount.Target");
        Assert.Equal(expectedRefill, target, precision: 1);
    }
}

// tests/unit/SimController.Telemetry.Tests/AccConnectorTests.cs

public class AccConnectorTests {
    [Fact]
    public void ReadTelemetry_WhenSimulatorNotRunning_ThrowsException() {
        // Arrange
        var connector = new AccConnector();

        // Act & Assert
        Assert.Throws<ConnectorException>(() => connector.ReadTelemetry());
    }

    [Fact]
    public void ReadTelemetry_WhenVersionMismatch_ThrowsWithDetails() {
        // Arrange
        var mockMemory = CreateMockSharedMemory(version: 0x0107);  // Old version
        var connector = new AccConnector(mockMemory);

        // Act & Assert
        var ex = Assert.Throws<ConnectorException>(() => connector.ReadTelemetry());
        Assert.Contains("version mismatch", ex.Message);
        Assert.Contains("expected 0x0108", ex.Message);
    }
}
```

### 5.3 Integration Test Examples

```csharp
// tests/integration/SimController.Api.Tests/SessionApiTests.cs

public class SessionApiTests : IClassFixture<ApiTestFixture> {
    private readonly HttpClient _client;
    private readonly TestDatabase _db;

    [Fact]
    public async Task CreateSession_WithValidData_ReturnsCreatedSession() {
        // Arrange
        var request = new CreateSessionRequest {
            SimulatorCode = "ACC",
            CarCode = "ferrari_296_gt3",
            TrackCode = "spa",
            SessionType = "practice"
        };

        // Act
        var response = await _client.PostAsJsonAsync("/api/sessions", request);

        // Assert
        response.EnsureSuccessStatusCode();
        var session = await response.Content.ReadFromJsonAsync<Session>();

        Assert.NotNull(session);
        Assert.Equal("ACC", session.SimulatorCode);
        Assert.Equal("ferrari_296_gt3", session.CarCode);
    }

    [Fact]
    public async Task GetSession_WithTelemetry_ReturnsLapData() {
        // Arrange
        var session = await _db.CreateTestSession();
        await _db.CreateTestLaps(session.Id, count: 5);

        // Act
        var response = await _client.GetAsync($"/api/sessions/{session.Id}?include=laps");

        // Assert
        response.EnsureSuccessStatusCode();
        var result = await response.Content.ReadFromJsonAsync<SessionWithLaps>();

        Assert.Equal(5, result.Laps.Count);
    }
}
```

### 5.4 E2E Test Examples

```typescript
// tests/e2e/onboarding.spec.ts

import { test, expect } from '@playwright/test';

test.describe('Onboarding Flow', () => {
  test('new user can complete quick start', async ({ page }) => {
    // Clear any existing config
    await page.evaluate(() => localStorage.clear());

    await page.goto('/');

    // Should show welcome screen
    await expect(page.getByText('Welcome to Simulator Controller')).toBeVisible();

    // Click quick start
    await page.getByRole('button', { name: /quick start/i }).click();

    // Should show simulator detection
    await expect(page.getByText('Scanning for installed simulators')).toBeVisible();

    // Wait for detection (mock shows ACC installed)
    await expect(page.getByText('Assetto Corsa Competizione')).toBeVisible();

    // Continue
    await page.getByRole('button', { name: /continue/i }).click();

    // Select assistants
    await expect(page.getByText('What would you like to use')).toBeVisible();
    await page.getByLabel('Race Engineer').check();
    await page.getByRole('button', { name: /continue/i }).click();

    // Skip voice setup
    await page.getByRole('button', { name: /skip for now/i }).click();

    // Should show completion
    await expect(page.getByText("You're all set")).toBeVisible();

    // Go to dashboard
    await page.getByRole('button', { name: /open dashboard/i }).click();

    // Verify dashboard loaded
    await expect(page.getByText('Ready to connect')).toBeVisible();
  });
});
```

---

## Phase 6: Deployment & Distribution (Months 8-10)

### 6.1 Build Pipeline

```yaml
# .github/workflows/ci.yml

name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup .NET
        uses: actions/setup-dotnet@v4
        with:
          dotnet-version: '8.0.x'

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Restore dependencies
        run: |
          dotnet restore
          npm ci --prefix apps/desktop
          cargo fetch --manifest-path libs/connectors/Cargo.toml

      - name: Build
        run: |
          dotnet build --no-restore
          npm run build --prefix apps/desktop
          cargo build --release --manifest-path libs/connectors/Cargo.toml

      - name: Test
        run: |
          dotnet test --no-build --verbosity normal
          npm test --prefix apps/desktop
          cargo test --manifest-path libs/connectors/Cargo.toml

  build-release:
    needs: test
    runs-on: windows-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4

      # ... setup steps ...

      - name: Build Release
        run: npm run make --prefix apps/desktop

      - name: Sign Executable
        run: |
          # Sign with code signing certificate
          signtool sign /f ${{ secrets.CERT_FILE }} /p ${{ secrets.CERT_PASSWORD }} \
            apps/desktop/out/make/squirrel.windows/x64/*.exe

      - name: Upload Artifacts
        uses: actions/upload-artifact@v4
        with:
          name: release-windows
          path: apps/desktop/out/make/squirrel.windows/x64/
```

### 6.2 Auto-Update System

```typescript
// apps/desktop/src/main/updater.ts

import { autoUpdater } from 'electron-updater';
import { dialog, BrowserWindow } from 'electron';

export function initAutoUpdater(mainWindow: BrowserWindow) {
  autoUpdater.autoDownload = false;
  autoUpdater.autoInstallOnAppQuit = true;

  autoUpdater.on('update-available', (info) => {
    dialog.showMessageBox(mainWindow, {
      type: 'info',
      title: 'Update Available',
      message: `Version ${info.version} is available. Would you like to download it now?`,
      buttons: ['Download', 'Later'],
      defaultId: 0,
    }).then((result) => {
      if (result.response === 0) {
        autoUpdater.downloadUpdate();
        mainWindow.webContents.send('update-downloading', info);
      }
    });
  });

  autoUpdater.on('update-downloaded', (info) => {
    dialog.showMessageBox(mainWindow, {
      type: 'info',
      title: 'Update Ready',
      message: 'Update downloaded. Restart to install?',
      buttons: ['Restart', 'Later'],
      defaultId: 0,
    }).then((result) => {
      if (result.response === 0) {
        autoUpdater.quitAndInstall();
      }
    });
  });

  // Check for updates every 4 hours
  setInterval(() => autoUpdater.checkForUpdates(), 4 * 60 * 60 * 1000);
  autoUpdater.checkForUpdates();
}
```

### 6.3 Installation Package

```javascript
// apps/desktop/forge.config.js

module.exports = {
  packagerConfig: {
    name: 'Simulator Controller',
    icon: './assets/icon',
    appBundleId: 'com.simulatorcontroller.app',
    appCategoryType: 'public.app-category.utilities',
    win32metadata: {
      CompanyName: 'Simulator Controller',
      FileDescription: 'Racing Simulator Assistant',
      ProductName: 'Simulator Controller'
    }
  },
  makers: [
    {
      name: '@electron-forge/maker-squirrel',
      config: {
        name: 'SimulatorController',
        setupIcon: './assets/icon.ico',
        loadingGif: './assets/installing.gif'
      }
    }
  ],
  plugins: [
    {
      name: '@electron-forge/plugin-auto-unpack-natives',
      config: {}
    }
  ]
};
```

---

## Migration Strategy

### For Existing Users

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        MIGRATION PATH                                        │
└─────────────────────────────────────────────────────────────────────────────┘

Phase 1: Parallel Installation
┌─────────────────────────────────────────────────────────────────┐
│  • New version installs alongside old version                    │
│  • Automatic detection of legacy installation                    │
│  • One-click migration wizard                                    │
│  • Can switch between versions during transition                 │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
Phase 2: Data Migration
┌─────────────────────────────────────────────────────────────────┐
│  • Import sessions and lap data                                  │
│  • Convert tire pressure databases                               │
│  • Migrate setup files                                           │
│  • Preserve Team Server connections                              │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
Phase 3: Configuration Migration
┌─────────────────────────────────────────────────────────────────┐
│  • Convert INI settings to new format                           │
│  • Map button box configurations                                 │
│  • Preserve voice command customizations                         │
│  • Migrate Stream Deck profiles                                  │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
Phase 4: Validation
┌─────────────────────────────────────────────────────────────────┐
│  • Verify all data migrated correctly                           │
│  • Test assistant functionality                                  │
│  • Confirm hardware still works                                  │
│  • Option to rollback if issues                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Migration Code

```typescript
// tools/config-migrator/src/wizard.tsx

export function MigrationWizard() {
  const [step, setStep] = useState<MigrationStep>('detect');
  const [legacyPath, setLegacyPath] = useState<string | null>(null);
  const [progress, setProgress] = useState<MigrationProgress | null>(null);

  return (
    <div className="migration-wizard">
      {step === 'detect' && (
        <DetectLegacyInstallation
          onFound={(path) => {
            setLegacyPath(path);
            setStep('confirm');
          }}
          onNotFound={() => setStep('manual')}
        />
      )}

      {step === 'confirm' && (
        <ConfirmMigration
          legacyPath={legacyPath!}
          onConfirm={() => setStep('migrate')}
          onSkip={() => setStep('complete')}
        />
      )}

      {step === 'migrate' && (
        <MigrationProgress
          legacyPath={legacyPath!}
          onProgress={setProgress}
          onComplete={() => setStep('verify')}
          onError={(err) => setStep('error')}
        />
      )}

      {step === 'verify' && (
        <VerifyMigration
          progress={progress!}
          onConfirm={() => setStep('complete')}
          onRollback={() => setStep('rollback')}
        />
      )}

      {step === 'complete' && (
        <MigrationComplete
          stats={progress}
        />
      )}
    </div>
  );
}
```

---

## Timeline Summary

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           PROJECT TIMELINE                                   │
└─────────────────────────────────────────────────────────────────────────────┘

Month:  1    2    3    4    5    6    7    8    9    10   11   12
        │    │    │    │    │    │    │    │    │    │    │    │
        ├────┤
        │ P0 │ Quick Wins (Security, Logging, Basic UX)
        │    │
        ├─────────────────────┤
        │        Phase 1       │ Architecture (Stack, Structure)
        │                      │
             ├──────────────────────┤
             │       Phase 2        │ UX Overhaul
             │                      │
                  ├──────────────────────┤
                  │       Phase 3        │ Data Layer
                  │                      │
                       ├─────────────────────────────┤
                       │          Phase 4            │ AI Assistants
                       │                             │
                            ├────────────────────────────────┤
                            │         Phase 5 (ongoing)      │ Testing
                            │                                │
                                      ├──────────────┤
                                      │   Phase 6    │ Deploy
                                      │              │

Milestones:
  Month 2:  Alpha release (core functionality)
  Month 5:  Beta release (full feature parity)
  Month 8:  Release Candidate (migration tools)
  Month 10: v2.0 GA Release
```

---

## Resource Requirements

### Team Composition (Ideal)

| Role | Count | Focus |
|------|-------|-------|
| Tech Lead | 1 | Architecture, code review |
| .NET Developer | 2 | Services, rule engine, connectors |
| Frontend Developer | 1-2 | Electron/React UI |
| Rust Developer | 1 | Telemetry connectors (part-time) |
| QA Engineer | 1 | Testing, automation |
| UX Designer | 1 | UI/UX (part-time, first 4 months) |

### Solo Developer Path

If maintaining as a single developer:

1. **Months 1-4:** Focus on Phase 0 + Phase 2 (security fixes + UX improvements)
2. **Months 5-8:** Gradually port services to .NET, keep AHK UI temporarily
3. **Months 9-12:** Replace UI with Electron once backend stable
4. **Ongoing:** Incremental improvements, community contributions

### Budget Considerations

| Item | Cost |
|------|------|
| Code signing certificate | $200-500/year |
| Cloud hosting (optional) | $20-100/month |
| CI/CD (GitHub Actions) | Free for public repo |
| LLM API costs | Variable ($50-200/month) |
| Domain + hosting for docs | $50-100/year |

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Time to first session | ~30 minutes | < 5 minutes |
| New user retention (7 day) | Unknown | > 60% |
| Crash rate | Unknown | < 1% of sessions |
| GitHub stars | Current | +500 |
| Active monthly users | Current | 2x |
| Community contributions | Low | 10+ PRs/month |

---

## Conclusion

This refactoring plan transforms Simulator Controller from a powerful but complex tool into a modern, approachable application while preserving its advanced capabilities. The key principles are:

1. **Simplify first-run experience** - Get users racing, not configuring
2. **Modern foundation** - Maintainable code that developers want to contribute to
3. **Security by default** - No more plaintext passwords
4. **Incremental migration** - Ship value continuously, don't disappear for 18 months
5. **Preserve power-user features** - Advanced configuration still available, just not required

The result will be an application that's easier to start using, easier to maintain, and easier for the community to improve.
