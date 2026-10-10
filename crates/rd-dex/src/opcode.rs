//! Tabela completa de opcodes Dalvik 0x00–0xFF + formatos de instrução.
//!
//! Formatos (nomenclatura oficial do dex-format): 10x 12x 11n 11x 10t 20t 22x
//! 21t 21s 21h 21c 23x 22b 22t 22s 22c 30t 31i 31t 31c 32x 35c 3rc 45cc 4rcc 51l.
//! Opcodes "unused" (0x3e–43, 0x73, 0x79–7a, 0xe3–f9) ficam na tabela como 10x —
//! o disassembler os renderiza como `nop // unused 0xXX`. Atenção: 0xd8–0xe2 são
//! os 11 `*int/lit8` VÁLIDOS (add…ushr); o intervalo vazio real é 0xe3–0xf9
//! (0xfa–0xff são invoke-polymorphic/custom e const-method-*).
//!
//! Pseudoinstruções (payloads) usam o slot do opcode 0x00 com identificador na
//! palavra alta: 0x0100 packed-switch-payload, 0x0200 sparse-switch-payload,
//! 0x0300 fill-array-data-payload.

/// Formato de instrução (espaçamento/interpretação dos code units).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    F10x,
    F12x,
    F11n,
    F11x,
    F10t,
    F20t,
    F22x,
    F21t,
    F21s,
    F21h,
    F21c,
    F23x,
    F22b,
    F22t,
    F22s,
    F22c,
    F30t,
    F31i,
    F31t,
    F31c,
    F32x,
    F35c,
    F3rc,
    F45cc,
    F4rcc,
    F51l,
}

impl Format {
    /// Tamanho em code units (16 bits).
    pub fn size(self) -> u16 {
        match self {
            Format::F10x | Format::F12x | Format::F11n | Format::F11x | Format::F10t => 1,
            Format::F20t
            | Format::F22x
            | Format::F21t
            | Format::F21s
            | Format::F21h
            | Format::F21c
            | Format::F23x
            | Format::F22b
            | Format::F22t
            | Format::F22s
            | Format::F22c
            | Format::F32x => 2,
            Format::F30t
            | Format::F31i
            | Format::F31t
            | Format::F31c
            | Format::F35c
            | Format::F3rc => 3,
            // 45cc/4rcc = 4 units (primeiro dígito do nome = tamanho):
            // AG|op, BBBB (method), FEDC (regs), HHHH (proto)
            Format::F45cc | Format::F4rcc => 4,
            Format::F51l => 5,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Format::F10x => "10x",
            Format::F12x => "12x",
            Format::F11n => "11n",
            Format::F11x => "11x",
            Format::F10t => "10t",
            Format::F20t => "20t",
            Format::F22x => "22x",
            Format::F21t => "21t",
            Format::F21s => "21s",
            Format::F21h => "21h",
            Format::F21c => "21c",
            Format::F23x => "23x",
            Format::F22b => "22b",
            Format::F22t => "22t",
            Format::F22s => "22s",
            Format::F22c => "22c",
            Format::F30t => "30t",
            Format::F31i => "31i",
            Format::F31t => "31t",
            Format::F31c => "31c",
            Format::F32x => "32x",
            Format::F35c => "35c",
            Format::F3rc => "3rc",
            Format::F45cc => "45cc",
            Format::F4rcc => "4rcc",
            Format::F51l => "51l",
        }
    }
}

/// Payloads (pseudoinstruções alinhadas a code unit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadKind {
    PackedSwitch,
    SparseSwitch,
    ArrayData,
}

impl PayloadKind {
    pub fn from_unit(unit: u16) -> Option<PayloadKind> {
        match unit {
            0x0100 => Some(PayloadKind::PackedSwitch),
            0x0200 => Some(PayloadKind::SparseSwitch),
            0x0300 => Some(PayloadKind::ArrayData),
            _ => None,
        }
    }
}

