"""Hua Xia Bank Hong Kong — 存款指示报价（正文表格）."""
from __future__ import annotations

import re
import sys
from datetime import date
from pathlib import Path

_INGEST = Path(__file__).resolve().parent.parent
if str(_INGEST) not in sys.path:
    sys.path.insert(0, str(_INGEST))

from banks._base import DepositRow, aliases, finish, normalize_currency, parse_rate

SCRIPT = "hxb"

CN_TERM = {
    "隔夜": "O/N",
    "一周": "1W",
    "两周": "2W",
    "一个月": "1M",
    "两个月": "2M",
    "三个月": "3M",
    "四个月": "4M",
    "五个月": "5M",
    "六个月": "6M",
    "九个月": "9M",
    "十二个月": "1Y",
}


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
    header_ccy: list[str] = []
    for raw in body.splitlines():
        line = raw.strip()
        if not line:
            continue
        if "期限" in line and re.search(r"USD|HKD|CNH|EUR|CNY", line, re.I):
            parts = re.split(r"[\t ]+", line)
            header_ccy = []
            for p in parts[1:]:
                c = normalize_currency(p, al)
                if c:
                    header_ccy.append(c)
            continue
        for cn, term in CN_TERM.items():
            if not line.startswith(cn):
                continue
            rest = line[len(cn) :].strip()
            vals = [v for v in re.split(r"[\t ]+", rest) if v]
            if not header_ccy:
                header_ccy = ["USD"]
            for i, ccy in enumerate(header_ccy):
                if i >= len(vals):
                    break
                rate = parse_rate(vals[i])
                if rate is None:
                    continue
                rows.append(
                    DepositRow(term=term, currency=ccy, bid=rate, note="indication")
                )
            break
    return finish(
        inst=inst,
        sender=sender,
        subject=subject,
        quote_date=quote_date,
        body=body,
        rows=rows,
        source="hxb/body",
    )


if __name__ == "__main__":
    from banks._runner import run_bank

    raise SystemExit(run_bank(SCRIPT))
