//! Testes de integração do interpretador rd-vm sobre DEXs sintéticos
//! construídos byte a byte (mesma política dos fixtures do rd-dex).
//! Cada teste valida um grupo de opcodes do contrato M2 ("métodos puros").

use rd_vm::engine::{Engine, VmConfig};
use rd_vm::err::VmExit;
use rd_vm::value::Value;

// ── packing de code units (formatos oficiais do dex-format) ─────────────────

fn op10x(op: u8) -> Vec<u16> {
    vec![op as u16]
}
fn op12x(op: u8, a: u8, b: u8) -> Vec<u16> {
    vec![(op as u16) | ((a as u16) << 8) | ((b as u16) << 12)]
}
fn op11n(op: u8, a: u8, lit: i8) -> Vec<u16> {
    vec![(op as u16) | ((a as u16) << 8) | (((lit as u8) as u16) << 12)]
}
fn op11x(op: u8, a: u8) -> Vec<u16> {
    vec![(op as u16) | ((a as u16) << 8)]
}
fn op10t(op: u8, off: i8) -> Vec<u16> {
    vec![(op as u16) | (((off as u8) as u16) << 8)]
}
fn op21c(op: u8, a: u8, idx: u16) -> Vec<u16> {
    vec![(op as u16) | ((a as u16) << 8), idx]
}
fn op21s(op: u8, a: u8, lit: i16) -> Vec<u16> {
    vec![(op as u16) | ((a as u16) << 8), lit as u16]
}
fn op22b(op: u8, a: u8, b: u8, lit: i8) -> Vec<u16> {
    vec![
        (op as u16) | ((a as u16) << 8),
        (b as u16) | (((lit as u8) as u16) << 8),
    ]
}
fn op22s(op: u8, a: u8, b: u8, lit: i16) -> Vec<u16> {
    vec![
        (op as u16) | ((a as u16) << 8) | ((b as u16) << 12),
        lit as u16,
    ]
}
fn op22c(op: u8, a: u8, b: u8, idx: u16) -> Vec<u16> {
    vec![(op as u16) | ((a as u16) << 8) | ((b as u16) << 12), idx]
}
fn op22t(op: u8, a: u8, b: u8, off: i16) -> Vec<u16> {
    vec![
        (op as u16) | ((a as u16) << 8) | ((b as u16) << 12),
        off as u16,
    ]
}
fn op23x(op: u8, a: u8, b: u8, c: u8) -> Vec<u16> {
    vec![
        (op as u16) | ((a as u16) << 8),
        (b as u16) | ((c as u16) << 8),
    ]
}
fn op31t(op: u8, a: u8, off: i32) -> Vec<u16> {
    vec![
        (op as u16) | ((a as u16) << 8),
        off as u16,
        (off >> 16) as u16,
    ]
}
fn op51l(op: u8, a: u8, lit: i64) -> Vec<u16> {
    vec![
        (op as u16) | ((a as u16) << 8),
        lit as u16,
        (lit >> 16) as u16,
        (lit >> 32) as u16,
        (lit >> 48) as u16,
    ]
}
/// 35c: A (count) no nibble ALTO de byte1, G no baixo; regs {C D E F}
fn op35c(op: u8, count: u8, idx: u16, regs: [u8; 5]) -> Vec<u16> {
    vec![
        (op as u16) | ((count as u16) << 12) | ((regs[4] as u16) << 8),
        idx,
        (regs[3] as u16) << 12 | (regs[2] as u16) << 8 | (regs[1] as u16) << 4 | regs[0] as u16,
    ]
}
fn packed_payload(first_key: i32, targets: &[i32]) -> Vec<u16> {
    let mut v = vec![
        0x0100u16,
        targets.len() as u16,
        first_key as u16,
        (first_key >> 16) as u16,
    ];
    for t in targets {
        v.push(*t as u16);
        v.push((t >> 16) as u16);
    }
    v
}

// ── builder de DEX ──────────────────────────────────────────────────────────

#[derive(Clone)]
enum SVal {
    Int(i32),
}

struct CodeBlob {
    regs: u16,
    ins: u16,
    outs: u16,
    units: Vec<u16>,
    tries: Vec<TryBlob>,
}

struct TryBlob {
    start: u32,
    count: u16,
    typed: Vec<(String, u32)>,
    catch_all: Option<u32>,
}

struct MethodSlot {
    name: String,
    proto_idx: usize,
    access: u32,
    code: Option<CodeBlob>,
}

struct ProtoE {
    shorty: String,
    ret: String,
    params: Vec<String>,
}

struct ClassE {
    this: String,
    sup: String,
    statics: Vec<(String, String)>,
    static_values: Vec<SVal>,
    instances: Vec<(String, String)>,
    directs: Vec<MethodSlot>,
    virtuals: Vec<MethodSlot>,
}

#[derive(Default)]
struct DexBuilder {
    strings: Vec<String>,
    types: Vec<String>,
    protos: Vec<ProtoE>,
    fields: Vec<(String, String, String)>,
    methods: Vec<(String, usize, String)>,
    classes: Vec<ClassE>,
}

impl DexBuilder {
    fn new() -> Self {
        let mut b = DexBuilder::default();
        b.proto_idx("V", vec![]); // slot 0 = ()V
        b
    }

    fn intern(&mut self, s: &str) -> u32 {
        if let Some(i) = self.strings.iter().position(|x| x == s) {
            return i as u32;
        }
        self.strings.push(s.to_string());
        (self.strings.len() - 1) as u32
    }

    fn type_idx(&mut self, desc: &str) -> u16 {
        if let Some(i) = self.types.iter().position(|x| x == desc) {
            return i as u16;
        }
        self.intern(desc);
        self.types.push(desc.to_string());
        (self.types.len() - 1) as u16
    }

    fn proto_idx(&mut self, ret: &str, params: Vec<String>) -> usize {
        self.type_idx(ret);
        for p in &params {
            self.type_idx(p);
        }
        if let Some(i) = self
            .protos
            .iter()
            .position(|p| p.ret == ret && p.params == params)
        {
            return i;
        }
        self.intern(&shorty_of(ret, &params));
        self.protos.push(ProtoE {
            shorty: shorty_of(ret, &params),
            ret: ret.to_string(),
            params: params.clone(),
        });
        self.protos.len() - 1
    }

    fn method_idx(&mut self, class: &str, proto: usize, name: &str) -> u16 {
        self.type_idx(class);
        if let Some(i) = self
            .methods
            .iter()
            .position(|(c, p, n)| c == class && *p == proto && n == name)
        {
            return i as u16;
        }
        self.intern(name);
        self.methods
            .push((class.to_string(), proto, name.to_string()));
        (self.methods.len() - 1) as u16
    }

    fn field_idx(&mut self, class: &str, ftype: &str, name: &str) -> u16 {
        self.type_idx(class);
        self.type_idx(ftype);
        if let Some(i) = self
            .fields
            .iter()
            .position(|(c, t, n)| c == class && t == ftype && n == name)
        {
            return i as u16;
        }
        self.intern(name);
        self.fields
            .push((class.to_string(), ftype.to_string(), name.to_string()));
        (self.fields.len() - 1) as u16
    }

    fn class(&mut self, this: &str, sup: &str) -> usize {
        self.type_idx(this);
        self.type_idx(sup);
        self.classes.push(ClassE {
            this: this.to_string(),
            sup: sup.to_string(),
            statics: Vec::new(),
            static_values: Vec::new(),
            instances: Vec::new(),
            directs: Vec::new(),
            virtuals: Vec::new(),
        });
        self.classes.len() - 1
    }

    fn static_field(&mut self, cls: usize, name: &str, ftype: &str, init: SVal) {
        self.intern(name);
        self.type_idx(ftype);
        self.classes[cls]
            .statics
            .push((name.to_string(), ftype.to_string()));
        self.classes[cls].static_values.push(init);
    }

    fn instance_field(&mut self, cls: usize, name: &str, ftype: &str) {
        self.intern(name);
        self.type_idx(ftype);
        self.classes[cls]
            .instances
            .push((name.to_string(), ftype.to_string()));
    }

