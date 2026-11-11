"""Sonda do Scanner do Comprix via servidor MCP (`rd mcp`).

Uso: python3 probe_scanner.py [--apk CAMINHO] [--out DIR]
Dirige: launch -> Pular -> Nova lista -> Compra do Mês -> Escanear um produto,
concede CAMERA e registra a árvore de views e o estado da câmera.
"""
import argparse
import json
import os
import pathlib
import selectors
import subprocess
import sys
import time

ROOT = pathlib.Path("/home/user/RustyDroid")
ap = argparse.ArgumentParser()
ap.add_argument("--apk", default="/home/user/comprix/comprix-v1.6.0-x86_64-release.apk")
ap.add_argument("--out", default="/home/user/comprix/probe")
ap.add_argument("--rd", default=str(ROOT / "target/debug/rd"))
ap.add_argument("--steps", default="scanner")
a = ap.parse_args()
out = pathlib.Path(a.out)
out.mkdir(parents=True, exist_ok=True)

env = dict(os.environ, RD_MCP_SANDBOX=os.pathsep.join([str(pathlib.Path(a.apk).parent), str(out)]))
p = subprocess.Popen([a.rd, "mcp"], cwd=str(ROOT), env=env, stdin=subprocess.PIPE,
                     stdout=subprocess.PIPE, stderr=(out / "stderr.log").open("wb"), bufsize=0)
sel = selectors.DefaultSelector()
sel.register(p.stdout, selectors.EVENT_READ)
pending = b""
seq = 0
log = (out / "transcript.jsonl").open("w")


def rpc(name, args=None):
    global seq, pending
    seq += 1
    req = {"jsonrpc": "2.0", "id": seq, "method": "tools/call",
           "params": {"name": name, "arguments": args or {}}}
    t = time.monotonic()
    p.stdin.write((json.dumps(req) + "\n").encode())
    limit = time.monotonic() + 300
    while True:
        while b"\n" not in pending:
            remaining = limit - time.monotonic()
            if remaining <= 0 or not sel.select(max(0, remaining)):
                raise RuntimeError(f"timeout em {name}")
            chunk = os.read(p.stdout.fileno(), 65536)
            if not chunk:
                raise RuntimeError(f"MCP saiu {p.poll()} em {name}")
            pending += chunk
        line, pending = pending.split(b"\n", 1)
        resp = json.loads(line)
        if resp.get("id") == seq:
            break
    ms = round((time.monotonic() - t) * 1000, 1)
    result = resp.get("result", {})
    texts = [c.get("text", "") for c in result.get("content", []) if c.get("type") == "text"]
    log.write(json.dumps({"seq": seq, "name": name, "args": args or {}, "ms": ms,
                          "resp": resp}, ensure_ascii=False) + "\n")
    log.flush()
    err = result.get("isError") or resp.get("error")
    print(f"step={seq} {name} {ms}ms{' ERRO' if err else ''}", flush=True)
    if err:
        print("   ", "\n    ".join("\n".join(texts)[:800].splitlines()), flush=True)
    return texts


def salvar(nome, texts):
    (out / f"{nome}.txt").write_text("\n".join(texts), encoding="utf-8")


try:
    rpc("install", {"apk": a.apk})
    t = rpc("launch", {"observe": True})
    salvar("01_launch", t)
    rpc("tap_text", {"text": "Pular", "observe": True})
    rpc("tap_text", {"text": "Nova lista", "observe": True})
    rpc("wait", {"ms": 500, "observe": True})
    rpc("tap_text", {"text": "Compra do Mês", "observe": True})
    rpc("tap_text", {"text": "Criar e adicionar itens", "observe": True})
    rpc("wait", {"ms": 800, "observe": True})
    t = rpc("observe", {"format": "text"})
    salvar("02_lista", t)
    rpc("permissions_grant", {"permission": "android.permission.CAMERA"})
    t = rpc("tap_text", {"text": "Escanear um produto", "observe": True})
    salvar("03_scanner", t)
    rpc("wait", {"ms": 3000, "observe": True})
    t = rpc("observe", {"format": "text"})
    salvar("04_scanner_depois_3s", t)
finally:
    log.close()
    p.stdin.close()
    p.terminate()
    print("saida em", out, flush=True)
