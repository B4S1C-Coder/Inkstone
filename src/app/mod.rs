// Top level eframe/egui  application
// owns the PDF document, annotation store, (maybe the tablet)
// server handle. Writes them all together and the render loop

use anyhow::Result;
use eframe::egui;
use tracing::info;

// anno, pdf, renderer
use crate::annotations::AnnotationStore;
use crate::pdf::PdfDocument;
use crate::renderer::PageRenderer;

pub struct AppConfig {
    pub pdf_path: Option<String>,
    pub enable_tablet: bool,
}

pub fn run(config: AppConfig) -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Inkstone")
            .with_inner_size([1280.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "inkstone",
        native_options,
        Box::new(move |cc| Ok(Box::new(InkstoneApp::new(cc, config)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e}"))
}

pub struct InkstoneApp {
    document: Option<PdfDocument>,
    renderer: PageRenderer,
    annotations: AnnotationStore,
    current_page: usize,
    zoom: f32,
    draw_mode: bool,
    active_stroke: Vec<egui::Pos2>,
}

impl InkstoneApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, config: AppConfig) -> Self {
        if config.enable_tablet {
            info!("tablet server requested -- spawning (stub)");
            // TO-DO: crate::tablet_server::spawn(annotations_tx);
        }

        let mut app = Self {
            document: None,
            renderer: PageRenderer::new(),
            annotations: AnnotationStore::new(),
            current_page: 0,
            zoom: 1.0,
            draw_mode: false,
            active_stroke: Vec::new(),
        };

        if let Some(path) = config.pdf_path {
            app.open_pdf(&path);
        }

        app
    }

    fn open_pdf(&mut self, path: &str) {
        match PdfDocument::open(path) {
            Ok(doc) => {
                info!("opened PDF: {} ({} pages)", path, doc.page_count());
                self.document = Some(doc);
                self.current_page = 0;
            }
            Err(e) => {
                tracing::error!("failed open PDF {path}: {e}");
            }
        }
    }
}

/// egui update loop
impl eframe::App for InkstoneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top Menue Bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open PDF").clicked() {
                        // TODO: native file dialog (rdf crate, milestone 2)
                        ui.close_menu();
                    }
                });

                ui.separator();

                // Page navigation
                if let Some(doc) = &self.document {
                    let total = doc.page_count();
                    ui.label(format!("Page {}/{}", self.current_page + 1, total));

                    if ui.button("◀").clicked() && self.current_page > 0 {
                        self.current_page -= 1;
                    }

                    if ui.button("▶").clicked() && self.current_page + 1 < total {
                        self.current_page += 1;
                    }
                }

                ui.separator();

                // Zoom
                if ui.button("-").clicked() {
                    self.zoom = (self.zoom - 0.1).max(0.2);
                }

                ui.label(format!("{:.0}%", self.zoom * 100.0));

                if ui.button("+").clicked() {
                    self.zoom = (self.zoom + 0.1).min(4.0);
                }

                ui.separator();

                // Draw mode toggle
                let label = if self.draw_mode { "Draw ON" } else { "Draw OFF" };
                if ui.button(label).clicked() {
                    self.draw_mode = !self.draw_mode;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                match &self.document {
                    None => {
                        ui.centered_and_justified(|ui| {
                            ui.label("No PDF Open. File -> Open PDF");
                        });
                    }
                    Some(doc) => {
                        // Render current page to egui texture and display it. The renderer caches
                        // the texture so we only re-rasterize the page / zoom changes.
                        let page_idx = self.current_page;
                        let zoom = self.zoom;

                        match self.renderer.get_or_render(doc, page_idx, zoom, ui.ctx()) {
                            Ok(tex_id) => {
                                let size = self.renderer.last_size();
                                let response = ui.image(egui::load::SizedTexture::new(tex_id, size));

                                // Ink overlay
                                // Collect pointer input on top of the image.

                                if self.draw_mode {
                                    let painter = ui.painter_at(response.rect);

                                    if response.hovered() {
                                        let input = ui.input(|i| i.clone());

                                        if input.pointer.primary_down() {
                                            if let Some(pos) = input.pointer.hover_pos() {
                                                self.active_stroke.push(pos);
                                            }
                                        } else if !self.active_stroke.is_empty() {
                                            // Pen lifted -> commit stroke
                                            let stroke = std::mem::take(&mut self.active_stroke);
                                            self.annotations.add_stroke(
                                                page_idx,
                                                stroke, egui::Color32::from_rgb(220, 50, 50),
                                                2.0
                                            );
                                        }

                                        for win in self.active_stroke.windows(2) {
                                            painter.line_segment([win[0], win[1]], egui::Stroke::new(
                                                2.0,
                                                egui::Color32::from_rgb(220, 50, 50),
                                            ));
                                        }
                                    }

                                    for s in self.annotations.strokes_on_page(page_idx) {
                                        for win in s.points.windows(2) {
                                            painter.line_segment([win[0], win[1]], egui::Stroke::new(s.width, s.color));
                                        }
                                    }
                                }
                            }

                            Err(e) => {
                                ui.label(format!("Render error: {e}"));
                            }
                        }
                    }
                }
            });
        });
    }
}
