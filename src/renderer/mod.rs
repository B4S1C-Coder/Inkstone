// Turns a PdfDocument into an egui TextureHandle.
// Caches the last rendered texture. Re-renders only when the page index
// or zoom level changes. This keeps GPU uploads to the minimum needed.

use anyhow::Result;
use egui::{Context, TextureHandle, TextureOptions, Vec2};

use crate::pdf::PdfDocument;

const MIN_PX_WIDTH: u16 = 200;
const MAX_PX_WIDTH: u16 = 6000;

pub struct  PageRenderer {
    cache: Option<CachedPage>,
}

struct CachedPage {
    texture: TextureHandle,
    page_idx: usize,        // Page index that was rendered
    zoom: f32,              // Zoom level that was used
    size: Vec2,             // Pixel dimensions of the texture
}

impl PageRenderer {
    pub fn new() -> Self {
        Self { cache: None }
    }

    /// Return (or lazily create) the egui texture for `page_idx` at `zoom`
    pub fn get_or_render(
        &mut self, doc: &PdfDocument, page_idx: usize, zoom: f32, ctx: &Context
    ) -> Result<egui::TextureId> {
        // Cache hit
        if let Some(cached) = &self.cache {
            if cached.page_idx == page_idx && (cached.zoom - zoom).abs() < 0.001 {
                return Ok(cached.texture.id());
            }
        }

        // Rastrise on cache miss
        let (pt_w, pt_h) = doc.page_size_pts(page_idx)?;

        // 96 DPI * zoom -> Pixel Dimensions
        // (PDF Points are 1/72 inch; 96/72 = 1.333 px/pt at zoom=1)
        let scale = 96.0 / 72.0 * zoom;
        let px_w = ((pt_w * scale) as u16).clamp(MIN_PX_WIDTH, MAX_PX_WIDTH);
        let px_h = ((pt_h * scale) as u16).clamp(MIN_PX_WIDTH, MAX_PX_WIDTH);

        let (rgba, w, h) = doc.render_page_rgba(page_idx, px_w, px_h)?;

        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [w as usize, h as usize],
            &rgba,
        );

        let texture = ctx.load_texture(
            format!("page-{page_idx}"),
            color_image,
            TextureOptions::LINEAR,
        );

        let size = Vec2::new(w as f32, h as f32);
        self.cache = Some(CachedPage { texture, page_idx, zoom, size });

        Ok(self.cache.as_ref().unwrap().texture.id())
    }

    /// Pixel size of the most recently rendered page.
    pub fn last_size(&self) -> Vec2 {
        self.cache
            .as_ref()
            .map(|c| c.size)
            .unwrap_or(Vec2::new(1.0, 1.0))
    }
}