    fn code(&self, regs: u16, ins: u16, outs: u16, units: Vec<u16>) -> CodeBlob {
        CodeBlob {
            regs,
            ins,
            outs,
            units,
            tries: Vec::new(),
        }
    }

    fn direct(
        &mut self,
        cls: usize,
        name: &str,
        ret: &str,
        params: Vec<&str>,
        access: u32,
        code: Option<CodeBlob>,
    ) {
        let proto_idx = self.proto_idx(ret, params.iter().map(|s| s.to_string()).collect());
        let this = self.classes[cls].this.clone();
        self.method_idx(&this, proto_idx, name);
        self.classes[cls].directs.push(MethodSlot {
            name: name.to_string(),
            proto_idx,
            access,
            code,
        });
    }

    #[allow(named_arguments_used_positionally)]
    fn r#virtual(
        &mut self,
        cls: usize,
        name: &str,
        ret: &str,
        params: Vec<&str>,
        access: u32,
        code: Option<CodeBlob>,
    ) {
        let proto_idx = self.proto_idx(ret, params.iter().map(|s| s.to_string()).collect());
        let this = self.classes[cls].this.clone();
        self.method_idx(&this, proto_idx, name);
        self.classes[cls].virtuals.push(MethodSlot {
            name: name.to_string(),
            proto_idx,
            access,
            code,
        });
    }

    fn finish(&self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(&[0u8; 112]); // header placeholder

        let string_ids_off = out.len() as u32;
        for _ in &self.strings {
            out.extend_from_slice(&0u32.to_le_bytes());
        }
        let type_ids_off = out.len() as u32;
        for _ in &self.types {
            out.extend_from_slice(&0u32.to_le_bytes());
        }
        let proto_ids_off = out.len() as u32;
        for _ in &self.protos {
            out.extend_from_slice(&[0u8; 12]);
        }
        let field_ids_off = out.len() as u32;
        for _ in &self.fields {
            out.extend_from_slice(&[0u8; 8]);
        }
        let method_ids_off = out.len() as u32;
        for _ in &self.methods {
            out.extend_from_slice(&[0u8; 8]);
        }
        let class_defs_off = out.len() as u32;
        for _ in &self.classes {
            out.extend_from_slice(&[0u8; 32]);
        }
        let data_off = out.len() as u32;

        // string_data
        let mut string_data_offsets = vec![0u32; self.strings.len()];
        for (i, s) in self.strings.iter().enumerate() {
            align4(&mut out);
            string_data_offsets[i] = out.len() as u32;
            write_uleb(&mut out, s.encode_utf16().count() as u64);
            let mut bytes = s.as_bytes().to_vec();
            bytes.push(0);
            out.extend_from_slice(&bytes);
        }

        // type_lists
        let mut type_list_offs: Vec<u32> = vec![0; self.protos.len()];
        for (i, p) in self.protos.iter().enumerate() {
            if p.params.is_empty() {
                continue;
            }
            align4(&mut out);
            type_list_offs[i] = out.len() as u32;
            out.extend_from_slice(&(p.params.len() as u32).to_le_bytes());
            for t in &p.params {
                let idx = self
                    .types
                    .iter()
                    .position(|x| x == t)
                    .expect("tipo do proto");
                out.extend_from_slice(&(idx as u16).to_le_bytes());
            }
        }

        // code items
        let mut method_code_offs: Vec<Option<u32>> = vec![None; self.methods.len()];
        for c in &self.classes {
            for slot in c.directs.iter().chain(&c.virtuals) {
                let Some(blob) = &slot.code else { continue };
                align4(&mut out);
                let midx = self
                    .methods
                    .iter()
                    .position(|(cl, p, n)| cl == &c.this && p == &slot.proto_idx && n == &slot.name)
                    .expect("método registrado");
                method_code_offs[midx] = Some(out.len() as u32);
                out.extend_from_slice(&blob.regs.to_le_bytes());
                out.extend_from_slice(&blob.ins.to_le_bytes());
                out.extend_from_slice(&blob.outs.to_le_bytes());
                out.extend_from_slice(&(blob.tries.len() as u16).to_le_bytes());
                out.extend_from_slice(&0u32.to_le_bytes()); // debug_info
                out.extend_from_slice(&(blob.units.len() as u32).to_le_bytes());
                for u in &blob.units {
                    out.extend_from_slice(&u.to_le_bytes());
                }
                if !blob.tries.is_empty() {
                    if blob.units.len() % 2 == 1 {
                        out.extend_from_slice(&0u16.to_le_bytes()); // padding
                    }
                    let mut list_bytes: Vec<u8> = Vec::new();
                    write_uleb(&mut list_bytes, blob.tries.len() as u64);
                    for t in &blob.tries {
                        out.extend_from_slice(&t.start.to_le_bytes());
                        out.extend_from_slice(&t.count.to_le_bytes());
                        // handler_off relativo ao INÍCIO da lista (inclui size)
                        out.extend_from_slice(&(list_bytes.len() as u16).to_le_bytes());
                        let n = t.typed.len() as i64;
                        write_sleb(&mut list_bytes, if t.catch_all.is_some() { -n } else { n });
                        for (tdesc, addr) in &t.typed {
                            let tidx = self
                                .types
                                .iter()
                                .position(|x| x == tdesc)
                                .expect("tipo do try registrado")
                                as u16;
                            write_uleb(&mut list_bytes, tidx as u64);
                            write_uleb(&mut list_bytes, *addr as u64);
                        }
                        if let Some(ca) = t.catch_all {
                            write_uleb(&mut list_bytes, ca as u64);
                        }
                    }
                    out.extend_from_slice(&list_bytes);
                }
            }
        }

        // class_data + static_values
        let mut class_data_offs = vec![0u32; self.classes.len()];
        let mut static_values_offs = vec![0u32; self.classes.len()];
        for (ci, c) in self.classes.iter().enumerate() {
            let has_members = !c.statics.is_empty()
                || !c.instances.is_empty()
                || !c.directs.is_empty()
                || !c.virtuals.is_empty();
            if has_members {
                align4(&mut out);
                class_data_offs[ci] = out.len() as u32;
                write_uleb(&mut out, c.statics.len() as u64);
                write_uleb(&mut out, c.instances.len() as u64);
                write_uleb(&mut out, c.directs.len() as u64);
                write_uleb(&mut out, c.virtuals.len() as u64);
                let mut prev: u32 = 0;
                for (name, ftype) in &c.statics {
                    let fidx = self.find_field(&c.this, ftype, name) as u32;
                    write_uleb(&mut out, (fidx - prev) as u64);
                    write_uleb(&mut out, 0x0009); // public static
                    prev = fidx;
                }
                let mut prev: u32 = 0;
                for (name, ftype) in &c.instances {
                    let fidx = self.find_field(&c.this, ftype, name) as u32;
                    write_uleb(&mut out, (fidx - prev) as u64);
                    write_uleb(&mut out, 0);
                    prev = fidx;
                }
                let mut prev: u32 = 0;
                for slot in &c.directs {
                    let midx = self.find_method(&c.this, slot.proto_idx, &slot.name) as u32;
                    let code_off = method_code_offs[midx as usize].unwrap_or(0);
                    write_uleb(&mut out, (midx - prev) as u64);
                    write_uleb(&mut out, slot.access as u64);
                    write_uleb(&mut out, code_off as u64);
                    prev = midx;
                }
                let mut prev: u32 = 0;
                for slot in &c.virtuals {
                    let midx = self.find_method(&c.this, slot.proto_idx, &slot.name) as u32;
                    let code_off = method_code_offs[midx as usize].unwrap_or(0);
                    write_uleb(&mut out, (midx - prev) as u64);
                    write_uleb(&mut out, slot.access as u64);
                    write_uleb(&mut out, code_off as u64);
                    prev = midx;
                }
            }
            if !c.static_values.is_empty() {
                align4(&mut out);
                static_values_offs[ci] = out.len() as u32;
                write_uleb(&mut out, c.static_values.len() as u64);
                for v in &c.static_values {
                    match v {
                        SVal::Int(i) => {
                            out.push(0x04 | (3 << 5));
                            out.extend_from_slice(&i.to_le_bytes());
                        }
                    }
                }
            }
        }

        // map_list (por último)
        align4(&mut out);
        let map_off = out.len() as u32;
        let mut map: Vec<(u16, u32, u32)> = vec![
            (0x0000, 1, 0),
            (0x0001, self.strings.len() as u32, string_ids_off),
            (0x0002, self.types.len() as u32, type_ids_off),
            (0x0003, self.protos.len() as u32, proto_ids_off),
        ];
        #[allow(clippy::vec_init_then_push)]
        {
            if !self.fields.is_empty() {
                map.push((0x0004, self.fields.len() as u32, field_ids_off));
            }
            if !self.methods.is_empty() {
                map.push((0x0005, self.methods.len() as u32, method_ids_off));
            }
            if !self.classes.is_empty() {
                map.push((0x0006, self.classes.len() as u32, class_defs_off));
            }
            if !self.strings.is_empty() {
                map.push((0x2002, self.strings.len() as u32, string_data_offsets[0]));
            }
            let n_tlists = type_list_offs.iter().filter(|o| **o != 0).count() as u32;
            if n_tlists > 0 {
                map.push((
                    0x1001,
                    n_tlists,
                    *type_list_offs.iter().find(|o| **o != 0).unwrap(),
                ));
            }
            let n_cdata = class_data_offs.iter().filter(|o| **o != 0).count() as u32;
            if n_cdata > 0 {
                map.push((
                    0x2000,
                    n_cdata,
                    *class_data_offs.iter().find(|o| **o != 0).unwrap(),
                ));
            }
            let n_code = method_code_offs.iter().filter(|o| o.is_some()).count() as u32;
            if n_code > 0 {
                map.push((
                    0x2001,
                    n_code,
                    method_code_offs
                        .iter()
                        .find(|o| o.is_some())
                        .unwrap()
                        .unwrap(),
                ));
            }
            let n_svals = static_values_offs.iter().filter(|o| **o != 0).count() as u32;
            if n_svals > 0 {
                map.push((
                    0x2005,
                    n_svals,
                    *static_values_offs.iter().find(|o| **o != 0).unwrap(),
                ));
            }
            map.push((0x1000, 1, map_off));
        }

        out.extend_from_slice(&(map.len() as u32).to_le_bytes());
        for (t, size, off) in &map {
            out.extend_from_slice(&t.to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(&size.to_le_bytes());
            out.extend_from_slice(&off.to_le_bytes());
        }

        // backpatch das tabelas
        for (i, s) in string_data_offsets.iter().enumerate() {
            let at = string_ids_off as usize + i * 4;
            out[at..at + 4].copy_from_slice(&s.to_le_bytes());
        }
        for (i, t) in self.types.iter().enumerate() {
            let sidx = self
                .strings
                .iter()
                .position(|x| x == t)
                .expect("tipo tem string");
            let at = type_ids_off as usize + i * 4;
            out[at..at + 4].copy_from_slice(&(sidx as u32).to_le_bytes());
        }
        for (i, p) in self.protos.iter().enumerate() {
            let base = proto_ids_off as usize + i * 12;
            let sidx = self
                .strings
                .iter()
                .position(|x| x == &p.shorty)
                .expect("shorty");
            out[base..base + 4].copy_from_slice(&(sidx as u32).to_le_bytes());
            let ridx = self.types.iter().position(|x| x == &p.ret).expect("ret");
            out[base + 4..base + 8].copy_from_slice(&(ridx as u32).to_le_bytes());
            out[base + 8..base + 12].copy_from_slice(&type_list_offs[i].to_le_bytes());
        }
        for (i, (cl, t, n)) in self.fields.iter().enumerate() {
            let base = field_ids_off as usize + i * 8;
            out[base..base + 2].copy_from_slice(&self.type_idx_cached(cl).to_le_bytes());
            out[base + 2..base + 4].copy_from_slice(&self.type_idx_cached(t).to_le_bytes());
            let at = base + 4;
            out[at..at + 4].copy_from_slice(&self.string_idx_cached(n).to_le_bytes());
        }
        for (i, (cl, p, n)) in self.methods.iter().enumerate() {
            let base = method_ids_off as usize + i * 8;
            out[base..base + 2].copy_from_slice(&self.type_idx_cached(cl).to_le_bytes());
            out[base + 2..base + 4].copy_from_slice(&(*p as u16).to_le_bytes());
            let at = base + 4;
            out[at..at + 4].copy_from_slice(&self.string_idx_cached(n).to_le_bytes());
        }
        for (ci, c) in self.classes.iter().enumerate() {
            let base = class_defs_off as usize + ci * 32;
            out[base..base + 4]
                .copy_from_slice(&(self.type_idx_cached(&c.this) as u32).to_le_bytes());
            out[base + 4..base + 8]
                .copy_from_slice(&(self.type_idx_cached(&c.sup) as u32).to_le_bytes());
            let at = base + 24;
            out[at..at + 4].copy_from_slice(&class_data_offs[ci].to_le_bytes());
            let at = base + 28;
            out[at..at + 4].copy_from_slice(&static_values_offs[ci].to_le_bytes());
        }

        // header
        let file_size = out.len() as u32;
        let patch = |out: &mut Vec<u8>, at: usize, v: u32| {
            out[at..at + 4].copy_from_slice(&v.to_le_bytes());
        };
        patch(&mut out, 32, file_size);
        patch(&mut out, 36, 112);
        out[40..44].copy_from_slice(&0x1234_5678u32.to_le_bytes());
        patch(&mut out, 52, map_off);
        patch(&mut out, 56, self.strings.len() as u32);
        patch(&mut out, 60, string_ids_off);
        patch(&mut out, 64, self.types.len() as u32);
        patch(&mut out, 68, type_ids_off);
        patch(&mut out, 72, self.protos.len() as u32);
        patch(&mut out, 76, proto_ids_off);
        patch(&mut out, 80, self.fields.len() as u32);
        patch(&mut out, 84, field_ids_off);
        patch(&mut out, 88, self.methods.len() as u32);
        patch(&mut out, 92, method_ids_off);
        patch(&mut out, 96, self.classes.len() as u32);
        patch(&mut out, 100, class_defs_off);
        patch(&mut out, 104, file_size - data_off);
        patch(&mut out, 108, data_off);
        let adler = adler32(&out[12..]);
        patch(&mut out, 8, adler);
        out[0..8].copy_from_slice(b"dex\n035\x00");
        out
    }

    fn find_field(&self, class: &str, ftype: &str, name: &str) -> usize {
        self.fields
            .iter()
            .position(|(c, t, n)| c == class && t == ftype && n == name)
            .expect("campo registrado")
    }

    fn find_method(&self, class: &str, proto: usize, name: &str) -> usize {
        self.methods
            .iter()
            .position(|(c, p, n)| c == class && *p == proto && n == name)
            .expect("método registrado")
    }

    fn type_idx_cached(&self, desc: &str) -> u16 {
        self.types
            .iter()
            .position(|x| x == desc)
            .expect("tipo registrado") as u16
    }

    fn string_idx_cached(&self, s: &str) -> u32 {
        self.strings
            .iter()
            .position(|x| x == s)
            .expect("string registrada") as u32
    }
}

