//! Dump estrutural de um .dex — debug do parser rd-dex (M1).
//! Uso: cargo run -p rd-dex --example dump_dex -- <arquivo.dex> [N classes]

use rd_dex::Dex;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("uso: dump_dex <arquivo.dex> [N classes]");
    let n: usize = std::env::args()
        .nth(2)
        .and_then(|a| a.parse().ok())
        .unwrap_or(5);

    let dex = Dex::parse(std::fs::read(&path).expect("ler arquivo")).expect("parse dex");

    println!(
        "arquivo: {path} — {} bytes (0x{:x})",
        dex.data.len(),
        dex.data.len()
    );
    println!(
        "header: magic={:?} versão={} checksum=0x{:08x} (adler32 ok: {}) file_size={} header_size={}",
        String::from_utf8_lossy(&dex.header.magic[..4]),
        dex.header.version_str(),
        dex.header.checksum,
        dex.header.checksum_ok,
        dex.header.file_size,
        dex.header.header_size
    );

    // map list = cobertura de seções do arquivo
    let mut valid_offsets = 0usize;
    for item in &dex.map_list {
        if (item.offset as usize) < dex.data.len() {
            valid_offsets += 1;
        }
        println!(
            "  map {:<28} size={:<6} offset=0x{:x}",
            item.map_type.name(),
            item.size,
            item.offset
        );
    }
    println!(
        "map coverage: {}/{} itens com offset válido (< file_size)",
        valid_offsets,
        dex.map_list.len()
    );

    println!(
        "seções: strings={} types={} protos={} fields={} methods={} classes={} \
         call_sites={} method_handles={}",
        dex.strings.len(),
        dex.type_ids.len(),
        dex.proto_ids.len(),
        dex.field_ids.len(),
        dex.method_ids.len(),
        dex.class_defs.len(),
        dex.call_site_offsets.len(),
        dex.method_handles.len()
    );

    // contagens derivadas dos class_data/code items
    let mut methods_with_code = 0usize;
    let mut units = 0usize;
    let mut tries = 0usize;
    for def in &dex.class_defs {
        if let Ok(Some(cd)) = dex.class_data(def) {
            for m in cd.direct_methods.iter().chain(&cd.virtual_methods) {
                if let Ok(Some(ci)) = dex.code(m.code_off) {
                    methods_with_code += 1;
                    units += ci.insns.len();
                    tries += ci.tries.len();
                }
            }
        }
    }
    println!("código: {methods_with_code} métodos com code_item ({units} units, {tries} tries)");

    println!("\nprimeiras {n} classes:");
    for def in dex.class_defs.iter().take(n) {
        println!(
            "  #{} {} flags=0x{:x} super={}",
            def.index,
            dex.type_str(def.class_idx),
            def.access_flags,
            dex.type_str(def.superclass_idx)
        );
        if let Ok(Some(cd)) = dex.class_data(def) {
            println!(
                "     static_fields={} instance_fields={} direct={} virtual={}",
                cd.static_fields.len(),
                cd.instance_fields.len(),
                cd.direct_methods.len(),
                cd.virtual_methods.len()
            );
        }
        let sv = dex.static_values(def).unwrap_or_default();
        if !sv.is_empty() {
            println!("     static_values: {} valores", sv.len());
        }
    }
}
