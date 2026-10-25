//! ResStringPool — string pool do Android (usada por AXML e resources.arsc).
//!
//! Formato: `source.android.com` → "String Pool Resource" / ResStringPool_header.
//! Suporta UTF-16 e UTF-8 (flag `UTF8_FLAG`), com comprimentos prefixados e
//! terminador nulo — decodificação total com bounds-checks (parsers nunca confiam).
//!
//! **Lazy (ADR 014):** o parse valida o header/limites e retém os bytes crus do
//! chunk; strings individuais são decodificadas NA PRIMEIRA leitura (`get`) e
//! memoizadas por slot (`OnceLock`). O cold path do ADR 008 toca <2–5% das
//! strings; pior caso de memória = eager de hoje (todas tocadas), estritamente
//! menor quando um subconjunto é tocado. Strings malformadas individuais
//! retornam `None` no acesso (o parse eager as rejeitava antecipadamente);
//! erros estruturais do pool continuam `RdError` no parse — Lei 1 preservada.

use crate::error::{RdError, RdResult};
use std::sync::{Arc, OnceLock};

pub const RES_STRING_POOL_TYPE: u16 = 0x0001;
pub const UTF8_FLAG: u32 = 1 << 8;

#[inline]
fn u16_at(d: &[u8], off: usize) -> RdResult<u16> {
    d.get(off..off + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| RdError::parse(format!("string pool: truncated read u16 @ {off}")))
}

#[inline]
fn u32_at(d: &[u8], off: usize) -> RdResult<u32> {
    d.get(off..off + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| RdError::parse(format!("string pool: truncated read u32 @ {off}")))
}

/// Pool lazy (ADR 014): bytes crus compartilhados + cache por slot.
///
/// `raw` é o documento INTEIRO (ARSC ou AXML) atrás de um `Arc` — o pool é uma
/// REGIÃO lógica `[chunk_off..chunk_off+chunk_size]` dele (Lei 2: bytes/offsets
/// crus preservados; clones compartilham o buffer). `offsets_base`/`strings_base`
/// são relativos ao início do pool.
#[derive(Debug, Default)]
pub struct StringPool {
    raw: Arc<Vec<u8>>,
    chunk_off: usize,
    chunk_size: usize,
    count: usize,
    offsets_base: usize,
    strings_base: usize,
    utf8: bool,
    /// Cache por slot (ALOCAÇÃO PREGUIÇOSA — ADR 014): `Vec<OnceLock<..>>`
    /// só existe após o primeiro `get`; pools nunca tocadas não pagam os
    /// 410k slots do torture-huge-arsc. Decodifica 1x, memoiza; `None`
    /// interno = string malformada (falha memoizada).
    cache: OnceLock<Vec<OnceLock<Option<String>>>>,
}

impl Clone for StringPool {
    /// Clone = mesma região (Arc compartilhado), cache renasce vazio
    /// (decodifica de novo por instância — determinístico; nunca compartilha
    /// `OnceLock` inicializados).
    fn clone(&self) -> Self {
        Self {
            raw: Arc::clone(&self.raw),
            chunk_off: self.chunk_off,
            chunk_size: self.chunk_size,
            count: self.count,
            offsets_base: self.offsets_base,
            strings_base: self.strings_base,
            utf8: self.utf8,
            cache: OnceLock::new(), // renasce vazio (decodifica de novo)
        }
    }
}
impl StringPool {
    /// Parse de um chunk de string pool em `data[chunk_off..]` (caminho
    /// standalone — AXML): retém UMA cópia do documento via `Arc`.
    /// Retorna `(pool, tamanho total do chunk)` — o caller continua nos bytes seguintes.
    pub fn parse(data: &[u8], chunk_off: usize) -> RdResult<(Self, u32)> {
        let raw = Arc::new(data.to_vec());
        Self::parse_shared(&raw, chunk_off)
    }