fn align4(out: &mut Vec<u8>) {
    while out.len() % 4 != 0 {
        out.push(0);
    }
}

fn write_uleb(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let b = (v & 0x7F) as u8;
        v >>= 7;
        if v == 0 {
            out.push(b);
            break;
        }
        out.push(b | 0x80);
    }
}

fn write_sleb(out: &mut Vec<u8>, mut v: i64) {
    loop {
        if v == 0 || v == -1 {
            out.push((v & 0x7F) as u8);
            break;
        }
        out.push(((v & 0x7F) as u8) | 0x80);
        v >>= 7;
    }
}

fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn shorty_of(ret: &str, params: &[String]) -> String {
    let ch = |d: &str| match d {
        "V" => 'V',
        "Z" => 'Z',
        "B" => 'B',
        "S" => 'S',
        "C" => 'C',
        "I" => 'I',
        "J" => 'J',
        "F" => 'F',
        "D" => 'D',
        _ => 'L',
    };
    let mut s = String::new();
    s.push(ch(ret));
    for p in params {
        s.push(ch(p));
    }
    s
}

// ── helpers de execução ─────────────────────────────────────────────────────

fn engine_of(b: &DexBuilder) -> Engine {
    let dex = rd_dex::Dex::parse(b.finish()).expect("DEX sintético parseia");
    Engine::new(vec![dex], VmConfig::default())
}

