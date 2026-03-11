# Desktop Pet (OpenClaw-lite) - macOS First

A local-first desktop pet built with Tauri 2 + React + Rust.

## Features (V1)

- Persona-based chat with short-term + long-term memory summary
- Todo CRUD + reminder scheduling
- Self-talk engine with cooldown and quiet hours
- Whitelisted system actions with confirmation and audit logging
- Local SQLite persistence

## Stack

- Frontend: React + TypeScript + Vite
- Desktop: Tauri 2 (Rust)
- Data: SQLite + SQLx migrations
- Scheduling: tokio-cron-scheduler

## Prerequisites

- Node.js 20+
- pnpm 10+
- Rust stable toolchain (`rustup`)
- Tauri prerequisites for macOS

## Quick Start

```bash
pnpm install
pnpm tauri dev
```

After startup:

- A small bouncing pet icon stays on desktop and can be dragged to move.
- Hover the pet icon to show a right-side quick menu.
- Quick menu includes chat/todo/controls/account/API/logout/quit actions.
- Click the icon (or quick menu items) to open the menu panel.
- Click menu items (`对话` / `待办` / `开关`) to enter features.

## Verification Commands

```bash
pnpm typecheck
pnpm test:rust
pnpm build
```

or run all checks:

```bash
pnpm check
```

## Build Release Binary

```bash
pnpm tauri build --no-bundle
```

Binary output:

```text
src-tauri/target/release/desktop_pet
```

## Runtime Notes

- If you need a custom local DB path, set `DESKPET_DB_PATH`, for example:
  - `DESKPET_DB_PATH=/absolute/path/desktop_pet.db pnpm tauri dev`
- `OPENAI_API_KEY` is optional. If unset, the app uses local fallback replies.

## Project Layout

- `frontend/` React UI
- `src-tauri/` Rust core + Tauri commands
- `src-tauri/migrations/` SQLite schema
- `docs/acceptance.md` acceptance criteria
- `tests/acceptance_checklist.md` manual regression checklist
