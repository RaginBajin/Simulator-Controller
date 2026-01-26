# Simulator Controller - AI Assistants

## Overview

The Simulator Controller includes four specialized AI assistants that run as separate processes. They use a RETE-based rule engine for decision-making and support natural language interaction through voice recognition and synthesis.

## Assistant Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          AI ASSISTANTS                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐           │
│  │  Race Engineer  │   │ Race Strategist │   │   Race Spotter  │           │
│  │     (Jona)      │   │     (Cato)      │   │    (Elisa)      │           │
│  │─────────────────│   │─────────────────│   │─────────────────│           │
│  │ • Pitstop Plan  │   │ • Race Strategy │   │ • Traffic Info  │           │
│  │ • Fuel Mgmt     │   │ • Weather       │   │ • Gap Tracking  │           │
│  │ • Tyre Setup    │   │ • Pit Timing    │   │ • Blue Flags    │           │
│  │ • Damage Report │   │ • Position Sim  │   │ • Opponent Pits │           │
│  └─────────────────┘   └─────────────────┘   └─────────────────┘           │
│                                                                              │
│                    ┌─────────────────┐                                       │
│                    │  Driving Coach  │                                       │
│                    │    (Aiden)      │                                       │
│                    │─────────────────│                                       │
│                    │ • LLM Coaching  │                                       │
│                    │ • Corner Advice │                                       │
│                    │ • Telemetry     │                                       │
│                    │ • Performance   │                                       │
│                    └─────────────────┘                                       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ASSISTANT ARCHITECTURE                                    │
└─────────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────────┐
│                         MAIN APPLICATION                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  Simulator Controller                                                │    │
│  │     │                                                                │    │
│  │     └──► Race Assistant Plugin ─────────────────────────────────────┼────┤
│  │              │                                                       │    │
│  └──────────────┼───────────────────────────────────────────────────────┘    │
│                 │                                                             │
│                 │ messageSend() / IPC                                        │
│                 │                                                             │
└─────────────────┼─────────────────────────────────────────────────────────────┘
                  │
    ┌─────────────┴─────────────┬─────────────────────┬─────────────────────┐
    │                           │                     │                     │
    ▼                           ▼                     ▼                     ▼
┌──────────────┐         ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│Race Engineer │         │Race Strategist│     │ Race Spotter │     │Driving Coach │
│    .exe      │         │     .exe      │     │     .exe     │     │     .exe     │
│──────────────│         │───────────────│     │──────────────│     │──────────────│
│              │         │               │     │              │     │              │
│ ┌──────────┐ │         │ ┌───────────┐ │     │ ┌──────────┐ │     │ ┌──────────┐ │
│ │Knowledge │ │         │ │ Knowledge │ │     │ │Knowledge │ │     │ │Knowledge │ │
│ │  Base    │ │         │ │   Base    │ │     │ │  Base    │ │     │ │  Base    │ │
│ └────┬─────┘ │         │ └─────┬─────┘ │     │ └────┬─────┘ │     │ └────┬─────┘ │
│      │       │         │       │       │     │      │       │     │      │       │
│ ┌────┴─────┐ │         │ ┌─────┴─────┐ │     │ ┌────┴─────┐ │     │ ┌────┴─────┐ │
│ │  Rule    │ │         │ │   Rule    │ │     │ │  Rule    │ │     │ │  Rule    │ │
│ │  Engine  │ │         │ │  Engine   │ │     │ │  Engine  │ │     │ │  Engine  │ │
│ └──────────┘ │         │ └───────────┘ │     │ └──────────┘ │     │ └──────────┘ │
│              │         │               │     │              │     │      │       │
│ ┌──────────┐ │         │ ┌───────────┐ │     │ ┌──────────┐ │     │ ┌────┴─────┐ │
│ │  Voice   │ │         │ │   Voice   │ │     │ │  Voice   │ │     │ │   LLM    │ │
│ │ Manager  │ │         │ │  Manager  │ │     │ │ Manager  │ │     │ │Connector │ │
│ └──────────┘ │         │ └───────────┘ │     │ └──────────┘ │     │ └──────────┘ │
│              │         │               │     │              │     │              │
└──────────────┘         └───────────────┘     └──────────────┘     └──────────────┘
```

## Race Engineer (Jona)

### Responsibilities

- Real-time vehicle telemetry monitoring (fuel, energy, tire wear, brake wear)
- Pitstop planning and execution
- Technical warning system (low fuel, tire degradation, damage)
- Damage analysis and recovery predictions
- Pressure loss compensation
- Tire and brake temperature monitoring

### Event Types

| Event | Description |
|-------|-------------|
| `FuelLowEvent` | Low fuel warnings |
| `EnergyLowEvent` | Energy depletion warnings |
| `GripLowEvent` | Tire grip degradation |
| `TyreWearEvent` | Tire tread analysis |
| `BrakeWearEvent` | Brake pad wear tracking |
| `DamageEvent` | Collision/damage detection |
| `TimeLossEvent` | Performance impact analysis |
| `PressureLossEvent` | Tire pressure monitoring |

### Decision Making

```
┌─────────────────────────────────────────────────────────────────┐
│                RACE ENGINEER DECISION FLOW                       │
└─────────────────────────────────────────────────────────────────┘

    Lap Event
        │
        ▼
