//! Apêndice D — `androidx.credentials:credentials` 1.7.0-alpha03 (host-shim, sem provedor).
//!
//! Modela o ponto de entrada `CredentialManager` (interface; a implementação
//! é `CredentialManagerImpl`) nas três chamadas assíncronas com callback que
//! apps usam para pedir ou criar credenciais:
//! - `CredentialManager.create(Context)`;
//! - `getCredentialAsync(Context, GetCredentialRequest, CancellationSignal, Executor, callback)`;
//! - `createCredentialAsync(Context, CreateCredentialRequest, CancellationSignal, Executor, callback)`;
//! - `clearCredentialStateAsync(ClearCredentialStateRequest, CancellationSignal, Executor, callback)`.
//!
//! Sem provedor instalado (o RustyDroid não tem provedor fake; a decisão de
//! produto está em `docs/appendix-d-e.md`):
//! - get → `onError(NoCredentialException)`;
//! - create → `onError(CreateCredentialUnsupportedException)`;
//! - clear → `onResult(null)`, já que não há estado a limpar.
//!
//! Desvios documentados (nunca silenciosos):
//! - O callback é entregue **inline**, antes de a chamada retornar. O executor
//!   é ignorado, como nos demais shims.
//! - A exceção é alocada no host com os campos `type` e `errorMessage` que o
//!   construtor real grava, sem executar o construtor do DEX. `getType()` do
//!   DEX lê esses campos e devolve a constante correta.
//! - Não modelados (caem no DEX da biblioteca): `prepareGetCredentialAsync`,
//!   `getCredentialAsync(PendingGetCredentialHandle, …)`,
//!   `createSettingsPendingIntent`, `signalCredentialState*` e as variantes
//!   `suspend`.
//! - Argumentos obrigatórios nulos → NPE, como o `checkNotNullParameter` da
//!   biblioteca. Executor e `CancellationSignal` são opcionais.

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::heap::ObjRef;
use crate::intrinsics;
use crate::value::Value;

pub const CREDENTIAL_MANAGER: &str = "Landroidx/credentials/CredentialManager;";
/// Classe real da implementação devolvida por `CredentialManager.create`.
pub const CREDENTIAL_MANAGER_IMPL: &str = "Landroidx/credentials/CredentialManagerImpl;";

const NO_CREDENTIAL: &str = "Landroidx/credentials/exceptions/NoCredentialException;";
const CREATE_UNSUPPORTED: &str =
    "Landroidx/credentials/exceptions/CreateCredentialUnsupportedException;";
const NPE: &str = "Ljava/lang/NullPointerException;";

/// Constantes `TYPE_*` das bibliotecas (lidas do bytecode do AAR 1.7.0-alpha03).
const TYPE_NO_CREDENTIAL: &str = "android.credentials.GetCredentialException.TYPE_NO_CREDENTIAL";
const TYPE_CREATE_UNSUPPORTED: &str =
    "androidx.credentials.TYPE_CREATE_CREDENTIAL_UNSUPPORTED_EXCEPTION";

const MSG_NO_CREDENTIAL: &str =
    "Nenhuma credencial disponível: nenhum provedor instalado no RustyDroid";
const MSG_CREATE_UNSUPPORTED: &str =
    "Criação de credencial não suportada: nenhum provedor instalado no RustyDroid";

const CREATE_SIG: &str = "(Landroid/content/Context;)Landroidx/credentials/CredentialManager;";
const GET_SIG: &str = "(Landroid/content/Context;Landroidx/credentials/GetCredentialRequest;Landroid/os/CancellationSignal;Ljava/util/concurrent/Executor;Landroidx/credentials/CredentialManagerCallback;)V";
const CREATE_ASYNC_SIG: &str = "(Landroid/content/Context;Landroidx/credentials/CreateCredentialRequest;Landroid/os/CancellationSignal;Ljava/util/concurrent/Executor;Landroidx/credentials/CredentialManagerCallback;)V";
const CLEAR_SIG: &str = "(Landroidx/credentials/ClearCredentialStateRequest;Landroid/os/CancellationSignal;Ljava/util/concurrent/Executor;Landroidx/credentials/CredentialManagerCallback;)V";
/// Erasure de `CredentialManagerCallback<R, E>`: `onResult(Object)` e `onError(Object)`.
const CALLBACK_ARG_SIG: &str = "(Ljava/lang/Object;)V";

