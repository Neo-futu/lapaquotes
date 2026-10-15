#!/usr/bin/env python3
"""Daily: platform OTC quotes → BI tables → Feishu Bitable.

Preferred path: call the running agent BI HTTP API.
Fallback: invoke `cargo run -p agent --example bi_sync -- daily`.

Env:
  PG__URL                          (fallback path / health)
  FEISHU_BITABLE_ACCESS_TOKEN      pt-…
  FEISHU_BITABLE_APP_TOKEN
  FEISHU_BITABLE_API_BASE          optional
  AGENT_BI_URL                     default http://127.0.0.1:4141/agent/bi
  BI_SYNC_DATE                     optional YYYY-MM-DD (else latest in DB)
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def load_dotenv() -> None:
    env_path = ROOT / ".env"
    if not env_path.is_file():
        return
    for line in env_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        os.environ.setdefault(k.strip(), v.strip())


def http_json(method: str, url: str, body: dict | None = None) -> dict:
    data = None if body is None else json.dumps(body).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=data,
        method=method,
        headers={"Content-Type": "application/json", "Accept": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        return json.loads(resp.read().decode("utf-8"))


def run_via_http(base: str, date: str | None) -> int:
    print(f"[bi] catalog refresh → {base}/catalog/refresh")
    catalog = http_json("POST", f"{base.rstrip('/')}/catalog/refresh")
    tables = catalog.get("tables") or []
    print(f"[bi] catalog tables={len(tables)}")
    for t in tables:
        print(f"  - {t.get('role')}: {t.get('name')} ({t.get('table_id')})")

    need_migrate = any(
        not any(f.get("name") == "同步键" for f in (t.get("fields") or []))
        or (
            t.get("role") == "quote_ledger"
            and not any(f.get("name") == "Bid" for f in (t.get("fields") or []))
        )
        for t in tables
    )
    if need_migrate:
        print("[bi] migrate bitable schemas…")
        report = http_json("POST", f"{base.rstrip('/')}/migrate")
        print(json.dumps(report, ensure_ascii=False, indent=2))

    print("[bi] ensure dashboard…")
    try:
        dash = http_json("POST", f"{base.rstrip('/')}/dashboards/ensure")
        print(json.dumps(dash, ensure_ascii=False, indent=2))
    except Exception as e:
        print(f"[bi] dashboard ensure skipped: {e}")

    payload = {} if not date else {"date": date}
    print(f"[bi] sync {payload or '(latest)'}…")
    sync = http_json("POST", f"{base.rstrip('/')}/sync", payload)
    print(json.dumps(sync, ensure_ascii=False, indent=2))
    return 0


def run_via_cargo(date: str | None) -> int:
    cmd = [
        "cargo",
        "run",
        "-p",
        "agent",
        "--example",
        "bi_sync",
        "--features",
        "standalone",
        "--",
        "daily" if not date else "sync",
    ]
    if date:
        cmd.append(date)
    print("[bi] fallback:", " ".join(cmd))
    return subprocess.call(cmd, cwd=str(ROOT))


def main() -> int:
    load_dotenv()
    base = os.environ.get("AGENT_BI_URL", "http://127.0.0.1:4141/agent/bi")
    date = os.environ.get("BI_SYNC_DATE") or (sys.argv[1] if len(sys.argv) > 1 else None)
    try:
        return run_via_http(base, date)
    except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError) as e:
        print(f"[bi] HTTP unavailable ({e}); falling back to cargo example")
        return run_via_cargo(date)


if __name__ == "__main__":
    raise SystemExit(main())
