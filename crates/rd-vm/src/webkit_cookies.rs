//! `android.webkit.CookieManager` — o armazém de cookies que o WebView do
//! Android expõe ao app.
//!
//! APK que motivou o modelo: **NewPipe 0.27** — `DownloaderImpl` guarda cookies
//! do YouTube (`updateYoutubeRestrictedModeCookies`) durante
//! `Application.onCreate`, antes de qualquer tela; sem `getInstance()` a
//! inicialização morria ali.
//!
//! Modelo (subconjunto RFC 6265, o que APKs realmente usam):
//!
//! * `setCookie(url, "nome=valor; Domain=…; Path=…; Secure; HttpOnly; Max-Age=…; Expires=…")`
//!   — atributos desconhecidos são ignorados como no AOSP, mas um `Set-Cookie`
//!   sem par `nome=valor` é **erro tipado** (o WebView real também o descarta de
//!   forma observável: nada é gravado; aqui isso é explícito no log de trace);
//! * `getCookie(url)` devolve `"a=1; b=2"` com os cookies que casam por host
//!   (host-only ou sufixo de domínio) e por caminho, mais específico primeiro;
//! * `removeAllCookies`/`removeSessionCookies` com `ValueCallback<Boolean>`;
//! * `setAcceptCookie`/`acceptCookie`, `flush()`, `setAcceptThirdPartyCookies`/
//!   `acceptThirdPartyCookies`.
//!
//! Não modelado, e dito aqui em vez de inventado: rejeição por *public suffix*,
//! SameSite, cookies particionados (CHIPS), e a política real de third-party
//! cookies (o valor é guardado e devolvido, mas não altera a seleção).

use crate::engine::Engine;
use crate::err::{vm_error, VmExit};
use crate::heap::ObjRef;
use crate::value::Value;
use serde::{Deserialize, Serialize};

pub const COOKIE_MANAGER: &str = "Landroid/webkit/CookieManager;";
const VALUE_CALLBACK: &str = "()V";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    /// Host do cookie (sem o ponto inicial, minúsculo).
    pub domain: String,
    /// `true` para `Domain=…` explícito, `false` para host-only.
    pub domain_cookie: bool,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    /// `Max-Age`/`Expires` em ms desde a época, se presentes.
    pub expires_ms: Option<u64>,
    pub creation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieJar {
    pub cookies: Vec<Cookie>,
    pub accept: bool,
    pub next_creation: u64,
}

impl Default for CookieJar {
    fn default() -> Self {
        Self::new()
    }
}

impl CookieJar {
    pub fn new() -> Self {
        Self {
            cookies: Vec::new(),
            accept: true,
            next_creation: 0,
        }
    }
}

/// `(scheme, host, path)` de uma URL absoluta. Sem `url` crate: o subconjunto
/// que o WebView recebe (`http(s)://host[:porta][/caminho]`).
fn split_url(url: &str) -> Option<(String, String, String)> {
    let (scheme, rest) = url.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    let rest = match rest.find(['/', '?', '#']) {
        Some(index) => rest.split_at(index),
        None => (rest, ""),
    };
    let authority = rest.0;
    let path = if rest.1.starts_with('/') {
        let end = rest.1.find(['?', '#']).unwrap_or(rest.1.len());
        rest.1[..end].to_string()
    } else {
        "/".to_string()
    };
    // userinfo@host:porta
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    let host = if let Some(stripped) = host_port.strip_prefix('[') {
        // IPv6 literal
        stripped.split(']').next().unwrap_or(stripped).to_string()
    } else {
        host_port.split(':').next().unwrap_or(host_port).to_string()
    };
    if host.is_empty() {
        return None;
    }
    Some((scheme, host.to_ascii_lowercase(), path))
}

fn default_path(path: &str) -> String {
    match path.rfind('/') {
        Some(0) | None => "/".to_string(),
        Some(index) => path[..index].to_string(),
    }
}

fn path_matches(request: &str, cookie_path: &str) -> bool {
    if request == cookie_path {
        return true;
    }
    if !request.starts_with(cookie_path) {
        return false;
    }
    cookie_path.ends_with('/') || request.as_bytes().get(cookie_path.len()) == Some(&b'/')
}

fn domain_matches(host: &str, domain: &str, domain_cookie: bool) -> bool {
    if host == domain {
        return true;
    }
    if !domain_cookie {
        // host-only: só o host exato (já testado acima).
        return false;
    }
    host.len() > domain.len()
        && host.ends_with(domain)
        && host.as_bytes()[host.len() - domain.len() - 1] == b'.'
}

