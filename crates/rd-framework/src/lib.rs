//! rd-framework — implementação nativa do subconjunto `android.*` por domínio.
//!
//! Domínios (Apêndice C da spec): fd-ui, fd-content, fd-os, fd-net, fd-media,
//! fd-hardware, fd-system-services, fd-permissions. M0 entrega **fd-permissions**
//! (motor completo, dado versionado), **fd-devices** (perfis/budgets) e
//! **fd-behavior** (comutadores por targetSdk). Os demais domínios são stubs
//! estruturados `NOT_IMPLEMENTED` (Lei 1).

pub mod fd_behavior;
pub mod fd_devices;
pub mod fd_permissions;

pub use fd_behavior::{BehaviorSwitch, SwitchTable};
pub use fd_devices::{DeviceProfile, DeviceTable};
pub use fd_permissions::{GrantState, PermissionDef, PermissionEngine, PermissionTable};

use rd_apk::RdError;

/// Stub estruturado de qualquer domínio ainda não implementado.
pub fn not_implemented(module_id: &str, milestone: &str) -> RdError {
    RdError::not_implemented(module_id, milestone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_follows_error_contract() {
        let e = not_implemented("fd-ui", "M3");
        assert_eq!(e.code, "NOT_IMPLEMENTED");
        assert_eq!(e.module_id, "fd-ui");
    }
}
