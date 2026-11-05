//! fd-permissions — motor de permissões completo sobre dado versionado
//! (`data/permissions.toml`, Apêndice B da spec).
//!
//! Cobre: níveis de proteção (B1), grupos dangerous (B2), especiais/AppOps (B3),
//! normais (B4), signature/internal (B5), roles + appops (B6). O estado é
//! programável (grant/deny/revoke) — base para o AG-11 nas fases M5+.

use rd_apk::manifest::Manifest;
use rd_apk::{RdError, RdResult};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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
    /// Apêndice B6 `role`: a permissão é concedida enquanto o app é holder do
    /// role indicado (ex.: `SMS`). Ver `PermissionEngine::set_role`.
    #[serde(default)]
    pub role: Option<String>,
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
    /// issue #110: na whitelist B5 da spec (Apêndice B5) — a ÚNICA lista de
    /// signature-class que o runtime concede ao app em análise ("o runtime é
    /// o system", ver `initial_state`). Marcada no dado com a flag `b5`.
    pub fn is_b5(&self) -> bool {
        self.flags.iter().any(|f| f == "b5")
    }
    /// Apêndice B1 `hardRestricted`: não pode ser concedida em runtime a menos
    /// que o instalador esteja na allowlist (`PermissionEngine::allow_hard_restricted`).
    pub fn is_hard_restricted(&self) -> bool {
        self.flags.iter().any(|f| f == "hardRestricted")
    }
    /// Apêndice B1 `immutable`: o estado vem do sistema e o runtime não o altera
    /// (grant/deny/revoke não têm efeito).
    pub fn is_immutable(&self) -> bool {
        self.flags.iter().any(|f| f == "immutable")
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
    /// normal/signature-B5/internal-B5 — concedida automaticamente (B5).
    AutoGranted,
    /// dangerous — exige diálogo runtime.
    RuntimePending,
    /// special access — via AppOps/settings, não dialog comum (B3).
    SpecialPending,
    /// concedida após grant() do agente/fluxo runtime.
    Granted,
    /// negada pelo usuário/agente.
    Denied,
    /// issue #110: signature-class FORA da whitelist B5 da spec (Apêndice B5)
    /// — o Android real nega a apps de terceiros (ex.: BATTERY_STATS). O
    /// registro existe, mas não é concedida NEM concedível na sessão:
    /// grant()/deny() a preservam (decisão registrada em RustyDroid.md §B5).
    DeniedSystem,
    /// não existe na tabela versionada (desconhecida para o motor).
    Unknown,
    /// issue #69: declarada pelo PRÓPRIO APK (`<permission>` no manifesto).
    /// Não é desconhecida — o app a define e a detém; o Android real só
    /// concede a apps de terceiros conforme o `protectionLevel` declarado.
    AppDefined,
    /// issue #31: nem chega a ser pedida — maxSdkVersion vencido, since_api
    /// acima do targetSdk, ou uses-permission-sdk-23 com target < 23 (o
    /// Android real nem registra a permissão nesses casos).
    NotRequested,
}

