// Included in MCP tests (ADR 031): sessões do mesmo arquivo compartilham o APK
// e os DEX montados; o registro não mantém nada vivo; arquivo alterado não é
// reusado; falha de montagem não fica em cache.

fn share_apk_path(tag: &str) -> PathBuf {
    let src = fixture_apk_path();
    let dir = std::env::temp_dir().join(format!(
        "rd-apk-share-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("tmpdir");
    let dst = dir.join("app.apk");
    std::fs::copy(&src, &dst).expect("copia o fixture");
    dst
}

fn share_install(s: &mut AgentSession, path: &Path) {
    let r = s.call_tool("install", &serde_json::json!({"apk": path}));
    assert!(!r.is_error, "install {}: {:?}", path.display(), r.content);
}

fn share_of(s: &AgentSession) -> Arc<apk_share::SharedApk> {
    s.find_installed("com.test.mcp")
        .expect("app instalado")
        .shared
        .clone()
        .expect("install registra o APK compartilhado")
}

#[test]
fn sessions_on_same_file_share_apk_and_dex_arcs() {
    let _guard = SESSION_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let path = share_apk_path("same");
    let mut s1 = AgentSession::new();
    let mut s2 = AgentSession::new();
    share_install(&mut s1, &path);
    share_install(&mut s2, &path);
    let (a, b) = (share_of(&s1), share_of(&s2));
    assert!(Arc::ptr_eq(&a, &b), "mesmo arquivo: mesmo registro");
    assert!(Arc::ptr_eq(&a.apk, &b.apk), "mesmo Arc<Apk>");

    let d1 = a.dexes(dexes_of_apk).expect("monta DEX");
    let d2 = b.dexes(dexes_of_apk).expect("reusa DEX");
    assert!(!d1.is_empty());
    assert_eq!(d1.len(), d2.len());
    for (x, y) in d1.iter().zip(&d2) {
        assert!(Arc::ptr_eq(x, y), "mesmo Arc<Dex> entre sessões");
    }

    // Os dois lançamentos continuam funcionando (mesmo app, mesma tela).
    for s in [&mut s1, &mut s2] {
        let r = s.call_tool("launch", &serde_json::json!({"observe": true}));
        assert!(!r.is_error, "launch: {:?}", r.content);
    }
}

#[test]
fn shared_entry_is_released_when_last_session_closes() {
    let _guard = SESSION_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let path = share_apk_path("drop");
    let key = apk_share::ApkKey::of(&path).expect("chave");
    let mut s1 = AgentSession::new();
    let mut s2 = AgentSession::new();
    share_install(&mut s1, &path);
    share_install(&mut s2, &path);
    assert!(apk_share::lookup(&key).is_some(), "enquanto há sessão");
    drop(s1);
    assert!(apk_share::lookup(&key).is_some(), "uma sessão ainda usa");
    drop(s2);
    assert!(
        apk_share::lookup(&key).is_none(),
        "o registro não mantém o APK vivo"
    );
}

#[test]
fn changed_file_is_not_reused() {
    let _guard = SESSION_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let path = share_apk_path("mtime");
    let mut s1 = AgentSession::new();
    share_install(&mut s1, &path);
    let before = share_of(&s1);
    // Mesmo conteúdo, mtime diferente: outra identidade, sem reuso.
    let later = std::fs::metadata(&path)
        .unwrap()
        .modified()
        .unwrap()
        .checked_add(std::time::Duration::from_secs(60))
        .unwrap();
    std::fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(later)
        .unwrap();
    let mut s2 = AgentSession::new();
    share_install(&mut s2, &path);
    let after = share_of(&s2);
    assert!(!Arc::ptr_eq(&before, &after), "mtime mudou: sem reuso");
}

#[test]
fn failed_dex_mount_is_not_cached() {
    let _guard = SESSION_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let path = share_apk_path("fail");
    let apk = rd_apk::Apk::from_bytes(std::fs::read(&path).unwrap()).expect("apk");
    let shared = apk_share::SharedApk::new(apk, None);
    let mut calls = 0;
    let first = shared.dexes(|_| {
        calls += 1;
        Err(RdError::new("TEST_MOUNT", "falha simulada", MODULE_ID))
    });
    assert!(first.is_err());
    let second = shared.dexes(|a| {
        calls += 1;
        dexes_of_apk(a)
    });
    let third = shared.dexes(|_| {
        calls += 1;
        panic!("depois de montar, não deve montar de novo");
    });
    assert_eq!(calls, 2, "a falha não ficou em cache; o sucesso sim");
    let (s, t) = (second.unwrap(), third.unwrap());
    for (x, y) in s.iter().zip(&t) {
        assert!(Arc::ptr_eq(x, y));
    }
    // Sem engine segurando os DEX, a memória volta: a próxima montagem é nova.
    drop((s, t));
    let fourth = shared.dexes(|a| {
        calls += 1;
        dexes_of_apk(a)
    });
    assert!(fourth.is_ok());
    assert_eq!(calls, 3, "sem Arc vivo, os DEX saem da memória e remontam");
}
