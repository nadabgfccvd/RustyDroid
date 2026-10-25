//! resources.arsc — parse mínimo porém correto (ResTable) + resolução de recursos.
//!
//! Escopo M0: resolver **strings** (labels) e **nomes** (`@type/key`) por resource id,
//! com preferência pela config default. Valores complexos são marcados (`complex`)
//! sem desdobrar bag/maps — suficiente para o inspect e para o motor de permissões.
//!
//! **Lazy (ADR 014):** o parse valida a estrutura e constrói um ÍNDICE leve
//! (`resid → chunks de tipo`); as entradas pesadas (`ResEntry`, bags, strings) são
//! materializadas SOB DEMANDA por resid nos `resolve_*` e memoizadas num cache
//! limitado. O cold path do ADR 008 toca <2–5% dos recursos; o resultado de cada
//! resolução é byte-idêntico ao eager (mesma ordem de walk, mesma seleção de
//! config, mesmos guards — Lei 4). O buffer inteiro fica retido atrás de um
//! `Arc` (Lei 2: offsets crus preservados; clones baratos).

use crate::axml::ComplexUnit;
use crate::error::{RdError, RdResult};
use crate::pool::StringPool;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

pub const RES_TABLE_TYPE: u16 = 0x0002;
pub const RES_TABLE_PACKAGE_TYPE: u16 = 0x0200;
pub const RES_TABLE_TYPE_SPEC: u16 = 0x0202;
pub const RES_TABLE_TYPE_CHUNK: u16 = 0x0201;

const FLAG_SPARSE: u8 = 0x01;
const FLAG_OFFSET16: u8 = 0x02;
const ENTRY_FLAG_COMPLEX: u16 = 0x0001;

/// Capacidade do cache de entradas materializadas (ADR 014): limita o pico de
/// RSS em APKs hostis; ao estourar, o cache é reconstruído do zero (bounded,
/// determinístico — só afeta QUANDO decodificar, nunca O QUÊ retornar).
const ENTRY_CACHE_CAP: usize = 8192;

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

/// Uma entrada de um bag/mapa de recurso (por exemplo, um atributo de style).
#[derive(Debug, Clone, PartialEq)]
pub struct ResMapEntry {
    /// Nome/resource id do atributo dentro do bag.
    pub name: u32,
    pub data_type: u8,
    pub data: u32,
    /// Pré-resolvida quando `data_type == 3` (STRING → pool global).
    pub string: Option<String>,
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
    /// Parent style resource id, when this entry is a complex bag.
    pub parent: Option<u32>,
    /// Resource map values when this entry is a complex bag (style/array/etc.).
    pub bag: Option<Vec<ResMapEntry>>,
    /// Índice no key string pool (para `resolve_name`).
    pub key_idx: u32,
}

/// Referência leve a um chunk `ResTable_type` (ADR 014): offsets ABSOLUTOS no
/// buffer do ARSC inteiro — a materialização usa exatamente os mesmos endereços
/// (e os mesmos guards) que o parse eager usava, garantindo paridade byte-a-byte.
#[derive(Debug, Clone)]
struct TypeChunkRef {
    chunk_off: usize,
    chunk_size: usize,
    header_size: usize,
    config: ResConfigInfo,
    entry_count: usize,
    flags: u8,
    entries_start: usize,
}

/// Pacote com índice + materialização sob demanda (ADR 014). O `Clone` é
/// manual: buffers/índice são `Arc` (compartilhados, baratos); o cache de
/// entradas renasce vazio (mesmo conteúdo, decodificação por instância).
#[derive(Debug)]
pub struct Package {
    pub id: u8,
    pub name: String,
    pub type_strings: StringPool,
    pub key_strings: StringPool,
    /// Clone do pool global (Arc compartilhado — barato): a pré-resolução de
    /// strings das entradas materializadas precisa dele, e o pacote fica
    /// autocontido (`resolve_name`/`entries_for` sem parâmetro extra).
    global: StringPool,
    /// Índice leve (ADR 014): chunks de tipo do pacote + pares
    /// (resid, chunk_idx) ORDENADOS. O append durante o parse é O(1); a
    /// busca por resid é binária. A ordem (resid, chunk_idx) reproduz a
    /// ordem de walk do eager — decisiva para o tie-breaking do `min_by_key`.
    chunks: Arc<Vec<TypeChunkRef>>,
    pairs: Arc<Vec<(u32, u32)>>,
    /// Buffer do ARSC inteiro compartilhado (Arc) — Leis 2 e clones baratos.
    raw: Arc<Vec<u8>>,
    /// Cache de entradas materializadas por resid (cap `ENTRY_CACHE_CAP`;
    /// reset ao estourar). Estado POR INSTÂNCIA (nunca global): `Mutex` direto,
    /// sem `OnceLock` — `Mutex::new(BTreeMap::new())` não aloca, então a
    /// inicialização preguiçosa não comprava nada. Lock envenenado = cache
    /// ignorado (re-decodifica), nunca panic.
    cache: Mutex<BTreeMap<u32, Arc<Vec<ResEntry>>>>,
}

