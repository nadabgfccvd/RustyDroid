//! ADR 025, etapa 1: regressão E2E do `Application.onCreate` do F-Droid.
//!
//! Antes desta etapa, `App.onCreate` falhava em duas frentes do WorkManager:
//! o `WorkDatabase` (Room) nunca rodava `RoomDatabase.init`, e o
//! `WorkRequest$Builder` do shim devolvia null nos setters fluentes. Depois de
//! corrigir essas frentes e as seguintes (setGroup(null), setShowBadge do
//! NotificationChannel), o `onCreate` roda até o fim. A falha que sobra fica
//! na UI (Compose), fora do escopo do ADR 025.
//!
//! Este teste fixa a parte que o ADR cobre: nenhum frame de
//! `App.onCreate` pode aparecer numa exceção. Se uma regressão voltar em
//! WorkManager, Room, builders ou notificações, o teste falha aqui.
//!
//! O APK golden não é versionado (política GH-05). O teste pula quando ele
//! não está presente, como o `readme_inspect`.

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn fdroid_app_oncreate_does_not_throw_through_room_and_workmanager() {
    let apk = repo_root().join("golden-apks/org.fdroid.fdroid.apk");
    if !apk.exists() {
        eprintln!(
            "PULANDO: {} ausente — rode ./golden-apks/fetch.sh para o E2E do onCreate",
            apk.display()
        );
        return;
    }

    let out = Command::new(env!("CARGO_BIN_EXE_rd"))
        .args(["app", "run", "--trace", &apk.to_string_lossy()])
        .output()
        .expect("rd app run executa");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    // Os marcos da regressão: o lateinit do Room e o NPE do WorkRequest.
    assert!(
        !combined.contains("connectionManager has not been initialized"),
        "Room não rodou RoomDatabase.init: {combined}"
    );
    assert!(
        !combined.contains("at Lorg/fdroid/App;->onCreate"),
        "App.onCreate voltou a lançar exceção: {combined}"
    );
    assert!(
        !combined.contains("RepoUpdateWorker$Companion;->scheduleOrCancel"),
        "scheduleOrCancel voltou a falhar (WorkManager): {combined}"
    );
}

/// Regressão do primeiro quadro: `rd app shot` no F-Droid real não pode
/// reportar falha no draw pass (antes, a pilha nativa da thread principal de
/// 8 MiB cortava o Compose em ~400 quadros: `StackOverflowError` e tela branca).
#[test]
fn fdroid_first_frame_draws_without_stack_overflow() {
    let apk = repo_root().join("golden-apks/org.fdroid.fdroid.apk");
    if !apk.exists() {
        eprintln!(
            "PULANDO: {} ausente — rode ./golden-apks/fetch.sh",
            apk.display()
        );
        return;
    }
    let out_png = std::env::temp_dir().join(format!("rd-fdroid-shot-{}.png", std::process::id()));
    let out = Command::new(env!("CARGO_BIN_EXE_rd"))
        .args(["app", "shot", "--out"])
        .arg(&out_png)
        .arg(&apk)
        .output()
        .expect("rd app shot executa");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let _ = std::fs::remove_file(&out_png);
    assert!(out.status.success(), "rd app shot falhou: {stderr}");
    assert!(
        !stderr.contains("draw pass failed"),
        "o primeiro quadro não desenhou: {stderr}"
    );
    assert!(!stderr.contains("StackOverflowError"), "{stderr}");
}
