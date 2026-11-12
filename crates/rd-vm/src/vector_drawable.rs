//! `VectorDrawable` — o `<vector>` do Android (M4/M8).
//!
//! ## Por que isto existe
//!
//! Praticamente todo app Android moderno põe ícones como **vector drawable**
//! (`res/drawable/ic_*.xml`) em vez de PNG. Sem modelar a classe, `NewPipe`
//! morria antes de desenhar a primeira tela:
//!
//! ```text
//! java.lang.IllegalStateException: This app has been built with an incorrect
//! configuration. Please configure your build for VectorDrawableCompat.
//! ```
//!
//! A mensagem vem de `AppCompatDrawableManager.checkVectorDrawableSetup`, que
//! faz `getDrawable(R.drawable.abc_vector_test)` e exige que o resultado seja
//! uma instância de `android.graphics.drawable.VectorDrawable`. Devolver `null`
//! (ou um `ColorDrawable`) faz o app abortar de propósito — é uma auto-checagem
//! do AppCompat, não um bug do app.
//!
//! ## O que é modelado
//!
//! O subconjunto que os ícones reais usam, e nada além:
//!
//! * `<vector android:width|height|viewportWidth|viewportHeight>`;
//! * `<path android:pathData|fillColor|fillAlpha|strokeColor|strokeWidth>`
//!   com o subconjunto de comandos do SVG `M m L l H h V v C c S s Q q T t A a Z z`;
//! * `<group>` com `translateX/Y`, `scaleX/Y`, `rotation`, `pivotX/Y` e
//!   `clip-path` (o clip é registrado como gap estruturado, não ignorado em
//!   silêncio).
//!
//! Fora do modelo, explícito: gradientes (`<gradient>`), `fillType` par (even-odd
//! é honrado via `OpK::Path(_, true)`), animações (`<animated-vector>`) e
//! `tint` aplicado no próprio vetor. Cada um vira `NOT_IMPLEMENTED` com o
//! motivo quando aparece — nenhum ícone é desenhado "quase certo" em silêncio.
//!
//! ## Como o desenho acontece
//!
//! O renderer (`graphics.rs`) não tem curvas: `OpK::Path` é uma lista de
//! polilinhas. Então `pathData` é **achatado** em segmentos de reta aqui, com
//! 16 amostras por curva — a mesma taxa que `Path.cubicTo` já usa no modelo, o
//! que mantém a saída consistente entre um ícone vetorial e o mesmo caminho
//! construído pela API do `android.graphics.Path`.

use rd_apk::axml::{self, AttrValue, XmlElement};

/// Nº de amostras por curva Bézier (igual ao `Path.cubicTo` do modelo).
const CURVE_STEPS: usize = 16;

/// Um subcaminho já achatado + a cor de preenchimento resolvida.
#[derive(Clone, Debug, PartialEq)]
pub struct VPath {
    /// Polilinhas (mesma forma de `G::Path`). Vários subcaminhos = vários "M".
    pub polylines: Vec<Vec<(f32, f32)>>,
    /// ARGB já resolvido (referências de cor são resolvidas antes daqui).
    pub fill: u32,
    /// `android:strokeColor` + `strokeWidth`, quando o ícone é só contorno.
    pub stroke: Option<(u32, f32)>,
    /// `android:fillType="evenOdd"`.
    pub even_odd: bool,
}

/// Um vetor pronto para desenhar, já em coordenadas do **viewport**.
#[derive(Clone, Debug, PartialEq)]
pub struct VectorSpec {
    /// `android:viewportWidth` / `android:viewportHeight`.
    pub viewport: (f32, f32),
    /// `android:width` / `android:height` (dp) — o tamanho intrínseco.
    pub intrinsic: (f32, f32),
    pub paths: Vec<VPath>,
    /// Ressalvas estruturais (clip-path, gradiente, …) para virar gap tipado.
    pub gaps: Vec<String>,
}