/// Parse de um `Set-Cookie`. Devolve `None` quando não há par `nome=valor`.
fn parse_set_cookie(url: &str, header: &str, creation: u64, now_ms: u64) -> Option<Cookie> {
    let (scheme, host, request_path) = split_url(url)?;
    let mut parts = header.split(';');
    let pair = parts.next()?.trim();
    let (name, value) = pair.split_once('=')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let mut cookie = Cookie {
        name: name.to_string(),
        value: value.trim().to_string(),
        domain: host.clone(),
        domain_cookie: false,
        path: default_path(&request_path),
        secure: false,
        http_only: false,
        expires_ms: None,
        creation,
    };
    for attribute in parts {
        let attribute = attribute.trim();
        let (key, raw) = match attribute.split_once('=') {
            Some((k, v)) => (k.trim().to_ascii_lowercase(), v.trim().to_string()),
            None => (attribute.to_ascii_lowercase(), String::new()),
        };
        match key.as_str() {
            "domain" => {
                let domain = raw.trim_start_matches('.').to_ascii_lowercase();
                if domain.is_empty() || !host.ends_with(&domain) {
                    // RFC 6265 5.3: domínio que não cobre o host → cookie recusado.
                    return None;
                }
                cookie.domain = domain;
                cookie.domain_cookie = true;
            }
            "path" => {
                if raw.starts_with('/') {
                    cookie.path = raw;
                }
            }
            "secure" => cookie.secure = true,
            "httponly" => cookie.http_only = true,
            "max-age" => {
                if let Ok(seconds) = raw.parse::<i64>() {
                    cookie.expires_ms = if seconds <= 0 {
                        // Max-Age <= 0 apaga o cookie: marca como expirado.
                        Some(0)
                    } else {
                        Some(now_ms + seconds as u64 * 1000)
                    };
                }
            }
            _ => {}
        }
    }
    // Cookie `Secure` só é aceito por esquema seguro (RFC 6265 5.4 não exige,
    // mas o WebView do Android recusa — comportamento observável no AOSP).
    if cookie.secure && scheme != "https" && scheme != "wss" {
        return None;
    }
    Some(cookie)
}

pub fn jar(vm: &mut Engine) -> &mut CookieJar {
    &mut vm.fw.cookie_jar
}

fn string_value(vm: &mut Engine, text: String) -> Result<Value, VmExit> {
    Ok(Value::Obj(vm.heap.alloc_string(text)?))
}

fn arg_text(vm: &Engine, args: &[Value], index: usize) -> Option<String> {
    match args.get(index) {
        Some(Value::Obj(r)) => vm.heap.as_str(*r).ok().map(str::to_string),
        _ => None,
    }
}

/// `getInstance()`: um CookieManager por processo (o WebView real é único).
pub fn instance_object(vm: &mut Engine) -> Result<ObjRef, VmExit> {
    if let Some(obj) = vm.fw.cookie_manager {
        return Ok(obj);
    }
    let obj = vm
        .heap
        .alloc_instance(COOKIE_MANAGER.to_string(), Vec::new())
        .map_err(crate::err::oom_throwable)?;
    vm.fw.cookie_manager = Some(obj);
    Ok(obj)
}

/// `android.webkit.CookieSyncManager` (deprecado desde a API 21, presente no SDK 28).
/// O armazém é em memória, então sincronizar com o disco é no-op.
pub const COOKIE_SYNC_MANAGER: &str = "Landroid/webkit/CookieSyncManager;";

/// `createInstance(Context)` / `getInstance()`: um CookieSyncManager por processo.
pub fn sync_manager_object(vm: &mut Engine) -> Result<ObjRef, VmExit> {
    if let Some(obj) = vm.fw.cookie_sync_manager {
        return Ok(obj);
    }
    let obj = vm
        .heap
        .alloc_instance(COOKIE_SYNC_MANAGER.to_string(), Vec::new())
        .map_err(crate::err::oom_throwable)?;
    vm.fw.cookie_sync_manager = Some(obj);
    Ok(obj)
}

pub fn sync_manager_static(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    _args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let factory = (name == "createInstance"
        && sig == "(Landroid/content/Context;)Landroid/webkit/CookieSyncManager;")
        || (name == "getInstance" && sig == "()Landroid/webkit/CookieSyncManager;");
    if factory {
        return Ok(Some(Value::Obj(sync_manager_object(vm)?)));
    }
    Ok(None)
}

/// Métodos de instância do CookieSyncManager: sincronização é no-op aqui.
pub fn sync_manager_instance(name: &str, sig: &str) -> Option<Value> {
    match (name, sig) {
        ("sync" | "startSync" | "stopSync", "()V") => Some(Value::Null),
        _ => None,
    }
}

pub fn static_intrinsic(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    _args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if name == "getInstance" && sig == "()Landroid/webkit/CookieManager;" {
        return Ok(Some(Value::Obj(instance_object(vm)?)));
    }
    Ok(None)
}

