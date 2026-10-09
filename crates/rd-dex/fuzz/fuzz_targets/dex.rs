#![no_main]
//! Fuzz: parser DEX completo (header, map, strings, class_data, code, debug,
//! annotations) + render das primeiras classes — nunca entra em pânico.
use libfuzzer_sys::fuzz_target;
use rd_dex::Dex;

fuzz_target!(|data: &[u8]| {
    if let Ok(dex) = Dex::parse(data.to_vec()) {
        for def in dex.class_defs.iter().take(8) {
            let _ = rd_dex::disasm::render_class(&dex, def.index);
        }
    }
});
