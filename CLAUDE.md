# Orc

macOS native universal AI agent harness. Rust core + Tauri 2 + React.

## Stack

- Tauri 2 (macOS 14+)
- Rust (cargo workspace, edition 2024)
- React 19 + TypeScript + Vite 8
- Tailwind CSS 4
- zustand (state management)
- xterm.js (terminal emulation)

## Structure

```
crates/        rust workspace members
  core/        pty, provider, sandbox (tauri-agnostic)
  agent/       config, runtime, session, engine, tool, skill, hook, memory
  ffi/         (reserved)
src/           react frontend
  entities/    domain types (panel, tab, workspace, message)
  features/    business logic (terminal, agent-chat, split-panel)
  widgets/     ui components (panel, sidebar, tab-bar)
src-tauri/     tauri app (thin wrapper over core/agent)
```

## Commands

```
cargo check                    # check rust workspace
cargo test -p orc-core         # run core tests
pnpm dev                       # start vite dev server
pnpm tauri dev                 # start tauri dev
```

## UI Hierarchy

Workspace → Tab → Panel (recursive tree) → Surface (terminal/agent/browser)

- Panel is a tree: leaf (has Surface) or split (has children)
- Surface is independent from Panel — swappable between slots

## Architecture

- Manager + EventHandler trait pattern (PtyManager, AgentManager)
- CompletionProvider trait for multi-model (Anthropic/OpenAI/Google/local)
- "provider/model" string format for model routing
- Per-agent model + fallback chain + in-loop escalation
- Tool trait with CancellationToken propagation
- Built-in tools (~8) + MCP for extension
- macOS Seatbelt sandbox for command execution
- Hybrid event log (append-only SQLite) + snapshots

## Gotchas

- `target/` is gitignored — cargo build output
- core crate has no tauri dependency — pure rust library
- agent crate uses tokio async, core pty uses std::thread sync
- sync-async bridge: spawn_blocking for PtyManager access from async context
