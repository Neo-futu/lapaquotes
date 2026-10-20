"""Shared helpers for per-bank scripts."""
from __future__ import annotations

import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
INGEST = HERE.parent
if str(INGEST) not in sys.path:
    sys.path.insert(0, str(INGEST))

from common import (  # noqa: E402
    DepositRow,
    ParsedQuote,
    dedupe_rows,
    normalize_currency,
    normalize_term,
    parse_date_from_text,
    parse_rate,
)


def aliases(cfg: dict) -> dict:
    return (cfg.get("defaults") or {}).get("currency_aliases") or {}


def finish(
    *,
    inst: dict,
    sender: str,
    subject: str,
    quote_date,
    body: str,
    rows: list[DepositRow],
    source: str,
) -> ParsedQuote:
    qd = parse_date_from_text(subject, body) or quote_date
    return ParsedQuote(
        institution_key=inst["key"],
        institution_name=inst["name"],
        short_name=inst.get("short_name"),
        cn_name=inst.get("cn_name"),
        sender_email=sender,
        quote_date=qd,
        subject=subject,
        rows=dedupe_rows(rows),
        source=source,
    )


__all__ = [
    "DepositRow",
    "ParsedQuote",
    "aliases",
    "dedupe_rows",
    "finish",
    "normalize_currency",
    "normalize_term",
    "parse_date_from_text",
    "parse_rate",
]
