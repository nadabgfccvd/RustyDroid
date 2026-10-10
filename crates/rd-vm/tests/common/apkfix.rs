//! Fixtures de APK sintético para os testes do M3.2 (LayoutInflater):
//! ZIP (STORED) + string pool UTF-16 + AXML + ARSC mínimos-corretos,
//! gerados byte a byte — mesma política do `DexBuilder` (nenhum fixture
//! binário commitado, tudo determinístico e auditável).

// ────────────────────────────────────────────────────────────────────────────
// CRC-32 (IEEE, refletido) — o Zip do rd-apk verifica CRC em toda leitura.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

pub fn pad4(out: &mut Vec<u8>) {
    while out.len() % 4 != 0 {
        out.push(0);
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ResStringPool UTF-16 (flags = 0) — formato confirmado contra o parser
// (`pool.rs`) e o builder de teste do axml.rs.
pub fn string_pool_utf16(strs: &[&str]) -> Vec<u8> {
    fn utf16_unit(s: &str) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend((s.encode_utf16().count() as u16).to_le_bytes());
        for u in s.encode_utf16() {
            v.extend(u.to_le_bytes());
        }
        v.extend(0u16.to_le_bytes());
        v
    }
    let mut pool = Vec::new();
    pool.extend(0x0001u16.to_le_bytes()); // RES_STRING_POOL_TYPE
    pool.extend(28u16.to_le_bytes()); // headerSize
    pool.extend(0u32.to_le_bytes()); // size (patched)
    pool.extend((strs.len() as u32).to_le_bytes());
    pool.extend(0u32.to_le_bytes()); // styleCount
    pool.extend(0u32.to_le_bytes()); // flags (UTF-16)
    let strings_start = 28 + strs.len() * 4;
    pool.extend((strings_start as u32).to_le_bytes());
    pool.extend(0u32.to_le_bytes()); // stylesStart
    let mut offs = Vec::new();
    let mut body = Vec::new();
    for s in strs {
        offs.push(body.len() as u32);
        body.extend(utf16_unit(s));
    }
    pool.extend(offs.iter().flat_map(|o| o.to_le_bytes()));
    pool.extend(&body);
    pad4(&mut pool);
    let size = pool.len() as u32;
    pool[4..8].copy_from_slice(&size.to_le_bytes());
    pool
}

// ────────────────────────────────────────────────────────────────────────────
// AXML (XML binário) — subset de emissão usado pelos layouts/manifest de teste.

pub const NO_INDEX: u32 = 0xFFFF_FFFF;
pub const NS_ANDROID: &str = "http://schemas.android.com/apk/res/android";

pub const TYPE_REFERENCE: u8 = 0x01;
pub const TYPE_STRING: u8 = 0x03;
pub const TYPE_DIMENSION: u8 = 0x05;
pub const TYPE_FIRST_INT: u8 = 0x10;
pub const TYPE_BOOLEAN: u8 = 0x12;

pub enum AxVal {
    /// string com raw no pool (tipo STRING)
    Str(String),
    /// referência de recurso (ex.: @+id/tv → 0x7f030000)
    Ref(u32),
    Int(i32),
    Bool(bool),
    /// dimensão crua Res_value (mantissa<<8 | unit) — ex.: (80<<8)|1 = 80dp
    Dim {
        raw: u32,
    },
}

pub struct AxAttr {
    pub android: bool,
    pub name: String,
    pub val: AxVal,
}

pub fn a(android: bool, name: &str, val: AxVal) -> AxAttr {
    AxAttr {
        android,
        name: name.to_string(),
        val,
    }
}

#[derive(Default)]
pub struct AxNode {
    pub name: String,
    pub attrs: Vec<AxAttr>,
    pub children: Vec<AxNode>,
}

impl AxNode {
    pub fn new(name: &str, attrs: Vec<AxAttr>, children: Vec<AxNode>) -> Self {
        AxNode {
            name: name.to_string(),
            attrs,
            children,
        }
    }
}

fn idx_of(strs: &mut Vec<String>, s: &str) -> u32 {
    if let Some(i) = strs.iter().position(|x| x == s) {
        return i as u32;
    }
    strs.push(s.to_string());
    (strs.len() - 1) as u32
}

fn collect_strings(node: &AxNode, strs: &mut Vec<String>) {
    idx_of(strs, &node.name);
    for at in &node.attrs {
        if at.android {
            // uri + prefix (a URI fica no slot 0 — os attrs android apontam p/ ela)
            if !strs.contains(&NS_ANDROID.to_string()) {
                strs.insert(0, NS_ANDROID.to_string());
                strs.insert(1, "android".to_string());
            }
        }
        idx_of(strs, &at.name);
        if let AxVal::Str(s) = &at.val {
            idx_of(strs, s);
        }
    }
    for c in &node.children {
        collect_strings(c, strs);
    }
}

fn emit_node(node: &AxNode, strs: &[String], out: &mut Vec<u8>) {
    // todas as strings já foram internadas por collect_strings
    let ix =
        |s: &str| -> u32 { strs.iter().position(|x| x == s).expect("string internada") as u32 };
    let ns_idx = |android: bool| -> u32 {
        if android {
            ix(NS_ANDROID)
        } else {
            NO_INDEX
        }
    };
    let mut el = Vec::new();
    el.extend(0x0102u16.to_le_bytes()); // RES_XML_START_ELEMENT_TYPE
    el.extend(16u16.to_le_bytes()); // headerSize (8 + line + comment)
    el.extend(0u32.to_le_bytes()); // size (patched)
    el.extend(1u32.to_le_bytes()); // line
    el.extend(NO_INDEX.to_le_bytes()); // comment
    el.extend(NO_INDEX.to_le_bytes()); // ns do elemento
    el.extend(ix(&node.name).to_le_bytes());
    el.extend(20u16.to_le_bytes()); // attributeStart (a partir do ns)
    el.extend(20u16.to_le_bytes()); // attributeSize
    el.extend((node.attrs.len() as u16).to_le_bytes());
    el.extend(0u16.to_le_bytes()); // idIndex
    el.extend(0u16.to_le_bytes()); // classIndex
    el.extend(0u16.to_le_bytes()); // styleIndex
    for at in &node.attrs {
        el.extend(ns_idx(at.android).to_le_bytes());
        el.extend(ix(&at.name).to_le_bytes());
        match &at.val {
            AxVal::Str(s) => {
                let i = ix(s);
                el.extend(i.to_le_bytes()); // raw
                el.extend(8u16.to_le_bytes()); // Res_value.size
                el.push(0u8); // res0
                el.push(TYPE_STRING);
                el.extend(i.to_le_bytes()); // data
            }
            AxVal::Ref(r) => {
                el.extend(NO_INDEX.to_le_bytes());
                el.extend(8u16.to_le_bytes());
                el.push(0u8);
                el.push(TYPE_REFERENCE);
                el.extend(r.to_le_bytes());
            }
            AxVal::Int(i) => {
                el.extend(NO_INDEX.to_le_bytes());
                el.extend(8u16.to_le_bytes());
                el.push(0u8);
                el.push(TYPE_FIRST_INT);
                el.extend((*i as u32).to_le_bytes());
            }
            AxVal::Bool(b) => {
                el.extend(NO_INDEX.to_le_bytes());
                el.extend(8u16.to_le_bytes());
                el.push(0u8);
                el.push(TYPE_BOOLEAN);
                el.extend((*b as u32).to_le_bytes());
            }
            AxVal::Dim { raw } => {
                el.extend(NO_INDEX.to_le_bytes());
                el.extend(8u16.to_le_bytes());
                el.push(0u8);
                el.push(TYPE_DIMENSION);
                el.extend(raw.to_le_bytes());
            }
        }
    }
    pad4(&mut el);
    let el_size = el.len() as u32;
    el[4..8].copy_from_slice(&el_size.to_le_bytes());
    out.extend(&el);

    for c in &node.children {
        emit_node(c, strs, out);
    }

    let mut end = Vec::new();
    end.extend(0x0103u16.to_le_bytes()); // RES_XML_END_ELEMENT_TYPE
    end.extend(16u16.to_le_bytes());
    end.extend(24u32.to_le_bytes());
    end.extend(1u32.to_le_bytes()); // line
    end.extend(NO_INDEX.to_le_bytes()); // comment
    end.extend(NO_INDEX.to_le_bytes()); // ns
    end.extend(ix(&node.name).to_le_bytes());
    out.extend(&end);
}

/// Monta um documento AXML binário completo (doc header + pool + elementos).
/// Sem resource map (attrs casam por NOME no inflater/parser — o mapa é
/// opcional no formato e no consumidor).
pub fn build_axml(root: &AxNode) -> Vec<u8> {
    let mut strs: Vec<String> = vec![NS_ANDROID.to_string(), "android".to_string()];
    collect_strings(root, &mut strs);

    let mut out = Vec::new();
    out.extend(0x0003u16.to_le_bytes()); // RES_XML_TYPE
    out.extend(8u16.to_le_bytes());
    out.extend(0u32.to_le_bytes()); // total size (patched)
    out.extend(string_pool_utf16(
        &strs.iter().map(String::as_str).collect::<Vec<_>>(),
    ));
    emit_node(root, &strs, &mut out);
    let total = out.len() as u32;
    out[4..8].copy_from_slice(&total.to_le_bytes());
    out
}

/// Manifest mínimo válido (root <manifest package=…> + application/activity).
pub fn build_manifest_axml(package: &str, activity_desc: &str) -> Vec<u8> {
    let root = AxNode::new(
        "manifest",
        vec![
            a(false, "package", AxVal::Str(package.to_string())),
            a(true, "versionCode", AxVal::Int(1)),
        ],
        vec![AxNode::new(
            "application",
            vec![],
            vec![AxNode::new(
                "activity",
                vec![a(true, "name", AxVal::Str(activity_desc.to_string()))],
                vec![],
            )],
        )],
    );
    build_axml(&root)
}

// ────────────────────────────────────────────────────────────────────────────
// ZIP (STORED) — local headers + central directory + EOCD, CRC verificado.

pub struct ZipFix {
    entries: Vec<(String, Vec<u8>)>,
}

impl ZipFix {
    pub fn new() -> Self {
        ZipFix {
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, name: &str, data: Vec<u8>) {
        self.entries.push((name.to_string(), data));
    }

    pub fn build(self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut central = Vec::new();
        for (name, data) in &self.entries {
            let off = out.len() as u32;
            let crc = crc32(data);
            out.extend(0x0403_4B50u32.to_le_bytes()); // local file header
            out.extend(20u16.to_le_bytes()); // version needed
            out.extend(0u16.to_le_bytes()); // flags
            out.extend(0u16.to_le_bytes()); // method = STORED
            out.extend(0u16.to_le_bytes()); // mod time
            out.extend(0u16.to_le_bytes()); // mod date
            out.extend(crc.to_le_bytes());
            out.extend((data.len() as u32).to_le_bytes()); // compressed
            out.extend((data.len() as u32).to_le_bytes()); // uncompressed
            out.extend((name.len() as u16).to_le_bytes());
            out.extend(0u16.to_le_bytes()); // extra len
            out.extend(name.as_bytes());
            out.extend(data);

            central.extend(0x0201_4B50u32.to_le_bytes());
            central.extend(20u16.to_le_bytes()); // version made by
            central.extend(20u16.to_le_bytes()); // version needed
            central.extend(0u16.to_le_bytes()); // flags
            central.extend(0u16.to_le_bytes()); // method
            central.extend(0u16.to_le_bytes()); // time
            central.extend(0u16.to_le_bytes()); // date
            central.extend(crc.to_le_bytes());
            central.extend((data.len() as u32).to_le_bytes());
            central.extend((data.len() as u32).to_le_bytes());
            central.extend((name.len() as u16).to_le_bytes());
            central.extend(0u16.to_le_bytes()); // extra
            central.extend(0u16.to_le_bytes()); // comment
            central.extend(0u16.to_le_bytes()); // disk start
            central.extend(0u16.to_le_bytes()); // internal attrs
            central.extend(0u32.to_le_bytes()); // external attrs
            central.extend(off.to_le_bytes());
            central.extend(name.as_bytes());
        }
        let cd_off = out.len() as u32;
        let cd_size = central.len() as u32;
        out.extend(&central);
        out.extend(0x0605_4B50u32.to_le_bytes()); // EOCD
        out.extend(0u16.to_le_bytes()); // disk
        out.extend(0u16.to_le_bytes()); // cd disk
        out.extend((self.entries.len() as u16).to_le_bytes());
        out.extend((self.entries.len() as u16).to_le_bytes());
        out.extend(cd_size.to_le_bytes());
        out.extend(cd_off.to_le_bytes());
        out.extend(0u16.to_le_bytes()); // comment len
        out
    }
}

// ────────────────────────────────────────────────────────────────────────────
// resources.arsc — tabela com 1 pacote (id 0x7f) e N tipos, config default.

pub enum ArscVal {
    /// valor string no pool global do pacote
    Str(String),
    Int(i32),
    Ref(u32),
}

pub struct ArscEntry {
    pub name: String,
    pub val: ArscVal,
}

#[derive(Default)]
pub struct ArscFix {
    values: Vec<String>,
    types: Vec<(&'static str, Vec<ArscEntry>)>,
}

impl ArscFix {
    pub fn new() -> Self {
        Default::default()
    }

    fn slot(&mut self, t: &'static str) -> &mut Vec<ArscEntry> {
        if self.types.iter().any(|(n, _)| *n == t) {
            let pos = self.types.iter().position(|(n, _)| *n == t).unwrap();
            return &mut self.types[pos].1;
        }
        self.types.push((t, Vec::new()));
        let last = self.types.len() - 1;
        &mut self.types[last].1
    }

    /// resid de um recurso já registrado (type_id = ordem de inserção, 1-based).
    pub fn resid_of(&self, t: &str, name: &str) -> u32 {
        let ti = self.types.iter().position(|(n, _)| *n == t).expect("tipo") as u32 + 1;
        let es = &self.types.iter().find(|(n, _)| *n == t).unwrap().1;
        let ei = es.iter().position(|e| e.name == name).expect("entrada") as u32;
        0x7F00_0000 | (ti << 16) | ei
    }

    pub fn add_layout(&mut self, name: &str) -> u32 {
        self.slot("layout").push(ArscEntry {
            name: name.to_string(),
            val: ArscVal::Int(0),
        });
        self.resid_of("layout", name)
    }

    pub fn add_string(&mut self, name: &str, value: &str) -> u32 {
        if !self.values.iter().any(|v| v == value) {
            self.values.push(value.to_string());
        }
        self.slot("string").push(ArscEntry {
            name: name.to_string(),
            val: ArscVal::Str(value.to_string()),
        });
        self.resid_of("string", name)
    }

    pub fn add_id(&mut self, name: &str) -> u32 {
        self.slot("id").push(ArscEntry {
            name: name.to_string(),
            val: ArscVal::Int(0),
        });
        self.resid_of("id", name)
    }

    pub fn build(&self) -> Vec<u8> {
        // strings de tipo e chave (ordem de inserção → type_id/entry idx estáveis)
        let type_names: Vec<&str> = self.types.iter().map(|(n, _)| *n).collect();
        let mut keys: Vec<&str> = Vec::new();
        for (_, es) in &self.types {
            for e in es {
                if !keys.contains(&e.name.as_str()) {
                    keys.push(&e.name);
                }
            }
        }
        let type_pool = string_pool_utf16(&type_names);
        let key_pool = string_pool_utf16(&keys);
        let global_pool =
            string_pool_utf16(&self.values.iter().map(String::as_str).collect::<Vec<_>>());

        // chunks de tipo (um por type)
        let mut type_chunks = Vec::new();
        for (i, (_, entries)) in self.types.iter().enumerate() {
            let type_id = (i + 1) as u8;
            let mut config = vec![0u8; 28];
            config[0..4].copy_from_slice(&28u32.to_le_bytes()); // size (default config)
            let cheader: u16 = 20 + 28; // header + config default (28 bytes)

            let mut offsets = Vec::new();
            let mut body = Vec::new();
            for e in entries {
                offsets.push(body.len() as u32);
                let key_idx = keys.iter().position(|k| *k == e.name).unwrap() as u32;
                body.extend(8u16.to_le_bytes()); // ResTable_entry.size
                body.extend(0u16.to_le_bytes()); // flags (simples)
                body.extend(key_idx.to_le_bytes());
                let (dtype, data) = match &e.val {
                    ArscVal::Str(s) => (
                        TYPE_STRING,
                        self.values.iter().position(|v| v == s).unwrap() as u32,
                    ),
                    ArscVal::Int(i) => (TYPE_FIRST_INT, *i as u32),
                    ArscVal::Ref(r) => (TYPE_REFERENCE, *r),
                };
                body.extend(8u16.to_le_bytes()); // Res_value.size
                body.push(0u8); // res0
                body.push(dtype);
                body.extend(data.to_le_bytes());
            }
            pad4(&mut body);
            let mut c = Vec::new();
            c.extend(0x0201u16.to_le_bytes()); // RES_TABLE_TYPE_CHUNK
            c.extend(cheader.to_le_bytes()); // headerSize
            c.extend(0u32.to_le_bytes()); // size (patched)
            c.push(type_id);
            c.push(0u8); // flags (não-sparse)
            c.extend(0u16.to_le_bytes()); // reserved
            c.extend((entries.len() as u32).to_le_bytes()); // entryCount
            let entries_start = cheader as u32 + (offsets.len() as u32) * 4;
            c.extend(entries_start.to_le_bytes());
            c.extend(&config);
            c.extend(offsets.iter().flat_map(|o| o.to_le_bytes()));
            c.extend(&body);
            let csz = c.len() as u32;
            c[4..8].copy_from_slice(&csz.to_le_bytes());
            type_chunks.push(c);
        }

        // pacote (header de 288 bytes + pools + tipos)
        let mut pkg = vec![0u8; 288];
        pkg[0..2].copy_from_slice(&0x0200u16.to_le_bytes());
        pkg[2..4].copy_from_slice(&288u16.to_le_bytes());
        // size patcheado no fim
        pkg[8..12].copy_from_slice(&0x7Fu32.to_le_bytes()); // package id
        let name = "com.test";
        for (i, u) in name.encode_utf16().chain(std::iter::once(0)).enumerate() {
            pkg[12 + i * 2..14 + i * 2].copy_from_slice(&u.to_le_bytes());
        }
        let type_strings_off = 288u32;
        let key_strings_off = 288u32 + type_pool.len() as u32;
        pkg[268..272].copy_from_slice(&type_strings_off.to_le_bytes());
        pkg[272..276].copy_from_slice(&0u32.to_le_bytes()); // lastPublicType
        pkg[276..280].copy_from_slice(&key_strings_off.to_le_bytes());
        pkg[280..284].copy_from_slice(&0u32.to_le_bytes()); // lastPublicKey
        pkg.extend(&type_pool);
        pkg.extend(&key_pool);
        for c in &type_chunks {
            pkg.extend(c);
        }
        pad4(&mut pkg);
        let pkg_size = pkg.len() as u32;
        pkg[4..8].copy_from_slice(&pkg_size.to_le_bytes());

        // tabela
        let mut out = Vec::new();
        out.extend(0x0002u16.to_le_bytes()); // RES_TABLE_TYPE
        out.extend(12u16.to_le_bytes()); // headerSize
        out.extend(0u32.to_le_bytes()); // total size (patched)
        out.extend(1u32.to_le_bytes()); // packageCount
        out.extend(&global_pool);
        out.extend(&pkg);
        let total = out.len() as u32;
        out[4..8].copy_from_slice(&total.to_le_bytes());
        out
    }
}

/// APK sintético completo: manifest + arsc + entradas extras (layouts).
pub fn build_apk(
    package: &str,
    activity_desc: &str,
    arsc: Vec<u8>,
    extras: Vec<(&str, Vec<u8>)>,
) -> Vec<u8> {
    let mut z = ZipFix::new();
    z.add(
        "AndroidManifest.xml",
        build_manifest_axml(package, activity_desc),
    );
    z.add("resources.arsc", arsc);
    for (name, data) in extras {
        z.add(name, data);
    }
    z.build()
}

// issue #40: o AxNode do fixture tem o MESMO risco de drop recursivo que o
// XmlElement do parser (o teste de AXML profundo constrói a árvore com este
// tipo) — drop iterativo com stack explícita.
impl Drop for AxNode {
    fn drop(&mut self) {
        let mut stack = Vec::new();
        stack.extend(std::mem::take(&mut self.children));
        while let Some(mut node) = stack.pop() {
            stack.extend(std::mem::take(&mut node.children));
        }
    }
}
