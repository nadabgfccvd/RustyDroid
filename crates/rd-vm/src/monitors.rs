//! Monitores de objeto (`monitor-enter` / `monitor-exit`, opcodes 0x1D/0x1E).
//!
//! Só as entradas de monitores **mantidos** ocupam memória: a tabela mapeia
//! `(objeto, thread)` para a contagem de holds, e uma entrada é removida
//! quando a contagem chega a zero. Objetos sem monitor não pagam nada.
//!
//! Semântica (como na JVM, com um desvio documentado):
//! - `monitor-enter` em `null` lança `NullPointerException` (em `interp.rs`).
//! - `monitor-exit` sem um `monitor-enter` correspondente da mesma thread
//!   lança `IllegalMonitorStateException`.
//! - enters aninhados e balanceados são contados; o monitor libera ao zerar.
//! - **Desvio:** o escalonador verde é cooperativo e não bloqueia no
//!   `monitor-enter` contendido. Uma thread que tenta entrar num monitor
//!   mantido por outra adquire mesmo assim (como `jlocks.rs`).
//!
//! As chaves (objeto e thread) são raízes do GC enquanto o monitor estiver
//! mantido. Assim um handle não pode ser liberado e reaproveitado por outro
//! objeto enquanto a tabela ainda o referencia.

use std::collections::HashMap;

use crate::heap::ObjRef;

#[derive(Debug, Default)]
pub struct Monitors {
    holds: HashMap<(ObjRef, ObjRef), u32>,
}

impl Monitors {
    /// Registra um `monitor-enter` de `thread` sobre `obj`.
    pub(crate) fn enter(&mut self, obj: ObjRef, thread: ObjRef) {
        let n = self.holds.entry((obj, thread)).or_insert(0);
        *n = n.saturating_add(1);
    }

    /// Registra um `monitor-exit`. Devolve `false` quando `thread` não mantém
    /// `obj`, caso em que a VM deve lançar `IllegalMonitorStateException`.
    pub(crate) fn exit(&mut self, obj: ObjRef, thread: ObjRef) -> bool {
        match self.holds.get_mut(&(obj, thread)) {
            Some(n) => {
                *n -= 1;
                if *n == 0 {
                    self.holds.remove(&(obj, thread));
                }
                true
            }
            None => false,
        }
    }

    /// Raízes do GC: todo objeto e thread que aparece numa entrada mantida.
    pub(crate) fn roots(&self) -> impl Iterator<Item = ObjRef> + '_ {
        self.holds.keys().flat_map(|&(obj, thread)| [obj, thread])
    }

    /// Número de monitores mantidos, para diagnóstico e testes.
    #[cfg(test)]
    pub(crate) fn held(&self) -> usize {
        self.holds.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmatched_exit_is_rejected_and_balanced_nesting_releases() {
        let mut m = Monitors::default();
        assert!(!m.exit(7, 1), "exit sem enter precisa falhar");
        m.enter(7, 1);
        m.enter(7, 1);
        assert_eq!(m.held(), 1);
        assert!(m.exit(7, 1));
        assert_eq!(m.held(), 1, "ainda mantido após o primeiro exit");
        assert!(m.exit(7, 1));
        assert_eq!(m.held(), 0, "libera ao zerar");
        assert!(!m.exit(7, 1), "exit após liberar precisa falhar");
    }

    #[test]
    fn exit_by_other_thread_is_rejected() {
        let mut m = Monitors::default();
        m.enter(7, 1);
        assert!(!m.exit(7, 2), "outra thread não mantém o monitor");
        assert!(m.exit(7, 1));
    }

    #[test]
    fn held_monitor_objects_and_threads_are_roots() {
        let mut m = Monitors::default();
        m.enter(7, 1);
        let mut roots: Vec<ObjRef> = m.roots().collect();
        roots.sort_unstable();
        assert_eq!(roots, vec![1, 7]);
        m.exit(7, 1);
        assert_eq!(m.roots().count(), 0);
    }
}
