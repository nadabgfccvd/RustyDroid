//! fd-behavior — comutadores de comportamento por targetSdk 26→36 (PRF-07).
//!
//! O mesmo APK com target 26 vs 36 tem comportamentos distintos (Lei 9).
//! Fonte: `data/behavior-switches.toml`.

use rd_apk::{RdError, RdResult};
use serde::{Deserialize, Serialize};

pub const MODULE_ID: &str = "fd-behavior";
pub const EMBEDDED_BEHAVIOR_TOML: &str = include_str!("../../../data/behavior-switches.toml");

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BehaviorSwitch {
    pub id: String,
    /// targetSdk a partir do qual o switch ativa.
    pub api: u32,
    pub title: String,
    /// storage | network | notifications | alarms | foreground-service | ui | media | security | lifecycle
    pub impact: String,
    /// enforced | opt-in | opt-out | info
    pub default_state: String,
    pub summary: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SwitchTable {
    #[serde(default = "one")]
    pub schema_version: u32,
    #[serde(default)]
    pub switch: Vec<BehaviorSwitch>,
}

fn one() -> u32 {
    1
}

impl SwitchTable {
    pub fn from_toml(contents: &str) -> RdResult<Self> {
        toml::from_str(contents).map_err(|e| {
            RdError::new(
                "DATA_ERROR",
                format!("behavior-switches.toml: {e}"),
                MODULE_ID,
            )
        })
    }

    pub fn embedded() -> RdResult<Self> {
        Self::from_toml(EMBEDDED_BEHAVIOR_TOML)
    }

    pub fn load(dir: Option<&std::path::Path>) -> RdResult<Self> {
        match dir {
            Some(d) => {
                let path = d.join("behavior-switches.toml");
                if path.exists() {
                    let s = std::fs::read_to_string(&path)
                        .map_err(|e| RdError::io(&e, format!("reading {}", path.display())))?;
                    Self::from_toml(&s)
                } else {
                    Self::embedded()
                }
            }
            None => Self::embedded(),
        }
    }

    /// Switches que **ativam** para este targetSdk (api ≤ target), ordenados por API.
    pub fn active_for(&self, target_sdk: u32) -> Vec<&BehaviorSwitch> {
        let mut v: Vec<&BehaviorSwitch> =
            self.switch.iter().filter(|s| s.api <= target_sdk).collect();
        v.sort_by_key(|s| (s.api, s.id.clone()));
        v
    }

    /// Comportamentos AINDA NÃO ativados neste target (visibilidade futura).
    pub fn inactive_for(&self, target_sdk: u32) -> Vec<&BehaviorSwitch> {
        let mut v: Vec<&BehaviorSwitch> =
            self.switch.iter().filter(|s| s.api > target_sdk).collect();
        v.sort_by_key(|s| (s.api, s.id.clone()));
        v
    }

    pub fn impacts(&self) -> Vec<String> {
        let mut v: Vec<String> = self.switch.iter().map(|s| s.impact.clone()).collect();
        v.sort();
        v.dedup();
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_table_parses_and_ranges_hold() {
        let t = SwitchTable::embedded().unwrap();
        assert!(t.switch.len() >= 15, "{} switches", t.switch.len());
        for s in &t.switch {
            assert!(
                (26..=36).contains(&s.api),
                "{}: api {} fora do piso/teto",
                s.id,
                s.api
            );
        }
    }

    #[test]
    fn target_26_gets_fewer_switches_than_36() {
        let t = SwitchTable::embedded().unwrap();
        let low = t.active_for(26).len();
        let high = t.active_for(36).len();
        assert!(low < high, "26→{} vs 36→{}", low, high);
        // e os de 29+ não ativam em 26
        assert!(t.active_for(26).iter().all(|s| s.api <= 26));
    }
}
