//! Diagnóstico manual (não é gate): dump de entradas ARSC por res_id.
//! Uso: RD_DUMP_APK=golden-apks/org.fossify.contacts.apk RD_DUMP_IDS=0x7f0401c6,0x7f030183 cargo test -p rd-apk --test dump_res -- --nocapture

use rd_apk::Apk;
use std::path::Path;

#[test]
fn dump_resource_entries() {
    let Ok(apk_path) = std::env::var("RD_DUMP_APK") else {
        eprintln!("RD_DUMP_APK não definido — skip");
        return;
    };
    let Ok(ids_raw) = std::env::var("RD_DUMP_IDS") else {
        eprintln!("RD_DUMP_IDS não definido — skip");
        return;
    };
    let apk = Apk::open(Path::new(&apk_path)).expect("apk open");
    let Some(arsc) = apk.arsc.as_ref() else {
        eprintln!("APK sem resources.arsc");
        return;
    };
    for id_raw in ids_raw.split(',') {
        let id = u32::from_str_radix(id_raw.trim().trim_start_matches("0x"), 16).unwrap();
        println!("=== 0x{id:08x} ===");
        for pkg in &arsc.packages {
            if pkg.id != ((id >> 24) & 0xFF) as u8 {
                continue;
            }
            println!("  package {} (id 0x{:02x})", pkg.name, pkg.id);
            println!("  resolve_name: {:?}", pkg.resolve_name(id));
            match pkg.entries_for(id) {
                None => println!("  entries_for: None (id fora do índice pairs)"),
                Some(entries) => {
                    println!("  entries_for: {} entrada(s)", entries.len());
                    for (i, e) in entries.iter().enumerate() {
                        println!(
                            "    [{i}] config={:?} data_type=0x{:02x} data=0x{:08x} str={:?} complex={} parent={:?} key_idx={} bag_len={:?}",
                            e.config,
                            e.data_type,
                            e.data,
                            e.string,
                            e.complex,
                            e.parent,
                            e.key_idx,
                            e.bag.as_ref().map(|b| b.len())
                        );
                    }
                }
            }
            println!("  resolve_value: {:?}", arsc.resolve_value(id));
        }
    }
}
