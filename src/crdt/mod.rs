// Thin wrapper around Automerge that models annotation operations as CRDT ops.
//
// The document schema (logical):
//
//   {
//     "strokes": [
//       {
//         "id":    <string uuid>,
//         "page":  <u64>,
//         "color": <u32 RGBA>,
//         "width": <f64>,
//         "pts":   [ [x, y], ... ]   // f64 pairs
//       },
//       ...
//     ]
//   }
//
// Every call to `add_stroke` produces a minimal Automerge change (a byte
// blob) that can be sent over the wire to a remote peer and applied with
// `apply_change`.  The CRDT guarantees that applying changes in any order
// converges to the same state.
//
// This module is intentionally decoupled from the rest of the app.
// The `AnnotationStore` in ../annotations/ is the live read model;
// the CrdtDoc here is the write-ahead / sync layer.

use anyhow::{Context, Result};
use automerge::{transaction::Transactable, AutoCommit, ObjType, ReadDoc};

// reexport for convienience
pub type ChangeBytes = Vec<u8>;

pub struct CrdtDoc {
    doc: AutoCommit,
}

impl CrdtDoc {
    pub fn new() -> Self {
        let mut doc = AutoCommit::new();
        doc.put_object(automerge::ROOT, "strokes", ObjType::List)
           .expect("init strokes list");
        Self { doc }
    }

    /// Encode a new ink stroke into the doc and return change bytes.
    /// Caller will update the live Annotation Store.
    pub fn add_stroke(
        &mut self, id: &str, page: usize, color_rgba: u32, width: f64, points: &[(f64, f64)],
    ) -> Result<ChangeBytes> {

        let strokes = self
            .doc
            .get(automerge::ROOT, "strokes")
            .context("no strokes key")?
            .and_then(|(val, obj)| {
                if matches!(val, automerge::Value::Object(ObjType::List)) {
                    Some(obj)
                } else {
                    None
                }
            })
            .context("strokes is not a list")?;

        // append new map to lust
        let stroke_map = self
            .doc
            .insert_object(&strokes, self.doc.length(&strokes), ObjType::Map)
            .context("insert stroke map")?;

        self.doc.put(&stroke_map, "id", id)?;
        self.doc.put(&stroke_map, "page", page as u64)?;
        self.doc.put(&stroke_map, "color", color_rgba as u64)?;
        self.doc.put(&stroke_map, "width", width)?;

        // points as a nested list of [x, y]
        let pts_list = self
            .doc
            .put_object(&stroke_map, "pts", ObjType::List)
            .context("insert pts list")?;

        for (i, (x, y)) in points.iter().enumerate() {
            let pair = self
                .doc
                .insert_object(&pts_list, i, ObjType::List)
                .context("insert pt pair")?;
            self.doc.insert(&pair, 0, *x)?;
            self.doc.insert(&pair, 1, *y)?;
        }

        // change bytes
        let change = self
            .doc
            .get_last_local_change()
            .context("no local change after add_stroke")?;

        Ok(change.raw_bytes().to_vec())
    }

    /// Apply a change received from a remote pair (iPad).
    /// Returns the new strokes that were introduced so the caller can update
    /// the live annotation store. (Stub - full decoding in milestone 3.)
    pub fn apply_change(&mut self, bytes: &[u8]) -> Result<()> {
        let change = automerge::Change::from_bytes(bytes.to_vec())
            .map_err(|e| anyhow::anyhow!("bad change bytes: {e:?}"))?;

        self.doc
            .apply_changes([change])
            .map_err(|e| anyhow::anyhow!("automerge apply error: {e:?}"))?;
        Ok(())
    }

    /// Serialize full doc state to save to disk
    pub fn save(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    /// Load from a previously saved byte blob
    pub fn load(bytes: &[u8]) -> Result<Self> {
        let doc = AutoCommit::load(bytes)
            .map_err(|e| anyhow::anyhow!("automerge load error: {e:?}"))?;
        Ok(Self { doc })
    }
}
