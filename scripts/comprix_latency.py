#!/usr/bin/env python3
"""Medição de latência MCP do Comprix v1.6.0 via `rd mcp` (stdio, JSON-RPC 2.0).

Metas do projeto:
  - lançamento (`launch`, observe desligado): 450 ms
  - cada ferramenta MCP (observe ligado): 345 ms

Uso:
  python3 scripts/comprix_latency.py <rd> <apk> [--runs N] [--out docs/...json]

Cada chamada é cronometrada do write da requisição até a resposta parseada
(time.perf_counter). Uma sessão nova de `rd mcp` por execução. O fluxo é o de
onboarding do Comprix: launch, observe, tap "Pular", wait 1000, tap "Nova lista",
swipe, key BACK, screenshot, ui_tree. Saída: mediana por ferramenta.

O script não grava o APK nem o token no repositório; o caminho do APK é um argumento.
"""
import argparse
import json
import os
import statistics
import subprocess
import sys
import tempfile
import time

PKG = "br.com.comprix"


class Rd:
    def __init__(self, rd_bin, sandbox):
        env = dict(os.environ, RD_MCP_SANDBOX=sandbox)
        self.p = subprocess.Popen(
            [rd_bin, "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, text=True, env=env, bufsize=1,
        )
        self.n = 0
        self.rpc("initialize", {"protocolVersion": "2024-11-05", "capabilities": {},
                                "clientInfo": {"name": "comprix_latency", "version": "1"}})
        self.p.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
        self.p.stdin.flush()

    def rpc(self, method, params):
        self.n += 1
        req = {"jsonrpc": "2.0", "id": self.n, "method": method, "params": params}
        t0 = time.perf_counter()
        self.p.stdin.write(json.dumps(req) + "\n")
        self.p.stdin.flush()
        line = self.p.stdout.readline()
        ms = (time.perf_counter() - t0) * 1000.0
        if not line:
            raise RuntimeError("rd mcp fechou stdout")
        return json.loads(line), ms

    def call(self, name, args, label=None):
        resp, ms = self.rpc("tools/call", {"name": name, "arguments": args})
        res = resp.get("result", {})
        err = bool(res.get("isError")) or "error" in resp
        if err:
            cause = (res.get("content") or [{}])[0].get("text", "")[:200] if res else ""
            print(f"  erro em {label or name}: {cause}", file=sys.stderr)
        return {"tool": label or name, "ms": round(ms, 2), "error": err}

    def close(self):
        self.p.stdin.close()
        self.p.wait(timeout=30)


def run_once(rd_bin, apk, sandbox, shot_path):
    rd = Rd(rd_bin, sandbox)
    rows = []
    rows.append(rd.call("install", {"apk": apk}))
    rows.append(rd.call("launch", {"package": PKG, "observe": True}))
    rows.append(rd.call("observe", {}))
    rows.append(rd.call("tap_text", {"text": "Pular", "observe": True}, label="tap_text:Pular"))
    rows.append(rd.call("wait", {"ms": 1000, "observe": True}))
    rows.append(rd.call("tap_text", {"text": "Nova lista", "observe": True}, label="tap_text:Nova lista"))
    rows.append(rd.call("swipe", {"x1": 360, "y1": 1200, "x2": 360, "y2": 400, "ms": 800, "observe": True}))
    rows.append(rd.call("key", {"key": "BACK", "observe": True}))
    rows.append(rd.call("screenshot", {"path": shot_path}))
    rows.append(rd.call("ui_tree", {}))
    rd.close()
    # medição de lançamento sem observe, em sessão própria
    rd2 = Rd(rd_bin, sandbox)
    rd2.call("install", {"apk": apk})
    off = rd2.call("launch", {"package": PKG, "observe": False})
    rd2.close()
    return rows, off


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("rd")
    ap.add_argument("apk")
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--out", default="")
    a = ap.parse_args()
    apk = os.path.abspath(a.apk)
    sandbox = os.path.dirname(apk)
    tmp = tempfile.mkdtemp(dir=sandbox)
    shot = os.path.join(tmp, "shot.png")

    samples = {}
    off_samples = []
    for i in range(a.runs):
        rows, off = run_once(a.rd, apk, sandbox, shot)
        for r in rows:
            samples.setdefault(r["tool"], []).append(r)
        off_samples.append(off)
        print(f"run {i+1}: launch(observe off)={off['ms']:.1f} ms  "
              + "  ".join(f"{r['tool']}={r['ms']:.1f}" for r in rows), flush=True)

    summary = {"launch_observe_off": {"median_ms": round(statistics.median(s["ms"] for s in off_samples), 1),
                                      "samples": [s["ms"] for s in off_samples],
                                      "target_ms": 450, "errors": sum(s["error"] for s in off_samples)}}
    for tool, rs in samples.items():
        ms = [r["ms"] for r in rs]
        summary[tool + "_observe_on"] = {"median_ms": round(statistics.median(ms), 1),
                                         "samples": ms, "target_ms": 345,
                                         "errors": sum(r["error"] for r in rs)}
    print("\n| chamada | mediana ms | amostras | meta ms | ok |")
    print("|---|---:|---|---:|---|")
    for k, v in summary.items():
        print(f"| {k} | {v['median_ms']} | {', '.join(str(x) for x in v['samples'])} | {v['target_ms']} | {'sim' if v['median_ms'] <= v['target_ms'] and not v['errors'] else 'não'} |")
    if a.out:
        with open(a.out, "w", encoding="utf-8") as f:
            json.dump(summary, f, indent=2, ensure_ascii=False)
    try:
        os.remove(shot)
        os.rmdir(tmp)
    except OSError:
        pass


if __name__ == "__main__":
    sys.exit(main())