┌───────────────────┐
│ Analyze Lap Data  │
│ - Fuel consumed   │
│ - Tire wear       │
│ - Damage state    │
│ - Lap time        │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ Update Knowledge  │
│ Base Facts        │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐     ┌───────────────────┐
│ Execute Rules     │────►│ Generate Warnings │
│                   │     │ - Low fuel        │
│                   │     │ - Tire wear       │
│                   │     │ - Damage          │
└─────────┬─────────┘     └───────────────────┘
          │
          ▼
┌───────────────────┐
│ Calculate Optimal │
│ - Refuel amount   │
│ - Tire compound   │
│ - Pressures       │
│ - Repair needs    │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ Voice Output      │
│ to Driver         │
└───────────────────┘
```

## Race Strategist (Cato)

### Responsibilities

- Race strategy optimization (pitstop timing, tire strategies)
- Weather forecast integration and response planning
- Multi-lap strategy simulation and validation
- Standings computation and extrapolation
- Fuel consumption rate analysis
- Race pace management
- Full course yellow and safety car handling

### Event Types

| Event | Description |
|-------|-------------|
| `PitstopUpcomingEvent` | Notifies of pitstop timing |
| `WeatherForecastEvent` | Weather changes and tire recommendations |
| `RecommendPitstopEvent` | Optimal pitstop lap suggestion |
| `SimulateStrategyEvent` | Multi-lap scenario analysis |

### Strategy Flow

```
┌─────────────────────────────────────────────────────────────────┐
│               RACE STRATEGIST DECISION FLOW                      │
└─────────────────────────────────────────────────────────────────┘

    Race Data
        │
        ▼
┌───────────────────┐
│ Monitor Conditions│
│ - Weather         │
│ - Standings       │
│ - Pace            │
│ - Fuel rate       │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ Weather Forecast  │
│ Integration       │
│ - 10 min ahead    │
│ - 30 min ahead    │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ Simulate Multiple │
│ Strategy Options  │
│ - No-stop         │
│ - 1-stop          │
│ - 2-stop          │
│ - Weather-based   │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ Recommend Optimal │
│ Strategy          │
│ - Pit window      │
│ - Tire selection  │
│ - Fuel load       │
└───────────────────┘
```

## Race Spotter (Elisa)

### Responsibilities

- Real-time opponent tracking and gap monitoring
- Safety alerts (blue flag, attack imminent warnings)
- Stint and session progression tracking
- Opponent pit activity monitoring
- Multi-class racing support
- Position delta analysis
- Track-relative gap measurements

### Event Types

| Event | Description |
|-------|-------------|
| `AheadGapUpdateEvent` | Gap to car ahead changes |
| `BehindGapUpdateEvent` | Gap to car behind changes |
| `AttackImminentEvent` | Threat of being overtaken |
| `BlueFlagAlertEvent` | Lap down/safety notification |
| `StintEndingEvent` | Stint conclusion warning |
| `LastLapEvent` | Final lap notification |
| `SessionOverEvent` | Session completion |
| `OpponentPittingEvent` | Pit activity alerts |

### CarInfo Tracking

```autohotkey
class CarInfo {
    ; Per-opponent telemetry
    LapTimes[]          ; Historical lap times
    SectorTimes[]       ; Sector split times
    BestLap             ; Best lap time
    Damage              ; Damage/incident tracking
    InvalidLap          ; Invalid lap detection

    ; Delta computation methods
    ; - Static: Fixed reference
    ; - Dynamic: Rolling average
    ; - Both: Combined analysis
}
```

## Driving Coach (Aiden)

### Responsibilities

- Driver performance coaching via telemetry analysis
- Lap-by-lap performance feedback
- Corner-specific coaching (approach, technique, recovery)
- Brake coaching with real-time feedback
- Reference lap management and comparison
- Telemetry data collection and analysis
- Multi-mode operation (Conversation, Coaching, Analysis)

### Operating Modes

| Mode | Description |
|------|-------------|
| **Conversation** | Interactive dialogue with driver |
| **Coaching** | Active performance feedback |
| **Analysis** | Post-lap review and recommendations |

### LLM Integration

```
┌─────────────────────────────────────────────────────────────────┐
│                  DRIVING COACH LLM FLOW                          │
└─────────────────────────────────────────────────────────────────┘

    Event Trigger
    (Lap/Corner/Stint)
        │
        ▼
