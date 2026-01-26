# Simulator Controller - Data Models

## Overview

This document describes the data models and relationships used throughout the Simulator Controller system.

## Team Server Data Model

The Team Server uses SQLite with the `sqlite-net-pcl` ORM library. All models inherit from `ModelObject` which provides auto-incrementing IDs and GUID-based identifiers.

### Entity Relationship Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        TEAM SERVER DATABASE                                  │
└─────────────────────────────────────────────────────────────────────────────┘

┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│   Account    │ 1     N │    Token     │ 1     N │  Connection  │
│──────────────│─────────│──────────────│─────────│──────────────│
│ ID (PK)      │         │ ID (PK)      │         │ ID (PK)      │
│ Identifier   │         │ Identifier   │         │ Identifier   │
│ Name         │         │ AccountID(FK)│         │ TokenID (FK) │
│ EMail        │         │ Type         │         │ SessionID    │
│ Password     │         │ Created      │         │ Type         │
│ Virgin       │         │ Until        │         │ Client       │
│ Administrator│         │ Used         │         │ Name         │
│ AvailMinutes │         └──────────────┘         │ Created      │
│ DataAccess   │                                  │ Valid        │
│ SessionAccess│                                  └──────────────┘
│ Contract     │
└──────┬───────┘
       │
       │ 1
       │
       ├────────────────────────────────┐
       │                                │
       │ N                              │ N
┌──────┴───────┐                 ┌──────┴───────┐
│     Team     │ 1             N │  DataObject  │
│──────────────│─────────────────│──────────────│
│ ID (PK)      │                 │ ID (PK)      │
│ Identifier   │                 │ Identifier   │
│ AccountID(FK)│                 │ AccountID(FK)│
│ Name         │                 │ Modified     │
└──────┬───────┘                 │ Simulator    │
       │                         │ Car          │
       │ 1                       │ Track        │
       │                         │ Driver       │
       ├─────────────┐           └──────────────┘
       │             │                  ▲
       │ N           │ N                │ Inherits
┌──────┴───────┐  ┌──┴───────────┐     │
│    Driver    │  │   Session    │     ├── Document
│──────────────│  │──────────────│     ├── License
│ ID (PK)      │  │ ID (PK)      │     ├── Electronics
│ Identifier   │  │ Identifier   │     ├── Tyres
│ TeamID (FK)  │  │ TeamID (FK)  │     ├── Brakes
│ ForName      │  │ Name         │     ├── TyresPressures
│ SurName      │  │ Duration     │     └── TyresPressuresDistribution
│ NickName     │  │ Track        │
└──────┬───────┘  │ Car          │
       │          │ Started      │
       │ 1        │ Finished     │
       │          │ StartTime    │
       │          │ FinishTime   │
       │          └──────┬───────┘
       │                 │
       │                 │ 1
       │ N               │
┌──────┴─────────────────┴───────┐
│            Stint               │
│────────────────────────────────│
│ ID (PK)                        │
│ Identifier                     │
│ SessionID (FK)                 │
│ DriverID (FK)                  │
│ Nr (stint number)              │
│ Lap (starting lap)             │
└────────────────┬───────────────┘
                 │
                 │ 1
                 │
                 │ N
          ┌──────┴───────┐
          │     Lap      │
          │──────────────│
          │ ID (PK)      │
          │ Identifier   │
          │ SessionID(FK)│
          │ StintID (FK) │
          │ Nr (lap num) │
          └──────────────┘
```

### Token Types

```
┌─────────────────────────────────────────────────────────────────┐
│                     TOKEN HIERARCHY                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Internal Token (System Level)                                   │
│       │                                                          │
│       └──► Account Token (Login Required)                        │
│                 │                                                │
│                 ├──► Session Token (Team/Session Operations)     │
│                 │                                                │
│                 └──► Data Token (Telemetry Data Access)          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Contract Types

| Type | Description |
|------|-------------|
| `Expired` | No API access allowed |
| `OneTime` | Single session, deleted after minutes exhausted |
| `FixedMinutes` | Renews to fixed amount monthly |
| `AdditionalMinutes` | Adds fixed amount monthly |
| `Unlimited` | Unlimited session access |

