// ADR 023, etapa 1 (sessões): N sessões de app por processo. Cada sessão é um
// `AgentSession` próprio, com engine, disco virtual, logs, permissões e tela
// isolados. Incluído em `mcp.rs`, como os demais módulos de tools.
//
// Roteamento: uma tool com `engine_id` vai para a sessão desse id. Sem
// `engine_id`, a tool vai para a sessão padrão (a primeira, criada com o
// servidor), então clientes atuais não mudam. Os resultados continuam
// carregando `sessionId` (a sessão que atendeu).

/// Teto padrão de sessões abertas no processo (ADR 023, decisão 5).
pub const DEFAULT_MAX_SESSIONS: usize = 2;

/// Tools que a camada de sessões atende. Não passam por `AgentSession`.
const SESSION_TOOLS: [&str; 3] = ["session_open", "session_list", "session_close"];

/// Tools que injetam entrada no app: toque, gesto, texto, tecla e lote (que
/// pode conter toques). Todas passam pelo `InputRouter` (ADR 023, decisão 3).
const INPUT_TOOLS: [&str; 7] = ["tap", "swipe", "tap_view", "tap_text", "type_text", "key", "batch"];

/// Quantos eventos de entrada o roteador guarda (os mais recentes).
const INPUT_LOG_CAP: usize = 256;

/// Um evento de entrada roteado: a sessão de destino e se o app aceitou.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputEvent {
    pub seq: u64,
    pub session_id: String,
    pub tool: String,
    pub accepted: bool,
}

/// Roteador de entrada do hub (ADR 023, decisão 3). Toda entrada injetada num
/// app passa por aqui e fica registrada com a sessão de destino, na ordem em
/// que chegou. Não é o roteador do M10: o M10 não tem roteador de entrada.
#[derive(Default)]
pub struct InputRouter {
    seq: u64,
    log: std::collections::VecDeque<InputEvent>,
}

impl InputRouter {
    fn record(&mut self, session_id: &str, tool: &str, accepted: bool) -> u64 {
        self.seq += 1;
        if self.log.len() == INPUT_LOG_CAP {
            self.log.pop_front();
        }
        self.log.push_back(InputEvent {
            seq: self.seq,
            session_id: session_id.to_string(),
            tool: tool.to_string(),
            accepted,
        });
        self.seq
    }

    /// Eventos guardados, do mais antigo para o mais recente.
    pub fn events(&self) -> impl Iterator<Item = &InputEvent> {
        self.log.iter()
    }

    /// Número de sequência do último evento roteado para esta sessão.
    pub fn last_seq_for(&self, session_id: &str) -> Option<u64> {
        self.log.iter().rev().find(|e| e.session_id == session_id).map(|e| e.seq)
    }
}

fn max_sessions_from_env() -> usize {
    std::env::var("RD_MCP_MAX_SESSIONS")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_MAX_SESSIONS)
        .clamp(1, 8)
}

/// Orçamento de heap de uma sessão, em bytes (ADR 023, decisão 5).
///
/// O teto é o do processo (`ram_process_cap_mb`, ADR 015) dividido por N
/// sessões, e o resultado nunca passa do heap do app (`ram_app_heap_mb`). Como
/// cada sessão recebe no máximo `teto / N` (com arredondamento para baixo),
/// a soma dos orçamentos de N sessões nunca passa do teto do processo.
pub fn session_heap_budget_bytes(process_cap_mb: u32, app_heap_mb: u32, max_sessions: usize) -> usize {
    let share_mb = u64::from(process_cap_mb) / max_sessions.max(1) as u64;
    let mb = share_mb.min(u64::from(app_heap_mb));
    (mb * 1024 * 1024) as usize
}

/// Conjunto de sessões de app do processo. A posição 0 é a sessão padrão.
pub struct SessionHub {
    sessions: Vec<AgentSession>,
    max_sessions: usize,
    /// Orçamento de heap de cada sessão deste hub (bytes).
    heap_budget: usize,
    /// Roteador de entrada: registra toda entrada injetada (ADR 023, decisão 3).
    pub input: InputRouter,
}