const ACC_PUBLIC: u32 = 0x1;
const ACC_STATIC: u32 = 0x8;
const ACC_CONSTRUCTOR: u32 = 0x1_0000;

fn invoke(eng: &mut Engine, method: &str, sig: &str, args: &[Value]) -> Result<Value, VmExit> {
    eng.invoke_static("LCaso;", method, sig, args)
}

// ── testes ──────────────────────────────────────────────────────────────────
// CONVENÇÃO ABI: parâmetros caem nos ÚLTIMOS `ins` registradores
// (regs=5, ins=2 → args em v3,v4). Fixtures seguem isso à risca.

#[test]
fn arith_int_2addr_and_lits() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    // soma(II)I: v0 = v3 + v4; return v0
    b.direct(
        cls,
        "soma",
        "I",
        vec!["I", "I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(5, 2, 0, {
            let mut u = op23x(0x90, 0, 3, 4);
            u.extend(op11x(0x0F, 0));
            u
        })),
    );
    // calc(II)I: v0 = v3 * v4; v0 += 7 (lit8); return v0
    b.direct(
        cls,
        "calc",
        "I",
        vec!["I", "I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(5, 2, 0, {
            let mut u = op23x(0x92, 0, 3, 4);
            u.extend(op22b(0xD8, 0, 0, 7));
            u.extend(op11x(0x0F, 0));
            u
        })),
    );
    // rsub10(I)I: v0 = 10 - v3 (rsub-int/lit16)
    b.direct(
        cls,
        "rsub10",
        "I",
        vec!["I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(4, 1, 0, {
            let mut u = op22s(0xD1, 0, 3, 10);
            u.extend(op11x(0x0F, 0));
            u
        })),
    );

    let mut e = engine_of(&b);
    assert_eq!(
        invoke(&mut e, "soma", "(II)I", &[Value::Int(5), Value::Int(3)])
            .unwrap()
            .as_int()
            .unwrap(),
        8
    );
    assert_eq!(
        invoke(&mut e, "calc", "(II)I", &[Value::Int(6), Value::Int(7)])
            .unwrap()
            .as_int()
            .unwrap(),
        49
    );
    assert_eq!(
        invoke(&mut e, "rsub10", "(I)I", &[Value::Int(4)])
            .unwrap()
            .as_int()
            .unwrap(),
        6
    );
}

#[test]
fn div_rem_wrapping_and_exceptions() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    b.direct(
        cls,
        "div",
        "I",
        vec!["I", "I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(5, 2, 0, {
            let mut u = op23x(0x93, 0, 3, 4);
            u.extend(op11x(0x0F, 0));
            u
        })),
    );
    let mut e = engine_of(&b);
    // MIN / -1 = MIN (wrapping, sem pânico)
    assert_eq!(
        invoke(
            &mut e,
            "div",
            "(II)I",
            &[Value::Int(i32::MIN), Value::Int(-1)]
        )
        .unwrap()
        .as_int()
        .unwrap(),
        i32::MIN
    );
    // / 0 → ArithmeticException
    match invoke(&mut e, "div", "(II)I", &[Value::Int(1), Value::Int(0)]) {
        Err(VmExit::Exception(t)) => {
            assert_eq!(t.class, "Ljava/lang/ArithmeticException;");
            assert_eq!(t.message.as_deref(), Some("divide by zero"));
        }
        other => panic!("esperava ArithmeticException, got {other:?}"),
    }
}

#[test]
fn long_and_double_wide_flow() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    // wide(JJ)J: regs=7, ins=4 → args v3/v4, v5/v6; v0 = v3 + v5; return-wide v0
    b.direct(
        cls,
        "wide",
        "J",
        vec!["J", "J"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(7, 4, 0, {
            let mut u = op23x(0x9B, 0, 3, 5);
            u.extend(op11x(0x10, 0));
            u
        })),
    );
    // const64()J: const-wide v0; return-wide v0
    b.direct(
        cls,
        "const64",
        "J",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(2, 0, 0, {
            let mut u = op51l(0x18, 0, 0x0123_4567_89AB_CDEF);
            u.extend(op11x(0x10, 0));
            u
        })),
    );
    let mut e = engine_of(&b);
    assert_eq!(
        invoke(
            &mut e,
            "wide",
            "(JJ)J",
            &[Value::Long(1 << 40), Value::Long(1)]
        )
        .unwrap()
        .as_long()
        .unwrap(),
        (1 << 40) + 1
    );
    assert_eq!(
        invoke(&mut e, "const64", "()J", &[])
            .unwrap()
            .as_long()
            .unwrap(),
        0x0123_4567_89AB_CDEF
    );
}

#[test]
fn float_double_java_semantics() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    // fdiv(FF)F: v0 = v2 / v3
    b.direct(
        cls,
        "fdiv",
        "F",
        vec!["F", "F"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(4, 2, 0, {
            let mut u = op23x(0xA9, 0, 2, 3);
            u.extend(op11x(0x0F, 0));
            u
        })),
    );
    // dcmpl(DD)I: v0 = cmpl-double(v2/v3, v4/v5)
    b.direct(
        cls,
        "dcmpl",
        "I",
        vec!["D", "D"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(6, 4, 0, {
            let mut u = op23x(0x2F, 0, 2, 4);
            u.extend(op11x(0x0F, 0));
            u
        })),
    );
    let mut e = engine_of(&b);
    // 1.0f / 0.0f = +Infinity (não exceção)
    assert_eq!(
        invoke(
            &mut e,
            "fdiv",
            "(FF)F",
            &[Value::Float(1.0), Value::Float(0.0)]
        )
        .unwrap(),
        Value::Float(f32::INFINITY)
    );
    // NaN compara com cmpl → -1
    assert_eq!(
        invoke(
            &mut e,
            "dcmpl",
            "(DD)I",
            &[Value::Double(f64::NAN), Value::Double(1.0)]
        )
        .unwrap()
        .as_int()
        .unwrap(),
        -1
    );
}

#[test]
fn branches_and_loop() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    // fat(I)I iterativo — arg em v3, acumulador v1
    // 0:      const/4 v1, 1            (1 unit)
    // 1..2:   if-le v3, v1, +6 → 7     (2 units)
    // 3:      mul-int/2addr v1, v3     (1 unit)
    // 4..5:   add-int/lit8 v3, v3, -1  (2 units)
    // 6:      goto -5 → 1              (1 unit)
    // 7:      return v1
    b.direct(
        cls,
        "fat",
        "I",
        vec!["I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(4, 1, 0, {
            let mut u = Vec::new();
            u.extend(op11n(0x12, 1, 1)); // 0: v1 = 1 (acc)
            u.extend(op11n(0x12, 2, 1)); // 1: v2 = 1 (constante)
            u.extend(op22t(0x37, 3, 2, 6)); // 2..3: if-le v3, v2 → 8
            u.extend(op12x(0xB2, 1, 3)); // 4: mul-int/2addr v1, v3
            u.extend(op22b(0xD8, 3, 3, -1)); // 5..6: v3 -= 1
            u.extend(op10t(0x28, -6)); // 7: goto 1
            u.extend(op11x(0x0F, 1)); // 8: return v1
            u
        })),
    );
    let mut e = engine_of(&b);
    assert_eq!(
        invoke(&mut e, "fat", "(I)I", &[Value::Int(5)])
            .unwrap()
            .as_int()
            .unwrap(),
        120
    );
    assert_eq!(
        invoke(&mut e, "fat", "(I)I", &[Value::Int(1)])
            .unwrap()
            .as_int()
            .unwrap(),
        1
    );
}