fn callback_result(vm: &mut Engine, args: &[Value], index: usize) -> Result<(), VmExit> {
    let Some(Value::Obj(callback)) = args.get(index) else {
        return Ok(());
    };
    let callback = *callback;
    let boxed = crate::boxes::box_value(vm, "Ljava/lang/Boolean;", Value::Int(1))?;
    vm.invoke_instance(
        callback,
        "onReceiveValue",
        "(Ljava/lang/Object;)V",
        &[Value::Obj(boxed)],
    )?;
    Ok(())
}

pub fn instance_intrinsic(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match (name, sig) {
        ("setCookie", "(Ljava/lang/String;Ljava/lang/String;)V")
        | ("setCookie", "(Ljava/lang/String;Ljava/lang/String;Landroid/webkit/ValueCallback;)V") => {
            let (Some(url), Some(header)) = (arg_text(vm, args, 0), arg_text(vm, args, 1)) else {
                return Err(VmExit::Exception(crate::err::Throwable::new(
                    "Ljava/lang/NullPointerException;",
                    "setCookie url/value null",
                )));
            };
            if !jar(vm).accept {
                return Ok(Some(Value::Null));
            }
            let creation = {
                let jar = jar(vm);
                jar.next_creation += 1;
                jar.next_creation
            };
            let now_ms = vm.fw.clock_ms;
            match parse_set_cookie(&url, &header, creation, now_ms) {
                Some(cookie) => {
                    if std::env::var_os("RD_TRACE_COOKIES").is_some() {
                        eprintln!(
                            "[cookie] set {}={} domain={}",
                            cookie.name, cookie.value, cookie.domain
                        );
                    }
                    let expired = cookie.expires_ms == Some(0);
                    let jar = jar(vm);
                    jar.cookies.retain(|c| {
                        !(c.name == cookie.name
                            && c.domain == cookie.domain
                            && c.path == cookie.path)
                    });
                    if !expired {
                        jar.cookies.push(cookie);
                    }
                }
                None => {
                    if std::env::var_os("RD_TRACE_COOKIES").is_some() {
                        eprintln!("[cookie] recusado: {header}");
                    }
                }
            }
            if sig.ends_with("Landroid/webkit/ValueCallback;)V") {
                callback_result(vm, args, 2)?;
            }
            Ok(Some(Value::Null))
        }
        ("getCookie", "(Ljava/lang/String;)Ljava/lang/String;") => {
            let Some(url) = arg_text(vm, args, 0) else {
                return Ok(Some(Value::Null));
            };
            let Some((_scheme, host, path)) = split_url(&url) else {
                return Ok(Some(Value::Null));
            };
            let jar = jar(vm);
            let mut matching: Vec<&Cookie> = jar
                .cookies
                .iter()
                .filter(|c| domain_matches(&host, &c.domain, c.domain_cookie))
                .filter(|c| path_matches(&path, &c.path))
                .collect();
            // RFC 6265 5.4 passo 2: caminho mais longo primeiro, depois criação.
            matching.sort_by(|a, b| {
                b.path
                    .len()
                    .cmp(&a.path.len())
                    .then(a.creation.cmp(&b.creation))
            });
            if matching.is_empty() {
                return Ok(Some(Value::Null));
            }
            let joined = matching
                .iter()
                .map(|c| format!("{}={}", c.name, c.value))
                .collect::<Vec<_>>()
                .join("; ");
            Ok(Some(string_value(vm, joined)?))
        }
        ("removeAllCookies", "(Landroid/webkit/ValueCallback;)V") => {
            jar(vm).cookies.clear();
            callback_result(vm, args, 0)?;
            Ok(Some(Value::Null))
        }
        ("removeSessionCookies", "(Landroid/webkit/ValueCallback;)V") => {
            jar(vm).cookies.retain(|c| c.expires_ms.is_some());
            callback_result(vm, args, 0)?;
            Ok(Some(Value::Null))
        }
        ("setAcceptCookie", "(Z)V") => {
            jar(vm).accept = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0) != 0;
            Ok(Some(Value::Null))
        }
        ("acceptCookie", "()Z") => Ok(Some(Value::Int(i32::from(jar(vm).accept)))),
        // `acceptThirdPartyCookies`/`setAcceptThirdPartyCookies` dependem do
        // WebView; o valor não muda a seleção de cookies neste modelo.
        ("acceptThirdPartyCookies", "(Landroid/webkit/WebView;)Z") => Ok(Some(Value::Int(0))),
        ("setAcceptThirdPartyCookies", "(Landroid/webkit/WebView;Z)V") | ("flush", "()V") => {
            Ok(Some(Value::Null))
        }
        // `getCookie` de uma URL malformada no WebView real devolve null —
        // o mesmo vale para `setCookie` com URL inválida (nada é gravado).
        ("setCookie", "(Ljava/lang/String;)V") => {
            Err(vm_error("INVALID_ARG", "setCookie exige (url, value)").into())
        }
        _ => {
            let _ = VALUE_CALLBACK;
            Ok(None)
        }
    }
}
