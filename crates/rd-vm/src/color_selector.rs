//! Seletor de cor XML (`res/color/*.xml`) para `Resources.getColorStateList`.
//!
//! Forma modelada: `<selector>` com `<item android:color="…">` (literal `#…`
//! ou referência `@color/…`) e, opcionalmente, `android:state_*` com
//! `true`/`false`. Os IDs dos atributos `state_*` vêm do resource map do próprio
//! APK (`XmlAttribute::res_id`), e não de uma tabela fixa no código.
//!
//! Fora da forma (ex.: `<selector>` aninhado, `android:alpha`, atributo de
//! estado sem `res_id`): devolve `None` e o chamador falha visivelmente.

use rd_apk::axml::{self, AttrValue, XmlElement};

use crate::vector_drawable::{color_attr, RawColor};

/// Um item do seletor. Estados seguem o formato de `ColorStateList`
/// (`state_set_matches`): `+id` para `state_* = true`, `-id` para `false`.
pub(crate) struct Selector {
    /// Lista de estados por item, no formato `ColorStateList`.
    pub(crate) states: Vec<Vec<i32>>,
    /// Cor por item. Referências `@color/…` ficam sem resolver aqui; ver
    /// [`parse_selector`].
    pub(crate) colors: Vec<i32>,
}

/// Lê um seletor. `None` quando a forma não é a modelada.
///
/// Cores em referência (`@color/…`) não são resolvidas aqui e o seletor
/// inteiro devolve `None`: o ARSC pertence ao chamador, e trocar a referência
/// por outra cor seria errado em silêncio.
pub(crate) fn parse_selector(bytes: &[u8]) -> Option<Selector> {
    let doc = axml::parse(bytes).ok()?;
    let root = &doc.root;
    if root.name != "selector" {
        return None;
    }
    let mut states = Vec::new();
    let mut colors = Vec::new();
    for item in root.children.iter().filter(|c| c.name == "item") {
        let (state, color) = parse_item(item)?;
        states.push(state);
        colors.push(color);
    }
    if colors.is_empty() {
        return None;
    }
    Some(Selector { states, colors })
}

fn parse_item(item: &XmlElement) -> Option<(Vec<i32>, i32)> {
    let color = match color_attr(item, "color")? {
        RawColor::Literal(c) => c as i32,
        RawColor::Ref(_) => return None,
    };
    let mut state = Vec::new();
    for a in item.attrs.iter().filter(|a| a.is_android()) {
        let Some(_) = a.name.strip_prefix("state_") else {
            continue;
        };
        let id = a.res_id?;
        let on = match a.value {
            AttrValue::Bool(b) => b,
            _ => return None,
        };
        let id = i32::try_from(id).ok()?;
        state.push(if on { id } else { -id });
    }
    Some((state, color))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_root_is_not_modelled() {
        assert!(parse_selector(b"not axml").is_none());
    }
}