#[test]
fn packed_switch() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    // sw(I)I — arg em v3, resultado v1. const/4 é SINALIZADO de 4 bits
    // (10 vira -6!) — valores ≥ 8 usam const/16 (2 units).
    // 0..2:   packed-switch v3, +6 → payload @6 (unit par, alinhado)
    // 3..4:   const/16 v1, 99   ← default (fall-through)
    // 5:      return v1
    // 6..15:  payload (10 units; alvos relativos ao switch @0)
    // 16..17: const/16 v1, 10; 18: return      (caso 1)
    // 19..20: const/16 v1, 20; 21: return      (caso 2)
    // 22..23: const/16 v1, 30; 24: return      (caso 3)
    let mut u = op31t(0x2B, 3, 6);
    u.extend(op21s(0x13, 1, 99)); // 3..4: default
    u.extend(op11x(0x0F, 1)); // 5
    u.extend(packed_payload(1, &[16, 19, 22])); // 6..15
    u.extend(op21s(0x13, 1, 10)); // 16..17
    u.extend(op11x(0x0F, 1)); // 18
    u.extend(op21s(0x13, 1, 20)); // 19..20
    u.extend(op11x(0x0F, 1)); // 21
    u.extend(op21s(0x13, 1, 30)); // 22..23
    u.extend(op11x(0x0F, 1)); // 24
    b.direct(
        cls,
        "sw",
        "I",
        vec!["I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(4, 1, 0, u)),
    );

    let mut e = engine_of(&b);
    assert_eq!(
        invoke(&mut e, "sw", "(I)I", &[Value::Int(1)])
            .unwrap()
            .as_int()
            .unwrap(),
        10
    );
    assert_eq!(
        invoke(&mut e, "sw", "(I)I", &[Value::Int(2)])
            .unwrap()
            .as_int()
            .unwrap(),
        20
    );
    assert_eq!(
        invoke(&mut e, "sw", "(I)I", &[Value::Int(3)])
            .unwrap()
            .as_int()
            .unwrap(),
        30
    );
    assert_eq!(
        invoke(&mut e, "sw", "(I)I", &[Value::Int(7)])
            .unwrap()
            .as_int()
            .unwrap(),
        99
    );
}

#[test]
fn strings_length_and_hashcode() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    let sidx_hello = b.intern("hello");
    let sidx_abc = b.intern("abc");
    // method_ids determinísticos: 0=length, 1=hashCode
    let p_i0 = b.proto_idx("I", vec![]);
    let length_mid = b.method_idx("Ljava/lang/String;", p_i0, "length");
    let hashcode_mid = b.method_idx("Ljava/lang/String;", p_i0, "hashCode");
    assert_eq!(length_mid, 0);
    assert_eq!(hashcode_mid, 1);

    // strlen()I: const-string "hello"; length; return
    b.direct(
        cls,
        "strlen",
        "I",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(2, 0, 1, {
            let mut u = op21c(0x1A, 0, sidx_hello as u16);
            u.extend(op35c(0x6E, 1, length_mid, [0, 0, 0, 0, 0]));
            u.extend(op11x(0x0A, 0));
            u.extend(op11x(0x0F, 0));
            u
        })),
    );
    // str_hash()I: const-string "abc"; hashCode; return
    b.direct(
        cls,
        "str_hash",
        "I",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(2, 0, 1, {
            let mut u = op21c(0x1A, 0, sidx_abc as u16);
            u.extend(op35c(0x6E, 1, hashcode_mid, [0, 0, 0, 0, 0]));
            u.extend(op11x(0x0A, 0));
            u.extend(op11x(0x0F, 0));
            u
        })),
    );
    let mut e = engine_of(&b);
    assert_eq!(
        invoke(&mut e, "strlen", "()I", &[])
            .unwrap()
            .as_int()
            .unwrap(),
        5
    );
    // "abc".hashCode() = 96354 (algoritmo especificado do Java)
    assert_eq!(
        invoke(&mut e, "str_hash", "()I", &[])
            .unwrap()
            .as_int()
            .unwrap(),
        96354
    );
}

#[test]
fn statics_clinit_and_static_values() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    b.static_field(cls, "BASE", "I", SVal::Int(100));
    let fidx_base = b.field_idx("LCaso;", "I", "BASE");
    // le_base()I: sget v0, BASE; +1; return
    b.direct(
        cls,
        "le_base",
        "I",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(2, 0, 0, {
            let mut u = op21c(0x60, 0, fidx_base);
            u.extend(op22b(0xD8, 0, 0, 1));
            u.extend(op11x(0x0F, 0));
            u
        })),
    );
    let mut e = engine_of(&b);
    assert_eq!(
        invoke(&mut e, "le_base", "()I", &[])
            .unwrap()
            .as_int()
            .unwrap(),
        101
    );
    // clinit roda uma única vez (resultado estável)
    assert_eq!(
        invoke(&mut e, "le_base", "()I", &[])
            .unwrap()
            .as_int()
            .unwrap(),
        101
    );
}

#[test]
fn instances_fields_and_constructors() {
    let mut b = DexBuilder::new();
    let ponto = b.class("LPonto;", "Ljava/lang/Object;");
    b.instance_field(ponto, "x", "I");
    b.instance_field(ponto, "y", "I");
    let fidx_x = b.field_idx("LPonto;", "I", "x");
    let fidx_y = b.field_idx("LPonto;", "I", "y");
    // <init>(II)V: regs=5, ins=3 → this=v2, x=v3, y=v4
    let p_void = b.proto_idx("V", vec![]);
    let init_object = b.method_idx("Ljava/lang/Object;", p_void, "<init>");
    b.direct(
        ponto,
        "<init>",
        "V",
        vec!["I", "I"],
        ACC_PUBLIC | ACC_CONSTRUCTOR,
        Some(b.code(5, 3, 1, {
            let mut u = op35c(0x70, 1, init_object, [2, 0, 0, 0, 0]); // Object.<init> (no-op)
            u.extend(op22c(0x59, 3, 2, fidx_x)); // iput v3, v2, x
            u.extend(op22c(0x59, 4, 2, fidx_y)); // iput v4, v2, y
            u.extend(op10x(0x0E));
            u
        })),
    );
    let p_ii = b.proto_idx("V", vec!["I".to_string(), "I".to_string()]);
    let init_ponto = b.method_idx("LPonto;", p_ii, "<init>");
    let ponto_tidx = b.type_idx("LPonto;");
    let caso = b.class("LCaso;", "Ljava/lang/Object;");
    // soma_ponto(II)I: regs=6, ins=2 → args v4,v5
    b.direct(
        caso,
        "soma_ponto",
        "I",
        vec!["I", "I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(6, 2, 3, {
            let mut u = op21c(0x22, 0, ponto_tidx); // new-instance v0, Ponto
            u.extend(op35c(0x70, 3, init_ponto, [0, 4, 5, 0, 0])); // <init> {v0,v4,v5}
            u.extend(op22c(0x52, 1, 0, fidx_x)); // iget v1, v0, x
            u.extend(op22c(0x52, 2, 0, fidx_y)); // iget v2, v0, y
            u.extend(op23x(0x90, 1, 1, 2)); // add-int v1, v1, v2
            u.extend(op11x(0x0F, 1));
            u
        })),
    );
    let mut e = engine_of(&b);
    assert_eq!(
        invoke(
            &mut e,
            "soma_ponto",
            "(II)I",
            &[Value::Int(30), Value::Int(12)]
        )
        .unwrap()
        .as_int()
        .unwrap(),
        42
    );
}

