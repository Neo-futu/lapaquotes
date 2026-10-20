"""Upsert Institution + DepositQuote; track daily ingest checks + mail logs."""
from __future__ import annotations

import os
import uuid
from datetime import date, datetime
from typing import Any, Optional

import psycopg2
from dotenv import load_dotenv

from common import ParsedQuote


def connect(dsn: Optional[str] = None):
    load_dotenv()
    dsn = dsn or os.environ.get("PG__URL") or os.environ.get("DATABASE_URL")
    if not dsn:
        dsn = "postgresql://postgres:postgres@127.0.0.1:5432/lapa"
    return psycopg2.connect(dsn)


def upsert_institution(
    cur,
    *,
    name: str,
    short_name: Optional[str],
    cn_name: Optional[str],
    email: str,
    ingest_key: Optional[str] = None,
) -> str:
    """Resolve by unique name first (same SMTP can map to CCB HK vs CCB Asia)."""
    cur.execute('SELECT id FROM "Institution" WHERE name = %s', (name,))
    row = cur.fetchone()
    if row:
        cur.execute(
            'UPDATE "Institution" SET email = COALESCE(%s, email), '
            "short_name = COALESCE(%s, short_name), "
            "cn_name = COALESCE(%s, cn_name), "
            "ingest_key = COALESCE(%s, ingest_key), "
            "is_active = true WHERE id = %s "
            "RETURNING id",
            (email or None, short_name, cn_name, ingest_key, row[0]),
        )
        return cur.fetchone()[0]

    iid = uuid.uuid4().hex
    cur.execute(
        'INSERT INTO "Institution" (id, name, short_name, cn_name, email, ingest_key, is_active) '
        "VALUES (%s, %s, %s, %s, %s, %s, true) "
        'ON CONFLICT ("name") DO UPDATE SET '
        "email = COALESCE(EXCLUDED.email, \"Institution\".email), "
        "short_name = COALESCE(EXCLUDED.short_name, \"Institution\".short_name), "
        "cn_name = COALESCE(EXCLUDED.cn_name, \"Institution\".cn_name), "
        "ingest_key = COALESCE(EXCLUDED.ingest_key, \"Institution\".ingest_key) "
        "RETURNING id",
        (iid, name, short_name, cn_name, email or None, ingest_key),
    )
    return cur.fetchone()[0]


def upsert_deposit_quote(
    cur,
    *,
    institution_id: str,
    quote_date,
    term: str,
    term_sort: int,
    currency: str,
    bid,
    ask,
    note: Optional[str],
) -> None:
    cur.execute(
        """
        INSERT INTO "DepositQuote"
          (id, quote_date, institution_id, term, term_sort, currency, bid_rate, ask_rate, note)
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s)
        ON CONFLICT (institution_id, quote_date, term, currency) DO UPDATE SET
          bid_rate = EXCLUDED.bid_rate,
          ask_rate = EXCLUDED.ask_rate,
          note = EXCLUDED.note,
          term_sort = EXCLUDED.term_sort
        """,
        (
            uuid.uuid4().hex,
            quote_date,
            institution_id,
            term,
            term_sort,
            currency,
            bid,
            ask,
            note,
        ),
    )


def save_parsed(conn, parsed: ParsedQuote) -> dict:
    cur = conn.cursor()
    inst_id = upsert_institution(
        cur,
        name=parsed.institution_name,
        short_name=parsed.short_name,
        cn_name=parsed.cn_name,
        email=parsed.sender_email,
        ingest_key=parsed.institution_key,
    )
    n = 0
    for r in parsed.rows:
        upsert_deposit_quote(
            cur,
            institution_id=inst_id,
            quote_date=parsed.quote_date,
            term=r.term,
            term_sort=r.term_sort,
            currency=r.currency,
            bid=r.bid,
            ask=r.ask,
            note=r.note,
        )
        n += 1
    conn.commit()
    return {
        "institution_id": inst_id,
        "institution": parsed.institution_name,
        "quote_date": str(parsed.quote_date),
        "rows": n,
        "subject": parsed.subject,
        "source": parsed.source,
    }


