#!/usr/bin/env python3
"""Golden ponta a ponta do Comprix v1.6.0 via `rd mcp` (stdio, JSON-RPC 2.0).

Fluxo: onboarding (Pular) -> Nova lista (nome) -> adicionar item digitado
"2 kg de arroz" + ENTER -> a lista mostra "Arroz", "2 x kg" e "1 item".
Cada passo confere o texto visível (observe). Falha = exit 1 com o passo e
o texto visto. Nada é simulado: passa pelo parser do app, Compose e Room.

Uso: python3 scripts/comprix_golden.py <rd> <apk>
"""
import json
import os
import subprocess
import sys
import time

PKG = "br.com.comprix"


class Rd:
    def __init__(self, rd_bin, sandbox):
        env = dict(os.environ, RD_MCP_SANDBOX=sandbox)
        self.p = subprocess.Popen([rd_bin, "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=subprocess.DEVNULL, text=True, env=env, bufsize=1)
        self.n = 0
        self.rpc("initialize", {"protocolVersion": "2024-11-05", "capabilities": {},
                                "clientInfo": {"name": "comprix_golden", "version": "1"}})

    def rpc(self, method, params):
        self.n += 1
        req = {"jsonrpc": "2.0", "id": self.n, "method": method, "params": params}
        t0 = time.perf_counter()
        self.p.stdin.write(json.dumps(req) + "\n")
        self.p.stdin.flush()
        line = self.p.stdout.readline()
        if not line:
            raise RuntimeError("rd mcp fechou stdout")
        return json.loads(line), (time.perf_counter() - t0) * 1000.0

    def call(self, name, args):
        resp, ms = self.rpc("tools/call", {"name": name, "arguments": args})
        res = resp.get("result", {})
        text = "".join(c.get("text", "") for c in res.get("content", []) if c.get("type") == "text")
        return res.get("isError", False) or "error" in resp, text, ms

    def close(self):
        self.p.stdin.close()
        self.p.wait(timeout=30)


def visible(rd):
    _, text, _ = rd.call("observe", {})
    return text


def expect(rd, step, must, text_now=None):
    text = text_now if text_now is not None else visible(rd)
    missing = [m for m in must if m not in text]
    status = "OK " if not missing else "FALHA"
    print(f"[{status}] {step}", flush=True)
    if missing:
        print(f"       faltando: {missing}", flush=True)
        print("       texto visível:", flush=True)
        for line in text.splitlines()[:40]:
            print("         " + line, flush=True)
        return False
    return True


def step_call(rd, step, name, args, timed=None):
    err, text, ms = rd.call(name, args)
    if err:
        print(f"[FALHA] {step}: erro em {name}: {text[:200]}", flush=True)
        return None
    print(f"       {step}: {ms:.1f} ms", flush=True)
    return text


def main():
    rd_bin, apk = sys.argv[1], os.path.abspath(sys.argv[2])
    sandbox = os.path.dirname(apk)
    rd = Rd(rd_bin, sandbox)
    ok = True
    err, _, _ = rd.call("install", {"apk": apk})
    ok &= not err
    err, _, _ = rd.call("launch", {"package": PKG, "observe": True})
    ok &= not err
    ok &= expect(rd, "launch mostra o onboarding", ["Pular", "100% Offline"]) if not err else False

    if step_call(rd, "tap Pular", "tap_text", {"text": "Pular", "observe": True}) is None:
        ok = False
    ok &= expect(rd, "home: Nenhuma lista ainda", ["Nenhuma lista ainda"])

    if step_call(rd, "tap Nova lista", "tap_text", {"text": "Nova lista", "observe": True}) is None:
        ok = False
    step_call(rd, "espera da sheet", "wait", {"ms": 1500, "observe": True})
    ok &= expect(rd, "sheet Criar nova lista", ["Criar nova lista", "Nome da lista"])

    # Mesmo caminho do regressão real (scripts/comprix-regression.py): escolhe o
    # chip de sugestão, cria a lista, cai na tela "Adição rápida" e digita o item.
    if step_call(rd, "tap chip Compra do Mês", "tap_text", {"text": "Compra do Mês", "observe": True}) is None:
        ok = False
    if step_call(rd, "tap Criar e adicionar itens", "tap_text",
                 {"text": "Criar e adicionar itens", "observe": True}) is None:
        ok = False
    step_call(rd, "espera da tela", "wait", {"ms": 400})
    ok &= expect(rd, "tela Adição rápida", ["Adição rápida"])

    if step_call(rd, "tap no campo de item", "tap", {"x": 260, "y": 309}) is None:
        ok = False
    text = step_call(rd, "digita item", "type_text",
                     {"text": "2 kg de arroz", "replace": True, "observe": True})
    if text is None:
        ok = False
    if step_call(rd, "ENTER", "key", {"key": "ENTER", "observe": True}) is None:
        ok = False
    step_call(rd, "espera do item", "wait", {"ms": 500, "observe": True})
    ok &= expect(rd, "item: Arroz / 2 x kg / 1 item", ["Arroz", "2 x kg", "1 item"])

    rd.close()
    print("GOLDEN " + ("PASSOU" if ok else "FALHOU"), flush=True)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