### Database Tables

| Table | Purpose |
|-------|---------|
| `Access_Accounts` | User accounts |
| `Access_Tokens` | Authentication tokens |
| `Access_Connections` | Active client connections |
| `Teams` | Team entities |
| `Drivers` | Team drivers |
| `Sessions` | Racing sessions |
| `Stints` | Driver stints within sessions |
| `Laps` | Individual lap records |
| `Data_Documents` | Generic telemetry documents |
| `Data_Licenses` | Driver license data |
| `Data_Electronics` | Electronics setup data |
| `Data_Tyres` | Tyre telemetry |
| `Data_Brakes` | Brake telemetry |
| `Data_TyresPressures` | Tyre pressure data |
| `Data_TyresPressuresDistribution` | Pressure distribution statistics |
| `Task_Tasks` | Background maintenance tasks |
| `Attributes` | Generic key-value storage |

---

## Session Database Data Model

The Session Database is a file-based system using CSV files organized hierarchically.

### Directory Structure

```
DatabasePath/
├── User/
│   ├── [SimulatorCode]/          # ACC, RFactor2, iRacing, etc.
│   │   ├── Settings.csv
│   │   └── [CarCode]/
│   │       └── [TrackCode]/
│   │           ├── Tyres.Pressures.csv
│   │           ├── Tyres.Pressures.Distribution.csv
│   │           ├── Electronics.csv
│   │           └── Tyres.csv
│   └── ...
├── Community/
│   └── [Same structure as User]
├── ID                             # Driver identifier file
└── Session Database.ini           # Configuration
```

### Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     SESSION DATABASE DATA FLOW                               │
└─────────────────────────────────────────────────────────────────────────────┘

┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Simulator  │     │  Telemetry   │     │   Session    │
│    Game      │────►│  Collector   │────►│   Database   │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                  │
         ┌────────────────────────────────────────┤
         │                                        │
         ▼                                        ▼
┌──────────────┐                         ┌──────────────┐
│    Tyres     │                         │    Laps      │
│   Database   │                         │   Database   │
│──────────────│                         │──────────────│
│ Pressures    │                         │ Electronics  │
│ Distribution │                         │ Tyres        │
│ Hot/Cold     │                         │ Lap Times    │
└──────┬───────┘                         └──────┬───────┘
       │                                        │
       │         ┌──────────────┐               │
       │         │   Settings   │               │
       │         │   Database   │               │
       │         │──────────────│               │
       └────────►│ Strategy     │◄──────────────┘
                 │ Setup        │
                 │ Preferences  │
                 └──────────────┘
                        │
                        ▼
              ┌──────────────────┐
              │  AI Assistants   │
              │  (Race Engineer, │
              │   Strategist,    │
              │   Coach)         │
              └──────────────────┘