def clear_institution_day(
    conn,
    *,
    institution_key: str,
    quote_date: date | str,
    business: str = "deposit",
) -> dict[str, Any]:
    """Delete DepositQuote for that day + reset IngestDayCheck to pending (for re-ingest tests)."""
    if isinstance(quote_date, str):
        quote_date = date.fromisoformat(quote_date)
    cur = conn.cursor()
    cur.execute(
        'SELECT id FROM "Institution" WHERE ingest_key = %s',
        (institution_key,),
    )
    row = cur.fetchone()
    iid = row[0] if row else None
    deleted = 0
    if iid:
        cur.execute(
            'DELETE FROM "DepositQuote" WHERE institution_id = %s AND quote_date = %s',
            (iid, quote_date),
        )
        deleted = cur.rowcount
    cur.execute(
        """
        INSERT INTO "IngestDayCheck"
          (id, quote_date, business, institution_key, institution_id, status,
           rows_n, mail_count, last_subject, last_entry_id, last_message,
           mail_received_at, checked_at)
        VALUES (%s,%s,%s,%s,%s,'pending',0,0,NULL,NULL,%s,NULL,NULL)
        ON CONFLICT (quote_date, business, institution_key) DO UPDATE SET
          institution_id = COALESCE(EXCLUDED.institution_id, "IngestDayCheck".institution_id),
          status = 'pending',
          rows_n = 0,
          mail_count = 0,
          last_subject = NULL,
          last_entry_id = NULL,
          last_message = EXCLUDED.last_message,
          mail_received_at = NULL,
          checked_at = NULL,
          updated_at = CURRENT_TIMESTAMP
        """,
        (
            uuid.uuid4().hex,
            quote_date,
            business,
            institution_key,
            iid,
            "cleared for re-ingest",
        ),
    )
    conn.commit()
    return {
        "institution_key": institution_key,
        "quote_date": str(quote_date),
        "deleted_quotes": deleted,
        "institution_id": iid,
        "status": "pending",
    }


def ensure_day_roster(conn, quote_date: date, cfg: dict, *, business: str = "deposit") -> int:
    """Insert pending IngestDayCheck rows for every yaml institution on quote_date."""
    cur = conn.cursor()
    n = 0
    for inst in cfg.get("institutions") or []:
        key = inst.get("key")
        if not key:
            continue
        name = inst.get("name") or key
        iid = upsert_institution(
            cur,
            name=name,
            short_name=inst.get("short_name"),
            cn_name=inst.get("cn_name"),
            email=(inst.get("emails") or [None])[0] or "",
            ingest_key=key,
        )
        cur.execute(
            """
            INSERT INTO "IngestDayCheck"
              (id, quote_date, business, institution_key, institution_id, status)
            VALUES (%s, %s, %s, %s, %s, 'pending')
            ON CONFLICT (quote_date, business, institution_key) DO UPDATE SET
              institution_id = COALESCE(EXCLUDED.institution_id, "IngestDayCheck".institution_id),
              updated_at = CURRENT_TIMESTAMP
            """,
            (uuid.uuid4().hex, quote_date, business, key, iid),
        )
        n += 1
    conn.commit()
    return n


def backfill_ok_from_quotes(conn, quote_date: date, *, business: str = "deposit") -> int:
    """Mark day-check ok when DepositQuote rows already exist for the institution."""
    cur = conn.cursor()
    cur.execute(
        """
        UPDATE "IngestDayCheck" AS c SET
          status = 'ok',
          rows_n = GREATEST(c.rows_n, q.cnt),
          checked_at = COALESCE(c.checked_at, CURRENT_TIMESTAMP),
          updated_at = CURRENT_TIMESTAMP,
          last_message = COALESCE(c.last_message, 'backfill from DepositQuote')
        FROM (
          SELECT institution_id, COUNT(*)::int AS cnt
          FROM "DepositQuote"
          WHERE quote_date = %s
          GROUP BY institution_id
        ) AS q
        WHERE c.quote_date = %s
          AND c.business = %s
          AND c.institution_id = q.institution_id
          AND c.status <> 'ok'
        """,
        (quote_date, quote_date, business),
    )
    n = cur.rowcount
    conn.commit()
    return n


