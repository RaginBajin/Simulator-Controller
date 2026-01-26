# Simulator Controller - Plugin System

## Overview

The Simulator Controller uses a modular plugin architecture that allows for extensibility and clean separation of concerns. The system supports over 40 plugins covering simulators, hardware controllers, AI assistants, and more.

## Plugin Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PLUGIN CLASS HIERARCHY                               │
└─────────────────────────────────────────────────────────────────────────────┘

ConfigurationItem
       │
       ▼
    Plugin (Abstract Base)
       │
       ▼
ControllerPlugin (Controller-specific plugins)
       │
       ├──────────────────────────────────────────────────────────┐
       │                                                          │
       ▼                                                          ▼
  SystemPlugin                                            SimulatorPlugin
  CorePlugin                                                     │
  VoiceControlPlugin                                             │
  MotionFeedbackPlugin                                           ▼
  TactileFeedbackPlugin                              RaceAssistantSimulatorPlugin
  PedalCalibrationPlugin                                         │
  TeamServerPlugin                          ┌────────────────────┼────────────────────┐
  ButtonBoxPlugin                           │                    │                    │
  StreamDeckPlugin                          ▼                    ▼                    ▼
                                        ACPlugin            IRCPlugin          RF2Plugin
                                        ACCPlugin           R3EPlugin          LMUPlugin
                                        AMS2Plugin          PCARS2Plugin       ...
       │
       └──────────────────────────────────────────────────────────┐
                                                                  │
                                                                  ▼
                                                        RaceAssistantPlugin
                                                                  │
                                              ┌───────────────────┼───────────────────┐
                                              │                   │                   │
                                              ▼                   ▼                   ▼
                                    RaceEngineerPlugin   RaceStrategistPlugin  RaceSpotterPlugin
                                    DrivingCoachPlugin
```

## Plugin Loading Sequence

Plugins are loaded in a specific order defined in `Controller Plugins.ahk`:

```
1. Controller Plugins (ButtonBox, StreamDeck)    ── Optional, loaded first
                    ↓
2. System Plugin                                 ── Required, core system
                    ↓
3. Voice Control Plugin                          ── Optional, after System
                    ↓
4. Core Plugin                                   ── Optional, after System
                    ↓
5. Feedback & Assistant Plugins                  ── Motion, Tactile, Pedal, TeamServer
                    ↓
6. Race Assistant Plugins                        ── Engineer, Strategist, Spotter (before Simulators)
                    ↓
7. Simulator Plugins                             ── AC, ACC, AMS2, IRC, R3E, RF2, etc.
                    ↓
8. Integration Plugin                            ── Must load last
```

## Plugin Lifecycle

### Registration

```
                 ┌──────────────────┐
                 │  Plugin Created  │
                 │   __New()        │
                 └────────┬─────────┘
                          │
                          ▼
                 ┌──────────────────┐
                 │ registerPlugin() │
                 │ Called by        │
                 │ Controller       │
                 └────────┬─────────┘
                          │
            ┌─────────────┼─────────────┐
            │ Active?     │             │ Inactive
            │ Yes         │             │
            ▼             │             ▼
   ┌──────────────────┐   │   ┌──────────────────┐
   │    activate()    │   │   │ Plugin logged    │
   │                  │   │   │ as inactive      │
   └────────┬─────────┘   │   └──────────────────┘
            │             │
            ▼             │
   ┌──────────────────┐   │
   │  Connect all     │   │
   │  actions to      │   │
   │  controller      │   │
   │  functions       │   │
   └──────────────────┘   │
```

### Action Connection Flow

```
plugin.activate()
        │
        ▼
controller.connectAction(plugin, function, action)
        │
        ├──► function.connectAction(plugin, action)
        │
        ├──► action.connectFunction(plugin, function)
        │
        └──► fnController.connectAction(plugin, function, action)
                    │
                    ├──► Sets control label/icon
                    │
                    └──► Registers hotkeys
```

### Deactivation

```
plugin.deactivate()
        │
        ▼
controller.disconnectAction(plugin, function, action)
        │
        ├──► function.disconnectAction(plugin, action)
        │
        ├──► action.disconnectFunction(plugin, function)
        │
        └──► fnController.disconnectAction(plugin, function, action)
```

## Plugin Base Classes

### Plugin Class

```autohotkey
class Plugin extends ConfigurationItem {
    ; Properties
    iPlugin := ""              ; Plugin name/identifier
    iIsActive := false         ; Activation status
    iSimulators := []          ; Associated simulators
    iArguments := CaseInsenseMap()  ; Configuration arguments

    ; Key Properties
    Name                       ; Plugin identifier
    Active                     ; Activation status
    Simulators                 ; List of associated simulators
    Arguments                  ; Configuration parameters

    ; Key Methods
    loadFromConfiguration(configuration)
    saveToConfiguration(configuration, merge, excludes)
    getArgumentValue(argument, default)
    hasArgument(parameter)
}
```

### ControllerPlugin Class

```autohotkey
class ControllerPlugin extends Plugin {
    ; Properties
    iController := false
    iModes := []
    iActions := []