impl Default for SessionHub {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionHub {
    /// Servidor `rd mcp`: limite pela env `RD_MCP_MAX_SESSIONS` (padrão 2).
    pub fn new() -> Self {
        Self::with_limit(max_sessions_from_env())
    }

    pub fn with_limit(max_sessions: usize) -> Self {
        let max_sessions = max_sessions.clamp(1, 8);
        // Perfil de piso de hardware (moto-e5): o mesmo que `rd perf` mede.
        let floor = DeviceTable::embedded()
            .and_then(|t| t.hardware_floor().map(|p| (p.ram_process_cap_mb, p.ram_app_heap_mb)))
            .expect("devices.toml embutido tem o piso de hardware");
        let heap_budget = session_heap_budget_bytes(floor.0, floor.1, max_sessions);
        let mut first = AgentSession::new();
        first.set_heap_budget(heap_budget);
        SessionHub {
            sessions: vec![first],
            max_sessions,
            heap_budget,
            input: InputRouter::default(),
        }
    }

    /// Sessão padrão: a que atende tools sem `engine_id`.
    pub fn default_session(&mut self) -> &mut AgentSession {
        &mut self.sessions[0]
    }

    fn default_id(&self) -> &str {
        &self.sessions[0].session_id
    }

    fn index_of(&self, engine_id: &str) -> Option<usize> {
        self.sessions.iter().position(|s| s.session_id == engine_id)
    }

    fn open(&mut self) -> Result<String, RdError> {
        if self.sessions.len() >= self.max_sessions {
            return Err(agent_error(
                "SESSION_LIMIT",
                format!(
                    "há {} sessões abertas (limite {})",
                    self.sessions.len(),
                    self.max_sessions
                ),
                Some("feche uma com `session_close` ou aumente RD_MCP_MAX_SESSIONS (máx. 8)"),
            ));
        }
        let mut s = AgentSession::new();
        s.set_heap_budget(self.heap_budget);
        // o id sai do relógio; o sufixo garante unicidade dentro do processo
        s.session_id = format!("{}-{}", s.session_id, self.sessions.len());
        let id = s.session_id.clone();
        self.sessions.push(s);
        Ok(id)
    }

    fn close(&mut self, engine_id: &str) -> Result<(), RdError> {
        let i = self.index_of(engine_id).ok_or_else(|| session_not_found(engine_id))?;
        if i == 0 {
            return Err(agent_error(
                "INVALID_ARG",
                "a sessão padrão não pode ser fechada",
                Some("ela fecha junto com o servidor; feche só as abertas com `session_open`"),
            ));
        }
        self.sessions.remove(i);
        Ok(())
    }

    fn list_json(&self) -> serde_json::Value {
        serde_json::json!({
            "max_sessions": self.max_sessions,
            "sessions": self.sessions.iter().enumerate().map(|(i, s)| serde_json::json!({
                "engine_id": s.session_id,
                "default": i == 0,
                "active_package": s.active,
                "running_package": s.running.engine_package,
                "last_input_seq": self.input.last_seq_for(&s.session_id),
            })).collect::<Vec<_>>(),
        })
    }

