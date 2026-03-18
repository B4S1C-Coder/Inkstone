// Design Rule: this module must not know about egui or pdfium.
// The only egui type that sneaks in (Pos2, Color32) is used as a plain
// data carrier.

// M1: plain Vec storage (fast, no persistence).
// M2: back this with Automerge CRDT so every mutation produces an Op
// that can be broadcast to the iPad.

use egui::{Color32, Pos2};
use std::collections::HashMap;


#[derive(Debug, Clone)]
pub struct InkStroke {
    pub page: usize,         // Page Index (0-based)
    pub points: Vec<Pos2>,   // Ordered sequence of screen-space points
    pub color: Color32,      // Stroke Colour
    pub width: f32,          // Stroke width in logical pixels
}

pub struct AnnotationStore {
    strokes: HashMap<usize, Vec<InkStroke>>,  // Strokes keyed by page index
}

impl AnnotationStore {
    pub fn new() -> Self {
        Self {
            strokes: HashMap::new(),
        }
    }

    /// Append a completed ink stroke
    pub fn add_stroke(&mut self, page: usize, points: Vec<Pos2>, color: Color32, width: f32) {
        // Discard single input taps
        if points.len() < 2 {
            return;
        }

        self.strokes.entry(page).or_default().push(InkStroke { page, points, color, width });
    }

    /// Iterate over all strokes on a page
    pub fn strokes_on_page(&self, page: usize) -> &[InkStroke] {
        self.strokes
            .get(&page)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Remove most recently added stroke on a page (undo).
    pub fn undo_last_stroke(&mut self, page: usize) {
        if let Some(v) = self.strokes.get_mut(&page) {
            v.pop();
        }
    }

    /// Total number of strokes across all pages.
    pub fn total_stroke_count(&self) -> usize {
        self.strokes.values().map(|v| v.len()).sum()
    }
}
