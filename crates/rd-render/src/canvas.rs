//! Canvas vetorial do snapshot (M4): retângulos e texto com shaping real.
//!
//! Desde a M4 o `render_snapshot` pinta sobre um `tiny_skia::Pixmap`
//! (RGBA premultiplicado, rasterização anti-aliased de caminhos) e o texto é
//! *shaped* pelo `rustybuzz` (HarfBuzz em Rust puro: kerning, ligaduras e
//! advances da fonte) com contornos extraídos pelo `ttf-parser`.
//!
//! Fonte: DejaVu Sans (licença Bitstream Vera, ver `assets/`), a mesma
//! família do atlas da Canvas da VM. Não é a fonte do Android (Roboto): o
//! objetivo é um texto legível e *determinístico*, não fidelidade pixel a
//! pixel com um aparelho real.
//!
//! Determinismo: mesmo input e mesma plataforma → mesmos bytes. Entre
//! plataformas diferentes o ponto flutuante do tiny-skia pode diferir em
//! 1 LSB; os goldens são travados no CI Linux.

use crate::shot::{Framebuffer, Rgb};
use rustybuzz::{Face, UnicodeBuffer};
use std::sync::OnceLock;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Rect, Transform};

/// Fonte da UI do snapshot (DejaVu Sans, regular).
pub const UI_FONT_TTF: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");

/// Tamanho do texto da UI em pixels (mesma escala visual da versão de atlas).
pub const UI_TEXT_PX: f32 = 18.0;

/// Padding horizontal do texto dentro da view (px).
const TEXT_PAD_X: i32 = 4;

fn ui_face() -> Option<&'static Face<'static>> {
    static FACE: OnceLock<Option<Face<'static>>> = OnceLock::new();
    FACE.get_or_init(|| Face::from_slice(UI_FONT_TTF, 0))
        .as_ref()
}

/// Canvas RGBA premultiplicado com fundo opaco.
pub struct Canvas {
    pix: Pixmap,
}

impl Canvas {
    /// `None` se a dimensão for inválida para um `Pixmap` (zero ou gigante).
    pub fn new(bg: Rgb, w: usize, h: usize) -> Option<Self> {
        let (w, h) = (u32::try_from(w).ok()?, u32::try_from(h).ok()?);
        let mut pix = Pixmap::new(w, h)?;
        pix.fill(tiny_skia::Color::from_rgba8(bg.0, bg.1, bg.2, 255));
        Some(Canvas { pix })
    }

    /// Retângulo sólido, recortado ao canvas (bounds hostis não panicam).
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb) {
        if w <= 0 || h <= 0 {
            return;
        }
        let (cw, ch) = (self.pix.width() as i32, self.pix.height() as i32);
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = x.saturating_add(w).min(cw);
        let y1 = y.saturating_add(h).min(ch);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        let Some(rect) = Rect::from_xywh(x0 as f32, y0 as f32, (x1 - x0) as f32, (y1 - y0) as f32)
        else {
            return;
        };
        let mut paint = Paint::default();
        paint.set_color_rgba8(c.0, c.1, c.2, 255);
        paint.anti_alias = false; // retângulos de view são alinhados ao pixel
        self.pix
            .fill_rect(rect, &paint, Transform::identity(), None);
    }

    /// Contorno de 1 px (mesma geometria do `Framebuffer::rect_border`).
    pub fn rect_border(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb) {
        if w <= 0 || h <= 0 {
            return;
        }
        self.fill_rect(x, y, w, 1, c);
        self.fill_rect(x, y.saturating_add(h.saturating_sub(1)), w, 1, c);
        self.fill_rect(x, y, 1, h, c);
        self.fill_rect(x.saturating_add(w.saturating_sub(1)), y, 1, h, c);
    }

    /// Texto de uma linha, centralizado verticalmente na caixa `(x, y, _, h)`,
    /// com shaping do rustybuzz e contornos preenchidos pelo tiny-skia.
    pub fn text_in_box(&mut self, x: i32, y: i32, h: i32, s: &str, c: Rgb) {
        let Some(face) = ui_face() else {
            return;
        };
        if s.is_empty() {
            return;
        }
        let upem = face.units_per_em() as f32;
        if upem <= 0.0 {
            return;
        }
        let scale = UI_TEXT_PX / upem;
        let ascent = face.ascender() as f32 * scale;
        // topo da linha = (h-18)/2 dentro da caixa (mesmo critério do atlas);
        // baseline = topo + ascender da fonte.
        let box_top = y.saturating_add(h.saturating_sub(18).max(0) / 2) as f32;
        let baseline = box_top + ascent;
        let pen_x0 = x.saturating_add(TEXT_PAD_X) as f32;

        let mut buf = UnicodeBuffer::new();
        buf.push_str(s);
        buf.guess_segment_properties();
        let shaped = rustybuzz::shape(face, &[], buf);

        let mut paint = Paint::default();
        paint.set_color_rgba8(c.0, c.1, c.2, 255);
        paint.anti_alias = true;

        let mut pen_x = pen_x0;
        for (info, pos) in shaped
            .glyph_infos()
            .iter()
            .zip(shaped.glyph_positions().iter())
        {
            let gx = pen_x + pos.x_offset as f32 * scale;
            let gy = baseline - pos.y_offset as f32 * scale;
            let mut outline = GlyphPath {
                pb: PathBuilder::new(),
                ox: gx,
                oy: gy,
                s: scale,
                any: false,
            };
            let _ = face.outline_glyph(ttf_parser_glyph(info.glyph_id as u16), &mut outline);
            if outline.any {
                if let Some(path) = outline.pb.finish() {
                    self.pix.fill_path(
                        &path,
                        &paint,
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                }
            }
            pen_x += pos.x_advance as f32 * scale;
        }
    }

    /// Converte para o `Framebuffer` RGB compacto usado pelo PNG e pelo agente.
    /// O fundo é opaco, então o premultiplicado coincide com o RGB.
    pub fn into_framebuffer(self) -> Framebuffer {
        let (w, h) = (self.pix.width() as usize, self.pix.height() as usize);
        let rgba = self.pix.data();
        let mut px = Vec::with_capacity(w * h * 3);
        let (pixels, _) = rgba.as_chunks::<4>();
        for p in pixels {
            px.extend_from_slice(&p[..3]);
        }
        Framebuffer { w, h, px }
    }
}