```

### Tyres Database Schema

#### Tyres.Pressures Table

| Column | Type | Description |
|--------|------|-------------|
| Weather | String | Dry, Drizzle, LightRain, MediumRain, HeavyRain, Thunderstorm |
| Temperature.Air | Integer | Air temperature (°C) |
| Temperature.Track | Integer | Track temperature (°C) |
| Compound | String | Tyre compound (e.g., "Dry (S)", "Wet (Black)") |
| Compound.Color | String | Compound color identifier |
| Tyre.Pressure.Cold.Front.Left | Float | Cold pressure FL (bar) |
| Tyre.Pressure.Cold.Front.Right | Float | Cold pressure FR (bar) |
| Tyre.Pressure.Cold.Rear.Left | Float | Cold pressure RL (bar) |
| Tyre.Pressure.Cold.Rear.Right | Float | Cold pressure RR (bar) |
| Tyre.Pressure.Hot.Front.Left | Float | Hot pressure FL (bar) |
| Tyre.Pressure.Hot.Front.Right | Float | Hot pressure FR (bar) |
| Tyre.Pressure.Hot.Rear.Left | Float | Hot pressure RL (bar) |
| Tyre.Pressure.Hot.Rear.Right | Float | Hot pressure RR (bar) |
| Driver | String | Driver identifier |
| Identifier | GUID | Synchronization identifier |
| Synchronized | Timestamp | Last sync time |

#### Tyres.Pressures.Distribution Table

| Column | Type | Description |
|--------|------|-------------|
| Weather | String | Weather condition |
| Temperature.Air | Integer | Air temperature |
| Temperature.Track | Integer | Track temperature |
| Compound | String | Tyre compound |
| Compound.Color | String | Compound color |
| Type | String | "Cold" or "Hot" |
| Tyre | String | "FL", "FR", "RL", "RR" |
| Pressure | Float | Pressure value (rounded to 0.1) |
| Count | Integer | Statistical occurrence count |
| Driver | String | Driver identifier |
| Identifier | GUID | Sync identifier |
| Synchronized | Timestamp | Last sync time |

### Settings Database Schema

| Column | Type | Description |
|--------|------|-------------|
| Owner | String | User/Driver ID |
| Car | String | Car code (or "*" for any) |
| Track | String | Track code (or "*" for any) |
| Weather | String | Weather condition |
| Section | String | Setting category |
| Key | String | Setting name |
| Value | String | Setting value |
| Mode | String | Setup mode (DQ, DR, WQ, WR) |

#### Setup Modes

| Mode | Description |
|------|-------------|
| DQ | Dry Qualification |
| DR | Dry Race |
| WQ | Wet Qualification |
| WR | Wet Race |

### Electronics Table

| Column | Type | Description |
|--------|------|-------------|
| Weather | String | Weather condition |
| Temperature.Air | Integer | Air temperature |
| Temperature.Track | Integer | Track temperature |
| Tyre.Compound | String | Compound |
| Tyre.Compound.Color | String | Compound color |
| Fuel.Remaining | Float | Remaining fuel (L) |
| Fuel.Consumption | Float | Fuel consumption |
| Lap.Time | Integer | Lap time (ms) |
| TC | Integer | Traction control setting |
| ABS | Integer | ABS setting |
| BB | Float | Brake bias |
| Driver | String | Driver ID |
| Identifier | GUID | Sync ID |
| Synchronized | Timestamp | Sync time |

### Tyres Table (Lap Data)

| Column | Type | Description |
|--------|------|-------------|
| Weather | String | Weather condition |
| Temperature.Air | Integer | Air temperature |
| Temperature.Track | Integer | Track temperature |
| Tyre.Compound | String | Compound |
| Tyre.Compound.Color | String | Compound color |
| Fuel.Remaining | Float | Remaining fuel |
| Fuel.Consumption | Float | Fuel consumption |
| Lap.Time | Integer | Lap time (ms) |
| Tyre.Laps | Integer | Tyre age (laps) |
| Tyre.Pressure.Front.Left | Float | FL pressure |
| Tyre.Pressure.Front.Right | Float | FR pressure |
| Tyre.Pressure.Rear.Left | Float | RL pressure |
| Tyre.Pressure.Rear.Right | Float | RR pressure |
| Tyre.Temperature.Front.Left | Float | FL temperature |
| Tyre.Temperature.Front.Right | Float | FR temperature |
| Tyre.Temperature.Rear.Left | Float | RL temperature |
| Tyre.Temperature.Rear.Right | Float | RR temperature |
| Tyre.Wear.Front.Left | Float | FL wear % |
| Tyre.Wear.Front.Right | Float | FR wear % |
| Tyre.Wear.Rear.Left | Float | RL wear % |
| Tyre.Wear.Rear.Right | Float | RR wear % |
| Driver | String | Driver ID |
| Identifier | GUID | Sync ID |
| Synchronized | Timestamp | Sync time |

---

## Telemetry Data Model

### Telemetry Channel Structure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     TELEMETRY DATA CHANNELS                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Index 1:  Distance        - Track distance traveled (meters)                │
│  Index 2:  Throttle        - Throttle input (0.0 - 1.0)                     │
│  Index 3:  Brake           - Brake input (0.0 - 1.0)                        │
│  Index 4:  Steering        - Steering angle (degrees)                        │
│  Index 5:  Gear            - Current gear (0 = neutral, -1 = reverse)       │
│  Index 6:  RPM             - Engine RPM                                      │
│  Index 7:  Speed           - Vehicle speed (km/h)                           │
│  Index 8:  TC              - Traction control activation (0/1)              │
│  Index 9:  ABS             - ABS activation (0/1)                           │
│  Index 10: Long G          - Longitudinal G-force                           │
│  Index 11: Lat G           - Lateral G-force                                │
│  Index 12: PosX            - World X coordinate                             │
│  Index 13: PosY            - World Y coordinate                             │
│  Index 14: Time            - Timestamp (milliseconds)                       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Section Analysis Structure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      LAP SECTION ANALYSIS                                    │
└─────────────────────────────────────────────────────────────────────────────┘

                    ┌───────────────────────┐
                    │        LAP            │
                    │───────────────────────│
                    │ Lap Number            │
                    │ Lap Time              │
                    │ Valid Flag            │
                    │ Sections[]            │
                    └───────────┬───────────┘
                                │
            ┌───────────────────┼───────────────────┐
            │                   │                   │
            ▼                   ▼                   ▼
    ┌───────────────┐   ┌───────────────┐   ┌───────────────┐
    │    Corner     │   │   Straight    │   │    Corner     │
    │───────────────│   │───────────────│   │───────────────│
    │ Nr (1, 2, 3..)│   │ Duration      │   │ Nr            │
    │ Entry Phase   │   │ Min Speed     │   │ Entry Phase   │
    │ Apex Phase    │   │ Max Speed     │   │ Apex Phase    │
    │ Exit Phase    │   │ Avg Speed     │   │ Exit Phase    │
    │ Steering      │   │ Lat G = 0     │   │ Steering      │
    └───────────────┘   └───────────────┘   └───────────────┘

    ┌───────────────────────────────────────────────────────────┐
    │                   CORNER PHASES                            │
    ├───────────────────────────────────────────────────────────┤
    │                                                            │
    │  Entry/Braking Phase:                                      │
    │    - Brake Point Distance (from apex)                      │
    │    - Max Brake Pressure                                    │
    │    - Brake Ramp-up Distance                                │
    │    - ABS Activation %                                      │
    │                                                            │
    │  Apex/Rolling Phase:                                       │
    │    - Minimum Speed                                         │
    │    - Lateral G (min/max/avg)                               │
    │    - Rolling Gear & RPM                                    │
    │    - Curvature: -Log(((Speed/3.6)²)/LatG)                 │
    │                                                            │
    │  Exit/Acceleration Phase:                                  │
    │    - Throttle Smoothness                                   │
    │    - TC Activation %                                       │
    │    - Gear/RPM Progression                                  │
    │    - Final Speed                                           │
    │                                                            │
    │  Steering Analysis:                                        │
    │    - Corrections Count                                     │
    │    - Smoothness % (vs 90% threshold)                       │
    │    - Max Steering Angle Range                              │
    │                                                            │
    └───────────────────────────────────────────────────────────┘
```

