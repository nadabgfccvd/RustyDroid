//! resources.arsc — parse mínimo porém correto (ResTable) + resolução de recursos.
//!
//! Escopo M0: resolver **strings** (labels) e **nomes** (`@type/key`) por resource id,
//! com preferência pela config default. Valores complexos são marcados (`complex`)
//! sem desdobrar bag/maps — suficiente para o inspect e para o motor de permissões.

use crate::error::{RdError, RdResult};
use crate::pool::StringPool;
use std::collections::BTreeMap;

pub const RES_TABLE_TYPE: u16 = 0x0002;
pub const RES_TABLE_PACKAGE_TYPE: u16 = 0x0200;
pub const RES_TABLE_TYPE_SPEC: u16 = 0x0202;
pub const RES_TABLE_TYPE_CHUNK: u16 = 0x0201;

const FLAG_SPARSE: u8 = 0x01;
const FLAG_OFFSET16: u8 = 0x02;
const ENTRY_FLAG_COMPLEX: u16 = 0x0001;

#[inline]
fn u16_at(d: &[u8], off: usize) -> RdResult<u16> {
    d.get(off..off + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| RdError::parse(format!("arsc: truncated u16 @ {off}")))
}

#[inline]
fn u32_at(d: &[u8], off: usize) -> RdResult<u32> {
    d.get(off..off + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| RdError::parse(format!("arsc: truncated u32 @ {off}")))
}

/// Config simplificada — só o que o M0 precisa para escolher a variante certa.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResConfigInfo {
    pub language: Option<String>,
    pub country: Option<String>,
    pub density: Option<u16>,
    pub api_level: Option<u16>,
}

impl ResConfigInfo {
    pub fn is_default(&self) -> bool {
        self.language.is_none() && self.density.map_or(true, |d| d == 0 || d == 160)
    }
    fn specificity(&self) -> u8 {
        let mut s = 0;
        if self.language.is_some() {
            s += 2;
        }
        if self.country.is_some() {
            s += 1;
        }
        s
    }
}

/// Uma entrada de recurso resolvida para uma config.
#[derive(Debug, Clone)]
pub struct ResEntry {
    pub config: ResConfigInfo,
    pub data_type: u8,
    pub data: u32,
    /// Pré-resolvida quando `data_type == 3` (STRING → pool global).
    pub string: Option<String>,
    pub complex: bool,
    /// Índice no key string pool (para `resolve_name`).
    pub key_idx: u32,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub id: u8,
    pub name: String,
    pub type_strings: StringPool,
    pub key_strings: StringPool,
    /// resid completo → entradas por config.
    pub entries: BTreeMap<u32, Vec<ResEntry>>,
}

#[derive(Debug, Clone, Default)]
pub struct Arsc {
    pub global_strings: StringPool,
    pub packages: Vec<Package>,
}

impl Package {
    /// Nome qualificado `@type/key` de um resid (sem pacote).
    pub fn resolve_name(&self, res_id: u32) -> Option<String> {
        let entries = self.entries.get(&res_id)?;
        let key = entries.first()?;
        let type_idx = ((res_id >> 16) & 0xFF) as usize; // 1-based
        let type_name = self.type_strings.get((type_idx as u32).wrapping_sub(1))?;
        let key_name = self.key_strings.get(key.key_idx)?;
        Some(format!("@{type_name}/{key_name}"))
    }
}

