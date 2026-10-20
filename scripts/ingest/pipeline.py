"""Shared ingest pipeline: classify mail → parse → upsert Postgres → sync Feishu.

Local DB upsert is by (institution, quote_date, term, currency).
Feishu sync uses BI sync_key (create or update same day rows).
"""
from __future__ import annotations

import logging
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Optional

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from common import received_to_date  # noqa: E402
from db import connect, record_ingest_attempt, save_parsed  # noqa: E402
from outlook_fetch import DEPOSIT_SUBJ, MailMessage  # noqa: E402
from registry import load_cfg, load_parse_fn, match_institution  # noqa: E402

log = logging.getLogger("lapa.ingest.pipeline")

BOND_SUBJ = re.compile(
    r"\bbond\b|债券|国债|信用债|固收报价|Indicative Bond|Bond Indication",
    re.I,
)
# Authoritative correction mails (e.g. HXB 「以此為準」) should win on same day.
AUTH_SUBJ = re.compile(r"以此[為为]準|以此为准", re.I)


def is_authoritative_subject(subject: str) -> bool:
    return bool(AUTH_SUBJ.search(subject or ""))


def mail_process_key(mail: MailMessage) -> tuple:
    """Sort key: older first; authoritative subjects last so they upsert over drafts."""
    return (1 if is_authoritative_subject(mail.subject) else 0, mail.received)


@dataclass
class ProcessResult:
    status: str  # ok | skip | error
    business: str = ""  # deposit | bond | unknown
    institution_key: str = ""
    quote_date: str = ""
    rows: int = 0
    subject: str = ""
    message: str = ""
    bi_sync: Optional[dict] = None
    detail: dict = field(default_factory=dict)


def classify_business(subject: str) -> str:
    """Return deposit | bond | unknown from subject heuristics."""
    subj = subject or ""
    is_dep = bool(DEPOSIT_SUBJ.search(subj))
    is_bond = bool(BOND_SUBJ.search(subj))
    if is_dep and not is_bond:
        return "deposit"
    if is_bond and not is_dep:
        return "bond"
    if is_dep and is_bond:
        # Prefer deposit when both keywords appear (common in bank marketing).
        return "deposit"
    return "unknown"


def sync_feishu(quote_date: str) -> dict:
    """Push that quote date to Feishu (HTTP agent BI, else cargo bi_sync)."""
    bi_dir = ROOT / "scripts" / "bi"
    if str(bi_dir) not in sys.path:
        sys.path.insert(0, str(bi_dir))
    import daily_sync  # type: ignore

    daily_sync.load_dotenv()
    import os

    base = os.environ.get("AGENT_BI_URL", "http://127.0.0.1:4141/agent/bi")
    try:
        daily_sync.run_via_http(base, quote_date)
        return {"ok": True, "via": "http", "date": quote_date}
    except Exception as e:
        log.warning("BI HTTP failed (%s); falling back to cargo", e)
        code = daily_sync.run_via_cargo(quote_date)
        return {"ok": code == 0, "via": "cargo", "date": quote_date, "exit": code}


def _track(
    conn,
    *,
    result: ProcessResult,
    mail: MailMessage,
    institution_id: Optional[str] = None,
    source: str = "ingest",
    dry_run: bool = False,
) -> None:
    if dry_run or not result.institution_key or not result.quote_date:
        return
    if result.status == "skip" and result.business in {"unknown", ""}:
        return
    try:
        record_ingest_attempt(
            conn,
            quote_date=result.quote_date,
            business=result.business or "deposit",
            institution_key=result.institution_key,
            institution_id=institution_id,
            entry_id=mail.entry_id,
            subject=mail.subject,
            sender=mail.sender,
            status=result.status,
            rows_n=result.rows,
            message=result.message,
            source=source,
            mail_received_at=mail.received,
        )
    except Exception:
        log.exception("record_ingest_attempt failed")