fn ttf_parser_glyph(id: u16) -> rustybuzz::ttf_parser::GlyphId {
    rustybuzz::ttf_parser::GlyphId(id)
}

/// Adaptador ttf-parser → tiny-skia: coordenadas da fonte (y para cima) viram
/// pixels do canvas (y para baixo), com origem e escala do glifo.
struct GlyphPath {
    pb: PathBuilder,
    ox: f32,
    oy: f32,
    s: f32,
    any: bool,
}

impl GlyphPath {
    fn map(&self, x: f32, y: f32) -> (f32, f32) {
        (self.ox + x * self.s, self.oy - y * self.s)
    }
}

impl rustybuzz::ttf_parser::OutlineBuilder for GlyphPath {
    fn move_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.map(x, y);
        self.pb.move_to(x, y);
        self.any = true;
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.map(x, y);
        self.pb.line_to(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (x1, y1) = self.map(x1, y1);
        let (x, y) = self.map(x, y);
        self.pb.quad_to(x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (x1, y1) = self.map(x1, y1);
        let (x2, y2) = self.map(x2, y2);
        let (x, y) = self.map(x, y);
        self.pb.cubic_to(x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        self.pb.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_loads_and_shapes_latin_text() {
        let face = ui_face().expect("DejaVuSans.ttf parseável");
        let mut buf = UnicodeBuffer::new();
        buf.push_str("Olá");
        buf.guess_segment_properties();
        let out = rustybuzz::shape(face, &[], buf);
        assert_eq!(out.len(), 3);
        // glifo 0 (.notdef) não pode aparecer para letras latinas comuns
        assert!(out.glyph_infos().iter().all(|g| g.glyph_id != 0));
    }

    #[test]
    fn text_paints_dark_pixels_on_light_background() {
        let mut c = Canvas::new((255, 255, 255), 200, 60).unwrap();
        c.text_in_box(0, 0, 60, "Hello", (0, 0, 0));
        let fb = c.into_framebuffer();
        let dark = fb
            .px
            .as_chunks::<3>()
            .0
            .iter()
            .filter(|p| p[0] < 128)
            .count();
        assert!(dark > 100, "texto deve pintar pixels (got {dark})");
    }

    #[test]
    fn hostile_rect_bounds_do_not_panic() {
        let mut c = Canvas::new((0, 0, 0), 16, 16).unwrap();
        c.fill_rect(i32::MAX, i32::MAX, i32::MAX, i32::MAX, (255, 0, 0));
        c.fill_rect(i32::MIN, i32::MIN, 4, 4, (255, 0, 0));
        c.rect_border(-5, -5, 100, 100, (0, 255, 0));
        c.text_in_box(i32::MAX, i32::MIN, i32::MAX, "x", (1, 2, 3));
        let fb = c.into_framebuffer();
        assert_eq!(fb.px.len(), 16 * 16 * 3);
    }

    #[test]
    fn zero_or_huge_canvas_is_rejected_not_panicking() {
        assert!(Canvas::new((0, 0, 0), 0, 10).is_none());
        assert!(Canvas::new((0, 0, 0), usize::MAX, 10).is_none());
    }
}
