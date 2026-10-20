"""CLI helper: run one bank script against Outlook for a date."""
from __future__ import annotations

import argparse
import datetime as dt
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
INGEST = HERE.parent
if str(INGEST) not in sys.path:
    sys.path.insert(0, str(INGEST))

from common import received_to_date  # noqa: E402
from db import connect, save_parsed  # noqa: E402
from outlook_fetch import fetch_deposit_mails, last_friday  # noqa: E402
from registry import load_cfg, load_parse_fn, match_institution  # noqa: E402


def run_bank(script_stem: str) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")

    p = argparse.ArgumentParser(description=f"Ingest OTC deposit quotes for {script_stem}")
    g = p.add_mutually_exclusive_group()
    g.add_argument("--date", help="YYYY-MM-DD")
    g.add_argument("--friday", action="store_true")
    g.add_argument("--today", action="store_true")
    p.add_argument("--dry-run", action="store_true")
    p.add_argument("--json", action="store_true")
    args = p.parse_args()

    if args.date:
        on_date = dt.date.fromisoformat(args.date)
    elif args.friday:
        on_date = last_friday()
    else:
        # --today or no flag → 默认今天
        on_date = dt.date.today()

    cfg = load_cfg()
    defaults = cfg.get("defaults") or {}
    store = defaults.get("store") or "fixed income"
    folder = defaults.get("folder") or "Inbox"

    # Institutions that use this script (e.g. ccbhk + ccba → ccb)
    targets = [
        inst
        for inst in (cfg.get("institutions") or [])
        if (inst.get("script") or inst.get("parser") or inst.get("key"))
        in {script_stem, f"{script_stem}.py", f"banks/{script_stem}", f"banks.{script_stem}"}
        or inst.get("key") == script_stem
    ]
    if not targets:
        print(f"no institution in institutions.yaml for script={script_stem!r}")
        return 1

    keys = {t["key"] for t in targets}
    print(f"bank={script_stem} keys={sorted(keys)} store={store!r} date={on_date}")
    mails = fetch_deposit_mails(store=store, folder=folder, on_date=on_date)
    reports = []
    conn = None if args.dry_run else connect()

    for mail in mails:
        inst = match_institution(mail.subject, mail.sender, cfg)
        if not inst or inst["key"] not in keys:
            continue
        fn = load_parse_fn(inst)
        parsed = fn(
            subject=mail.subject,
            body=mail.body,
            sender=mail.sender,
            quote_date=received_to_date(mail.received),
            inst=inst,
            cfg=cfg,
            attachment_paths=mail.attachment_paths,
        )
        print(
            f"  OK {inst['key']:8} date={parsed.quote_date} "
            f"rows={len(parsed.rows):3} | {mail.subject[:60]}"
        )
        if args.dry_run:
            reports.append(
                {
                    "institution": inst["key"],
                    "rows": len(parsed.rows),
                    "date": str(parsed.quote_date),
                    "dry_run": True,
                }
            )
            continue
        assert conn is not None
        reports.append(save_parsed(conn, parsed))

    if conn:
        conn.close()
    if args.json:
        print(json.dumps(reports, ensure_ascii=False, indent=2))
    else:
        print(f"done: messages={len(reports)} rows={sum(r.get('rows', 0) for r in reports)}")
    return 0
