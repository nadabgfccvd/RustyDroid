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
    /// Apêndice B `instant` (`PermissionInfo.PROTECTION_FLAG_INSTANT`): a permissão
    /// pode ser concedida a app instantâneo. Ver `PermissionEngine::set_instant`.
    pub fn is_instant(&self) -> bool {
        self.flags.iter().any(|f| f == "instant")
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
    /// Apêndice B1 `softRestriction`: a plataforma concede a forma completa só
    /// a apps que cumprem a política da plataforma; senão, uma forma fraca
    /// (ver `PermissionEngine::soft_weak_form`).
    pub fn is_soft_restricted(&self) -> bool {
        self.flags.iter().any(|f| f == "softRestriction")
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

/// Split permission (AOSP `platform.xml`, `<split-permission>`): quando o app pede
/// `from`, o sistema adiciona `to` como permissão implícita, se o targetSdk for
/// menor que `target_below`.
#[derive(Debug, Clone, Deserialize)]
pub struct SplitPermission {
    pub from: String,
    pub to: String,
    #[serde(default = "default_split_target_below")]
    pub target_below: u32,
}

fn default_split_target_below() -> u32 {
    SPLIT_ALL_TARGETS
}

#[derive(Debug, Clone, Deserialize)]
pub struct PermissionTable {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub permission: Vec<PermissionDef>,
    #[serde(default)]
    pub split: Vec<SplitPermission>,
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
#[derive(Clone)]
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
    /// Apêndice B6 `revokeWhenRequested`: permissões implícitas (split) que o
    /// sistema concedeu sem o app pedir explicitamente. Chave = nome do estado.
    revoke_when_requested: BTreeSet<String>,
    /// Apêndice B `instant`: app instalado como instantâneo (`install` com `instant`).
    instant: bool,
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

/// API 29 (Android 10): a partir dela, a política de storage libera a forma
/// completa de READ/WRITE_EXTERNAL_STORAGE sem allowlist (ver `soft_may_grant`).
pub const SOFT_STORAGE_FULL_FORM_API: u32 = 29;

/// Forma fraca de uma permissão soft restricted quando a política não libera a
/// forma completa (AOSP `SoftRestrictedPermissionPolicy`; doc oficial de
/// `READ_EXTERNAL_STORAGE`: sem allowlist, só as coleções de mídia visual e áudio).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftWeakForm {
    /// Só acesso às coleções de mídia visual e áudio (READ_EXTERNAL_STORAGE).
    MediaCollectionsOnly,
    /// Nenhum acesso concedido pela permissão (WRITE_EXTERNAL_STORAGE).
    NoAccess,
}

fn short_permission_name(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

/// Prefixo do nome completo de permissão no manifesto.
const ANDROID_PERMISSION_PREFIX: &str = "android.permission.";

/// API 23 (M): abaixo dela não há permissão de runtime (AOSP `supportsRuntimePermissions`).
pub const RUNTIME_PERMISSIONS_API: u32 = 23;

/// AOSP `SystemConfig.readSplitPermission`: split sem `targetSdk` vale para todo app
/// (`CUR_DEVELOPMENT + 1`).
pub const SPLIT_ALL_TARGETS: u32 = 10_001;

/// AOSP `PermissionManagerServiceImpl`: permissões de mídia aural/visual e
/// ACTIVITY_RECOGNITION não recebem a flag `revokeWhenRequested` no split.
const READ_MEDIA_AURAL: &[&str] = &["READ_MEDIA_AUDIO"];
const READ_MEDIA_VISUAL: &[&str] = &[
    "READ_MEDIA_VIDEO",
    "READ_MEDIA_IMAGES",
    "ACCESS_MEDIA_LOCATION",
    "READ_MEDIA_VISUAL_USER_SELECTED",
];

/// AOSP `NEARBY_DEVICES_PERMISSIONS`: preservadas na revogação se o app já tem
/// `ACCESS_BACKGROUND_LOCATION` concedida (sem a flag).
const NEARBY_DEVICES_PERMISSIONS: &[&str] =
    &["BLUETOOTH_ADVERTISE", "BLUETOOTH_CONNECT", "BLUETOOTH_SCAN"];

fn split_gets_revoke_flag(to: &str) -> bool {
    to != "ACTIVITY_RECOGNITION"
        && !READ_MEDIA_AURAL.contains(&to)
        && !READ_MEDIA_VISUAL.contains(&to)
}

/// Permissões implícitas do manifesto: destino de cada split cuja origem o app
/// pede, válido para o targetSdk, e que o app não pede explicitamente (AOSP
/// `ParsingPackageUtils`). Chave = nome curto do destino; valor = origens.
fn implicit_permissions(
    table: &PermissionTable,
    requested: &[String],
    target_sdk: u32,
) -> BTreeMap<String, BTreeSet<String>> {
    let req: BTreeSet<&str> = requested.iter().map(|n| short_permission_name(n)).collect();
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for split in &table.split {
        if target_sdk >= split.target_below {
            continue;
        }
        let from = short_permission_name(&split.from);
        let to = short_permission_name(&split.to);
        if !req.contains(from) || req.contains(to) {
            continue;
        }
        out.entry(to.to_string())
            .or_default()
            .insert(from.to_string());
    }
    out
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
            let state = Self::request_state(&table, &declared, up, manifest.target_sdk);
            states.insert(up.name.clone(), state);
        }
        let mut engine = PermissionEngine {
            table,
            states,
            declared,
            target_sdk: manifest.target_sdk,
            denials: BTreeMap::new(),
            restricted_allowed: false,
            roles_held: BTreeSet::new(),
            role_granted: BTreeSet::new(),
            revoke_when_requested: BTreeSet::new(),
            instant: false,
        };
        engine.apply_implicit(&manifest.uses_permissions, manifest.target_sdk);
        engine
    }

    /// Estado de uma permissão pedida explicitamente pelo manifesto.
    /// issue #31: gating por maxSdkVersion, `uses-permission-sdk-23`, since_api e
    /// existência na tabela.
    fn request_state(
        table: &PermissionTable,
        declared: &BTreeSet<String>,
        up: &rd_apk::manifest::UsesPermission,
        target_sdk: u32,
    ) -> GrantState {
        let gated_out =
            up.max_sdk.is_some_and(|max| target_sdk > max) || (up.sdk23 && target_sdk < 23);
        if gated_out {
            return GrantState::NotRequested;
        }
        match table.get(&up.name) {
            Some(def) if def.since_api as u32 > target_sdk => GrantState::NotRequested,
            Some(def) => Self::initial_state(def),
            // issue #69: ausente da tabela MAS declarada pelo próprio APK não é "desconhecida".
            None if declared.contains(&up.name) => GrantState::AppDefined,
            None => GrantState::Unknown,
        }
    }

    /// Apêndice B6 (split permissions): acrescenta as permissões implícitas que
    /// ainda não têm estado. Runtime implícita recebe a flag `revokeWhenRequested`
    /// (exceto mídia e ACTIVITY_RECOGNITION). Recebe a concessão se alguma origem
    /// já está concedida, ou se o alvo é pré-M: o AOSP concede as permissões de
    /// runtime de apps pré-M na instalação. (As explícitas pré-M continuam
    /// `RuntimePending`: essa é a modelagem existente, não foi alterada aqui.)
    fn apply_implicit(&mut self, requested: &[rd_apk::manifest::UsesPermission], target_sdk: u32) {
        let names: Vec<String> = requested.iter().map(|u| u.name.clone()).collect();
        for (to, sources) in implicit_permissions(&self.table, &names, target_sdk) {
            let key = format!("{ANDROID_PERMISSION_PREFIX}{to}");
            if self.states.contains_key(&key) {
                continue;
            }
            let Some(def) = self.table.get(&to) else {
                continue;
            };
            let level = def.level;
            let inherits = target_sdk < RUNTIME_PERMISSIONS_API
                || sources.iter().any(|s| self.source_is_granted(s));
            let state = if inherits && level == ProtectionLevel::Dangerous {
                GrantState::Granted
            } else {
                Self::initial_state(def)
            };
            self.states.insert(key.clone(), state);
            if level == ProtectionLevel::Dangerous && split_gets_revoke_flag(&to) {
                self.revoke_when_requested.insert(key);
            }
        }
    }

    /// A origem (nome curto) está concedida em algum estado?
    fn source_is_granted(&self, short: &str) -> bool {
        self.states
            .iter()
            .any(|(k, st)| short_permission_name(k) == short && st.is_granted())
    }

    /// Apêndice B `instant` (AOSP `checkSinglePermissionInternalLocked`): num app
    /// instantâneo, uma concessão só conta se a permissão tem a flag `instant`. As
    /// demais viram `DeniedSystem` (não concedidas) e não são concedíveis em runtime.
    /// Só vale na instalação como instantâneo: não há volta para o modo completo.
    pub fn set_instant(&mut self, instant: bool) {
        self.instant = instant;
        if instant {
            self.apply_instant_rule();
        }
    }

    pub fn is_instant(&self) -> bool {
        self.instant
    }

    /// Permissão que o modo instantâneo bloqueia (não tem a flag `instant`).
    fn instant_blocks(&self, name: &str) -> bool {
        self.instant && self.table.get(name).is_some_and(|d| !d.is_instant())
    }

    fn apply_instant_rule(&mut self) {
        let blocked: Vec<String> = self
            .states
            .iter()
            .filter(|(k, st)| {
                self.instant_blocks(k)
                    && matches!(
                        st,
                        GrantState::AutoGranted
                            | GrantState::Granted
                            | GrantState::RuntimePending
                            | GrantState::SpecialPending
                            | GrantState::Denied
                    )
            })
            .map(|(k, _)| k.clone())
            .collect();
        for k in blocked {
            self.states.insert(k.clone(), GrantState::DeniedSystem);
            self.denials.remove(&k);
            self.revoke_when_requested.remove(&k);
        }
    }

    /// Apêndice B6 `revokeWhenRequested`: a permissão é implícita e ainda não foi
    /// pedida explicitamente pelo app (a flag sai quando o app a pede ou o usuário a concede).
    pub fn revoke_when_requested(&self, name: &str) -> bool {
        self.revoke_when_requested.contains(name)
    }

    /// Ciclo de update de pacote (AOSP `revokePermissionsNoLongerImplicitLocked`).
    ///
    /// Aplica o manifesto novo ao motor existente, preservando o estado de cada
    /// permissão já conhecida. Devolve as permissões revogadas: concedidas, com a
    /// flag `revokeWhenRequested`, e que deixaram de ser implícitas (o app passou a
    /// pedi-las explicitamente). Só revoga com targetSdk >= 23 e fora da preservação
    /// de "Nearby devices" (ver `preserves_nearby_grant`). A flag sai sempre que a
    /// permissão era concedida, revogada ou não.
    pub fn update(&mut self, manifest: &Manifest) -> Vec<String> {
        let target = manifest.target_sdk;
        self.declared = manifest
            .declared_permissions
            .iter()
            .map(|d| d.name.clone())
            .collect();
        self.target_sdk = target;
        let names: Vec<String> = manifest
            .uses_permissions
            .iter()
            .map(|u| u.name.clone())
            .collect();
        let still_implicit = implicit_permissions(&self.table, &names, target);

        let flagged_granted: Vec<String> = self
            .revoke_when_requested
            .iter()
            .filter(|k| {
                self.states
                    .get(*k)
                    .is_some_and(|st| *st == GrantState::Granted)
            })
            .cloned()
            .collect();
        let mut revoked = Vec::new();
        for key in flagged_granted {
            let short = short_permission_name(&key).to_string();
            if still_implicit.contains_key(&short) {
                continue;
            }
            self.revoke_when_requested.remove(&key);
            if target < RUNTIME_PERMISSIONS_API || self.preserves_nearby_grant(&key) {
                continue;
            }
            let Some(def) = self.table.get(&short) else {
                continue;
            };
            let init = Self::initial_state(def);
            self.states.insert(key.clone(), init);
            revoked.push(key);
        }

        for up in &manifest.uses_permissions {
            if !self.states.contains_key(&up.name) {
                let st = Self::request_state(&self.table, &self.declared, up, target);
                self.states.insert(up.name.clone(), st);
            }
        }
        self.apply_implicit(&manifest.uses_permissions, target);
        self.apply_instant_rule();
        revoked
    }

    /// AOSP: a permissão "Nearby devices" fica concedida se o app já tem
    /// `ACCESS_BACKGROUND_LOCATION` concedida (e sem a flag).
    fn preserves_nearby_grant(&self, key: &str) -> bool {
        if !NEARBY_DEVICES_PERMISSIONS.contains(&short_permission_name(key)) {
            return false;
        }
        self.states.iter().any(|(k, st)| {
            short_permission_name(k) == "ACCESS_BACKGROUND_LOCATION"
                && *st == GrantState::Granted
                && !self.revoke_when_requested.contains(k)
        })
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
        if self.target_sdk < PARTIAL_MEDIA_API || self.instant_blocks(VISUAL) {
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
                .filter(|k| !self.instant_blocks(k))
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

    /// Apêndice B1 `hardRestricted` e `softRestriction`: o instalador do APK está
    /// na allowlist (`SessionParams.setWhitelistedRestrictedPermissions`, que vale
    /// para as duas classes). Sem isso, `grant()` de permissão hard-restricted não
    /// muda o estado, e a forma completa de soft-restricted não é concedida.
    pub fn allow_hard_restricted(&mut self, allowed: bool) {
        self.restricted_allowed = allowed;
    }

    /// Apêndice B1 `softRestriction`: a política da plataforma libera a forma
    /// completa? Só READ/WRITE_EXTERNAL_STORAGE têm regra própria (AOSP
    /// `SoftRestrictedPermissionPolicy.forPermission`): liberam com allowlist
    /// do instalador ou com targetSdk >= 29. As demais seguem `DUMMY_POLICY`
    /// (concessão plena).
    pub fn soft_may_grant(&self, def: &PermissionDef) -> bool {
        match short_permission_name(&def.name) {
            "READ_EXTERNAL_STORAGE" | "WRITE_EXTERNAL_STORAGE" => {
                self.restricted_allowed || self.target_sdk >= SOFT_STORAGE_FULL_FORM_API
            }
            _ => true,
        }
    }

    /// Apêndice B1 `softRestriction`: forma fraca em vigor para a permissão, ou
    /// `None` se a permissão não é soft-restricted ou a forma completa vale.
    pub fn soft_weak_form(&self, name: &str) -> Option<SoftWeakForm> {
        let def = self.table.get(name)?;
        if !def.is_soft_restricted() || self.soft_may_grant(def) {
            return None;
        }
        match short_permission_name(&def.name) {
            "READ_EXTERNAL_STORAGE" => Some(SoftWeakForm::MediaCollectionsOnly),
            _ => Some(SoftWeakForm::NoAccess),
        }
    }

    /// Runtime grant (diálogo) — dangerous → Granted.
    pub fn grant(&mut self, name: &str) -> RdResult<GrantState> {
        if self.is_frozen(name) || self.instant_blocks(name) {
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
        // B1 softRestriction: sem a regra da plataforma, a forma completa não é
        // concedida pelo runtime (AOSP: "Cannot grant soft restricted permission").
        let soft_blocked = self
            .table
            .get(name)
            .is_some_and(|d| d.is_soft_restricted() && !self.soft_may_grant(d));
        if soft_blocked {
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
                self.revoke_when_requested.remove(name);
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
    fn soft_restricted_storage_is_refused_below_q_without_allowlist() {
        const WRITE: &str = "android.permission.WRITE_EXTERNAL_STORAGE";
        const READ: &str = "android.permission.READ_EXTERNAL_STORAGE";
        let mut e = engine(28, &["WRITE_EXTERNAL_STORAGE", "READ_EXTERNAL_STORAGE"]);
        // sem a regra da plataforma (target < 29, sem allowlist): grant não muda o estado
        assert_eq!(e.grant(WRITE).unwrap(), GrantState::RuntimePending);
        assert_eq!(e.grant(READ).unwrap(), GrantState::RuntimePending);
        // forma fraca: WRITE não dá acesso; READ dá só as coleções de mídia
        assert_eq!(e.soft_weak_form(WRITE), Some(SoftWeakForm::NoAccess));
        assert_eq!(
            e.soft_weak_form(READ),
            Some(SoftWeakForm::MediaCollectionsOnly)
        );
        // com allowlist do instalador, a forma completa é concedida
        e.allow_hard_restricted(true);
        assert_eq!(e.grant(WRITE).unwrap(), GrantState::Granted);
        assert_eq!(e.grant(READ).unwrap(), GrantState::Granted);
        assert_eq!(e.soft_weak_form(WRITE), None);
        assert_eq!(e.soft_weak_form(READ), None);
    }

    #[test]
    fn soft_restricted_storage_is_granted_in_full_from_api_29() {
        const WRITE: &str = "android.permission.WRITE_EXTERNAL_STORAGE";
        let mut e = engine(29, &["WRITE_EXTERNAL_STORAGE"]);
        assert_eq!(e.grant(WRITE).unwrap(), GrantState::Granted);
        assert_eq!(e.soft_weak_form(WRITE), None);
    }

    #[test]
    fn soft_restricted_permission_without_platform_rule_keeps_full_grant() {
        // ACCESS_BACKGROUND_LOCATION tem a flag, mas a política AOSP só tem regra
        // para storage: as demais seguem DUMMY_POLICY (concessão plena).
        const BG: &str = "android.permission.ACCESS_BACKGROUND_LOCATION";
        let mut e = engine(29, &["ACCESS_BACKGROUND_LOCATION"]);
        assert_eq!(e.grant(BG).unwrap(), GrantState::Granted);
        assert_eq!(e.soft_weak_form(BG), None);
    }

    fn manifest_for(target: u32, perms: &[&str]) -> Manifest {
        Manifest {
            package: "com.t".into(),
            min_sdk: 26,
            target_sdk: target,
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

    const READ_EXT: &str = "android.permission.READ_EXTERNAL_STORAGE";
    const BG_LOC: &str = "android.permission.ACCESS_BACKGROUND_LOCATION";
    const BT_SCAN: &str = "android.permission.BLUETOOTH_SCAN";

    #[test]
    fn implicit_grant_is_revoked_when_updated_app_requests_it_explicitly() {
        // pré-M: a implícita READ_EXTERNAL_STORAGE (split de WRITE) nasce concedida e com a flag
        let mut e = PermissionEngine::install(
            &manifest_for(22, &["WRITE_EXTERNAL_STORAGE"]),
            PermissionTable::embedded().unwrap(),
        );
        assert_eq!(e.state(READ_EXT), Some(GrantState::Granted));
        assert!(e.revoke_when_requested(READ_EXT));
        // update para M que pede READ explicitamente: deixa de ser implícita e é revogada
        let revoked = e.update(&manifest_for(
            23,
            &["WRITE_EXTERNAL_STORAGE", "READ_EXTERNAL_STORAGE"],
        ));
        assert_eq!(revoked, vec![READ_EXT.to_string()]);
        assert_eq!(e.state(READ_EXT), Some(GrantState::RuntimePending));
        assert!(!e.revoke_when_requested(READ_EXT));
    }

    #[test]
    fn implicit_grant_is_kept_while_app_still_does_not_request_it() {
        let mut e = PermissionEngine::install(
            &manifest_for(22, &["WRITE_EXTERNAL_STORAGE"]),
            PermissionTable::embedded().unwrap(),
        );
        let revoked = e.update(&manifest_for(23, &["WRITE_EXTERNAL_STORAGE"]));
        assert!(revoked.is_empty());
        assert_eq!(e.state(READ_EXT), Some(GrantState::Granted));
        assert!(e.revoke_when_requested(READ_EXT));
    }

    #[test]
    fn explicit_request_below_m_drops_flag_without_revoking() {
        // sem runtime abaixo de M: a flag sai, a concessão fica
        let mut e = PermissionEngine::install(
            &manifest_for(22, &["WRITE_EXTERNAL_STORAGE"]),
            PermissionTable::embedded().unwrap(),
        );
        let revoked = e.update(&manifest_for(
            22,
            &["WRITE_EXTERNAL_STORAGE", "READ_EXTERNAL_STORAGE"],
        ));
        assert!(revoked.is_empty());
        assert_eq!(e.state(READ_EXT), Some(GrantState::Granted));
        assert!(!e.revoke_when_requested(READ_EXT));
    }

    #[test]
    fn nearby_devices_grant_is_preserved_when_background_location_is_granted() {
        const BLUETOOTH: &[&str] = &["BLUETOOTH", "ACCESS_BACKGROUND_LOCATION"];
        let mut e = PermissionEngine::install(
            &manifest_for(30, BLUETOOTH),
            PermissionTable::embedded().unwrap(),
        );
        // BLUETOOTH_SCAN é implícita (split de BLUETOOTH abaixo de 31) e tem a flag
        assert!(e.revoke_when_requested(BT_SCAN));
        assert_eq!(e.grant(BG_LOC).unwrap(), GrantState::Granted);
        // simula a concessão já feita da permissão implícita
        e.states.insert(BT_SCAN.to_string(), GrantState::Granted);
        let revoked = e.update(&manifest_for(
            30,
            &["BLUETOOTH", "ACCESS_BACKGROUND_LOCATION", "BLUETOOTH_SCAN"],
        ));
        assert!(revoked.is_empty());
        assert_eq!(e.state(BT_SCAN), Some(GrantState::Granted));
        assert!(!e.revoke_when_requested(BT_SCAN));
    }

    #[test]
    fn user_grant_clears_revoke_when_requested_flag() {
        let mut e = PermissionEngine::install(
            &manifest_for(30, &["WRITE_EXTERNAL_STORAGE"]),
            PermissionTable::embedded().unwrap(),
        );
        assert!(e.revoke_when_requested(READ_EXT));
        assert_eq!(e.grant(READ_EXT).unwrap(), GrantState::Granted);
        assert!(!e.revoke_when_requested(READ_EXT));
    }

    #[test]
    fn split_implicit_permissions_depend_on_target_sdk() {
        let t = PermissionTable::embedded().unwrap();
        let req = vec!["android.permission.WRITE_EXTERNAL_STORAGE".to_string()];
        // WRITE -> READ vale para todo app; WRITE -> READ_MEDIA_* só abaixo de 33
        let at32 = implicit_permissions(&t, &req, 32);
        assert!(at32.contains_key("READ_EXTERNAL_STORAGE"));
        assert!(at32.contains_key("READ_MEDIA_IMAGES"));
        let at33 = implicit_permissions(&t, &req, 33);
        assert!(at33.contains_key("READ_EXTERNAL_STORAGE"));
        assert!(!at33.contains_key("READ_MEDIA_IMAGES"));
        // pedir a permissão explícita tira a implícita
        let both = vec![
            "android.permission.WRITE_EXTERNAL_STORAGE".to_string(),
            READ_EXT.to_string(),
        ];
        assert!(!implicit_permissions(&t, &both, 32).contains_key("READ_EXTERNAL_STORAGE"));
    }

    /// AOSP `frameworks/base/core/res/AndroidManifest.xml` (main): permissões com
    /// `protectionLevel` contendo `instant`, exceto as que não estão na tabela
    /// (INSTANT_APP_FOREGROUND_SERVICE, FOREGROUND_SERVICE_FILE_MANAGEMENT).
    const AOSP_INSTANT_PERMISSIONS: &[&str] = &[
        "ACCESS_FINE_LOCATION",
        "ACCESS_COARSE_LOCATION",
        "ACCESS_BACKGROUND_LOCATION",
        "READ_PHONE_NUMBERS",
        "RECORD_AUDIO",
        "ACTIVITY_RECOGNITION",
        "CAMERA",
        "POST_NOTIFICATIONS",
        "INTERNET",
        "ACCESS_NETWORK_STATE",
        "VIBRATE",
        "WAKE_LOCK",
        "FOREGROUND_SERVICE",
        "FOREGROUND_SERVICE_CAMERA",
        "FOREGROUND_SERVICE_CONNECTED_DEVICE",
        "FOREGROUND_SERVICE_DATA_SYNC",
        "FOREGROUND_SERVICE_LOCATION",
        "FOREGROUND_SERVICE_MEDIA_PLAYBACK",
        "FOREGROUND_SERVICE_MEDIA_PROJECTION",
        "FOREGROUND_SERVICE_MICROPHONE",
        "FOREGROUND_SERVICE_PHONE_CALL",
        "FOREGROUND_SERVICE_HEALTH",
        "FOREGROUND_SERVICE_REMOTE_MESSAGING",
        "FOREGROUND_SERVICE_SYSTEM_EXEMPTED",
        "FOREGROUND_SERVICE_MEDIA_PROCESSING",
        "FOREGROUND_SERVICE_SPECIAL_USE",
    ];

    #[test]
    fn instant_flag_in_table_matches_framework_manifest() {
        let t = PermissionTable::embedded().unwrap();
        for p in &t.permission {
            if p.is_instant() {
                assert!(
                    AOSP_INSTANT_PERMISSIONS.contains(&p.name.as_str()),
                    "{} tem a flag instant mas não está no core/res do AOSP",
                    p.name
                );
            }
        }
        for name in AOSP_INSTANT_PERMISSIONS {
            if let Some(p) = t.get(name) {
                assert!(
                    p.is_instant(),
                    "{name} é instant no AOSP e falta a flag no dado"
                );
            }
        }
    }

    #[test]
    fn instant_app_keeps_only_instant_capable_grants() {
        // INTERNET (normal, instant) segue concedida; WIFI_STATE (normal, não instant) e
        // READ_CONTACTS (dangerous, não instant) viram DeniedSystem; CAMERA (instant) fica pendente.
        const WIFI: &str = "android.permission.ACCESS_WIFI_STATE";
        const CONTACTS: &str = "android.permission.READ_CONTACTS";
        const INET: &str = "android.permission.INTERNET";
        let mut e = PermissionEngine::install(
            &manifest_for(
                30,
                &["INTERNET", "ACCESS_WIFI_STATE", "CAMERA", "READ_CONTACTS"],
            ),
            PermissionTable::embedded().unwrap(),
        );
        assert_eq!(e.state(WIFI), Some(GrantState::AutoGranted));
        e.set_instant(true);
        assert!(e.is_instant());
        assert_eq!(e.state(INET), Some(GrantState::AutoGranted));
        assert_eq!(e.state(WIFI), Some(GrantState::DeniedSystem));
        assert_eq!(e.state(CONTACTS), Some(GrantState::DeniedSystem));
        assert_eq!(e.state(CAM), Some(GrantState::RuntimePending));
        let granted: Vec<&String> = e
            .all_states()
            .iter()
            .filter(|(_, s)| s.is_granted())
            .map(|(k, _)| k)
            .collect();
        assert!(granted.iter().any(|k| k.as_str() == INET));
        assert!(!granted
            .iter()
            .any(|k| k.as_str() == WIFI || k.as_str() == CONTACTS));
    }

    #[test]
    fn instant_app_cannot_be_granted_non_instant_permission() {
        const CONTACTS: &str = "android.permission.READ_CONTACTS";
        let mut e = PermissionEngine::install(
            &manifest_for(30, &["READ_CONTACTS", "CAMERA"]),
            PermissionTable::embedded().unwrap(),
        );
        e.set_instant(true);
        // AOSP: concessão de não-instant a app instantâneo é recusada
        assert_eq!(e.grant(CONTACTS).unwrap(), GrantState::DeniedSystem);
        assert_eq!(e.state(CONTACTS), Some(GrantState::DeniedSystem));
        // a permissão instant continua concedível
        assert_eq!(e.grant(CAM).unwrap(), GrantState::Granted);
    }

    #[test]
    fn instant_rule_applies_to_permissions_requested_by_update() {
        const CONTACTS: &str = "android.permission.READ_CONTACTS";
        let mut e = PermissionEngine::install(
            &manifest_for(30, &["INTERNET"]),
            PermissionTable::embedded().unwrap(),
        );
        e.set_instant(true);
        let revoked = e.update(&manifest_for(30, &["INTERNET", "READ_CONTACTS"]));
        assert!(revoked.is_empty());
        assert_eq!(e.state(CONTACTS), Some(GrantState::DeniedSystem));
    }

    #[test]
    fn instant_app_cannot_receive_non_instant_role_permissions() {
        const SMS: &str = "android.permission.READ_SMS";
        let mut e = engine(34, &["READ_SMS", "CAMERA"]);
        e.set_instant(true);
        let changed = e.set_role("SMS", true).unwrap();
        assert!(changed.is_empty());
        assert_eq!(e.state(SMS), Some(GrantState::DeniedSystem));
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
