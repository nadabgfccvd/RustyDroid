#![no_main]
//! Fuzz: parser ZIP (EOCD/CD/local headers/DEFLATE) nunca entra em pânico.
use libfuzzer_sys::fuzz_target;
use rd_apk::zip::Zip;

fuzz_target!(|data: &[u8]| {
    let _ = Zip::parse(data.to_vec());
});