    /// Notificações de recurso de todas as sessões (o servidor as envia após a resposta).
    fn take_notifications(&mut self) -> Vec<serde_json::Value> {
        self.sessions
            .iter_mut()
            .flat_map(|s| s.take_notifications())
            .collect()
    }
}

fn session_not_found(engine_id: &str) -> RdError {
    agent_error(
        "SESSION_NOT_FOUND",
        format!("nenhuma sessão aberta com engine_id `{engine_id}`"),
        Some("use `session_list` para ver as sessões abertas"),
    )
}

/// Entrada do servidor com sessões. Igual a `handle_message`, mas cada tool
/// vai para a sessão indicada por `engine_id` (ou para a padrão).
fn handle_message_hub(hub: &mut SessionHub, line: &str) -> Option<Result<serde_json::Value, serde_json::Value>> {
    let msg: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        // erro de parse: a resposta é a do caminho comum
        Err(_) => return handle_message(hub.default_session(), line),
    };
    match msg {
        serde_json::Value::Array(items) if !items.is_empty() => {
            let mut responses = Vec::new();
            for item in items {
                match handle_single_hub(hub, item) {
                    None => continue,
                    Some(Ok(v)) | Some(Err(v)) => responses.push(v),
                }
            }
            if responses.is_empty() {
                None
            } else {
                Some(Ok(serde_json::Value::Array(responses)))
            }
        }
        // lote vazio: erro -32600 do caminho comum
        serde_json::Value::Array(_) => handle_message(hub.default_session(), line),
        other => handle_single_hub(hub, other),
    }
}