impl Clone for Package {
    /// Clone manual (ADR 014): buffer e índice são `Arc` (compartilhados,
    /// O(1)); o cache de entradas renasce vazio — mesmo conteúdo, decodifica
    /// de novo por instância (determinístico).
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            type_strings: self.type_strings.clone(),
            key_strings: self.key_strings.clone(),
            global: self.global.clone(),
            chunks: Arc::clone(&self.chunks),
            pairs: Arc::clone(&self.pairs),
            raw: Arc::clone(&self.raw),
            cache: Mutex::new(BTreeMap::new()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Arsc {
    pub global_strings: StringPool,
    pub packages: Vec<Package>,
}

impl Package {
    /// Construtor sintético (testes/testsupport): entradas já materializadas
    /// viram o cache pré-povoado; o índice fica vazio (nada a materializar).
    pub fn synthetic(
        id: u8,
        name: String,
        type_strings: StringPool,
        key_strings: StringPool,
        entries: BTreeMap<u32, Vec<ResEntry>>,
    ) -> Self {
        let cache = Mutex::new(
            entries
                .into_iter()
                .map(|(k, v)| (k, Arc::new(v)))
                .collect::<BTreeMap<_, _>>(),
        );
        Package {
            id,
            name,
            type_strings,
            key_strings,
            global: StringPool::default(),
            chunks: Arc::new(Vec::new()),
            pairs: Arc::new(Vec::new()),
            raw: Arc::new(Vec::new()),
            cache,
        }
    }

    /// Entradas de um resid, materializando sob demanda (ADR 014) —
    /// snapshot em cache (`Arc` compartilha com resoluções futuras).
    /// Mesma ordem/conteúdo que o eager produzia no parse (mesmos guards,
    /// mesma pré-resolução de strings pela pool global). Ferramentas de
    /// diagnóstico e testes inspecionam aqui; caminho quente usa os
    /// `resolve_*` (que chamam isto internamente).
    pub fn entries_for(&self, res_id: u32) -> Option<Arc<Vec<ResEntry>>> {
        let cache = &self.cache;
        if let Ok(guard) = cache.lock() {
            if let Some(hit) = guard.get(&res_id) {
                return Some(Arc::clone(hit));
            }
        }
        let start = self.pairs.partition_point(|(r, _)| *r < res_id);
        let end = self.pairs.partition_point(|(r, _)| *r <= res_id);
        if start == end {
            return None; // resid desconhecido (índice e cache vazios p/ ele)
        }
        let mut out: Vec<ResEntry> = Vec::new();
        for &(_, ci) in &self.pairs[start..end] {
            if let Some(r) = self.chunks.get(ci as usize) {
                materialize_resid(&self.raw, r, res_id, &mut out);
            }
        }
        // Pré-resolução de strings (idêntica ao eager): STRING simples →
        // pool global; STRING de bag → pool global por item.
        for e in out.iter_mut() {
            if e.data_type == 3 && !e.complex {
                e.string = self.global.get(e.data).map(str::to_owned);
            }
            if let Some(bag) = e.bag.as_mut() {
                for v in bag.iter_mut() {
                    if v.data_type == 3 {
                        v.string = self.global.get(v.data).map(str::to_owned);
                    }
                }
            }
        }
        let arc = Arc::new(out);
        if let Ok(mut guard) = cache.lock() {
            if guard.len() >= ENTRY_CACHE_CAP {
                guard.clear(); // bounded: reset determinístico (só re-decodifica)
            }
            guard.insert(res_id, Arc::clone(&arc));
        }
        Some(arc)
    }

    /// Todos os resids conhecidos do pacote (índice + cache sintético),
    /// ordenados e sem duplicatas — equivalente ao `entries.keys()` do eager.
    /// Consumidores tipo `getIdentifier` iteram aqui e chamam `resolve_name`.
    pub fn resource_ids(&self) -> Vec<u32> {
        // pairs já está ordenado por resid — unique preservando a ordem;
        // cache sintético (testes) contribui com chaves extras, re-ordenadas.
        let mut ids: Vec<u32> = Vec::with_capacity(self.pairs.len());
        for &(r, _) in self.pairs.iter() {
            if ids.last() != Some(&r) {
                ids.push(r);
            }
        }
        if let Ok(guard) = self.cache.lock() {
            ids.extend(guard.keys().copied());
        }
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    /// Snapshot materializado de TODAS as entradas (resid, entradas),
    /// ordenado por resid — ferramentas de DIAGNÓSTICO (ex.: example
    /// verify_config_apilevel). Custo ≈ eager de hoje; nunca usar em caminho
    /// quente (materializa o pacote inteiro, inclusive bags/strings).
    pub fn entries_snapshot(&self) -> Vec<(u32, Vec<ResEntry>)> {
        self.resource_ids()
            .into_iter()
            .map(|id| {
                let es = self
                    .entries_for(id)
                    .map(|a| (*a).clone())
                    .unwrap_or_default();
                (id, es)
            })
            .collect()
    }

    /// Nome qualificado `@type/key` de um resid (sem pacote).
    pub fn resolve_name(&self, res_id: u32) -> Option<String> {
        let entries = self.entries_for(res_id)?;
        let key = entries.first()?;
        let type_idx = ((res_id >> 16) & 0xFF) as usize; // 1-based
        let type_name = self.type_strings.get((type_idx as u32).wrapping_sub(1))?;
        let key_name = self.key_strings.get(key.key_idx)?;
        Some(format!("@{type_name}/{key_name}"))
    }
}

/// issue #61: valor TIPADO de uma entrada de recurso (melhor config) — o que
/// o LayoutInflater precisa para resolver `@dimen`/`@bool` por referência.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedValue {
    Str(String),
    Boolean(bool),
    Integer(i32),
    Float(f32),
    /// Valor complexo (mantissa×radix) com unidade — px/dp/sp/…
    Dimension {
        value: f32,
        unit: ComplexUnit,
    },
    /// Referência a outro recurso (resid) — o caller decide se segue a cadeia.
    Reference(u32),
    /// Referência a um atributo de tema (`?attr/...`) inside a style bag.
    Attribute(u32),
}

/// complexToFloat (TypedValue) — mesma matemática do axml (mantissa bits
/// 8–31 com sinal no bit 31, radix bits 4–5).
fn complex_to_float(data: u32) -> f32 {
    const RADIX_MULTS: [f32; 4] = [
        0.003_906_25,   // radix 0: 2⁻⁸
        3.051_758e-5,   // radix 1: 2⁻¹⁵
        1.192_092_9e-7, // radix 2: 2⁻²³
        4.656_613e-10,  // radix 3: 2⁻³¹
    ];
    let mantissa = (data & 0xFFFF_FF00) as i32;
    mantissa as f32 * RADIX_MULTS[((data >> 4) & 0x3) as usize]
}

/// Unidade de um Res_value complexo (bits 0–3 — mesma tabela do axml).
fn complex_unit_of(data: u32) -> ComplexUnit {
    match (data & 0xF) as u8 {
        0 => ComplexUnit::Px,
        1 => ComplexUnit::Dip,
        2 => ComplexUnit::Sp,
        3 => ComplexUnit::Pt,
        4 => ComplexUnit::In,
        5 => ComplexUnit::Mm,
        other => ComplexUnit::Unknown(other),
    }
}

impl Arsc {
    /// Parse a partir de bytes emprestados (copia 1× para o `Arc` interno).
    pub fn parse(data: &[u8]) -> RdResult<Arsc> {
        Self::parse_inner(Arc::new(data.to_vec()))
    }

    /// Parse tomando posse do buffer (zero cópias — caminho do `Apk::open`,
    /// que já possui o `Vec<u8>` inflado do ZIP; ADR 008: RSS conta).
    pub fn parse_owned(data: Vec<u8>) -> RdResult<Arsc> {
        Self::parse_inner(Arc::new(data))
    }

    fn parse_inner(raw: Arc<Vec<u8>>) -> RdResult<Arsc> {
        let data: &[u8] = &raw;
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
        // ADR 014: o buffer (limitado ao file_size declarado) é o único Arc
        // compartilhado por pools e pacotes — Lei 2, clones baratos, zero
        // cópias por pool/pacote. Truncar o Arc seria copy-on-write caro;
        // guards de leitura usam raw.len() com offsets absolutos validados
        // pelo file_size abaixo (chunks além do limite rejeitados como hoje).
        let data = &data[..file_size.min(data.len())];
        let package_count = u32_at(data, 8)?;

        // pool global logo após o header (12 bytes)
        let (global_strings, pool_used) = StringPool::parse_shared(&raw, 12)?;
        let mut p = 12 + pool_used as usize;

        let mut packages = Vec::new();
        let mut seen_packages = 0u32;
        let limit = file_size.min(raw.len());
        while p + 8 <= limit && seen_packages < package_count {
            let ctype = u16_at(&raw, p)?;
            let cheader = u16_at(&raw, p + 2)? as usize;
            let csize = u32_at(&raw, p + 4)? as usize;
            // progresso garantido: chunk mínimo = 8 bytes (mesmo guard dos
            // irmãos em axml.rs e no loop interno de parse_package — sem ele,
            // csize=0 com packageCount≥1 gira eternamente; issue #9)
            if csize < 8 {
                return Err(RdError::parse(format!(
                    "arsc: chunk at {p} declares size {csize} < 8 (no progress possible)"
                )));
            }
            if csize < cheader || p + csize > limit {
                return Err(RdError::parse(format!(
                    "arsc: package chunk at {p} declares {csize} beyond bounds"
                )));
            }
            if ctype == RES_TABLE_PACKAGE_TYPE {
                let pkg_header_size = u16_at(&raw, p + 2)? as usize;
                packages.push(parse_package(
                    &raw,
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
        let entries = pkg.entries_for(res_id)?;
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

    /// issue #61: valor TIPADO da melhor config de um resid. `None` = resid
    /// desconhecido ou tipo não representável (bag/complex — o caller falha
    /// tipado; nunca default silencioso).
    pub fn resolve_value(&self, res_id: u32) -> Option<ResolvedValue> {
        let pkg = self.package_for(res_id)?;
        let entries = pkg.entries_for(res_id)?;
        let best = entries
            .iter()
            .min_by_key(|e| (e.config.specificity(), !e.config.is_default()))?;
        match best.data_type {
            0x03 => best.string.clone().map(ResolvedValue::Str),
            0x01 => Some(ResolvedValue::Reference(best.data)),
            0x02 => Some(ResolvedValue::Attribute(best.data)),
            // INT_DEC (u32 com semântica i32 — TypedValue.data é cru)
            0x10 => Some(ResolvedValue::Integer(best.data as i32)),
            0x11 => Some(ResolvedValue::Integer(best.data as i32)),
            0x12 => Some(ResolvedValue::Boolean(best.data != 0)),
            // INT_COLOR_ARGB8/RGB8/ARGB4/RGB4: aapt2 stores the expanded ARGB
            0x1c..=0x1f => Some(ResolvedValue::Integer(best.data as i32)),
            0x04 => Some(ResolvedValue::Float(f32::from_bits(best.data))),
            0x05 => Some(ResolvedValue::Dimension {
                value: complex_to_float(best.data),
                unit: complex_unit_of(best.data),
            }),
            _ => None,
        }
    }

    /// Resolve one attribute from a style, including its parent style and
    /// `?attr/…` aliases. Resource references remain explicit so the VM can
    /// dereference them using the APK's normal resource policy.
    pub fn resolve_style_attribute(&self, style_id: u32, attr_id: u32) -> Option<ResolvedValue> {
        self.resolve_style_attribute_inner(style_id, attr_id, 0)
    }

    fn resolve_style_attribute_inner(
        &self,
        style_id: u32,
        attr_id: u32,
        depth: usize,
    ) -> Option<ResolvedValue> {
        if depth >= 64 {
            return None;
        }
        let pkg = self.package_for(style_id)?;
        let entries = pkg.entries_for(style_id)?;
        let style = entries
            .iter()
            .min_by_key(|e| (e.config.specificity(), !e.config.is_default()))?;
        if let Some(value) = style
            .bag
            .as_ref()
            .and_then(|bag| bag.iter().find(|value| value.name == attr_id))
        {
            let resolved = match value.data_type {
                0x01 => ResolvedValue::Reference(value.data),
                0x02 => ResolvedValue::Attribute(value.data),
                0x03 => ResolvedValue::Str(value.string.clone()?),
                0x04 => ResolvedValue::Float(f32::from_bits(value.data)),
                0x05 => ResolvedValue::Dimension {
                    value: complex_to_float(value.data),
                    unit: complex_unit_of(value.data),
                },
                0x10 | 0x11 | 0x1c..=0x1f => ResolvedValue::Integer(value.data as i32),
                0x12 => ResolvedValue::Boolean(value.data != 0),
                _ => return None,
            };
            return match resolved {
                ResolvedValue::Attribute(next) => {
                    self.resolve_style_attribute_inner(style_id, next, depth + 1)
                }
                other => Some(other),
            };
        }
        let parent = style.parent?;
        self.resolve_style_attribute_inner(parent, attr_id, depth + 1)
    }

    pub fn package_for(&self, res_id: u32) -> Option<&Package> {
        let pid = ((res_id >> 24) & 0xFF) as u8;
        // issue #54: SEM fallback para o primeiro package — um arsc hostil
        // não pode nomear ids de package estranho (ex.: 0x01 = framework
        // android:id/…). Package id inexistente → resolução honesta falha.
        self.packages.iter().find(|p| p.id == pid)
    }
}

fn parse_package(
    raw: &Arc<Vec<u8>>,
    base: usize,
    size: usize,
    header_size: usize,
    global: &StringPool,
) -> RdResult<Package> {
    let data: &[u8] = raw;
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

    let (type_strings, _) = StringPool::parse_shared(raw, base + type_strings_off)?;
    let (key_strings, _) = StringPool::parse_shared(raw, base + key_strings_off)?;

    // ADR 014: índice leve em vez de materialização — o walk é o MESMO do
    // eager (mesmos guards/breaks); por chunk de tipo guardamos a referência
    // em `chunks` e os resids cobertos viram pares (resid, chunk_idx) em
    // `pairs` (append O(1); sort único no fim; busca binária por resid).
    let mut chunks: Vec<TypeChunkRef> = Vec::new();
    let mut pairs: Vec<(u32, u32)> = Vec::new();
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

            // issue #29: ResTable_type tem no mínimo 20 bytes de header
            // (8 do ResChunk_header + id/res0/res1 + entryCount + entriesStart);
            // cheader < 20 causava underflow `cheader - 20` (pânico em
            // overflow-checks, config garbage em release)
            if cheader < 20 {
                break; // chunk malformado: não fatal — o que veio antes fica
            }
            let config = parse_config(data, p + 20, cheader - 20)?;

            let chunk_idx = chunks.len() as u32;
            chunks.push(TypeChunkRef {
                chunk_off: p,
                chunk_size: csize,
                header_size: cheader,
                config,
                entry_count,
                flags,
                entries_start,
            });

            if flags & FLAG_SPARSE != 0 {
                // ResTable_sparseTypeEntry: {idx u16, offset u16} — offset/4
                let sparse_pairs = entry_count;
                for i in 0..sparse_pairs {
                    let o = p + cheader + i * 4;
                    if o + 4 > data.len() {
                        break;
                    }
                    let idx = u16_at(data, o)? as usize;
                    let off4 = u16_at(data, o + 2)? as u32;
                    if off4 == 0xFFFF {
                        continue; // sem entrada (equivalente ao 0xFFFFFFFF do coletor)
                    }
                    let resid =
                        ((pkg_id as u32) << 24) | ((type_id as u32) << 16) | (idx as u32 & 0xFFFF);
                    pairs.push((resid, chunk_idx));
                }
            } else if flags & FLAG_OFFSET16 != 0 {
                for i in 0..entry_count {
                    let o = p + cheader + i * 2;
                    if o + 2 > data.len() {
                        break;
                    }
                    let off = u16_at(data, o)? as u32;
                    if off == 0xFFFF {
                        continue;
                    }
                    let resid =
                        ((pkg_id as u32) << 24) | ((type_id as u32) << 16) | (i as u32 & 0xFFFF);
                    pairs.push((resid, chunk_idx));
                }
            } else {
                for i in 0..entry_count {
                    let o = p + cheader + i * 4;
                    if o + 4 > data.len() {
                        break;
                    }
                    let off = u32_at(data, o)?;
                    if off == 0xFFFF_FFFF {
                        continue;
                    }
                    let resid =
                        ((pkg_id as u32) << 24) | ((type_id as u32) << 16) | (i as u32 & 0xFFFF);
                    pairs.push((resid, chunk_idx));
                }
            }
        }
        p += csize;
    }

    // ordem determinística (Lei 4): (resid, chunk_idx) — chunk_idx é a ordem
    // de walk, então o agrupamento por resid reproduz exatamente a ordem em
    // que o eager empilhava as entradas (tie-breaking do min_by_key igual).
    pairs.sort_unstable();

    Ok(Package {
        id: pkg_id,
        name,
        type_strings,
        key_strings,
        global: global.clone(),
        chunks: Arc::new(chunks),
        pairs: Arc::new(pairs),
        raw: Arc::clone(raw),
        cache: Mutex::new(BTreeMap::new()),
    })
}

/// Materializa as entradas de `resid` presentes no chunk `r` (ADR 014).
/// Porta 1:1 do coletor eager: mesmos endereços absolutos, mesmos guards
/// (skip silencioso de entrada truncada), mesma ordem de push.
fn materialize_resid(raw: &Arc<Vec<u8>>, r: &TypeChunkRef, resid: u32, out: &mut Vec<ResEntry>) {
    let data: &[u8] = raw;
    let p = r.chunk_off;
    let csize = r.chunk_size;
    let cheader = r.header_size;
    let entries_abs = p + r.entries_start;
    let idx = (resid & 0xFFFF) as usize;

    let mut collect = |rel_off: u32| {
        if rel_off == 0xFFFF_FFFF {
            return;
        }
        let eoff = entries_abs + rel_off as usize;
        if eoff + 8 > data.len() {
            return; // entrada truncada: skip
        }
        let e_size = u16_at(data, eoff).unwrap_or(0) as usize;
        let e_flags = u16_at(data, eoff + 2).unwrap_or(0);
        let key_idx = u32_at(data, eoff + 4).unwrap_or(0);
        let (data_type, ddata, complex, parent, bag) = if e_flags & ENTRY_FLAG_COMPLEX != 0 {
            let parent = if e_size >= 16 && eoff + 16 <= p + csize {
                let parent = u32_at(data, eoff + 8).unwrap_or(0);
                (parent != 0).then_some(parent)
            } else {
                None
            };
            let map_count = if e_size >= 16 && eoff + 16 <= p + csize {
                u32_at(data, eoff + 12).unwrap_or(0) as usize
            } else {
                usize::MAX
            };
            let maps_start = eoff.saturating_add(e_size);
            let available = (p + csize).saturating_sub(maps_start) / 12;
            let bag = if map_count <= 65_536 && map_count <= available {
                let mut bag = Vec::with_capacity(map_count);
                for i in 0..map_count {
                    let m = maps_start + i * 12;
                    let value_size = u16_at(data, m + 4).unwrap_or(0);
                    if value_size < 8 {
                        bag.clear();
                        break;
                    }
                    let name = u32_at(data, m).unwrap_or(0);
                    let dtype = *data.get(m + 7).unwrap_or(&0);
                    let value = u32_at(data, m + 8).unwrap_or(0);
                    bag.push(ResMapEntry {
                        name,
                        data_type: dtype,
                        data: value,
                        string: None, // resolvido abaixo (pool global não existe aqui)
                    });
                }
                (bag.len() == map_count).then_some(bag)
            } else {
                None
            };
            (0u8, 0u32, true, parent, bag)
        } else {
            // Res_value após ResTable_entry (e_size bytes)
            let Some(v) = eoff.checked_add(e_size) else {
                return;
            };
            if v + 8 > p + csize || v + 8 > data.len() {
                return;
            }
            let dtype = *data.get(v + 3).unwrap_or(&0);
            let d = u32_at(data, v + 4).unwrap_or(0);
            (dtype, d, false, None, None)
        };
        out.push(ResEntry {
            config: r.config.clone(),
            data_type,
            data: ddata,
            string: None, // pré-resolvido em entries_for (precisa da pool global)
            complex,
            parent,
            bag,
            key_idx,
        });
    };

    if r.flags & FLAG_SPARSE != 0 {
        // sparse: TODAS as entradas cujo idx casa (o eager processava cada par)
        let pairs = r.entry_count;
        for i in 0..pairs {
            let o = p + cheader + i * 4;
            if o + 4 > data.len() {
                break;
            }
            let eidx = u16_at(data, o).unwrap_or(0) as usize;
            let off4 = u16_at(data, o + 2).unwrap_or(0xFFFF) as u32;
            if off4 == 0xFFFF {
                continue;
            }
            if eidx == idx {
                collect(off4 * 4);
            }
        }
    } else if r.flags & FLAG_OFFSET16 != 0 {
        let o = p + cheader + idx * 2;
        if o + 2 <= data.len() {
            let off = u16_at(data, o).unwrap_or(0xFFFF) as u32;
            if off != 0xFFFF {
                collect(off * 4);
            }
        }
    } else {
        let o = p + cheader + idx * 4;
        if o + 4 <= data.len() {
            collect(u32_at(data, o).unwrap_or(0xFFFF_FFFF));
        }
    }
}

/// ResTable_config — extrai locale/densidade/api sem reclamar de variações de tamanho.
///
/// Layout do `ResTable_config` (frameworks/base/libs/androidfw/include/androidfw/
/// ResourceTypes.h) — offsets conferidos campo a campo:
///
/// ```text
/// size:u32@0 | mcc:u16@4 mnc:u16@6 | language[2]@8 country[2]@10
/// orientation:u8@12 touchscreen:u8@13 density:u16@14
/// keyboard:u8@16 navigation:u8@17 inputFlags:u8@18 pad0:u8@19
/// screenWidth:u16@20 screenHeight:u16@22
/// sdkVersion:u16@24 minorVersion:u16@26
/// screenLayout:u8@28 uiMode:u8@29 smallestScreenWidthDp:u16@30 (se size ≥ 32)
/// ```
///
/// `api_level` é o `sdkVersion` (u16 @24 — o mesmo campo que dá nome às configs
/// "v26" do aapt2). NÃO é `screenHeight` (u16 @22); o bloco [20..24) é o par
/// screenWidth/screenHeight de 2+2 bytes, não dois u32 (verificado empiricamente
/// contra resources.arsc reais — configs "vNN" decodificam com o valor da API).
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
    if readable >= 26 {
        // sdkVersion:u16 @24 (minorVersion:u16 @26 — não lido)
        cfg.api_level = Some(u16_at(data, cfg_base + 24)?);
    }
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_global_pool() -> Vec<u8> {
        // ResStringPool vazio válido: 28 bytes de header, 0 strings
        let mut p = Vec::new();
        p.extend_from_slice(&0x0001u16.to_le_bytes()); // type
        p.extend_from_slice(&28u16.to_le_bytes()); // headerSize
        p.extend_from_slice(&28u32.to_le_bytes()); // size
        p.extend_from_slice(&0u32.to_le_bytes()); // stringCount
        p.extend_from_slice(&0u32.to_le_bytes()); // styleCount
        p.extend_from_slice(&0u32.to_le_bytes()); // flags (UTF-16)
        p.extend_from_slice(&28u32.to_le_bytes()); // stringsStart
        p.extend_from_slice(&0u32.to_le_bytes()); // stylesStart
        p
    }

    /// PoC da issue #9: 48 bytes = header ResTable (packageCount=1) + pool
    /// global vazia + chunk {type=0x0202, headerSize=0, size=0}. Sem o guard
    /// `csize < 8` o loop externo não progride (`p += 0`) e trava para sempre
    /// (hang, não crash). Deve retornar erro tipado em tempo finito.
    #[test]
    fn zero_size_chunk_is_typed_error_not_hang() {
        let mut d = Vec::new();
        d.extend_from_slice(&0x0002u16.to_le_bytes()); // RES_TABLE_TYPE
        d.extend_from_slice(&12u16.to_le_bytes()); // headerSize
        d.extend_from_slice(&48u32.to_le_bytes()); // file size
        d.extend_from_slice(&1u32.to_le_bytes()); // packageCount = 1
        assert_eq!(d.len(), 12);
        d.extend(empty_global_pool());
        assert_eq!(d.len(), 40);
        d.extend_from_slice(&0x0202u16.to_le_bytes()); // tipo ≠ package
        d.extend_from_slice(&0u16.to_le_bytes()); // headerSize = 0
        d.extend_from_slice(&0u32.to_le_bytes()); // size = 0 → sem progresso
        assert_eq!(d.len(), 48);

        let e = Arsc::parse(&d).expect_err("csize=0 deve ser rejeitado");
        assert_eq!(e.code, "PARSE_ERROR");
        assert!(e.cause.contains("< 8"), "mensagem: {}", e.cause);
    }

    /// Issue #16: `api_level` é o `sdkVersion` u16 @24 do ResTable_config —
    /// NÃO é screenHeight (@22). screenHeight=1440 (0x5A0) NÃO pode vazar
    /// para api_level; sdkVersion=26 deve decodificar como 26 (config "v26").
    #[test]
    fn config_api_level_is_sdk_version_u16_at_offset_24() {
        let mut cfg = vec![0u8; 28];
        cfg[0..4].copy_from_slice(&28u32.to_le_bytes()); // size
        cfg[8..10].copy_from_slice(b"en"); // language
        cfg[10..12].copy_from_slice(b"US"); // country
        cfg[14..16].copy_from_slice(&256u16.to_le_bytes()); // density
        cfg[20..22].copy_from_slice(&720u16.to_le_bytes()); // screenWidth
        cfg[22..24].copy_from_slice(&1440u16.to_le_bytes()); // screenHeight
        cfg[24..26].copy_from_slice(&26u16.to_le_bytes()); // sdkVersion

        let info = parse_config(&cfg, 0, cfg.len()).expect("config");
        assert_eq!(
            info.api_level,
            Some(26),
            "sdkVersion@24, não screenHeight@22"
        );
        assert_eq!(info.density, Some(256));
        assert_eq!(info.language.as_deref(), Some("en"));
        assert_eq!(info.country.as_deref(), Some("US"));
    }

    #[test]
    fn style_attributes_follow_parent_and_theme_attribute_aliases() {
        let parent = 0x7f12_0001;
        let child = 0x7f12_0002;
        let primary = 0x7f04_0001;
        let alias = 0x7f04_0002;
        let inherited = 0x7f04_0003;
        let absent = 0x7f04_0004;
        let entry = |parent, bag| ResEntry {
            config: ResConfigInfo::default(),
            data_type: 0,
            data: 0,
            string: None,
            complex: true,
            parent,
            bag: Some(bag),
            key_idx: 0,
        };
        let mut entries = BTreeMap::new();
        entries.insert(
            parent,
            vec![entry(
                None,
                vec![
                    ResMapEntry {
                        name: primary,
                        data_type: 0x12,
                        data: 1,
                        string: None,
                    },
                    ResMapEntry {
                        name: inherited,
                        data_type: 0x12,
                        data: 1,
                        string: None,
                    },
                ],
            )],
        );
        entries.insert(
            child,
            vec![entry(
                Some(parent),
                vec![
                    ResMapEntry {
                        name: primary,
                        data_type: 0x12,
                        data: 0,
                        string: None,
                    },
                    ResMapEntry {
                        name: alias,
                        data_type: 0x02,
                        data: primary,
                        string: None,
                    },
                ],
            )],
        );
        let arsc = Arsc {
            global_strings: StringPool::default(),
            packages: vec![Package::synthetic(
                0x7f,
                "test".into(),
                StringPool::default(),
                StringPool::default(),
                entries,
            )],
        };
        assert_eq!(
            arsc.resolve_style_attribute(child, primary),
            Some(ResolvedValue::Boolean(false)),
            "the child style overrides its parent"
        );
        assert_eq!(
            arsc.resolve_style_attribute(child, alias),
            Some(ResolvedValue::Boolean(false)),
            "?attr alias resolves through the child's effective theme"
        );
        assert_eq!(
            arsc.resolve_style_attribute(child, inherited),
            Some(ResolvedValue::Boolean(true)),
            "a missing child attribute is inherited from the parent"
        );
        assert_eq!(
            arsc.resolve_style_attribute(child, absent),
            None,
            "absent attributes stay absent"
        );
    }

    /// ADR 014: materialização sob demanda produz os mesmos resolvables do
    /// eager — aqui um ARSC mínimo com 1 pacote, 1 chunk de tipo, 1 string.
    #[test]
    fn lazy_index_resolves_like_eager() {
        // constrói via ArscFix-like bytes: header + pool global ["Oi"] +
        // pacote com type_strings ["string"], key_strings ["hello"],
        // 1 chunk ResTable_type com 1 entrada STRING apontando p/ "Oi".
        let mut d = Vec::new();
        // ResTable header (12) — packageCount=1 (patched no fim)
        d.extend_from_slice(&0x0002u16.to_le_bytes());
        d.extend_from_slice(&12u16.to_le_bytes());
        d.extend_from_slice(&0u32.to_le_bytes()); // size (patch)
        d.extend_from_slice(&1u32.to_le_bytes());
        // pool global UTF-16: ["Oi"] — header 28 + 1 offset + 6 bytes
        let mut g = Vec::new();
        g.extend_from_slice(&0x0001u16.to_le_bytes());
        g.extend_from_slice(&28u16.to_le_bytes());
        g.extend_from_slice(&0u32.to_le_bytes()); // size (patch)
        g.extend_from_slice(&1u32.to_le_bytes()); // count
        g.extend_from_slice(&0u32.to_le_bytes()); // styleCount
        g.extend_from_slice(&0u32.to_le_bytes()); // flags = UTF-16
        g.extend_from_slice(&32u32.to_le_bytes()); // stringsStart
        g.extend_from_slice(&0u32.to_le_bytes()); // stylesStart
        g.extend_from_slice(&0u32.to_le_bytes()); // offset da string 0
        g.extend_from_slice(&2u16.to_le_bytes()); // len 2 ("Oi")
        g.extend_from_slice(&"O".encode_utf16().next().unwrap().to_le_bytes());
        g.extend_from_slice(&"i".encode_utf16().next().unwrap().to_le_bytes());
        g.extend_from_slice(&0u16.to_le_bytes()); // terminador
        let gs = g.len() as u32;
        g[4..8].copy_from_slice(&gs.to_le_bytes());
        d.extend(g);

        // pacote: header 288 (headerSize) + typeStrings/keyStrings + 1 chunk tipo
        let pkg_header_size: usize = 288;
        let mut type_pool = Vec::new();
        type_pool.extend_from_slice(&0x0001u16.to_le_bytes());
        type_pool.extend_from_slice(&28u16.to_le_bytes());
        type_pool.extend_from_slice(&0u32.to_le_bytes());
        type_pool.extend_from_slice(&1u32.to_le_bytes());
        type_pool.extend_from_slice(&0u32.to_le_bytes());
        type_pool.extend_from_slice(&0u32.to_le_bytes());
        type_pool.extend_from_slice(&32u32.to_le_bytes());
        type_pool.extend_from_slice(&0u32.to_le_bytes());
        type_pool.extend_from_slice(&0u32.to_le_bytes());
        type_pool.extend_from_slice(&6u16.to_le_bytes()); // 's','t' via chars
        for ch in "string".encode_utf16() {
            type_pool.extend_from_slice(&ch.to_le_bytes());
        }
        type_pool.extend_from_slice(&0u16.to_le_bytes());
        let ts = type_pool.len() as u32;
        type_pool[4..8].copy_from_slice(&ts.to_le_bytes());
        // (key pool = mesmo formato, 1 string "hello")
        let mut key_pool = Vec::new();
        key_pool.extend_from_slice(&0x0001u16.to_le_bytes());
        key_pool.extend_from_slice(&28u16.to_le_bytes());
        key_pool.extend_from_slice(&0u32.to_le_bytes());
        key_pool.extend_from_slice(&1u32.to_le_bytes());
        key_pool.extend_from_slice(&0u32.to_le_bytes());
        key_pool.extend_from_slice(&0u32.to_le_bytes());
        key_pool.extend_from_slice(&32u32.to_le_bytes());
        key_pool.extend_from_slice(&0u32.to_le_bytes());
        key_pool.extend_from_slice(&0u32.to_le_bytes());
        key_pool.extend_from_slice(&5u16.to_le_bytes()); // len 5
        for ch in "hello".encode_utf16() {
            key_pool.extend_from_slice(&ch.to_le_bytes());
        }
        key_pool.extend_from_slice(&0u16.to_le_bytes());
        let ks = key_pool.len() as u32;
        key_pool[4..8].copy_from_slice(&ks.to_le_bytes());

        let type_strings_off = pkg_header_size; // logo após o header do pacote
        let key_strings_off = type_strings_off + type_pool.len();

        // chunk ResTable_type: header 20 + config 8 + offsets 4 + entry 8 + value 8
        // ResTable_type = header(8) + id@8 + res0@9 + res1@10 + reserved@11 +
        // entryCount@12 + entriesStart@16 (+ config após os 20 bytes de header)
        let entries_start_off: u32 = 20 + 8 + 4; // relativo ao início do chunk
        let mut tc = Vec::new();
        tc.extend_from_slice(&0x0201u16.to_le_bytes()); // type
        tc.extend_from_slice(&28u16.to_le_bytes()); // headerSize (20 header + config 8)
        tc.extend_from_slice(&0u32.to_le_bytes()); // size (patch)
        tc.push(1u8); // id@8
        tc.push(0u8); // res0@9
        tc.push(0u8); // res1@10
        tc.push(0u8); // reserved@11
        tc.extend_from_slice(&1u32.to_le_bytes()); // entryCount@12
        tc.extend_from_slice(&entries_start_off.to_le_bytes()); // entriesStart@16
        tc.extend_from_slice(&8u32.to_le_bytes()); // config size = 8 (vazia)
        tc.extend_from_slice(&[0u8; 4]); // config: 4 bytes restantes (screenConfig mínimo)
                                         // offsets (1 entrada, rel a entriesStart): 0
        tc.extend_from_slice(&0u32.to_le_bytes());
        // ResTable_entry: size 8, flags 0, key_idx 0
        tc.extend_from_slice(&8u16.to_le_bytes()); // entry size
        tc.extend_from_slice(&0u16.to_le_bytes()); // flags = simples
        tc.extend_from_slice(&0u32.to_le_bytes()); // key idx (na key pool)
                                                   // Res_value: size 8, res0 0, dataType 0x03 (STRING), data = índice 0 na global
        tc.extend_from_slice(&8u16.to_le_bytes()); // value size
        tc.push(0u8); // res0
        tc.push(0x03u8); // dataType STRING
        tc.extend_from_slice(&0u32.to_le_bytes()); // data = 0 (global idx 0)
        let tcs = tc.len() as u32;
        tc[4..8].copy_from_slice(&tcs.to_le_bytes());

        let pkg_size = pkg_header_size + type_pool.len() + key_pool.len() + tc.len();
        let mut pkg = Vec::new();
        pkg.extend_from_slice(&0x0200u16.to_le_bytes()); // RES_TABLE_PACKAGE_TYPE
        pkg.extend_from_slice(&(pkg_header_size as u16).to_le_bytes());
        pkg.extend_from_slice(&(pkg_size as u32).to_le_bytes()); // size (patch se precisar)
        pkg.extend_from_slice(&0x7fu32.to_le_bytes()); // id@8 = 0x7f → [8..12)
                                                       // name: 128 UTF-16 units @12 — "t\0..." (termina em \0) → [12..268)
        let mut name = vec![0u16; 128];
        name[0] = 't' as u16;
        for u in name {
            pkg.extend_from_slice(&u.to_le_bytes());
        }
        // [12..268) = name; typeStrings@268, lastPublicType@272,
        // keyStrings@276, lastPublicKey@280; header termina em 288.
        while pkg.len() < 268 {
            pkg.push(0);
        }
        pkg.extend_from_slice(&(type_strings_off as u32).to_le_bytes()); // typeStrings @268
        pkg.extend_from_slice(&1u32.to_le_bytes()); // lastPublicType @272
        pkg.extend_from_slice(&(key_strings_off as u32).to_le_bytes()); // keyStrings @276
        pkg.extend_from_slice(&1u32.to_le_bytes()); // lastPublicKey @280
                                                    // header 288: resto zero até 288
        while pkg.len() < 288 {
            pkg.push(0);
        }
        pkg.extend(type_pool);
        pkg.extend(key_pool);
        pkg.extend(tc);

        d.extend(pkg);
        let total = d.len() as u32;
        d[4..8].copy_from_slice(&total.to_le_bytes());

        let arsc = Arsc::parse(&d).expect("arsc válido");
        assert_eq!(arsc.packages.len(), 1);
        let resid = 0x7f01_0000u32; // pkg 0x7f, tipo 1, idx 0
        assert_eq!(arsc.resolve_string(resid).as_deref(), Some("Oi"));
        assert_eq!(arsc.resolve_name(resid).as_deref(), Some("@string/hello"));
        // resolve_value pela melhor config (default)
        match arsc.resolve_value(resid) {
            Some(ResolvedValue::Str(s)) => assert_eq!(s, "Oi"),
            other => panic!("esperava Str(\"Oi\"), veio {other:?}"),
        }
        // resid desconhecido → None honesto
        assert_eq!(arsc.resolve_string(0x7f01_0001), None);
    }
}
