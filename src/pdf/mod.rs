// Thin wrapper around pdfium-reader. It is responsible for:
// 1. Open / close pdfs.
// 2. Return raw RGBA pixel data for a given (page, width, height).
// 3. Report page dimensions in points (PDF coordinate space).

// Nothing here should know about egui, wgpu or annotations.

use anyhow::{Context, Result};
use pdfium_render::prelude::*;

pub struct PdfDocument {
    _pdfium: Pdfium,                 // Pdfium instance needed to stay alive for lifetime of the doc
    document: pdfium_render::prelude::PdfDocument<'static>,  // The document
    page_count: usize,               // Number of Pages
}

// SAFETY: PdfDocument is never shared across threads
// The tablet server communicates via channels. It would never touch this struct
unsafe impl Send for PdfDocument {}

impl PdfDocument {
    /// Open a file from disk
    pub fn open(path: &str) -> Result<Self> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                .or_else(|_| Pdfium::bind_to_system_library())
                .context(
                    "could not load pdfium library - place the pdfium shared library next to the binary or install it system wide."
                )?,
        );

        // SAFETY: document's lifetime is tied to struct, which owns the Pdfium instance, so 'static cast
        // is sound as long as we never handout raw references with a shorter lifetime.

        let doc = unsafe {
            let d = pdfium
                .load_pdf_from_file(path, None)
                .context("pdfium could not open file")?;

            std::mem::transmute::<
                pdfium_render::prelude::PdfDocument<'_>,
                pdfium_render::prelude::PdfDocument<'static>
            >(d)
        };

        let page_count = doc.pages().len() as usize;

        Ok(Self {
            _pdfium: pdfium,
            document: doc,
            page_count,
        })
    }

    pub fn page_count(&self) -> usize {
        self.page_count
    }

    /// Return the size of page in PDF Points (1 pt = 1/72 inch)
    pub fn page_size_pts(&self, page_idx: usize) -> Result<(f32, f32)> {
        let page = self
            .document
            .pages()
            .get(page_idx as u16)
            .context("page index out of range")?;
        Ok((page.width().value, page.height().value))
    }

    /// Rasterise a page to an RGBA byte buffer at the requested pixel size.
    /// Returns `(pixels, width, height)`.
    pub fn render_page_rgba(
        &self, page_idx: usize, pixel_width: u16, pixel_height: u16
    ) -> Result<(Vec<u8>, u32, u32)> {

        let page = self
            .document
            .pages()
            .get(page_idx as u16)
            .context("page index out of range")?;

        let bitmap = page
            .render_with_config(
                &PdfRenderConfig::new()
                    .set_target_width(pixel_width as i32)
                    .set_target_height(pixel_height as i32)
                    .rotate_if_landscape(PdfPageRenderRotation::None, true),
            )
            .context("pdfium render failed")?;

        // pdfium renders BGRA; egui / wgpu need RGBA so swap B and R
        let bgra = bitmap.as_bytes();
        let mut rgba = Vec::with_capacity(bgra.len());

        for chunk in bgra.chunks_exact(4) {
            rgba.push(chunk[2]); // R <- B
            rgba.push(chunk[1]); // G
            rgba.push(chunk[0]); // B <- R
            rgba.push(chunk[3]); // A
        }

        let w = bitmap.width() as u32;
        let h = bitmap.height() as u32;
        Ok((rgba, w, h))
    }
}