impl Arsc {
    pub fn parse(data: &[u8]) -> RdResult<Arsc> {
        if data.len() < 12 {
            return Err(RdError::parse("arsc: shorter than ResTable header"));
        }
        if u16_at(data, 0)? != RES_TABLE_TYPE {
            return Err(RdError::parse(format!(
                "arsc: expected type 0x{RES_TABLE_TYPE:04x}, got 0x{:04x}",
                u16_at(data, 0)?
            )));
        }
        let _header_size = u16_at(data, 2)? as usize;
        let file_size = u32_at(data, 4)? as usize;
        let data = &data[..file_size.min(data.len())];
        let package_count = u32_at(data, 8)?;

        // pool global logo após o header (12 bytes)
        let (global_strings, pool_used) = StringPool::parse(data, 12)?;
        let mut p = 12 + pool_used as usize;

        let mut packages = Vec::new();
        let mut seen_packages = 0u32;
        while p + 8 <= data.len() && seen_packages < package_count {
            let ctype = u16_at(data, p)?;
            let cheader = u16_at(data, p + 2)? as usize;
            let csize = u32_at(data, p + 4)? as usize;
            if csize < cheader || p + csize > data.len() {
                return Err(RdError::parse(format!(
                    "arsc: package chunk at {p} declares {csize} beyond bounds"
                )));
            }
            if ctype == RES_TABLE_PACKAGE_TYPE {
                let pkg_header_size = u16_at(data, p + 2)? as usize;
                packages.push(parse_package(
                    data,
                    p,
                    csize,
                    pkg_header_size,
                    &global_strings,
                )?);
                seen_packages += 1;
            }
            p += csize;
        }
        if seen_packages != package_count {
            return Err(RdError::invalid_format(format!(
                "arsc: header announces {package_count} packages, walked {seen_packages}"
            )));
        }

        Ok(Arsc {
            global_strings,
            packages,
        })
    }

    /// Resolve o resource id para string, seguindo cadeias de referência
    /// (ex.: `@string/app_name` → alias → texto). Limite de 8 saltos contra ciclos.
    pub fn resolve_string(&self, res_id: u32) -> Option<String> {
        self.resolve_string_depth(res_id, 0)
    }

    fn resolve_string_depth(&self, res_id: u32, depth: usize) -> Option<String> {
        if depth > 8 {
            return None;
        }
        let pkg = self.package_for(res_id)?;
        let entries = pkg.entries.get(&res_id)?;
        let best = entries
            .iter()
            .min_by_key(|e| (e.config.specificity(), !e.config.is_default()))?;
        match best.data_type {
            3 => best.string.clone(),
            1 => self.resolve_string_depth(best.data, depth + 1), // REFERENCE
            _ => None,
        }
    }

    /// `@type/key` do recurso (primeiro pacote que o conhece).
    pub fn resolve_name(&self, res_id: u32) -> Option<String> {
        let pkg = self.package_for(res_id)?;
        pkg.resolve_name(res_id)
    }

    pub fn package_for(&self, res_id: u32) -> Option<&Package> {
        let pid = ((res_id >> 24) & 0xFF) as u8;
        self.packages
            .iter()
            .find(|p| p.id == pid)
            .or_else(|| self.packages.first())
    }
}

