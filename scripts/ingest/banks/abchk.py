"""Agricultural Bank of China Hong Kong — Time Deposit Indication (body)."""
from __future__ import annotations

import re
import sys
from datetime import date
from pathlib import Path

_INGEST = Path(__file__).resolve().parent.parent
if str(_INGEST) not in sys.path:
    sys.path.insert(0, str(_INGEST))

from banks._base import DepositRow, aliases, finish, normalize_currency, normalize_term, parse_rate

SCRIPT = "abchk"


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
    # Rates may be plain (`1.15`) or percent-suffixed (`1.15%`); columns are tab-separated.
    line_re = re.compile(
        r"(?P<t1>O/N|T/N|\d+\s*(?:Week|Weeks|month|months|year|years)?)"
        r"\s+(?P<r1>[\d.]+)%?\s+"
        r"(?P<t2>O/N|T/N|\d+\s*(?:Week|Weeks|month|months|year|years)?)"
        r"\s+(?P<r2>[\d.]+)%?\s+"
        r"(?P<t3>O/N|T/N|\d+\s*(?:Week|Weeks|month|months|year|years)?)"
        r"\s+(?P<r3>[\d.]+)%?",
        re.I,
    )
    currencies = ["CNY", "HKD", "USD"]
    for line in body.splitlines():
        m = line_re.search(line.replace("\u2003", " "))
        if not m:
            continue
        for i, ccy in enumerate(currencies, start=1):
            term = normalize_term(m.group(f"t{i}"))
            rate = parse_rate(m.group(f"r{i}"))
            if term and rate is not None:
                rows.append(
                    DepositRow(
                        term=term,
                        currency=normalize_currency(ccy, al) or ccy,
                        bid=rate,
                        note="indication",
                    )
                )
    return finish(
        inst=inst,
        sender=sender,
        subject=subject,
        quote_date=quote_date,
        body=body,
        rows=rows,
        source="abchk/body",
    )


if __name__ == "__main__":
    from banks._runner import run_bank

    raise SystemExit(run_bank(SCRIPT))
