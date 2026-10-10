//! Motor da VM: estado global (classpath, heap arena, campos estáticos),
//! máquina de chamadas, inicialização de classes (`<clinit>`/static_values)
//! e propagação de exceções entre frames.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rd_dex::classes::ClassDef;
use rd_dex::code::CodeItem;
use rd_dex::methods::EncodedMethod;

use crate::classpath::Classpath;
use crate::err::{Throwable, VmExit};
use crate::heap::{Heap, ObjRef};
use crate::interp;
use crate::intrinsics;
use crate::value::Value;

pub const ACC_STATIC: u32 = 0x0008;
pub const ACC_NATIVE: u32 = 0x0100;
pub const ACC_ABSTRACT: u32 = 0x0400;

/// Configuração de execução (orçamentos do piso moto-e5 por default).
#[derive(Debug, Clone)]
pub struct VmConfig {
    /// limite de instruções executadas (contra loop infinito) — VM_FUEL
    pub fuel: u64,
    /// profundidade máxima da pilha de chamadas — VM_STACK_OVERFLOW
    pub max_depth: usize,
    /// heap do app em bytes — 256 MB = piso E5 (PRF-06)
    pub heap_budget: usize,
}

impl Default for VmConfig {
    fn default() -> Self {
        VmConfig {
            fuel: 200_000_000,
            max_depth: 512,
            heap_budget: crate::heap::DEFAULT_HEAP_BUDGET,
        }
    }
}

/// Motor interpretador. Semântica de arena: o heap inteiro é descartado
/// quando o Engine cai (M2: "GC" = arena-por-execução com teto contratual).
pub struct Engine {
    pub cp: Classpath,
    pub heap: Heap,
    pub config: VmConfig,
    /// (classe, campo) → valor
    pub statics: HashMap<(String, String), Value>,
    clinit_done: HashSet<String>,
    clinit_running: HashSet<String>,
    depth: usize,
    pub(crate) fuel_used: u64,
}

impl Engine {
    pub fn new(dexes: Vec<rd_dex::Dex>, config: VmConfig) -> Self {
        let budget = config.heap_budget;
        Engine {
            cp: Classpath::new(dexes),
            heap: Heap::new(budget),
            config,
            statics: HashMap::new(),
            clinit_done: HashSet::new(),
            clinit_running: HashSet::new(),
            depth: 0,
            fuel_used: 0,
        }
    }

    pub fn heap_used(&self) -> usize {
        self.heap.used()
    }

    pub fn fuel_used(&self) -> u64 {
        self.fuel_used
    }

    /// Ponto de entrada público: invoca um método static e devolve o valor.
    /// Exceções Java não capturadas sobem como `VmExit::Exception`.
    pub fn invoke_static(
        &mut self,
        class: &str,
        method: &str,
        sig: &str,
        args: &[Value],
    ) -> Result<Value, VmExit> {
        let param_types = parse_param_types(sig).ok_or_else(|| {
            crate::err::vm_error("INVALID_ARG", format!("assinatura malformada: {sig}"))
        })?;
        if param_types.len() != args.len() {
            return Err(crate::err::vm_error(
                "INVALID_ARG",
                format!(
                    "{sig} pede {} argumentos, recebi {}",
                    param_types.len(),
                    args.len()
                ),
            )
            .into());
        }
        // intrínseco de plataforma (Math/Integer/…) não precisa de DEX
        // (strings placeholder → heap primeiro)
        let args: Vec<Value> = args
            .iter()
            .map(|v| match v {
                Value::StrPlaceholder(s) => {
                    intrinsics::alloc_string(self, s.clone()).map(Value::Obj)
                }
                other => Ok(other.clone()),
            })
            .collect::<Result<Vec<_>, VmExit>>()?;
        if let Some(v) = intrinsics::call_static_intrinsic(self, class, method, sig, &args)? {
            return Ok(v);
        }
        let Some((dex_idx, def, m)) = self.cp.resolve_method(class, method, sig) else {
            return Err(crate::classpath::unresolved_method(class, method, sig).into());
        };
        if m.access_flags & ACC_STATIC == 0 {
            return Err(crate::err::vm_error(
                "INVALID_ARG",
                format!("{class}->{method}{sig} não é static"),
            )
            .into());
        }
        self.ensure_initialized(class)?;
        let slots = slot_form(&args, &param_types);
        self.call(dex_idx, def, &m, slots)
    }

