//! APK e DEX compartilhados entre sessões do mesmo arquivo (ADR 031).
//!
//! Antes, cada sessão do `rd mcp` lia o APK inteiro no `install` e montava os
//! DEX de novo no `launch`. Os bytes são idênticos entre sessões, e um `Dex`
//! montado é imutável, então o estado pode ser compartilhado por `Arc`.
//!
//! Regras:
//! - O registro é um mapa `ApkKey -> Weak<SharedApk>`. Ele não mantém nada vivo:
//!   o conteúdo sai da memória quando a última sessão que o usa é fechada (ou
//!   reinstala o pacote), e a próxima instalação reabre o arquivo.
//! - Os DEX montados também são guardados por `Weak`. Enquanto um engine roda
//!   com eles, outra sessão os reusa; depois que o último engine para, a memória
//!   volta, como antes deste módulo.
//! - A chave é o caminho canônico, o tamanho e o mtime (ns). Qualquer mudança
//!   nesses três campos é outro APK: não há reuso.
//! - Um falha de montagem de DEX não é guardada; a próxima tentativa refaz.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::UNIX_EPOCH;

use rd_apk::xapk::SplitSummary;
use rd_apk::Apk;
use rd_apk::RdError;
use rd_dex::Dex;
use rd_vm::classpath::DecodeCache;

/// Identidade de um arquivo de APK no disco.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ApkKey {
    path: PathBuf,
    len: u64,
    mtime_ns: u128,
}

impl ApkKey {
    /// Chave do arquivo em `path`. Erro de E/S (inexistente, sem permissão)
    /// devolve `Err`; quem chama cai no caminho sem compartilhamento.
    pub(crate) fn of(path: &Path) -> std::io::Result<ApkKey> {
        let path = std::fs::canonicalize(path)?;
        let meta = std::fs::metadata(&path)?;
        let mtime_ns = meta
            .modified()?
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        Ok(ApkKey {
            path,
            len: meta.len(),
            mtime_ns,
        })
    }
}

/// Um APK aberto e os DEX montados a partir dele, compartilhados por `Arc`.
pub(crate) struct SharedApk {
    /// O APK completo (manifesto, ZIP, recursos).
    pub(crate) apk: Arc<Apk>,
    /// Resumo dos splits XAPK, para a resposta de `install`.
    pub(crate) bundle_splits: Option<Vec<SplitSummary>>,
    /// DEX montados, na ordem de `dexes_of_apk`, só como `Weak`: quem segura
    /// os `Arc<Dex>` são os engines em execução.
    dexes: Mutex<Option<Vec<Weak<Dex>>>>,
    /// Cache de decodificação (ADR 032) criado junto com esta montagem de DEX.
    /// Também só como `Weak`: vive enquanto houver engine com ele.
    decode: Mutex<Weak<DecodeCache>>,
}

impl SharedApk {
    pub(crate) fn new(apk: Apk, bundle_splits: Option<Vec<SplitSummary>>) -> Arc<SharedApk> {
        Arc::new(SharedApk {
            apk: Arc::new(apk),
            bundle_splits,
            dexes: Mutex::new(None),
            decode: Mutex::new(Weak::new()),
        })
    }

    /// DEX montados deste APK e o cache de decodificação que os acompanha
    /// (ADR 032). Se ainda houver engine com os DEX vivos, devolve os mesmos
    /// `Arc`s; senão monta de novo com `mount` e cria um cache novo. Falha de
    /// montagem não é guardada.
    pub(crate) fn dexes(
        &self,
        mount: impl FnOnce(&Apk) -> Result<Vec<Dex>, RdError>,
    ) -> Result<(Vec<Arc<Dex>>, Arc<DecodeCache>), RdError> {
        let mut slot = self.dexes.lock().unwrap_or_else(|e| e.into_inner());
        let mut decode_slot = self.decode.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(weak) = slot.as_ref() {
            let alive: Option<Vec<Arc<Dex>>> = weak.iter().map(Weak::upgrade).collect();
            if let Some(d) = alive {
                // Cache ainda vivo: reusa. Se morreu, o conjunto de DEX continua
                // válido, então o cache novo só começa vazio.
                let decode = match decode_slot.upgrade() {
                    Some(c) => c,
                    None => {
                        let c = Arc::new(DecodeCache::default());
                        *decode_slot = Arc::downgrade(&c);
                        c
                    }
                };
                return Ok((d, decode));
            }
        }
        let mounted: Vec<Arc<Dex>> = mount(&self.apk)?.into_iter().map(Arc::new).collect();
        let decode = Arc::new(DecodeCache::default());
        *decode_slot = Arc::downgrade(&decode);
        *slot = Some(mounted.iter().map(Arc::downgrade).collect());
        Ok((mounted, decode))
    }
}

fn registry() -> &'static Mutex<HashMap<ApkKey, Weak<SharedApk>>> {
    static REG: OnceLock<Mutex<HashMap<ApkKey, Weak<SharedApk>>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Entrada viva para `key`, se alguma sessão ainda a segura.
pub(crate) fn lookup(key: &ApkKey) -> Option<Arc<SharedApk>> {
    let reg = registry().lock().unwrap_or_else(|e| e.into_inner());
    reg.get(key).and_then(Weak::upgrade)
}

/// Registra `shared` para `key`. Entradas mortas são podadas aqui.
pub(crate) fn register(key: ApkKey, shared: &Arc<SharedApk>) {
    let mut reg = registry().lock().unwrap_or_else(|e| e.into_inner());
    reg.retain(|_, w| w.strong_count() > 0);
    reg.insert(key, Arc::downgrade(shared));
}
