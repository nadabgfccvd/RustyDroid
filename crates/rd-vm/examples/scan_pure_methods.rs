//! Scan de métodos static puros (primitivos, código curto, sem try) em APK real —
//! candidatos à validação "métodos puros de APK real" do M2.

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: scan_pure_methods <apk>");
    let path = std::path::Path::new(&path);
    let apk = rd_apk::Apk::open(path).expect("apk");
    let mut count = 0;
    for name in &apk.dex_files {
        let entry = apk.zip.find(name).expect("dex");
        let bytes = apk.zip.read_entry(entry).expect("bytes");
        let dex = rd_dex::Dex::parse(bytes).expect("dex");
        for def in &dex.class_defs {
            let class_desc = dex.type_str(def.class_idx).to_string();
            if class_desc.starts_with("Lkotlin") || class_desc.contains("$$") {
                continue;
            }
            let Ok(Some(cd)) = dex.class_data(def) else {
                continue;
            };
            for m in cd.direct_methods.iter() {
                let Ok(mid) = dex.method(m.method_idx) else {
                    continue;
                };
                let name = dex.string(mid.name_idx);
                let Ok(proto) = dex.proto(mid.proto_idx) else {
                    continue;
                };
                let ret = dex.type_str(proto.return_type_idx);
                if ret != "I" && ret != "J" && ret != "F" && ret != "D" && ret != "Z" {
                    continue;
                }
                let Ok(params) = dex.proto_params(mid.proto_idx) else {
                    continue;
                };
                if params.len() > 3 {
                    continue;
                }
                if params.iter().any(|p| {
                    let d = dex.type_str(*p);
                    d.starts_with('L') || d.starts_with('[')
                }) {
                    continue;
                }
                let Ok(Some(code)) = dex.code(m.code_off) else {
                    continue;
                };
                if code.tries_size > 0 || code.insns.len() > 80 || code.insns.len() < 4 {
                    continue;
                }
                // só opcodes núcleo M2 (sem invoke, sem const-string, sem objetos)
                let ok = code.instructions.iter().all(|(_, i)| {
                    let op = i.opcode;
                    matches!(op,
                        0x00..=0x14 | 0x16..=0x18 | 0x21 | 0x25..=0x31
                        | 0x32..=0x3D | 0x44..=0x51 | 0x7B..=0xE2)
                });
                if !ok {
                    continue;
                }
                let sig_params: String = params.iter().map(|p| dex.type_str(*p)).collect();
                println!(
                    "{class_desc} {name} ({sig_params}){ret} units={}",
                    code.insns.len()
                );
                count += 1;
                if count > 25 {
                    return;
                }
            }
        }
    }
}
