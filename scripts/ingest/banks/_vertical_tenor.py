"""Shared layout: Tenor header then HKD/USD/CNH blocks (CCB / CITIC style)."""
from __future__ import annotations

import sys
from datetime import date
from pathlib import Path

_INGEST = Path(__file__).resolve().parent.parent
if str(_INGEST) not in sys.path:
    sys.path.insert(0, str(_INGEST))

from banks._base import DepositRow, aliases, finish, normalize_currency, normalize_term, parse_rate


def parse_vertical_tenor(
    *,
    subject: str,
    body: str,
    sender: str,
    quote_date: date,
    inst: dict,
    cfg: dict,
    source: str,
    **_kwargs,
):
    al = aliases(cfg)
    lines = [ln.strip() for ln in body.splitlines()]
    try:
        i0 = next(i for i, ln in enumerate(lines) if ln.lower() == "tenor")
    except StopIteration:
        i0 = -1
    ccys: list[str] = []
    i = i0 + 1
    while i < len(lines) and len(ccys) < 3:
        c = normalize_currency(lines[i], al)
        if c:
            ccys.append(c)
        elif lines[i] and normalize_term(lines[i]):
            break
        i += 1
    if not ccys:
        ccys = ["HKD", "USD", "CNY"]

    rows: list[DepositRow] = []
    while i < len(lines):
        ln = lines[i]
        if not ln:
            i += 1
            continue
        if ln.lower().startswith("bank buy") or "kindly note" in ln.lower():
            break
        term = normalize_term(ln)
        if not term:
            i += 1
            continue
        rates: list = []
        j = i + 1
        while j < len(lines) and len(rates) < len(ccys):
            if not lines[j]:
                j += 1
                continue
            if normalize_term(lines[j]):
                break
            if lines[j].lower().startswith("bank"):
                break
            rates.append(lines[j])
            j += 1
        for k, ccy in enumerate(ccys):
            if k >= len(rates):
                break
            rate = parse_rate(rates[k])
            if rate is None:
                continue
            rows.append(
                DepositRow(term=term, currency=ccy, bid=rate, note="indication")
            )
        i = j

    return finish(
        inst=inst,
        sender=sender,
        subject=subject,
        quote_date=quote_date,
        body=body,
        rows=rows,
        source=source,
    )