### Performance Thresholds

| Metric | Threshold | Description |
|--------|-----------|-------------|
| TC Activations | 20% | Maximum acceptable TC usage |
| ABS Activations | 30% | Maximum acceptable ABS usage |
| Steering Smoothness | 90% | Minimum smoothness target |
| Throttle Smoothness | 90% | Minimum smoothness target |
| Brake Smoothness | 90% | Minimum smoothness target |

---

## Simulator Telemetry Data Model

### Shared Memory Structures (ACC Example)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ACC SHARED MEMORY STRUCTURE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  SPageFilePhysics (Local\acpmf_physics)                             │    │
│  │─────────────────────────────────────────────────────────────────────│    │
│  │  - gas, brake, fuel, steerAngle, speedKmh                           │    │
│  │  - velocity[3], accG[3]                                             │    │
│  │  - wheelSlip[4], wheelLoad[4], wheelsPressure[4]                    │    │
│  │  - tyreCoreTemperature[4], brakeTemp[4]                             │    │
│  │  - suspensionDamage[4], tyreTemp[4]                                 │    │
│  │  - gear, rpms, pitLimiterOn, isEngineRunning                        │    │
│  │  - (70+ additional fields)                                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  SPageFileGraphic (Local\acpmf_graphics)                            │    │
│  │─────────────────────────────────────────────────────────────────────│    │
│  │  - status, session, completedLaps, position                         │    │
│  │  - sessionTimeLeft, gapAhead, gapBehind                             │    │
│  │  - currentTyreSet, strategyTyreSet                                  │    │
│  │  - activeCars, carCoordinates[60][3], carID[60]                     │    │
│  │  - (40+ additional fields for penalties, damage, lights)            │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  SPageFileStatic (Local\acpmf_static)                               │    │
│  │─────────────────────────────────────────────────────────────────────│    │
│  │  - carModel[33], track[33], playerName[33]                          │    │
│  │  - numberOfSessions, numCars, sectorCount                           │    │
│  │  - maxTorque, maxPower, maxFuel                                     │    │
│  │  - (vehicle capabilities, AI aids settings)                         │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Normalized Output Format

