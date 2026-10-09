//! Dump dos atributos crus do <manifest> e <application> — ferramenta de debug
//! do parser AXML. Uso: cargo run -p rd-apk --example dump_manifest_attrs -- <apk>

use rd_apk::Apk;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: dump_manifest_attrs <apk>");
    let apk = Apk::open(std::path::Path::new(&path)).expect("apk");
    let doc = apk.manifest_raw().expect("axml");
    println!("namespaces: {:?}", doc.namespaces);
    println!("\n<manifest> attrs ({}):", doc.root.attrs.len());
    for a in &doc.root.attrs {
        println!(
            "  ns={:?} name={} raw={:?} value={:?} resid={:?}",
            a.ns.as_deref().map(|s| &s[..40.min(s.len())]),
            a.name,
            a.raw,
            a.value,
            a.res_id
        );
    }
    if let Some(app) = doc.root.children_named("application").next() {
        println!("\n<application> attrs ({}):", app.attrs.len());
        for a in app.attrs.iter().take(14) {
            println!(
                "  ns={:?} name={} raw={:?} value={:?} resid={:?}",
                a.ns.as_deref().map(|s| &s[..40.min(s.len())]),
                a.name,
                a.raw,
                a.value,
                a.res_id
            );
        }
    };
}