/// `true` quando o receptor é o `CredentialManager` do host.
pub fn is_manager(recv_class: &str) -> bool {
    recv_class == CREDENTIAL_MANAGER_IMPL
}

/// Dispatch estático: só `CredentialManager.create(Context)`. Outros
/// métodos estáticos da interface voltam `None` e seguem para o DEX.
pub fn call_static(
    vm: &mut Engine,
    class: &str,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if class != CREDENTIAL_MANAGER || name != "create" || sig != CREATE_SIG {
        return Ok(None);
    }
    required(args, 0, "context")?;
    let mgr = vm
        .heap
        .alloc_instance(CREDENTIAL_MANAGER_IMPL.to_string(), Vec::new())?;
    Ok(Some(Value::Obj(mgr)))
}

/// Dispatch de instância sobre o `CredentialManagerImpl` do host. Métodos
/// fora do modelo voltam `None` e seguem para o DEX.
pub fn call_instance(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    _recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match (name, sig) {
        ("getCredentialAsync", GET_SIG) => {
            required(args, 0, "context")?;
            required(args, 1, "request")?;
            let cb = required(args, 4, "callback")?;
            let exc = exception(vm, NO_CREDENTIAL, TYPE_NO_CREDENTIAL, MSG_NO_CREDENTIAL)?;
            deliver(vm, cb, "onError", Value::Obj(exc))?;
            Ok(Some(Value::Null))
        }
        ("createCredentialAsync", CREATE_ASYNC_SIG) => {
            required(args, 0, "context")?;
            required(args, 1, "request")?;
            let cb = required(args, 4, "callback")?;
            let exc = exception(
                vm,
                CREATE_UNSUPPORTED,
                TYPE_CREATE_UNSUPPORTED,
                MSG_CREATE_UNSUPPORTED,
            )?;
            deliver(vm, cb, "onError", Value::Obj(exc))?;
            Ok(Some(Value::Null))
        }
        ("clearCredentialStateAsync", CLEAR_SIG) => {
            required(args, 0, "request")?;
            let cb = required(args, 3, "callback")?;
            deliver(vm, cb, "onResult", Value::Null)?;
            Ok(Some(Value::Null))
        }
        _ => Ok(None),
    }
}

/// Argumento obrigatório: nulo ou ausente → NPE.
fn required(args: &[Value], i: usize, what: &str) -> Result<ObjRef, VmExit> {
    match args.get(i) {
        Some(Value::Obj(r)) => Ok(*r),
        _ => Err(VmExit::Exception(Throwable::new(
            NPE,
            format!("{what} null"),
        ))),
    }
}

/// Exceção do host com os campos que o construtor real grava: `type` e
/// `errorMessage`. `message` espelha o texto para `Throwable.getMessage`.
fn exception(vm: &mut Engine, class: &str, ty: &str, msg: &str) -> Result<ObjRef, VmExit> {
    let ty = intrinsics::alloc_string(vm, ty.to_string())?;
    let msg = intrinsics::alloc_string(vm, msg.to_string())?;
    let exc = vm.heap.alloc_instance(
        class.to_string(),
        vec![
            ("type".to_string(), Value::Obj(ty)),
            ("errorMessage".to_string(), Value::Obj(msg)),
            ("message".to_string(), Value::Obj(msg)),
        ],
    )?;
    Ok(exc)
}

/// Entrega ao `CredentialManagerCallback` do app. Callback sem corpo DEX
/// vira erro explícito, não sucesso silencioso.
fn deliver(vm: &mut Engine, cb: ObjRef, method: &str, arg: Value) -> Result<(), VmExit> {
    match crate::m8::call_dex(vm, cb, method, CALLBACK_ARG_SIG, vec![arg])? {
        Some(_) => Ok(()),
        None => Err(crate::err::vm_error(
            "CALLBACK_SEM_CORPO",
            format!("CredentialManagerCallback sem {method}(Object) no DEX do app"),
        )
        .into()),
    }
}
