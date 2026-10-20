"""
从本地 Outlook「fixed income」收件箱拉取 OTC 银行存款报价邮件 → 按机构脚本解析 → 写入 lapa.DepositQuote。

用法：
    python scripts/ingest/run_ingest.py              # 默认今天
    python scripts/ingest/run_ingest.py --today
    python scripts/ingest/run_ingest.py --date 2026-10-20
    python scripts/ingest/run_ingest.py --friday
    python scripts/ingest/run_ingest.py --date 2026-10-16 --dry-run
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from common import received_to_date  # noqa: E402
from db import (  # noqa: E402
    backfill_ok_from_quotes,
    connect,
    ensure_day_roster,
    record_ingest_attempt,
    save_parsed,
)
from outlook_fetch import fetch_deposit_mails, last_friday  # noqa: E402
from registry import load_cfg, load_parse_fn, match_institution  # noqa: E402


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Outlook OTC deposit quote → Postgres")
    g = p.add_mutually_exclusive_group()
    g.add_argument("--date", help="报价日 YYYY-MM-DD（按邮件 ReceivedTime 过滤）")
    g.add_argument("--friday", action="store_true", help="最近一个周五")
    g.add_argument("--today", action="store_true", help="今天")
    p.add_argument("--store", default=None, help="Outlook 邮箱/组，默认 institutions.yaml")
    p.add_argument("--folder", default=None, help="文件夹名，默认 Inbox")
    p.add_argument("--bank", default=None, help="只跑指定 institution key，如 abchk")
    p.add_argument("--dry-run", action="store_true", help="只解析不写库")
    p.add_argument("--json", action="store_true", help="JSON 输出汇总")
    return p.parse_args()


def resolve_date(args: argparse.Namespace) -> dt.date | None:
    if args.date:
        return dt.date.fromisoformat(args.date)
    if args.friday:
        return last_friday()
    # --today or no flag → 默认今天
    return dt.date.today()


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")

    args = parse_args()
    cfg = load_cfg()
    defaults = cfg.get("defaults") or {}
    store = args.store or defaults.get("store") or "fixed income"
    folder = args.folder or defaults.get("folder") or "Inbox"
    on_date = resolve_date(args)

    print(f"Outlook store={store!r} folder={folder!r} date={on_date}")
    mails = fetch_deposit_mails(store=store, folder=folder, on_date=on_date)
    print(f"matched deposit mails: {len(mails)}")

    reports = []
    conn = None if args.dry_run else connect()
    if conn and on_date:
        ensure_day_roster(conn, on_date, cfg)

    for mail in mails:
        inst = match_institution(mail.subject, mail.sender, cfg)
        if not inst:
            print(f"  SKIP unknown sender/subj: {mail.sender!r} | {mail.subject!r}")
            continue
        if args.bank and inst["key"] != args.bank:
            continue
        try:
            fn = load_parse_fn(inst)
        except Exception as e:
            print(f"  SKIP load script for {inst['key']}: {e}")
            if conn:
                record_ingest_attempt(
                    conn,
                    quote_date=on_date or received_to_date(mail.received),
                    business="deposit",
                    institution_key=inst["key"],
                    institution_id=None,
                    entry_id=mail.entry_id,
                    subject=mail.subject,
                    sender=mail.sender,
                    status="error",
                    message=f"load script: {e}",
                    source="run_ingest",
                    mail_received_at=mail.received,
                )
            continue
        qd = received_to_date(mail.received)
        try:
            parsed = fn(
                subject=mail.subject,
                body=mail.body,
                sender=mail.sender,
                quote_date=qd,
                inst=inst,
                cfg=cfg,
                attachment_paths=mail.attachment_paths,
            )
        except Exception as e:
            print(f"  ERR {inst['key']:8} parse: {e}")
            if conn:
                record_ingest_attempt(
                    conn,
                    quote_date=qd,
                    business="deposit",
                    institution_key=inst["key"],
                    institution_id=None,
                    entry_id=mail.entry_id,
                    subject=mail.subject,
                    sender=mail.sender,
                    status="error",
                    message=f"parse: {e}",
                    source="run_ingest",
                    mail_received_at=mail.received,
                )
            continue
        print(
            f"  OK {inst['key']:8} date={parsed.quote_date} "
            f"rows={len(parsed.rows):3} | {mail.subject[:60]}"
        )
        if args.dry_run:
            sample = [
                {
                    "term": r.term,
                    "ccy": r.currency,
                    "bid": str(r.bid) if r.bid is not None else None,
                }
                for r in parsed.rows[:5]
            ]
            print(f"       sample={sample}")
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
        if not parsed.rows:
            record_ingest_attempt(
                conn,
                quote_date=parsed.quote_date,
                business="deposit",
                institution_key=inst["key"],
                institution_id=None,
                entry_id=mail.entry_id,
                subject=mail.subject,
                sender=mail.sender,
                status="skip",
                message="parser returned 0 rows",
                source="run_ingest",
                mail_received_at=mail.received,
            )
            continue
        report = save_parsed(conn, parsed)
        record_ingest_attempt(
            conn,
            quote_date=parsed.quote_date,
            business="deposit",
            institution_key=inst["key"],
            institution_id=report.get("institution_id"),
            entry_id=mail.entry_id,
            subject=mail.subject,
            sender=mail.sender,
            status="ok",
            rows_n=int(report.get("rows") or 0),
            message="upserted",
            source="run_ingest",
            mail_received_at=mail.received,
        )
        reports.append(report)

    if conn and on_date:
        backfill_ok_from_quotes(conn, on_date)
        conn.close()

    if args.json:
        print(json.dumps(reports, ensure_ascii=False, indent=2))
    else:
        total = sum(r.get("rows", 0) for r in reports)
        print(f"done: messages={len(reports)} deposit_rows_upserted={total}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