fn parse_package(
    data: &[u8],
    base: usize,
    size: usize,
    header_size: usize,
    global: &StringPool,
) -> RdResult<Package> {
    if base + 288 > data.len() {
        return Err(RdError::parse("arsc: package chunk truncated (header)"));
    }
    let pkg_id = (u32_at(data, base + 8)? & 0xFF) as u8;
    let type_strings_off = u32_at(data, base + 268)? as usize;
    let key_strings_off = u32_at(data, base + 276)? as usize;

    // name: 128 UTF-16 units @12
    let mut name_units = Vec::with_capacity(128);
    for i in 0..128 {
        let u = u16_at(data, base + 12 + i * 2)?;
        if u == 0 {
            break;
        }
        name_units.push(u);
    }
    let name = String::from_utf16_lossy(&name_units);

    let (type_strings, _) = StringPool::parse(data, base + type_strings_off)?;
    let (key_strings, _) = StringPool::parse(data, base + key_strings_off)?;

    let mut entries: BTreeMap<u32, Vec<ResEntry>> = BTreeMap::new();

    // caminhar chunks internos a partir do fim do header real do pacote
    let mut p = base + header_size;
    let end = base + size;
    while p + 8 <= end {
        let ctype = u16_at(data, p)?;
        let cheader = u16_at(data, p + 2)? as usize;
        let csize = u32_at(data, p + 4)? as usize;
        // progresso garantido (chunk mínimo = 8 bytes) — endurece contra fuzz
        if csize < 8 || csize < cheader || p + csize > end {
            break; // chunk truncado: não fatal para o M0 — o que veio antes fica
        }
        if ctype == RES_TABLE_TYPE_CHUNK {
            let type_id = data.get(p + 8).copied().unwrap_or(0);
            let flags = data.get(p + 9).copied().unwrap_or(0);
            let entry_count = u32_at(data, p + 12)? as usize;
            let entries_start = u32_at(data, p + 16)? as usize;

            let config = parse_config(data, p + 20, cheader - 20)?;

            let entries_abs = p + entries_start;
            let mut collect = |idx: usize, rel_off: u32| -> RdResult<()> {
                if rel_off == 0xFFFF_FFFF {
                    return Ok(());
                }
                let eoff = entries_abs + rel_off as usize;
                if eoff + 8 > data.len() {
                    return Ok(()); // entrada truncada: skip
                }
                let e_size = u16_at(data, eoff)? as usize;
                let e_flags = u16_at(data, eoff + 2)?;
                let key_idx = u32_at(data, eoff + 4)?;
                let (data_type, ddata, complex) = if e_flags & ENTRY_FLAG_COMPLEX != 0 {
                    (0u8, 0u32, true)
                } else {
                    // Res_value após ResTable_entry (e_size bytes)
                    let v = eoff + e_size;
                    if v + 8 > data.len() {
                        return Ok(());
                    }
                    let dtype = *data.get(v + 3).unwrap_or(&0);
                    let d = u32_at(data, v + 4)?;
                    (dtype, d, false)
                };
                let string = if data_type == 3 && !complex {
                    global.get(ddata).map(str::to_owned)
                } else {
                    None
                };
                let resid =
                    ((pkg_id as u32) << 24) | ((type_id as u32) << 16) | (idx as u32 & 0xFFFF);
                entries.entry(resid).or_default().push(ResEntry {
                    config: config.clone(),
                    data_type,
                    data: ddata,
                    string,
                    complex,
                    key_idx,
                });
                Ok(())
            };

            if flags & FLAG_SPARSE != 0 {
                // ResTable_sparseTypeEntry: {idx u16, offset u16} — offset/4
                let pairs = entry_count;
                for i in 0..pairs {
                    let o = p + cheader + i * 4;
                    if o + 4 > data.len() {
                        break;
                    }
                    let idx = u16_at(data, o)? as usize;
                    let off4 = u16_at(data, o + 2)? as u32;
                    collect(idx, off4 * 4)?;
                }
            } else if flags & FLAG_OFFSET16 != 0 {
                for i in 0..entry_count {
                    let o = p + cheader + i * 2;
                    if o + 2 > data.len() {
                        break;
                    }
                    collect(i, u16_at(data, o)? as u32 * 4)?;
                }
            } else {
                for i in 0..entry_count {
                    let o = p + cheader + i * 4;
                    if o + 4 > data.len() {
                        break;
                    }
                    collect(i, u32_at(data, o)?)?;
                }
            }
        }
        p += csize;
    }

    Ok(Package {
        id: pkg_id,
        name,
        type_strings,
        key_strings,
        entries,
    })
}

/// ResTable_config — extrai locale/densidade/api sem reclamar de variações de tamanho.
fn parse_config(data: &[u8], cfg_base: usize, cfg_max: usize) -> RdResult<ResConfigInfo> {
    let mut cfg = ResConfigInfo::default();
    if cfg_max < 4 || cfg_base + 4 > data.len() {
        return Ok(cfg);
    }
    let cfg_size = u32_at(data, cfg_base)? as usize;
    let readable = cfg_size
        .min(cfg_max)
        .min(data.len().saturating_sub(cfg_base));
    let lang = |off: usize| -> Option<String> {
        let b = data.get(cfg_base + off..cfg_base + off + 2)?;
        if b[0] == 0 && b[1] == 0 {
            None
        } else {
            // estilo AOSP: 0 se byte < 128, senão 'a' + (b - 128) (packed unicode é raro)
            let chars: Vec<char> = b
                .iter()
                .filter(|&&c| c != 0)
                .map(|&c| {
                    if c < 128 {
                        c as char
                    } else {
                        char::from_u32('a' as u32 + (c - 128) as u32).unwrap_or('?')
                    }
                })
                .collect();
            Some(chars.into_iter().collect())
        }
    };
    if readable >= 12 {
        cfg.language = lang(8);
        cfg.country = lang(10);
    }
    if readable >= 16 {
        let d = u16_at(data, cfg_base + 14)?;
        cfg.density = Some(d);
    }
    if readable >= 28 {
        cfg.api_level = Some(u16_at(data, cfg_base + 24)?);
    }
    Ok(cfg)
}
