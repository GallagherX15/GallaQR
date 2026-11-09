use crate::history::{QrHistory, QrHistoryItem};
use crate::qr::{ErrorCorrection, QrGenerator};
use eframe::egui::{
    self, Align, Align2, Button, Color32, ComboBox, CornerRadius, DragValue, Frame, Layout, Margin,
    Panel, Pos2, Rect, RichText, ScrollArea, Sense, Stroke, TextEdit, TextureHandle,
    TextureOptions, Ui, Vec2, ViewportCommand,
};
use qrcode::QrCode;
use std::time::Instant;

fn parse_hex_color(hex: &str) -> Option<[u8; 3]> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some([r, g, b])
    } else if s.len() == 3 {
        let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
        let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
        let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
        Some([r, g, b])
    } else {
        None
    }
}

fn format_hex_color(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

pub struct GallaQrApp {
    // Historial
    pub history: QrHistory,
    pub history_search: String,
    pub selected_history_id: Option<String>,
    pub sidebar_open: bool,

    // Campos de entrada
    pub title_input: String,
    pub content_input: String,

    // Configuración del QR
    pub error_correction: ErrorCorrection,
    pub qr_color: [u8; 3],
    pub qr_color_hex: String,

    // Fondo del QR
    pub has_background: bool,
    pub bg_color: [u8; 3],
    pub bg_color_hex: String,

    pub margin_modules: u32,

    // Dimensiones de exportación
    pub export_width: u32,
    pub export_height: u32,
    pub lock_aspect_ratio: bool,

    // Estado del QR activo
    pub active_qr: Option<QrCode>,
    pub preview_texture: Option<TextureHandle>,

    // Mensaje de estado / notificación (mensaje, es_éxito, momento)
    pub status_message: Option<(String, bool, Instant)>,

    // Confirmación para vaciar historial
    pub confirm_clear_history: bool,
}

impl Default for GallaQrApp {
    fn default() -> Self {
        let history = QrHistory::load();
        let default_qr_color = [0, 0, 0];
        let default_bg_color = [255, 255, 255];

        let mut app = Self {
            history,
            history_search: String::new(),
            selected_history_id: None,
            sidebar_open: true,
            title_input: String::new(),
            content_input: String::new(),
            error_correction: ErrorCorrection::Medium,
            qr_color: default_qr_color,
            qr_color_hex: format_hex_color(default_qr_color),
            has_background: false, // Por defecto sin fondo
            bg_color: default_bg_color,
            bg_color_hex: format_hex_color(default_bg_color),
            margin_modules: 2,
            export_width: 1024,
            export_height: 1024,
            lock_aspect_ratio: true,
            active_qr: None,
            preview_texture: None,
            status_message: None,
            confirm_clear_history: false,
        };

        if let Some(first) = app.history.items.first().cloned() {
            app.load_history_item(&first);
        }

        app
    }
}

impl GallaQrApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self::default()
    }

    fn set_status(&mut self, message: impl Into<String>, is_success: bool) {
        self.status_message = Some((message.into(), is_success, Instant::now()));
    }

    fn load_history_item(&mut self, item: &QrHistoryItem) {
        self.selected_history_id = Some(item.id.clone());
        self.title_input = item.title.clone();
        self.content_input = item.content.clone();
        self.error_correction = ErrorCorrection::from_code(&item.ecc_level);

        if let Some(rgb) = parse_hex_color(&item.color_hex) {
            self.qr_color = rgb;
            self.qr_color_hex = format_hex_color(rgb);
        }

        self.generate_active_qr(false);
    }

    fn generate_active_qr(&mut self, save_to_history: bool) {
        let content = self.content_input.trim();
        if content.is_empty() {
            self.set_status("Por favor ingresa un enlace o contenido para el QR.", false);
            return;
        }

        match QrGenerator::generate_matrix(content, self.error_correction) {
            Ok(qr) => {
                self.active_qr = Some(qr);
                self.preview_texture = None;

                if save_to_history {
                    let title = if self.title_input.trim().is_empty() {
                        "Sin Título".to_string()
                    } else {
                        self.title_input.trim().to_string()
                    };

                    let hex_color = format_hex_color(self.qr_color);

                    let item = self.history.add(
                        title,
                        content.to_string(),
                        self.error_correction.code().to_string(),
                        hex_color,
                    );
                    self.selected_history_id = Some(item.id);
                    self.set_status("✨ ¡Código QR generado y guardado en el historial!", true);
                }
            }
            Err(e) => {
                self.set_status(format!("Error generando QR: {e}"), false);
            }
        }
    }

    fn new_qr(&mut self) {
        self.selected_history_id = None;
        self.title_input.clear();
        self.content_input.clear();
        self.active_qr = None;
        self.preview_texture = None;
        self.set_status("Formulario limpio. Listo para crear un nuevo código QR.", true);
    }

    fn sanitize_slug(&self) -> String {
        let base = if !self.title_input.trim().is_empty() {
            self.title_input.trim()
        } else {
            "gallaqr-code"
        };

        let slug: String = base
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect();

        let clean = slug
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        if clean.is_empty() {
            "gallaqr-code".to_string()
        } else {
            clean
        }
    }

    fn get_bg_rgba(&self) -> Option<[u8; 4]> {
        if self.has_background {
            Some([self.bg_color[0], self.bg_color[1], self.bg_color[2], 255])
        } else {
            None
        }
    }

    fn get_bg_hex(&self) -> Option<String> {
        if self.has_background {
            Some(format_hex_color(self.bg_color))
        } else {
            None
        }
    }

    fn download_svg(&mut self) {
        let qr = match &self.active_qr {
            Some(qr) => qr.clone(),
            None => {
                self.set_status("Primero genera o selecciona un código QR para descargar.", false);
                return;
            }
        };

        let filename = format!("{}.svg", self.sanitize_slug());
        let dialog = rfd::FileDialog::new()
            .set_title("Guardar QR como SVG")
            .set_file_name(&filename)
            .add_filter("Archivo Vectorial SVG (*.svg)", &["svg"]);

        if let Some(path) = dialog.save_file() {
            let fg_hex = format_hex_color(self.qr_color);
            let bg_hex = self.get_bg_hex();
            let svg_content = QrGenerator::generate_svg(
                &qr,
                self.export_width,
                self.export_height,
                self.margin_modules,
                &fg_hex,
                bg_hex.as_deref(),
            );

            match std::fs::write(&path, svg_content) {
                Ok(_) => {
                    let desc = if self.has_background { "con fondo" } else { "sin fondo" };
                    self.set_status(
                        format!("✅ SVG ({desc}) guardado con éxito en: {}", path.display()),
                        true,
                    );
                }
                Err(e) => {
                    self.set_status(format!("❌ Error al guardar archivo SVG: {e}"), false);
                }
            }
        }
    }

    fn download_png(&mut self) {
        let qr = match &self.active_qr {
            Some(qr) => qr.clone(),
            None => {
                self.set_status("Primero genera o selecciona un código QR para descargar.", false);
                return;
            }
        };

        let filename = format!("{}.png", self.sanitize_slug());
        let dialog = rfd::FileDialog::new()
            .set_title("Guardar QR como PNG")
            .set_file_name(&filename)
            .add_filter("Imagen PNG (*.png)", &["png"]);

        if let Some(path) = dialog.save_file() {
            let fg_rgba = [self.qr_color[0], self.qr_color[1], self.qr_color[2], 255];
            let bg_rgba = self.get_bg_rgba();
            match QrGenerator::generate_png(
                &qr,
                self.export_width,
                self.export_height,
                self.margin_modules,
                fg_rgba,
                bg_rgba,
            ) {
                Ok(bytes) => match std::fs::write(&path, bytes) {
                    Ok(_) => {
                        let desc = if self.has_background { "con fondo" } else { "sin fondo" };
                        self.set_status(
                            format!(
                                "✅ PNG ({desc}, {}x{} px) guardado con éxito en: {}",
                                self.export_width,
                                self.export_height,
                                path.display()
                            ),
                            true,
                        );
                    }
                    Err(e) => {
                        self.set_status(format!("❌ Error al escribir archivo PNG: {e}"), false);
                    }
                },
                Err(e) => {
                    self.set_status(format!("❌ Error generando imagen PNG: {e}"), false);
                }
            }
        }
    }

    fn download_jpg(&mut self) {
        let qr = match &self.active_qr {
            Some(qr) => qr.clone(),
            None => {
                self.set_status("Primero genera o selecciona un código QR para descargar.", false);
                return;
            }
        };

        let filename = format!("{}.jpg", self.sanitize_slug());
        let dialog = rfd::FileDialog::new()
            .set_title("Guardar QR como JPG (Con Fondo)")
            .set_file_name(&filename)
            .add_filter("Imagen JPEG (*.jpg, *.jpeg)", &["jpg", "jpeg"]);

        if let Some(path) = dialog.save_file() {
            match QrGenerator::generate_jpeg(
                &qr,
                self.export_width,
                self.export_height,
                self.margin_modules,
                self.qr_color,
                self.bg_color,
            ) {
                Ok(bytes) => match std::fs::write(&path, bytes) {
                    Ok(_) => {
                        self.set_status(
                            format!(
                                "✅ JPG ({}x{} px) guardado con éxito en: {}",
                                self.export_width,
                                self.export_height,
                                path.display()
                            ),
                            true,
                        );
                    }
                    Err(e) => {
                        self.set_status(format!("❌ Error al guardar archivo JPG: {e}"), false);
                    }
                },
                Err(e) => {
                    self.set_status(format!("❌ Error generando imagen JPG: {e}"), false);
                }
            }
        }
    }

    fn copy_svg_to_clipboard(&mut self, ctx: &egui::Context) {
        let qr = match &self.active_qr {
            Some(qr) => qr.clone(),
            None => {
                self.set_status("Primero genera un código QR para copiar el SVG.", false);
                return;
            }
        };

        let fg_hex = format_hex_color(self.qr_color);
        let bg_hex = self.get_bg_hex();
        let svg = QrGenerator::generate_svg(
            &qr,
            self.export_width,
            self.export_height,
            self.margin_modules,
            &fg_hex,
            bg_hex.as_deref(),
        );

        ctx.copy_text(svg);
        self.set_status("📋 ¡Código SVG copiado al portapapeles!", true);
    }

    fn copy_png_to_clipboard(&mut self) {
        let qr = match &self.active_qr {
            Some(qr) => qr.clone(),
            None => {
                self.set_status("Primero genera un código QR para copiar el PNG.", false);
                return;
            }
        };

        let fg_rgba = [self.qr_color[0], self.qr_color[1], self.qr_color[2], 255];
        let bg_rgba = self.get_bg_rgba();

        match QrGenerator::generate_raw_rgba(
            &qr,
            self.export_width,
            self.export_height,
            self.margin_modules,
            fg_rgba,
            bg_rgba,
        ) {
            Ok(raw_rgba) => {
                match arboard::Clipboard::new() {
                    Ok(mut cb) => {
                        let img_data = arboard::ImageData {
                            width: self.export_width as usize,
                            height: self.export_height as usize,
                            bytes: std::borrow::Cow::from(raw_rgba),
                        };
                        if let Err(e) = cb.set_image(img_data) {
                            self.set_status(format!("❌ Error copiando imagen al portapapeles: {e}"), false);
                        } else {
                            let desc = if self.has_background { "con fondo" } else { "sin fondo" };
                            self.set_status(
                                format!("📋 ¡Imagen PNG ({desc}) copiada al portapapeles!",),
                                true,
                            );
                        }
                    }
                    Err(e) => {
                        self.set_status(format!("❌ No se pudo abrir el portapapeles: {e}"), false);
                    }
                }
            }
            Err(e) => {
                self.set_status(format!("❌ Error procesando imagen: {e}"), false);
            }
        }
    }

    fn render_sidebar(&mut self, ui: &mut Ui) {
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("📚 Historial de QRs")
                    .size(15.0)
                    .strong()
                    .color(Color32::from_rgb(226, 232, 240)),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let badge_text = format!("{}", self.history.items.len());
                let badge_frame = Frame::NONE
                    .fill(Color32::from_rgb(45, 50, 65))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(8, 2));
                badge_frame.show(ui, |ui| {
                    ui.label(
                        RichText::new(badge_text)
                            .size(11.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                });
            });
        });

        ui.add_space(8.0);

        let new_btn = Button::new(RichText::new("➕ Nuevo QR").strong().color(Color32::WHITE))
            .fill(Color32::from_rgb(99, 102, 241))
            .corner_radius(CornerRadius::same(6))
            .min_size(Vec2::new(ui.available_width(), 32.0));

        if ui.add(new_btn).clicked() {
            self.new_qr();
        }

        ui.add_space(8.0);

        ui.add(
            TextEdit::singleline(&mut self.history_search)
                .hint_text("🔍 Filtrar historial...")
                .desired_width(f32::INFINITY),
        );

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        let mut item_to_load = None;
        let mut item_to_delete = None;

        let query = self.history_search.to_lowercase();
        let filtered: Vec<_> = self
            .history
            .items
            .iter()
            .filter(|it| {
                if query.is_empty() {
                    true
                } else {
                    it.title.to_lowercase().contains(&query)
                        || it.content.to_lowercase().contains(&query)
                }
            })
            .collect();

        if filtered.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                if self.history.items.is_empty() {
                    ui.label(
                        RichText::new("Aún no tienes QRs guardados.\nGenera el primero en el formulario.")
                            .size(12.5)
                            .color(Color32::from_gray(140)),
                    );
                } else {
                    ui.label(
                        RichText::new("Sin coincidencias en el historial.")
                            .size(12.5)
                            .color(Color32::from_gray(140)),
                    );
                }
            });
        } else {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for item in filtered {
                        let is_selected = self.selected_history_id.as_deref() == Some(&item.id);

                        let card_fill = if is_selected {
                            Color32::from_rgb(37, 43, 62)
                        } else {
                            Color32::from_rgb(26, 28, 35)
                        };

                        let card_stroke = if is_selected {
                            Stroke::new(1.5, Color32::from_rgb(99, 102, 241))
                        } else {
                            Stroke::new(1.0, Color32::from_rgb(45, 48, 60))
                        };

                        let item_frame = Frame::NONE
                            .fill(card_fill)
                            .stroke(card_stroke)
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::same(10));

                        item_frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let title_text = if item.title.is_empty() {
                                    "Sin Título"
                                } else {
                                    &item.title
                                };
                                ui.label(
                                    RichText::new(title_text)
                                        .size(13.0)
                                        .strong()
                                        .color(Color32::from_rgb(241, 245, 249)),
                                );

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui
                                        .button(RichText::new("🗑").size(12.0).color(Color32::from_gray(160)))
                                        .on_hover_text("Eliminar del historial")
                                        .clicked()
                                    {
                                        item_to_delete = Some(item.id.clone());
                                    }
                                });
                            });

                            ui.add_space(2.0);

                            let preview_content = if item.content.len() > 34 {
                                format!("{}...", &item.content[..31])
                            } else {
                                item.content.clone()
                            };
                            ui.label(
                                RichText::new(preview_content)
                                    .size(11.5)
                                    .color(Color32::from_rgb(148, 163, 184)),
                            );

                            ui.add_space(4.0);

                            ui.horizontal(|ui| {
                                let date_str = item.created_at.format("%d/%m/%Y %H:%M").to_string();
                                ui.label(
                                    RichText::new(date_str)
                                        .size(10.5)
                                        .color(Color32::from_gray(115)),
                                );

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let badge = format!("ECC: {}", item.ecc_level);
                                    ui.label(
                                        RichText::new(badge)
                                            .size(10.0)
                                            .color(Color32::from_rgb(99, 102, 241)),
                                    );
                                });
                            });

                            let card_rect = ui.min_rect();
                            let card_response = ui.interact(
                                card_rect,
                                ui.scope_id().with(&item.id),
                                Sense::click(),
                            );
                            if card_response.clicked() {
                                item_to_load = Some(item.clone());
                            }
                        });

                        ui.add_space(6.0);
                    }
                });
        }

        if let Some(item) = item_to_load {
            self.load_history_item(&item);
        }

        if let Some(id) = item_to_delete {
            if self.selected_history_id.as_deref() == Some(&id) {
                self.selected_history_id = None;
            }
            self.history.remove(&id);
            self.set_status("Elemento eliminado del historial.", true);
        }

        if !self.history.items.is_empty() {
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            if self.confirm_clear_history {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("¿Seguro?").size(11.5).color(Color32::from_rgb(248, 113, 113)));
                    if ui.button(RichText::new("Sí, borrar").size(11.0).color(Color32::WHITE)).clicked() {
                        self.history.clear();
                        self.confirm_clear_history = false;
                        self.set_status("Historial vaciado por completo.", true);
                    }
                    if ui.button(RichText::new("Cancelar").size(11.0)).clicked() {
                        self.confirm_clear_history = false;
                    }
                });
            } else {
                if ui
                    .button(RichText::new("Limpiar todo el historial").size(11.5).color(Color32::from_gray(140)))
                    .clicked()
                {
                    self.confirm_clear_history = true;
                }
            }
        }
    }

    fn render_topbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Botón para alternar el panel lateral de historial
            let toggle_icon = if self.sidebar_open { "◀ Historial" } else { "▶ Historial" };
            if ui
                .button(RichText::new(toggle_icon).size(12.0).color(Color32::from_rgb(199, 210, 254)))
                .on_hover_text("Mostrar u ocultar panel de historial")
                .clicked()
            {
                self.sidebar_open = !self.sidebar_open;
            }

            ui.add_space(4.0);

            ui.label(
                RichText::new("GallaQR")
                    .size(18.0)
                    .strong()
                    .color(Color32::from_rgb(129, 140, 248)),
            );
            ui.label(
                RichText::new("v0.1.0")
                    .size(10.5)
                    .color(Color32::from_rgb(99, 102, 241)),
            );

            // Botones de control de ventana (Minimizar, Maximizar/Separar, Cerrar)
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Cerrar
                let close_btn = Button::new(RichText::new(" ✕ ").size(13.0).strong().color(Color32::from_gray(200)))
                    .fill(Color32::from_rgb(45, 20, 25))
                    .corner_radius(CornerRadius::same(4));
                if ui.add(close_btn).on_hover_text("Cerrar ventana").clicked() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                }

                ui.add_space(2.0);

                // Maximizar / Separar de mosaico / Restaurar
                let is_max = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                let max_icon = if is_max { " 🗗 " } else { " 🗖 " };
                let max_tooltip = if is_max { "Restaurar / Separar tamaño" } else { "Maximizar ventana" };

                let max_btn = Button::new(RichText::new(max_icon).size(13.0).color(Color32::from_gray(200)))
                    .fill(Color32::from_rgb(30, 34, 45))
                    .corner_radius(CornerRadius::same(4));
                if ui.add(max_btn).on_hover_text(max_tooltip).clicked() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(!is_max));
                }

                ui.add_space(2.0);

                // Minimizar
                let min_btn = Button::new(RichText::new(" 🗕 ").size(13.0).color(Color32::from_gray(200)))
                    .fill(Color32::from_rgb(30, 34, 45))
                    .corner_radius(CornerRadius::same(4));
                if ui.add(min_btn).on_hover_text("Minimizar ventana").clicked() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
                }
            });
        });

        ui.add_space(6.0);

        // Toast de estado / alerta
        if let Some((msg, is_success, time)) = &self.status_message {
            if time.elapsed().as_secs() < 8 {
                let (fill, stroke, icon) = if *is_success {
                    (
                        Color32::from_rgb(16, 45, 28),
                        Stroke::new(1.0, Color32::from_rgb(34, 197, 94)),
                        "✅",
                    )
                } else {
                    (
                        Color32::from_rgb(50, 20, 20),
                        Stroke::new(1.0, Color32::from_rgb(239, 68, 68)),
                        "⚠️",
                    )
                };

                let toast_frame = Frame::NONE
                    .fill(fill)
                    .stroke(stroke)
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(12, 6));

                toast_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(icon).size(12.0));
                        ui.label(
                            RichText::new(msg)
                                .size(12.0)
                                .color(Color32::from_rgb(241, 245, 249)),
                        );
                    });
                });
                ui.add_space(4.0);
            }
        }
    }

    fn render_card<R>(ui: &mut Ui, title: &str, content: impl FnOnce(&mut Ui) -> R) -> R {
        let frame = Frame::NONE
            .fill(Color32::from_rgb(22, 24, 30))
            .stroke(Stroke::new(1.0, Color32::from_rgb(45, 48, 60)))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(12));

        frame
            .show(ui, |ui| {
                ui.label(
                    RichText::new(title)
                        .size(14.0)
                        .strong()
                        .color(Color32::from_rgb(226, 232, 240)),
                );
                ui.add_space(6.0);
                content(ui)
            })
            .inner
    }

    fn render_generator_section(&mut self, ui: &mut Ui) {
        // Tarjeta 1: Datos del QR
        Self::render_card(ui, "📝 Datos del Código QR", |ui| {
            ui.label(
                RichText::new("Título descriptivo:")
                    .size(12.0)
                    .color(Color32::from_rgb(203, 213, 225)),
            );
            ui.add(
                TextEdit::singleline(&mut self.title_input)
                    .hint_text("Ej: Mi Portafolio, Menú...")
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(8.0);

            ui.label(
                RichText::new("Enlace URL o Contenido:")
                    .size(12.0)
                    .color(Color32::from_rgb(203, 213, 225)),
            );
            ui.add(
                TextEdit::multiline(&mut self.content_input)
                    .hint_text("https://ejemplo.com o texto...")
                    .desired_rows(2)
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(10.0);

            let generate_btn = Button::new(
                RichText::new("⚡ Generar y Guardar QR")
                    .size(13.5)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(79, 70, 229))
            .corner_radius(CornerRadius::same(8))
            .min_size(Vec2::new(ui.available_width(), 36.0));

            if ui.add(generate_btn).clicked() {
                self.generate_active_qr(true);
            }
        });

        ui.add_space(10.0);

        // Tarjeta 2: Personalización del QR
        Self::render_card(ui, "🎨 Estilo y Configuración", |ui| {
            // Color del QR con selector y entrada de código HEX
            ui.label(
                RichText::new("Color del código:")
                    .size(12.0)
                    .color(Color32::from_rgb(203, 213, 225)),
            );

            ui.horizontal(|ui| {
                if ui.color_edit_button_srgb(&mut self.qr_color).changed() {
                    self.qr_color_hex = format_hex_color(self.qr_color);
                    self.preview_texture = None;
                }

                ui.label(RichText::new("HEX:").size(11.5).color(Color32::from_gray(170)));
                let hex_edit = ui.add(
                    TextEdit::singleline(&mut self.qr_color_hex)
                        .desired_width(80.0)
                        .hint_text("#000000"),
                );
                if hex_edit.changed() {
                    if let Some(rgb) = parse_hex_color(&self.qr_color_hex) {
                        self.qr_color = rgb;
                        self.preview_texture = None;
                    }
                }
            });

            ui.add_space(4.0);

            // Paleta rápida de colores predeterminados (incluyendo Blanco #FFFFFF)
            ui.horizontal_wrapped(|ui| {
                let presets: [(&str, [u8; 3]); 7] = [
                    ("Negro", [0, 0, 0]),
                    ("Blanco", [255, 255, 255]),
                    ("Azul", [30, 64, 175]),
                    ("Índigo", [79, 70, 229]),
                    ("Esmeralda", [5, 150, 105]),
                    ("Rojo", [220, 38, 38]),
                    ("Púrpura", [124, 58, 237]),
                ];

                for (name, col) in presets {
                    let col32 = Color32::from_rgb(col[0], col[1], col[2]);
                    let border_col = if col == [255, 255, 255] {
                        Color32::from_gray(180)
                    } else {
                        Color32::from_gray(60)
                    };
                    let btn = Button::new(RichText::new("■").color(col32).size(17.0))
                        .stroke(Stroke::new(1.0, border_col))
                        .fill(Color32::from_rgb(30, 32, 40))
                        .corner_radius(CornerRadius::same(4));
                    if ui.add(btn).on_hover_text(name).clicked() {
                        self.qr_color = col;
                        self.qr_color_hex = format_hex_color(col);
                        self.preview_texture = None;
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // Nivel de corrección de errores
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Corrección:")
                        .size(12.0)
                        .color(Color32::from_rgb(203, 213, 225)),
                );
                let current_ecc = self.error_correction;
                ComboBox::from_id_salt("ecc_combo")
                    .selected_text(current_ecc.as_str())
                    .show_ui(ui, |ui| {
                        for ec in [
                            ErrorCorrection::Low,
                            ErrorCorrection::Medium,
                            ErrorCorrection::Quartile,
                            ErrorCorrection::High,
                        ] {
                            if ui
                                .selectable_value(&mut self.error_correction, ec, ec.as_str())
                                .changed()
                            {
                                if self.active_qr.is_some() {
                                    self.generate_active_qr(false);
                                }
                            }
                        }
                    });
            });

            ui.add_space(8.0);

            // Margen (Quiet Zone)
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Margen:")
                        .size(12.0)
                        .color(Color32::from_rgb(203, 213, 225)),
                );
                for m in [0, 1, 2, 4] {
                    let text = if m == 2 { "2 (Normal)" } else { &format!("{m}") };
                    if ui
                        .selectable_label(self.margin_modules == m, text)
                        .clicked()
                    {
                        self.margin_modules = m;
                        self.preview_texture = None;
                    }
                }
            });
        });
    }

    fn render_preview_and_download_section(&mut self, ui: &mut Ui) {
        // Tarjeta: Vista previa (renombrada y con toggle de fondo limpio)
        Self::render_card(ui, "👁️ Vista previa", |ui| {
            // Botón para colocarle o quitarle fondo
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Fondo:")
                        .size(12.0)
                        .color(Color32::from_rgb(203, 213, 225)),
                );

                let bg_text = if self.has_background {
                    "⬜ Con Fondo"
                } else {
                    "🔳 Sin Fondo (Transparente)"
                };

                let bg_toggle_btn = Button::new(
                    RichText::new(bg_text).size(12.0).strong().color(
                        if self.has_background {
                            Color32::from_rgb(241, 245, 249)
                        } else {
                            Color32::from_rgb(147, 197, 253)
                        },
                    ),
                )
                .fill(
                    if self.has_background {
                        Color32::from_rgb(55, 65, 81)
                    } else {
                        Color32::from_rgb(30, 41, 59)
                    },
                )
                .stroke(Stroke::new(
                    1.0,
                    if self.has_background {
                        Color32::from_rgb(148, 163, 184)
                    } else {
                        Color32::from_rgb(59, 130, 246)
                    },
                ))
                .corner_radius(CornerRadius::same(6));

                if ui.add(bg_toggle_btn).clicked() {
                    self.has_background = !self.has_background;
                    self.preview_texture = None;
                }
            });

            // Si está activado "Con Fondo", mostrar selector de color de fondo y HEX
            if self.has_background {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Color fondo:")
                            .size(11.5)
                            .color(Color32::from_gray(180)),
                    );

                    if ui.color_edit_button_srgb(&mut self.bg_color).changed() {
                        self.bg_color_hex = format_hex_color(self.bg_color);
                        self.preview_texture = None;
                    }

                    ui.label(RichText::new("HEX:").size(11.0).color(Color32::from_gray(170)));
                    let bg_hex_edit = ui.add(
                        TextEdit::singleline(&mut self.bg_color_hex)
                            .desired_width(75.0)
                            .hint_text("#FFFFFF"),
                    );
                    if bg_hex_edit.changed() {
                        if let Some(rgb) = parse_hex_color(&self.bg_color_hex) {
                            self.bg_color = rgb;
                            self.preview_texture = None;
                        }
                    }
                });

                ui.add_space(2.0);

                // Presets rápidos para color de fondo
                ui.horizontal_wrapped(|ui| {
                    let bg_presets: [(&str, [u8; 3]); 5] = [
                        ("Blanco", [255, 255, 255]),
                        ("Negro", [0, 0, 0]),
                        ("Crema", [253, 251, 247]),
                        ("Gris", [226, 232, 240]),
                        ("Azul Oscuro", [15, 23, 42]),
                    ];

                    for (name, col) in bg_presets {
                        let col32 = Color32::from_rgb(col[0], col[1], col[2]);
                        let btn = Button::new(RichText::new("■").color(col32).size(15.0))
                            .stroke(Stroke::new(1.0, Color32::from_gray(70)))
                            .fill(Color32::from_rgb(30, 32, 40))
                            .corner_radius(CornerRadius::same(4));
                        if ui.add(btn).on_hover_text(name).clicked() {
                            self.bg_color = col;
                            self.bg_color_hex = format_hex_color(col);
                            self.preview_texture = None;
                        }
                    }
                });
            }

            ui.add_space(8.0);

            // Asegurar textura
            if let Some(qr) = &self.active_qr {
                if self.preview_texture.is_none() {
                    let bg_rgba = self.get_bg_rgba();
                    let img = QrGenerator::generate_preview_image(
                        qr,
                        self.margin_modules,
                        [self.qr_color[0], self.qr_color[1], self.qr_color[2], 255],
                        bg_rgba,
                    );
                    self.preview_texture = Some(ui.ctx().load_texture(
                        "qr_preview_tex",
                        img,
                        TextureOptions::NEAREST,
                    ));
                }
            }

            // Canvas del QR (se ajusta dinámicamente)
            let max_canvas = 220.0f32;
            let available_width = ui.available_width().max(120.0);
            let preview_dimension = max_canvas.min(available_width - 20.0).max(140.0);
            let preview_size = Vec2::new(preview_dimension, preview_dimension);

            ui.vertical_centered(|ui| {
                let (rect, _response) = ui.allocate_exact_size(preview_size, Sense::hover());
                let painter = ui.painter();

                if self.has_background {
                    // Fondo de color elegido
                    let bg_col32 = Color32::from_rgb(self.bg_color[0], self.bg_color[1], self.bg_color[2]);
                    painter.rect_filled(rect, CornerRadius::same(6), bg_col32);
                } else {
                    // Cuadrícula de ajedrez indicando transparencia
                    let cell_size = 10.0;
                    let cols = (rect.width() / cell_size).ceil() as usize;
                    let rows = (rect.height() / cell_size).ceil() as usize;

                    for r in 0..rows {
                        for c in 0..cols {
                            let x0 = rect.min.x + c as f32 * cell_size;
                            let y0 = rect.min.y + r as f32 * cell_size;
                            let x1 = (x0 + cell_size).min(rect.max.x);
                            let y1 = (y0 + cell_size).min(rect.max.y);
                            let col = if (r + c) % 2 == 0 {
                                Color32::from_rgb(32, 35, 44)
                            } else {
                                Color32::from_rgb(48, 52, 65)
                            };
                            painter.rect_filled(
                                Rect::from_min_max(Pos2::new(x0, y0), Pos2::new(x1, y1)),
                                CornerRadius::ZERO,
                                col,
                            );
                        }
                    }
                }

                // Dibujar el QR
                if let Some(texture) = &self.preview_texture {
                    painter.image(
                        texture.id(),
                        rect,
                        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    painter.text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        "Genera un QR para ver aquí",
                        egui::FontId::proportional(12.0),
                        Color32::from_gray(160),
                    );
                }

                ui.add_space(4.0);

                if let Some(qr) = &self.active_qr {
                    let bg_label = if self.has_background { "Con fondo" } else { "Sin fondo (Transparente)" };
                    ui.label(
                        RichText::new(format!(
                            "Matriz: {}x{} módulos • {}",
                            qr.width(),
                            qr.width(),
                            bg_label
                        ))
                        .size(10.5)
                        .color(Color32::from_rgb(148, 163, 184)),
                    );
                }
            });
        });

        ui.add_space(10.0);

        // Tarjeta: Medidas y Proporciones
        Self::render_card(ui, "📐 Medidas y Proporciones", |ui| {
            ui.label(
                RichText::new("Presets rápidos 1:1:")
                    .size(11.5)
                    .color(Color32::from_gray(170)),
            );

            ui.horizontal_wrapped(|ui| {
                let presets = [
                    (256, "256 px"),
                    (512, "512 px"),
                    (1024, "1024 px (HD)"),
                    (2048, "2048 px"),
                    (4096, "4096 px"),
                ];

                for (size, label) in presets {
                    let is_current = self.export_width == size && self.export_height == size;
                    if ui.selectable_label(is_current, label).clicked() {
                        self.export_width = size;
                        self.export_height = size;
                    }
                }
            });

            ui.add_space(6.0);

            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("Ancho:")
                        .size(11.5)
                        .color(Color32::from_rgb(203, 213, 225)),
                );
                let w_resp = ui.add(
                    DragValue::new(&mut self.export_width)
                        .range(64..=8192)
                        .speed(4.0)
                        .suffix(" px"),
                );

                ui.label(
                    RichText::new("x  Alto:")
                        .size(11.5)
                        .color(Color32::from_rgb(203, 213, 225)),
                );
                let h_resp = ui.add(
                    DragValue::new(&mut self.export_height)
                        .range(64..=8192)
                        .speed(4.0)
                        .suffix(" px"),
                );

                if self.lock_aspect_ratio {
                    if w_resp.changed() {
                        self.export_height = self.export_width;
                    } else if h_resp.changed() {
                        self.export_width = self.export_height;
                    }
                }
            });

            ui.add_space(4.0);

            ui.checkbox(
                &mut self.lock_aspect_ratio,
                "Mantener proporción 1:1 (cuadrado)",
            );
        });

        ui.add_space(10.0);

        // Tarjeta: Descargar y Copiar en Portapapeles
        Self::render_card(ui, "💾 Opciones de Descarga y Portapapeles", |ui| {
            // Fila de botones de descarga
            ui.label(
                RichText::new("Descargar archivo en carpeta:")
                    .size(11.5)
                    .color(Color32::from_gray(160)),
            );

            ui.add_space(4.0);

            ui.horizontal_wrapped(|ui| {
                // Descargar SVG
                let svg_btn = Button::new(
                    RichText::new("📥 SVG\nVectorial")
                        .size(12.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(16, 185, 129))
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(88.0, 40.0));

                if ui.add(svg_btn).on_hover_text("Descargar como archivo SVG vectorial").clicked() {
                    self.download_svg();
                }

                // Descargar PNG
                let png_btn = Button::new(
                    RichText::new("📥 PNG\nImagen")
                        .size(12.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(14, 165, 233))
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(88.0, 40.0));

                if ui.add(png_btn).on_hover_text("Descargar como imagen PNG de alta fidelidad").clicked() {
                    self.download_png();
                }

                // Descargar JPG (con fondo)
                let jpg_btn = Button::new(
                    RichText::new("📥 JPG\nCon Fondo")
                        .size(12.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(249, 115, 22))
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(88.0, 40.0));

                if ui.add(jpg_btn).on_hover_text("Descargar como imagen JPG sólida").clicked() {
                    self.download_jpg();
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // Fila de botones para copiar al portapapeles
            ui.label(
                RichText::new("Copiar directo al portapapeles:")
                    .size(11.5)
                    .color(Color32::from_gray(160)),
            );

            ui.add_space(4.0);

            ui.horizontal_wrapped(|ui| {
                // Copiar SVG
                let copy_svg_btn = Button::new(
                    RichText::new("📋 Copiar SVG")
                        .size(12.0)
                        .color(Color32::from_rgb(199, 210, 254)),
                )
                .fill(Color32::from_rgb(30, 36, 55))
                .stroke(Stroke::new(1.0, Color32::from_rgb(99, 102, 241)))
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(110.0, 32.0));

                let ctx = ui.ctx().clone();
                if ui.add(copy_svg_btn).on_hover_text("Copiar código vectorial SVG al portapapeles").clicked() {
                    self.copy_svg_to_clipboard(&ctx);
                }

                // Copiar PNG
                let copy_png_btn = Button::new(
                    RichText::new("📋 Copiar PNG")
                        .size(12.0)
                        .color(Color32::from_rgb(199, 210, 254)),
                )
                .fill(Color32::from_rgb(30, 36, 55))
                .stroke(Stroke::new(1.0, Color32::from_rgb(56, 189, 248)))
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(110.0, 32.0));

                if ui.add(copy_png_btn).on_hover_text("Copiar imagen PNG al portapapeles para pegar en cualquier app").clicked() {
                    self.copy_png_to_clipboard();
                }
            });
        });
    }
}

impl eframe::App for GallaQrApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        // Panel lateral izquierdo: Historial
        if self.sidebar_open {
            Panel::left("gallaqr_sidebar")
                .default_size(280.0)
                .min_size(210.0)
                .max_size(420.0)
                .show(ui, |ui| {
                    self.render_sidebar(ui);
                });
        }

        // Panel central
        egui::CentralPanel::default().show(ui, |ui| {
            // Barra superior fija con controles de ventana
            self.render_topbar(ui);
            ui.separator();
            ui.add_space(4.0);

            // Contenido con scroll vertical
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let available_width = ui.available_width();

                    // Responsive: si el ancho es menor a 680px, apilar en 1 columna
                    if available_width < 680.0 {
                        self.render_generator_section(ui);
                        ui.add_space(10.0);
                        self.render_preview_and_download_section(ui);
                    } else {
                        // Modo 2 columnas proporcionadas
                        ui.columns(2, |columns| {
                            columns[0].vertical(|ui| {
                                self.render_generator_section(ui);
                            });
                            columns[1].vertical(|ui| {
                                self.render_preview_and_download_section(ui);
                            });
                        });
                    }

                    ui.add_space(20.0);
                });
        });
    }
}
