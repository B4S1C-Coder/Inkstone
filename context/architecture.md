# Architecture

## System overview

```mermaid
graph TB
    subgraph Laptop["Windows laptop (Rust binary)"]
        direction TB
        PDF["pdf/\npdfium-render"]
        RND["renderer/\nPageRenderer"]
        ANN["annotations/\nAnnotationStore"]
        CRDT["crdt/\nCrdtDoc (Automerge)"]
        APP["app/\nInkstoneApp (egui)"]
        SRV["tablet_server/\nTokio WS server"]
        PROTO["protocol/\nwire types (postcard)"]

        PDF --> RND
        RND --> APP
        ANN --> APP
        CRDT --> APP
        SRV --> PROTO
        APP --> SRV
    end

    subgraph iPad["iPad (Safari PWA)"]
        PWA["tablet-ui/index.html\nCanvas + PointerEvent"]
    end

    SRV <-->|"USB / LAN\nWS binary frames\n~30 bytes/sample"| PWA
```

## Module contracts

| Module | Inputs | Outputs | Must not depend on |
|--------|--------|---------|-------------------|
| `pdf/` | file path | `PdfDocument` | all other inkstone modules |
| `renderer/` | `&PdfDocument`, page idx, zoom | `egui::TextureId` | annotations, crdt, protocol |
| `annotations/` | stroke data | `&[InkStroke]` | all other inkstone modules |
| `crdt/` | stroke data | change bytes (`Vec<u8>`) | all other inkstone modules |
| `protocol/` | typed messages | encoded `Vec<u8>` | all other inkstone modules |
| `tablet_server/` | TCP stream | `TabletEvent` channel | app, pdf, renderer, annotations, crdt |
| `app/` | all of the above | egui frame | nothing external |

## Data flow: annotation lifecycle

```mermaid
sequenceDiagram
    participant Pencil as Apple Pencil
    participant PWA as iPad PWA
    participant WS as WebSocket
    participant SRV as tablet_server
    participant APP as app (GUI thread)
    participant ANN as annotations
    participant CRDT as crdt

    Pencil->>PWA: pointerdown / pointermove
    PWA->>PWA: render ink locally (immediate)
    PWA->>WS: StrokeBatch (binary, every 8 ms)
    WS->>SRV: binary frame
    SRV->>APP: TabletEvent::StrokeBatch (mpsc channel)
    APP->>ANN: add_stroke()
    APP->>CRDT: add_stroke() → ChangeBytes
    APP->>APP: request_repaint()
    Pencil->>PWA: pointerup
    PWA->>WS: StrokeEnd
    WS->>SRV: binary frame
    SRV->>APP: TabletEvent::StrokeEnd
```

## Rendering pipeline

```mermaid
flowchart LR
    PDF["PdfDocument\n.render_page_rgba()"]
    RGBA["Raw RGBA buffer\n(pdfium → CPU)"]
    TEX["egui ColorImage\n→ GPU texture"]
    DISP["egui display\n(GPU blit)"]
    INK["Ink overlay\negui::Painter"]

    PDF --> RGBA --> TEX --> DISP
    INK --> DISP
```

The renderer caches the last GPU texture.  It only re-rasterises when the
page index **or** zoom changes.  This means:
- Scrolling within a page: 0 CPU work, 1 GPU blit.
- Page turn: one pdfium render (~5–20 ms), then cached.
- Zoom change: one pdfium render at new resolution.

## Threading model

```mermaid
graph LR
    GUI["GUI thread\n(egui / wgpu)"]
    TAB["Tablet thread\n(Tokio runtime)"]
    CH["mpsc channel\nTabletEvent"]

    TAB -->|"send()"| CH
    CH -->|"try_recv() each frame"| GUI
```

The GUI thread never blocks.  It polls the mpsc channel at the start of each
frame and applies any pending stroke data before rendering.

## Dependencies

```mermaid
graph TD
    pdfium-render --> pdf
    egui --> app
    eframe --> app
    automerge --> crdt
    postcard --> protocol
    tokio --> tablet_server
    tokio-tungstenite --> tablet_server
    protocol --> tablet_server
    pdf --> renderer
    renderer --> app
    annotations --> app
    crdt --> app
    tablet_server --> app
```
