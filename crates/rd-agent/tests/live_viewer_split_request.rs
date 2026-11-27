//! Regressão do visualizador ao vivo: a requisição HTTP pode chegar em pedaços.
//!
//! `live_serve_conn` precisa ler o cabeçalho até `CRLFCRLF`. Com uma única
//! `read`, a linha de requisição `GET /frame.png` chegando em dois pacotes era
//! interpretada como `/fr` e respondida com 404.

use rd_agent::mcp::AgentSession;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn read_response(c: &mut TcpStream) -> String {
    let mut out = Vec::new();
    c.read_to_end(&mut out).expect("read response");
    String::from_utf8_lossy(&out).into_owned()
}

#[test]
fn live_viewer_parses_request_line_split_across_writes() {
    let mut s = AgentSession::new();
    let addr = s.enable_live("127.0.0.1:0").expect("bind");

    let mut c = TcpStream::connect(addr).expect("connect");
    c.write_all(b"GET /fr").expect("first half");
    c.flush().expect("flush");
    std::thread::sleep(Duration::from_millis(100));
    c.write_all(b"ame.png HTTP/1.1\r\nHost: x\r\n\r\n")
        .expect("second half");

    // Sem frame ainda: 204 para /frame.png. Com a leitura antiga viria 404.
    let out = read_response(&mut c);
    assert!(out.starts_with("HTTP/1.1 204"), "{out}");
}

#[test]
fn live_viewer_parses_headers_split_across_writes() {
    let mut s = AgentSession::new();
    let addr = s.enable_live("127.0.0.1:0").expect("bind");

    let mut c = TcpStream::connect(addr).expect("connect");
    c.write_all(b"GET / HTTP/1.1\r\nHost: x\r\n")
        .expect("first half");
    c.flush().expect("flush");
    std::thread::sleep(Duration::from_millis(100));
    c.write_all(b"\r\n").expect("second half");

    let out = read_response(&mut c);
    assert!(out.starts_with("HTTP/1.1 200"), "{out}");
    assert!(out.contains("RustyDroid"), "página não servida");
}

#[test]
fn live_viewer_html_matches_published_state_schema() {
    let mut s = AgentSession::new();
    let addr = s.enable_live("127.0.0.1:0").expect("bind");

    let mut c = TcpStream::connect(addr).expect("connect");
    c.write_all(b"GET / HTTP/1.1\r\nHost: x\r\n\r\n")
        .expect("request");
    let html = read_response(&mut c);
    assert!(html.starts_with("HTTP/1.1 200"), "{html}");
    assert!(
        !html.contains("s.running.activity"),
        "LIVE_HTML não pode ler s.running.activity (o JSON publicado expõe s.activity no topo)"
    );
    assert!(html.contains("s.activity"), "LIVE_HTML deve ler activity");
    assert!(html.contains("s.size"), "LIVE_HTML deve ler s.size");

    let mut c2 = TcpStream::connect(addr).expect("connect state");
    c2.write_all(b"GET /state.json HTTP/1.1\r\nHost: x\r\n\r\n")
        .expect("request state");
    let state_resp = read_response(&mut c2);
    assert!(state_resp.starts_with("HTTP/1.1 200"), "{state_resp}");
    let body = state_resp.split_once("\r\n\r\n").expect("http body").1;
    let parsed: serde_json::Value = serde_json::from_str(body).expect("valid state JSON");
    assert_eq!(parsed["version"], 0);
}
