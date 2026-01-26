# Simulator Controller - Telemetry Providers

## Overview

The Simulator Controller uses a multi-layered architecture for connecting to racing simulators. Each simulator connection follows a standardized pattern with specialized components for data acquisition.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     TELEMETRY PROVIDER ARCHITECTURE                          │
└─────────────────────────────────────────────────────────────────────────────┘

┌──────────────────┐
│  Simulator Game  │
│  (ACC, IRC, RF2) │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Shared Memory /  │
│      UDP         │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│    Connector     │
│  (C++ DLL / C#)  │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│    Provider      │
│    (C# EXE)      │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Main Application │
│    (AHK)         │
└────────┬─────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌────────┐ ┌────────┐
│ Coach  │ │Spotter │
│ (C#)   │ │ (C#)   │
└────────┘ └────────┘
```

## Simulator Connection Methods

| Simulator | Abbreviation | Protocol | Connector Type |
|-----------|--------------|----------|----------------|
| **Assetto Corsa Competizione** | ACC | Shared Memory + UDP | C++ DLL |
| **Assetto Corsa** | AC | Shared Memory | C# .NET |
| **rFactor 2** | RF2 | Shared Memory (8 buffers) | C# .NET |
| **Le Mans Ultimate** | LMU | Shared Memory | C# .NET |
| **Automobilista 2** | AMS2 | Shared Memory | C++ DLL |
| **RaceRoom Racing Experience** | R3E | Shared Memory | C++ DLL |
| **iRacing** | IRC | Shared Memory + IBT Files | File Reader |
| **Project CARS 2** | PCARS2 | UDP Multicast | C# UDP |
| **Project Motor Racing** | PMR | UDP | C# UDP |

## Shared Memory Connectors

### ACC Shared Memory Structure

The ACC connector reads from three named memory regions:

```cpp
// Memory Regions
Local\acpmf_physics    // Physics data
Local\acpmf_graphics   // Graphics/race data
Local\acpmf_static     // Static car/track data
```

#### SPageFilePhysics

```cpp
struct SPageFilePhysics {
    // Driver Inputs
    float gas;              // Throttle (0.0-1.0)
    float brake;            // Brake (0.0-1.0)
    float steerAngle;       // Steering angle (degrees)

    // Vehicle State
    float fuel;             // Fuel remaining (liters)
    float speedKmh;         // Speed (km/h)
    int gear;               // Current gear
    int rpms;               // Engine RPM

    // Dynamics
    float velocity[3];      // Velocity vector
    float accG[3];          // G-forces

    // Tires
    float wheelSlip[4];     // Wheel slip ratio
    float wheelLoad[4];     // Wheel load
    float wheelsPressure[4];// Tire pressures
    float tyreCoreTemperature[4];  // Core temps
    float tyreTemp[4];      // Surface temps

    // Brakes
    float brakeTemp[4];     // Brake temperatures

    // Damage
    float suspensionDamage[4];  // Suspension damage

    // Status
    int pitLimiterOn;       // Pit limiter active
    int isEngineRunning;    // Engine running

    // 70+ additional fields...
};
```

#### SPageFileGraphic

```cpp
struct SPageFileGraphic {
    // Session
    int status;             // AC_STATUS enum
    int session;            // AC_SESSION enum
    int completedLaps;      // Laps completed
    int position;           // Race position

    // Timing
    float sessionTimeLeft;  // Remaining time
    float gapAhead;         // Gap to car ahead
    float gapBehind;        // Gap to car behind

    // Strategy
    int currentTyreSet;     // Current tire set
    int strategyTyreSet;    // Strategy tire set

    // Other Cars
    int activeCars;         // Number of cars
    float carCoordinates[60][3];  // Car positions
    int carID[60];          // Car identifiers

    // 40+ additional fields...
};
```

#### SPageFileStatic

```cpp
struct SPageFileStatic {
    // Identification
    wchar_t carModel[33];   // Car model name
    wchar_t track[33];      // Track name
    wchar_t playerName[33]; // Player name

    // Session Info
    int numberOfSessions;   // Total sessions
    int numCars;            // Number of cars
    int sectorCount;        // Track sectors

    // Vehicle Capabilities
    float maxTorque;        // Max torque
    float maxPower;         // Max power
    float maxFuel;          // Fuel capacity
};
```

### RF2 Shared Memory Buffers

rFactor 2 uses 8 separate memory-mapped buffers:

```csharp
MappedBuffer<rF2Telemetry> telemetryBuffer;        // Physics & vehicle state
MappedBuffer<rF2Scoring> scoringBuffer;            // Race standings
MappedBuffer<rF2Rules> rulesBuffer;                // Session rules
MappedBuffer<rF2ForceFeedback> forceFeedbackBuffer;// FFB data
MappedBuffer<rF2Graphics> graphicsBuffer;          // Camera & UI
MappedBuffer<rF2PitInfo> pitInfoBuffer;            // Pit lane data
MappedBuffer<rF2Weather> weatherBuffer;            // Environmental
MappedBuffer<rF2Extended> extendedBuffer;          // Extended telemetry

// Write buffers for feedback
MappedBuffer<rF2HWControl> hwControlBuffer;        // Hardware control
MappedBuffer<rF2RulesControl> rulesControlBuffer;  // Rules control
```

### Memory-Mapped File Access Pattern

```csharp
// Open existing memory-mapped file
MemoryMappedFile mmf = MemoryMappedFile.OpenExisting(name);

// Create view stream
using (var stream = mmf.CreateViewStream()) {
    var reader = new BinaryReader(stream);
    var bytes = reader.ReadBytes(size);

    // Pin memory and marshal to structure
    var handle = GCHandle.Alloc(bytes, GCHandleType.Pinned);
    var data = (T)Marshal.PtrToStructure(
        handle.AddrOfPinnedObject(),
        typeof(T)
    );
}
```

## UDP Providers

### ACC UDP Provider

```csharp
class UDPProvider {
    ACCUdpRemoteClient client;

    // Event handlers
    MessageHandler.OnRealtimeUpdate += OnRealtimeUpdate;
    MessageHandler.OnTrackDataUpdate += OnTrackDataUpdate;
    MessageHandler.OnEntrylistUpdate += OnEntryListUpdate;
    MessageHandler.OnRealtimeCarUpdate += OnRealtimeCarUpdate;
    MessageHandler.OnBroadcastingEvent += OnBroadcastingEvent;
}

class CarData {
    int CarIndex;
    int RaceNumber;
    int CarModelEnum;
    int Position;
    DriverData CurrentDriver;
    LapData BestLap;
    LapData LastLap;
    LapData CurrentLap;
    float SplinePosition;
    float Kmh;
    float Yaw;
    CarLocationEnum CarLocation;
}

class LapData {
    int? LaptimeMS;
    int[] Splits;       // Sector times
    LapType Type;
    bool IsValid;
}
```

### PMR UDP Provider

```csharp
// Multicast configuration
IPAddress multicastAddress = IPAddress.Parse("224.0.0.150");
int port = 7576;

// Join multicast group
udpClient.JoinMulticastGroup(multicastAddress);

// Receive UDP packets
byte[] data = udpClient.Receive(ref remoteEndPoint);

// Parse packet header and data
```

## Normalized Output Format

All providers output data in a standardized key=value format:

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
BrakeTemperature=450, 460, 440, 445
TyreWear=98, 97, 99, 98

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
Car.1.Driver.Surname=Name
Car.1.InPitlane=false
Car.1.LastLap=91500
Car.1.BestLap=91230
```

## Coach Components

### Purpose

Coach components provide real-time driver coaching and telemetry analysis feedback.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      COACH COMPONENT                             │
└─────────────────────────────────────────────────────────────────┘

┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│ Shared Memory    │────►│  Coach Process   │────►│  Driving Coach   │
│ Buffers          │     │  (C# EXE)        │     │  Window          │
└──────────────────┘     └────────┬─────────┘     └──────────────────┘
                                  │
                                  │ WM_COPYDATA
                                  ▼
                         ┌──────────────────┐
                         │  Setup Workbench │
                         │  (Analyzer)      │
                         └──────────────────┘
```

### Coach Data Analysis

```csharp
class SHMCoach {
    // Data buffers
    MappedBuffer<rF2Scoring> scoringBuffer;
    MappedBuffer<rF2Telemetry> telemetryBuffer;
    MappedBuffer<rF2Extended> extendedBuffer;

    // Driving dynamics analysis
    List<CornerDynamics> cornerDynamicsList;
    List<float> recentSteerAngles;
    List<float> recentGLongs;

    // Understeer/Oversteer thresholds
    int understeerLightThreshold = 12;
    int understeerMediumThreshold = 20;
    int understeerHeavyThreshold = 35;
    int oversteerLightThreshold = 2;
    int oversteerMediumThreshold = -6;
    int oversteerHeavyThreshold = -10;

    // Output methods
    void SendTriggerMessage(string message);    // To Driving Coach
    void SendAnalyzerMessage(string message);   // To Setup Workbench
    void playSound(string wavFile);             // Audio feedback
}
```

### IPC Communication

```csharp
// Windows message for IPC
const int WM_COPYDATA = 0x004A;

struct COPYDATASTRUCT {
    IntPtr dwData;      // Message ID: (256 * 'D' + 'C')
    int cbData;         // Message length
    string lpData;      // Message content
}

// Send to Driving Coach window
int winHandle = FindWindowEx(0, 0, null, "Driving Coach.exe");
SendMessage(hWnd, WM_COPYDATA, wParam, ref COPYDATASTRUCT);
```

## Spotter Components

### Purpose

Spotter components monitor other cars' positions and provide race position and strategy information.

### Functionality

- Monitor other cars' positions relative to player
- Track lap differences and gaps
- Provide position/gap information to Race Spotter assistant
- Analyze pit strategy of opponents

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         COMPLETE DATA FLOW                                   │
└─────────────────────────────────────────────────────────────────────────────┘

    Racing Simulator
          │
          ├─────────────────────────────────────────┐
          │                                         │
          ▼                                         ▼
    ┌───────────┐                            ┌───────────┐
    │  Shared   │                            │   UDP     │
    │  Memory   │                            │ Broadcast │
    └─────┬─────┘                            └─────┬─────┘
          │                                         │
          ▼                                         ▼
    ┌───────────┐                            ┌───────────┐
    │ Connector │                            │    UDP    │
    │  (DLL)    │                            │ Provider  │
    └─────┬─────┘                            └─────┬─────┘
          │                                         │
          ▼                                         │
    ┌───────────┐                                   │
    │ Provider  │◄──────────────────────────────────┘
    │   (EXE)   │
    └─────┬─────┘
          │
          │  Output File (key=value format)
          ▼
    ┌───────────────────────────────────────┐
    │        Main Application               │
    │  ┌─────────────────────────────────┐  │
    │  │     Simulator Provider          │  │
    │  │     (AHK Class)                 │  │
    │  └─────────────┬───────────────────┘  │
    │                │                      │
    │    ┌───────────┴───────────┐          │
    │    │                       │          │
    │    ▼                       ▼          │
    │ ┌────────┐           ┌────────┐       │
    │ │Telemetry│           │Standings│      │
    │ │ Data   │           │  Data  │       │
    │ └────┬───┘           └────┬───┘       │
    └──────┼────────────────────┼───────────┘
           │                    │
           ├────────────────────┼────────────────┐
           │                    │                │
           ▼                    ▼                ▼
    ┌────────────┐      ┌────────────┐   ┌────────────┐
    │   Race     │      │    Race    │   │   Race     │
    │  Engineer  │      │ Strategist │   │  Spotter   │
    └────────────┘      └────────────┘   └────────────┘
           │                    │                │
           └────────────────────┴────────────────┘
                               │
                               ▼
                        ┌────────────┐
                        │  Driving   │
                        │   Coach    │
                        └────────────┘
```

## Provider Registration

### SimulatorProvider Base Class

```autohotkey
class SimulatorProvider {
    static sSimulatorProviders := Map()

    static Protocols {
        Get {
            return {
                Connector: {
                    Type: "DLL",
                    File: "Connectors\%simulator% %protocol% Connector.dll"
                },
                Provider: {
                    Type: "EXE",
                    File: "Providers\%simulator% %protocol% Provider.exe"
                },
                Spotter: {
                    Type: "EXE",
                    File: "Providers\%simulator% %protocol% Spotter.exe"
                },
                Coach: {
                    Type: "EXE",
                    File: "Providers\%simulator% %protocol% Coach.exe"
                }
            }
        }
    }

    static createSimulatorProvider(simulator, car, track) {
        return %simulator%Provider(car, track)
    }
}
```

### Simulator-Specific Providers

```autohotkey
class ACCProvider extends SimulatorProvider {
    iUDPProvider := ACCUDPProvider()

    acquireTelemetryData() {
        ; Read from Connector DLL
        ; Process data
        ; Return normalized multimap
    }

    acquireStandingsData(telemetryData) {
        ; Get standings from UDP Provider
        ; Merge with telemetry
    }
}
```

## File-Based Coordination

### Async Standings Acquisition

```
Main Application
        │
        ├── Create UDP Provider task
        │
        ├── Write "Read\n" to ACCUDP.cmd
        │
        ├── Wait for ACCUDP.cmd to be consumed
        │
        ├── Read ACCUDP.out (multimap format)
        │
        └── Parse and integrate into session
```

### File Paths

```
%TEMP%\ACCUDP.cmd           - Commands from application
%TEMP%\ACCUDP.out           - Output standings data
%TEMP%\ACCUDP.out.Trace     - Debug trace log
```

## Performance Optimizations

| Technique | Description |
|-----------|-------------|
| **Partial Memory Reads** | Only read changed portions of buffer (RF2) |
| **Skip Unchanged Flag** | Avoid processing duplicate data |
| **Version Blocks** | Detect stale data without full reads |
| **Observable Collections** | .NET change notifications |
| **Async File I/O** | Non-blocking standings acquisition |
| **Thread Pooling** | Separate threads for Coach/Spotter |

## File Locations

| Component | Path |
|-----------|------|
| Connectors | `/Sources/Special/*SHM Connector/` |
| Providers | `/Sources/Special/*SHM Provider/` |
| UDP Providers | `/Sources/Special/*UDP Provider/` |
| Coaches | `/Sources/Special/*SHM Coach/` |
| Spotters | `/Sources/Special/*SHM Spotter/` |
| Provider Base | `/Sources/Plugins/Libraries/SimulatorProvider.ahk` |