impl GrantState {
    pub fn is_granted(self) -> bool {
        matches!(
            self,
            GrantState::AutoGranted | GrantState::Granted | GrantState::AppDefined
        )
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
/// issue #31: o gating por maxSdkVersion/since_api/sdk-23 está implementado —
/// permissão fora da janela do targetSdk vira `NotRequested` (o Android real
/// nem a registra).
pub struct PermissionEngine {
    pub table: PermissionTable,
    states: BTreeMap<String, GrantState>,
    /// issue #69: nomes declarados pelo próprio manifesto (`<permission>`).
    declared: BTreeSet<String>,
    /// targetSdk do APK: gate do auto-revoke (API 30+) e do acesso parcial a mídia (API 34+).
    target_sdk: u32,
    /// Negativas do diálogo runtime por permissão (Apêndice B). 1 = há justificativa
    /// a mostrar; 2 ou mais = "don't ask again" (negada de forma permanente).
    denials: BTreeMap<String, u32>,
    /// Apêndice B1 `hardRestricted`: o instalador do APK está na allowlist
    /// (`PackageInstaller.SessionParams.setWhitelistedRestrictedPermissions`).
    /// Padrão: não — o Android nega essas permissões a apps fora da allowlist.
    restricted_allowed: bool,
    /// Apêndice B6: roles cujo app é holder nesta sessão.
    roles_held: BTreeSet<String>,
    /// Permissões cuja concessão veio de um role (revertidas quando o role sai).
    role_granted: BTreeSet<String>,
}

/// Dias sem uso após os quais o Android revoga permissões runtime de um app
/// que mira API 30+ (unused-app auto-revoke, Apêndice B6). Limiar configurável
/// pelo chamador; este é o valor padrão do motor.
pub const UNUSED_APP_REVOKE_DAYS: u32 = 90;

/// API em que o acesso parcial a fotos/vídeos (`READ_MEDIA_VISUAL_USER_SELECTED`)
/// passou a existir (Apêndice B5/B6: justificativas API 34+).
pub const PARTIAL_MEDIA_API: u32 = 34;

/// targetSdk mínimo para o auto-revoke de apps sem uso.
pub const AUTO_REVOKE_TARGET_SDK: u32 = 30;

impl PermissionEngine {
    /// Estado inicial de uma permissão dado o nível/firmware.
    pub fn initial_state(def: &PermissionDef) -> GrantState {
        if def.is_special() {
            GrantState::SpecialPending
        } else {
            match def.level {
                ProtectionLevel::Normal => GrantState::AutoGranted,
                ProtectionLevel::Dangerous => GrantState::RuntimePending,
                // B5 (decisão da issue #110): signature-class só é "concedida"
                // ao app em análise se estiver na WHITELIST do Apêndice B5 da
                // spec (flag `b5` no dado) — para essas, o runtime se posiciona
                // como system (marcador runtime-is-system). Fora da whitelist
                // o Android real nega a app de terceiro (ex.: BATTERY_STATS,
                // signature|privileged|development) — estado DeniedSystem,
                // nem concedida nem concedível na sessão.
                ProtectionLevel::Signature
                | ProtectionLevel::SignatureOrSystem
                | ProtectionLevel::Internal => {
                    if def.is_b5() {
                        GrantState::AutoGranted
                    } else {
                        GrantState::DeniedSystem
                    }
                }
            }
        }
    }

    /// Instala as permissões declaradas de um manifest.
    ///
    /// issue #31: gating aplicado na instalação —
    /// (a) `android:maxSdkVersion`: declarada só até aquela API; target acima
    ///     → `NotRequested` (WRITE_EXTERNAL_STORAGE maxSdk=28 em target 33+);
    /// (b) `since_api` da tabela: permissão que ainda não existe no targetSdk
    ///     → `NotRequested` (POST_NOTIFICATIONS since 33 em target 26);
    /// (c) `uses-permission-sdk-23`: só pedida com target ≥ 23.
    pub fn install(manifest: &Manifest, table: PermissionTable) -> Self {
        let declared: BTreeSet<String> = manifest
            .declared_permissions
            .iter()
            .map(|d| d.name.clone())
            .collect();
        let mut states = BTreeMap::new();
        for up in &manifest.uses_permissions {
            let gated_out = up.max_sdk.is_some_and(|max| manifest.target_sdk > max)
                || (up.sdk23 && manifest.target_sdk < 23);
            let state = if gated_out {
                GrantState::NotRequested
            } else {
                match table.get(&up.name) {
                    Some(def) if def.since_api as u32 > manifest.target_sdk => {
                        GrantState::NotRequested
                    }
                    Some(def) => Self::initial_state(def),
                    // issue #69: ausente da tabela MAS declarada pelo próprio
                    // APK não é "desconhecida" — o app a define.
                    None if declared.contains(&up.name) => GrantState::AppDefined,
                    None => GrantState::Unknown,
                }
            };
            states.insert(up.name.clone(), state);
        }
        PermissionEngine {
            table,
            states,
            declared,
            target_sdk: manifest.target_sdk,
            denials: BTreeMap::new(),
            restricted_allowed: false,
            roles_held: BTreeSet::new(),
            role_granted: BTreeSet::new(),
        }
    }

    /// Negativas acumuladas do diálogo runtime desta permissão.
    pub fn denial_count(&self, name: &str) -> u32 {
        self.denials.get(name).copied().unwrap_or(0)
    }

    /// `shouldShowRequestPermissionRationale`: verdadeiro só após exatamente uma
    /// negativa do diálogo. Nunca pedida ou já negada de forma permanente → falso.
    pub fn shows_rationale(&self, name: &str) -> bool {
        self.denial_count(name) == 1
    }

    /// Nomes com justificativa a mostrar (o que o VM responde em
    /// `shouldShowRequestPermissionRationale`).
    pub fn rationale_names(&self) -> Vec<String> {
        self.states
            .keys()
            .filter(|n| self.shows_rationale(n))
            .cloned()
            .collect()
    }

    /// "Don't ask again": duas negativas (ou mais) do diálogo runtime.
    pub fn is_permanently_denied(&self, name: &str) -> bool {
        self.denial_count(name) >= 2
    }

    /// Unused-app auto-revoke (Apêndice B6). Apps com targetSdk < 30 ficam de fora.
    /// Revoga as permissões runtime (dangerous) concedidas que estão sem uso há
    /// `threshold_days` ou mais, voltando-as ao estado inicial. Devolve os nomes revogados.
    pub fn auto_revoke_unused(&mut self, days_unused: u32, threshold_days: u32) -> Vec<String> {
        if self.target_sdk < AUTO_REVOKE_TARGET_SDK || days_unused < threshold_days {
            return Vec::new();
        }
        let granted: Vec<String> =
            self.states
                .iter()
                .filter(|(name, st)| {
                    **st == GrantState::Granted
                        && self.table.get(name).is_some_and(|d| {
                            d.level == ProtectionLevel::Dangerous && !d.is_immutable()
                        })
                })
                .map(|(n, _)| n.clone())
                .collect();
        for name in &granted {
            // `revoke` só falha para nomes ausentes; a lista vem do próprio mapa.
            let _ = self.revoke(name);
        }
        granted
    }

    /// Acesso parcial a mídia (API 34+): concede `READ_MEDIA_VISUAL_USER_SELECTED`
    /// sem conceder `READ_MEDIA_IMAGES`/`READ_MEDIA_VIDEO`. Devolve `false` se o
    /// app não declara a permissão ou se o targetSdk é anterior à API 34.
    pub fn grant_partial_media(&mut self) -> bool {
        const VISUAL: &str = "READ_MEDIA_VISUAL_USER_SELECTED";
        if self.target_sdk < PARTIAL_MEDIA_API {
            return false;
        }
        // a chave do mapa é o nome do manifesto (`android.permission.X`) ou o curto
        let Some(key) = self
            .states
            .keys()
            .find(|k| k.rsplit('.').next() == Some(VISUAL))
            .cloned()
        else {
            return false;
        };
        let Some(cur) = self.states.get_mut(&key) else {
            return false;
        };
        if matches!(cur, GrantState::RuntimePending | GrantState::Denied) {
            *cur = GrantState::Granted;
            self.denials.remove(&key);
            true
        } else {
            false
        }
    }

    /// targetSdk do APK instalado (gates do auto-revoke e do acesso parcial).
    pub fn target_sdk(&self) -> u32 {
        self.target_sdk
    }

    /// issue #69: permissões que o próprio APK declara via `<permission>`
    /// (na ordem do manifesto), com o `protectionLevel` decodificado.
    pub fn declared_permissions(&self) -> &BTreeSet<String> {
        &self.declared
    }

    /// issue #31: marcador de confiança para o output — permissões de nível
    /// signature/signatureOrSystem/internal são "auto-concedidas" apenas
    /// porque o runtime se posiciona como system; apps reais de terceiros
    /// as teriam NEGADAS. O JSON do CLI usa isto para não enganar agentes.
    /// issue #51: Internal é o mais system-only de todos — entrou no marker.
    pub fn granted_by(&self, name: &str) -> Option<&'static str> {
        // issue #69: permissão definida pelo próprio APK — o app a detém, mas
        // o agente precisa saber que a origem é a própria declaração.
        if self.state(name) == Some(GrantState::AppDefined) {
            return Some("self-declared");
        }
        if self.state(name) == Some(GrantState::AutoGranted) {
            if let Some(def) = self.table.get(name) {
                return matches!(
                    def.level,
                    ProtectionLevel::Signature
                        | ProtectionLevel::SignatureOrSystem
                        | ProtectionLevel::Internal
                )
                .then_some("runtime-is-system");
            }
        }
        None
    }

    pub fn state(&self, name: &str) -> Option<GrantState> {
        self.states.get(name).copied()
    }

    pub fn all_states(&self) -> &BTreeMap<String, GrantState> {
        &self.states
    }

    /// Apêndice B1 `immutable`: a permissão não muda por grant/deny/revoke.
    fn is_frozen(&self, name: &str) -> bool {
        self.table.get(name).is_some_and(|d| d.is_immutable())
    }

    /// Apêndice B6: o app é holder do role que concede esta permissão.
    fn role_covers(&self, def: &PermissionDef) -> bool {
        def.role
            .as_deref()
            .is_some_and(|r| self.roles_held.contains(r))
    }

    /// Roles de que o app é holder nesta sessão.
    pub fn roles_held(&self) -> &BTreeSet<String> {
        &self.roles_held
    }

    /// Apêndice B6: torna o app holder (`held=true`) ou tira o role (`held=false`).
    /// Ao virar holder, as permissões declaradas com `role = <nome>` passam a
    /// `Granted`; ao sair, as que vieram do role voltam ao estado inicial.
    /// Devolve os nomes de permissão alterados. Role desconhecido é erro.
    pub fn set_role(&mut self, role: &str, held: bool) -> RdResult<Vec<String>> {
        if !self.table.role.iter().any(|r| r.name == role) {
            return Err(RdError::missing_entry(role));
        }
        let mut changed = Vec::new();
        if held {
            self.roles_held.insert(role.to_string());
            let names: Vec<String> = self
                .states
                .keys()
                .filter(|k| {
                    self.table
                        .get(k)
                        .is_some_and(|d| d.role.as_deref() == Some(role))
                })
                .cloned()
                .collect();
            for name in names {
                let cur = self.states.get_mut(&name).expect("chave listada");
                if !matches!(cur, GrantState::Granted | GrantState::AutoGranted) {
                    *cur = GrantState::Granted;
                    self.denials.remove(&name);
                    self.role_granted.insert(name.clone());
                    changed.push(name);
                }
            }
        } else {
            self.roles_held.remove(role);
            let leaving: Vec<String> = self
                .role_granted
                .iter()
                .filter(|n| {
                    self.table
                        .get(n)
                        .is_some_and(|d| d.role.as_deref() == Some(role))
                })
                .cloned()
                .collect();
            for name in leaving {
                self.role_granted.remove(&name);
                let _ = self.revoke(&name);
                changed.push(name);
            }
        }
        Ok(changed)
    }

    /// Estado atual (sem mudança) ou erro se a permissão não existe no mapa.
    fn current_or_missing(&self, name: &str) -> RdResult<GrantState> {
        self.states
            .get(name)
            .copied()
            .ok_or_else(|| RdError::missing_entry(name))
    }

    /// Apêndice B1 `hardRestricted`: o instalador do APK está na allowlist.
    /// Sem isso, `grant()` de permissão hard-restricted não muda o estado.
    pub fn allow_hard_restricted(&mut self, allowed: bool) {
        self.restricted_allowed = allowed;
    }

    /// Runtime grant (diálogo) — dangerous → Granted.
    pub fn grant(&mut self, name: &str) -> RdResult<GrantState> {
        if self.is_frozen(name) {
            return self.current_or_missing(name);
        }
        let hard_blocked = self
            .table
            .get(name)
            .is_some_and(|d| d.is_hard_restricted() && !self.role_covers(d))
            && !self.restricted_allowed;
        if hard_blocked {
            // B1 hardRestricted: sem allowlist do instalador a permissão não é concedível.
            return self.current_or_missing(name);
        }
        let cur = self
            .states
            .get_mut(name)
            .ok_or_else(|| RdError::missing_entry(name))?;
        match cur {
            GrantState::RuntimePending | GrantState::SpecialPending | GrantState::Denied => {
                *cur = GrantState::Granted;
                // conceder limpa o histórico de negativas (Android zera o flag)
                self.denials.remove(name);
                Ok(GrantState::Granted)
            }
            s => Ok(*s),
        }
    }

    /// Deny — pending → Denied (com "don't ask again" simulado pelo estado).
    pub fn deny(&mut self, name: &str) -> RdResult<GrantState> {
        if self.is_frozen(name) {
            return self.current_or_missing(name);
        }
        let is_runtime = self
            .table
            .get(name)
            .is_some_and(|d| d.level == ProtectionLevel::Dangerous);
        let cur = self
            .states
            .get_mut(name)
            .ok_or_else(|| RdError::missing_entry(name))?;
        match cur {
            GrantState::RuntimePending | GrantState::SpecialPending => {
                *cur = GrantState::Denied;
            }
            // nova negativa do diálogo sobre uma permissão já negada: conta
            GrantState::Denied => {}
            s => return Ok(*s),
        }
        if is_runtime && *cur == GrantState::Denied {
            let n = self.denials.entry(name.to_string()).or_insert(0);
            *n = n.saturating_add(1);
        }
        Ok(*cur)
    }

    /// Revoga de volta ao estado inicial da instalação (unused-app auto-revoke usa isto).
    pub fn revoke(&mut self, name: &str) -> RdResult<GrantState> {
        if self.is_frozen(name) {
            return self.current_or_missing(name);
        }
        let def = self.table.get(name).map(Self::initial_state);
        // issue #69: permissão do próprio app volta a AppDefined, não Unknown
        let init = def.unwrap_or(if self.declared.contains(name) {
            GrantState::AppDefined
        } else {
            GrantState::Unknown
        });
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
            if self.table.get(&up.name).is_some() {
                continue;
            }
            // issue #69: declarada no `<permission>` do próprio APK → INFO
            // (é modelo do app, não um nome suspeito), com o nível decodificado.
            if let Some(decl) = manifest
                .declared_permissions
                .iter()
                .find(|d| d.name == up.name)
            {
                let level = decl
                    .protection_level
                    .clone()
                    .unwrap_or_else(|| "não informado".to_string());
                findings.push(AuditFinding {
                    severity: Severity::Info,
                    code: "APP_DEFINED_PERMISSION".into(),
                    message: format!(
                        "permission {} is declared by this APK (<permission>) with protectionLevel={}",
                        up.name, level
                    ),
                    permission: Some(up.name.clone()),
                    component: None,
                });
                continue;
            }
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

#[cfg(test)]
mod gating_tests {
    use super::*;
    use rd_apk::manifest::UsesPermission;

    fn manifest_target(target: u32, perms: &[(&str, Option<u32>, bool)]) -> Manifest {
        Manifest {
            package: "com.t".into(),
            min_sdk: 21,
            target_sdk: target,
            uses_permissions: perms
                .iter()
                .map(|(n, max, sdk23)| UsesPermission {
                    name: format!("android.permission.{n}"),
                    max_sdk: *max,
                    sdk23: *sdk23,
                })
                .collect(),
            ..Default::default()
        }
    }

    /// issue #31: maxSdkVersion vencido → NotRequested; dentro da janela → normal
    #[test]
    fn max_sdk_version_gates_not_requested() {
        let table = PermissionTable::embedded().unwrap();
        let mut m = manifest_target(34, &[("WRITE_EXTERNAL_STORAGE", Some(28), false)]);
        let e = PermissionEngine::install(&m, table.clone());
        assert_eq!(
            e.state("android.permission.WRITE_EXTERNAL_STORAGE"),
            Some(GrantState::NotRequested),
            "target 34 > maxSdk 28: o Android nem registra"
        );
        m.target_sdk = 26;
        let e = PermissionEngine::install(&m, table);
        assert_eq!(
            e.state("android.permission.WRITE_EXTERNAL_STORAGE"),
            Some(GrantState::RuntimePending),
            "target 26 ≤ maxSdk 28: pedida normalmente"
        );
    }

    /// issue #31: since_api acima do targetSdk → NotRequested
    #[test]
    fn since_api_gates_not_requested() {
        let table = PermissionTable::embedded().unwrap();
        let mut m = manifest_target(26, &[("POST_NOTIFICATIONS", None, false)]);
        let e = PermissionEngine::install(&m, table.clone());
        assert_eq!(
            e.state("android.permission.POST_NOTIFICATIONS"),
            Some(GrantState::NotRequested),
            "POST_NOTIFICATIONS since 33 não existe em target 26"
        );
        m.target_sdk = 34;
        let e = PermissionEngine::install(&m, table);
        assert_eq!(
            e.state("android.permission.POST_NOTIFICATIONS"),
            Some(GrantState::RuntimePending),
        );
    }

    /// issue #31: uses-permission-sdk-23 com target < 23 → NotRequested
    #[test]
    fn sdk23_tag_gates_below_23() {
        let table = PermissionTable::embedded().unwrap();
        let m = manifest_target(22, &[("CAMERA", None, true)]);
        let e = PermissionEngine::install(&m, table.clone());
        assert_eq!(
            e.state("android.permission.CAMERA"),
            Some(GrantState::NotRequested),
        );
        let m = manifest_target(23, &[("CAMERA", None, true)]);
        let e = PermissionEngine::install(&m, table);
        assert_eq!(
            e.state("android.permission.CAMERA"),
            Some(GrantState::RuntimePending)
        );
    }

    /// issue #69: permissão declarada no `<permission>` do próprio APK é
    /// modelo do app (não "desconhecida") e a auditoria emite INFO em vez do
    /// falso positivo `UNKNOWN_PERMISSION`.
    #[test]
    fn app_declared_permission_is_not_unknown() {
        use rd_apk::manifest::DeclaredPermission;

        let table = PermissionTable::embedded().unwrap();
        let mut m = manifest_target(34, &[("CAMERA", None, false)]);
        m.uses_permissions.push(UsesPermission {
            name: "com.t.permission.CUSTOM".to_string(),
            max_sdk: None,
            sdk23: false,
        });
        m.uses_permissions.push(UsesPermission {
            name: "com.t.permission.GHOST".to_string(),
            max_sdk: None,
            sdk23: false,
        });
        m.declared_permissions.push(DeclaredPermission {
            name: "com.t.permission.CUSTOM".to_string(),
            protection_level: Some("signature|privileged".to_string()),
            protection_level_raw: Some(18),
            ..Default::default()
        });

        let engine = PermissionEngine::install(&m, table);
        assert_eq!(
            engine.state("com.t.permission.CUSTOM"),
            Some(GrantState::AppDefined),
            "declarada pelo APK: nem unknown, nem auto-grant de terceiro"
        );
        assert_eq!(
            engine.granted_by("com.t.permission.CUSTOM"),
            Some("self-declared")
        );
        assert!(engine
            .state("com.t.permission.CUSTOM")
            .unwrap()
            .is_granted());
        // não declarada em lugar nenhum: continua UNKNOWN_PERMISSION
        assert_eq!(
            engine.state("com.t.permission.GHOST"),
            Some(GrantState::Unknown)
        );

        let findings = engine.audit(&m);
        let codes: Vec<&str> = findings.iter().map(|f| f.code.as_str()).collect();
        assert!(
            codes.contains(&"APP_DEFINED_PERMISSION"),
            "esperava INFO da permissão do app: {codes:?}"
        );
        assert!(
            codes.contains(&"UNKNOWN_PERMISSION"),
            "a realmente desconhecida continua sendo reportada: {codes:?}"
        );
        // e o UNKNOWN_PERMISSION NÃO é o nome declarado pelo app
        for f in findings.iter().filter(|f| f.code == "UNKNOWN_PERMISSION") {
            assert_eq!(
                f.permission.as_deref(),
                Some("com.t.permission.GHOST"),
                "falso positivo sobre permissão declarada pelo APK (issue #69)"
            );
        }
        // a mensagem do INFO traz o protectionLevel decodificado
        let info = findings
            .iter()
            .find(|f| f.code == "APP_DEFINED_PERMISSION")
            .expect("finding APP_DEFINED_PERMISSION");
        assert!(
            info.message
                .contains("protectionLevel=signature|privileged"),
            "mensagem: {}",
            info.message
        );
    }

    /// issue #69: `revoke` de permissão do próprio app volta a AppDefined
    /// (e não a Unknown, que reintroduziria o falso positivo).
    #[test]
    fn revoke_app_declared_permission_keeps_it_defined() {
        use rd_apk::manifest::DeclaredPermission;

        let table = PermissionTable::embedded().unwrap();
        let mut m = manifest_target(34, &[]);
        m.uses_permissions.push(UsesPermission {
            name: "com.t.permission.CUSTOM".to_string(),
            max_sdk: None,
            sdk23: false,
        });
        m.declared_permissions.push(DeclaredPermission {
            name: "com.t.permission.CUSTOM".to_string(),
            protection_level: Some("signature".to_string()),
            protection_level_raw: Some(2),
            ..Default::default()
        });
        let mut engine = PermissionEngine::install(&m, table);
        assert_eq!(
            engine.revoke("com.t.permission.CUSTOM").unwrap(),
            GrantState::AppDefined
        );
    }

    /// issue #68: a tabela precisa bater com as contagens publicadas em
    /// `README.md` e `docs/ROADMAP.md` (total 175). Se o dado mudar, os docs
    /// mudam junto — o teste é o guarda dessa coerência.
    #[test]
    fn table_matches_documented_counts() {
        let table = PermissionTable::embedded().unwrap();
        let lvl = |l: ProtectionLevel| table.permission.iter().filter(|p| p.level == l).count();
        assert_eq!(table.len(), 175, "total publicado em README/ROADMAP");
        assert_eq!(lvl(ProtectionLevel::Dangerous), 43);
        assert_eq!(lvl(ProtectionLevel::Normal), 93);
        assert_eq!(lvl(ProtectionLevel::Signature), 36);
        assert_eq!(lvl(ProtectionLevel::Internal), 3);
        assert_eq!(table.specials().count(), 15);
    }

    /// issue #110 (corrige o remanescente da #68): `BATTERY_STATS` é
    /// `signature|privileged|development` no Android real e NÃO está na
    /// whitelist B5 da spec — apps de terceiros não a recebem. Estado
    /// `DeniedSystem`: nem concedida (inspect não conta em auto-granted) nem
    /// concedível na sessão (grant/deny preservam; revoke restaura).
    #[test]
    fn battery_stats_is_denied_system_outside_b5_whitelist() {
        let table = PermissionTable::embedded().unwrap();
        let def = table
            .get("android.permission.BATTERY_STATS")
            .expect("BATTERY_STATS na tabela");
        assert_eq!(def.level, ProtectionLevel::Signature);
        assert!(
            def.flags.iter().any(|f| f == "development"),
            "flags: {:?}",
            def.flags
        );
        assert!(
            !def.is_b5(),
            "BATTERY_STATS não pode entrar na whitelist B5: {:?}",
            def.flags
        );
        assert!(!def.description.contains("Apêndice B4"));

        let m = manifest_target(34, &[("BATTERY_STATS", None, false)]);
        let mut e = PermissionEngine::install(&m, table);
        assert_eq!(
            e.state("android.permission.BATTERY_STATS"),
            Some(GrantState::DeniedSystem),
            "fora da whitelist B5 o Android real nega a terceiros"
        );
        assert_eq!(e.granted_by("android.permission.BATTERY_STATS"), None);
        assert!(!e
            .state("android.permission.BATTERY_STATS")
            .unwrap()
            .is_granted());

        // nem concedível: grant()/deny() preservam o estado
        assert_eq!(
            e.grant("android.permission.BATTERY_STATS").unwrap(),
            GrantState::DeniedSystem
        );
        assert_eq!(
            e.deny("android.permission.BATTERY_STATS").unwrap(),
            GrantState::DeniedSystem
        );
        // revoke restaura o estado inicial de instalação (DeniedSystem)
        assert_eq!(
            e.revoke("android.permission.BATTERY_STATS").unwrap(),
            GrantState::DeniedSystem
        );
    }

    /// issue #110: signature-class DENTRO da whitelist B5 segue `AutoGranted`
    /// com marcador, e toda entrada signature/internal da tabela versionada
    /// está em exatamente um dos dois lados da regra (sem zona cinza).
    #[test]
    fn signature_entries_are_partitioned_by_b5_whitelist() {
        let table = PermissionTable::embedded().unwrap();
        let m = manifest_target(34, &[("WRITE_SECURE_SETTINGS", None, false)]);
        let e = PermissionEngine::install(&m, table.clone());
        assert_eq!(
            e.state("android.permission.WRITE_SECURE_SETTINGS"),
            Some(GrantState::AutoGranted)
        );

        let mut b5 = 0usize;
        let mut fora = 0usize;
        for def in &table.permission {
            if matches!(
                def.level,
                ProtectionLevel::Signature
                    | ProtectionLevel::SignatureOrSystem
                    | ProtectionLevel::Internal
            ) {
                if def.is_b5() {
                    b5 += 1;
                    assert_ne!(
                        def.name, "BATTERY_STATS",
                        "BATTERY_STATS jamais pode ser b5"
                    );
                } else {
                    fora += 1;
                    assert_eq!(
                        def.name, "BATTERY_STATS",
                        "signature-class fora da whitelist B5 hoje é só BATTERY_STATS: {}",
                        def.name
                    );
                }
            }
        }
        assert!(b5 >= 35, "whitelist B5 coberta no dado: {b5}");
        assert_eq!(fora, 1, "único fora da whitelist: BATTERY_STATS");
    }

    /// issue #31: signature AutoGranted carrega marcador runtime-is-system
    #[test]
    fn signature_autogranted_has_trust_marker() {
        let table = PermissionTable::embedded().unwrap();
        let m = manifest_target(34, &[("WRITE_SECURE_SETTINGS", None, false)]);
        let e = PermissionEngine::install(&m, table.clone());
        assert_eq!(
            e.state("android.permission.WRITE_SECURE_SETTINGS"),
            Some(GrantState::AutoGranted)
        );
        assert_eq!(
            e.granted_by("android.permission.WRITE_SECURE_SETTINGS"),
            Some("runtime-is-system"),
            "agents precisam saber que apps reais teriam essa NEGADA"
        );
        // dangerous não tem marcador
        let m = manifest_target(34, &[("CAMERA", None, false)]);
        let e = PermissionEngine::install(&m, table);
        assert_eq!(e.granted_by("android.permission.CAMERA"), None);
    }
}

#[cfg(test)]
mod appendix_b_behavior_tests {
    use super::*;
    use rd_apk::manifest::UsesPermission;

    #[test]
    fn hard_restricted_sms_is_refused_until_installer_allowlists_it() {
        const SMS: &str = "android.permission.READ_SMS";
        let mut e = engine(34, &["READ_SMS", "CAMERA"]);
        assert_eq!(e.state(SMS), Some(GrantState::RuntimePending));
        // sem allowlist do instalador: o grant não muda o estado
        assert_eq!(e.grant(SMS).unwrap(), GrantState::RuntimePending);
        assert_eq!(e.state(SMS), Some(GrantState::RuntimePending));
        // permissão comum continua concedível
        assert_eq!(e.grant(CAM).unwrap(), GrantState::Granted);
        // com allowlist, o grant volta a valer
        e.allow_hard_restricted(true);
        assert_eq!(e.grant(SMS).unwrap(), GrantState::Granted);
    }

    #[test]
    fn immutable_permission_ignores_grant_deny_and_revoke() {
        let toml = r#"
schema_version = 1
[[permission]]
name = "FAKE_IMMUTABLE"
level = "dangerous"
group = "CAMERA"
since_api = 1
flags = ["runtime", "immutable"]
description = "teste"
"#;
        let table = PermissionTable::from_toml(toml).unwrap();
        let m = Manifest {
            package: "com.t".into(),
            min_sdk: 26,
            target_sdk: 34,
            uses_permissions: vec![UsesPermission {
                name: "android.permission.FAKE_IMMUTABLE".into(),
                max_sdk: None,
                sdk23: false,
            }],
            ..Default::default()
        };
        let mut e = PermissionEngine::install(&m, table);
        let name = "android.permission.FAKE_IMMUTABLE";
        let before = e.state(name).unwrap();
        assert_eq!(e.grant(name).unwrap(), before);
        assert_eq!(e.deny(name).unwrap(), before);
        assert_eq!(e.revoke(name).unwrap(), before);
        assert_eq!(e.state(name), Some(before));
        assert!(e.auto_revoke_unused(365, 90).is_empty());
    }

    #[test]
    fn role_holder_gets_role_permissions_and_loses_them_on_exit() {
        const SMS: &str = "android.permission.READ_SMS";
        let mut e = engine(34, &["READ_SMS", "CAMERA"]);
        assert_eq!(e.state(SMS), Some(GrantState::RuntimePending));
        let changed = e.set_role("SMS", true).unwrap();
        assert_eq!(changed, vec![SMS.to_string()]);
        assert_eq!(e.state(SMS), Some(GrantState::Granted));
        assert!(e.roles_held().contains("SMS"));
        // a permissão não-role continua fora do role
        assert_eq!(e.state(CAM), Some(GrantState::RuntimePending));
        // sair do role reverte a concessão que veio dele
        let changed = e.set_role("SMS", false).unwrap();
        assert_eq!(changed, vec![SMS.to_string()]);
        assert_eq!(e.state(SMS), Some(GrantState::RuntimePending));
        assert!(e.roles_held().is_empty());
    }

    #[test]
    fn role_holding_satisfies_hard_restriction_without_allowlist() {
        const SMS: &str = "android.permission.SEND_SMS";
        let mut e = engine(34, &["SEND_SMS"]);
        e.set_role("SMS", true).unwrap();
        // sem allowlist, mas com role: grant é aceito (no-op porque já concedida)
        assert_eq!(e.grant(SMS).unwrap(), GrantState::Granted);
        e.set_role("SMS", false).unwrap();
        assert_eq!(e.grant(SMS).unwrap(), GrantState::RuntimePending);
    }

    #[test]
    fn role_grant_takes_precedence_over_an_earlier_denial() {
        // negativa do usuário só vale enquanto o role não é assumido; ao assumir,
        // o role concede (B6) — documentado: o role tem precedência sobre a negativa.
        const SMS: &str = "android.permission.READ_SMS";
        let mut e = engine(34, &["READ_SMS"]);
        assert_eq!(e.deny(SMS).unwrap(), GrantState::Denied);
        e.set_role("SMS", true).unwrap();
        assert_eq!(e.state(SMS), Some(GrantState::Granted));
        assert_eq!(e.denial_count(SMS), 0);
    }

    #[test]
    fn unknown_role_is_rejected() {
        let mut e = engine(34, &["READ_SMS"]);
        assert!(e.set_role("NOT_A_ROLE", true).is_err());
        assert!(e.roles_held().is_empty());
    }

    fn engine(target_sdk: u32, perms: &[&str]) -> PermissionEngine {
        let m = Manifest {
            package: "com.t".into(),
            min_sdk: 26,
            target_sdk,
            uses_permissions: perms
                .iter()
                .map(|p| UsesPermission {
                    name: format!("android.permission.{p}"),
                    max_sdk: None,
                    sdk23: false,
                })
                .collect(),
            ..Default::default()
        };
        PermissionEngine::install(&m, PermissionTable::embedded().unwrap())
    }

    const CAM: &str = "android.permission.CAMERA";

    #[test]
    fn first_denial_shows_rationale_second_is_dont_ask_again() {
        let mut e = engine(34, &["CAMERA"]);
        assert!(!e.shows_rationale(CAM), "nunca pedida: sem justificativa");
        e.deny(CAM).unwrap();
        assert_eq!(e.denial_count(CAM), 1);
        assert!(e.shows_rationale(CAM));
        assert!(!e.is_permanently_denied(CAM));
        // segunda negativa: \"don't ask again\"
        e.deny(CAM).unwrap();
        assert_eq!(e.denial_count(CAM), 2);
        assert!(!e.shows_rationale(CAM));
        assert!(e.is_permanently_denied(CAM));
    }

    #[test]
    fn grant_clears_denial_history() {
        let mut e = engine(34, &["CAMERA"]);
        e.deny(CAM).unwrap();
        e.deny(CAM).unwrap();
        e.grant(CAM).unwrap();
        assert_eq!(e.denial_count(CAM), 0);
        assert!(!e.is_permanently_denied(CAM));
    }

    #[test]
    fn rationale_names_lists_only_single_denials() {
        let mut e = engine(34, &["CAMERA", "RECORD_AUDIO"]);
        e.deny(CAM).unwrap();
        e.deny("android.permission.RECORD_AUDIO").unwrap();
        e.deny("android.permission.RECORD_AUDIO").unwrap();
        // CAMERA: uma negativa → justificativa; RECORD_AUDIO: permanente → não
        assert_eq!(e.rationale_names(), vec![CAM.to_string()]);
    }

    #[test]
    fn non_runtime_permissions_are_not_counted() {
        let mut e = engine(34, &["INTERNET"]);
        // INTERNET é normal (auto-concedida): deny não muda estado nem conta
        let st = e.deny("android.permission.INTERNET").unwrap();
        assert_eq!(st, GrantState::AutoGranted);
        assert_eq!(e.denial_count("android.permission.INTERNET"), 0);
    }

    #[test]
    fn auto_revoke_only_for_target_30_plus_and_only_after_threshold() {
        let mut old = engine(29, &["CAMERA"]);
        old.grant(CAM).unwrap();
        assert!(old
            .auto_revoke_unused(400, UNUSED_APP_REVOKE_DAYS)
            .is_empty());
        assert_eq!(old.state(CAM), Some(GrantState::Granted));

        let mut e = engine(34, &["CAMERA", "INTERNET"]);
        e.grant(CAM).unwrap();
        assert!(e.auto_revoke_unused(89, UNUSED_APP_REVOKE_DAYS).is_empty());
        assert_eq!(e.state(CAM), Some(GrantState::Granted));
        let revoked = e.auto_revoke_unused(90, UNUSED_APP_REVOKE_DAYS);
        assert_eq!(revoked, vec![CAM.to_string()]);
        // volta ao estado inicial de runtime; normal continua auto-concedida
        assert_eq!(e.state(CAM), Some(GrantState::RuntimePending));
        assert_eq!(
            e.state("android.permission.INTERNET"),
            Some(GrantState::AutoGranted)
        );
    }

    #[test]
    fn partial_media_grants_visual_selection_only_on_api_34_plus() {
        let v = "android.permission.READ_MEDIA_VISUAL_USER_SELECTED";
        let mut e = engine(
            34,
            &["READ_MEDIA_VISUAL_USER_SELECTED", "READ_MEDIA_IMAGES"],
        );
        assert!(e.grant_partial_media());
        assert_eq!(e.state(v), Some(GrantState::Granted));
        assert_eq!(
            e.state("android.permission.READ_MEDIA_IMAGES"),
            Some(GrantState::RuntimePending),
            "acesso parcial não concede a leitura completa de imagens"
        );

        // since_api=34 > targetSdk 33: o registro nem existe para esse app
        let mut old = engine(33, &["READ_MEDIA_VISUAL_USER_SELECTED"]);
        assert!(!old.grant_partial_media());
        assert_eq!(old.state(v), Some(GrantState::NotRequested));

        let mut none = engine(34, &["CAMERA"]);
        assert!(
            !none.grant_partial_media(),
            "app que não declara a permissão"
        );
    }
}