┌───────────────────┐
│ Collect Telemetry │
│ - Corner data     │
│ - Brake data      │
│ - Speed data      │
│ - G-forces        │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ Build Context     │
│ - Instructions    │
│ - Knowledge       │
│ - Hints           │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ LLM Connector     │
│ - Claude          │
│ - OpenAI          │
│ - Ollama/Local    │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ Process Response  │
│ - Extract advice  │
│ - Format output   │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ Voice Output      │
│ to Driver         │
└───────────────────┘
```

### Instruction Templates

| Template Type | Purpose |
|---------------|---------|
| `Character` | Coach personality definition |
| `Simulation` | Simulator-specific context |
| `Session` | Session type context |
| `Stint` | Stint-specific instructions |
| `Knowledge` | Data context |
| `Handling` | UI/control instructions |
| `Coaching` | Coaching mode instructions |
| `Coaching.Lap` | Lap review instructions |
| `Coaching.Corner` | Corner analysis |
| `Coaching.Corner.Approaching` | Entry technique |
| `Coaching.Corner.Problems` | Issue detection |
| `Coaching.Corner.Review` | Post-corner analysis |
| `Coaching.Reference` | Comparison coaching |

## Rule Engine

### Overview

The assistants use a RETE-based forward-chaining rule engine for decision-making. Rules are defined in `.rules` files and operate on a knowledge base of facts.

### Rule File Format

```prolog
; Global Triggers
[?Lap] => ...          ; Triggered each lap
[?Update] => ...       ; Invoked for each rule engine cycle
[?Pitstop.Plan] => ... ; Pitstop planning trigger
[?Pitstop.Prepare] =>  ; Pitstop MFD preparation
[?Cleanup] => ...      ; Cleanup unnecessary knowledge

; Production Rules
{Any: [?Lap], {None: [?Fuel.Amount.Target]}} =>
    (Prove: updateFuelTarget(?Lap))

; Backward-chaining predicates
updateFuelTarget(?lap) <=
    lapAvgFuelConsumption(?lap, ?avgConsumption),
    lapRemainingFuel(?lap, ?remainingFuel),
    remainingSessionLaps(?lap, ?sessionLaps),
    ?neededFuel = ?avgConsumption * ?sessionLaps,
    ?neededFuel > ?remainingFuel,
    ?refillAmount = ?neededFuel - ?remainingFuel,
    Set(Fuel.Amount.Target, ?refillAmount), !
```

### Rule Categories

| Category | Purpose |
|----------|---------|
| Car Information Retrieval | Extract current vehicle state |
| Lap Information Retrieval | Analyze lap performance data |
| Session Information Retrieval | Session-wide metrics |
| Pitstop Computations | Calculate pitstop requirements |
| Standings Computations | Calculate race positions |
| Statistical Computations | Performance statistics |
| Strategy Validation | Verify strategy viability |
| Tyre Information Retrieval | Tire data extraction |
| Weather Information Retrieval | Weather state queries |
| Weather Notifications | Weather change alerts |
| Action Callbacks | Rule completion handlers |

### Rule Execution Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    RULE ENGINE EXECUTION                         │
└─────────────────────────────────────────────────────────────────┘

1. Assistant receives trigger (lap event, pitstop event, etc.)
                │
                ▼
2. Knowledge base fact is set
                │
                ▼
3. Rule engine produce() - executes all matching rules
                │
                ▼
4. Rules query knowledge base with conditions
                │
                ▼
5. Matching rule heads execute (Set, Clear, Prove, ProveAll)
                │
                ▼
6. Knowledge updated; dependent rules trigger
                │
                ▼
7. Results propagated back to assistant methods
```

## Voice System

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     VOICE SYSTEM ARCHITECTURE                    │
└─────────────────────────────────────────────────────────────────┘

┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│   Driver Voice   │────►│  Voice Server    │────►│    Assistant     │
│     Input        │     │      .exe        │     │                  │
└──────────────────┘     └────────┬─────────┘     └──────────────────┘
                                  │
                    ┌─────────────┼─────────────┐
                    │             │             │
                    ▼             ▼             ▼
            ┌────────────┐ ┌────────────┐ ┌────────────┐
            │  Grammar   │ │   Text     │ │   TTS      │
            │Recognition │ │Recognition │ │ Synthesis  │
            └────────────┘ └────────────┘ └────────────┘
```

### Voice Manager Class

```autohotkey
class VoiceManager {
    iSynthesizer     ; "dotNET" (text-to-speech engine)
    iRecognizer      ; "Desktop" (speech recognition engine)
    iRecognizerMode  ; "Grammar" or "Text"