fn handle_single_hub(hub: &mut SessionHub, msg: serde_json::Value) -> Option<Result<serde_json::Value, serde_json::Value>> {
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("").to_string();
    if method != "tools/call" {
        return handle_single(hub.default_session(), msg);
    }
    let id = msg.get("id").cloned();
    let name = msg
        .pointer("/params/name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let mut args = msg
        .pointer("/params/arguments")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let default_id = hub.default_id().to_string();

    if SESSION_TOOLS.contains(&name.as_str()) {
        let result = hub_session_tool(hub, &name, &args);
        return reply(id, tool_call_envelope(result, &default_id, None));
    }

    let target = match args.get("engine_id") {
        None | Some(serde_json::Value::Null) => 0,
        Some(v) => match v.as_str() {
            None => {
                let e = agent_error("INVALID_FORMAT", "engine_id deve ser uma string", None);
                return reply(id, tool_call_envelope(ToolResult::error(&e), &default_id, None));
            }
            Some(engine_id) => match hub.index_of(engine_id) {
                Some(i) => i,
                None => {
                    let e = session_not_found(engine_id);
                    return reply(id, tool_call_envelope(ToolResult::error(&e), &default_id, None));
                }
            },
        },
    };
    // `engine_id` é de roteamento: a tool roteada não o vê
    if let Some(map) = args.as_object_mut() {
        map.remove("engine_id");
    }
    let mut routed = msg;
    if let Some(p) = routed.get_mut("params").and_then(|p| p.as_object_mut()) {
        p.insert("arguments".into(), args);
    }
    if INPUT_TOOLS.contains(&name.as_str()) {
        // A entrada passa pelo roteador: o evento fica com a sessão de destino.
        let target_id = hub.sessions[target].session_id.clone();
        let out = handle_single(&mut hub.sessions[target], routed);
        let accepted = matches!(&out, Some(Ok(v)) if !v["result"]["isError"].as_bool().unwrap_or(true));
        hub.input.record(&target_id, &name, accepted);
        return out;
    }
    handle_single(&mut hub.sessions[target], routed)
}

/// Resposta de uma requisição; notificação (sem `id`) não responde.
fn reply(
    id: Option<serde_json::Value>,
    result: serde_json::Value,
) -> Option<Result<serde_json::Value, serde_json::Value>> {
    if id.is_none() {
        None
    } else {
        Some(Ok(jsonrpc_result(id, result)))
    }
}

/// Envelope de `tools/call` (mesmo formato de `handle_single`).
fn tool_call_envelope(result: ToolResult, session_id: &str, package: Option<&str>) -> serde_json::Value {
    let content: Vec<serde_json::Value> = result
        .content
        .iter()
        .map(|c| match c {
            ToolContent::Text(t) => serde_json::json!({"type": "text", "text": t}),
            ToolContent::Image { data_b64, mime } => serde_json::json!({
                "type": "image", "data": data_b64, "mimeType": mime,
            }),
        })
        .collect();
    let mut v = serde_json::json!({
        "content": content,
        "isError": result.is_error,
        "sessionId": session_id,
    });
    if let Some(pkg) = package {
        v["package"] = serde_json::json!(pkg);
    }
    v
}

fn hub_session_tool(hub: &mut SessionHub, name: &str, args: &serde_json::Value) -> ToolResult {
    let outcome: Result<serde_json::Value, RdError> = match name {
        "session_open" => hub
            .open()
            .map(|id| serde_json::json!({"engine_id": id, "max_sessions": hub.max_sessions})),
        "session_list" => Ok(hub.list_json()),
        _ => match args.get("engine_id").and_then(|v| v.as_str()) {
            Some(id) => hub.close(id).map(|()| serde_json::json!({"closed": id})),
            None => Err(agent_error(
                "INVALID_FORMAT",
                "session_close exige `engine_id` (string)",
                Some("veja os ids com `session_list`"),
            )),
        },
    };
    match outcome {
        Ok(v) => ToolResult::json(&v),
        Err(e) => ToolResult::error(&e),
    }
}

#[cfg(test)]
mod session_hub_tests {
    use super::*;

    fn rpc(hub: &mut SessionHub, method: &str, name: &str, args: serde_json::Value) -> serde_json::Value {
        let line = serde_json::json!({
            "jsonrpc": "2.0", "id": 1, "method": method,
            "params": {"name": name, "arguments": args},
        })
        .to_string();
        handle_message_hub(hub, &line)
            .expect("requisição com id responde")
            .expect("sem erro JSON-RPC")
    }

    /// Conteúdo textual do envelope de tools/call, decodificado.
    fn payload(resp: &serde_json::Value) -> serde_json::Value {
        let text = resp["result"]["content"][0]["text"].as_str().expect("texto");
        serde_json::from_str(text).unwrap_or_else(|_| serde_json::json!({"raw": text}))
    }

    fn is_error(resp: &serde_json::Value) -> bool {
        resp["result"]["isError"].as_bool().unwrap_or(false)
    }

    fn apk_on_disk() -> String {
        let bytes = rd_testsupport::apkfix::build_apk_sdk(
            "com.test.hub",
            "com.test.hub.Main",
            rd_testsupport::apkfix::ArscFix::new().build(),
            vec![],
            26,
            30,
        );
        let dir = std::env::temp_dir().join(format!(
            "rd-hub-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("tmpdir");
        let path = dir.join("hub.apk");
        std::fs::write(&path, bytes).expect("apk");
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn session_open_respects_the_limit_and_list_puts_the_default_first() {
        let _guard = crate::mcp::tests::SESSION_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut hub = SessionHub::with_limit(2);
        let opened = payload(&rpc(&mut hub, "tools/call", "session_open", serde_json::json!({})));
        let second = opened["engine_id"].as_str().expect("engine_id").to_string();
        let over = rpc(&mut hub, "tools/call", "session_open", serde_json::json!({}));
        assert!(is_error(&over), "terceira sessão passa do limite: {over}");
        assert_eq!(payload(&over)["code"], "SESSION_LIMIT", "{over}");

        let list = payload(&rpc(&mut hub, "tools/call", "session_list", serde_json::json!({})));
        let sessions = list["sessions"].as_array().expect("sessions");
        assert_eq!(sessions.len(), 2, "{list}");
        assert_eq!(sessions[0]["default"], true);
        assert_eq!(sessions[1]["engine_id"], second.as_str());
        assert_ne!(sessions[0]["engine_id"], sessions[1]["engine_id"]);
    }

    #[test]
    fn engine_id_routes_each_tool_to_its_session_and_unknown_ids_are_refused() {
        let _guard = crate::mcp::tests::SESSION_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut hub = SessionHub::with_limit(2);
        let default_id = hub.default_id().to_string();
        let second = payload(&rpc(&mut hub, "tools/call", "session_open", serde_json::json!({})))["engine_id"]
            .as_str()
            .unwrap()
            .to_string();

        let to_second = rpc(&mut hub, "tools/call", "session_info", serde_json::json!({"engine_id": second}));
        assert!(!is_error(&to_second), "{to_second}");
        assert_eq!(to_second["result"]["sessionId"], second.as_str());

        let to_default = rpc(&mut hub, "tools/call", "session_info", serde_json::json!({}));
        assert_eq!(to_default["result"]["sessionId"], default_id.as_str());

        let ghost = rpc(&mut hub, "tools/call", "session_info", serde_json::json!({"engine_id": "nope"}));
        assert!(is_error(&ghost), "{ghost}");
        assert_eq!(payload(&ghost)["code"], "SESSION_NOT_FOUND", "{ghost}");
    }

    #[test]
    fn an_app_installed_in_one_session_is_invisible_to_the_other() {
        let _guard = crate::mcp::tests::SESSION_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut hub = SessionHub::with_limit(2);
        let default_id = hub.default_id().to_string();
        let second = payload(&rpc(&mut hub, "tools/call", "session_open", serde_json::json!({})))["engine_id"]
            .as_str()
            .unwrap()
            .to_string();
        let apk = apk_on_disk();

        let installed = rpc(
            &mut hub,
            "tools/call",
            "install",
            serde_json::json!({"apk": apk, "engine_id": second}),
        );
        assert!(!is_error(&installed), "install na sessão 2: {installed}");

        let in_second = payload(&rpc(&mut hub, "tools/call", "session_info", serde_json::json!({"engine_id": second})));
        assert_eq!(in_second["installed_count"], 1, "{in_second}");
        let in_default = payload(&rpc(&mut hub, "tools/call", "session_info", serde_json::json!({"engine_id": default_id})));
        assert_eq!(in_default["installed_count"], 0, "a sessão padrão não vê o app da outra: {in_default}");
        assert!(in_default["active_package"].is_null(), "{in_default}");
    }

    #[test]
    fn taps_are_routed_to_the_session_that_runs_the_app() {
        let _guard = crate::mcp::tests::SESSION_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        // Mesmo lock dos testes de `mcp::tests`: o estado de processo (sandbox, env) é compartilhado.
        let mut hub = SessionHub::with_limit(2);
        let default_id = hub.default_id().to_string();
        let second = payload(&rpc(&mut hub, "tools/call", "session_open", serde_json::json!({})))["engine_id"]
            .as_str()
            .unwrap()
            .to_string();
        let apk = crate::mcp::tests::fixture_apk_path();
        // Mesmo APK nas duas sessões; só a sessão 2 lança o app.
        for id in [&default_id, &second] {
            let r = rpc(&mut hub, "tools/call", "install", serde_json::json!({"apk": apk, "engine_id": id}));
            assert!(!is_error(&r), "install em {id}: {r}");
        }
        let launched = rpc(&mut hub, "tools/call", "launch", serde_json::json!({"engine_id": second}));
        assert!(!is_error(&launched), "launch na sessão 2: {launched}");

        // A sessão 2 acha a view e recebe o toque.
        let found = payload(&rpc(
            &mut hub,
            "tools/call",
            "find_views",
            serde_json::json!({"resource_id": "com.test:id/tv", "engine_id": second}),
        ));
        assert_eq!(found["total"], 1, "{found}");
        let token = found["tree_token"].as_str().unwrap();
        let tap = rpc(
            &mut hub,
            "tools/call",
            "tap_view",
            serde_json::json!({"resource_id": "com.test:id/tv", "expected_token": token, "engine_id": second}),
        );
        assert!(!is_error(&tap), "toque roteado à sessão 2: {tap}");
        assert_eq!(tap["result"]["sessionId"], second.as_str(), "{tap}");

        // A sessão padrão não lançou nada: o mesmo toque ali é recusado, nunca vai para a sessão 2.
        let tap_default = rpc(
            &mut hub,
            "tools/call",
            "tap_view",
            serde_json::json!({"resource_id": "com.test:id/tv", "expected_token": token}),
        );
        assert!(is_error(&tap_default), "{tap_default}");
        assert_eq!(payload(&tap_default)["code"], "NOT_RUNNING", "{tap_default}");
        let info_default = payload(&rpc(&mut hub, "tools/call", "session_info", serde_json::json!({"engine_id": default_id})));
        assert_eq!(info_default["running"], false, "{info_default}");
        assert!(info_default["running_package"].is_null(), "{info_default}");
        std::fs::remove_file(apk).ok();
    }

    #[test]
    fn close_refuses_the_default_session_and_frees_a_slot() {
        let _guard = crate::mcp::tests::SESSION_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut hub = SessionHub::with_limit(2);
        let default_id = hub.default_id().to_string();
        let second = payload(&rpc(&mut hub, "tools/call", "session_open", serde_json::json!({})))["engine_id"]
            .as_str()
            .unwrap()
            .to_string();

        let refused = rpc(&mut hub, "tools/call", "session_close", serde_json::json!({"engine_id": default_id}));
        assert!(is_error(&refused), "{refused}");
        assert_eq!(payload(&refused)["code"], "INVALID_ARG", "{refused}");

        let closed = rpc(&mut hub, "tools/call", "session_close", serde_json::json!({"engine_id": second}));
        assert!(!is_error(&closed), "{closed}");
        let gone = rpc(&mut hub, "tools/call", "session_info", serde_json::json!({"engine_id": second}));
        assert_eq!(payload(&gone)["code"], "SESSION_NOT_FOUND", "{gone}");
        // o slot liberado volta a servir
        let reopened = rpc(&mut hub, "tools/call", "session_open", serde_json::json!({}));
        assert!(!is_error(&reopened), "{reopened}");
    }

    #[test]
    fn tools_list_offers_the_session_tools_and_a_bare_session_refuses_them() {
        let _guard = crate::mcp::tests::SESSION_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut hub = SessionHub::with_limit(2);
        let line = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}).to_string();
        let resp = handle_message_hub(&mut hub, &line).unwrap().unwrap();
        let names: Vec<&str> = resp["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();
        for t in SESSION_TOOLS {
            assert!(names.contains(&t), "tools/list sem {t}");
        }

        // sem o servidor com sessões, a tool não tem como atender: erro explícito
        let mut bare = AgentSession::new();
        let r = bare.call_tool("session_open", &serde_json::json!({}));
        assert!(r.is_error, "sessão avulsa não abre sessões");
    }

    #[test]
    fn heap_budget_splits_the_process_cap_and_never_exceeds_it() {
        const MB: usize = 1024 * 1024;
        // moto-e5: teto 1024 MB, heap do app 256 MB.
        assert_eq!(session_heap_budget_bytes(1024, 256, 1), 256 * MB);
        assert_eq!(session_heap_budget_bytes(1024, 256, 2), 256 * MB, "min(teto/N, heap do app)");
        assert_eq!(session_heap_budget_bytes(1024, 256, 5), 204 * MB, "teto/N arredonda para baixo");
        assert_eq!(session_heap_budget_bytes(256, 256, 2), 128 * MB, "teto/N quando o teto é o limite");
        for n in 1..=8usize {
            let budget = session_heap_budget_bytes(1024, 256, n);
            assert!(budget * n <= 1024 * MB, "N={n}: soma dos orçamentos passa do teto");
        }
    }

    #[test]
    fn sessions_report_the_heap_budget_they_were_given() {
        let _guard = crate::mcp::tests::SESSION_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut hub = SessionHub::with_limit(4);
        let default_id = hub.default_id().to_string();
        let second = payload(&rpc(&mut hub, "tools/call", "session_open", serde_json::json!({})))["engine_id"]
            .as_str()
            .unwrap()
            .to_string();
        for id in [&default_id, &second] {
            let info = payload(&rpc(&mut hub, "tools/call", "session_info", serde_json::json!({"engine_id": id})));
            // 1024 MB / 4 sessões = 256 MB, igual ao heap do app
            assert_eq!(info["memory"]["heap_budget_mb"], 256, "{info}");
        }
        // com 8 sessões o teto por sessão cai abaixo do heap do app
        let mut wide = SessionHub::with_limit(8);
        let info = payload(&rpc(&mut wide, "tools/call", "session_info", serde_json::json!({})));
        assert_eq!(info["memory"]["heap_budget_mb"], 128, "{info}");
    }

    #[test]
    fn a_session_over_its_heap_budget_fails_closed_without_touching_the_other() {
        let _guard = crate::mcp::tests::SESSION_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut hub = SessionHub::with_limit(2);
        let default_id = hub.default_id().to_string();
        let second = payload(&rpc(&mut hub, "tools/call", "session_open", serde_json::json!({})))["engine_id"]
            .as_str()
            .unwrap()
            .to_string();
        let apk = crate::mcp::tests::fixture_apk_path();
        for id in [&default_id, &second] {
            let r = rpc(&mut hub, "tools/call", "install", serde_json::json!({"apk": apk, "engine_id": id}));
            assert!(!is_error(&r), "install em {id}: {r}");
        }
        // Orçamento zero na sessão 2: a primeira alocação do app passa do limite,
        // então o launch falha fechado. (Um orçamento de 1 KB não basta para
        // falhar: o app de fixture cabe nele.)
        hub.sessions[1].set_heap_budget(0);
        let launched = rpc(&mut hub, "tools/call", "launch", serde_json::json!({"engine_id": second}));
        assert!(is_error(&launched), "launch com orçamento zero devia falhar: {launched}");
        // A sessão padrão, com o orçamento normal, continua de pé.
        let ok = rpc(&mut hub, "tools/call", "launch", serde_json::json!({"engine_id": default_id}));
        assert!(!is_error(&ok), "sessão padrão com orçamento normal: {ok}");
        std::fs::remove_file(apk).ok();
    }

    #[test]
    fn two_apps_run_at_once_and_interleaved_touches_reach_only_their_own_session() {
        let _guard = crate::mcp::tests::SESSION_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut hub = SessionHub::with_limit(2);
        let a = hub.default_id().to_string();
        let b = payload(&rpc(&mut hub, "tools/call", "session_open", serde_json::json!({})))["engine_id"]
            .as_str()
            .unwrap()
            .to_string();
        let apk = crate::mcp::tests::fixture_apk_path();
        for id in [&a, &b] {
            assert!(!is_error(&rpc(&mut hub, "tools/call", "install", serde_json::json!({"apk": apk, "engine_id": id}))));
            assert!(!is_error(&rpc(&mut hub, "tools/call", "launch", serde_json::json!({"engine_id": id}))));
        }
        let find = |hub: &mut SessionHub, id: &str| {
            let v = payload(&rpc(
                hub,
                "tools/call",
                "find_views",
                serde_json::json!({"resource_id": "com.test:id/tv", "engine_id": id}),
            ));
            assert_eq!(v["total"], 1, "{v}");
            v["tree_token"].as_str().unwrap().to_string()
        };
        let tap = |hub: &mut SessionHub, id: &str, token: &str| {
            rpc(
                hub,
                "tools/call",
                "tap_view",
                serde_json::json!({"resource_id": "com.test:id/tv", "expected_token": token, "engine_id": id}),
            )
        };
        let token_a = find(&mut hub, &a);

        // O token de uma sessão não age sobre a outra.
        let cross = tap(&mut hub, &b, &token_a);
        assert!(is_error(&cross), "token da sessão A aceito na B: {cross}");
        // A recusa pode observar a árvore de B de novo; a linha de base é tirada depois dela.
        let token_b = find(&mut hub, &b);

        // Toques intercalados: cada um cai na sessão pedida.
        let t1 = tap(&mut hub, &a, &token_a);
        assert!(!is_error(&t1), "{t1}");
        assert_eq!(t1["result"]["sessionId"], a.as_str());
        // a sessão B não mudou com o toque em A: o token dela continua válido
        let token_b2 = find(&mut hub, &b);
        assert_eq!(token_b2, token_b, "toque na sessão A mudou a árvore da sessão B");
        let t2 = tap(&mut hub, &b, &token_b2);
        assert!(!is_error(&t2), "{t2}");
        assert_eq!(t2["result"]["sessionId"], b.as_str());
        // o token antigo da sessão A agora está velho (o toque anterior mudou a revisão)
        let stale = tap(&mut hub, &a, &token_a);
        assert!(is_error(&stale), "token antigo da A ainda aceito: {stale}");
        let token_a2 = find(&mut hub, &a);
        let t3 = tap(&mut hub, &a, &token_a2);
        assert!(!is_error(&t3), "{t3}");
        std::fs::remove_file(apk).ok();
    }

    #[test]
    fn input_router_records_each_touch_in_order_with_its_session() {
        let _guard = crate::mcp::tests::SESSION_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut hub = SessionHub::with_limit(2);
        let a = hub.default_id().to_string();
        let b = payload(&rpc(&mut hub, "tools/call", "session_open", serde_json::json!({})))["engine_id"]
            .as_str()
            .unwrap()
            .to_string();
        let apk = crate::mcp::tests::fixture_apk_path();
        for id in [&a, &b] {
            assert!(!is_error(&rpc(&mut hub, "tools/call", "install", serde_json::json!({"apk": apk, "engine_id": id}))));
            assert!(!is_error(&rpc(&mut hub, "tools/call", "launch", serde_json::json!({"engine_id": id}))));
        }
        let token = |hub: &mut SessionHub, id: &str| {
            payload(&rpc(
                hub,
                "tools/call",
                "find_views",
                serde_json::json!({"resource_id": "com.test:id/tv", "engine_id": id}),
            ))["tree_token"]
                .as_str()
                .unwrap()
                .to_string()
        };
        let tap = |hub: &mut SessionHub, id: &str, t: &str| {
            rpc(
                hub,
                "tools/call",
                "tap_view",
                serde_json::json!({"resource_id": "com.test:id/tv", "expected_token": t, "engine_id": id}),
            )
        };
        let ta = token(&mut hub, &a);
        let tb = token(&mut hub, &b);
        assert!(!is_error(&tap(&mut hub, &a, &ta)));
        assert!(!is_error(&tap(&mut hub, &b, &tb)));
        // token antigo: recusado pelo app, e o roteador registra a tentativa como não aceita
        assert!(is_error(&tap(&mut hub, &a, &ta)));

        let events: Vec<InputEvent> = hub.input.events().cloned().collect();
        let got: Vec<(&str, bool)> = events
            .iter()
            .map(|e| (e.session_id.as_str(), e.accepted))
            .collect();
        assert_eq!(got, vec![(a.as_str(), true), (b.as_str(), true), (a.as_str(), false)], "{events:?}");
        assert!(events.windows(2).all(|w| w[0].seq < w[1].seq), "sequência fora de ordem: {events:?}");
        // session_list expõe o último evento de cada sessão
        let listed = payload(&rpc(&mut hub, "tools/call", "session_list", serde_json::json!({})));
        let seq_of = |id: &str| {
            listed["sessions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["engine_id"] == id)
                .unwrap()["last_input_seq"]
                .as_u64()
                .unwrap()
        };
        assert_eq!(seq_of(&a), events[2].seq);
        assert_eq!(seq_of(&b), events[1].seq);
        std::fs::remove_file(apk).ok();
    }
}
