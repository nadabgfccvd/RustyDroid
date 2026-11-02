//! XAPK: bundle de splits (`manifest.json` + APKs de split + OBB opcionais).
//!
//! Um `.xapk` é um ZIP sem `AndroidManifest.xml` na raiz. O `manifest.json`
//! lista os APKs (`split_apks`, com `id`: `base` ou `config.*`/`feature.*`) e
//! as expansões OBB (`expansions`). O split `base` é o APK que `Apk::open`
//! devolve; os demais ficam em [`Xapk::splits`] e participam do inventário
//! de `rd inspect --json` via [`Xapk::split_summaries`].
//!
//! Limitação atual: `rd dex disasm` cobre os DEX de todos os splits (ver
//! `rd-cli`), mas `rd test` e a instalação ainda operam só sobre o APK base.

use crate::zip::Zip;
use crate::{Apk, RdError, RdResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Nome do manifesto do bundle dentro do ZIP.
pub const BUNDLE_MANIFEST: &str = "manifest.json";
/// Id de split que representa o APK base (convenção XAPK/APKPure).
pub const BASE_SPLIT_ID: &str = "base";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XapkManifest {
    #[serde(default)]
    pub xapk_version: Option<serde_json::Value>,
    #[serde(default)]
    pub package_name: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version_name: Option<String>,
    #[serde(default)]
    pub version_code: Option<serde_json::Value>,
    #[serde(default)]
    pub min_sdk_version: Option<serde_json::Value>,
    #[serde(default)]
    pub split_apks: Vec<SplitEntry>,
    #[serde(default)]
    pub expansions: Vec<Expansion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitEntry {
    pub file: String,
    #[serde(default)]
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expansion {
    pub file: String,
    #[serde(default)]
    pub install_location: Option<String>,
}

/// Um split já parseado como APK.
#[derive(Debug, Clone, Serialize)]
pub struct LoadedSplit {
    pub id: String,
    pub file: String,
    pub apk: Apk,
}

/// Bundle XAPK completo.
#[derive(Debug, Clone, Serialize)]
pub struct Xapk {
    pub manifest: XapkManifest,
    /// Todos os splits listados, na ordem do manifesto (inclui o base).
    pub splits: Vec<LoadedSplit>,
    /// Índice do split base em `splits`.
    pub base_index: usize,
    /// Arquivos OBB listados em `expansions` e presentes no ZIP.
    pub obbs: Vec<String>,
}

/// Resumo serializável de um split (sem o APK inteiro).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SplitSummary {
    pub id: String,
    pub file: String,
    pub base: bool,
    pub dex_files: usize,
    pub native_libs: usize,
    pub abis: Vec<String>,
    pub res_count: usize,
}

/// `true` quando o ZIP é um bundle XAPK (manifest.json sem AndroidManifest.xml).
pub fn is_bundle(zip: &Zip) -> bool {
    zip.contains(BUNDLE_MANIFEST) && !zip.contains("AndroidManifest.xml")
}

impl Xapk {
    /// Abre um `.xapk` do disco.
    pub fn open(path: &Path) -> RdResult<Xapk> {
        let zip = Zip::open(path)?;
        if !is_bundle(&zip) {
            return Err(RdError::invalid_format(format!(
                "{} não é um XAPK (manifest.json ausente ou AndroidManifest.xml na raiz)",
                path.display()
            )));
        }
        Self::from_zip(&zip)
    }

    /// Lê o bundle a partir de um ZIP já aberto.
    pub fn from_zip(zip: &Zip) -> RdResult<Xapk> {
        let raw = zip.read(BUNDLE_MANIFEST)?;
        let manifest: XapkManifest = serde_json::from_slice(&raw)
            .map_err(|e| RdError::parse(format!("manifest.json do XAPK inválido: {e}")))?;
        if manifest.split_apks.is_empty() {
            return Err(RdError::invalid_format(
                "XAPK sem split_apks no manifest.json",
            ));
        }

        let mut splits = Vec::with_capacity(manifest.split_apks.len());
        for entry in &manifest.split_apks {
            if !zip.contains(&entry.file) {
                return Err(RdError::missing_entry(entry.file.clone()));
            }
            let bytes = zip.read(&entry.file)?;
            let apk = Apk::from_bytes(bytes)?;
            splits.push(LoadedSplit {
                id: entry.id.clone(),
                file: entry.file.clone(),
                apk,
            });
        }

        let base_index = splits
            .iter()
            .position(|s| s.id == BASE_SPLIT_ID)
            .ok_or_else(|| {
                RdError::invalid_format("XAPK sem split base (id \"base\" em split_apks)")
            })?;

        let mut obbs = Vec::new();
        for exp in &manifest.expansions {
            if zip.contains(&exp.file) {
                obbs.push(exp.file.clone());
            }
        }

        Ok(Xapk {
            manifest,
            splits,
            base_index,
            obbs,
        })
    }

    /// Split base (APK principal do app).
    pub fn base(&self) -> &Apk {
        &self.splits[self.base_index].apk
    }

    /// Consome o bundle e devolve o APK base.
    pub fn into_base(mut self) -> Apk {
        self.splits.swap_remove(self.base_index).apk
    }

    /// Resumo por split para o `inspect`.
    pub fn split_summaries(&self) -> Vec<SplitSummary> {
        self.splits
            .iter()
            .enumerate()
            .map(|(i, s)| SplitSummary {
                id: s.id.clone(),
                file: s.file.clone(),
                base: i == self.base_index,
                dex_files: s.apk.dex_files.len(),
                native_libs: s.apk.native_libs.len(),
                abis: s.apk.abis.clone(),
                res_count: s.apk.res_count,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{minimal_manifest_axml, stored_zip};

    /// APK mínimo (ZIP com AndroidManifest.xml) para um split.
    fn apk_for(package: &str) -> Vec<u8> {
        stored_zip(&[("AndroidManifest.xml", &minimal_manifest_axml(package))])
    }

    fn manifest_json(files: &[(&str, &str)], obbs: &[&str]) -> Vec<u8> {
        let splits: Vec<serde_json::Value> = files
            .iter()
            .map(|(file, id)| serde_json::json!({ "file": file, "id": id }))
            .collect();
        let exp: Vec<serde_json::Value> = obbs
            .iter()
            .map(|f| serde_json::json!({ "file": f, "install_location": "shared_storage" }))
            .collect();
        serde_json::to_vec(&serde_json::json!({
            "xapk_version": 2,
            "package_name": "com.example.game",
            "name": "Example",
            "version_name": "1.2.3",
            "version_code": "123",
            "min_sdk_version": "21",
            "split_apks": splits,
            "expansions": exp,
        }))
        .unwrap()
    }

    fn bundle(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
        let refs: Vec<(&str, &[u8])> = entries.iter().map(|(n, b)| (*n, b.as_slice())).collect();
        stored_zip(&refs)
    }

    #[test]
    fn detects_bundle_and_loads_base_split() {
        let base = apk_for("com.example.game");
        let cfg = apk_for("com.example.game");
        let mf = manifest_json(
            &[
                ("com.example.game.apk", "base"),
                ("config.arm64.apk", "config.arm64"),
            ],
            &[],
        );
        let bytes = bundle(&[
            ("manifest.json", mf),
            ("com.example.game.apk", base.clone()),
            ("config.arm64.apk", cfg),
        ]);
        let zip = Zip::parse(bytes.clone()).unwrap();
        assert!(is_bundle(&zip));

        let x = Xapk::from_zip(&zip).unwrap();
        assert_eq!(x.manifest.package_name, "com.example.game");
        assert_eq!(x.splits.len(), 2);
        assert_eq!(x.base_index, 0);
        let summaries = x.split_summaries();
        assert!(summaries[0].base);
        assert!(!summaries[1].base);

        // Apk::from_bytes sobre o bundle devolve o split base.
        let apk = Apk::from_bytes(bytes).unwrap();
        assert_eq!(apk.manifest.package, "com.example.game");
    }

    #[test]
    fn base_is_selected_by_id_not_by_position() {
        let mf = manifest_json(
            &[("config.xxhdpi.apk", "config.xxhdpi"), ("app.apk", "base")],
            &[],
        );
        let bytes = bundle(&[
            ("manifest.json", mf),
            ("config.xxhdpi.apk", apk_for("com.example.game")),
            ("app.apk", apk_for("com.example.base")),
        ]);
        let x = Xapk::from_zip(&Zip::parse(bytes).unwrap()).unwrap();
        assert_eq!(x.base_index, 1);
        assert_eq!(x.base().manifest.package, "com.example.base");
    }

    #[test]
    fn lists_obb_expansions_present_in_zip() {
        let mf = manifest_json(
            &[("app.apk", "base")],
            &[
                "main.42.com.example.game.obb",
                "patch.42.com.example.game.obb",
            ],
        );
        let bytes = bundle(&[
            ("manifest.json", mf),
            ("app.apk", apk_for("com.example.game")),
            ("main.42.com.example.game.obb", vec![0u8; 16]),
        ]);
        let x = Xapk::from_zip(&Zip::parse(bytes).unwrap()).unwrap();
        assert_eq!(x.obbs, vec!["main.42.com.example.game.obb".to_string()]);
    }

    #[test]
    fn missing_split_file_is_reported() {
        let mf = manifest_json(&[("app.apk", "base"), ("gone.apk", "config.hdpi")], &[]);
        let bytes = bundle(&[
            ("manifest.json", mf),
            ("app.apk", apk_for("com.example.game")),
        ]);
        let err = Xapk::from_zip(&Zip::parse(bytes).unwrap()).unwrap_err();
        assert!(err.cause.contains("gone.apk"), "{}", err.cause);
    }

    #[test]
    fn bundle_without_base_split_is_rejected() {
        let mf = manifest_json(&[("cfg.apk", "config.hdpi")], &[]);
        let bytes = bundle(&[
            ("manifest.json", mf),
            ("cfg.apk", apk_for("com.example.game")),
        ]);
        let err = Xapk::from_zip(&Zip::parse(bytes).unwrap()).unwrap_err();
        assert!(err.cause.contains("split base"), "{}", err.cause);
    }

    #[test]
    fn apk_with_root_manifest_is_not_a_bundle() {
        let bytes = bundle(&[
            (
                "AndroidManifest.xml",
                minimal_manifest_axml("com.example.game"),
            ),
            ("manifest.json", b"{}".to_vec()),
        ]);
        assert!(!is_bundle(&Zip::parse(bytes).unwrap()));
    }

    #[test]
    fn invalid_manifest_json_is_a_parse_error() {
        let bytes = bundle(&[("manifest.json", b"{not json".to_vec())]);
        let err = Xapk::from_zip(&Zip::parse(bytes).unwrap()).unwrap_err();
        assert!(err.cause.contains("manifest.json"), "{}", err.cause);
    }
}