/// Entrada da tabela de opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpcodeInfo {
    pub value: u8,
    pub name: &'static str,
    pub format: Format,
}

impl OpcodeInfo {
    /// Opcodes reservados/unused — renderizados como `nop // unused 0xXX`.
    pub fn is_unused(&self) -> bool {
        self.name.starts_with("unused")
    }
}

/// Tabela estática completa (256 entradas — teste garante).
pub const OPCODES: [OpcodeInfo; 256] = build_table();

const fn op(value: u8, name: &'static str, format: Format) -> OpcodeInfo {
    OpcodeInfo {
        value,
        name,
        format,
    }
}

const fn unused(value: u8) -> OpcodeInfo {
    OpcodeInfo {
        value,
        name: "unused",
        format: Format::F10x,
    }
}

// geração const da tabela — ordem é o valor do opcode
const fn build_table() -> [OpcodeInfo; 256] {
    let mut t = [op(0, "nop", Format::F10x); 256];
    macro_rules! set {
        ($t:expr, $idx:expr, $name:expr, $fmt:expr) => {
            $t[$idx as usize] = op($idx, $name, $fmt);
        };
    }
    set!(t, 0x00, "nop", Format::F10x);
    set!(t, 0x01, "move", Format::F12x);
    set!(t, 0x02, "move/from16", Format::F22x);
    set!(t, 0x03, "move/16", Format::F32x);
    set!(t, 0x04, "move-wide", Format::F12x);
    set!(t, 0x05, "move-wide/from16", Format::F22x);
    set!(t, 0x06, "move-wide/16", Format::F32x);
    set!(t, 0x07, "move-object", Format::F12x);
    set!(t, 0x08, "move-object/from16", Format::F22x);
    set!(t, 0x09, "move-object/16", Format::F32x);
    set!(t, 0x0a, "move-result", Format::F11x);
    set!(t, 0x0b, "move-result-wide", Format::F11x);
    set!(t, 0x0c, "move-result-object", Format::F11x);
    set!(t, 0x0d, "move-exception", Format::F11x);
    set!(t, 0x0e, "return-void", Format::F10x);
    set!(t, 0x0f, "return", Format::F11x);
    set!(t, 0x10, "return-wide", Format::F11x);
    set!(t, 0x11, "return-object", Format::F11x);
    set!(t, 0x12, "const/4", Format::F11n);
    set!(t, 0x13, "const/16", Format::F21s);
    set!(t, 0x14, "const", Format::F31i);
    set!(t, 0x15, "const/high16", Format::F21h);
    set!(t, 0x16, "const-wide/16", Format::F21s);
    set!(t, 0x17, "const-wide/32", Format::F31i);
    set!(t, 0x18, "const-wide", Format::F51l);
    set!(t, 0x19, "const-wide/high16", Format::F21h);
    set!(t, 0x1a, "const-string", Format::F21c);
    set!(t, 0x1b, "const-string/jumbo", Format::F31c);
    set!(t, 0x1c, "const-class", Format::F21c);
    set!(t, 0x1d, "monitor-enter", Format::F11x);
    set!(t, 0x1e, "monitor-exit", Format::F11x);
    set!(t, 0x1f, "check-cast", Format::F21c);
    set!(t, 0x20, "instance-of", Format::F22c);
    set!(t, 0x21, "array-length", Format::F12x);
    set!(t, 0x22, "new-instance", Format::F21c);
    set!(t, 0x23, "new-array", Format::F22c);
    set!(t, 0x24, "filled-new-array", Format::F35c);
    set!(t, 0x25, "filled-new-array/range", Format::F3rc);
    set!(t, 0x26, "fill-array-data", Format::F31t);
    set!(t, 0x27, "throw", Format::F11x);
    set!(t, 0x28, "goto", Format::F10t);
    set!(t, 0x29, "goto/16", Format::F20t);
    set!(t, 0x2a, "goto/32", Format::F30t);
    set!(t, 0x2b, "packed-switch", Format::F31t);
    set!(t, 0x2c, "sparse-switch", Format::F31t);
    set!(t, 0x2d, "cmpl-float", Format::F23x);
    set!(t, 0x2e, "cmpg-float", Format::F23x);
    set!(t, 0x2f, "cmpl-double", Format::F23x);
    set!(t, 0x30, "cmpg-double", Format::F23x);
    set!(t, 0x31, "cmp-long", Format::F23x);
    set!(t, 0x32, "if-eq", Format::F22t);
    set!(t, 0x33, "if-ne", Format::F22t);
    set!(t, 0x34, "if-lt", Format::F22t);
    set!(t, 0x35, "if-ge", Format::F22t);
    set!(t, 0x36, "if-gt", Format::F22t);
    set!(t, 0x37, "if-le", Format::F22t);
    set!(t, 0x38, "if-eqz", Format::F21t);
    set!(t, 0x39, "if-nez", Format::F21t);
    set!(t, 0x3a, "if-ltz", Format::F21t);
    set!(t, 0x3b, "if-gez", Format::F21t);
    set!(t, 0x3c, "if-gtz", Format::F21t);
    set!(t, 0x3d, "if-lez", Format::F21t);
    // 0x3e..0x43 unused (já cobertos pelo array inicial)
    set!(t, 0x44, "aget", Format::F23x);
    set!(t, 0x45, "aget-wide", Format::F23x);
    set!(t, 0x46, "aget-object", Format::F23x);
    set!(t, 0x47, "aget-boolean", Format::F23x);
    set!(t, 0x48, "aget-byte", Format::F23x);
    set!(t, 0x49, "aget-char", Format::F23x);
    set!(t, 0x4a, "aget-short", Format::F23x);
    set!(t, 0x4b, "aput", Format::F23x);
    set!(t, 0x4c, "aput-wide", Format::F23x);
    set!(t, 0x4d, "aput-object", Format::F23x);
    set!(t, 0x4e, "aput-boolean", Format::F23x);
    set!(t, 0x4f, "aput-byte", Format::F23x);
    set!(t, 0x50, "aput-char", Format::F23x);
    set!(t, 0x51, "aput-short", Format::F23x);
    set!(t, 0x52, "iget", Format::F22c);
    set!(t, 0x53, "iget-wide", Format::F22c);
    set!(t, 0x54, "iget-object", Format::F22c);
    set!(t, 0x55, "iget-boolean", Format::F22c);
    set!(t, 0x56, "iget-byte", Format::F22c);
    set!(t, 0x57, "iget-char", Format::F22c);
    set!(t, 0x58, "iget-short", Format::F22c);
    set!(t, 0x59, "iput", Format::F22c);
    set!(t, 0x5a, "iput-wide", Format::F22c);
    set!(t, 0x5b, "iput-object", Format::F22c);
    set!(t, 0x5c, "iput-boolean", Format::F22c);
    set!(t, 0x5d, "iput-byte", Format::F22c);
    set!(t, 0x5e, "iput-char", Format::F22c);
    set!(t, 0x5f, "iput-short", Format::F22c);
    set!(t, 0x60, "sget", Format::F21c);
    set!(t, 0x61, "sget-wide", Format::F21c);
    set!(t, 0x62, "sget-object", Format::F21c);
    set!(t, 0x63, "sget-boolean", Format::F21c);
    set!(t, 0x64, "sget-byte", Format::F21c);
    set!(t, 0x65, "sget-char", Format::F21c);
    set!(t, 0x66, "sget-short", Format::F21c);
    set!(t, 0x67, "sput", Format::F21c);
    set!(t, 0x68, "sput-wide", Format::F21c);
    set!(t, 0x69, "sput-object", Format::F21c);
    set!(t, 0x6a, "sput-boolean", Format::F21c);
    set!(t, 0x6b, "sput-byte", Format::F21c);
    set!(t, 0x6c, "sput-char", Format::F21c);
    set!(t, 0x6d, "sput-short", Format::F21c);
    set!(t, 0x6e, "invoke-virtual", Format::F35c);
    set!(t, 0x6f, "invoke-super", Format::F35c);
    set!(t, 0x70, "invoke-direct", Format::F35c);
    set!(t, 0x71, "invoke-static", Format::F35c);
    set!(t, 0x72, "invoke-interface", Format::F35c);
    // 0x73 unused
    set!(t, 0x74, "invoke-virtual/range", Format::F3rc);
    set!(t, 0x75, "invoke-super/range", Format::F3rc);
    set!(t, 0x76, "invoke-direct/range", Format::F3rc);
    set!(t, 0x77, "invoke-static/range", Format::F3rc);
    set!(t, 0x78, "invoke-interface/range", Format::F3rc);
    // 0x79, 0x7a unused
    set!(t, 0x7b, "neg-int", Format::F12x);
    set!(t, 0x7c, "not-int", Format::F12x);
    set!(t, 0x7d, "neg-long", Format::F12x);
    set!(t, 0x7e, "not-long", Format::F12x);
    set!(t, 0x7f, "neg-float", Format::F12x);
    set!(t, 0x80, "neg-double", Format::F12x);
    set!(t, 0x81, "int-to-long", Format::F12x);
    set!(t, 0x82, "int-to-float", Format::F12x);
    set!(t, 0x83, "int-to-double", Format::F12x);
    set!(t, 0x84, "long-to-int", Format::F12x);
    set!(t, 0x85, "long-to-float", Format::F12x);
    set!(t, 0x86, "long-to-double", Format::F12x);
    set!(t, 0x87, "float-to-int", Format::F12x);
    set!(t, 0x88, "float-to-long", Format::F12x);
    set!(t, 0x89, "float-to-double", Format::F12x);
    set!(t, 0x8a, "double-to-int", Format::F12x);
    set!(t, 0x8b, "double-to-long", Format::F12x);
    set!(t, 0x8c, "double-to-float", Format::F12x);
    set!(t, 0x8d, "int-to-byte", Format::F12x);
    set!(t, 0x8e, "int-to-char", Format::F12x);
    set!(t, 0x8f, "int-to-short", Format::F12x);
    set!(t, 0x90, "add-int", Format::F23x);
    set!(t, 0x91, "sub-int", Format::F23x);
    set!(t, 0x92, "mul-int", Format::F23x);
    set!(t, 0x93, "div-int", Format::F23x);
    set!(t, 0x94, "rem-int", Format::F23x);
    set!(t, 0x95, "and-int", Format::F23x);
    set!(t, 0x96, "or-int", Format::F23x);
    set!(t, 0x97, "xor-int", Format::F23x);
    set!(t, 0x98, "shl-int", Format::F23x);
    set!(t, 0x99, "shr-int", Format::F23x);
    set!(t, 0x9a, "ushr-int", Format::F23x);
    set!(t, 0x9b, "add-long", Format::F23x);
    set!(t, 0x9c, "sub-long", Format::F23x);
    set!(t, 0x9d, "mul-long", Format::F23x);
    set!(t, 0x9e, "div-long", Format::F23x);
    set!(t, 0x9f, "rem-long", Format::F23x);
    set!(t, 0xa0, "and-long", Format::F23x);
    set!(t, 0xa1, "or-long", Format::F23x);
    set!(t, 0xa2, "xor-long", Format::F23x);
    set!(t, 0xa3, "shl-long", Format::F23x);
    set!(t, 0xa4, "shr-long", Format::F23x);
    set!(t, 0xa5, "ushr-long", Format::F23x);
    set!(t, 0xa6, "add-float", Format::F23x);
    set!(t, 0xa7, "sub-float", Format::F23x);
    set!(t, 0xa8, "mul-float", Format::F23x);
    set!(t, 0xa9, "div-float", Format::F23x);
    set!(t, 0xaa, "rem-float", Format::F23x);
    set!(t, 0xab, "add-double", Format::F23x);
    set!(t, 0xac, "sub-double", Format::F23x);
    set!(t, 0xad, "mul-double", Format::F23x);
    set!(t, 0xae, "div-double", Format::F23x);
    set!(t, 0xaf, "rem-double", Format::F23x);
    set!(t, 0xb0, "add-int/2addr", Format::F12x);
    set!(t, 0xb1, "sub-int/2addr", Format::F12x);
    set!(t, 0xb2, "mul-int/2addr", Format::F12x);
    set!(t, 0xb3, "div-int/2addr", Format::F12x);
    set!(t, 0xb4, "rem-int/2addr", Format::F12x);
    set!(t, 0xb5, "and-int/2addr", Format::F12x);
    set!(t, 0xb6, "or-int/2addr", Format::F12x);
    set!(t, 0xb7, "xor-int/2addr", Format::F12x);
    set!(t, 0xb8, "shl-int/2addr", Format::F12x);
    set!(t, 0xb9, "shr-int/2addr", Format::F12x);
    set!(t, 0xba, "ushr-int/2addr", Format::F12x);
    set!(t, 0xbb, "add-long/2addr", Format::F12x);
    set!(t, 0xbc, "sub-long/2addr", Format::F12x);
    set!(t, 0xbd, "mul-long/2addr", Format::F12x);
    set!(t, 0xbe, "div-long/2addr", Format::F12x);
    set!(t, 0xbf, "rem-long/2addr", Format::F12x);
    set!(t, 0xc0, "and-long/2addr", Format::F12x);
    set!(t, 0xc1, "or-long/2addr", Format::F12x);
    set!(t, 0xc2, "xor-long/2addr", Format::F12x);
    set!(t, 0xc3, "shl-long/2addr", Format::F12x);
    set!(t, 0xc4, "shr-long/2addr", Format::F12x);
    set!(t, 0xc5, "ushr-long/2addr", Format::F12x);
    set!(t, 0xc6, "add-float/2addr", Format::F12x);
    set!(t, 0xc7, "sub-float/2addr", Format::F12x);
    set!(t, 0xc8, "mul-float/2addr", Format::F12x);
    set!(t, 0xc9, "div-float/2addr", Format::F12x);
    set!(t, 0xca, "rem-float/2addr", Format::F12x);
    set!(t, 0xcb, "add-double/2addr", Format::F12x);
    set!(t, 0xcc, "sub-double/2addr", Format::F12x);
    set!(t, 0xcd, "mul-double/2addr", Format::F12x);
    set!(t, 0xce, "div-double/2addr", Format::F12x);
    set!(t, 0xcf, "rem-double/2addr", Format::F12x);
    set!(t, 0xd0, "add-int/lit16", Format::F22s);
    set!(t, 0xd1, "rsub-int", Format::F22s);
    set!(t, 0xd2, "mul-int/lit16", Format::F22s);
    set!(t, 0xd3, "div-int/lit16", Format::F22s);
    set!(t, 0xd4, "rem-int/lit16", Format::F22s);
    set!(t, 0xd5, "and-int/lit16", Format::F22s);
    set!(t, 0xd6, "or-int/lit16", Format::F22s);
    set!(t, 0xd7, "xor-int/lit16", Format::F22s);
    set!(t, 0xd8, "add-int/lit8", Format::F22b);
    set!(t, 0xd9, "rsub-int/lit8", Format::F22b);
    set!(t, 0xda, "mul-int/lit8", Format::F22b);
    set!(t, 0xdb, "div-int/lit8", Format::F22b);
    set!(t, 0xdc, "rem-int/lit8", Format::F22b);
    set!(t, 0xdd, "and-int/lit8", Format::F22b);
    set!(t, 0xde, "or-int/lit8", Format::F22b);
    set!(t, 0xdf, "xor-int/lit8", Format::F22b);
    set!(t, 0xe0, "shl-int/lit8", Format::F22b);
    set!(t, 0xe1, "shr-int/lit8", Format::F22b);
    set!(t, 0xe2, "ushr-int/lit8", Format::F22b);
    // 0xe3..0xf9 unused (0xd8..0xe2 são *int/lit8 válidos — issue #13/#36)
    set!(t, 0xfa, "invoke-polymorphic", Format::F45cc);
    set!(t, 0xfb, "invoke-polymorphic/range", Format::F4rcc);
    set!(t, 0xfc, "invoke-custom", Format::F35c);
    set!(t, 0xfd, "invoke-custom/range", Format::F3rc);
    set!(t, 0xfe, "const-method-handle", Format::F21c);
    set!(t, 0xff, "const-method-type", Format::F21c);
    // marca os slots unused explicitamente (nomes p/ testes)
    let mut i = 0x3e;
    while i <= 0x43 {
        t[i as usize] = unused(i);
        i += 1;
    }
    t[0x73] = unused(0x73);
    t[0x79] = unused(0x79);
    t[0x7a] = unused(0x7a);
    let mut i = 0xe3; // 0xe0..0xe2 são shl/shr/ushr-int/lit8 (22b) — não unused
    while i <= 0xf9 {
        t[i as usize] = unused(i);
        i += 1;
    }
    t
}

