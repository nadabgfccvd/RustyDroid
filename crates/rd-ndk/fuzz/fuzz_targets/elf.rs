#![no_main]
//! Fuzz: parser ELF do M9 (`rd_ndk::elf::Elf::parse`): header, program
//! headers, PT_DYNAMIC, símbolos, relocações e DT_NEEDED. Com entrada
//! adversarial ele devolve erro tipado (`INVALID_ELF`) e nunca entra em
//! pânico nem estoura o orçamento de memória (RustyDroid.md §3.1).
use libfuzzer_sys::fuzz_target;
use rd_ndk::elf::Elf;

fuzz_target!(|data: &[u8]| {
    if let Ok(elf) = Elf::parse(data) {
        // exercita os caminhos de consulta sobre a imagem já validada
        let _ = elf.abi.android_name();
        let _ = elf.abi.pointer_size();
        if let Some(first) = elf.symbols.first() {
            let _ = elf.symbol(&first.name);
        }
        let _ = elf.symbol("JNI_OnLoad");
        let _ = rd_ndk::elf::relative_kind(elf.abi);
    }
});
