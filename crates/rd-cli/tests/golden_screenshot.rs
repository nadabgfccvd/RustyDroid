//! Golden de screenshot (M4): trava a regressão do `rd app shot` sobre o APK
//! `golden/screens/trivial-app.apk` contra `golden/screens/trivial-app.png`.
//!
//! ATENÇÃO — o que isto prova: que a saída *não muda* sem aviso (regressão).
//! NÃO prova fidelidade ao Android: não há referência de aparelho real para
//! este app. A fonte é DejaVu Sans, não Roboto.
//!
//! Atualizar o golden (só com justificativa no PR):
//!   RD_UPDATE_GOLDEN=1 cargo test -p rd-cli --test golden_screenshot

use std::path::PathBuf;
use std::process::Command;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../golden/screens")
}

#[test]
fn trivial_app_screenshot_matches_committed_golden() {
    let dir = golden_dir();
    let apk_path = dir.join("trivial-app.apk");
    let png_path = dir.join("trivial-app.png");
    let update = std::env::var("RD_UPDATE_GOLDEN").is_ok_and(|v| v == "1");

    // 1) o APK commitado é exatamente o fixture gerado pelo rd-testsupport
    let apk_bytes = rd_testsupport::trivial_app::trivial_app_apk();
    if update {
        std::fs::write(&apk_path, &apk_bytes).expect("escrever APK golden");
    }
    let committed_apk = std::fs::read(&apk_path).expect("golden/screens/trivial-app.apk");
    assert!(
        committed_apk == apk_bytes,
        "golden APK divergente do fixture (rode com RD_UPDATE_GOLDEN=1 se a mudança for intencional)"
    );

    // 2) renderiza pelo binário real (mesmo caminho do usuário)
    let out = std::env::temp_dir().join(format!("rd-golden-shot-{}.png", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_rd"))
        .args(["app", "shot"])
        .arg(&apk_path)
        .arg("--out")
        .arg(&out)
        .status()
        .expect("rodar rd app shot");
    assert!(status.success(), "rd app shot falhou: {status}");
    let got = std::fs::read(&out).expect("PNG gerado");
    let _ = std::fs::remove_file(&out);

    if update {
        std::fs::write(&png_path, &got).expect("escrever PNG golden");
        return;
    }
    let want = std::fs::read(&png_path)
        .expect("golden/screens/trivial-app.png (RD_UPDATE_GOLDEN=1 para gerar)");
    assert!(
        got == want,
        "screenshot diverge do golden ({} vs {} bytes). Se for intencional, atualize com RD_UPDATE_GOLDEN=1.",
        got.len(),
        want.len()
    );
}