    ; Key Methods
    findMode(name)             ; Locate mode by name
    findAction(label)          ; Locate action by label
    registerMode(mode)         ; Register a plugin mode
    registerAction(action)     ; Register an action
    activate()                 ; Activate plugin
    deactivate()               ; Deactivate plugin
    updateFunctions()          ; Update controller function states
    simulatorStartup(simulator); Hook when simulator starts
    simulatorShutdown(simulator); Hook when simulator shuts down
    runningSimulator(active)   ; Check if simulator is running
}
```

### SimulatorPlugin Class

```autohotkey
class SimulatorPlugin extends RaceAssistantSimulatorPlugin {
    ; Properties
    Car                        ; Current car
    Track                      ; Current track
    Session                    ; Current session
    Provider                   ; Simulator data provider
    RaceEngineer               ; Associated race engineer
    RaceStrategist             ; Associated race strategist
    RaceSpotter                ; Associated race spotter

    ; Virtual Methods (implement per simulator)
    getPitstopActions()        ; Define available pitstop options
    selectPitstopOption()      ; UI selection logic
    changePitstopOption()      ; Modify pitstop settings
    createSimulatorProvider()  ; Create telemetry provider

    ; Hooks
    simulatorStartup(simulator)
    simulatorShutdown(simulator)
    updateSession(session)
}
```

## Plugin Types

### Core/System Plugins

| Plugin | Purpose | Key Features |
|--------|---------|--------------|
| **SystemPlugin** | System-level functions | Launch mode, custom modes, application state |
| **CorePlugin** | Extended functionality | Face recognition, Voice Macro integration |
| **VoiceControlPlugin** | Voice commands | Speech recognition, command routing |

### Simulator Plugins

| Plugin | Simulator | Connection |
|--------|-----------|------------|
| **ACPlugin** | Assetto Corsa | Shared Memory |
| **ACCPlugin** | Assetto Corsa Competizione | SHM + UDP |
| **IRCPlugin** | iRacing | SHM + IBT Files |
| **RF2Plugin** | rFactor 2 | Shared Memory (8 buffers) |
| **LMUPlugin** | Le Mans Ultimate | Shared Memory |
| **AMS2Plugin** | Automobilista 2 | Shared Memory |
| **R3EPlugin** | RaceRoom Racing Experience | Shared Memory |
| **PCARS2Plugin** | Project CARS 2 | UDP Multicast |
| **PMRPlugin** | Project Motor Racing | UDP |
| **RSPPlugin** | Rennsport | Shared Memory |

### Race Assistant Plugins

| Plugin | Persona | Responsibilities |
|--------|---------|------------------|
| **RaceEngineerPlugin** | Jona | Pitstop strategy, fuel, tyres |
| **RaceStrategistPlugin** | Cato | Race strategy, weather |
| **RaceSpotterPlugin** | Elisa | Traffic, gaps, positions |
| **DrivingCoachPlugin** | Aiden | LLM-powered coaching |

### Hardware Plugins

| Plugin | Device | Features |
|--------|--------|----------|
| **ButtonBoxPlugin** | Custom button boxes | Grid layout, labels, icons |
| **StreamDeckPlugin** | Elgato Stream Deck | Button images, profiles |
| **MotionFeedbackPlugin** | Motion platforms | SimFeedback integration |
| **TactileFeedbackPlugin** | Haptic devices | SimHub vibration control |
| **PedalCalibrationPlugin** | Pedals | Calibration curves |

## Mode System

### Mode Structure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          MODE SYSTEM                                         │
└─────────────────────────────────────────────────────────────────────────────┘

┌──────────────────┐
│ ControllerMode   │
│──────────────────│
│ iPlugin          │◄────── Associated plugin
│ iActions[]       │◄────── Available actions
│ iFunctionController[]    │◄────── Hardware controllers
└────────┬─────────┘
         │
         │ Lifecycle
         ▼
┌─────────────────────────────────────────┐
│                                         │
│  activate()                             │
│     │                                   │
│     ├──► Enable all actions in mode     │
│     │                                   │
│     └──► Register with controller       │
│                                         │
│  deactivate()                           │
│     │                                   │
│     ├──► Disable all actions in mode    │
│     │                                   │
│     └──► Unregister from controller     │
│                                         │
└─────────────────────────────────────────┘
```

### Mode Activation Flow

```
Simulator Starts
        │
        ▼
controller.setModes(simulator, session)
        │
        ▼
Lookup modes in settings for (simulator, session)
        │
        ▼
for each mode:
    controller.setMode(mode)
        │
        ▼
    mode.activate()
```

### Pitstop Mode (Special Mode)

```autohotkey
class PitstopMode extends AssistantMode {
    ; Updates action availability based on session state
    updateActions(session) {
        ; Enable/disable pitstop actions
        ; Based on:
        ; - Session type (Practice, Qualify, Race)
        ; - Pitstop availability
        ; - Current position (pit lane)
    }
}
```

## Action System

### ControllerAction Class