    /// Parse compartilhado (caminho ARSC): o documento inteiro já vive num
    /// `Arc` — o pool é só uma região lógica dele, ZERO cópias extras.
    pub fn parse_shared(raw: &Arc<Vec<u8>>, chunk_off: usize) -> RdResult<(Self, u32)> {
        let data: &[u8] = raw;
        let base = chunk_off;
        if data.len() < base + 28 {
            return Err(RdError::parse(
                "string pool: chunk shorter than header (28)",
            ));
        }
        let chunk_type = u16_at(data, base)?;
        if chunk_type != RES_STRING_POOL_TYPE {
            return Err(RdError::parse(format!(
                "string pool: expected type 0x{RES_STRING_POOL_TYPE:04x}, got 0x{chunk_type:04x}"
            )));
        }
        let header_size = u16_at(data, base + 2)? as usize;
        let chunk_size = u32_at(data, base + 4)? as usize;
        let string_count = u32_at(data, base + 8)? as usize;
        let _style_count = u32_at(data, base + 12)? as usize;
        let flags = u32_at(data, base + 16)?;
        let strings_start = u32_at(data, base + 20)? as usize;
        let _styles_start = u32_at(data, base + 24)? as usize;

        // Heurística de sanidade: pools reais têm headerSize 28 e chunk limitado.
        if chunk_size == 0 || chunk_size > data.len() - base {
            return Err(RdError::parse(format!(
                "string pool: chunk size {chunk_size} exceeds container"
            )));
        }
        if string_count > chunk_size {
            return Err(RdError::parse(format!(
                "string pool: implausible string count {string_count}"
            )));
        }
        if header_size < 28 {
            return Err(RdError::parse(format!(
                "string pool: header size {header_size} < 28"
            )));
        }
        // Validação O(1) (ADR 014): a tabela de offsets precisa caber no
        // documento. Offsets individuais fora do documento/truncados são
        // rejeitados NO ACESSO (`get` → None): o eager falhava o parse
        // inteiro na primeira string ruim; o lazy isola a string ruim
        // (Lei 1 mantida no nível estrutural do header/tabela).
        if string_count > 0 {
            let offsets_end = base + header_size + string_count * 4;
            if offsets_end > data.len() {
                return Err(RdError::parse(format!(
                    "string pool: offsets table ({string_count} entries) exceeds document"
                )));
            }
        }

        Ok((
            StringPool {
                raw: Arc::clone(raw),
                chunk_off: base,
                chunk_size,
                count: string_count,
                offsets_base: header_size,
                strings_base: strings_start,
                utf8: flags & UTF8_FLAG != 0,
                cache: OnceLock::new(),
            },
            chunk_size as u32,
        ))
    }

    /// UTF-16: [len:u16 (ou 2×u16 se bit alto)] [len × u16 units] [0x0000].
    fn read_utf16(data: &[u8], off: usize) -> RdResult<String> {
        let mut p = off;
        let mut len = u16_at(data, p)? as usize;
        p += 2;
        if len & 0x8000 != 0 {
            len = ((len & 0x7FFF) << 16) | u16_at(data, p)? as usize;
            p += 2;
        }
        if len > data.len() {
            return Err(RdError::parse("string pool: utf16 length out of bounds"));
        }
        let mut units = Vec::with_capacity(len);
        for _ in 0..len {
            units.push(u16_at(data, p)?);
            p += 2;
        }
        Ok(String::from_utf16_lossy(&units))
    }