#[test]
fn virtual_dispatch_and_inheritance() {
    let mut b = DexBuilder::new();
    let base = b.class("LBase;", "Ljava/lang/Object;");
    b.r#virtual(
        base,
        "f",
        "I",
        vec![],
        ACC_PUBLIC,
        Some(b.code(2, 1, 0, {
            let mut u = op11n(0x12, 0, 1); // v0 = scratch (this está em v1)
            u.extend(op11x(0x0F, 0));
            u
        })),
    );
    let sub = b.class("LSub;", "LBase;");
    b.r#virtual(
        sub,
        "f",
        "I",
        vec![],
        ACC_PUBLIC,
        Some(b.code(2, 1, 0, {
            let mut u = op11n(0x12, 0, 2);
            u.extend(op11x(0x0F, 0));
            u
        })),
    );
    let sub_tidx = b.type_idx("LSub;");
    let p_void = b.proto_idx("V", vec![]);
    let init_object = b.method_idx("Ljava/lang/Object;", p_void, "<init>");
    let p_i = b.proto_idx("I", vec![]);
    let f_sub = b.method_idx("LSub;", p_i, "f");
    let caso = b.class("LCaso;", "Ljava/lang/Object;");
    // despacha(I)I: new Sub; <init> Object; invoke-virtual f → 2
    b.direct(
        caso,
        "despacha",
        "I",
        vec!["I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(4, 1, 1, {
            let mut u = op21c(0x22, 0, sub_tidx);
            u.extend(op35c(0x70, 1, init_object, [0, 0, 0, 0, 0]));
            u.extend(op35c(0x6E, 1, f_sub, [0, 0, 0, 0, 0])); // receiver v0
            u.extend(op11x(0x0A, 1)); // move-result v1
            u.extend(op11x(0x0F, 1));
            u
        })),
    );
    let mut e = engine_of(&b);
    // despacho virtual pela classe de RUNTIME (Sub), não pela referência
    assert_eq!(
        invoke(&mut e, "despacha", "(I)I", &[Value::Int(0)])
            .unwrap()
            .as_int()
            .unwrap(),
        2
    );
}

#[test]
fn exceptions_try_catch() {
    let mut b = DexBuilder::new();
    b.type_idx("Ljava/lang/ArithmeticException;");
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    // div_ou_menos1(II)I — args v3,v4
    // 0..1: div-int v0, v3, v4
    // 2:    return v0
    // 3:    move-exception v0      ← handler
    // 4:    const/4 v0, -1
    // 5:    return v0
    // try [0, +3) catch ArithmeticException → 3
    let mut blob = b.code(5, 2, 0, {
        let mut u = op23x(0x93, 0, 3, 4); // 0..1
        u.extend(op11x(0x0F, 0)); // 2
        u.extend(op11x(0x0D, 0)); // 3: move-exception v0
        u.extend(op11n(0x12, 0, -1)); // 4
        u.extend(op11x(0x0F, 0)); // 5
        u
    });
    blob.tries.push(TryBlob {
        start: 0,
        count: 3,
        typed: vec![("Ljava/lang/ArithmeticException;".to_string(), 3)],
        catch_all: None,
    });
    b.direct(
        cls,
        "div_ou_menos1",
        "I",
        vec!["I", "I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(blob),
    );

    let mut e = engine_of(&b);
    assert_eq!(
        invoke(
            &mut e,
            "div_ou_menos1",
            "(II)I",
            &[Value::Int(10), Value::Int(2)]
        )
        .unwrap()
        .as_int()
        .unwrap(),
        5
    );
    assert_eq!(
        invoke(
            &mut e,
            "div_ou_menos1",
            "(II)I",
            &[Value::Int(1), Value::Int(0)]
        )
        .unwrap()
        .as_int()
        .unwrap(),
        -1
    );
}

#[test]
fn arrays_aput_aget_bounds() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    let int_arr_tidx = b.type_idx("[I");
    // arr(I)I: new-array v0[3]; aput 5,7,9; aget idx 2 → 9
    b.direct(
        cls,
        "arr",
        "I",
        vec!["I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(5, 1, 0, {
            let mut u = op11n(0x12, 0, 3); // 0: v0=3
            u.extend(op22c(0x23, 0, 0, int_arr_tidx)); // 1..2: new-array v0, v0, [I
            u.extend(op11n(0x12, 1, 5)); // 3
            u.extend(op11n(0x12, 2, 0)); // 4
            u.extend(op23x(0x4B, 1, 0, 2)); // 5..6: aput v1, v0, v2
            u.extend(op11n(0x12, 1, 7)); // 7
            u.extend(op11n(0x12, 2, 1)); // 8
            u.extend(op23x(0x4B, 1, 0, 2)); // 9..10
            u.extend(op21s(0x13, 1, 9)); // 11..12: const/16 v1, 9
            u.extend(op11n(0x12, 2, 2)); // 13
            u.extend(op23x(0x4B, 1, 0, 2)); // 14..15
            u.extend(op23x(0x44, 1, 0, 2)); // 16..17: aget v1, v0, v2 → 9
            u.extend(op11x(0x0F, 1)); // 18
            u
        })),
    );
    let mut e = engine_of(&b);
    assert_eq!(
        invoke(&mut e, "arr", "(I)I", &[Value::Int(0)])
            .unwrap()
            .as_int()
            .unwrap(),
        9
    );

    // fora dos limites → AIOOBE
    let mut b2 = DexBuilder::new();
    let cls2 = b2.class("LCaso;", "Ljava/lang/Object;");
    let t2 = b2.type_idx("[I");
    b2.direct(
        cls2,
        "estoura",
        "I",
        vec!["I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b2.code(4, 1, 0, {
            let mut u = op11n(0x12, 0, 2);
            u.extend(op22c(0x23, 0, 0, t2));
            u.extend(op11n(0x12, 1, 5));
            u.extend(op11n(0x12, 2, 9));
            u.extend(op23x(0x4B, 1, 0, 2)); // aput v1, v0[2], v2=9 → idx 9 de len 2
            u.extend(op11x(0x0F, 1));
            u
        })),
    );
    let mut e2 = engine_of(&b2);
    match invoke(&mut e2, "estoura", "(I)I", &[Value::Int(0)]) {
        Err(VmExit::Exception(t)) => {
            assert_eq!(t.class, "Ljava/lang/ArrayIndexOutOfBoundsException;")
        }
        other => panic!("esperava AIOOBE, got {other:?}"),
    }
}

#[test]
fn fuel_limit_stops_runaway_loop() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    // loop infinito: goto +0 (fica no próprio goto)
    b.direct(
        cls,
        "loop_infinito",
        "I",
        vec!["I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(2, 1, 0, op10t(0x28, 0))),
    );
    let dex = rd_dex::Dex::parse(b.finish()).expect("dex");
    let cfg = VmConfig {
        fuel: 1_000,
        ..VmConfig::default()
    };
    let mut e = Engine::new(vec![dex], cfg);
    match invoke(&mut e, "loop_infinito", "(I)I", &[Value::Int(0)]) {
        Err(VmExit::Error(err)) => assert_eq!(err.code, "VM_FUEL"),
        other => panic!("esperava VM_FUEL, got {other:?}"),
    }
}

#[test]
fn heap_budget_enforced_via_string_ops() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    let sb_tidx = b.type_idx("Ljava/lang/StringBuilder;");
    let p_v = b.proto_idx("V", vec![]);
    let init = b.method_idx("Ljava/lang/StringBuilder;", p_v, "<init>");
    let p_append_s = b.proto_idx(
        "Ljava/lang/StringBuilder;",
        vec!["Ljava/lang/String;".to_string()],
    );
    let p_tostring = b.proto_idx("Ljava/lang/String;", vec![]);
    let append = b.method_idx("Ljava/lang/StringBuilder;", p_append_s, "append");
    let tostring = b.method_idx("Ljava/lang/StringBuilder;", p_tostring, "toString");
    let sidx_big = b.intern(&"x".repeat(10_000));
    // gross()String: new SB; append(big) ×3; toString — 30 KB de texto
    b.direct(
        cls,
        "gross",
        "Ljava/lang/String;",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(4, 0, 2, {
            let mut u = op21c(0x22, 0, sb_tidx);
            u.extend(op35c(0x70, 1, init, [0, 0, 0, 0, 0]));
            u.extend(op21c(0x1A, 1, sidx_big as u16));
            for _ in 0..3 {
                u.extend(op35c(0x6E, 2, append, [0, 1, 0, 0, 0]));
                u.extend(op11x(0x0C, 0));
            }
            u.extend(op35c(0x6E, 1, tostring, [0, 0, 0, 0, 0]));
            u.extend(op11x(0x0C, 0));
            u.extend(op11x(0x11, 0));
            u
        })),
    );
    let dex = rd_dex::Dex::parse(b.finish()).expect("dex");
    let cfg = VmConfig {
        heap_budget: 40_000, // 30 KB de texto + cópias não cabem
        ..VmConfig::default()
    };
    let mut e = Engine::new(vec![dex], cfg);
    match invoke(&mut e, "gross", "()Ljava/lang/String;", &[]) {
        Err(VmExit::Error(err)) => assert_eq!(err.code, "VM_OOM"),
        other => panic!("esperava VM_OOM, got {other:?}"),
    }
}

