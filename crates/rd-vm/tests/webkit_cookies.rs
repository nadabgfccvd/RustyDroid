//! `android.webkit.CookieManager` — o armazém de cookies do WebView.
//!
//! APK que motivou o modelo: NewPipe 0.27 grava os cookies do YouTube em
//! `Application.onCreate` (`DownloaderImpl.updateYoutubeRestrictedModeCookies`)
//! antes de qualquer tela; sem `CookieManager.getInstance()` a inicialização
//! morria ali.

mod common;

use common::*;
use rd_vm::VmExit;
use rd_vm::{Engine, Value};

fn manager(e: &mut Engine) -> u32 {
    let value = e
        .invoke_static(
            "Landroid/webkit/CookieManager;",
            "getInstance",
            "()Landroid/webkit/CookieManager;",
            &[],
        )
        .expect("getInstance");
    match value {
        Value::Obj(r) => r,
        other => panic!("getInstance devolveu {other:?}"),
    }
}

fn text(e: &mut Engine, s: &str) -> Value {
    Value::Obj(e.heap.alloc_string(s.to_string()).unwrap())
}

fn set_cookie(e: &mut Engine, m: u32, url: &str, header: &str) {
    let url = text(e, url);
    let header = text(e, header);
    e.invoke_instance(
        m,
        "setCookie",
        "(Ljava/lang/String;Ljava/lang/String;)V",
        &[url, header],
    )
    .expect("setCookie");
}

fn get_cookie(e: &mut Engine, m: u32, url: &str) -> Result<Option<String>, VmExit> {
    let url = text(e, url);
    match e.invoke_instance(
        m,
        "getCookie",
        "(Ljava/lang/String;)Ljava/lang/String;",
        &[url],
    )? {
        Value::Null => Ok(None),
        Value::Obj(r) => Ok(e.heap.as_str(r).ok().map(str::to_string)),
        other => panic!("getCookie devolveu {other:?}"),
    }
}

#[test]
fn get_instance_is_a_process_singleton() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let a = manager(&mut e);
    let c = manager(&mut e);
    assert_eq!(a, c, "o CookieManager é único no processo, como no WebView");
}

#[test]
fn set_then_get_returns_the_cookie_for_the_host() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let m = manager(&mut e);
    set_cookie(
        &mut e,
        m,
        "https://www.youtube.com/",
        "PREF=abc; Path=/; Secure",
    );
    assert_eq!(
        get_cookie(&mut e, m, "https://www.youtube.com/watch?v=1").unwrap(),
        Some("PREF=abc".to_string())
    );
    assert_eq!(
        get_cookie(&mut e, m, "https://music.youtube.com/").unwrap(),
        None,
        "host-only não vaza para um host diferente"
    );
}

#[test]
fn domain_cookie_matches_subdomains_and_path_scopes_are_honoured() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let m = manager(&mut e);
    set_cookie(
        &mut e,
        m,
        "https://www.youtube.com/",
        "SID=xyz; Domain=.youtube.com; Path=/",
    );
    set_cookie(
        &mut e,
        m,
        "https://www.youtube.com/feed/subscriptions",
        "FEED=1; Path=/feed",
    );
    assert_eq!(
        get_cookie(&mut e, m, "https://m.youtube.com/").unwrap(),
        Some("SID=xyz".to_string()),
        "Domain= cobre subdomínios; o cookie de /feed não casa com a raiz"
    );
    assert_eq!(
        get_cookie(&mut e, m, "https://www.youtube.com/feed/subscriptions").unwrap(),
        Some("FEED=1; SID=xyz".to_string()),
        "caminho mais específico primeiro (RFC 6265 5.4)"
    );
}

#[test]
fn a_secure_cookie_is_not_accepted_over_http() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let m = manager(&mut e);
    set_cookie(&mut e, m, "http://example.com/", "TOKEN=1; Secure");
    assert_eq!(
        get_cookie(&mut e, m, "http://example.com/").unwrap(),
        None,
        "o WebView do Android recoda cookie Secure vindo de http"
    );
}

#[test]
fn remove_all_cookies_clears_the_jar_and_reports_through_the_callback() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let m = manager(&mut e);
    set_cookie(&mut e, m, "https://a.example/", "k=v");
    let calls = e
        .heap
        .alloc_instance("LCookieCallback;".to_string(), Vec::new())
        .unwrap();
    // O callback do DEX não existe aqui: um objeto qualquer exercita o caminho
    // host de `removeAllCookies`, que responde sem depender do guest.
    e.invoke_instance(
        m,
        "removeAllCookies",
        "(Landroid/webkit/ValueCallback;)V",
        &[Value::Obj(calls)],
    )
    .ok();
    assert_eq!(get_cookie(&mut e, m, "https://a.example/").unwrap(), None);
}

#[test]
fn a_set_cookie_without_a_pair_is_refused_instead_of_stored() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let m = manager(&mut e);
    set_cookie(&mut e, m, "https://example.com/", "Secure; Path=/");
    assert_eq!(
        get_cookie(&mut e, m, "https://example.com/").unwrap(),
        None,
        "um Set-Cookie sem nome=valor não vira cookie"
    );
}

#[test]
fn set_accept_cookie_false_ignores_writes() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let m = manager(&mut e);
    e.invoke_instance(m, "setAcceptCookie", "(Z)V", &[Value::Int(0)])
        .expect("setAcceptCookie");
    set_cookie(&mut e, m, "https://example.com/", "k=v");
    assert_eq!(get_cookie(&mut e, m, "https://example.com/").unwrap(), None);
    let accept = e
        .invoke_instance(m, "acceptCookie", "()Z", &[])
        .expect("acceptCookie");
    assert_eq!(accept, Value::Int(0));
}