    /// UTF-8: [declared chars: varint] [byte len: varint] [bytes] [0x00].
    fn read_utf8(data: &[u8], off: usize) -> RdResult<String> {
        let mut p = off;
        // 1º varint: comprimento em caracteres — decodificado mas ignorado
        let _decl_chars = Self::read_varint(data, &mut p)?;
        // 2º varint: comprimento em bytes (o que usamos de fato)
        let byte_len = Self::read_varint(data, &mut p)?;
        let end = p
            .checked_add(byte_len)
            .ok_or_else(|| RdError::parse("string pool: utf8 overflow"))?;
        let bytes = data
            .get(p..end)
            .ok_or_else(|| RdError::parse("string pool: utf8 string out of bounds"))?;
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    /// Varint do ResStringPool UTF-8: 1 byte, ou 2 se o bit alto do 1º estiver setado
    /// (nesse caso o valor é `((b0 & 0x7F) << 8) | b1`).
    fn read_varint(data: &[u8], p: &mut usize) -> RdResult<usize> {
        let b0 = *data
            .get(*p)
            .ok_or_else(|| RdError::parse("string pool: utf8 truncated (varint)"))?;
        *p += 1;
        if b0 & 0x80 == 0 {
            return Ok(b0 as usize);
        }
        let b1 = *data
            .get(*p)
            .ok_or_else(|| RdError::parse("string pool: utf8 truncated (varint hi)"))?;
        *p += 1;
        Ok((((b0 & 0x7F) as usize) << 8) | b1 as usize)
    }

    /// String por índice, decodificando NA PRIMEIRA leitura e memoizando.
    /// `None` = índice fora da pool OU string individual malformada (o eager
    /// rejeitava essa string no parse; o lazy reporta no acesso — ADR 014).
    /// Strings vazias legítimas permanecem `Some("")`.
    pub fn get(&self, idx: u32) -> Option<&str> {
        let idx = idx as usize;
        if idx >= self.count {
            return None;
        }
        let slots = self
            .cache
            .get_or_init(|| (0..self.count).map(|_| OnceLock::new()).collect::<Vec<_>>());
        let slot = slots.get(idx)?;
        // get_or_init é &self: primeira leitura decodifica; demais retornam o cache.
        slot.get_or_init(|| self.decode(idx)).as_deref()
    }

    /// Decodifica a string `idx` dos bytes crus (Lei 2). Erro → `None`
    /// (memoizado): string malformada individual não derruba o pool inteiro.
    /// Bounds contra o documento INTEIRO — idêntico ao eager (que decodificava
    /// com `data.len()` do buffer completo).
    fn decode(&self, idx: usize) -> Option<String> {
        let data: &[u8] = &self.raw;
        let rel = match u32_at(data, self.chunk_off + self.offsets_base + idx * 4) {
            Ok(r) => r as usize,
            Err(_) => return None,
        };
        let abs = self.chunk_off + self.strings_base + rel;
        if self.utf8 {
            Self::read_utf8(data, abs).ok()
        } else {
            Self::read_utf16(data, abs).ok()
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Itera decodificando sob demanda (mesma ordem/valores do eager quando
    /// todas as strings são sãs; malformadas são puladas — sem consumidores
    /// externos hoje, mantido por compatibilidade de API).
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        (0..self.count).filter_map(move |i| self.get(i as u32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Constrói um pool UTF-8 mínimo à mão: ["hello", "world"]
    fn handcrafted_utf8_pool() -> Vec<u8> {
        // header (28 bytes)
        let mut v = Vec::new();
        v.extend(RES_STRING_POOL_TYPE.to_le_bytes()); // type
        v.extend(28u16.to_le_bytes()); // headerSize
        v.extend(0u32.to_le_bytes()); // size (patched later)
        v.extend(2u32.to_le_bytes()); // stringCount
        v.extend(0u32.to_le_bytes()); // styleCount
        v.extend(UTF8_FLAG.to_le_bytes()); // flags
        v.extend(36u32.to_le_bytes()); // stringsStart = 28 + 2*4 offsets
        v.extend(0u32.to_le_bytes()); // stylesStart
                                      // offsets (string 2 começa em 8: [5][5]"hello"[0] = 8 bytes)
        v.extend(0u32.to_le_bytes());
        v.extend(8u32.to_le_bytes());
        // strings: [decl chars][byte len][bytes][\0]
        v.push(5);
        v.push(5);
        v.extend_from_slice(b"hello");
        v.push(0);
        v.push(5);
        v.push(5);
        v.extend_from_slice(b"world");
        v.push(0);
        let size = v.len() as u32;
        v[4..8].copy_from_slice(&size.to_le_bytes());
        v
    }

    #[test]
    fn parses_handcrafted_utf8_pool() {
        let bytes = handcrafted_utf8_pool();
        let (pool, size) = StringPool::parse(&bytes, 0).unwrap();
        assert_eq!(size as usize, bytes.len());
        assert_eq!(pool.get(0), Some("hello"));
        assert_eq!(pool.get(1), Some("world"));
        assert_eq!(pool.get(2), None);
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn rejects_garbage_type() {
        let mut bytes = handcrafted_utf8_pool();
        bytes[0..2].copy_from_slice(&0xBEEFu16.to_le_bytes());
        assert!(StringPool::parse(&bytes, 0).is_err());
    }

    #[test]
    fn rejects_out_of_bounds_chunk() {
        let bytes = handcrafted_utf8_pool();
        assert!(StringPool::parse(&bytes, usize::MAX / 2).is_err());
    }

    #[test]
    fn lazy_decodes_once_and_memoizes() {
        let bytes = handcrafted_utf8_pool();
        let (pool, _) = StringPool::parse(&bytes, 0).unwrap();
        let a = pool.get(0);
        let b = pool.get(0);
        // Mesmo endereço → memoizado (OnceLock devolve a mesma String).
        assert!(std::ptr::eq(a.unwrap(), b.unwrap()));
        assert_eq!(a, Some("hello"));
    }

    #[test]
    fn clone_resets_cache_but_keeps_content() {
        let bytes = handcrafted_utf8_pool();
        let (pool, _) = StringPool::parse(&bytes, 0).unwrap();
        let _ = pool.get(0); // aquece o cache
        let cloned = pool.clone();
        // Clone decodifica de novo (cache fresco) com o MESMO resultado.
        assert_eq!(cloned.get(0), Some("hello"));
        assert_eq!(cloned.get(1), Some("world"));
        // E os ponteiros são independentes (caches distintos).
        assert!(!std::ptr::eq(pool.get(0).unwrap(), cloned.get(0).unwrap()));
    }

    #[test]
    fn empty_string_is_legitimate() {
        // Pool com uma string vazia UTF-8: [0][0][\0]
        let mut v = Vec::new();
        v.extend(RES_STRING_POOL_TYPE.to_le_bytes());
        v.extend(28u16.to_le_bytes());
        v.extend(0u32.to_le_bytes());
        v.extend(1u32.to_le_bytes());
        v.extend(0u32.to_le_bytes());
        v.extend(UTF8_FLAG.to_le_bytes());
        v.extend(32u32.to_le_bytes()); // stringsStart = 28 + 1*4 (1 offset)
        v.extend(0u32.to_le_bytes()); // stylesStart
        v.extend(0u32.to_le_bytes()); // offset da string 0 (rel 0)
        v.push(0); // decl chars
        v.push(0); // byte len
        v.push(0); // terminador
        let size = v.len() as u32;
        v[4..8].copy_from_slice(&size.to_le_bytes());
        let (pool, _) = StringPool::parse(&v, 0).unwrap();
        assert_eq!(pool.get(0), Some(""));
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn malformed_string_yields_none_not_panic() {
        // ADR 014: offsets individuais ruins são rejeitados NO ACESSO —
        // o parse aceita a estrutura (Lei 1 segue no nível do pool/header).
        // string 0 → offset gigante (fora do documento): get → None.
        let mut v = handcrafted_utf8_pool();
        v[28..32].copy_from_slice(&0xFFFFu32.to_le_bytes());
        let (pool, _) = StringPool::parse(&v, 0).unwrap();
        assert_eq!(pool.get(0), None);
        assert_eq!(pool.get(1), Some("world"));

        // offset DENTRO do documento, mas em bytes que não formam string
        // válida: get → None (falha memoizada, não pânico, não "").
        let mut w = handcrafted_utf8_pool();
        // string 1 → rel 11 → abs 47 = byte 'd': decl varint 100, depois
        // byte_len varint fora do documento → Err → None.
        w[32..36].copy_from_slice(&11u32.to_le_bytes());
        let (pool, _) = StringPool::parse(&w, 0).unwrap();
        assert_eq!(pool.get(1), None);
        assert_eq!(pool.get(0), Some("hello"));
    }

    #[test]
    fn iter_decodes_on_demand() {
        let bytes = handcrafted_utf8_pool();
        let (pool, _) = StringPool::parse(&bytes, 0).unwrap();
        let collected: Vec<&str> = pool.iter().collect();
        assert_eq!(collected, vec!["hello", "world"]);
    }
}
