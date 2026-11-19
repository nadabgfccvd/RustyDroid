//! `<shape>` XML drawables → `android.graphics.drawable.GradientDrawable`.
//!
//! ## Por que isto existe
//!
//! Botões, cards e fundos arredondados de quase todo app Android são
//! `res/drawable/*.xml` com raiz `<shape>`. Antes daqui, `getDrawable` de um
//! `<shape>` devolvia `NOT_IMPLEMENTED` e `android:background="@drawable/x"`
//! ficava sem fundo, então a tela perdia cor, borda e cantos.
//!
//! ## O que é modelado
//!
//! * `android:shape` = `rectangle` (default), `oval` e `line`;
//! * `<solid android:color>`;
//! * `<stroke android:width android:color>` (contorno contínuo);
//! * `<corners android:radius>` e os quatro cantos individuais;
//! * `<size android:width android:height>` (tamanho intrínseco).
//!
//! Fora do modelo, registrado como gap estruturado (`rd_ggaps`), nunca em
//! silêncio: `ring`, `<gradient>` (desenhado com a cor inicial), traço
//! tracejado (`dashWidth`), `<padding>` e cores por estado.
//!
//! ## Como o desenho acontece
//!
//! O renderer só tem polígonos (`OpK::Path`). O contorno do retângulo
//! arredondado e da elipse vira polilinha aqui ([`outline`]). O traço é um anel
//! (contorno externo + contorno interno com `even_odd`), como o AOSP desenha o
//! stroke centrado na borda do retângulo recuado em meia largura.

use rd_apk::axml::{self, AttrValue, ComplexUnit, XmlElement};

use crate::vector_drawable::{color_attr, RawColor};

/// `GradientDrawable.RECTANGLE` … `RING` (mesmos valores do AOSP).
pub const RECTANGLE: i32 = 0;
pub const OVAL: i32 = 1;
pub const LINE: i32 = 2;
pub const RING: i32 = 3;

/// Amostras por canto arredondado (quarto de círculo).
const CORNER_STEPS: usize = 8;
/// Amostras da elipse inteira.
const OVAL_STEPS: usize = 48;

/// Dimensão ainda não resolvida: literal com unidade ou referência a
/// `@dimen`. A conversão para px usa a densidade de `Resources`, no chamador.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RawDim {
    Lit(f32, ComplexUnit),
    Ref(u32),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RawShape {
    pub shape: i32,
    pub solid: Option<RawColor>,
    pub stroke: Option<(RawColor, RawDim)>,
    /// `<corners android:radius>`.
    pub radius: Option<RawDim>,
    /// Cantos individuais na ordem top-left, top-right, bottom-right,
    /// bottom-left; sobrescrevem `radius` quando presentes.
    pub corners: [Option<RawDim>; 4],
    pub size: (Option<RawDim>, Option<RawDim>),
    pub gaps: Vec<String>,
}

/// Lê um `<shape>` binário (AXML dentro do APK). `None` quando a raiz não é
/// `<shape>`.
pub fn parse_shape(bytes: &[u8]) -> Option<RawShape> {
    let doc = axml::parse(bytes).ok()?;
    let root = &doc.root;
    if root.name != "shape" {
        return None;
    }
    let mut out = RawShape {
        shape: shape_kind(root),
        ..RawShape::default()
    };
    if out.shape == RING {
        out.gaps
            .push("android:shape=\"ring\" não é modelado; nada é desenhado".into());
    }
    for child in &root.children {
        match child.name.as_str() {
            "solid" => {
                out.solid = color_attr(child, "color");
                if out.solid.is_none() && child.android_attr("color").is_some() {
                    out.gaps
                        .push("<solid android:color> não é cor literal nem @color".into());
                }
            }
            "stroke" => {
                let width = dim_attr(child, "width").unwrap_or(RawDim::Lit(0.0, ComplexUnit::Px));
                if let Some(c) = color_attr(child, "color") {
                    out.stroke = Some((c, width));
                }
                if child.android_attr("dashWidth").is_some() {
                    out.gaps
                        .push("traço tracejado (dashWidth) desenhado contínuo".into());
                }
            }
            "corners" => {
                out.radius = dim_attr(child, "radius");
                out.corners = [
                    dim_attr(child, "topLeftRadius"),
                    dim_attr(child, "topRightRadius"),
                    dim_attr(child, "bottomRightRadius"),
                    dim_attr(child, "bottomLeftRadius"),
                ];
            }
            "size" => {
                out.size = (dim_attr(child, "width"), dim_attr(child, "height"));
            }
            "gradient" => {
                // A cor inicial é o melhor representante sólido; o gradiente em
                // si não é interpolado pelo renderer.
                if out.solid.is_none() {
                    out.solid = color_attr(child, "startColor");
                }
                out.gaps
                    .push("<gradient> desenhado com a cor inicial (sem interpolação)".into());
            }
            "padding" => out
                .gaps
                .push("<padding> do shape não altera o padding da view".into()),
            other => out
                .gaps
                .push(format!("elemento <{other}> não modelado em <shape>")),
        }
    }
    Some(out)
}

fn shape_kind(root: &XmlElement) -> i32 {
    match root.android_attr("shape").map(|a| &a.value) {
        Some(AttrValue::Int(i)) => *i,
        Some(AttrValue::String(s)) => match s.trim() {
            "oval" => OVAL,
            "line" => LINE,
            "ring" => RING,
            _ => RECTANGLE,
        },
        _ => RECTANGLE,
    }
}

