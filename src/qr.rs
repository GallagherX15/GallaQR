use eframe::egui::{Color32, ColorImage};
use image::{ImageBuffer, Rgb, RgbImage, Rgba, RgbaImage};
use qrcode::{Color, EcLevel, QrCode};
use std::io::Cursor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCorrection {
    Low,      // ~7%
    Medium,   // ~15%
    Quartile, // ~25%
    High,     // ~30%
}

impl ErrorCorrection {
    pub fn to_ec_level(self) -> EcLevel {
        match self {
            Self::Low => EcLevel::L,
            Self::Medium => EcLevel::M,
            Self::Quartile => EcLevel::Q,
            Self::High => EcLevel::H,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "L (7% corrección)",
            Self::Medium => "M (15% corrección)",
            Self::Quartile => "Q (25% corrección)",
            Self::High => "H (30% corrección)",
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Low => "L",
            Self::Medium => "M",
            Self::Quartile => "Q",
            Self::High => "H",
        }
    }

    pub fn from_code(s: &str) -> Self {
        match s {
            "L" => Self::Low,
            "Q" => Self::Quartile,
            "H" => Self::High,
            _ => Self::Medium,
        }
    }
}

pub struct QrGenerator;

impl QrGenerator {
    pub fn generate_matrix(content: &str, ec: ErrorCorrection) -> Result<QrCode, String> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Err("El contenido o enlace no puede estar vacío.".to_string());
        }
        QrCode::with_error_correction_level(trimmed.as_bytes(), ec.to_ec_level())
            .map_err(|e| format!("Error al generar QR: {e}"))
    }

    /// Genera los bytes RGBA sin comprimir para copiar al portapapeles o procesar.
    pub fn generate_raw_rgba(
        qr: &QrCode,
        width: u32,
        height: u32,
        margin_modules: u32,
        fg_rgba: [u8; 4],
        bg_rgba: Option<[u8; 4]>,
    ) -> Result<Vec<u8>, String> {
        let qr_width = qr.width() as u32;
        let total_modules = qr_width + margin_modules * 2;

        if width == 0 || height == 0 {
            return Err("Las dimensiones deben ser mayores a cero.".to_string());
        }

        let scale = (width.min(height) as f64) / (total_modules as f64);
        let qr_pixel_size = scale * (total_modules as f64);
        let offset_x = ((width as f64) - qr_pixel_size) / 2.0;
        let offset_y = ((height as f64) - qr_pixel_size) / 2.0;

        let initial_bg = bg_rgba.unwrap_or([0, 0, 0, 0]);
        let mut img: RgbaImage = ImageBuffer::from_pixel(width, height, Rgba(initial_bg));
        let fg_pixel = Rgba(fg_rgba);

        let colors = qr.to_colors();
        for y in 0..qr_width {
            for x in 0..qr_width {
                let idx = (y * qr_width + x) as usize;
                if colors[idx] == Color::Dark {
                    let px_start = ((offset_x + ((x + margin_modules) as f64) * scale).round() as u32).min(width);
                    let px_end = ((offset_x + ((x + margin_modules + 1) as f64) * scale).round() as u32).min(width);
                    let py_start = ((offset_y + ((y + margin_modules) as f64) * scale).round() as u32).min(height);
                    let py_end = ((offset_y + ((y + margin_modules + 1) as f64) * scale).round() as u32).min(height);

                    for py in py_start..py_end {
                        for px in px_start..px_end {
                            img.put_pixel(px, py, fg_pixel);
                        }
                    }
                }
            }
        }

        Ok(img.into_raw())
    }

    /// Genera una imagen PNG (con o sin fondo).
    pub fn generate_png(
        qr: &QrCode,
        width: u32,
        height: u32,
        margin_modules: u32,
        fg_rgba: [u8; 4],
        bg_rgba: Option<[u8; 4]>,
    ) -> Result<Vec<u8>, String> {
        let raw_bytes = Self::generate_raw_rgba(qr, width, height, margin_modules, fg_rgba, bg_rgba)?;
        let img: RgbaImage = ImageBuffer::from_raw(width, height, raw_bytes)
            .ok_or_else(|| "Error creando búfer de imagen PNG".to_string())?;

        let mut bytes: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&mut bytes);
        img.write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| format!("Error codificando PNG: {e}"))?;

        Ok(bytes)
    }

    /// Genera una imagen JPEG con fondo sólido.
    pub fn generate_jpeg(
        qr: &QrCode,
        width: u32,
        height: u32,
        margin_modules: u32,
        fg_rgb: [u8; 3],
        bg_rgb: [u8; 3],
    ) -> Result<Vec<u8>, String> {
        let qr_width = qr.width() as u32;
        let total_modules = qr_width + margin_modules * 2;

        if width == 0 || height == 0 {
            return Err("Las dimensiones deben ser mayores a cero.".to_string());
        }

        let scale = (width.min(height) as f64) / (total_modules as f64);
        let qr_pixel_size = scale * (total_modules as f64);
        let offset_x = ((width as f64) - qr_pixel_size) / 2.0;
        let offset_y = ((height as f64) - qr_pixel_size) / 2.0;

        let mut img: RgbImage = ImageBuffer::from_pixel(width, height, Rgb(bg_rgb));
        let fg_pixel = Rgb(fg_rgb);

        let colors = qr.to_colors();
        for y in 0..qr_width {
            for x in 0..qr_width {
                let idx = (y * qr_width + x) as usize;
                if colors[idx] == Color::Dark {
                    let px_start = ((offset_x + ((x + margin_modules) as f64) * scale).round() as u32).min(width);
                    let px_end = ((offset_x + ((x + margin_modules + 1) as f64) * scale).round() as u32).min(width);
                    let py_start = ((offset_y + ((y + margin_modules) as f64) * scale).round() as u32).min(height);
                    let py_end = ((offset_y + ((y + margin_modules + 1) as f64) * scale).round() as u32).min(height);

                    for py in py_start..py_end {
                        for px in px_start..px_end {
                            img.put_pixel(px, py, fg_pixel);
                        }
                    }
                }
            }
        }

        let mut bytes: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&mut bytes);
        img.write_to(&mut cursor, image::ImageFormat::Jpeg)
            .map_err(|e| format!("Error codificando JPG: {e}"))?;

        Ok(bytes)
    }

    /// Genera un archivo SVG vectorial (sin fondo si bg_hex es None, o con fondo rectangular si bg_hex es Some).
    pub fn generate_svg(
        qr: &QrCode,
        width: u32,
        height: u32,
        margin_modules: u32,
        fg_hex: &str,
        bg_hex: Option<&str>,
    ) -> String {
        let qr_width = qr.width() as u32;
        let total_modules = qr_width + margin_modules * 2;

        let scale = (width.min(height) as f64) / (total_modules as f64);
        let qr_pixel_size = scale * (total_modules as f64);
        let offset_x = ((width as f64) - qr_pixel_size) / 2.0;
        let offset_y = ((height as f64) - qr_pixel_size) / 2.0;

        let colors = qr.to_colors();
        let mut path_data = String::new();

        for y in 0..qr_width {
            for x in 0..qr_width {
                let idx = (y * qr_width + x) as usize;
                if colors[idx] == Color::Dark {
                    let px = offset_x + ((x + margin_modules) as f64) * scale;
                    let py = offset_y + ((y + margin_modules) as f64) * scale;
                    let w = scale;
                    let h = scale;
                    path_data.push_str(&format!(
                        "M{:.2},{:.2}h{:.2}v{:.2}h-{:.2}z ",
                        px, py, w, h, w
                    ));
                }
            }
        }

        let bg_element = match bg_hex {
            Some(hex) => format!(r#"  <rect width="100%" height="100%" fill="{hex}" />"#),
            None => String::new(),
        };

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}" shape-rendering="crispEdges">
{bg_element}
  <path d="{path_data}" fill="{fg_hex}" />
</svg>"#
        )
    }

    /// Genera un `ColorImage` para la previsualización interactiva en egui.
    pub fn generate_preview_image(
        qr: &QrCode,
        margin_modules: u32,
        fg_rgba: [u8; 4],
        bg_rgba: Option<[u8; 4]>,
    ) -> ColorImage {
        let qr_width = qr.width() as u32;
        let total_modules = qr_width + margin_modules * 2;
        let module_size = 10u32;
        let total_pixels = total_modules * module_size;

        let bg_color = match bg_rgba {
            Some(c) => Color32::from_rgba_premultiplied(c[0], c[1], c[2], c[3]),
            None => Color32::TRANSPARENT,
        };

        let mut pixels = vec![bg_color; (total_pixels * total_pixels) as usize];
        let fg_color = Color32::from_rgba_premultiplied(
            fg_rgba[0], fg_rgba[1], fg_rgba[2], fg_rgba[3],
        );

        let colors = qr.to_colors();
        for y in 0..qr_width {
            for x in 0..qr_width {
                let idx = (y * qr_width + x) as usize;
                if colors[idx] == Color::Dark {
                    let px_start = (x + margin_modules) * module_size;
                    let py_start = (y + margin_modules) * module_size;

                    for py in py_start..(py_start + module_size) {
                        for px in px_start..(px_start + module_size) {
                            pixels[(py * total_pixels + px) as usize] = fg_color;
                        }
                    }
                }
            }
        }

        ColorImage::new([total_pixels as usize, total_pixels as usize], pixels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_generation_success() {
        let qr = QrGenerator::generate_matrix("https://github.com", ErrorCorrection::Medium);
        assert!(qr.is_ok());
        let qr = qr.unwrap();
        assert!(qr.width() > 0);
    }

    #[test]
    fn test_qr_generation_empty_error() {
        let qr = QrGenerator::generate_matrix("   ", ErrorCorrection::Medium);
        assert!(qr.is_err());
    }

    #[test]
    fn test_svg_transparency_and_structure() {
        let qr = QrGenerator::generate_matrix("https://gallaqr.app", ErrorCorrection::High).unwrap();
        // SVG sin fondo
        let svg_transparent = QrGenerator::generate_svg(&qr, 512, 512, 2, "#123456", None);
        assert!(svg_transparent.contains("<svg"));
        assert!(svg_transparent.contains(r#"viewBox="0 0 512 512""#));
        assert!(svg_transparent.contains(r##"fill="#123456""##));
        assert!(!svg_transparent.contains("<rect width=\"100%\""));

        // SVG con fondo
        let svg_with_bg = QrGenerator::generate_svg(&qr, 512, 512, 2, "#000000", Some("#ffffff"));
        assert!(svg_with_bg.contains(r##"<rect width="100%" height="100%" fill="#ffffff" />"##));
    }

    #[test]
    fn test_png_transparency_and_alpha() {
        let qr = QrGenerator::generate_matrix("https://ejemplo.com", ErrorCorrection::Medium).unwrap();
        let bytes = QrGenerator::generate_png(&qr, 256, 256, 2, [0, 0, 0, 255], None).unwrap();

        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);

        let dyn_img = image::load_from_memory(&bytes).expect("Failed to decode PNG");
        let rgba = dyn_img.to_rgba8();
        assert_eq!(rgba.width(), 256);
        assert_eq!(rgba.height(), 256);

        // Sin fondo: esquina transparente (alfa = 0)
        let corner_pixel = rgba.get_pixel(0, 0);
        assert_eq!(corner_pixel[3], 0);
    }

    #[test]
    fn test_jpeg_generation() {
        let qr = QrGenerator::generate_matrix("https://ejemplo.com", ErrorCorrection::Medium).unwrap();
        let bytes = QrGenerator::generate_jpeg(&qr, 256, 256, 2, [0, 0, 0], [255, 255, 255]).unwrap();
        assert!(!bytes.is_empty());
        // Check JPEG magic numbers 0xFF 0xD8
        assert_eq!(&bytes[0..2], &[0xFF, 0xD8]);

        let dyn_img = image::load_from_memory(&bytes).expect("Failed to decode JPEG");
        assert_eq!(dyn_img.width(), 256);
        assert_eq!(dyn_img.height(), 256);
    }
}
