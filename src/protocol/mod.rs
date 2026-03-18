// Binary wire protocol between the laptop server and the iPad PWA.
//
// Design goals:
//   Tiny messages – a single stylus sample is < 40 bytes
//   No schema versioning pain yet – postcard + serde handles it
//   WebSocket binary frames (not text/JSON) to minimise overhead
//
// Message flow:
//
//   iPad → Laptop:  StrokeSample (batched, ~8 ms intervals)
//                   StrokeEnd
//                   Ping
//
//   Laptop → iPad:  PageSync (JPEG thumbnail of current page)
//                   AckChange (sequence number echo)
//                   Pong
//
// Using postcard for serialisation: it is no_std compatible, very compact,
// and fast enough that serialisation is never the bottleneck.

use serde::{Deserialize, Serialize};

// iPad -> Laptop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeBatch {
    pub stroke_id: u32,             // Monotonically increasing Stroke ID (reset tp 0 after each StrokeEnd)
    pub samples: Vec<StylusSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylusSample {
    pub x: f32,         // x pos in [0.0, 1.0] relative to page width
    pub y: f32,         // y pos in [0.0, 1.0] relative to page width
    pub pressure: f32,  // Pressure in [0.0, 1.0]. 0.5 for mouse/touch
    pub tilt: f32,      // Tilt in degrees from vertical (0 = perpendicular to screen)
    pub t_ms: u32,      // Milliseconds since stroke started.
}

// Sent when the stylus is lifted -- signals end of stroke
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeEnd {
    pub stroke_id: u32,
}

// Laptop -> iPad
/// Compressed JPEG Snapshot of current page for iPad to display under the ink overlay
/// Sent when user navigates to a new page or zoom changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSnapshot {
    pub page_idx: u32,
    pub width: u32,
    pub height: u32,
    pub jpeg: Vec<u8>, // JPEG Encoded bytes
}

/// Acknowledeges that change #seq was applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckChange {
    pub seq: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TabletMessage {
    StrokeBatch(StrokeBatch),
    StrokeEnd(StrokeEnd),
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LaptopMessage {
    PageSnapshot(PageSnapshot),
    AckChange(AckChange),
    Pong,
}

pub fn encode_tablet_msg(msg: &TabletMessage) -> anyhow::Result<Vec<u8>> {
    postcard::to_allocvec(msg).map_err(|e| anyhow::anyhow!("encode error: {e}"))
}

pub fn decode_tablet_msg(bytes: &[u8]) -> anyhow::Result<TabletMessage> {
    postcard::from_bytes(bytes).map_err(|e| anyhow::anyhow!("decode error: {e}"))
}

pub fn encode_laptop_msg(msg: &LaptopMessage) -> anyhow::Result<Vec<u8>> {
    postcard::to_allocvec(msg).map_err(|e| anyhow::anyhow!("encode error: {e}"))
}

pub fn decode_laptop_msg(bytes: &[u8]) -> anyhow::Result<LaptopMessage> {
    postcard::from_bytes(bytes).map_err(|e| anyhow::anyhow!("decode error: {e}"))
}
