# Contributing to Pitwall

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) v20+
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/) v2

### Platform-specific

**Windows (primary):**
- Visual Studio Build Tools with C++ workload
- WebView2 (included in Windows 10/11)

**macOS (development):**
- Xcode Command Line Tools

## Development Setup

```bash
# Install frontend dependencies
npm ci

# Run the development server (Tauri + Vite)
npm run tauri dev
```

## Running Tests

```bash
# Rust tests (all crates)
cargo test --workspace

# TypeScript type check
npm run typecheck

# Frontend build verification
npm run build

# Check Rust formatting
cargo fmt --check

# Run Clippy lints
cargo clippy --workspace -- -D warnings
```

## Code Quality

All checks run automatically in CI on every push and pull request:

1. `cargo fmt --check` -- Rust formatting
2. `cargo clippy --workspace -- -D warnings` -- Rust lints (zero warnings)
3. `cargo test --workspace` -- Rust unit and integration tests
4. `npm run typecheck` -- TypeScript type checking
5. `npm run build` -- Frontend build
6. Migration safety check -- no destructive SQL allowed

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes and ensure all checks pass locally
3. Push your branch and open a pull request
4. Fill out the PR template (summary, testing, breaking changes)
5. All CI checks must pass before merge

### Branch Protection

The `main` branch requires:
- All CI status checks to pass
- Branch to be up to date before merging

## Database Migrations

Migrations live in `crates/storage/migrations/` and are applied automatically at startup.

### Migration Policy

- **Additive only** at MVP: `CREATE TABLE`, `ALTER TABLE ADD COLUMN`
- **No destructive migrations**: `DROP TABLE`, `DROP COLUMN`, `DELETE FROM`, `TRUNCATE` are blocked by CI
- Number migrations sequentially: `001_`, `002_`, etc.

## Release Process

Releases are triggered by pushing a semantic version tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

This triggers the release workflow, which builds the Windows installer and creates a GitHub Release with the artifact attached.

### Version Tags

- Use semantic versioning: `v1.0.0`, `v0.1.0-beta`
- Release workflow triggers on any tag matching `v*`
