#![no_main]
//! Fuzz: caminho completo — ZIP + manifest + ARSC do APK inteiro em memória.
use libfuzzer_sys::fuzz_target;
use rd_apk::Apk;

fuzz_target!(|data: &[u8]| {
    let _ = Apk::from_bytes(data.to_vec());
});
