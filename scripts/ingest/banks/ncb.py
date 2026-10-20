"""Nanyang Commercial Bank — today's indicative deposit rates (body table)."""
from __future__ import annotations

import re
import sys
from datetime import date
from pathlib import Path

_INGEST = Path(__file__).resolve().parent.parent
if str(_INGEST) not in sys.path:
    sys.path.insert(0, str(_INGEST))

from banks._base import DepositRow, aliases, finish, normalize_currency, normalize_term, parse_rate

SCRIPT = "ncb"

_CCY_ROW = re.compile(
    r"^(?P<ccy>HKD|USD|CNY|CNH|EUR)\s*(?:>|≧|>=)?\s*\d*\s*M?\s*(?P<rest>.+)$",
    re.I,
)
_TERM_TOKEN = re.compile(r"^(O/N|T/N|\d+\s*[WDYMwdym])$", re.I)


def parse(
    *,
    subject: str,
    body: str,
    sender: str,
    quote_date: date,
    inst: dict,
    cfg: dict,
    **_kwargs,
):
    al = aliases(cfg)
    rows: list[DepositRow] = []
    terms: list[str] = []
    for raw in body.replace("\u2003", " ").splitlines():
        line = raw.strip()
        if not line:
            continue
        # Header: O/N  1W  2W  1M …
        parts = [p for p in re.split(r"[\t ]+", line) if p]
        if not terms and sum(1 for p in parts if _TERM_TOKEN.match(p)) >= 3:
            terms = []
            for p in parts:
                t = normalize_term(p)
                if t:
                    terms.append(t)
            continue
        m = _CCY_ROW.match(line)
        if not m or not terms:
            continue
        ccy = normalize_currency(m.group("ccy"), al) or m.group("ccy").upper()
        vals = [v for v in re.split(r"[\t ]+", m.group("rest").strip()) if v]
        for i, term in enumerate(terms):
            if i >= len(vals):
                break
            rate = parse_rate(vals[i])
            if rate is None:
                continue
            rows.append(
                DepositRow(term=term, currency=ccy, bid=rate, note="indication")
            )
    return finish(
        inst=inst,
        sender=sender,
        subject=subject,
        quote_date=quote_date,
        body=body,
        rows=rows,
        source="ncb/body",
    )


if __name__ == "__main__":
    from banks._runner import run_bank

    raise SystemExit(run_bank(SCRIPT))
