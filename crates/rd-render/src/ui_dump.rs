//! UI dump no formato uiautomator (M4 DoD: "get_ui_tree com ids certos").
//!
//! Fonte de verdade: `rd_vm::Engine::view_tree()` (árvore host estruturada).
//! O formato espelha o `uiautomator dump` do Android real: `<hierarchy>` com
//! `<node>` campo a campo (index/text/resource-id/class/package/bounds/…),
//! bounds "[x1,y1][x2,y2]" em px do viewport. Views GONE não aparecem
//! (comportamento do uiautomator real); INVISIBLE aparece com bounds (ocupa
//! espaço, não desenha — issue #46).

use rd_vm::framework::{UiNode, Vis};

/// Serializa a árvore no XML do uiautomator.
pub fn uiautomator_xml(root: &UiNode, package: &str) -> String {
    let mut out = String::from(
        "<?xml version='1.0' encoding='UTF-8' standalone='yes' ?>\n<hierarchy rotation=\"0\">\n",
    );
    let mut index = 0usize;
    emit_node(root, package, 1, &mut index, &mut out);
    out.push_str("</hierarchy>\n");
    out
}

fn emit_node(n: &UiNode, package: &str, depth: usize, index: &mut usize, out: &mut String) {
    if n.visibility == Vis::Gone {
        return; // uiautomator real: GONE não aparece no dump
    }
    let (x, y, w, h) = n.bounds;
    let idx = *index;
    *index += 1;
    let indent = "  ".repeat(depth);
    let res = n.resource_id.clone().unwrap_or_default();
    let clickable = if n.clickable { "true" } else { "false" };
    let enabled = if n.enabled { "true" } else { "false" };
    let empty = n.children.is_empty();
    out.push_str(&format!(
        "{indent}<node index=\"{idx}\" text=\"{}\" resource-id=\"{res}\" class=\"{}\" \
package=\"{package}\" content-desc=\"\" checkable=\"false\" checked=\"false\" \
clickable=\"{clickable}\" enabled=\"{enabled}\" focusable=\"{clickable}\" focused=\"false\" \
scrollable=\"false\" long-clickable=\"false\" password=\"false\" selected=\"false\" \
bounds=\"[{x},{y}][{},{}]\"{}>\n",
        xml_escape(&n.text),
        xml_escape(&n.class),
        x + w,
        y + h,
        if empty { "/" } else { "" }
    ));
    for c in &n.children {
        emit_node(c, package, depth + 1, index, out);
    }
    if !empty {
        out.push_str(&format!("{indent}</node>\n"));
    }
}

/// Escape mínimo de XML (uiautomator escapa os 5 reservados).
fn xml_escape(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\'' => o.push_str("&apos;"),
            _ => o.push(c),
        }
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(class: &str, rid: Option<&str>, text: &str, bounds: (i32, i32, i32, i32)) -> UiNode {
        UiNode {
            class: class.to_string(),
            resource_id: rid.map(str::to_string),
            text: text.to_string(),
            bounds,
            visibility: Vis::Visible,
            enabled: true,
            clickable: false,
            children: Vec::new(),
        }
    }

    #[test]
    fn dump_has_bounds_class_and_resource_id() {
        let mut root = node(
            "android.widget.LinearLayout",
            Some("com.test:id/root"),
            "",
            (0, 0, 720, 96),
        );
        root.children.push(node(
            "android.widget.TextView",
            Some("com.test:id/tv"),
            "Olá",
            (0, 0, 720, 48),
        ));
        let xml = uiautomator_xml(&root, "com.test");
        assert!(
            xml.contains("resource-id=\"com.test:id/tv\""),
            "ids certos: {xml}"
        );
        assert!(xml.contains("class=\"android.widget.TextView\""));
        assert!(xml.contains("bounds=\"[0,0][720,48]\""));
        assert!(xml.contains("text=\"Olá\""));
        assert!(xml.starts_with("<?xml"));
        assert!(xml.contains("<hierarchy rotation=\"0\">"));
    }

    #[test]
    fn gone_is_excluded_invisible_is_kept() {
        let mut root = node("android.widget.LinearLayout", None, "", (0, 0, 720, 144));
        let mut gone = node(
            "android.widget.TextView",
            Some("com.test:id/gone"),
            "",
            (0, 0, 0, 0),
        );
        gone.visibility = Vis::Gone;
        let mut inv = node(
            "android.widget.Button",
            Some("com.test:id/inv"),
            "",
            (0, 48, 720, 48),
        );
        inv.visibility = Vis::Invisible;
        root.children.push(gone);
        root.children.push(inv);
        let xml = uiautomator_xml(&root, "com.test");
        assert!(!xml.contains("com.test:id/gone"), "GONE não aparece: {xml}");
        assert!(
            xml.contains("com.test:id/inv"),
            "INVISIBLE aparece (ocupa espaço)"
        );
        assert!(xml.contains("bounds=\"[0,48][720,96]\""));
    }

    #[test]
    fn xml_escapes_reserved_chars() {
        let root = node(
            "android.widget.TextView",
            None,
            "a<b>&\"c\"",
            (0, 0, 10, 10),
        );
        let xml = uiautomator_xml(&root, "p");
        assert!(xml.contains("text=\"a&lt;b&gt;&amp;&quot;c&quot;"), "{xml}");
    }
}
