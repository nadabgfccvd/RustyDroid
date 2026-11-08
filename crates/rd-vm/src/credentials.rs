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
//! Padrão: sem provedor instalado.
//! - get → `onError(NoCredentialException)`;
//! - create → `onError(CreateCredentialUnsupportedException)`;
//! - clear → `onResult(null)`, já que não há estado a limpar.
//!
//! Provedor fake (opt-in, programável pelo teste, como os `gms_*_set_*` do
//! Apêndice E): só afeta `getCredentialAsync`. `credentials_set_password`
//! faz o get devolver um `GetCredentialResponse` com um `PasswordCredential`;
//! `credentials_set_get_failure` faz o get falhar com uma das exceções `Get*`
//! da biblioteca. `credentials_clear_fake` volta ao padrão. Create e clear
//! não mudam com o fake: ainda não há armazenamento para o create.
//! O fake não lê o `GetCredentialRequest`: responde igual a qualquer pedido.
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
const PASSWORD_CREDENTIAL: &str = "Landroidx/credentials/PasswordCredential;";
const GET_RESPONSE: &str = "Landroidx/credentials/GetCredentialResponse;";
const ARRAY_LIST: &str = "Ljava/util/ArrayList;";
const GET_CANCELLED: &str = "Landroidx/credentials/exceptions/GetCredentialCancellationException;";
const GET_INTERRUPTED: &str = "Landroidx/credentials/exceptions/GetCredentialInterruptedException;";
const GET_PROVIDER_CONFIG: &str =
    "Landroidx/credentials/exceptions/GetCredentialProviderConfigurationException;";
const GET_UNKNOWN: &str = "Landroidx/credentials/exceptions/GetCredentialUnknownException;";
/// `Credential.type` de `PasswordCredential` (constante da biblioteca).
const TYPE_PASSWORD: &str = "android.credentials.TYPE_PASSWORD_CREDENTIAL";

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

/// Falha programável de `getCredentialAsync` no provedor fake. Cada uma vira
/// a exceção `Get*` da biblioteca com o `type` que o `getType()` do DEX lê.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GetFailure {
    /// `GetCredentialCancellationException` (usuário cancelou).
    UserCanceled,
    /// `GetCredentialInterruptedException` (retentável).
    Interrupted,
    /// `GetCredentialProviderConfigurationException`.
    ProviderConfiguration,
    /// `GetCredentialUnknownException`.
    Unknown,
}

impl GetFailure {
    /// (classe da exceção, `type` da biblioteca, mensagem).
    fn spec(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::UserCanceled => (
                GET_CANCELLED,
                "android.credentials.GetCredentialException.TYPE_USER_CANCELED",
                "Operação cancelada pelo usuário (provedor fake)",
            ),
            Self::Interrupted => (
                GET_INTERRUPTED,
                "android.credentials.GetCredentialException.TYPE_INTERRUPTED",
                "Operação interrompida (provedor fake)",
            ),
            Self::ProviderConfiguration => (
                GET_PROVIDER_CONFIG,
                "androidx.credentials.TYPE_GET_CREDENTIAL_PROVIDER_CONFIGURATION_EXCEPTION",
                "Configuração do provedor inválida (provedor fake)",
            ),
            Self::Unknown => (
                GET_UNKNOWN,
                "android.credentials.GetCredentialException.TYPE_UNKNOWN",
                "Falha desconhecida (provedor fake)",
            ),
        }
    }
}

/// Resposta do provedor fake para `getCredentialAsync`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FakeGet {
    /// `GetCredentialResponse` com um `PasswordCredential`.
    Password { id: String, password: String },
    /// Falha `Get*`.
    Failure(GetFailure),
}

/// Estado do provedor fake do Apêndice D. `None` = sem provedor (padrão).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CredentialsState {
    pub fake: Option<FakeGet>,
}

impl Engine {
    /// Instala um provedor fake: o próximo `getCredentialAsync` devolve esta senha.
    pub fn credentials_set_password(&mut self, id: &str, password: &str) {
        self.fw.credentials.fake = Some(FakeGet::Password {
            id: id.to_string(),
            password: password.to_string(),
        });
    }

    /// Programa `getCredentialAsync` para falhar com a exceção `Get*` dada.
    pub fn credentials_set_get_failure(&mut self, failure: GetFailure) {
        self.fw.credentials.fake = Some(FakeGet::Failure(failure));
    }

    /// Remove o provedor fake: volta ao padrão sem provedor.
    pub fn credentials_clear_fake(&mut self) {
        self.fw.credentials.fake = None;
    }
}

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
            match vm.fw.credentials.fake.clone() {
                None => {
                    let exc = exception(vm, NO_CREDENTIAL, TYPE_NO_CREDENTIAL, MSG_NO_CREDENTIAL)?;
                    deliver(vm, cb, "onError", Value::Obj(exc))?;
                }
                Some(FakeGet::Failure(f)) => {
                    let (class, ty, msg) = f.spec();
                    let exc = exception(vm, class, ty, msg)?;
                    deliver(vm, cb, "onError", Value::Obj(exc))?;
                }
                Some(FakeGet::Password { id, password }) => {
                    let resp = password_response(vm, &id, &password)?;
                    deliver(vm, cb, "onResult", Value::Obj(resp))?;
                }
            }
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

/// `GetCredentialResponse` com um `PasswordCredential`. Os campos são os que o
/// construtor real grava: `Credential.type`/`data`, `id`/`password` e a lista
/// `credentials`. `data` fica nulo (o `Bundle` não é modelado).
fn password_response(vm: &mut Engine, id: &str, password: &str) -> Result<ObjRef, VmExit> {
    let ty = intrinsics::alloc_string(vm, TYPE_PASSWORD.to_string())?;
    let id = intrinsics::alloc_string(vm, id.to_string())?;
    let pw = intrinsics::alloc_string(vm, password.to_string())?;
    let cred = vm.heap.alloc_instance(
        PASSWORD_CREDENTIAL.to_string(),
        vec![
            ("type".to_string(), Value::Obj(ty)),
            ("data".to_string(), Value::Null),
            ("id".to_string(), Value::Obj(id)),
            ("password".to_string(), Value::Obj(pw)),
        ],
    )?;
    let list = crate::collections::new_list(vm, ARRAY_LIST, vec![Value::Obj(cred)])?;
    let resp = vm.heap.alloc_instance(
        GET_RESPONSE.to_string(),
        vec![("credentials".to_string(), list)],
    )?;
    Ok(resp)
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
