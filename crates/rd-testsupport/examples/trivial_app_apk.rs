//! Gera o APK `trivial` usado pelo golden de screenshot (M4).
//!
//! `cargo run -p rd-testsupport --example trivial_app_apk -- golden/screens/trivial-app.apk`
//!
//! Os bytes são determinísticos; `golden_screenshot` (rd-cli) exige que o
//! arquivo commitado seja idêntico ao gerado aqui.

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "golden/screens/trivial-app.apk".to_string());
    if let Err(e) = std::fs::write(&out, rd_testsupport::trivial_app::trivial_app_apk()) {
        eprintln!("escrevendo {out}: {e}");
        std::process::exit(1);
    }
    println!("{out}");
}