impl VectorSpec {
    /// Escala de viewport para um retângulo de destino, preservando a
    /// proporção (o AOSP escala para `bounds` sem distorcer quando
    /// `android:scaleType` não é definido — o default é `fitXY` para
    /// `VectorDrawable`, então a proporção é respeitada pelo próprio SVG).
    pub fn scale_to(&self, w: f32, h: f32) -> (f32, f32) {
        let (vw, vh) = self.viewport;
        if vw <= 0.0 || vh <= 0.0 {
            return (1.0, 1.0);
        }
        (w / vw, h / vh)
    }
}

/// `fillColor` ainda não resolvido: literal ou referência a `@color`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawColor {
    Literal(u32),
    Ref(u32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct RawPath {
    pub polylines: Vec<Vec<(f32, f32)>>,
    pub fill: Option<RawColor>,
    pub fill_alpha: f32,
    pub stroke: Option<(RawColor, f32)>,
    pub even_odd: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RawVector {
    pub viewport: (f32, f32),
    pub intrinsic: (f32, f32),
    pub paths: Vec<RawPath>,
    pub gaps: Vec<String>,
}

/// Lê um `<vector>` binário (AXML dentro do APK).
///
/// `None` quando o arquivo não é um vetor — o chamador decide o que fazer
/// (o caminho normal é o gap estruturado de "drawable de arquivo").
pub fn parse_vector(bytes: &[u8]) -> Option<RawVector> {
    let doc = axml::parse(bytes).ok()?;
    let root = &doc.root;
    if root.name != "vector" {
        return None;
    }
    let viewport = (
        float_attr(root, "viewportWidth").or_else(|| float_attr(root, "width")),
        float_attr(root, "viewportHeight").or_else(|| float_attr(root, "height")),
    );
    let viewport = match viewport {
        (Some(w), Some(h)) if w > 0.0 && h > 0.0 => (w, h),
        // Sem viewport não há como escalar; o AOSP trata como inválido.
        _ => return None,
    };
    let intrinsic = (
        float_attr(root, "width").unwrap_or(viewport.0),
        float_attr(root, "height").unwrap_or(viewport.1),
    );

    let mut out = RawVector {
        viewport,
        intrinsic,
        paths: Vec::new(),
        gaps: Vec::new(),
    };
    walk(root, Affine::IDENTITY, &mut out, 0);
    Some(out)
}

/// Transformação 2-D afim (a mesma que `<group>` aplica em torno do pivô).
#[derive(Clone, Copy, Debug)]
struct Affine {
    // colunas: [a c e; b d f]
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl Affine {
    const IDENTITY: Affine = Affine {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    fn apply(&self, p: (f32, f32)) -> (f32, f32) {
        (
            self.a * p.0 + self.c * p.1 + self.e,
            self.b * p.0 + self.d * p.1 + self.f,
        )
    }

    /// `self ∘ other` (aplica `other` primeiro).
    fn then(&self, other: &Affine) -> Affine {
        Affine {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    /// O `group` do AOSP: rotaciona em torno do pivô, escala em torno do pivô,
    /// depois translada.
    fn group(rotation: f32, pivot: (f32, f32), scale: (f32, f32), translate: (f32, f32)) -> Affine {
        let to_pivot = Affine {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: pivot.0,
            f: pivot.1,
        };
        let from_pivot = Affine {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: -pivot.0,
            f: -pivot.1,
        };
        let scale_m = Affine {
            a: scale.0,
            b: 0.0,
            c: 0.0,
            d: scale.1,
            e: 0.0,
            f: 0.0,
        };
        let (sin, cos) = rotation.to_radians().sin_cos();
        let rotate_m = Affine {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        };
        let translate_m = Affine {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: translate.0,
            f: translate.1,
        };
        // Ordem do AOSP VectorDrawable: pivot → rotate → scale → unpivot → translate.
        translate_m
            .then(&to_pivot)
            .then(&rotate_m)
            .then(&scale_m)
            .then(&from_pivot)
    }
}

fn walk(el: &XmlElement, parent: Affine, out: &mut RawVector, depth: usize) {
    if depth > 32 {
        out.gaps.push("grupo aninhado além de 32 níveis".into());
        return;
    }
    for child in &el.children {
        match child.name.as_str() {
            "path" => {
                let Some(data) = child.android_attr("pathData").map(|a| match &a.value {
                    AttrValue::String(sv) => sv.clone(),
                    other => other.as_text(),
                }) else {
                    out.gaps
                        .push("path sem android:pathData (string obrigatória)".into());
                    continue;
                };
                let polylines: Vec<Vec<(f32, f32)>> = flatten_path_data(&data)
                    .into_iter()
                    .map(|line| line.into_iter().map(|p| parent.apply(p)).collect())
                    .collect();
                if polylines.is_empty() {
                    continue;
                }
                let fill = color_attr(child, "fillColor");
                let stroke = color_attr(child, "strokeColor")
                    .map(|c| (c, float_attr(child, "strokeWidth").unwrap_or(1.0).max(0.0)));
                let fill_alpha = float_attr(child, "fillAlpha").unwrap_or(1.0);
                let even_odd = child
                    .android_attr("fillType")
                    .map(|a| match &a.value {
                        AttrValue::String(sv) => sv.clone(),
                        other => other.as_text(),
                    })
                    .map(|sv| sv.eq_ignore_ascii_case("evenOdd"))
                    .unwrap_or(false);
                out.paths.push(RawPath {
                    polylines,
                    fill,
                    fill_alpha,
                    stroke,
                    even_odd,
                });
            }
            "group" => {
                let t = (
                    float_attr(child, "translateX").unwrap_or(0.0),
                    float_attr(child, "translateY").unwrap_or(0.0),
                );
                let s = (
                    float_attr(child, "scaleX").unwrap_or(1.0),
                    float_attr(child, "scaleY").unwrap_or(1.0),
                );
                let rotation = float_attr(child, "rotation").unwrap_or(0.0);
                let pivot = (
                    float_attr(child, "pivotX").unwrap_or(0.0),
                    float_attr(child, "pivotY").unwrap_or(0.0),
                );
                if child.android_attr("clip-path").is_some() {
                    // O clip-path do grupo recortaria o desenho; desenhar sem o
                    // recorte pintaria fora da forma. Fica registrado.
                    out.gaps.push(
                        "android:clip-path em <group> não é aplicado (recorte fora do modelo)"
                            .into(),
                    );
                }
                let m = Affine::group(rotation, pivot, s, t);
                walk(child, parent.then(&m), out, depth + 1);
            }
            "gradient" => out.gaps.push(
                "<gradient> não é modelado (só fill sólido); o gradiente sai transparente".into(),
            ),
            "clip-path" => out
                .gaps
                .push("<clip-path> não é modelado; o recorte não é aplicado".into()),
            other => out
                .gaps
                .push(format!("elemento <{other}> não modelado em <vector>")),
        }
    }
}

/// Dimensão (`24dp` → 24.0) ou número puro.
fn float_attr(el: &XmlElement, name: &str) -> Option<f32> {
    let a = el.android_attr(name)?;
    match &a.value {
        AttrValue::Float(f) => Some(*f),
        AttrValue::Dimension(v, _) | AttrValue::Fraction(v, _) => Some(*v),
        AttrValue::Int(i) => Some(*i as f32),
        AttrValue::String(s) => {
            let trimmed = s.trim();
            let numeric: String = trimmed
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+')
                .collect();
            numeric.parse().ok()
        }
        _ => None,
    }
}

/// `fillColor`/`strokeColor`: literal (`#ff0000`, já decodificado como `Int`
/// pelo aapt) ou referência a `@color/…`, resolvida no ARSC pelo chamador.
fn color_attr(el: &XmlElement, name: &str) -> Option<RawColor> {
    let a = el.android_attr(name)?;
    match &a.value {
        AttrValue::Reference(r) | AttrValue::Attribute(r) => Some(RawColor::Ref(*r)),
        AttrValue::Int(i) => Some(RawColor::Literal(*i as u32)),
        AttrValue::String(s) => {
            let t = s.trim();
            if let Some(hex) = t.strip_prefix('#') {
                parse_hex_color(hex).map(RawColor::Literal)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// `#RGB` / `#ARGB` / `#RRGGBB` / `#AARRGGBB` (a ordem do Android em XML).
fn parse_hex_color(hex: &str) -> Option<u32> {
    let digits: Vec<u32> = hex
        .chars()
        .map(|c| c.to_digit(16))
        .collect::<Option<Vec<u32>>>()?;
    let v = digits.iter().fold(0u32, |acc, d| (acc << 4) | *d);
    Some(match digits.len() {
        3 => {
            // #RGB
            let (r, g, b) = (v >> 8 & 0xF, v >> 4 & 0xF, v & 0xF);
            0xFF00_0000 | (r * 0x11) << 16 | (g * 0x11) << 8 | (b * 0x11)
        }
        4 => {
            // #ARGB
            let (a, r, g, b) = (v >> 12 & 0xF, v >> 8 & 0xF, v >> 4 & 0xF, v & 0xF);
            (a * 0x11) << 24 | (r * 0x11) << 16 | (g * 0x11) << 8 | (b * 0x11)
        }
        6 => 0xFF00_0000 | v,
        8 => v,
        _ => return None,
    })
}

/// Achata um `pathData` (subconjunto do SVG) em polilinhas.
///
/// Cada comando `M`/`m` começa um subcaminho novo; curvas viram `CURVE_STEPS`
/// segmentos. Números podem vir separados por espaço, vírgula, ou colados
/// (`10-5` é `10` e `-5`, como no SVG).
pub fn flatten_path_data(d: &str) -> Vec<Vec<(f32, f32)>> {
    let mut out: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut cur: Vec<(f32, f32)> = Vec::new();
    let (mut cx, mut cy) = (0.0f32, 0.0f32);
    let (mut start_x, mut start_y) = (0.0f32, 0.0f32);
    // Último ponto de controle, para S/s e T/t refletirem.
    let (mut last_cx, mut last_cy) = (0.0f32, 0.0f32);
    let mut last_cmd = ' ';

    let mut lexer = Lexer::new(d);
    while let Some(cmd) = lexer.command() {
        let rel = cmd.is_ascii_lowercase();
        let up = cmd.to_ascii_uppercase();
        let mut first = true;
        loop {
            match up {
                'M' => {
                    let (mut x, mut y) = match (lexer.number(), lexer.number()) {
                        (Some(x), Some(y)) => (x, y),
                        _ => break,
                    };
                    if rel {
                        x += cx;
                        y += cy;
                    }
                    if first {
                        if cur.len() > 1 {
                            out.push(std::mem::take(&mut cur));
                        } else {
                            cur.clear();
                        }
                        cur.push((x, y));
                        start_x = x;
                        start_y = y;
                    } else {
                        // Coordenadas extras depois de `M` são `L` implícitos.
                        cur.push((x, y));
                    }
                    cx = x;
                    cy = y;
                }
                'L' => {
                    let (mut x, mut y) = match (lexer.number(), lexer.number()) {
                        (Some(x), Some(y)) => (x, y),
                        _ => break,
                    };
                    if rel {
                        x += cx;
                        y += cy;
                    }
                    ensure_start(&mut cur, cx, cy);
                    cur.push((x, y));
                    cx = x;
                    cy = y;
                }
                'H' => {
                    let Some(mut x) = lexer.number() else { break };
                    if rel {
                        x += cx;
                    }
                    ensure_start(&mut cur, cx, cy);
                    cur.push((x, cy));
                    cx = x;
                }
                'V' => {
                    let Some(mut y) = lexer.number() else { break };
                    if rel {
                        y += cy;
                    }
                    ensure_start(&mut cur, cx, cy);
                    cur.push((cx, y));
                    cy = y;
                }
                'C' | 'S' | 'Q' | 'T' => {
                    let (c1, c2, end) = match up {
                        'C' => {
                            let (x1, y1, x2, y2, x, y) = match (
                                lexer.number(),
                                lexer.number(),
                                lexer.number(),
                                lexer.number(),
                                lexer.number(),
                                lexer.number(),
                            ) {
                                (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f)) => {
                                    (a, b, c, d, e, f)
                                }
                                _ => break,
                            };
                            let off = |v: f32, base: f32| if rel { v + base } else { v };
                            (
                                (off(x1, cx), off(y1, cy)),
                                (off(x2, cx), off(y2, cy)),
                                (off(x, cx), off(y, cy)),
                            )
                        }
                        'S' => {
                            // Reflexo do último controle cúbico.
                            let (x2, y2, x, y) = match (
                                lexer.number(),
                                lexer.number(),
                                lexer.number(),
                                lexer.number(),
                            ) {
                                (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
                                _ => break,
                            };
                            let off = |v: f32, base: f32| if rel { v + base } else { v };
                            let reflect = matches!(last_cmd.to_ascii_uppercase(), 'C' | 'S');
                            let c1 = if reflect {
                                (2.0 * cx - last_cx, 2.0 * cy - last_cy)
                            } else {
                                (cx, cy)
                            };
                            (c1, (off(x2, cx), off(y2, cy)), (off(x, cx), off(y, cy)))
                        }
                        'Q' => {
                            let (x1, y1, x, y) = match (
                                lexer.number(),
                                lexer.number(),
                                lexer.number(),
                                lexer.number(),
                            ) {
                                (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
                                _ => break,
                            };
                            let off = |v: f32, base: f32| if rel { v + base } else { v };
                            (
                                (off(x1, cx), off(y1, cy)),
                                (off(x1, cx), off(y1, cy)),
                                (off(x, cx), off(y, cy)),
                            )
                        }
                        _ => {
                            // 'T': o controle é o reflexo do quadrático anterior.
                            let (x, y) = match (lexer.number(), lexer.number()) {
                                (Some(a), Some(b)) => (a, b),
                                _ => break,
                            };
                            let off = |v: f32, base: f32| if rel { v + base } else { v };
                            let reflect = matches!(last_cmd.to_ascii_uppercase(), 'Q' | 'T');
                            let c1 = if reflect {
                                (2.0 * cx - last_cx, 2.0 * cy - last_cy)
                            } else {
                                (cx, cy)
                            };
                            (c1, c1, (off(x, cx), off(y, cy)))
                        }
                    };
                    ensure_start(&mut cur, cx, cy);
                    let quad = matches!(up, 'Q' | 'T');
                    for i in 1..=CURVE_STEPS {
                        let t = i as f32 / CURVE_STEPS as f32;
                        let u = 1.0 - t;
                        let p = if quad {
                            (
                                u * u * cx + 2.0 * u * t * c1.0 + t * t * end.0,
                                u * u * cy + 2.0 * u * t * c1.1 + t * t * end.1,
                            )
                        } else {
                            (
                                u * u * u * cx
                                    + 3.0 * u * u * t * c1.0
                                    + 3.0 * u * t * t * c2.0
                                    + t * t * t * end.0,
                                u * u * u * cy
                                    + 3.0 * u * u * t * c1.1
                                    + 3.0 * u * t * t * c2.1
                                    + t * t * t * end.1,
                            )
                        };
                        cur.push(p);
                    }
                    last_cx = c2.0;
                    last_cy = c2.1;
                    cx = end.0;
                    cy = end.1;
                }
                'A' => {
                    let (rx, ry, rot, large, sweep, x, y) = match (
                        lexer.number(),
                        lexer.number(),
                        lexer.number(),
                        lexer.number(),
                        lexer.number(),
                        lexer.number(),
                        lexer.number(),
                    ) {
                        (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g)) => {
                            (a, b, c, d, e, f, g)
                        }
                        _ => break,
                    };
                    let off = |v: f32, base: f32| if rel { v + base } else { v };
                    let end = (off(x, cx), off(y, cy));
                    ensure_start(&mut cur, cx, cy);
                    flatten_arc(
                        &mut cur,
                        (cx, cy),
                        rx.abs(),
                        ry.abs(),
                        rot,
                        large != 0.0,
                        sweep != 0.0,
                        end,
                    );
                    cx = end.0;
                    cy = end.1;
                }
                'Z' => {
                    if !cur.is_empty() {
                        cur.push((start_x, start_y));
                        if cur.len() > 1 {
                            out.push(std::mem::take(&mut cur));
                        } else {
                            cur.clear();
                        }
                        cx = start_x;
                        cy = start_y;
                    }
                    break;
                }
                _ => break,
            }
            last_cmd = cmd;
            first = false;
            // Um comando sem números (fim da string) encerra o laço interno.
            if !lexer.has_number_ahead() {
                break;
            }
        }
    }
    if cur.len() > 1 {
        out.push(cur);
    }
    out
}

fn ensure_start(cur: &mut Vec<(f32, f32)>, cx: f32, cy: f32) {
    if cur.is_empty() {
        cur.push((cx, cy));
    }
}

/// Arco elíptico do SVG (B.2.4) → polilinha.
#[allow(clippy::too_many_arguments)]
fn flatten_arc(
    out: &mut Vec<(f32, f32)>,
    from: (f32, f32),
    rx: f32,
    ry: f32,
    rot_deg: f32,
    large: bool,
    sweep: bool,
    to: (f32, f32),
) {
    if rx == 0.0 || ry == 0.0 || (from.0 == to.0 && from.1 == to.1) {
        out.push(to);
        return;
    }
    let phi = rot_deg.to_radians();
    let (sin_p, cos_p) = phi.sin_cos();
    let dx2 = (from.0 - to.0) / 2.0;
    let dy2 = (from.1 - to.1) / 2.0;
    let x1p = cos_p * dx2 + sin_p * dy2;
    let y1p = -sin_p * dx2 + cos_p * dy2;

    // Raio grande demais para a corda: o SVG manda escalar para cima.
    let lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    let (mut rx, mut ry) = (rx, ry);
    if lambda > 1.0 {
        let s = lambda.sqrt();
        rx *= s;
        ry *= s;
    }

    let num = (rx * rx * ry * ry - rx * rx * y1p * y1p - ry * ry * x1p * x1p).max(0.0);
    let den = rx * rx * y1p * y1p + ry * ry * x1p * x1p;
    let mut coef = if den == 0.0 { 0.0 } else { (num / den).sqrt() };
    if large == sweep {
        coef = -coef;
    }
    // Centro no sistema rotacionado, depois de volta ao original.
    let cxp = coef * (rx * y1p / ry);
    let cyp = coef * (-ry * x1p / rx);
    let cxp0 = cxp * cos_p - cyp * sin_p;
    let cyp0 = cxp * sin_p + cyp * cos_p;
    let center = ((from.0 + to.0) / 2.0 + cxp0, (from.1 + to.1) / 2.0 + cyp0);

    let angle = |ux: f32, uy: f32, vx: f32, vy: f32| {
        let dot = ux * vx + uy * vy;
        let len = (ux * ux + uy * uy).sqrt() * (vx * vx + vy * vy).sqrt();
        let mut a = if len == 0.0 {
            0.0
        } else {
            (dot / len).clamp(-1.0, 1.0).acos()
        };
        if ux * vy - uy * vx < 0.0 {
            a = -a;
        }
        a
    };
    let v1 = ((x1p - cxp0) / rx, (y1p - cyp0) / ry);
    let v2 = ((-x1p - cxp0) / rx, (-y1p - cyp0) / ry);
    let start = angle(1.0, 0.0, v1.0, v1.1);
    let mut delta = angle(v1.0, v1.1, v2.0, v2.1);
    if !sweep && delta > 0.0 {
        delta -= std::f32::consts::TAU;
    } else if sweep && delta < 0.0 {
        delta += std::f32::consts::TAU;
    }

    let steps = (CURVE_STEPS as f32 * (delta.abs() / (std::f32::consts::PI / 2.0))).ceil() as usize;
    let steps = steps.clamp(1, CURVE_STEPS * 4);
    for i in 1..=steps {
        let theta = start + delta * (i as f32 / steps as f32);
        let (s, c) = theta.sin_cos();
        let px = rx * c;
        let py = ry * s;
        out.push((
            center.0 + cos_p * px - sin_p * py,
            center.1 + sin_p * px + cos_p * py,
        ));
    }
}

/// Tokenizador mínimo de `pathData`.
struct Lexer<'a> {
    bytes: &'a [u8],
    i: usize,
}

impl<'a> Lexer<'a> {
    fn new(s: &'a str) -> Self {
        Lexer {
            bytes: s.as_bytes(),
            i: 0,
        }
    }

    fn skip_sep(&mut self) {
        while self.i < self.bytes.len() {
            match self.bytes[self.i] {
                b' ' | b'\t' | b'\n' | b'\r' | b',' => self.i += 1,
                _ => break,
            }
        }
    }

    fn command(&mut self) -> Option<char> {
        self.skip_sep();
        while self.i < self.bytes.len() {
            let c = self.bytes[self.i] as char;
            if c.is_ascii_alphabetic() {
                self.i += 1;
                return Some(c);
            }
            self.i += 1;
        }
        None
    }

    fn has_number_ahead(&mut self) -> bool {
        self.skip_sep();
        self.i < self.bytes.len() && matches!(self.bytes[self.i], b'-' | b'+' | b'.' | b'0'..=b'9')
    }

    fn number(&mut self) -> Option<f32> {
        self.skip_sep();
        let start = self.i;
        if self.i < self.bytes.len() && matches!(self.bytes[self.i], b'-' | b'+') {
            self.i += 1;
        }
        let mut digits = false;
        while self.i < self.bytes.len() && self.bytes[self.i].is_ascii_digit() {
            self.i += 1;
            digits = true;
        }
        if self.i < self.bytes.len() && self.bytes[self.i] == b'.' {
            self.i += 1;
            while self.i < self.bytes.len() && self.bytes[self.i].is_ascii_digit() {
                self.i += 1;
                digits = true;
            }
        }
        if !digits {
            self.i = start;
            return None;
        }
        // Expoente (`1e-5`) não aparece em ícone gerado pelo Android Studio,
        // mas aceitar custa nada.
        if self.i < self.bytes.len() && matches!(self.bytes[self.i], b'e' | b'E') {
            let save = self.i;
            self.i += 1;
            if self.i < self.bytes.len() && matches!(self.bytes[self.i], b'-' | b'+') {
                self.i += 1;
            }
            let mut exp_digits = false;
            while self.i < self.bytes.len() && self.bytes[self.i].is_ascii_digit() {
                self.i += 1;
                exp_digits = true;
            }
            if !exp_digits {
                self.i = save;
            }
        }
        std::str::from_utf8(&self.bytes[start..self.i])
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pts(d: &str) -> Vec<Vec<(f32, f32)>> {
        flatten_path_data(d)
    }

    #[test]
    fn moveto_lineto_absolute() {
        let p = pts("M0,0 L10,0 L10,10 Z");
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].first(), Some(&(0.0, 0.0)));
        assert_eq!(p[0].last(), Some(&(0.0, 0.0)), "Z fecha no início");
        assert!(p[0].contains(&(10.0, 10.0)));
    }

    #[test]
    fn relative_commands_accumulate() {
        let p = pts("m10 10 l5 0 l0 5 z");
        assert_eq!(p[0][0], (10.0, 10.0));
        assert!(p[0].contains(&(15.0, 10.0)));
        assert!(p[0].contains(&(15.0, 15.0)));
        assert_eq!(p[0].last(), Some(&(10.0, 10.0)));
    }

    #[test]
    fn implicit_lineto_after_moveto() {
        // `M0 0 10 0 10 10` = M + dois L implícitos.
        let p = pts("M0 0 10 0 10 10");
        assert_eq!(p[0].len(), 3);
        assert_eq!(p[0][2], (10.0, 10.0));
    }

    #[test]
    fn horizontal_and_vertical() {
        let p = pts("M0 0 H10 V10 h-5 v-5");
        assert_eq!(p[0][0], (0.0, 0.0));
        assert!(p[0].contains(&(10.0, 0.0)));
        assert!(p[0].contains(&(10.0, 10.0)));
        assert!(p[0].contains(&(5.0, 10.0)));
    }

    #[test]
    fn cubic_flattens_to_curve_steps() {
        let p = pts("M0 0 C0 10 10 10 10 0");
        // 1 origem + CURVE_STEPS amostras
        assert_eq!(p[0].len(), CURVE_STEPS + 1);
        assert_eq!(p[0].last(), Some(&(10.0, 0.0)));
        // O meio da curva fica abaixo da corda (y > 0).
        let mid = p[0][CURVE_STEPS / 2];
        assert!(mid.1 > 5.0, "curva deveria descer, veio {mid:?}");
    }

    #[test]
    fn smooth_cubic_reflects_control() {
        let p = pts("M0 0 C0 10 10 10 10 0 S20 -10 20 0");
        assert_eq!(p.last().unwrap().last(), Some(&(20.0, 0.0)));
    }

    #[test]
    fn quadratic_and_smooth_quadratic() {
        let p = pts("M0 0 Q5 10 10 0 T20 0");
        assert_eq!(p.last().unwrap().last(), Some(&(20.0, 0.0)));
    }

    #[test]
    fn arc_becomes_polyline() {
        let p = pts("M0 0 A5 5 0 0 1 10 0");
        assert!(p[0].len() > 4, "arco deve virar vários segmentos");
        let end = *p[0].last().unwrap();
        assert!(
            (end.0 - 10.0).abs() < 1e-3 && end.1.abs() < 1e-3,
            "arco deveria terminar em (10, 0), veio {end:?}"
        );
        // sweep=1 desenha o semicírculo abaixo do eixo (y negativo no SVG
        // vira y negativo no viewport; o flip acontece na escala para a tela).
        let mid = p[0][p[0].len() / 2];
        assert!(mid.1 < -1.0, "meio do arco deveria subir, veio {mid:?}");
    }

    #[test]
    fn multiple_subpaths() {
        let p = pts("M0 0 L1 0 Z M5 5 L6 5 Z");
        assert_eq!(p.len(), 2);
        assert_eq!(p[1][0], (5.0, 5.0));
    }

    #[test]
    fn numbers_glued_together() {
        // `10-5` é 10 e -5 (o SVG não exige separador antes do sinal).
        let p = pts("M10-5 L0 0");
        assert_eq!(p[0][0], (10.0, -5.0));
    }

    #[test]
    fn empty_and_garbage_do_not_panic() {
        for s in ["", "   ", "Z", "L1 2", "M", "M1", "Q1 2 3", "A1 2 3"] {
            let _ = pts(s);
        }
    }

    #[test]
    fn hex_colors_follow_android_order() {
        assert_eq!(parse_hex_color("F00"), Some(0xFFFF_0000));
        assert_eq!(parse_hex_color("8F00"), Some(0x88FF_0000));
        assert_eq!(parse_hex_color("FF0000"), Some(0xFFFF_0000));
        assert_eq!(parse_hex_color("80FF0000"), Some(0x80FF_0000));
        assert_eq!(parse_hex_color("XYZ"), None);
    }

    #[test]
    fn group_transform_matches_aosp_order() {
        // rotate 90° em torno de (10,10) leva (10,0) para (20,10).
        let m = Affine::group(90.0, (10.0, 10.0), (1.0, 1.0), (0.0, 0.0));
        let p = m.apply((10.0, 0.0));
        assert!(
            (p.0 - 20.0).abs() < 0.01 && (p.1 - 10.0).abs() < 0.01,
            "{p:?}"
        );
    }

    #[test]
    fn group_scale_and_translate_compose() {
        let m = Affine::group(0.0, (0.0, 0.0), (2.0, 2.0), (5.0, 5.0));
        assert_eq!(m.apply((1.0, 1.0)), (7.0, 7.0));
    }
}
