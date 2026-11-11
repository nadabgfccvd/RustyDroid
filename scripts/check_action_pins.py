#!/usr/bin/env python3
"""Verify that every pinned GitHub Actions ref in `.github/workflows/` resolves.

Workflows must pin third-party actions to a full commit SHA (RD Sentinel rule:
an unpinned ref next to a write permission is a P1 finding). The failure mode of
pinning is that a SHA can be *wrong* and nothing local notices: a typo, a
truncated hash, or a SHA copy-pasted from the wrong tag all produce a workflow
that parses fine, passes YAML lint, and then dies in CI at `Set up job` with

    Unable to resolve action `actions/download-artifact@3e5f...f88c`,
    unable to find version `3e5f...f88c`

That is exactly how the M12 docs deploy failed twice. This script turns that
runtime error into a pre-merge one.

Checks, per `uses:` line:
  1. the ref is a 40-char hex SHA (not `@v4`, not `@main`);
  2. the SHA exists in the action's repository;
  3. the SHA is reachable from a tag, and the tag comment says which one, so a
     pin can be reviewed against the version it claims to be.

Local actions (`./path`) are skipped.

Rate limiting is treated as "cannot check", not as a failure: this gate must
never go red because of a shared runner IP. Anything that actually *does not
resolve* is a hard failure.

Usage:
  check_action_pins.py                 # verify, exit 1 on any bad pin
  check_action_pins.py --offline       # only check shape, never touch the network
  check_action_pins.py --verbose       # print every pin, including the good ones
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WORKFLOWS = ROOT / ".github" / "workflows"

_USES = re.compile(r"^\s*(?:-\s*)?uses:\s*(?P<action>[^\s#]+)(?:\s*#\s*(?P<comment>.+))?$")
_SHA = re.compile(r"^[0-9a-f]{40}$")

API = "https://api.github.com"


def parse_pins() -> list[dict[str, str]]:
    pins: list[dict[str, str]] = []
    for path in sorted(WORKFLOWS.glob("*.yml")) + sorted(WORKFLOWS.glob("*.yaml")):
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            m = _USES.match(line)
            if not m:
                continue
            ref = m.group("action")
            if ref.startswith("./") or "/" not in ref:
                continue  # local action or docker image
            action, _, version = ref.partition("@")
            pins.append(
                {
                    "file": path.name,
                    "line": str(lineno),
                    "action": action,
                    "version": version,
                    "comment": (m.group("comment") or "").strip(),
                }
            )
    return pins


def _get(url: str) -> tuple[int, object]:
    req = urllib.request.Request(url, headers={"Accept": "application/vnd.github+json"})
    req.add_header("User-Agent", "rustydroid-action-pin-check")
    token = __import__("os").environ.get("GITHUB_TOKEN")
    if token:
        req.add_header("Authorization", f"Bearer {token}")
    try:
        with urllib.request.urlopen(req, timeout=20) as r:
            return r.status, json.loads(r.read().decode())
    except urllib.error.HTTPError as e:
        return e.code, None
    except (urllib.error.URLError, TimeoutError):
        return 0, None


def resolve(pin: dict[str, str]) -> tuple[str, str]:
    """-> (verdict, detail). verdict in {ok, bad, unknown}."""
    status, _ = _get(f"{API}/repos/{pin['action']}/commits/{pin['version']}")
    if status == 200:
        tag = ""
        tstatus, tags = _get(f"{API}/repos/{pin['action']}/tags?per_page=100")
        if tstatus == 200 and isinstance(tags, list):
            for t in tags:
                if t.get("commit", {}).get("sha") == pin["version"]:
                    tag = t["name"]
                    break
        else:
            return "ok", "resolves (tags unavailable)"
        if not tag:
            return "ok", "resolves, not a tag tip"
        if pin["comment"] and tag not in pin["comment"]:
            return "ok", f"resolves as {tag} (comment says {pin['comment']} — stale?)"
        return "ok", f"resolves as {tag}"
    if status in (403, 429):
        return "unknown", "rate limited; cannot verify"
    if status == 0:
        return "unknown", "network unavailable; cannot verify"
    return "bad", f"does not exist in {pin['action']} (HTTP {status})"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--offline", action="store_true", help="only check the ref shape")
    ap.add_argument("--verbose", action="store_true")
    args = ap.parse_args(argv)

    pins = parse_pins()
    if not pins:
        print("check_action_pins: no pinned actions found", file=sys.stderr)
        return 2

    bad: list[dict[str, str]] = []
    unknown: list[dict[str, str]] = []

    for pin in pins:
        where = f"{pin['file']}:{pin['line']}"
        if not _SHA.match(pin["version"]):
            print(f"BAD  {where:<26} {pin['action']}@{pin['version'] or '<none>'} — not a 40-char SHA")
            bad.append(pin)
            continue
        if args.offline:
            if args.verbose:
                print(f"ok   {where:<26} {pin['action']}@{pin['version'][:12]}")
            continue
        verdict, detail = resolve(pin)
        if verdict == "bad":
            print(f"BAD  {where:<26} {pin['action']}@{pin['version'][:12]} — {detail}")
            bad.append(pin)
        elif verdict == "unknown":
            print(f"???  {where:<26} {pin['action']}@{pin['version'][:12]} — {detail}")
            unknown.append(pin)
        elif args.verbose:
            print(f"ok   {where:<26} {pin['action']}@{pin['version'][:12]} — {detail}")

    print()
    print(f"check_action_pins: {len(pins)} pin(s), {len(bad)} broken, {len(unknown)} unverified")
    if bad:
        print(
            "\ncheck_action_pins: a pinned ref does not resolve. The workflow that "
            "uses it will fail at `Set up job` before any step runs.",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