fn dim_attr(el: &XmlElement, name: &str) -> Option<RawDim> {
    let a = el.android_attr(name)?;
    match &a.value {
        AttrValue::Dimension(v, unit) => Some(RawDim::Lit(*v, *unit)),
        AttrValue::Float(f) => Some(RawDim::Lit(*f, ComplexUnit::Px)),
        AttrValue::Int(i) => Some(RawDim::Lit(*i as f32, ComplexUnit::Px)),
        AttrValue::Reference(r) => Some(RawDim::Ref(*r)),
        _ => None,
    }
}

/// Contorno fechado de um retângulo (cantos `radii` = TL, TR, BR, BL) ou de
/// uma elipse inscrita em `(l, t, r, b)`.
pub fn outline(shape: i32, l: f32, t: f32, r: f32, b: f32, radii: [f32; 4]) -> Vec<(f32, f32)> {
    let (w, h) = (r - l, b - t);
    if w <= 0.0 || h <= 0.0 {
        return Vec::new();
    }
    if shape == OVAL {
        let (cx, cy, rx, ry) = (l + w / 2.0, t + h / 2.0, w / 2.0, h / 2.0);
        return (0..OVAL_STEPS)
            .map(|i| {
                let a = i as f32 / OVAL_STEPS as f32 * std::f32::consts::TAU;
                (cx + rx * a.cos(), cy + ry * a.sin())
            })
            .collect();
    }
    // AOSP limita o raio à metade do menor lado.
    let max = w.min(h) / 2.0;
    let [tl, tr, br, bl] = radii.map(|v| v.clamp(0.0, max));
    if tl == 0.0 && tr == 0.0 && br == 0.0 && bl == 0.0 {
        return vec![(l, t), (r, t), (r, b), (l, b)];
    }
    let mut pts = Vec::with_capacity(4 * (CORNER_STEPS + 1));
    // Cada canto: centro do arco e ângulo inicial (sentido horário em y-baixo).
    let corners = [
        (l + tl, t + tl, tl, std::f32::consts::PI),
        (r - tr, t + tr, tr, 1.5 * std::f32::consts::PI),
        (r - br, b - br, br, 0.0),
        (l + bl, b - bl, bl, 0.5 * std::f32::consts::PI),
    ];
    for (cx, cy, rad, start) in corners {
        if rad == 0.0 {
            pts.push((cx, cy));
            continue;
        }
        for i in 0..=CORNER_STEPS {
            let a = start + i as f32 / CORNER_STEPS as f32 * std::f32::consts::FRAC_PI_2;
            pts.push((cx + rad * a.cos(), cy + rad * a.sin()));
        }
    }
    pts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_corners_are_four_points() {
        let p = outline(RECTANGLE, 0.0, 0.0, 10.0, 20.0, [0.0; 4]);
        assert_eq!(p, vec![(0.0, 0.0), (10.0, 0.0), (10.0, 20.0), (0.0, 20.0)]);
    }

    #[test]
    fn rounded_corners_stay_inside_bounds_and_touch_edges() {
        let p = outline(RECTANGLE, 10.0, 10.0, 110.0, 60.0, [8.0; 4]);
        assert_eq!(p.len(), 4 * (CORNER_STEPS + 1));
        let eps = 1e-3;
        for (x, y) in &p {
            assert!(*x >= 10.0 - eps && *x <= 110.0 + eps, "x={x}");
            assert!(*y >= 10.0 - eps && *y <= 60.0 + eps, "y={y}");
        }
        // O arco do canto superior esquerdo termina no topo, 8 px à direita.
        let (x, y) = p[CORNER_STEPS];
        assert!(
            (x - 18.0).abs() < eps && (y - 10.0).abs() < eps,
            "({x},{y})"
        );
        // O canto exato não é coberto (o raio arredonda).
        assert!(!p.contains(&(10.0, 10.0)));
    }

    #[test]
    fn radius_is_clamped_to_half_the_shortest_side() {
        // Raio 999 num 40x20 → pílula de raio 10.
        let p = outline(RECTANGLE, 0.0, 0.0, 40.0, 20.0, [999.0; 4]);
        let min_x = p.iter().map(|q| q.0).fold(f32::MAX, f32::min);
        let max_y = p.iter().map(|q| q.1).fold(f32::MIN, f32::max);
        assert!(min_x >= -1e-3 && max_y <= 20.0 + 1e-3);
        let (x, y) = p[CORNER_STEPS];
        assert!((x - 10.0).abs() < 1e-3 && y.abs() < 1e-3, "({x},{y})");
    }

    #[test]
    fn oval_is_inscribed_in_bounds() {
        let p = outline(OVAL, 0.0, 0.0, 100.0, 50.0, [0.0; 4]);
        assert_eq!(p.len(), OVAL_STEPS);
        assert!((p[0].0 - 100.0).abs() < 1e-3 && (p[0].1 - 25.0).abs() < 1e-3);
        for (x, y) in p {
            let nx = (x - 50.0) / 50.0;
            let ny = (y - 25.0) / 25.0;
            assert!((nx * nx + ny * ny - 1.0).abs() < 1e-3);
        }
    }

    #[test]
    fn empty_bounds_have_no_outline() {
        assert!(outline(RECTANGLE, 5.0, 5.0, 5.0, 9.0, [0.0; 4]).is_empty());
        assert!(outline(OVAL, 0.0, 0.0, 10.0, -1.0, [0.0; 4]).is_empty());
    }
}