```ini
[Session Data]
Active=true
Paused=false
Session=Race
ID=1
Car=ferrari488gte
Track=monza

[Stint Data]
DriverForname=John
DriverSurname=Doe
DriverNickname=JD
Laps=5
LapValid=true
LapLastTime=92450
LapBestTime=91230
Position=2
GapAhead=1500
GapBehind=2300
InPitLane=false

[Car Data]
MAP=8
TC=3
ABS=3
Ignition=true
FuelRemaining=67.5
TyreCompound=Dry
TyreTemperature=95, 96, 94, 95
TyrePressure=27.5, 27.6, 27.4, 27.5

[Track Data]
Temperature=32
Grip=Optimum

[Weather Data]
Temperature=28
Weather=Dry
Weather10min=Dry
Weather30min=Dry

[Position Data]
Car.Count=20
Car.1.ID=1
Car.1.Position=1
Car.1.Laps=5
Car.1.Driver.Forname=Leader
Car.1.InPitlane=false
```

---

## Knowledge Base Data Model

### Rule Engine Facts Structure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     KNOWLEDGE BASE FACTS                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Session Facts:                                                              │
│    Session.Started, Session.Finished                                         │
│    Session.Duration, Session.TimeRemaining                                   │
│    Session.Type (Practice, Qualify, Race)                                    │
│    Session.Weather, Session.Track, Session.Car                               │
│                                                                              │
│  Lap Facts:                                                                  │
│    Lap.Current, Lap.Best, Lap.Last                                          │
│    Lap.Valid, Lap.Time, Lap.Fuel.Consumption                                │
│    Lap.Position, Lap.Gap.Ahead, Lap.Gap.Behind                              │
│                                                                              │
│  Car Facts:                                                                  │
│    Car.Fuel.Remaining, Car.Fuel.AvgConsumption                              │
│    Car.Tyre.Compound, Car.Tyre.Pressure[FL,FR,RL,RR]                        │
│    Car.Tyre.Temperature[FL,FR,RL,RR]                                        │
│    Car.Tyre.Wear[FL,FR,RL,RR]                                               │
│    Car.Brake.Wear[FL,FR,RL,RR]                                              │
│    Car.Damage[Front,Rear,Left,Right,Center]                                 │
│    Car.TC, Car.ABS, Car.BB, Car.MAP                                         │
│                                                                              │
│  Pitstop Facts:                                                              │
│    Pitstop.Planned, Pitstop.Prepared                                        │
│    Pitstop.Lap.Target                                                        │
│    Pitstop.Fuel.Amount                                                       │
│    Pitstop.Tyre.Compound, Pitstop.Tyre.Set                                  │
│    Pitstop.Tyre.Pressure[FL,FR,RL,RR]                                       │
│    Pitstop.Repairs[Suspension,Bodywork]                                     │
│                                                                              │
│  Strategy Facts:                                                             │
│    Strategy.Weather.Current, Strategy.Weather.10Min, Strategy.Weather.30Min │
│    Strategy.Tyre.Remaining.Laps                                              │
│    Strategy.Fuel.Remaining.Laps                                              │
│    Strategy.Pitstop.Count, Strategy.Pitstop.Delta                           │
│                                                                              │
│  Standings Facts:                                                            │
│    Standings.Position[N].Car, Standings.Position[N].Driver                  │
│    Standings.Position[N].Laps, Standings.Position[N].Gap                    │
│    Standings.Position[N].LastLap, Standings.Position[N].BestLap             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Rule Execution Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      RULE ENGINE EXECUTION                                   │
└─────────────────────────────────────────────────────────────────────────────┘

        ┌──────────────────┐
        │   Trigger Event   │
        │  (Lap, Pitstop,   │
        │   Update, etc.)   │
        └────────┬─────────┘
                 │
                 ▼
        ┌──────────────────┐
        │ Set Trigger Fact │
        │ in Knowledge Base│
        └────────┬─────────┘
                 │
                 ▼
        ┌──────────────────┐
        │  produce()       │
        │  Execute Rules   │
        └────────┬─────────┘
                 │
        ┌────────┼────────┐
        │        │        │
        ▼        ▼        ▼
   ┌─────────┐ ┌─────────┐ ┌─────────┐
   │ Rule 1  │ │ Rule 2  │ │ Rule N  │
   │ Match?  │ │ Match?  │ │ Match?  │
   └────┬────┘ └────┬────┘ └────┬────┘
        │           │           │
        └─────┬─────┴─────┬─────┘
              │           │
        ┌─────┴─────┐ ┌───┴───┐
        │ Execute   │ │ Skip  │
        │ Actions   │ │       │
        └─────┬─────┘ └───────┘
              │
              ▼
        ┌──────────────────┐
        │ Update Facts     │
        │ (Set, Clear,     │
        │  Prove, ProveAll)│
        └────────┬─────────┘
                 │
                 ▼
        ┌──────────────────┐
        │ Trigger Dependent│
        │ Rules (if any)   │
        └────────┬─────────┘
                 │
                 ▼
        ┌──────────────────┐
        │ Return Results   │
        │ to Assistant     │
        └──────────────────┘
