//! Classpath da VM: resolução de classes/métodos/campos/protos sobre um ou
//! mais DEX (APK real = classes.dex/classes2.dex/…). Cache de `CodeItem`
//! parseado por offset (loops invocam o mesmo método repetidamente).

use std::collections::HashMap;
use std::sync::Arc;

use rd_dex::classes::{ClassData, ClassDef};
use rd_dex::code::CodeItem;
use rd_dex::{Dex, RdError, RdResult};

/// Descritor de método em formato Java (`(II)I`, `(Ljava/lang/String;)V`).
pub fn proto_descriptor(dex: &Dex, proto_idx: u16) -> RdResult<String> {
    let proto = dex.proto(proto_idx)?;
    let params = dex.proto_params(proto_idx)?;
    let mut out = String::from("(");
    for p in params {
        out.push_str(dex.type_str(p));
    }
    out.push(')');
    out.push_str(dex.type_str(proto.return_type_idx));
    Ok(out)
}

/// Uma entrada de method_id resolvida.
#[derive(Debug, Clone)]
pub struct MethodRef {
    /// descritor da classe declarante (`Lpkg/Cls;`)
    pub class: String,
    pub name: String,
    pub proto: String,
    /// descritores dos parâmetros em ordem (para largura de registradores)
    pub param_types: Vec<String>,
    pub return_type: String,
}

#[derive(Debug, Default)]
pub struct Classpath {
    pub dexes: Vec<Dex>,
    /// (índice do dex, offset do code_item) → parse em cache
    code_cache: HashMap<(usize, usize), Arc<CodeItem>>,
    /// (índice do dex, class_idx) → class_data em cache
    class_data_cache: HashMap<(usize, u32), Option<Arc<ClassData>>>,
}

impl Classpath {
    pub fn new(dexes: Vec<Dex>) -> Self {
        Classpath {
            dexes,
            code_cache: HashMap::new(),
            class_data_cache: HashMap::new(),
        }
    }

    /// method_id cru (class/name/idx).
    pub fn method_ref(&self, dex_idx: usize, idx: u32) -> RdResult<MethodRef> {
        let dex = &self.dexes[dex_idx];
        let m = dex.method(idx)?;
        let proto = proto_descriptor(dex, m.proto_idx)?;
        let params = dex.proto_params(m.proto_idx)?;
        let param_types = params
            .iter()
            .map(|p| dex.type_str(*p).to_string())
            .collect();
        let ret = dex
            .type_str(dex.proto(m.proto_idx)?.return_type_idx)
            .to_string();
        Ok(MethodRef {
            class: dex.type_str(m.class_idx as u32).to_string(),
            name: dex.string(m.name_idx).to_string(),
            proto,
            param_types,
            return_type: ret,
        })
    }

    /// field_id cru → (classe, nome, tipo).
    pub fn field_ref(&self, dex_idx: usize, idx: u32) -> RdResult<(String, String, String)> {
        let dex = &self.dexes[dex_idx];
        let f = dex.field(idx)?;
        Ok((
            dex.type_str(f.class_idx as u32).to_string(),
            dex.string(f.name_idx).to_string(),
            dex.type_str(f.type_idx as u32).to_string(),
        ))
    }

    /// string id (com fallback vazio tolerante — Lei 1).
    pub fn string(&self, dex_idx: usize, idx: u32) -> String {
        self.dexes[dex_idx].string(idx).to_string()
    }

    pub fn type_str(&self, dex_idx: usize, idx: u32) -> String {
        self.dexes[dex_idx].type_str(idx).to_string()
    }

    /// Procura a classe por descritor em todos os dex → (dex_idx, def).
    pub fn find_class(&self, descriptor: &str) -> Option<(usize, &ClassDef)> {
        for (i, dex) in self.dexes.iter().enumerate() {
            if let Some(def) = dex.find_class(descriptor) {
                return Some((i, def));
            }
        }
        None
    }

    /// class_data em cache.
    pub fn class_data(
        &mut self,
        dex_idx: usize,
        def: &ClassDef,
    ) -> RdResult<Option<Arc<ClassData>>> {
        let key = (dex_idx, def.class_idx);
        if let Some(cached) = self.class_data_cache.get(&key) {
            return Ok(cached.clone());
        }
        let parsed = self.dexes[dex_idx].class_data(def)?;
        let arc = parsed.map(Arc::new);
        self.class_data_cache.insert(key, arc.clone());
        Ok(arc)
    }

    /// code_item em cache.
    pub fn code(&mut self, dex_idx: usize, code_off: u32) -> RdResult<Option<Arc<CodeItem>>> {
        if code_off == 0 {
            return Ok(None);
        }
        let key = (dex_idx, code_off as usize);
        if let Some(cached) = self.code_cache.get(&key) {
            return Ok(Some(cached.clone()));
        }
        let parsed = self.dexes[dex_idx].code(code_off)?;
        match parsed {
            Some(ci) => {
                let arc = Arc::new(ci);
                self.code_cache.insert(key, arc.clone());
                Ok(Some(arc))
            }
            None => Ok(None),
        }
    }

