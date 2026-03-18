# Milestones

Status key: ✅ done · 🔧 in progress · ⬜ not started

---

## M1 – Viewer core  ✅

Goal: open any PDF, navigate pages, draw with the mouse, see annotations live.

| Task | Status | Module |
|------|--------|--------|
| pdfium-render integration | ✅ | `pdf/` |
| Page-to-egui-texture renderer with cache | ✅ | `renderer/` |
| egui app skeleton (menu, page nav, zoom) | ✅ | `app/` |
| In-memory annotation store | ✅ | `annotations/` |
| Mouse ink drawing on page | ✅ | `app/` |
| Automerge CRDT doc (add_stroke, apply_change) | ✅ | `crdt/` |
| Binary wire protocol types | ✅ | `protocol/` |
| Tablet WebSocket server (compile, not wired) | ✅ | `tablet_server/` |
| iPad PWA (draw, batch send, local ink) | ✅ | `tablet-ui/` |

**To verify M1 works:**
```bash
cargo build
# Place pdfium.dll / libpdfium.so next to binary or in project root
cargo run -- some_file.pdf
```

---

## M2 – Live iPad sync  ⬜

Goal: draw on the iPad, see strokes appear on the laptop in < 16 ms.

| Task | Status | Notes |
|------|--------|-------|
| Wire tablet server into app event loop | ⬜ | poll `TabletEvent` channel each frame |
| Normalised → screen coordinate mapping | ⬜ | StylusSample x/y [0,1] → page pixels |
| Page snapshot JPEG encoding & push | ⬜ | `image` crate, send on page navigation |
| Tablet server also serves PWA (HTTP GET /) | ⬜ | `hyper` or manual HTTP over TCP |
| USB iproxy setup docs | ⬜ | see `context/ipad-connectivity.md` |
| Pressure → stroke width on laptop side | ⬜ | `pressure * 4.0 + 1.0` |

**Design note for agents:** The mpsc channel receiver is created by
`tablet_server::spawn()`.  Store it in `InkstoneApp` and call
`rx.try_recv()` at the top of `update()`.  Convert each `TabletEvent` to
an `InkStroke` and call `self.annotations.add_stroke()`.

---

## M3 – Persistence  ⬜

Goal: annotations survive app restarts.  No PDF modification yet.

| Task | Status | Notes |
|------|--------|-------|
| Save Automerge doc to `<pdf_stem>.ink` | ⬜ | `crdt::CrdtDoc::save()` |
| Load `.ink` file on PDF open | ⬜ | `crdt::CrdtDoc::load()` |
| Replay CRDT doc into AnnotationStore on load | ⬜ | walk automerge "strokes" list |
| Undo / redo stack | ⬜ | Automerge branch + merge |

---

## M4 – Highlights & export  ⬜

Goal: highlight text, add text notes, export annotated PDF.

| Task | Status | Notes |
|------|--------|-------|
| Text selection on PDF page | ⬜ | pdfium text layer |
| Highlight annotation type | ⬜ | add `Highlight` variant to `AnnotationStore` |
| Sticky-note annotation type | ⬜ | `Note` variant |
| PDF export with embedded annotations | ⬜ | pdfium annotation API |

---

## M5 – Polish  ⬜

| Task | Status |
|------|--------|
| Native file open dialog (`rfd` crate) | ⬜ |
| Recent files list | ⬜ |
| Keyboard shortcuts | ⬜ |
| Dark / light theme toggle | ⬜ |
| Page thumbnail sidebar | ⬜ |
| Annotation colour picker on toolbar | ⬜ |
| Eraser tool | ⬜ |