    ; Nested classes
    class RemoteSpeaker {
        speak(text, focus, cache, options)
    }

    class VoiceGrammars {
        ; Multi-language grammar support
    }
}
```

### Speech Synthesis Parameters

| Parameter | Range | Description |
|-----------|-------|-------------|
| Speaker Volume | 0-100 | Output volume |
| Speaker Pitch | -10 to +10 | Voice pitch |
| Speaker Speed | -10 to +10 | Speech rate |
| SpeakerBooster | Service name | Audio enhancement |
| SpeakerVocalics | Characteristics | Voice style |
| UseTalking | Boolean | Batch phrase assembly |

### Speech Recognition Modes

| Mode | Description |
|------|-------------|
| **Grammar Mode** | Structured command recognition from predefined grammars |
| **Text Mode** | Free-form speech-to-text transcription |

### Grammar Structure

```ini
[Listener Grammars]
Call = [{ Hi, Hey, Hello } %name%, %name% do you hear me, ...]
Yes = [Yes { please, of course }, { Yes, Perfect } go on, ...]
No = [No { thank you, not now }, ...]
Joke = [(CanYou) tell me a joke, Do you have a joke for me]
Deactivate = [Shut up, Silence please, Be quiet please, ...]
Activate = [Okay you can talk, I can listen { now, again }, ...]
```

### Grammar Syntax

| Syntax | Meaning |
|--------|---------|
| `{ }` | Optional elements |
| `[ ]` | Phrase alternatives |
| `%var%` | Variable substitution |
| `*` | Wildcard patterns |
| `(Macro)` | Grammar macro reference |

### Supported Languages

- English (en)
- German (de)
- French (fr)
- Spanish (es)
- Italian (it)
- Portuguese (pt)
- Japanese (ja)
- Chinese (zh)

## Inter-Process Communication

### Message Protocol

```
Main App sends: "Race Engineer:methodName:arg1;arg2;arg3"
                            │
                            ▼
              handleEngineerMessage() receives
                            │
                            ▼
              StrSplit on ":" delimiter
                            │
                            ▼
              ObjBindMethod() calls method with args
                            │
                            ▼
              Result via RemoteHandler
```

### RemoteHandler Pattern

```autohotkey
class RaceAssistantRemoteHandler {
    __New(assistantName, remotePID) {
        this.iRemoteName := assistantName
        this.iRemotePID := remotePID
    }

    callRemote(method, arguments*) {
        messageSend(kFileMessage, this.iRemoteName,
            method . ":" . values2String(";", arguments*),
            "ahk_pid " . this.iRemotePID)
    }
}
```

### State Synchronization

```autohotkey
; Save session state
saveSessionState(settingsFile, stateFile)

; Save pitstop state
savePitstopState(lapNumber, fileName)

; Save session info
saveSessionInfo(lapNumber, fileName)
```

## Startup Parameters

### Race Engineer

```
Race Engineer.exe
    -Remote [PID]                 ; Remote handler PID
    -Name "Jona"                  ; Assistant name
    -Logo On                      ; Show splash screen
    -Language en                  ; Language code
    -Translator [service]         ; Translation service
    -Synthesizer dotNET           ; TTS engine
    -Speaker true                 ; Enable voice output
    -SpeakerVocalics [vocal]      ; Voice characteristics
    -SpeakerBooster [service]     ; Audio enhancement
    -Recognizer Desktop           ; Speech recognition engine
    -Listener true                ; Enable voice input
    -ListenerBooster [service]    ; Recognition enhancement
    -ConversationBooster [service]; Conversation LLM
    -AgentBooster [service]       ; Agent LLM
    -Muted                        ; No audio output
    -Voice [VoiceServer.exe]      ; Voice server path
    -Debug true                   ; Debug mode
```

## File Locations

| Component | Path |
|-----------|------|
| Assistant Apps | `/Sources/Assistants/*.ahk` |
| Assistant Libraries | `/Sources/Assistants/Libraries/` |
| Rule Files | `/Sources/Assistants/Rules/` |
| Grammars | `/Sources/Assistants/Grammars/` |
| Instructions | `/Sources/Assistants/Instructions/` |
| Voice Manager | `/Sources/Assistants/Libraries/VoiceManager.ahk` |

## Rule Files Statistics

| File | Size | Purpose |
|------|------|---------|
| Race Engineer.rules | 117 KB | Engineering rules |
| Race Strategist.rules | 34 KB | Strategy optimization |
| Race Spotter.rules | 11 KB | Spotter logic |
| Driving Coach.rules | 24 KB | Coaching rules |
| Agent Actions.rules | - | Action callbacks |
| Formula 1.rules | - | F1-specific rules |
