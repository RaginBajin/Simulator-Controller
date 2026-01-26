# Simulator Controller - Team Server API

## Overview

The Team Server is an ASP.NET Core 8.0 REST API that provides multiplayer coordination, session management, and data sharing capabilities. It uses SQLite for persistence and token-based authentication.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         TEAM SERVER ARCHITECTURE                             │
└─────────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────────┐
│                              ASP.NET Core Server                             │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         REST API Controllers                           │ │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐         │ │
│  │  │  Account   │ │   Login/   │ │    Team    │ │   Driver   │         │ │
│  │  │ Controller │ │  Access    │ │ Controller │ │ Controller │         │ │
│  │  └────────────┘ └────────────┘ └────────────┘ └────────────┘         │ │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐                        │ │
│  │  │  Session   │ │    Data    │ │    Task    │                        │ │
│  │  │ Controller │ │ Controller │ │ Controller │                        │ │
│  │  └────────────┘ └────────────┘ └────────────┘                        │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                     │                                        │
│  ┌──────────────────────────────────┼────────────────────────────────────┐  │
│  │                          Service Managers                              │  │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐         │  │
│  │  │   Token    │ │  Account   │ │    Team    │ │  Session   │         │  │
│  │  │  Issuer    │ │  Manager   │ │  Manager   │ │  Manager   │         │  │
│  │  └────────────┘ └────────────┘ └────────────┘ └────────────┘         │  │
│  │  ┌────────────┐ ┌────────────┐                                        │  │
│  │  │    Data    │ │    Task    │                                        │  │
│  │  │  Manager   │ │  Manager   │                                        │  │
│  │  └────────────┘ └────────────┘                                        │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                     │                                        │
│  ┌──────────────────────────────────┼────────────────────────────────────┐  │
│  │                         Object Manager (ORM)                           │  │
│  │                                  │                                     │  │
│  │                         ┌───────┴───────┐                             │  │
│  │                         │    SQLite     │                             │  │
│  │                         │   Database    │                             │  │
│  │                         └───────────────┘                             │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Authentication

### Token Hierarchy

