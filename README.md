# inkstone

A fast, battery-efficient PDF viewer for Windows with real-time iPad annotation
support via Apple Pencil.

## Features (planned)

- [x] PDF rendering via pdfium (same engine as Chrome)
- [x] Smooth page navigation and zoom
- [x] Freehand ink annotations (mouse)
- [x] CRDT annotation model (Automerge)
- [x] iPad PWA drawing surface
- [x] WebSocket wire protocol (binary, ~30 bytes/sample)
- [ ] USB transport via iproxy (M2)
- [ ] Ink strokes from iPad appear on laptop in real-time (M2)
- [ ] Page snapshot pushed to iPad on navigation (M2)
- [ ] Annotation persistence (.ink sidecar files) (M3)
- [ ] Highlights and text notes (M3)
- [ ] PDF export with embedded annotations (M4)

## Requirements

- Windows 10/11 (or Linux for development)
- Rust 1.77+
- pdfium shared library (see below)

## Getting the pdfium library

pdfium-render needs the Pdfium binary alongside your executable.

**Windows:**
```powershell
# Download the prebuilt DLL from the pdfium-binaries project
# https://github.com/bblanchon/pdfium-binaries/releases
# Place pdfium.dll next to inkstone.exe  (or in the project root for `cargo run`)
```

**Linux / WSL2 (for development):**
```bash
# Debian/Ubuntu
sudo apt install libpdfium-dev
# or download from https://github.com/bblanchon/pdfium-binaries/releases
# and set LD_LIBRARY_PATH
```

## Building

```bash
git clone <repo>
cd inkstone
cargo build              # debug
cargo build --release    # optimised
```

## Running

```bash
# Open a PDF
cargo run -- notes.pdf

# Open a PDF and start the iPad tablet server on port 9001
cargo run -- notes.pdf --tablet
```

## iPad setup

1. Make sure your laptop and iPad are on the same network, **or** connect via
   USB and run `iproxy 9001 9001` (see `context/ipad-connectivity.md`).
2. Open Safari on the iPad and navigate to `http://<laptop-ip>:9001`.
   (The tablet server also serves the PWA on the same port – M2 feature.)
3. Draw with Apple Pencil.  Strokes appear on the laptop in < 16 ms.

## Project structure

See `AGENTS.md` for a full module map and design rules.
See `context/` for detailed architecture, milestone, and protocol docs.

## Licence

MIT