```

---

## Configuration Data Model

### Plugin Configuration Format

```ini
[Plugins]
PluginName=Active|Simulators|Arguments

; Example:
ACC Plugin=true|ACC|openPitstopMFD: {Down}; previousOption: {Up}
Race Engineer Plugin=true|ACC,AC,IRC|engineer: Jona; language: en
```

### Argument Parsing Structure

```
Arguments String: "key1: value1; key2: value2; key3: value3"
                              │
                              ▼
                   ┌──────────────────┐
                   │  CaseInsenseMap  │
                   │──────────────────│
                   │ key1 → value1    │
                   │ key2 → value2    │
                   │ key3 → value3    │
                   └──────────────────┘
```

### Settings Query Hierarchy (13 Levels)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SETTINGS INHERITANCE HIERARCHY                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Level 1 (Least Specific):                                                   │
│    [*, *, *, *]                    Global defaults                           │
│                                                                              │
│  Level 2:                                                                    │
│    [Car, *, *, *]                  Car-specific                              │
│    [*, Track, *, *]                Track-specific                            │
│    [*, *, Mode, *]                 Mode-specific                             │
│    [*, *, *, Weather]              Weather-specific                          │
│                                                                              │
│  Level 3:                                                                    │
│    [Car, Track, *, *]                                                        │
│    [Car, *, Mode, *]                                                         │
│    [Car, *, *, Weather]                                                      │
│    [*, Track, Mode, *]                                                       │
│    [*, Track, *, Weather]                                                    │
│    [*, *, Mode, Weather]                                                     │
│                                                                              │
│  Level 4:                                                                    │
│    [Car, Track, Mode, *]                                                     │
│    [Car, Track, *, Weather]                                                  │
│    [Car, *, Mode, Weather]                                                   │
│    [*, Track, Mode, Weather]                                                 │
│                                                                              │
│  Level 5 (Most Specific):                                                    │
│    [Car, Track, Mode, Weather]     Exact match                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```