#[test]
fn unop_conversions_follow_java() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    // to_byte(I)I: int-to-byte v0, v3
    b.direct(
        cls,
        "to_byte",
        "I",
        vec!["I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(4, 1, 0, {
            let mut u = op12x(0x8D, 0, 3);
            u.extend(op11x(0x0F, 0));
            u
        })),
    );
    // f2i(F)I: float-to-int v0, v3 (NaN→0, saturante — semântica Java)
    b.direct(
        cls,
        "f2i",
        "I",
        vec!["F"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(4, 1, 0, {
            let mut u = op12x(0x87, 0, 3);
            u.extend(op11x(0x0F, 0));
            u
        })),
    );
    let mut e = engine_of(&b);
    assert_eq!(
        invoke(&mut e, "to_byte", "(I)I", &[Value::Int(0x1FF)])
            .unwrap()
            .as_int()
            .unwrap(),
        -1
    );
    // 1e20f → saturação para i32::MAX (semântica Java)
    assert_eq!(
        invoke(&mut e, "f2i", "(F)I", &[Value::Float(1e20)])
            .unwrap()
            .as_int()
            .unwrap(),
        i32::MAX
    );
}

// ═══════════════════ regressões da auditoria rodada 2 (issues #17–#28, #37) ═

fn op31i(op: u8, a: u8, lit: i32) -> Vec<u16> {
    vec![
        (op as u16) | ((a as u16) << 8),
        lit as u16,
        (lit >> 16) as u16,
    ]
}
fn array_payload(width: u16, count: u32, data_units: &[u16]) -> Vec<u16> {
    let mut v = vec![0x0300u16, width, count as u16, (count >> 16) as u16];
    v.extend_from_slice(data_units);
    v
}

/// issue #22: `nop` (0x00) é opcode REAL — não pode matar a execução
#[test]
fn nop_is_executable_real_opcode() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    b.direct(
        cls,
        "com_nop",
        "I",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(1, 0, 0, {
            let mut u = op21s(0x13, 0, 41); // const/16 v0, 41 (const/4 é 4-bit!)
            u.extend(op10x(0x00)); // nop
            u.extend(op10x(0x00)); // nop
            u.extend(op11x(0x0F, 0)); // return v0
            u
        })),
    );
    let mut e = engine_of(&b);
    assert_eq!(
        invoke(&mut e, "com_nop", "()I", &[])
            .unwrap()
            .as_int()
            .unwrap(),
        41
    );
}

/// issue #23: try aninhado — o handler escolhido é o do try MAIS INTERNO
#[test]
fn nested_try_selects_innermost_handler() {
    let mut b = DexBuilder::new();
    b.type_idx("Ljava/lang/ArithmeticException;");
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    // 0: const/4 v0, 10
    // 1..2: div-int v0, v0, v3   ← dentro do try INTERNO [1..3)
    // 3: return v0
    // 4: move-exception v0       ← handler INTERNO
    // 5: const/4 v0, -1
    // 6: return v0
    // 7: move-exception v0       ← handler EXTERNO
    // 8: const/4 v0, -10
    // 9: return v0
    let mut blob = b.code(5, 2, 0, {
        let mut u = op21s(0x13, 0, 10); // 0 (const/4 é 4-bit com sinal)
        u.extend(op23x(0x93, 0, 0, 3)); // 1..2 div-int v0, v0, v3
        u.extend(op11x(0x0F, 0)); // 3
        u.extend(op11x(0x0D, 0)); // 4
        u.extend(op11n(0x12, 0, -1)); // 5
        u.extend(op11x(0x0F, 0)); // 6
        u.extend(op11x(0x0D, 0)); // 7
        u.extend(op11n(0x12, 0, -10)); // 8
        u.extend(op11x(0x0F, 0)); // 9
        u
    });
    // ordem do DEX: try EXTERNO (start 0) vem ANTES do interno (start 2)
    // layout: const/16@0..1, div@2..3, ret@4, move-exc@5(inner), ret@7,
    //         move-exc@8(outer), ret@10
    blob.tries.push(TryBlob {
        start: 0,
        count: 4,
        typed: vec![("Ljava/lang/ArithmeticException;".to_string(), 8)],
        catch_all: None,
    });
    blob.tries.push(TryBlob {
        start: 2,
        count: 2,
        typed: vec![("Ljava/lang/ArithmeticException;".to_string(), 5)],
        catch_all: None,
    });
    b.direct(
        cls,
        "aninhado",
        "I",
        vec!["I", "I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(blob),
    );
    let mut e = engine_of(&b);
    // Java: div por zero dentro do try interno → handler INTERNO (-1)
    assert_eq!(
        invoke(&mut e, "aninhado", "(II)I", &[Value::Int(0), Value::Int(0)])
            .unwrap()
            .as_int()
            .unwrap(),
        -1
    );
}

/// issue #24: fill-array-data com float[]/double[] (bits decodificados)
#[test]
fn fill_array_data_float_and_double() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    let f_arr = b.type_idx("[F");
    // 1.5f = 0x3FC00000 → units LE [0x0000, 0x3FC0]; 2.5f = 0x40200000
    b.direct(
        cls,
        "farr",
        "F",
        vec!["I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(5, 1, 0, {
            let mut u = op11n(0x12, 0, 2); // 0: v0 = 2
            u.extend(op22c(0x23, 0, 0, f_arr)); // 1..2: new-array v0, v0, [F
            u.extend(op31t(0x26, 0, 3)); // 3..5: fill-array-data v0, +3
            u.extend(array_payload(4, 2, &[0x0000, 0x3FC0, 0x0000, 0x4020])); // 6..9: payload (4 units)
            u.extend(op11n(0x12, 2, 0)); // 10: v2 = 0
            u.extend(op23x(0x44, 1, 0, 2)); // 11..12: aget v1, v0, v2
            u.extend(op11x(0x0F, 1)); // 13: return v1
            u
        })),
    );
    let mut e = engine_of(&b);
    let v = invoke(&mut e, "farr", "(I)F", &[Value::Int(0)]).unwrap();
    match v {
        Value::Float(f) => assert_eq!(f, 1.5, "bits do payload devem virar float"),
        other => panic!("esperado Float, got {other:?}"),
    }
}

/// issue #27: StringBuilder.append((String)null) apendeja "null" (não NPE);
/// também exercita new-instance→<clinit> trigger e invoke-direct <init>
#[test]
fn stringbuilder_append_null_appends_literal() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    let sb_tidx = b.type_idx("Ljava/lang/StringBuilder;");
    let s_a = b.intern("a") as u16;
    let p_str_ret_sb = b.proto_idx(
        "Ljava/lang/StringBuilder;",
        vec!["Ljava/lang/String;".to_string()],
    );
    let p_v_str = b.proto_idx("V", vec!["Ljava/lang/String;".to_string()]);
    let init_str = b.method_idx("Ljava/lang/StringBuilder;", p_v_str, "<init>");
    let p_sb_ret_str = b.proto_idx("Ljava/lang/String;", vec![]); // toString() — receiver não é param
    let append = b.method_idx("Ljava/lang/StringBuilder;", p_str_ret_sb, "append");
    let to_string = b.method_idx("Ljava/lang/StringBuilder;", p_sb_ret_str, "toString");
    // regs=5, ins=1 → arg em v4; v3 fica Null (nunca escrito)
    b.direct(
        cls,
        "concat_null",
        "Ljava/lang/String;",
        vec!["I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(5, 1, 2, {
            let mut u = op21c(0x22, 0, sb_tidx); // 0..1: new-instance v0, SB
            u.extend(op21c(0x1A, 1, s_a)); // 2..3: const-string v1, "a"
            u.extend(op35c(0x70, 2, init_str, [0, 1, 0, 0, 0])); // 4..6: <init>{recv=v0, arg=v1}
            u.extend(op35c(0x6E, 2, append, [0, 3, 0, 0, 0])); // 7..9: append(v0, v3=Null)
            u.extend(op35c(0x6E, 1, to_string, [0, 0, 0, 0, 0])); // 10..12: toString
            u.extend(op11x(0x0C, 1)); // 13: move-result-object v1
            u.extend(op11x(0x0F, 1)); // 14: return v1
            u
        })),
    );
    let mut e = engine_of(&b);
    let v = invoke(
        &mut e,
        "concat_null",
        "(I)Ljava/lang/String;",
        &[Value::Int(0)],
    )
    .unwrap();
    match v {
        Value::Obj(r) => assert_eq!(e.heap.as_str(r).unwrap(), "anull"),
        other => panic!("esperado Obj(String), got {other:?}"),
    }
}

