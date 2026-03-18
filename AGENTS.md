# inkstone – agent / tool entry point

> Read this file first. It tells you what the project is, what exists, and
> where to look for deeper context.

## What is inkstone?

A fast, battery-efficient PDF viewer written in Rust for Windows, with a companion
iPad PWA that lets you annotate PDFs in real-time using an Apple Pencil.
Strokes drawn on the iPad appear on the laptop display in under 16 ms via a
local WebSocket connection (USB-preferred, LAN fallback).

## Quick orientation

```
inkstone/
├── src/
│   ├── main.rs            ← entry point, CLI parsing
│   ├── app/               ← egui application loop & top-level state
│   ├── pdf/               ← PdfDocument (pdfium-render wrapper)
│   ├── renderer/          ← page-to-egui-texture cache (PageRenderer)
│   ├── annotations/       ← in-memory annotation store (InkStroke etc.)
│   ├── crdt/              ← Automerge wrapper (CrdtDoc)
│   ├── protocol/          ← postcard wire types (TabletMessage, LaptopMessage)
│   └── tablet_server/     ← Tokio WebSocket server (spawned with --tablet)
├── tablet-ui/
│   └── index.html         ← iPad PWA (self-contained, no build step)
├── context/               ← detailed design docs (read these for any module)
│   ├── architecture.md
│   ├── milestones.md
│   ├── data-model.md
│   └── ipad-connectivity.md
├── Cargo.toml
├── AGENTS.md              ← YOU ARE HERE
└── README.md
```

## Technology choices (do not change without context/architecture.md)

| Concern | Crate / tech | Why |
|---|---|---|
| PDF rendering | `pdfium-render` | Chrome's engine, best fidelity |
| GUI | `egui` + `eframe` | Immediate mode, wgpu backend, low overhead |
| CRDT | `automerge` | Established, binary ops, peer-reviewed |
| Wire format | `postcard` | Tiny binary, no_std, fast |
| Async | `tokio` | Tablet server only; GUI is sync |
| Logging | `tracing` | Structured, low cost when disabled |

## Module boundaries (strict)

- `pdf/` must not import from `renderer/`, `annotations/`, `crdt/`, or `app/`.
- `renderer/` imports from `pdf/` only.
- `annotations/` has no imports from other inkstone modules.
- `crdt/` has no imports from other inkstone modules.
- `protocol/` has no imports from other inkstone modules.
- `tablet_server/` imports from `protocol/` only.
- `app/` is the only module allowed to import from all others.

## How to build & run

```bash
# Debug build (faster compile, readable panics)
cargo build

# Run with a PDF
cargo run -- path/to/file.pdf

# Run with tablet server enabled
cargo run -- path/to/file.pdf --tablet

# Release build (ship this)
cargo build --release
```

See context/milestones.md for what is implemented vs stubbed.

## Current milestone: M1 (viewer only)

The viewer opens PDFs, renders pages, and accepts mouse-drawn annotations.
The tablet server compiles but is not yet wired into the GUI event loop.
See context/milestones.md for M2 and M3 tasks.

## For agents: rules of engagement

1. **Read the relevant `context/*.md` before editing any module.**
2. **Do not add dependencies without updating Cargo.toml and context/architecture.md.**
3. **Keep module boundaries.** If you need cross-module data, use a channel or a shared type in `protocol/`.
4. **Every public function needs a doc-comment.**  `///` not `//`.
5. **Do not use `unwrap()` in library code.** Use `?` and `anyhow::Result`.
6. **Do not use `unsafe` without a `// SAFETY:` comment explaining why it is sound.**