```
┌─────────────────────────────────────────────────────────────────┐
│                     TOKEN HIERARCHY                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Internal Token (System Level - AccountID = 0)                   │
│       │                                                          │
│       └──► Account Token (Login Required)                        │
│                 │                                                │
│                 ├──► Session Token                               │
│                 │    (Team/Session Operations)                   │
│                 │                                                │
│                 └──► Data Token                                  │
│                      (Telemetry Data Access)                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Token Types

| Type | Access Level | Purpose |
|------|--------------|---------|
| `Internal` | Full system | Internal operations only |
| `Account` | User account | Login, token issuance |
| `Session` | Team/session | Team and session management |
| `Data` | Data storage | Telemetry data operations |

### Token Validation

1. Token must exist and not be expired (`Until` timestamp)
2. Account must not be expired (`Contract != Expired`)
3. Token must not exceed 5-minute idle window
4. Token type must have permission for operation

## API Endpoints

### Account Controller (`/api/account`)

| Method | Endpoint | Description | Required Token |
|--------|----------|-------------|----------------|
| GET | `/allaccounts?token={token}` | List all accounts (admin) | Account (Admin) |
| GET | `/{identifier}?token={token}` | Get account details | Account |
| PUT | `/{identifier}?token={token}` | Update account | Account |
| PUT | `/{identifier}/password?token={token}` | Change password | Account |
| PUT | `/{identifier}/contract?token={token}` | Update contract | Account (Admin) |
| PUT | `/{identifier}/minutes?token={token}` | Set available minutes | Account (Admin) |
| POST | `?token={token}` | Create new account | Account (Admin) |
| DELETE | `/{identifier}?token={token}` | Delete account | Account (Admin) |

### Login/Access Controller (`/api/login`)

| Method | Endpoint | Description | Required Token |
|--------|----------|-------------|----------------|
| GET | `?name={name}&password={password}` | Login | None |
| GET | `/token/{type}?token={token}` | Issue Session/Data token | Account |
| DELETE | `/token/{identifier}?token={token}` | Delete token | Account |
| GET | `/connect/{category}?token={token}&...` | Create connection | Token |
| GET | `/validatetoken?token={token}` | Validate token | Any |
| GET | `/accountavailableminutes?token={token}` | Get remaining minutes | Account |
| PUT | `/password?token={token}` | Change own password | Account |
| GET | `/warmup?token={token}` | Warm up database | Account |
| GET | `/allconnections?token={token}` | List connections (admin) | Account (Admin) |
| GET | `/allobjects?token={token}` | Get object counts | Account (Admin) |
| GET | `/allsessions?token={token}` | List all sessions | Account |
| GET | `/compactingdatabase?token={token}` | Check compaction status | Account (Admin) |
| GET | `/compactdatabase?token={token}` | Compact database | Account (Admin) |
| GET | `/{identifier}?token={token}&keepalive={bool}` | Get connection | Token |

### Logout Controller (`/api/logout`)

| Method | Endpoint | Description | Required Token |
|--------|----------|-------------|----------------|
| DELETE | `?token={token}` | Logout/delete token | Any |

### Team Controller (`/api/team`)

| Method | Endpoint | Description | Required Token |
|--------|----------|-------------|----------------|
| GET | `/allteams?token={token}` | List teams for account | Session |
| GET | `/{identifier}?token={token}` | Get team details | Session |
| GET | `/{identifier}/drivers?token={token}` | List team drivers | Session |
| GET | `/{identifier}/sessions?token={token}` | List team sessions | Session |
| PUT | `/{identifier}?token={token}` | Update team | Session |
| POST | `?token={token}` | Create team | Session |
| DELETE | `/{identifier}?token={token}` | Delete team | Session |

### Driver Controller (`/api/driver`)

| Method | Endpoint | Description | Required Token |
|--------|----------|-------------|----------------|
| GET | `/{identifier}?token={token}` | Get driver details | Session |
| PUT | `/{identifier}?token={token}` | Update driver | Session |
| POST | `?token={token}&team={team}` | Create driver | Session |
| DELETE | `/{identifier}?token={token}` | Delete driver | Session |

### Session Controller (`/api/session`)

| Method | Endpoint | Description | Required Token |
|--------|----------|-------------|----------------|
| GET | `/validatetoken?token={token}` | Validate Session token | Session |
| GET | `/sessions?token={token}` | List sessions | Session |
| GET | `/{identifier}?token={token}` | Get session details | Session |
| GET | `/{identifier}/value?token={token}&name={name}` | Get session attribute | Session |
| PUT | `/{identifier}/value?token={token}&name={name}` | Set session attribute | Session |
| GET | `/{identifier}/stint/{stint}/value?...` | Get stint attribute | Session |
| PUT | `/{identifier}/stint/{stint}/value?...` | Set stint attribute | Session |
| POST | `?token={token}&team={team}` | Create session | Session |
| DELETE | `/{identifier}?token={token}` | Delete session | Session |
| PUT | `/{identifier}/start?token={token}` | Start session | Session |
| PUT | `/{identifier}/finish?token={token}` | Finish session | Session |
| PUT | `/{identifier}/clear?token={token}` | Clear session data | Session |

### Data Controller (`/api/data`)

| Method | Endpoint | Description | Required Token |
|--------|----------|-------------|----------------|
| GET | `/validatetoken?token={token}` | Validate Data token | Data |
| GET | `/timestamp?token={token}` | Get server timestamp | Data |
| PUT | `/query/{table}?token={token}` | Query data | Data |
| PUT | `/count/{table}?token={token}` | Count records | Data |
| POST | `/{table}?token={token}` | Create record | Data |
| PUT | `/{table}/{identifier}?token={token}` | Update record | Data |
| DELETE | `/{table}/{identifier}?token={token}` | Delete record | Data |

**Supported Tables:**
- `Document`
- `License`
- `Electronics`
- `Tyres`
- `Brakes`
- `TyresPressures`
- `TyresPressuresDistribution`

### Task Controller (`/api/task`)

| Method | Endpoint | Description | Required Token |
|--------|----------|-------------|----------------|
| GET | `/alltasks?token={token}` | List all tasks (admin) | Account (Admin) |
| GET | `/{identifier}?token={token}` | Get task details | Account (Admin) |
| PUT | `/{identifier}?token={token}` | Update task | Account (Admin) |
| POST | `?token={token}` | Create task | Account (Admin) |
| DELETE | `/{identifier}?token={token}` | Delete task | Account (Admin) |

## Data Models

### Account

```csharp
class Account : ModelObject {
    string Name;
    string EMail;
    string Password;
    bool Virgin;              // First login flag
    bool Administrator;
    int AvailableMinutes;
    bool DataAccess;
    bool SessionAccess;
    ContractType Contract;    // Expired, OneTime, FixedMinutes, etc.
}
```

### Token

```csharp
class Token : ModelObject {
    int AccountID;
    TokenType Type;           // Invalid, Internal, Account, Session, Data
    DateTime Created;
    DateTime Until;
    DateTime Used;
}
```

### Connection

```csharp
class Connection : ModelObject {
    int TokenID;
    int SessionID;
    ConnectionType Type;      // Unknown, Internal, Admin, Manager, Driver
    string Client;
    string Name;
    DateTime Created;
    DateTime Valid;
}
```

### Team

```csharp
class Team : ModelObject {
    int AccountID;
    string Name;
}
```

### Driver

```csharp
class Driver : ModelObject {
    int TeamID;
    string ForName;
    string SurName;
    string NickName;
}
```

### Session

```csharp
class Session : ModelObject {
    int TeamID;
    string Name;
    int Duration;
    string Track;
    string Car;
    bool Started;
    bool Finished;
    DateTime StartTime;
    DateTime FinishTime;
}
```

### Stint

```csharp
class Stint : ModelObject {
    int SessionID;
    int DriverID;
    int Nr;                   // Stint number
    int Lap;                  // Starting lap
}
```

### Lap

```csharp
class Lap : ModelObject {
    int SessionID;
    int StintID;
    int Nr;                   // Lap number
}
```

### Data Objects

```csharp
abstract class DataObject : ModelObject {
    int AccountID;
    DateTime Modified;
}

