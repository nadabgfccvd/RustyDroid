#![no_main]
//! Fuzz: parser AXML (pools UTF-8/16, resource map, typed values) nunca entra em pânico.
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = rd_apk::axml::parse(data);
});
