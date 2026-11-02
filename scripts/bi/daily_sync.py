#!/usr/bin/env python3
"""Daily: (optional Outlook ingest) → platform OTC quotes → BI → Feishu Bitable.

Preferred path: call the running agent BI HTTP API.
Fallback: invoke `cargo run -p agent --example bi_sync -- daily`.

Usage:
  python scripts/bi/daily_sync.py                 # latest quote date in DB
  python scripts/bi/daily_sync.py 2026-10-27      # that day (auto-ingest if DB empty)
  python scripts/bi/daily_sync.py 2026-10-27 --ingest   # always re-run Outlook ingest first
  BI_SYNC_DATE=2026-10-27 python scripts/bi/daily_sync.py

Env:
  PG__URL                          (ingest count / fallback)
  FEISHU_BITABLE_*                 (via site or cargo)
  AGENT_BI_URL                     default http://127.0.0.1:4141/agent/bi
  BI_SYNC_DATE                     optional YYYY-MM-DD
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
INGEST = ROOT / "scripts" / "ingest"


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


def http_json(method: str, url: str, body: dict | None = None, timeout: float = 300) -> dict:
    data = None if body is None else json.dumps(body).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=data,
        method=method,
        headers={"Content-Type": "application/json", "Accept": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8"))


def deposit_count(quote_date: str) -> int:
    """How many DepositQuote rows exist for the date (0 → need Outlook ingest)."""
    if str(INGEST) not in sys.path:
        sys.path.insert(0, str(INGEST))
    from db import connect  # type: ignore

    conn = connect()
    try:
        cur = conn.cursor()
        cur.execute(
            'SELECT COUNT(*) FROM "DepositQuote" WHERE quote_date = %s',
            (quote_date,),
        )
        return int(cur.fetchone()[0])
    finally:
        conn.close()


def run_outlook_ingest(quote_date: str) -> int:
    cmd = [
        sys.executable,
        str(INGEST / "run_ingest.py"),
        "--date",
        quote_date,
    ]
    print("[bi] Outlook ingest:", " ".join(cmd))
    return subprocess.call(cmd, cwd=str(ROOT))


def ensure_platform_quotes(quote_date: str, *, force_ingest: bool) -> int:
    n = deposit_count(quote_date)
    if force_ingest or n == 0:
        if n == 0:
            print(
                f"[bi] DepositQuote empty for {quote_date} — "
                "daily_sync alone cannot invent rows; running Outlook ingest…"
            )
        else:
            print(f"[bi] --ingest: re-scan Outlook for {quote_date} (current rows={n})")
        code = run_outlook_ingest(quote_date)
        if code != 0:
            print(f"[bi] ingest exited {code}", file=sys.stderr)
        n = deposit_count(quote_date)
    print(f"[bi] platform DepositQuote rows for {quote_date}: {n}")
    if n == 0:
        print(
            f"[bi] WARNING: still 0 quotes for {quote_date} "
            "(weekend/holiday, no matching mails, or parse skip). "
            "Feishu sync will write 0 ledger facts for that day."
        )
    return n


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
    sync = http_json("POST", f"{base.rstrip('/')}/sync", payload, timeout=300)
    print(json.dumps(sync, ensure_ascii=False, indent=2))
    facts = int(sync.get("facts") or 0)
    if date and facts == 0:
        print(
            f"[bi] sync returned facts=0 for {date}. "
            "If you expected data, check Outlook Inbox + run_ingest for that date.",
            file=sys.stderr,
        )
        return 2
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


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description="Sync platform OTC quotes → Feishu BI (optionally ingest Outlook first)"
    )
    p.add_argument(
        "date",
        nargs="?",
        default=None,
        help="YYYY-MM-DD (default: BI_SYNC_DATE or latest in DB)",
    )
    p.add_argument(
        "--ingest",
        action="store_true",
        help="Always run scripts/ingest/run_ingest.py for that date before sync",
    )
    p.add_argument(
        "--no-auto-ingest",
        action="store_true",
        help="Do not auto-ingest when DepositQuote is empty for the date",
    )
    return p.parse_args()


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    load_dotenv()
    args = parse_args()
    base = os.environ.get("AGENT_BI_URL", "http://127.0.0.1:4141/agent/bi")
    date = args.date or os.environ.get("BI_SYNC_DATE") or None

    if date and not args.no_auto_ingest:
        ensure_platform_quotes(date, force_ingest=args.ingest)
    elif date and args.ingest:
        ensure_platform_quotes(date, force_ingest=True)

    try:
        return run_via_http(base, date)
    except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError) as e:
        print(f"[bi] HTTP unavailable ({e}); falling back to cargo example")
        return run_via_cargo(date)


if __name__ == "__main__":
    raise SystemExit(main())
