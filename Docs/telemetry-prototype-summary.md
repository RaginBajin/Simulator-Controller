# Telemetry Prototype: Build & Analysis Summary

## Session Overview

**Date:** January 31, 2026
**Track:** Lime Rock Park (Classic)
**Car:** Mazda MX-5 Cup
**Tools Built:** `iracing_telemetry_logger.py`, `llm_analysis.py`
**Location:** `/Users/jbajin/Library/CloudStorage/GoogleDrive-josephbajin@gmail.com/My Drive/telemetry/`

---

## What Was Built

### 1. Telemetry Logger (`iracing_telemetry_logger.py` — 472 lines)

A Python script that connects to iRacing's shared memory API via `irsdk` and captures real-time telemetry data.

**Capabilities:**
- Configurable sample rate: 1-60Hz (default 30Hz)
- Captures: throttle, brake, clutch, steering, speed, RPM, G-forces (lateral/longitudinal), lap position, gear
- Automatic lap detection and per-lap summary computation
- Brake application counting (transitions from <0.1 to ≥0.1 pressure)
- JSONL output with typed records: `metadata`, `sample`, `lap_summary`, `session_summary`
- Auto-generates timestamped filenames: `YYYYMMDD_HHMMSS_Track_Car.jsonl`

**Data Format (JSONL typed records):**
- **metadata**: Session start info, track, car, sample rate
- **sample**: Individual telemetry frame (throttle, brake, steering, speed, etc.)
- **lap_summary**: Per-lap aggregates (avg/max/min for each channel, brake application count, lap time)
- **session_summary**: Overall session stats, total laps, total samples

### 2. LLM Analysis Script (`llm_analysis.py` — 293 lines)

A Python script that reads JSONL telemetry data and generates structured LLM prompts for coaching analysis.

**Prompt Templates:**
- **MX-5-specific template**: Understands MX-5 Cup characteristics (low power, momentum car, threshold braking sensitivity)
- **Braking-specific template**: Analyzes initial brake application patterns, peak pressure utilization, trail braking correlation with steering, brake release timing relative to apex

**Analysis Approach:**
- Reads lap summaries from JSONL
- Computes cross-lap statistics (best/worst/average)
- Formats structured data into coaching prompts
- Outputs prompts ready for Claude API or manual paste

### 3. InfluxDB Export (`influxdb_export.py`)

Export script for pushing telemetry data to InfluxDB (exists but not detailed in session).

---

## Test Session Results

### Capture Stats
- **Laps recorded:** 25 (Laps 41-65)
- **Total samples:** 28,000+
- **Complete valid laps:** 10
- **Partial laps:** 15 (from mid-session resets)
- **Logger started at:** Lap 41 (first-ever test of the code)

### Performance Analysis

| Metric | Value |
|--------|-------|
| Best Lap | 58.32s (Lap 60) |
| VRS Coach Reference | ~57.5s |
| Gap to Reference | 0.82s |
| Consistency (std dev) | 0.65s |
| Avg Brake Pressure | 37% |
| Max Brake Pressure | Never exceeded 90% |

### Key Finding: Under-Braking

The primary performance gap is **insufficient brake pressure**. Analysis revealed:

- **Average initial brake application:** 0.77 (77%)
- **Best lap (60) initial brake:** 0.90 (90%) — significantly higher
- **Best lap brake release rate:** 1.50/s vs 0.68/s average — faster, more decisive release
- Joe is leaving ~10-15% brake pressure on the table at every brake zone

### Best Lap Breakdown (Lap 60 — 58.32s)

What made this lap faster:
1. **Higher initial brake pressure** (0.90 vs 0.77 avg) — committed to braking harder
2. **Faster brake release** (1.50/s vs 0.68/s) — quicker transition to trail braking
3. **Better trail braking execution** — smoother handoff from braking to cornering

### Partial Lap Insight

Joe corrected the assumption that partial laps were noise. **Partial laps contain valuable diagnostic data:**
- Lap 45 (partial) showed throttle oversteer at Turn 4 — 100% throttle input mid-corner
- This reveals where problems occur that force resets
- Product requirement: capture and analyze ALL data including incomplete laps

---

## Trail Braking Technique Discussion

The session included detailed analysis of proper trail braking technique:

### Two-Phase Brake Release
1. **Quick controlled release** (0.3-0.5s): From peak pressure down to ~30% at turn-in point
2. **Gradual trail** (0.5-1.0s): From 30% down to 0% through the apex

### Why This Matters for the Product
- Trail braking is the single biggest performance differentiator for amateur sim racers
- The telemetry data can precisely measure both phases
- AI coaching can identify which phase needs work per corner
- Session-over-session tracking shows improvement in trail braking execution

---

## Implications for AI Race Team Product

### Validated Concepts
1. **Core loop works**: Capture telemetry → structured data → LLM analysis → actionable coaching
2. **30Hz is sufficient** for meaningful brake/throttle analysis
3. **JSONL with typed records** is a clean, parseable format for telemetry storage
4. **Per-lap summaries** provide the right granularity for LLM coaching context
5. **Brake application counting** is a useful derived metric
6. **Partial lap data has value** — must not be discarded

### Technical Decisions Informed
- **Data format**: JSONL with typed records (metadata/sample/lap_summary/session_summary) is proven
- **Sample rate**: 30Hz default is good balance of detail vs file size
- **Channel set**: throttle, brake, clutch, steering, speed, RPM, G-forces, lap position, gear covers core needs
- **Analysis granularity**: Per-lap summaries are the right unit for coaching prompts
- **Car-specific prompts**: MX-5 template approach validates need for car-specific coaching knowledge

### What the Product Must Do Better Than This Prototype
- **Automatic analysis** (no manual prompt generation)
- **Session-over-session memory** (track improvement over time)
- **Corner-by-corner breakdown** (not just lap-level stats)
- **Visual overlay** (brake/throttle traces on track map)
- **Real-time feedback** (during session, not just post-session)
- **Multi-car support** (not hardcoded to MX-5)
- **Setup correlation** (how setup changes affect telemetry patterns)

---

## Files on Disk

```
/Users/jbajin/Library/CloudStorage/GoogleDrive-josephbajin@gmail.com/My Drive/telemetry/
├── influxdb_export.py
├── iracing_telemetry_logger.py    (472 lines - telemetry capture)
├── llm_analysis.py                (293 lines - LLM prompt generation)
├── README.md.gdoc
├── requirements.gdoc
└── telemetry/
    └── 20260131_212041_Lime_Rock_Park_Mazda_MX-5_Cup.jsonl
```