def record_ingest_attempt(
    conn,
    *,
    quote_date: date | str,
    business: str,
    institution_key: str,
    institution_id: Optional[str],
    entry_id: Optional[str],
    subject: Optional[str],
    sender: Optional[str],
    status: str,
    rows_n: int = 0,
    message: str = "",
    source: str = "ingest",
    mail_received_at: Optional[datetime] = None,
) -> None:
    """Append mail log + upsert day check (ok wins over pending/skip; error overwrites pending)."""
    if isinstance(quote_date, str):
        quote_date = date.fromisoformat(quote_date)
    if not institution_key or not quote_date:
        return
    cur = conn.cursor()
    cur.execute(
        """
        INSERT INTO "IngestMailLog"
          (id, quote_date, business, institution_key, institution_id, entry_id,
           subject, sender, status, rows_n, message, source, mail_received_at)
        VALUES (%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s)
        """,
        (
            uuid.uuid4().hex,
            quote_date,
            business or "deposit",
            institution_key,
            institution_id,
            entry_id,
            (subject or "")[:500],
            (sender or "")[:200],
            status,
            rows_n,
            (message or "")[:500],
            source,
            mail_received_at,
        ),
    )
    # Load current status to decide transition
    cur.execute(
        """
        SELECT status, rows_n, mail_count FROM "IngestDayCheck"
        WHERE quote_date = %s AND business = %s AND institution_key = %s
        """,
        (quote_date, business or "deposit", institution_key),
    )
    prev = cur.fetchone()
    prev_status = prev[0] if prev else None
    prev_rows = int(prev[1]) if prev else 0
    prev_mails = int(prev[2]) if prev else 0

    new_status = status
    if status == "ok":
        new_status = "ok"
    elif prev_status == "ok" and status in {"skip", "pending", "error"}:
        # keep ok if we already had a good mail that day; still log the failure
        new_status = "ok"
    elif status == "error":
        new_status = "error"
    elif status == "skip" and prev_status in {None, "pending"}:
        new_status = "skip"
    else:
        new_status = prev_status or status

    if new_status == "ok":
        rows_keep = max(prev_rows, rows_n)
    elif status in {"error", "skip"} and prev_status != "ok":
        rows_keep = rows_n
    else:
        rows_keep = rows_n or prev_rows
    checked_at = datetime.now() if new_status == "ok" else None
    # Persist Outlook ReceivedTime for UI (HH:MM).
    # ok → keep earliest successful receive; non-ok → stamp attempt time when not already ok.
    if new_status == "ok":
        recv_for_row = mail_received_at
    elif prev_status != "ok":
        recv_for_row = mail_received_at
    else:
        recv_for_row = None

    cur.execute(
        """
        INSERT INTO "IngestDayCheck"
          (id, quote_date, business, institution_key, institution_id, status,
           rows_n, mail_count, last_subject, last_entry_id, last_message,
           mail_received_at, checked_at)
        VALUES (%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s)
        ON CONFLICT (quote_date, business, institution_key) DO UPDATE SET
          institution_id = COALESCE(EXCLUDED.institution_id, "IngestDayCheck".institution_id),
          status = EXCLUDED.status,
          rows_n = EXCLUDED.rows_n,
          mail_count = "IngestDayCheck".mail_count + 1,
          last_subject = EXCLUDED.last_subject,
          last_entry_id = COALESCE(EXCLUDED.last_entry_id, "IngestDayCheck".last_entry_id),
          last_message = EXCLUDED.last_message,
          mail_received_at = CASE
            -- Already ok: keep earliest successful receive among ok mails.
            WHEN EXCLUDED.status = 'ok' AND "IngestDayCheck".status = 'ok' THEN COALESCE(
              LEAST(
                COALESCE("IngestDayCheck".mail_received_at, EXCLUDED.mail_received_at),
                COALESCE(EXCLUDED.mail_received_at, "IngestDayCheck".mail_received_at)
              ),
              EXCLUDED.mail_received_at,
              "IngestDayCheck".mail_received_at
            )
            -- First ok (or ok after empty/error): stamp the successful mail's ReceivedTime.
            WHEN EXCLUDED.status = 'ok' THEN EXCLUDED.mail_received_at
            WHEN "IngestDayCheck".status = 'ok' THEN "IngestDayCheck".mail_received_at
            ELSE COALESCE(EXCLUDED.mail_received_at, "IngestDayCheck".mail_received_at)
          END,
          checked_at = CASE
            WHEN EXCLUDED.status = 'ok' THEN COALESCE("IngestDayCheck".checked_at, EXCLUDED.checked_at, CURRENT_TIMESTAMP)
            WHEN "IngestDayCheck".status = 'ok' THEN "IngestDayCheck".checked_at
            ELSE NULL
          END,
          updated_at = CURRENT_TIMESTAMP
        """,
        (
            uuid.uuid4().hex,
            quote_date,
            business or "deposit",
            institution_key,
            institution_id,
            new_status,
            rows_keep,
            prev_mails + 1,
            (subject or "")[:500],
            entry_id,
            (message or "")[:500],
            recv_for_row,
            checked_at,
        ),
    )
    conn.commit()