abstract class SimulatorObject : DataObject {
    string Simulator;
}

abstract class CarObject : SimulatorObject {
    string Car;
    string Track;
    string Driver;
}

abstract class TelemetryObject : CarObject {
    string Weather;
    int Temperature_Air;
    int Temperature_Track;
    float Fuel_Remaining;
    float Fuel_Consumption;
    string Compound;
    string Compound_Color;
    int Lap_Time;
}
```

## Contract Types

| Type | Description | Behavior |
|------|-------------|----------|
| `Expired` | No access | API access denied |
| `OneTime` | Single use | Deleted after minutes exhausted |
| `FixedMinutes` | Monthly reset | Reset to fixed amount |
| `AdditionalMinutes` | Monthly addition | Add fixed amount |
| `Unlimited` | No limits | Unlimited access |

## Connection Types

| Type | Description |
|------|-------------|
| `Unknown` | Unspecified |
| `Internal` | System internal |
| `Admin` | Administrator |
| `Manager` | Team manager |
| `Driver` | Team driver |

## Background Tasks

The Task Manager runs scheduled maintenance tasks:

| Task Type | Operations |
|-----------|------------|
| `Token` | Cleanup expired tokens |
| `Session` | Delete/cleanup/reset sessions |
| `Account` | Renew/delete accounts |

### Task Frequencies

| Frequency | Description |
|-----------|-------------|
| `Daily` | Runs once per day |
| `Weekly` | Runs once per week |
| `Monthly` | Runs once per month |

## Configuration

### Settings.json

```json
{
    "DBPath": ":local:|:memory:|/path/to/db",
    "TokenLifeTime": "seconds",
    "ConnectionLifeTime": "seconds (min 300)",
    "Accounts": [
        {
            "Name": "admin",
            "Password": "secret",
            "Minutes": 1000,
            "Session": true,
            "Data": true,
            "Administrator": true,
            "Reset": false
        }
    ]
}
```

### Database Paths

| Path | Description |
|------|-------------|
| `:memory:` | In-memory database |
| `:local:` | Local application data |
| `/path/to/db` | Specific file path |

## Service Managers

### TokenIssuer

- Creates and validates tokens
- Issues Session and Data tokens
- Manages connections
- Cleans up expired tokens/connections

### AccountManager

- Account CRUD operations
- Password management
- Contract management
- Minutes management
- Account renewal/deletion

### TeamManager

- Team CRUD operations
- Driver management
- Links to accounts

### SessionManager

- Session CRUD operations
- Stint and lap management
- Session attributes
- Session lifecycle (start/finish/clear)

### DataManager

- Telemetry data CRUD
- Query operations
- Attribute management
- Multi-table support

### TaskManager

- Background task scheduling
- Task execution
- Frequency management

## Example Workflows

### Login and Session Flow

```
1. Login
   GET /api/login?name=user&password=pass
   → Returns Account Token

2. Issue Session Token
   GET /api/login/token/Session?token={accountToken}
   → Returns Session Token

3. Connect
   GET /api/login/connect/session?token={sessionToken}&client=app&name=User&type=Driver
   → Returns Connection

4. Create/Join Team
   POST /api/team?token={sessionToken}
   Body: { "Name": "My Team" }
   → Returns Team

5. Start Session
   POST /api/session?token={sessionToken}&team={teamId}
   PUT /api/session/{id}/start?token={sessionToken}
```

### Data Synchronization Flow

```
1. Issue Data Token
   GET /api/login/token/Data?token={accountToken}
   → Returns Data Token

2. Query Existing Data
   PUT /api/data/query/TyresPressures?token={dataToken}
   Body: { "Simulator": "ACC", "Car": "ferrari_488" }
   → Returns matching records

3. Upload New Data
   POST /api/data/TyresPressures?token={dataToken}
   Body: { pressure data... }
   → Creates new record

4. Update Existing
   PUT /api/data/TyresPressures/{id}?token={dataToken}
   Body: { updated data... }
```

## Error Responses

| HTTP Code | Meaning |
|-----------|---------|
| 200 | Success |
| 400 | Bad request / Invalid parameters |
| 401 | Authentication required |
| 403 | Permission denied |
| 404 | Resource not found |
| 500 | Server error |

## Security Notes

- Passwords are stored in plaintext (no hashing in current implementation)
- Virgin flag enforces password change on first login
- Token expiration times are configurable
- Connection keep-alive mechanism extends validity
- Automatic cleanup of expired tokens/connections
- Account-scoped data access (DataManager filters by AccountID)
