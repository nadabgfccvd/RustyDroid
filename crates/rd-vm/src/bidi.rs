//! Reordenação visual de texto (UAX #9) para o desenho da VM.
//!
//! O `rd-render` (#539) já shapeava o texto do *snapshot* por segmentos
//! visuais; este módulo traz a mesma reordenação para o caminho que os APKs
//! reais usam (`Canvas.drawText`/`drawTextRun` sobre o atlas de glifos), onde
//! não há shaper: cada glifo é um bitmap avulso, então um run RTL é desenhado
//! com os caracteres em ordem invertida e os pares direcionais espelhados
//! (regra L4). Sem espelhamento contextual de formas (árabe) — desvio
//! documentado no cabeçalho de `graphics.rs`.

use std::ops::Range;
use unicode_bidi::{BidiInfo, Level};

/// Direção de base do parágrafo desenhado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextDir {
    /// Primeiro caractere forte decide (padrão do `TextView`).
    Auto,
    /// `drawTextRun(..., isRtl=false, ...)`.
    Ltr,
    /// `drawTextRun(..., isRtl=true, ...)`.
    Rtl,
}

/// Runs em ordem visual (esquerda → direita) com a flag RTL por run.
pub(crate) fn visual_runs(s: &str, dir: TextDir) -> Vec<(Range<usize>, bool)> {
    match dir {
        TextDir::Auto => runs_of(&BidiInfo::new(s, None), 0),
        TextDir::Ltr => forced_runs(s, '\u{200E}', Level::ltr()), // LRM
        TextDir::Rtl => forced_runs(s, '\u{200F}', Level::rtl()), // RLM
    }
}

/// Extrai os runs visuais de cada parágrafo, deslocando os intervalos em
/// `shift` bytes (a marca LRM/RLM do prefixo, quando há) e descartando o que
/// sobrar vazio — a marca nunca é desenhada.
fn runs_of(info: &BidiInfo, shift: usize) -> Vec<(Range<usize>, bool)> {
    let mut out = Vec::new();
    for para in &info.paragraphs {
        let (levels, runs) = info.visual_runs(para, para.range.clone());
        for run in runs {
            let start = run.start.saturating_sub(shift);
            let end = run.end.saturating_sub(shift).max(start);
            if end > start {
                let rtl = levels.get(run.start).is_some_and(|l| l.is_rtl());
                out.push((start..end, rtl));
            }
        }
    }
    out
}

/// O `unicode-bidi` só usa o nível default quando o parágrafo não tem
/// caractere forte; para forçar a base de verdade — o `isRtl` do `drawTextRun`
/// é a decisão do layout do Android, não um palpite — prefixamos LRM/RLM: o
/// primeiro forte define a base (P2/P3) e a marca some dos intervalos de
/// saída. O nível default cobre os parágrafos seguintes (após `\n`) sem forte.
fn forced_runs(s: &str, mark: char, default: Level) -> Vec<(Range<usize>, bool)> {
    let mut prefixed = String::with_capacity(s.len() + mark.len_utf8());
    prefixed.push(mark);
    prefixed.push_str(s);
    runs_of(&BidiInfo::new(&prefixed, Some(default)), mark.len_utf8())
}

/// Pares espelháveis que o atlas cobre (UAX #9 L4); fora da tabela o glifo é
/// desenhado como está.
pub(crate) fn mirrored(c: char) -> char {
    match c {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn txt(s: &str, runs: &[(Range<usize>, bool)]) -> Vec<(String, bool)> {
        runs.iter()
            .map(|(r, rtl)| (s[r.clone()].to_string(), *rtl))
            .collect()
    }

    #[test]
    fn texto_ltr_puro_e_um_unico_run_ltr() {
        assert_eq!(
            txt("hello", &visual_runs("hello", TextDir::Auto)),
            vec![("hello".to_string(), false)]
        );
    }

    #[test]
    fn hebraico_puro_e_um_unico_run_rtl() {
        assert_eq!(
            txt("שלום", &visual_runs("שלום", TextDir::Auto)),
            vec![("שלום".to_string(), true)]
        );
    }

    #[test]
    fn base_ltr_deixa_latino_na_esquerda() {
        // "abc שלום": base LTR (primeiro forte é 'a') → hebraico à direita.
        assert_eq!(
            txt("abc שלום", &visual_runs("abc שלום", TextDir::Auto)),
            vec![("abc ".to_string(), false), ("שלום".to_string(), true)]
        );
    }

    #[test]
    fn base_rtl_deixa_latino_na_esquerda() {
        // "שלום abc": base RTL → o run latino continua à esquerda, com o
        // espaço neutro pertencendo ao run hebraico (nível da base).
        assert_eq!(
            txt("שלום abc", &visual_runs("שלום abc", TextDir::Auto)),
            vec![("abc".to_string(), false), ("שלום ".to_string(), true)]
        );
    }

    #[test]
    fn forca_rtl_move_neutro_de_sufixo_para_a_esquerda() {
        // "abc!" automático = um run LTR; forçado RTL = "!" vira run RTL na
        // esquerda (o começo lógico de um parágrafo RTL fica à direita).
        assert_eq!(
            txt("abc!", &visual_runs("abc!", TextDir::Auto)),
            vec![("abc!".to_string(), false)]
        );
        assert_eq!(
            txt("abc!", &visual_runs("abc!", TextDir::Rtl)),
            vec![("!".to_string(), true), ("abc".to_string(), false)]
        );
    }

    #[test]
    fn forca_ltr_inverte_a_ordem_dos_runs() {
        // Automático (base RTL): "abc" à esquerda; forçado LTR: hebraico à
        // esquerda e o espaço acompanha o run latino (nível da base).
        assert_eq!(
            txt("שלום abc", &visual_runs("שלום abc", TextDir::Auto)),
            vec![("abc".to_string(), false), ("שלום ".to_string(), true)]
        );
        assert_eq!(
            txt("שלום abc", &visual_runs("שלום abc", TextDir::Ltr)),
            vec![("שלום".to_string(), true), (" abc".to_string(), false)]
        );
    }

    #[test]
    fn parenteses_trocam_de_lado_em_paragrafo_rtl() {
        // "א(X)" base RTL: visual = ")", "X", "א(" — o span inteiro de nível
        // ≥1 é revertido (L2), o ")" lógico vai para a extremidade esquerda e
        // o rasterizador espelha os pares (regra L4).
        assert_eq!(
            txt("א(X)", &visual_runs("א(X)", TextDir::Auto)),
            vec![
                (")".to_string(), true),
                ("X".to_string(), false),
                ("א(".to_string(), true)
            ]
        );
    }

    #[test]
    fn vazio_nao_tem_runs() {
        assert!(visual_runs("", TextDir::Auto).is_empty());
        assert!(visual_runs("", TextDir::Rtl).is_empty());
    }

    #[test]
    fn pares_direcionais_sao_espelhados() {
        assert_eq!(mirrored('('), ')');
        assert_eq!(mirrored(')'), '(');
        assert_eq!(mirrored('['), ']');
        assert_eq!(mirrored('»'), '«');
        // Fora da tabela o glifo é desenhado como está.
        assert_eq!(mirrored('a'), 'a');
        assert_eq!(mirrored('א'), 'א');
    }
}