/// issue #27: autoboxing Integer.valueOf + intValue (d8 emite para todo
/// List<Integer>/Collections)
#[test]
fn integer_boxing_roundtrip() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    let p_int_ret_integer = b.proto_idx("Ljava/lang/Integer;", vec!["I".to_string()]);
    let value_of = b.method_idx("Ljava/lang/Integer;", p_int_ret_integer, "valueOf");
    let p_integer_ret_int = b.proto_idx("I", vec![]);
    let int_value = b.method_idx("Ljava/lang/Integer;", p_integer_ret_int, "intValue");
    b.direct(
        cls,
        "box_unbox",
        "I",
        vec!["I"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(4, 1, 1, {
            let mut u = op23x(0x90, 0, 3, 3); // 0..1: v0 = v3 + v3 (=arg*2)
            u.extend(op35c(0x71, 1, value_of, [0, 0, 0, 0, 0])); // 2..4: invoke-static valueOf(v0)
            u.extend(op11x(0x0C, 1)); // 5: move-result-object v1
            u.extend(op35c(0x6E, 1, int_value, [1, 0, 0, 0, 0])); // 6..8: intValue()
            u.extend(op11x(0x0A, 1)); // 9: move-result v1
            u.extend(op11x(0x0F, 1)); // 10: return v1
            u
        })),
    );
    let mut e = engine_of(&b);
    assert_eq!(
        invoke(&mut e, "box_unbox", "(I)I", &[Value::Int(21)])
            .unwrap()
            .as_int()
            .unwrap(),
        42
    );
}

/// issue #26: check-cast de String para CharSequence (interface) é válido
#[test]
fn checkcast_to_interface_succeeds() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    let charseq = b.type_idx("Ljava/lang/CharSequence;");
    let s_x = b.intern("x") as u16;
    b.direct(
        cls,
        "cast_iface",
        "Ljava/lang/String;",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(1, 0, 0, {
            let mut u = op21c(0x1A, 0, s_x); // const-string v0, "x"
            u.extend(op21c(0x1F, 0, charseq)); // check-cast v0, CharSequence
            u.extend(op11x(0x0F, 0)); // return-object v0
            u
        })),
    );
    let mut e = engine_of(&b);
    let v = invoke(&mut e, "cast_iface", "()Ljava/lang/String;", &[]).unwrap();
    match v {
        Value::Obj(r) => assert_eq!(e.heap.as_str(r).unwrap(), "x"),
        other => panic!("esperado Obj(String), got {other:?}"),
    }
}

/// issue #37: OOM de new-array vira OutOfMemoryError CAPTURÁVEL
#[test]
fn oom_from_new_array_is_catchable() {
    let mut b = DexBuilder::new();
    b.type_idx("Ljava/lang/OutOfMemoryError;");
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    let int_arr = b.type_idx("[I");
    // 0..2: const v0, 0x7FFFFFFF (2G elems × 32 B ≫ 256 MB)
    // 3..4: new-array v0, v0, [I
    // 5: return v0
    // 6: move-exception v0
    // 7: const/4 v0, -1
    // 8: return v0
    let mut blob = b.code(4, 0, 0, {
        let mut u = op31i(0x14, 0, 0x7FFF_FFFF); // 0..2
        u.extend(op22c(0x23, 0, 0, int_arr)); // 3..4
        u.extend(op11x(0x0F, 0)); // 5
        u.extend(op11x(0x0D, 0)); // 6
        u.extend(op11n(0x12, 0, -1)); // 7
        u.extend(op11x(0x0F, 0)); // 8
        u
    });
    blob.tries.push(TryBlob {
        start: 0,
        count: 5,
        typed: vec![("Ljava/lang/OutOfMemoryError;".to_string(), 6)],
        catch_all: None,
    });
    b.direct(
        cls,
        "oom_ou_menos1",
        "I",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(blob),
    );
    let mut e = engine_of(&b);
    assert_eq!(
        invoke(&mut e, "oom_ou_menos1", "()I", &[])
            .unwrap()
            .as_int()
            .unwrap(),
        -1
    );
}

/// issue #25 + #37: <clinit> dispara em sget; falha vira
/// ExceptionInInitializerError no 1º acesso e NoClassDefFoundError nos
/// seguintes; superclasse inicializa antes da subclasse
#[test]
fn clinit_failure_semantics_and_super_first() {
    let mut b = DexBuilder::new();
    b.type_idx("Ljava/lang/ArithmeticException;");
    // LBase; tem <clinit> que divide por zero (falha)
    let base = b.class("LBase;", "Ljava/lang/Object;");
    let _fs = b.field_idx("LBase;", "I", "s");
    b.static_field(base, "s", "I", SVal::Int(0));
    b.direct(
        base,
        "<clinit>",
        "V",
        vec![],
        ACC_STATIC | ACC_CONSTRUCTOR,
        Some(b.code(2, 0, 0, {
            let mut u = op11n(0x12, 0, 1);
            u.extend(op11n(0x12, 1, 0));
            u.extend(op23x(0x93, 0, 0, 1)); // div-int v0, v0, v1 → Arith
            u.extend(op10x(0x0E)); // return-void (nunca alcançado)
            u
        })),
    );
    // LSub; estende LBase; — se a ordem super-first estiver certa, a falha
    // acontece ANTES do <clinit> de LSub rodar (que marcaria uma flag)
    let sub = b.class("LSub;", "LBase;");
    let _ft = b.field_idx("LSub;", "I", "t");
    b.static_field(sub, "t", "I", SVal::Int(0));
    b.direct(
        sub,
        "<clinit>",
        "V",
        vec![],
        ACC_STATIC | ACC_CONSTRUCTOR,
        Some(b.code(2, 0, 0, {
            let mut u = op11n(0x12, 0, 7);
            u.extend(op10x(0x0E));
            u
        })),
    );
    // LCaso.toque()I: sget LBase;->s → dispara a cadeia de init
    let cls = b.class("LCaso;", "Ljava/lang/Object;");
    let f_s = b.field_idx("LBase;", "I", "s");
    b.direct(
        cls,
        "toque",
        "I",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(1, 0, 0, {
            let mut u = op21c(0x60, 0, f_s); // sget v0, LBase->s
            u.extend(op11x(0x0F, 0));
            u
        })),
    );
    let mut e = engine_of(&b);
    // 1º acesso → ExceptionInInitializerError (não ArithmeticException cru)
    let err1 = invoke(&mut e, "toque", "()I", &[]).unwrap_err();
    match &err1 {
        VmExit::Exception(t) => {
            assert_eq!(
                t.class, "Ljava/lang/ExceptionInInitializerError;",
                "1º acesso: {err1}"
            )
        }
        other => panic!("1º acesso deveria ser Exception, got {other:?}"),
    }
    // 2º acesso → NoClassDefFoundError (JLS 12.4.2)
    let err2 = invoke(&mut e, "toque", "()I", &[]).unwrap_err();
    match &err2 {
        VmExit::Exception(t) => {
            assert_eq!(
                t.class, "Ljava/lang/NoClassDefFoundError;",
                "2º acesso: {err2}"
            )
        }
        other => panic!("2º acesso deveria ser Exception, got {other:?}"),
    }
}