    /// Monta a forma de registradores (wide → par com WideHi).
    pub fn slot_form_public(&self, args: &[Value], param_types: &[String]) -> Vec<Value> {
        slot_form(args, param_types)
    }

    /// Executa um frame completo (loop de interpretador em `interp.rs`).
    pub(super) fn call(
        &mut self,
        dex_idx: usize,
        _def: ClassDef,
        m: &EncodedMethod,
        ins: Vec<Value>,
    ) -> Result<Value, VmExit> {
        if self.depth >= self.config.max_depth {
            return Err(crate::err::vm_error(
                "VM_STACK_OVERFLOW",
                format!(
                    "profundidade de chamadas ≥ {} (limite do motor)",
                    self.config.max_depth
                ),
            )
            .into());
        }
        let Some(code) = self.cp.code(dex_idx, m.code_off)? else {
            if m.access_flags & ACC_NATIVE != 0 {
                return Err(crate::err::not_implemented(format!(
                    "método nativo {}",
                    self.method_label(dex_idx, m)
                ))
                .into());
            }
            return Err(crate::err::vm_error(
                "NOT_FOUND",
                format!(
                    "método {} sem corpo (abstrato?)",
                    self.method_label(dex_idx, m)
                ),
            )
            .into());
        };
        self.depth += 1;
        let ret = interp::exec_frame(self, dex_idx, code, ins);
        self.depth -= 1;
        ret
    }

    pub(super) fn method_label(&self, dex_idx: usize, m: &EncodedMethod) -> String {
        match self.cp.method_ref(dex_idx, m.method_idx) {
            Ok(r) => format!("{}->{}{}", r.class, r.name, r.proto),
            Err(_) => format!("method@{}", m.method_idx),
        }
    }

    // ── inicialização de classe + campos estáticos ──────────────────────────

