//! fd-permissions — motor de permissões completo sobre dado versionado
//! (`data/permissions.toml`, Apêndice B da spec).
//!
//! Cobre: níveis de proteção (B1), grupos dangerous (B2), especiais/AppOps (B3),
//! normais (B4), signature/internal (B5), roles + appops (B6). O estado é
//! programável (grant/deny/revoke) — base para o AG-11 nas fases M5+.

use rd_apk::manifest::Manifest;
use rd_apk::{RdError, RdResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MODULE_ID: &str = "fd-permissions";

/// Fallback embutido — o binário funciona de qualquer CWD; em disco o override
/// via `--data`/`RD_DATA_DIR` tem precedência.
pub const EMBEDDED_PERMISSIONS_TOML: &str = include_str!("../../../data/permissions.toml");

// ─── modelo do dado ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectionLevel {
    Normal,
    Dangerous,
    Signature,
    SignatureOrSystem,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDef {
    pub name: String,
    pub level: ProtectionLevel,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub group_empty: Option<String>, // tolerância a `group = ""`
    #[serde(default = "default_since")]
    pub since_api: u16,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub description: String,
}

fn default_since() -> u16 {
    1
}

impl PermissionDef {
    pub fn is_special(&self) -> bool {
        self.flags.iter().any(|f| f == "special")
    }
    pub fn is_appop(&self) -> bool {
        self.flags.iter().any(|f| f == "appop")
    }
    pub fn group_name(&self) -> Option<&str> {
        self.group
            .as_deref()
            .filter(|g| !g.is_empty())
            .or(self.group_empty.as_deref().filter(|g| !g.is_empty()))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Role {
    pub name: String,
    #[serde(default = "default_since")]
    pub since_api: u16,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppOp {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PermissionTable {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub permission: Vec<PermissionDef>,
    #[serde(default)]
    pub role: Vec<Role>,
    #[serde(default)]
    pub appop: Vec<AppOp>,
}

fn default_schema_version() -> u32 {
    1
}

impl PermissionTable {
    /// Parse do TOML (fonte: arquivo ou embutido).
    pub fn from_toml(contents: &str) -> RdResult<Self> {
        toml::from_str(contents)
            .map_err(|e| RdError::new("DATA_ERROR", format!("permissions.toml: {e}"), MODULE_ID))
    }

    /// Tabela embutida no binário (fallback sempre disponível).
    pub fn embedded() -> RdResult<Self> {
        Self::from_toml(EMBEDDED_PERMISSIONS_TOML)
    }

    /// Aceita nome completo (`android.permission.CAMERA`) ou curto (`CAMERA`).
    pub fn get(&self, name: &str) -> Option<&PermissionDef> {
        let short = name.rsplit('.').next().unwrap_or(name);
        self.permission
            .iter()
            .find(|p| p.name == name || p.name.rsplit('.').next() == Some(short))
    }

    pub fn dangerous(&self) -> impl Iterator<Item = &PermissionDef> {
        self.permission
            .iter()
            .filter(|p| p.level == ProtectionLevel::Dangerous)
    }

    pub fn specials(&self) -> impl Iterator<Item = &PermissionDef> {
        self.permission.iter().filter(|p| p.is_special())
    }

    pub fn len(&self) -> usize {
        self.permission.len()
    }

    pub fn is_empty(&self) -> bool {
        self.permission.is_empty()
    }
}

// ─── estado de concessão ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantState {
    /// normal/signature/internal — concedida automaticamente (B5).
    AutoGranted,
    /// dangerous — exige diálogo runtime.
    RuntimePending,
    /// special access — via AppOps/settings, não dialog comum (B3).
    SpecialPending,
    /// concedida após grant() do agente/fluxo runtime.
    Granted,
    /// negada pelo usuário/agente.
    Denied,
    /// não existe na tabela versionada (desconhecida para o motor).
    Unknown,
}

impl GrantState {
    pub fn is_granted(self) -> bool {
        matches!(self, GrantState::AutoGranted | GrantState::Granted)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditFinding {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    High,
}

/// Motor de permissões: instala um manifest e mantém estado consultável/mutável.
pub struct PermissionEngine {
    pub table: PermissionTable,
    states: BTreeMap<String, GrantState>,
    /// targetSdk do app instalado — comuta semânticas (ex.: permissions @23+).
    target_sdk: u32,
}

impl PermissionEngine {
    /// Estado inicial de uma permissão dado o nível/firmware.
    pub fn initial_state(def: &PermissionDef) -> GrantState {
        if def.is_special() {
            GrantState::SpecialPending
        } else {
            match def.level {
                ProtectionLevel::Normal => GrantState::AutoGranted,
                ProtectionLevel::Dangerous => GrantState::RuntimePending,
                // B5: signature/signatureOrSystem/internal são "concedidas" para
                // o app em análise (o runtime é o "system" aqui)
                ProtectionLevel::Signature
                | ProtectionLevel::SignatureOrSystem
                | ProtectionLevel::Internal => GrantState::AutoGranted,
            }
        }
    }

    /// Instala as permissões declaradas de um manifest.
    pub fn install(manifest: &Manifest, table: PermissionTable) -> Self {
        let mut states = BTreeMap::new();
        for up in &manifest.uses_permissions {
            let state = match table.get(&up.name) {
                Some(def) => Self::initial_state(def),
                None => GrantState::Unknown,
            };
            states.insert(up.name.clone(), state);
        }
        PermissionEngine {
            table,
            states,
            target_sdk: manifest.target_sdk,
        }
    }

    pub fn target_sdk(&self) -> u32 {
        self.target_sdk
    }

    pub fn state(&self, name: &str) -> Option<GrantState> {
        self.states.get(name).copied()
    }

    pub fn all_states(&self) -> &BTreeMap<String, GrantState> {
        &self.states
    }

    /// Runtime grant (diálogo) — dangerous → Granted.
    pub fn grant(&mut self, name: &str) -> RdResult<GrantState> {
        let cur = self
            .states
            .get_mut(name)
            .ok_or_else(|| RdError::missing_entry(name))?;
        match cur {
            GrantState::RuntimePending | GrantState::SpecialPending | GrantState::Denied => {
                *cur = GrantState::Granted;
                Ok(GrantState::Granted)
            }
            s => Ok(*s),
        }
    }

    /// Deny — pending → Denied (com "don't ask again" simulado pelo estado).
    pub fn deny(&mut self, name: &str) -> RdResult<GrantState> {
        let cur = self
            .states
            .get_mut(name)
            .ok_or_else(|| RdError::missing_entry(name))?;
        match cur {
            GrantState::RuntimePending | GrantState::SpecialPending => {
                *cur = GrantState::Denied;
                Ok(GrantState::Denied)
            }
            s => Ok(*s),
        }
    }

    /// Revoga de volta ao estado inicial da instalação (unused-app auto-revoke usa isto).
    pub fn revoke(&mut self, name: &str) -> RdResult<GrantState> {
        let def = self.table.get(name).map(Self::initial_state);
        let init = def.unwrap_or(GrantState::Unknown);
        let cur = self
            .states
            .get_mut(name)
            .ok_or_else(|| RdError::missing_entry(name))?;
        *cur = init;
        Ok(init)
    }

    /// Permissões aguardando diálogo runtime.
    pub fn pending_runtime(&self) -> Vec<String> {
        self.states
            .iter()
            .filter(|(_, s)| **s == GrantState::RuntimePending)
            .map(|(n, _)| n.clone())
            .collect()
    }

    /// Permissões de acesso especial pendentes (AppOps).
    pub fn pending_special(&self) -> Vec<String> {
        self.states
            .iter()
            .filter(|(_, s)| **s == GrantState::SpecialPending)
            .map(|(n, _)| n.clone())
            .collect()
    }

    /// Agrupamento por grupo DANGEROUS (para diálogos e UI de settings).
    pub fn groups(&self) -> BTreeMap<String, Vec<String>> {
        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for name in self.states.keys() {
            if let Some(def) = self.table.get(name) {
                if let Some(g) = def.group_name() {
                    groups.entry(g.to_string()).or_default().push(name.clone());
                }
            }
        }
        groups
    }

    /// Auditoria estática (TST-09 parcial) sobre manifest + estado.
    pub fn audit(&self, manifest: &Manifest) -> Vec<AuditFinding> {
        let mut findings = Vec::new();

        for up in &manifest.uses_permissions {
            if self.table.get(&up.name).is_none() {
                findings.push(AuditFinding {
                    severity: Severity::Warning,
                    code: "UNKNOWN_PERMISSION".into(),
                    message: format!(
                        "permission {} is not in the versioned table — verify the exact name",
                        up.name
                    ),
                    permission: Some(up.name.clone()),
                    component: None,
                });
            }
        }

        let pending = self.pending_runtime();
        if !pending.is_empty() {
            findings.push(AuditFinding {
                severity: Severity::Info,
                code: "RUNTIME_PENDING".into(),
                message: format!(
                    "{} dangerous permission(s) require a runtime grant dialog: {}",
                    pending.len(),
                    pending.join(", ")
                ),
                permission: None,
                component: None,
            });
        }

        let specials = self.pending_special();
        if !specials.is_empty() {
            findings.push(AuditFinding {
                severity: Severity::Info,
                code: "SPECIAL_ACCESS".into(),
                message: format!(
                    "special app access required (AppOps/settings): {}",
                    specials.join(", ")
                ),
                permission: None,
                component: None,
            });
        }

        for c in &manifest.application.components {
            if c.exported_effective && c.permission.is_none() {
                let sensitive = matches!(
                    c.kind,
                    rd_apk::manifest::ComponentKind::Service
                        | rd_apk::manifest::ComponentKind::Receiver
                        | rd_apk::manifest::ComponentKind::Provider
                );
                findings.push(AuditFinding {
                    severity: if sensitive {
                        Severity::High
                    } else {
                        Severity::Info
                    },
                    code: "EXPORTED_NO_GUARD".into(),
                    message: format!(
                        "{} {} is exported without a permission guard{}",
                        c.kind,
                        c.class_name,
                        if c.intent_filters.is_empty() {
                            ""
                        } else {
                            " (has intent-filters)"
                        }
                    ),
                    permission: None,
                    component: Some(c.class_name.clone()),
                });
            }
        }

        if manifest.application.debuggable == Some(true) {
            findings.push(AuditFinding {
                severity: Severity::High,
                code: "DEBUGGABLE".into(),
                message: "application is debuggable — never ship this build".into(),
                permission: None,
                component: None,
            });
        }

        let cleartext_allowed =
            manifest.application.uses_cleartext_traffic == Some(true) || manifest.target_sdk < 28;
        if cleartext_allowed {
            findings.push(AuditFinding {
                severity: Severity::Info,
                code: "CLEARTEXT_ALLOWED".into(),
                message: format!(
                    "cleartext HTTP is allowed by policy (targetSdk {} < 28 or usesCleartextTraffic=true)",
                    manifest.target_sdk
                ),
                permission: None,
                component: None,
            });
        }

        findings.sort_by_key(|f| std::cmp::Reverse(f.severity));
        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rd_apk::manifest::UsesPermission;

    fn manifest_with(perms: &[&str]) -> Manifest {
        Manifest {
            package: "com.t".into(),
            min_sdk: 26,
            target_sdk: 34,
            uses_permissions: perms
                .iter()
                .map(|p| UsesPermission {
                    name: format!("android.permission.{p}"),
                    max_sdk: None,
                    sdk23: false,
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn embedded_table_parses_and_is_complete() {
        let table = PermissionTable::embedded().unwrap();
        assert!(table.len() >= 150, "table has {} permissions", table.len());
        assert!(table.role.len() >= 13);
        assert!(table.appop.len() >= 7);
        // 12 grupos dangerous da B2
        let mut groups: Vec<&str> = table
            .dangerous()
            .filter_map(|d| d.group_name())
            .collect::<Vec<_>>();
        groups.sort();
        groups.dedup();
        assert_eq!(groups.len(), 12, "groups: {groups:?}");
        // câmera: dangerous, grupo CAMERA
        let cam = table.get("android.permission.CAMERA").unwrap();
        assert_eq!(cam.level, ProtectionLevel::Dangerous);
        assert_eq!(cam.group_name(), Some("CAMERA"));
    }

    #[test]
    fn install_classifies_by_level() {
        let table = PermissionTable::embedded().unwrap();
        let m = manifest_with(&[
            "INTERNET",
            "CAMERA",
            "SYSTEM_ALERT_WINDOW",
            "NOT_A_REAL_PERM",
        ]);
        let engine = PermissionEngine::install(&m, table);
        assert_eq!(
            engine.state("android.permission.INTERNET"),
            Some(GrantState::AutoGranted)
        );
        assert_eq!(
            engine.state("android.permission.CAMERA"),
            Some(GrantState::RuntimePending)
        );
        assert_eq!(
            engine.state("android.permission.SYSTEM_ALERT_WINDOW"),
            Some(GrantState::SpecialPending)
        );
        assert_eq!(
            engine.state("android.permission.NOT_A_REAL_PERM"),
            Some(GrantState::Unknown)
        );
        assert_eq!(engine.pending_runtime(), vec!["android.permission.CAMERA"]);
    }

    #[test]
    fn grant_deny_revoke_lifecycle() {
        let table = PermissionTable::embedded().unwrap();
        let m = manifest_with(&["CAMERA", "RECORD_AUDIO"]);
        let mut engine = PermissionEngine::install(&m, table);
        let cam = "android.permission.CAMERA";
        assert_eq!(engine.grant(cam).unwrap(), GrantState::Granted);
        assert!(engine.state(cam).unwrap().is_granted());
        assert_eq!(engine.revoke(cam).unwrap(), GrantState::RuntimePending);
        assert_eq!(engine.deny(cam).unwrap(), GrantState::Denied);
        // unknown name → erro estruturado
        assert_eq!(
            engine.grant("android.permission.NOPE").unwrap_err().code,
            "MISSING_ENTRY"
        );
    }

    #[test]
    fn short_name_lookup_works() {
        let table = PermissionTable::embedded().unwrap();
        assert!(table.get("CAMERA").is_some());
        assert!(table.get("android.permission.CAMERA").is_some());
        assert!(table.get("totally-bogus").is_none());
    }

    #[test]
    fn audit_flags_exported_services_and_unknown_perms() {
        let table = PermissionTable::embedded().unwrap();
        let mut m = manifest_with(&["INTERNET", "MYSTERY_PERM"]);
        m.application.components.push(rd_apk::manifest::Component {
            kind: rd_apk::manifest::ComponentKind::Service,
            class_name: "com.t.Svc".into(),
            exported_effective: true,
            ..Default::default()
        });
        m.application.debuggable = Some(true);
        let engine = PermissionEngine::install(&m, table);
        let findings = engine.audit(&m);
        let codes: Vec<&str> = findings.iter().map(|f| f.code.as_str()).collect();
        assert!(codes.contains(&"UNKNOWN_PERMISSION"));
        assert!(codes.contains(&"EXPORTED_NO_GUARD"));
        assert!(codes.contains(&"DEBUGGABLE"));
        // INTERNET é normal (auto-grant) e MYSTERY é unknown → nada pendente de runtime
        assert!(!codes.contains(&"RUNTIME_PENDING"));
    }
}
