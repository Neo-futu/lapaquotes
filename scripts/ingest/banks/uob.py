"""United Overseas Bank Hong Kong — Indicative Deposit Rates (Excel attachment)."""
from __future__ import annotations

import sys
from datetime import date
from pathlib import Path
from typing import Optional

_INGEST = Path(__file__).resolve().parent.parent
if str(_INGEST) not in sys.path:
    sys.path.insert(0, str(_INGEST))

from banks._base import DepositRow, aliases, finish, normalize_currency, normalize_term, parse_rate

SCRIPT = "uob"


def parse(
    *,
    subject: str,
    body: str,
    sender: str,
    quote_date: date,
    inst: dict,
    cfg: dict,
    attachment_paths: Optional[list[Path]] = None,
    **_kwargs,
):
    al = aliases(cfg)
    rows: list[DepositRow] = []
    xlsx_files = [
        p
        for p in (attachment_paths or [])
        if p.suffix.lower() in {".xlsx", ".xlsm", ".xls"}
    ]
    if xlsx_files:
        import openpyxl

        wb = openpyxl.load_workbook(xlsx_files[0], data_only=True)
        ws = wb.active
        headers = None
        for row in ws.iter_rows(values_only=True):
            cells = list(row)
            if not any(cells):
                continue
            if headers is None:
                headers = [
                    str(c).strip().upper() if c is not None else "" for c in cells
                ]
                continue
            term = normalize_term(str(cells[0]) if cells[0] is not None else "")
            if not term:
                continue
            for idx, h in enumerate(headers[1:], start=1):
                ccy = normalize_currency(h, al)
                if not ccy or idx >= len(cells):
                    continue
                rate = parse_rate(cells[idx])
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
        source="uob/xlsx",
    )


if __name__ == "__main__":
    from banks._runner import run_bank

    raise SystemExit(run_bank(SCRIPT))