```autohotkey
class ControllerAction {
    ; Properties
    iFunction := false         ; Associated controller function
    iLabel := ""               ; Display label
    iIcon := false             ; Display icon

    ; Methods
    fireAction(function, trigger)    ; Execute action (virtual)
    connectFunction(plugin, function); Register connection
    disconnectFunction(plugin, function); Unregister connection
}
```

### Action Firing Chain

```
Controller Function (Hotkey/Button Press)
                │
                ▼
SimulatorController.fireActions()
                │
                ▼
ControllerAction.fireAction()
                │
                ▼
Plugin-specific handler
```

## Function Types

| Type | Class | Description |
|------|-------|-------------|
| `k1WayToggleType` | `ControllerOneWayToggleFunction` | Single-state toggle |
| `k2WayToggleType` | `ControllerTwoWayToggleFunction` | Bidirectional toggle |
| `kButtonType` | `ControllerButtonFunction` | Momentary button |
| `kDialType` | `ControllerDialFunction` | Rotary encoder |
| `kCustomType` | `ControllerCustomFunction` | Custom behavior |

## Configuration Format

### Plugin Configuration

```ini
[Plugins]
; Format: PluginName=Active|Simulators|Arguments
ACC Plugin=true|ACC|openPitstopMFD: {Down}; previousOption: {Up}
Race Engineer Plugin=true|ACC,AC,IRC|engineer: Jona; language: en
Button Box Plugin=true|*|layout: 2x3; visible: true
```

### Argument Parsing

```
Arguments: "key1: value1; key2: value2; key3: value3"
                            │
                            ▼
                    Parse semicolon-delimited
                            │
                            ▼
                    Parse colon key:value pairs
                            │
                            ▼
                    CaseInsenseMap {
                        key1 → value1
                        key2 → value2
                        key3 → value3
                    }
```

### Accessing Arguments

```autohotkey
; Get argument with default value
value := this.getArgumentValue("openPitstopMFD", "{Down}")

; Check if argument exists
if this.hasArgument("pitstopMFDMode") {
    ; ...
}

; Set argument value
this.setArgumentValue("key", "value")
```

## Inter-Plugin Communication

### Direct Method Calls

```autohotkey
; RaceAssistantSimulatorPlugin calls methods on assistants
if (this.RaceEngineer)
    this.RaceEngineer.planPitstop()
```

### Remote Process Communication

```autohotkey
class RemoteRaceAssistant {
    callRemote(function, arguments*) {
        messageSend(kFileMessage, this.iRemoteName,
            function . ":" . values2String(";", arguments*),
            this.RemotePID)
    }
}
```

### Event System

```autohotkey
; Raise event from plugin
raiseEvent(event, arguments*) {
    if (this.RaceAssistant)
        this.RaceAssistant.raiseEvent(event, arguments*)
}
```

### Message Manager

```autohotkey
class MessageManager extends PeriodicTask {
    ; Routes messages between plugins and processes
    ; Supports Function and Method handlers
}
```

## Plugin State Tracking

```autohotkey
; Periodic state writing via writeControllerState()
for each plugin in controller.Plugins {
    setMultiMapValue(configuration, plugin.Plugin, "State",
                     active ? "Active" : "Passive")

    ; For simulator plugins
    if (plugin instanceof SimulatorPlugin && active) {
        setMultiMapValue(configuration, "Simulators", simulator,
                         plugin . "|" . sessions)
    }
}
```

## Creating a Custom Plugin

### Basic Template

```autohotkey
class MyCustomPlugin extends ControllerPlugin {
    __New(controller, name, configuration := false) {
        super.__New(controller, name, configuration)

        ; Initialize plugin-specific properties
        this.iMyProperty := "value"
    }

    activate() {
        super.activate()

        ; Plugin activation logic
        logMessage(kLogInfo, "MyCustomPlugin activated")
    }

    deactivate() {
        ; Plugin deactivation logic

        super.deactivate()
    }

    simulatorStartup(simulator) {
        super.simulatorStartup(simulator)

        ; Handle simulator start
    }

    simulatorShutdown(simulator) {
        ; Handle simulator shutdown

        super.simulatorShutdown(simulator)
    }
}
```

### Registering Custom Actions

```autohotkey
class MyCustomPlugin extends ControllerPlugin {
    __New(controller, name, configuration) {
        super.__New(controller, name, configuration)

        ; Create and register action
        myAction := MyCustomAction(this, "MyAction")
        this.registerAction(myAction)
    }
}

class MyCustomAction extends ControllerAction {
    fireAction(function, trigger) {
        ; Handle action execution
        MsgBox("Action fired!")
    }
}
```

## File Locations

| Component | Path |
|-----------|------|
| Plugin Base Classes | `/Sources/Plugins/Libraries/` |
| Core Plugins | `/Sources/Plugins/Core Plugin.ahk` |
| System Plugin | `/Sources/Plugins/System Plugin.ahk` |
| Simulator Plugins | `/Sources/Plugins/*Plugin.ahk` |
| Plugin Loading | `/Sources/Plugins/Controller Plugins.ahk` |
| Controller | `/Sources/Controller/Simulator Controller.ahk` |
