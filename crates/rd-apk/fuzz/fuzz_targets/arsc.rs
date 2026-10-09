#![no_main]
//! Fuzz: parser ARSC (ResTable, configs, sparse/offset16, referências) nunca entra em pânico.
use libfuzzer_sys::fuzz_target;
use rd_apk::arsc::Arsc;

fuzz_target!(|data: &[u8]| {
    let _ = Arsc::parse(data);
});