    /// Resolve um método por (classe, nome, proto) caminhando a superclasse.
    /// Retorna (dex_idx, class_def_idx, método, classe_que_declara).
    pub fn resolve_method(
        &mut self,
        class_desc: &str,
        name: &str,
        proto: &str,
    ) -> Option<(usize, ClassDef, rd_dex::methods::EncodedMethod)> {
        let mut cur = class_desc.to_string();
        for _ in 0..16 {
            let (dex_idx, def) = self.find_class(&cur)?;
            let def = *def;
            let data = self.class_data(dex_idx, &def).ok().flatten()?;
            let found = data
                .direct_methods
                .iter()
                .chain(&data.virtual_methods)
                .find(|m| {
                    m.method_idx != rd_dex::strings::NO_INDEX
                        && self.method_name(dex_idx, m.method_idx) == Some(name.to_string())
                        && self.method_proto(dex_idx, m.method_idx).as_deref() == Some(proto)
                })
                .cloned();
            if let Some(m) = found {
                return Some((dex_idx, def, m));
            }
            // sobe para a superclasse (dentro do mesmo conjunto de dex)
            let super_desc = self.dexes[dex_idx].type_str(def.superclass_idx).to_string();
            if super_desc.is_empty() || super_desc == cur {
                return None;
            }
            cur = super_desc;
        }
        None
    }

    fn method_name(&self, dex_idx: usize, idx: u32) -> Option<String> {
        let d = self.dexes.get(dex_idx)?;
        let m = d.method(idx).ok()?;
        Some(d.string(m.name_idx).to_string())
    }

    fn method_proto(&self, dex_idx: usize, idx: u32) -> Option<String> {
        let d = self.dexes.get(dex_idx)?;
        let m = d.method(idx).ok()?;
        proto_descriptor(d, m.proto_idx).ok()
    }

    /// Superclasse de um descritor de classe ("" se nenhuma/raiz).
    pub fn superclass_of(&self, class_desc: &str) -> Option<String> {
        for dex in &self.dexes {
            if let Some(def) = dex.find_class(class_desc) {
                let s = dex.type_str(def.superclass_idx);
                if s.is_empty() || s == "?" {
                    return None;
                }
                return Some(s.to_string());
            }
        }
        None
    }

    /// `class A <: B` caminhando supers (usado por instanceof/catch/check-cast).
    /// String é subtipo de CharSequence/Object/String hierarquia mínima embutida.
    pub fn is_subtype(&self, sub: &str, sup: &str) -> bool {
        if sub == sup {
            return true;
        }
        // hierarquia mínima da plataforma conhecida pela VM (classes que não
        // estão no DEX mas fazem parte do contrato do interpretador M2)
        if let Some(supers) = builtin_hierarchy(sub) {
            if supers.contains(&sup) {
                return true;
            }
        }
        let mut cur = sub.to_string();
        for _ in 0..16 {
            let Some(next) = self.superclass_of(&cur) else {
                return false;
            };
            if next == sup {
                return true;
            }
            if let Some(supers) = builtin_hierarchy(&next) {
                if supers.contains(&sup) {
                    return true;
                }
            }
            cur = next;
        }
        false
    }

    /// Offset do call_site_item N (para invokedynamic/string concat).
    pub fn call_site_offset(&self, dex_idx: usize, n: usize) -> Option<u32> {
        self.dexes[dex_idx].call_site_offsets.get(n).copied()
    }
}

/// Hierarquia embutida mínima para classes de plataforma que a VM conhece sem
/// DEX (catch/instanceof/ hierarquia de exceções).
fn builtin_hierarchy(class: &str) -> Option<Vec<&'static str>> {
    const EXC: &[&str] = &[
        "Ljava/lang/Exception;",
        "Ljava/lang/Throwable;",
        "Ljava/lang/Object;",
    ];
    const ERR: &[&str] = &[
        "Ljava/lang/Error;",
        "Ljava/lang/Throwable;",
        "Ljava/lang/Object;",
    ];
    const RUNTIME: &[&str] = &[
        "Ljava/lang/RuntimeException;",
        "Ljava/lang/Exception;",
        "Ljava/lang/Throwable;",
        "Ljava/lang/Object;",
    ];
    match class {
        "Ljava/lang/String;" => Some(vec!["Ljava/lang/Object;"]),
        "Ljava/lang/StringBuilder;" => Some(vec!["Ljava/lang/Object;"]),
        "Ljava/lang/ArithmeticException;" => Some(RUNTIME.to_vec()),
        "Ljava/lang/NullPointerException;" => Some(RUNTIME.to_vec()),
        "Ljava/lang/ClassCastException;" => Some(RUNTIME.to_vec()),
        "Ljava/lang/ArrayIndexOutOfBoundsException;" => Some(RUNTIME.to_vec()),
        "Ljava/lang/NegativeArraySizeException;" => Some(RUNTIME.to_vec()),
        "Ljava/lang/IllegalStateException;" => Some(RUNTIME.to_vec()),
        "Ljava/lang/IllegalArgumentException;" => Some(RUNTIME.to_vec()),
        "Ljava/lang/RuntimeException;" => Some(EXC.to_vec()),
        "Ljava/lang/UnsupportedOperationException;" => Some(RUNTIME.to_vec()),
        "Ljava/lang/StackOverflowError;" => Some(ERR.to_vec()),
        "Ljava/lang/OutOfMemoryError;" => Some(ERR.to_vec()),
        "Ljava/lang/Throwable;" => Some(vec!["Ljava/lang/Object;"]),
        "Ljava/lang/Exception;" => Some(EXC.to_vec()),
        "Ljava/lang/Error;" => Some(ERR.to_vec()),
        _ => None,
    }
}

/// Erro de resolução com sugestão (Lei 1 — nunca silencioso).
pub fn unresolved_method(class: &str, name: &str, proto: &str) -> RdError {
    crate::err::with_suggestion(
        crate::err::vm_error(
            "NOT_FOUND",
            format!("método {class}->{name}{proto} não resolvido"),
        ),
        "a classe pode depender de plataforma fora do escopo M2",
    )
}
