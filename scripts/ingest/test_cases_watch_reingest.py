#!/usr/bin/env python3
"""
Verify three watch/ingest cases (ABCHK re-ingest + HXB empty + HXB 以此為準).

See: scripts/ingest/TEST_CASES_WATCH.md

Usage:
  python scripts/ingest/test_cases_watch_reingest.py
  python scripts/ingest/test_cases_watch_reingest.py --date 2026-10-20 --no-bi
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from db import clear_institution_day, connect  # noqa: E402
from outlook_fetch import fetch_deposit_mails  # noqa: E402
from pipeline import (  # noqa: E402
    is_authoritative_subject,
    mail_process_key,
    process_mail,
    result_to_dict,
)
from registry import load_cfg, match_institution  # noqa: E402
from watch_state import WatchState, default_state_path  # noqa: E402


def _quote_count(conn, institution_key: str, quote_date: dt.date) -> int:
    cur = conn.cursor()
    cur.execute(
        """
        SELECT COUNT(*) FROM "DepositQuote" d
        JOIN "Institution" i ON i.id = d.institution_id
        WHERE i.ingest_key = %s AND d.quote_date = %s
        """,
        (institution_key, quote_date),
    )
    return int(cur.fetchone()[0])


def _day_check(conn, institution_key: str, quote_date: dt.date) -> dict:
    cur = conn.cursor()
    cur.execute(
        """
        SELECT status, rows_n, last_message,
               to_char(mail_received_at, 'HH24:MI')
        FROM "IngestDayCheck"
        WHERE institution_key = %s AND quote_date = %s AND business = 'deposit'
        """,
        (institution_key, quote_date),
    )
    row = cur.fetchone()
    if not row:
        return {}
    return {
        "status": row[0],
        "rows_n": row[1],
        "last_message": row[2],
        "received_hm": row[3],
    }


def _find_mails(mails, cfg, *, bank: str, pred):
    out = []
    for m in mails:
        inst = match_institution(m.subject, m.sender, cfg)
        if not inst or inst.get("key") != bank:
            continue
        if pred(m):
            out.append(m)
    return out


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")

    p = argparse.ArgumentParser()
    p.add_argument("--date", default=None, help="default: today")
    p.add_argument("--store", default="fixed income")
    p.add_argument("--folder", default="Inbox")
    p.add_argument("--no-bi", action="store_true")
    p.add_argument("--state", type=Path, default=None)
    args = p.parse_args()

    qd = dt.date.fromisoformat(args.date) if args.date else dt.date.today()
    cfg = load_cfg()
    sync_bi = not args.no_bi
    save_dir = Path(tempfile.mkdtemp(prefix="lapa_cases_"))
    state = WatchState(args.state or default_state_path())

    report: dict = {"date": str(qd), "cases": {}}
    ok_all = True

    print(f"=== load Outlook deposit mails for {qd} ===")
    mails = fetch_deposit_mails(
        store=args.store, folder=args.folder, on_date=qd, limit=200, save_dir=save_dir
    )
    mails = sorted(mails, key=mail_process_key)
    print(f"matched {len(mails)} deposit mail(s)")

    abchk_mails = _find_mails(
        mails,
        cfg,
        bank="abchk",
        pred=lambda m: "ABCHK" in (m.subject or "")
        and "Indicative Deposit Rates" in (m.subject or ""),
    )
    hxb_empty = _find_mails(
        mails,
        cfg,
        bank="hxb",
        pred=lambda m: not is_authoritative_subject(m.subject),
    )
    # Prefer the empty table mail (gordon 11:47 style): subject without 以此為準,
    # and body that yields 0 rows when parsed — pick newest non-auth among those
    # that are NOT the early Vicky mail with rates. Heuristic: subject exact
    # without 以此, and received after 11:00 if multiple.
    hxb_auth = _find_mails(
        mails, cfg, bank="hxb", pred=lambda m: is_authoritative_subject(m.subject)
    )

    # Refine empty: among non-auth HXB, choose the one that dry-runs to 0 rows.
    hxb_empty_pick = None
    for m in sorted(hxb_empty, key=lambda x: x.received, reverse=True):
        dry = process_mail(m, cfg=cfg, dry_run=True, sync_bi=False, source="case-probe")
        if dry.rows == 0:
            hxb_empty_pick = m
            break
    if hxb_empty_pick is None and hxb_empty:
        # fallback: latest non-auth
        hxb_empty_pick = sorted(hxb_empty, key=lambda x: x.received)[-1]

    # ---------- Case 1: ABCHK delete → watch re-ingest ----------
    print("\n=== CASE 1: ABCHK clear → re-ingest ===")
    case1 = {"ok": False}
    if not abchk_mails:
        case1["error"] = "no ABCHK Indicative Deposit Rates mail found"
        ok_all = False
    else:
        mail = abchk_mails[0]
        conn = connect()
        try:
            cleared = clear_institution_day(
                conn, institution_key="abchk", quote_date=qd
            )
            n_un = state.unmark_institution("abchk", quote_date=str(qd))
            assert _quote_count(conn, "abchk", qd) == 0
            chk = _day_check(conn, "abchk", qd)
            assert chk.get("status") == "pending", chk
            case1["cleared"] = cleared
            case1["unmarked"] = n_un
            case1["after_clear"] = chk
        finally:
            conn.close()

        result = process_mail(
            mail, cfg=cfg, dry_run=False, sync_bi=sync_bi, source="case1-watch"
        )
        state.mark(
            mail.entry_id,
            status=result.status,
            institution=result.institution_key,
            quote_date=result.quote_date,
            business=result.business,
            message=result.message,
        )
        conn = connect()
        try:
            qn = _quote_count(conn, "abchk", qd)
            chk = _day_check(conn, "abchk", qd)
        finally:
            conn.close()
        case1["process"] = result_to_dict(result)
        case1["quotes"] = qn
        case1["check"] = chk
        case1["ok"] = (
            result.status == "ok"
            and qn > 0
            and chk.get("status") == "ok"
            and (not sync_bi or bool((result.bi_sync or {}).get("ok")))
        )
        if not case1["ok"]:
            ok_all = False
        print(
            f"  subject={mail.subject[:60]!r}\n"
            f"  status={result.status} rows={result.rows} quotes={qn} "
            f"check={chk} bi={result.bi_sync}"
        )
        print(f"  PASS={case1['ok']}")
    report["cases"]["1_abchk_reingest"] = case1

    # ---------- Case 2: HXB empty/wrong mail only ----------
    print("\n=== CASE 2: HXB empty mail → Ingest Check error ===")
    case2 = {"ok": False}
    if hxb_empty_pick is None:
        case2["error"] = "no empty HXB mail found"
        ok_all = False
    else:
        mail = hxb_empty_pick
        conn = connect()
        try:
            cleared = clear_institution_day(conn, institution_key="hxb", quote_date=qd)
            n_un = state.unmark_institution("hxb", quote_date=str(qd))
            assert _quote_count(conn, "hxb", qd) == 0
            case2["cleared"] = cleared
            case2["unmarked"] = n_un
        finally:
            conn.close()

        result = process_mail(
            mail, cfg=cfg, dry_run=False, sync_bi=False, source="case2-empty"
        )
        state.mark(
            mail.entry_id,
            status=result.status,
            institution=result.institution_key,
            quote_date=result.quote_date,
            business=result.business,
            message=result.message,
        )
        conn = connect()
        try:
            qn = _quote_count(conn, "hxb", qd)
            chk = _day_check(conn, "hxb", qd)
        finally:
            conn.close()
        case2["subject"] = mail.subject
        case2["process"] = result_to_dict(result)
        case2["quotes"] = qn
        case2["check"] = chk
        case2["ok"] = (
            result.status == "error"
            and qn == 0
            and chk.get("status") == "error"
            and "0 rows" in (chk.get("last_message") or "")
        )
        if not case2["ok"]:
            ok_all = False
        print(
            f"  subject={mail.subject[:70]!r}\n"
            f"  status={result.status} quotes={qn} check={chk}"
        )
        print(f"  PASS={case2['ok']}")
    report["cases"]["2_hxb_empty"] = case2

    # ---------- Case 3: HXB 以此為準 upsert ----------
    print("\n=== CASE 3: HXB (以此為準) → upsert ok + Feishu ===")
    case3 = {"ok": False}
    if not hxb_auth:
        case3["error"] = "no HXB 以此為準 mail found"
        ok_all = False
    else:
        mail = sorted(hxb_auth, key=lambda m: m.received)[-1]
        # ensure watch would re-see it
        state.unmark_entry(mail.entry_id)
        result = process_mail(
            mail, cfg=cfg, dry_run=False, sync_bi=sync_bi, source="case3-auth"
        )
        state.mark(
            mail.entry_id,
            status=result.status,
            institution=result.institution_key,
            quote_date=result.quote_date,
            business=result.business,
            message=result.message,
        )
        conn = connect()
        try:
            qn = _quote_count(conn, "hxb", qd)
            chk = _day_check(conn, "hxb", qd)
        finally:
            conn.close()
        case3["subject"] = mail.subject
        case3["process"] = result_to_dict(result)
        case3["quotes"] = qn
        case3["check"] = chk
        case3["ok"] = (
            result.status == "ok"
            and qn > 0
            and chk.get("status") == "ok"
            and (not sync_bi or bool((result.bi_sync or {}).get("ok")))
        )
        if not case3["ok"]:
            ok_all = False
        print(
            f"  subject={mail.subject[:70]!r}\n"
            f"  status={result.status} rows={result.rows} quotes={qn} "
            f"check={chk} bi={result.bi_sync}"
        )
        print(f"  PASS={case3['ok']}")
    report["cases"]["3_hxb_authoritative"] = case3

    state.close()
    report["all_passed"] = ok_all
    out_path = HERE / ".watch_state" / f"test_cases_{qd.isoformat()}.json"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(
        json.dumps(report, ensure_ascii=False, indent=2, default=str), encoding="utf-8"
    )
    print(f"\n=== RESULT all_passed={ok_all} → {out_path} ===")
    return 0 if ok_all else 1


if __name__ == "__main__":
    raise SystemExit(main())