def process_mail(
    mail: MailMessage,
    *,
    cfg: Optional[dict] = None,
    conn=None,
    dry_run: bool = False,
    sync_bi: bool = True,
    bank_filter: Optional[str] = None,
    source: str = "ingest",
) -> ProcessResult:
    """Match institution, parse, upsert DepositQuote, optionally Feishu sync."""
    cfg = cfg or load_cfg()
    business = classify_business(mail.subject)
    subj_short = (mail.subject or "")[:80]
    qd = received_to_date(mail.received)

    if business == "unknown":
        return ProcessResult(
            status="skip",
            business=business,
            subject=subj_short,
            message="not a recognized OTC quote mail",
        )
    if business == "bond":
        return ProcessResult(
            status="skip",
            business=business,
            subject=subj_short,
            message="bond ingest not configured yet",
        )

    inst = match_institution(mail.subject, mail.sender, cfg)
    if not inst:
        return ProcessResult(
            status="skip",
            business=business,
            subject=subj_short,
            message=f"unknown sender/subj: {mail.sender!r}",
        )
    if bank_filter and inst.get("key") != bank_filter:
        return ProcessResult(
            status="skip",
            business=business,
            institution_key=inst["key"],
            subject=subj_short,
            message="bank filter mismatch",
        )

    try:
        fn = load_parse_fn(inst)
    except Exception as e:
        result = ProcessResult(
            status="error",
            business=business,
            institution_key=inst.get("key", ""),
            quote_date=str(qd),
            subject=subj_short,
            message=f"load script: {e}",
        )
        if not dry_run:
            own = conn is None
            c = conn or connect()
            try:
                _track(c, result=result, mail=mail, source=source)
            finally:
                if own:
                    c.close()
        return result

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
        log.exception("parse failed for %s", inst.get("key"))
        result = ProcessResult(
            status="error",
            business=business,
            institution_key=inst.get("key", ""),
            quote_date=str(qd),
            subject=subj_short,
            message=f"parse: {e}",
        )
        if not dry_run:
            own = conn is None
            c = conn or connect()
            try:
                _track(c, result=result, mail=mail, source=source)
            finally:
                if own:
                    c.close()
        return result

    if not parsed.rows:
        # Empty / TBC-only tables are invalid quote mails (Ingest Check ✗), not a quiet skip.
        result = ProcessResult(
            status="error",
            business=business,
            institution_key=inst["key"],
            quote_date=str(parsed.quote_date),
            subject=subj_short,
            message="empty quote mail (0 rows)",
        )
        if not dry_run:
            own = conn is None
            c = conn or connect()
            try:
                _track(c, result=result, mail=mail, source=source)
            finally:
                if own:
                    c.close()
        return result

    if dry_run:
        return ProcessResult(
            status="ok",
            business=business,
            institution_key=inst["key"],
            quote_date=str(parsed.quote_date),
            rows=len(parsed.rows),
            subject=subj_short,
            message="dry-run (not written)",
            detail={"dry_run": True},
        )

    own_conn = conn is None
    if own_conn:
        conn = connect()
    report: dict[str, Any] = {}
    try:
        report = save_parsed(conn, parsed)
        result = ProcessResult(
            status="ok",
            business=business,
            institution_key=inst["key"],
            quote_date=str(parsed.quote_date),
            rows=int(report.get("rows") or 0),
            subject=subj_short,
            message="upserted local + feishu queued",
            detail=report,
        )
        _track(
            conn,
            result=result,
            mail=mail,
            institution_id=report.get("institution_id"),
            source=source,
        )
    except Exception as e:
        log.exception("db upsert failed")
        result = ProcessResult(
            status="error",
            business=business,
            institution_key=inst["key"],
            quote_date=str(parsed.quote_date),
            subject=subj_short,
            message=f"db: {e}",
        )
        try:
            _track(conn, result=result, mail=mail, source=source)
        except Exception:
            pass
        if own_conn and conn is not None:
            conn.close()
        return result

    bi: Optional[dict] = None
    if sync_bi:
        try:
            bi = sync_feishu(str(parsed.quote_date))
        except Exception as e:
            log.exception("feishu sync failed")
            bi = {"ok": False, "error": str(e), "date": str(parsed.quote_date)}
    result.bi_sync = bi

    if own_conn and conn is not None:
        conn.close()
    return result


def result_to_dict(r: ProcessResult) -> dict[str, Any]:
    return {
        "status": r.status,
        "business": r.business,
        "institution": r.institution_key,
        "quote_date": r.quote_date,
        "rows": r.rows,
        "subject": r.subject,
        "message": r.message,
        "bi_sync": r.bi_sync,
        "detail": r.detail,
    }
