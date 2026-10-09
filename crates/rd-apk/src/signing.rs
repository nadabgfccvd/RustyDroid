//! Detecção de esquemas de assinatura de APK (v1 / v2 / v3 / v3.1).
//!
//! Escopo M0: presença + versões (não validação criptográfica — chega em fase
//! posterior). v2/v3 = "APK Sig Block 42" antes do central directory (docs
//! source.android.com → APK Signature Scheme v2/v3). v1 = JAR signing (META-INF).

use crate::zip::Zip;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SigningInfo {
    /// JAR signing (META-INF/*.RSA|.DSA|.EC + MANIFEST.MF).
    pub v1: bool,
    /// APK Signature Scheme v2.
    pub v2: bool,
    /// v3 (rotação de chaves).
    pub v3: bool,
    /// v3.1 (rotação com lineage duplo).
    pub v3_1: bool,
}

impl SigningInfo {
    pub fn schemes(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if self.v1 {
            v.push("v1");
        }
        if self.v2 {
            v.push("v2");
        }
        if self.v3 {
            v.push("v3");
        }
        if self.v3_1 {
            v.push("v3.1");
        }
        v
    }

    pub fn is_signed(&self) -> bool {
        self.v1 || self.v2 || self.v3 || self.v3_1
    }
}

pub const APK_SIG_BLOCK_MAGIC: &[u8; 16] = b"APK Sig Block 42";
pub const BLOCK_ID_V2: u32 = 0x7109_871a;
pub const BLOCK_ID_V3: u32 = 0xf053_68c0;
pub const BLOCK_ID_V3_1: u32 = 0x3ba0_6f8c;

#[inline]
fn u32_at(d: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([d[off], d[off + 1], d[off + 2], d[off + 3]])
}

#[inline]
fn u64_at(d: &[u8], off: usize) -> u64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&d[off..off + 8]);
    u64::from_le_bytes(a)
}

/// `data` = bytes completos do APK; `cd_offset` = offset do central directory (do EOCD).
pub fn detect(data: &[u8], cd_offset: u64, zip: &Zip) -> SigningInfo {
    let mut info = SigningInfo {
        v1: detect_v1(zip),
        ..Default::default()
    };

    // bloco de assinatura termina exatamente onde o CD começa
    if cd_offset >= 32 {
        let cd = cd_offset as usize;
        if cd >= 24 && &data[cd - 16..cd] == APK_SIG_BLOCK_MAGIC {
            let block_size = u64_at(data, cd - 24) as usize;
            if block_size + 8 <= cd {
                let block_start = cd - block_size - 8;
                if block_start + 8 <= data.len() {
                    walk_pairs(data, block_start + 8, cd - 24, &mut info);
                }
            }
        }
    }
    info
}

fn walk_pairs(data: &[u8], mut p: usize, end: usize, info: &mut SigningInfo) {
    while p + 12 <= end {
        let pair_len = u64_at(data, p) as usize;
        if pair_len < 4 || p + 8 + pair_len > end + 8 {
            break; // par malformado: não travamos a detecção
        }
        let id = u32_at(data, p + 8);
        match id {
            BLOCK_ID_V2 => info.v2 = true,
            BLOCK_ID_V3 => info.v3 = true,
            BLOCK_ID_V3_1 => info.v3_1 = true,
            _ => {}
        }
        p += 8 + pair_len;
    }
}

fn detect_v1(zip: &Zip) -> bool {
    let mut has_manifest_mf = false;
    let mut has_sig = false;
    for name in zip.names() {
        if name == "META-INF/MANIFEST.MF" {
            has_manifest_mf = true;
        }
        let lower = name.to_ascii_lowercase();
        if lower.starts_with("meta-inf/")
            && (lower.ends_with(".rsa") || lower.ends_with(".dsa") || lower.ends_with(".ec"))
        {
            has_sig = true;
        }
    }
    has_manifest_mf && has_sig
}