    /// Garante que a classe passou por static_values + `<clinit>`.
    pub fn ensure_initialized(&mut self, class: &str) -> Result<(), VmExit> {
        if self.clinit_done.contains(class) || self.clinit_running.contains(class) {
            return Ok(());
        }
        self.clinit_running.insert(class.to_string());
        let result = self.initialize_class(class);
        self.clinit_running.remove(class);
        match result {
            Ok(()) => {
                self.clinit_done.insert(class.to_string());
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn initialize_class(&mut self, class: &str) -> Result<(), VmExit> {
        let Some((dex_idx, def)) = self.cp.find_class(class) else {
            return Ok(()); // classe de plataforma embutida: nada a inicializar
        };
        let (dex_idx, def) = (dex_idx, *def);
        // defaults por tipo + static_values por posição
        let static_values = self.cp.dexes[dex_idx].static_values(&def)?;
        let data = self.cp.class_data(dex_idx, &def)?;
        if let Some(cd) = &data {
            for (i, f) in cd.static_fields.iter().enumerate() {
                let (_fclass, fname, ftype) = self.cp.field_ref(dex_idx, f.field_idx)?;
                let v = static_values
                    .get(i)
                    .and_then(|ev| crate::value_from_encoded(ev, &self.cp.dexes[dex_idx]))
                    .unwrap_or_else(|| default_for(&ftype));
                self.statics.insert((class.to_string(), fname), v);
            }
        }
        // <clinit> (se existir)
        if let Some((dex_idx, def, m)) = self.cp.resolve_method(class, "<clinit>", "()V") {
            self.call(dex_idx, def, &m, Vec::new())?;
        }
        Ok(())
    }

    // ── exceções ────────────────────────────────────────────────────────────

    /// Procura handler para a exceção lançada em `at_pc`. Materializa o
    /// Throwable como objeto quando o handler vai usá-lo.
    pub(super) fn enter_handler(
        &mut self,
        dex_idx: usize,
        code: &Arc<CodeItem>,
        at_pc: usize,
        t: Throwable,
    ) -> Result<usize, VmExit> {
        for (i, try_item) in code.tries.iter().enumerate() {
            let start = try_item.start_addr as usize;
            if at_pc < start || at_pc >= start + try_item.insn_count as usize {
                continue;
            }
            let handler = &code.handlers[i];
            for (type_idx, addr) in &handler.typed {
                let catch_desc = self.cp.type_str(dex_idx, *type_idx);
                if catch_desc == t.class || self.cp.is_subtype(&t.class, &catch_desc) {
                    return Ok(*addr as usize);
                }
            }
            if let Some(addr) = handler.catch_all {
                return Ok(addr as usize);
            }
        }
        Err(VmExit::Exception(t))
    }

    /// Materializa um Throwable como instância de heap (para move-exception).
    pub(super) fn materialize_throwable(&mut self, t: &Throwable) -> Result<ObjRef, VmExit> {
        if let Some(obj) = t.obj {
            return Ok(obj);
        }
        let msg = match &t.message {
            Some(m) => Value::Obj(self.heap.alloc_string(m.clone())?),
            None => Value::Null,
        };
        let obj = self
            .heap
            .alloc_instance(t.class.clone(), vec![("message".to_string(), msg)])?;
        Ok(obj)
    }

    /// Cria um Throwable VM-built (ArithmeticException, NPE, …) com objeto.
    pub(super) fn vm_exception(&mut self, class: &str, message: &str) -> Result<Throwable, VmExit> {
        let obj = self.heap.alloc_instance(class.to_string(), Vec::new())?;
        let msg_ref = self.heap.alloc_string(message.to_string())?;
        self.heap.put_field(obj, "message", Value::Obj(msg_ref))?;
        Ok(Throwable {
            class: class.to_string(),
            message: Some(message.to_string()),
            obj: Some(obj),
        })
    }
}

/// Valor default por descritor de campo.
pub fn default_for(desc: &str) -> Value {
    match desc {
        "J" => Value::Long(0),
        "F" => Value::Float(0.0),
        "D" => Value::Double(0.0),
        "L" | "Z" | "B" | "C" | "S" | "I" => Value::Int(0),
        d if d.starts_with('L') || d.starts_with('[') => Value::Null,
        _ => Value::Int(0),
    }
}

/// Converte args tipados para a forma de registradores (wide = 2 slots).
pub fn slot_form(args: &[Value], param_types: &[String]) -> Vec<Value> {
    let mut out = Vec::with_capacity(args.len() + 2);
    for (v, t) in args.iter().zip(param_types) {
        let is_wide = t == "J" || t == "D";
        match (is_wide, v) {
            (true, Value::Long(l)) => {
                out.push(Value::Long(*l));
                out.push(Value::WideHi);
            }
            (true, Value::Double(d)) => {
                out.push(Value::Double(*d));
                out.push(Value::WideHi);
            }
            (_, Value::Long(l)) => {
                // arg passado como long mas param não-wide: trunca (IJ misto é
                // erro do caller; tolerar com truncamento explícito)
                out.push(Value::Int(*l as i32));
            }
            (_, other) => out.push(other.clone()),
        }
    }
    out
}

/// Quebra `(params)ret` em descritores de parâmetro.
pub fn parse_param_types(sig: &str) -> Option<Vec<String>> {
    let rest = sig.strip_prefix('(')?;
    let close = rest.rfind(')')?;
    let params = &rest[..close];
    let mut out = Vec::new();
    let b = params.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let start = i;
        while b[i] == b'[' {
            i += 1;
        }
        if b[i] == b'L' {
            while b[i] != b';' {
                i += 1;
            }
        }
        i += 1;
        out.push(params[start..i].to_string());
    }
    Some(out)
}