/// Lookup O(1) por valor do opcode.
pub fn info(value: u8) -> &'static OpcodeInfo {
    &OPCODES[value as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_complete_and_indexed() {
        for (i, e) in OPCODES.iter().enumerate() {
            assert_eq!(e.value as usize, i, "opcode 0x{i:02x} fora de ordem");
            assert!(!e.name.is_empty());
        }
    }

    #[test]
    fn spot_checks() {
        assert_eq!(info(0x00).name, "nop");
        assert_eq!(info(0x00).format, Format::F10x);
        let o = info(0x6e);
        assert_eq!(o.name, "invoke-virtual");
        assert_eq!(o.format, Format::F35c);
        assert_eq!(info(0x18).name, "const-wide");
        assert_eq!(info(0x18).format, Format::F51l);
        assert_eq!(info(0xfa).name, "invoke-polymorphic");
        assert_eq!(info(0xfa).format, Format::F45cc);
        assert_eq!(info(0xfb).format, Format::F4rcc);
        assert_eq!(info(0xfc).name, "invoke-custom");
        assert_eq!(info(0xff).name, "const-method-type");
        assert_eq!(info(0xcc).name, "sub-double/2addr");
        assert_eq!(info(0xb9).name, "shr-int/2addr");
    }

    #[test]
    fn unused_ranges() {
        for v in [0x3eu8, 0x40, 0x43, 0x73, 0x79, 0x7a, 0xe3, 0xf0, 0xf9] {
            assert!(info(v).is_unused(), "0x{v:02x} devia ser unused");
            assert_eq!(info(v).format, Format::F10x);
        }
        for v in [0x3du8, 0x44, 0x72, 0x78, 0xdf, 0xe0, 0xe2, 0xfa, 0xff] {
            assert!(!info(v).is_unused(), "0x{v:02x} não devia ser unused");
        }
    }

    #[test]
    fn format_sizes() {
        assert_eq!(Format::F10x.size(), 1);
        assert_eq!(Format::F35c.size(), 3);
        // 45cc/4rcc carregam method idx + regs + proto idx → 4 units
        assert_eq!(Format::F45cc.size(), 4);
        assert_eq!(Format::F4rcc.size(), 4);
        assert_eq!(Format::F51l.size(), 5);
    }

    #[test]
    fn payload_kinds() {
        assert_eq!(
            PayloadKind::from_unit(0x0100),
            Some(PayloadKind::PackedSwitch)
        );
        assert_eq!(
            PayloadKind::from_unit(0x0200),
            Some(PayloadKind::SparseSwitch)
        );
        assert_eq!(PayloadKind::from_unit(0x0300), Some(PayloadKind::ArrayData));
        assert_eq!(PayloadKind::from_unit(0x0000), None);
    }
}
