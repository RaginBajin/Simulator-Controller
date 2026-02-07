# Story 1.0: Project Foundation & Build Setup

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want the foundational project structure established,
so that all subsequent development has a consistent build and development environment.

## Acceptance Criteria

1. **Tauri 2.0 Project Scaffold**
   - Tauri 2.0 project created using `create-tauri-app` CLI (v2.6.0+)
   - React template selected with TypeScript
   - Tauri capabilities configured with minimal permissions (principle of least privilege)
   - Default "Hello Tauri" window renders successfully

2. **Cargo Workspace with 4 Crates**
   - Workspace includes: `src-tauri` (binary), `telemetry-engine` (lib), `ai-provider` (lib), `storage` (lib)
   - Dependency graph enforced: `src-tauri` depends on all 3 libs, `telemetry-engine` and `ai-provider` depend on `storage`, `storage` has no internal dependencies
   - `cargo build` completes successfully for all crates
   - `cargo test` runs successfully (even with no tests yet)

3. **Frontend Tooling: Tailwind CSS 4.x**
   - Tailwind CSS 4.x configured and processing during build
   - PostCSS configuration validated
   - Custom design tokens configured per UX spec:
     - Backgrounds: `bg-base` (#0A0A0F), `bg-surface` (#141419), `bg-elevated` (#1E1E26)
     - Text: `text-primary` (#F4F4F5), `text-secondary` (#B4B4BB), `text-muted` (#71717A)
     - Borders: `border-default` (#27272A), `border-subtle` (#1E1E26)
     - Accent: `accent-primary` (#F59E0B), `accent-hover` (#D97706)
     - Telemetry traces: `trace-best` (#22C55E), `trace-average` (#71717A), `trace-regression` (#EF4444)
     - Semantic: `success` (#22C55E), `warning-bg` (#F59E0B/10%), `warning-text` (#FBBF24), `error` (#EF4444), `info` (#3B82F6)
   - Inter and JetBrains Mono fonts loaded

4. **shadcn/ui Component System**
   - shadcn/ui initialized with components directory (owned source, not npm dependency)
   - Radix UI primitives configured
   - Example component (Button) renders successfully in dev mode
   - Dark theme set as default (no light mode at MVP)

5. **Local Dev Environment**
   - `npm run tauri dev` (or equivalent) starts Vite dev server on port 1420
   - Tauri window opens with hot-reload enabled
   - Rust changes trigger recompilation
   - Frontend changes trigger hot-reload without full rebuild
   - Build completes in <30 seconds for incremental changes

## Tasks / Subtasks

- [ ] Task 1: Initialize Tauri 2.0 project (AC: #1)
  - [ ] 1.1 Run `sh <(curl https://create.tauri.app/sh)` — select React, TypeScript, npm as package manager
  - [ ] 1.2 Verify default Tauri window renders "Hello Tauri" text
  - [ ] 1.3 Configure `src-tauri/capabilities/default.json` with minimal permissions
  - [ ] 1.4 Verify `npm run tauri dev` launches Vite on port 1420 + opens Tauri window

- [ ] Task 2: Set up Cargo workspace with 4 crates (AC: #2)
  - [ ] 2.1 Convert project to Cargo workspace — add `[workspace]` to root `Cargo.toml` or `src-tauri/Cargo.toml`
  - [ ] 2.2 Create `crates/storage/` library crate with `Cargo.toml` + `src/lib.rs`
  - [ ] 2.3 Create `crates/telemetry-engine/` library crate — add `storage` as dependency
  - [ ] 2.4 Create `crates/ai-provider/` library crate — add `storage` as dependency
  - [ ] 2.5 Add all 3 library crates as dependencies of `src-tauri`
  - [ ] 2.6 Verify `cargo build` succeeds for entire workspace
  - [ ] 2.7 Verify `cargo test` runs without errors

- [ ] Task 3: Configure Tailwind CSS 4.x with design tokens (AC: #3)
  - [ ] 3.1 Install Tailwind CSS 4.x and PostCSS dependencies
  - [ ] 3.2 Create `tailwind.config.ts` with custom theme extending design tokens (all color tokens from UX spec)
  - [ ] 3.3 Configure `postcss.config.cjs`
  - [ ] 3.4 Create `src/styles/globals.css` with Tailwind directives and CSS custom properties for design tokens
  - [ ] 3.5 Install and configure Inter font (from Google Fonts or local)
  - [ ] 3.6 Install and configure JetBrains Mono font (from Google Fonts or local)
  - [ ] 3.7 Verify Tailwind processes successfully during `npm run tauri dev`

- [ ] Task 4: Initialize shadcn/ui (AC: #4)
  - [ ] 4.1 Run `npx shadcn@latest init` — select default style, dark theme, CSS variables
  - [ ] 4.2 Configure `components.json` for project paths
  - [ ] 4.3 Add Button component: `npx shadcn@latest add button`
  - [ ] 4.4 Render Button in main App.tsx to verify dark theme + amber accent
  - [ ] 4.5 Set dark mode as default (class-based or CSS variable-based)

- [ ] Task 5: Configure project structure per architecture (AC: #1, #2, #5)
  - [ ] 5.1 Create frontend directory structure:
    - `src/features/` (empty feature dirs for future)
    - `src/components/ui/` (shadcn components land here)
    - `src/hooks/`
    - `src/lib/` (types.ts, errors.ts, constants.ts, tauri.ts, format.ts)
    - `src/state/`
    - `src/assets/icons/`
  - [ ] 5.2 Create Rust source structure under `src-tauri/src/`:
    - `commands/mod.rs` (empty command modules)
    - `events.rs`
    - `error.rs`
    - `state.rs`
  - [ ] 5.3 Create `crates/storage/src/` subdirectories: `sqlite/`, `parquet/`
  - [ ] 5.4 Create `crates/telemetry-engine/src/` subdirectories: `irsdk/`
  - [ ] 5.5 Create `crates/ai-provider/src/` subdirectories: `adapters/`, `prompt/`

- [ ] Task 6: Verify dev environment works end-to-end (AC: #5)
  - [ ] 6.1 Run `npm run tauri dev` — confirm Vite starts, Tauri window opens
  - [ ] 6.2 Make a frontend change (edit text) — verify hot-reload works
  - [ ] 6.3 Make a Rust change (add a log statement) — verify recompilation triggers
  - [ ] 6.4 Run `cargo build` from workspace root — all crates compile
  - [ ] 6.5 Run `cargo test` from workspace root — all tests pass (even if empty)
  - [ ] 6.6 Verify Tailwind classes render correctly in the Tauri window (dark bg, amber accent)

## Dev Notes

### Architecture Compliance

**Starter Template:** Use `create-tauri-app` official CLI. Do NOT use community starters.
```bash
sh <(curl https://create.tauri.app/sh)
```
Prompts to select: React, TypeScript, npm.

**Cargo Workspace Layout:**
```
project-root/
├── Cargo.toml           # workspace root (members = ["src-tauri", "crates/*"])
├── src-tauri/
│   ├── Cargo.toml       # binary crate, depends on storage, telemetry-engine, ai-provider
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── commands/mod.rs
│       ├── events.rs
│       ├── error.rs
│       └── state.rs
├── crates/
│   ├── storage/         # LEAF crate — no internal dependencies
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── sqlite/mod.rs
│   │       └── parquet/mod.rs
│   ├── telemetry-engine/  # depends on storage
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── irsdk/mod.rs
│   └── ai-provider/     # depends on storage
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── adapters/mod.rs
│           └── prompt/mod.rs
```

**CRITICAL: Dependency Graph Rules:**
- `storage` = leaf crate. ZERO workspace dependencies. Only external crates allowed.
- `telemetry-engine` depends on `storage` only.
- `ai-provider` depends on `storage` only.
- `src-tauri` depends on all three library crates.
- NEVER add `telemetry-engine` ↔ `ai-provider` cross-dependency.

### Frontend Architecture

**Vite Configuration:** Tauri starter configures Vite. Dev server runs on port 1420 by default.

**React Version:** 19.2.x (per architecture doc). Verify version in `package.json` after scaffold.

**Frontend Directory Structure:**
```
src/
├── main.tsx
├── App.tsx
├── routes.tsx
├── styles/
│   └── globals.css          # Tailwind directives + design tokens
├── assets/
│   └── icons/
├── components/
│   └── ui/                  # shadcn/ui components land here
├── features/                # Feature-based organization (empty for now)
├── hooks/
├── lib/
│   ├── tauri.ts             # Tauri invoke wrappers
│   ├── types.ts             # Shared TypeScript types
│   ├── errors.ts            # Error mapping
│   ├── format.ts            # Formatting utilities
│   └── constants.ts         # App constants
├── state/                   # Zustand stores (future)
└── tests/
    └── setup.ts
```

### Design Token Configuration

**Tailwind CSS 4.x Configuration:**

All design tokens go in `tailwind.config.ts` under `theme.extend.colors`:

```typescript
// tailwind.config.ts
export default {
  darkMode: 'class',
  content: ['./src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        base: '#0A0A0F',
        surface: '#141419',
        elevated: '#1E1E26',
        'text-primary': '#F4F4F5',
        'text-secondary': '#B4B4BB',
        'text-muted': '#71717A',
        'border-default': '#27272A',
        'border-subtle': '#1E1E26',
        'accent-primary': '#F59E0B',
        'accent-hover': '#D97706',
        'trace-best': '#22C55E',
        'trace-average': '#71717A',
        'trace-regression': '#EF4444',
        success: '#22C55E',
        warning: '#FBBF24',
        error: '#EF4444',
        info: '#3B82F6',
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      borderRadius: {
        sm: '4px',
        md: '6px',
        lg: '8px',
      },
      spacing: {
        '1': '4px',
        '2': '8px',
        '3': '12px',
        '4': '16px',
        '6': '24px',
        '8': '32px',
        '12': '48px',
      },
    },
  },
}
```

**Font Loading:** Use `@font-face` in `globals.css` or import from Google Fonts CDN. For a desktop app (Tauri), prefer bundling fonts locally in `src/assets/fonts/` to avoid network dependency.

### shadcn/ui Setup

**Initialization command:**
```bash
npx shadcn@latest init
```

**Key selections during init:**
- Style: Default
- Base color: Neutral (we override with our tokens)
- CSS variables: Yes
- `components.json` paths aligned with `src/components/ui/`

**Dark theme default:** Set `<html class="dark">` in `index.html` or apply via CSS variable theming in `globals.css`. No light mode toggle needed at MVP.

### Naming Conventions (Enforced from Day 1)

| Zone | Convention | Example |
|------|-----------|---------|
| Rust functions | `snake_case` | `get_session_data` |
| Rust types | `PascalCase` | `SessionData` |
| Rust constants | `SCREAMING_SNAKE_CASE` | `MAX_CAPTURE_RATE` |
| TypeScript functions/vars | `camelCase` | `getSessionData` |
| TypeScript types/interfaces | `PascalCase` | `SessionData` |
| React components | `PascalCase` | `SessionCard` |
| Frontend file names | `kebab-case` | `session-card.tsx` |
| IPC commands | `snake_case` | `get_session_data` |
| IPC events | `domain:action` kebab-case | `session:state-changed` |
| DB tables | `snake_case` plural | `sessions` |
| DB columns | `snake_case` | `session_id` |

### Tauri Capabilities (Minimal Permissions)

Configure `src-tauri/capabilities/default.json` with only the permissions needed for this story:
- `core:default` — basic window operations
- No file system, shell, or network permissions yet

Future stories will add permissions incrementally as features require them.

### Testing Setup

**Rust:** `cargo test` should pass with default generated tests. Each crate's `lib.rs` should include a basic module structure.

**Frontend:** No test framework configured in this story. Vitest setup deferred to a future story when actual components need testing.

### Project Structure Notes

- This story establishes Architecture Implementation Sequence steps 1-3 from the architecture document
- ALL subsequent stories depend on this infrastructure
- The project structure follows feature-based frontend + crate-based Rust organization
- File naming follows `kebab-case` for frontend, standard Rust conventions for backend

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Starter Template Evaluation]
- [Source: _bmad-output/planning-artifacts/architecture.md#Core Architectural Decisions]
- [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules]
- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 1.0: Project Foundation & Build Setup]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Design System Foundation]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Visual Design Foundation]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
