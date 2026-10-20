"""Shared helpers: tenor/currency normalize, rate parse, quote rows."""
from __future__ import annotations

import re
from dataclasses import dataclass, field
from datetime import date, datetime
from decimal import Decimal, InvalidOperation
from typing import Iterable, Optional


TERM_SORT = {
    "O/N": 0,
    "T/N": 1,
    "1W": 2,
    "2W": 3,
    "3W": 4,
    "1M": 5,
    "2M": 6,
    "3M": 7,
    "4M": 8,
    "5M": 9,
    "6M": 10,
    "9M": 11,
    "1Y": 12,
    "18M": 13,
    "2Y": 14,
    "3Y": 15,
    "5Y": 16,
    "10Y": 17,
}

_TERM_ALIASES = {
    "o/n": "O/N",
    "on": "O/N",
    "overnight": "O/N",
    "隔夜": "O/N",
    "t/n": "T/N",
    "tn": "T/N",
    "1w": "1W",
    "1 week": "1W",
    "1week": "1W",
    "一周": "1W",
    "2w": "2W",
    "2 week": "2W",
    "2 weeks": "2W",
    "2week": "2W",
    "两周": "2W",
    "3w": "3W",
    "3 week": "3W",
    "3 weeks": "3W",
    "三周": "3W",
    "1m": "1M",
    "1 month": "1M",
    "1month": "1M",
    "1-month": "1M",
    "一个月": "1M",
    "2m": "2M",
    "2 month": "2M",
    "2 months": "2M",
    "2month": "2M",
    "2-month": "2M",
    "两个月": "2M",
    "3m": "3M",
    "3 month": "3M",
    "3 months": "3M",
    "3month": "3M",
    "3-month": "3M",
    "三个月": "3M",
    "4m": "4M",
    "4 month": "4M",
    "4 months": "4M",
    "4-month": "4M",
    "四个月": "4M",
    "5m": "5M",
    "5 month": "5M",
    "5 months": "5M",
    "5-month": "5M",
    "五个月": "5M",
    "6m": "6M",
    "6 month": "6M",
    "6 months": "6M",
    "6-month": "6M",
    "六个月": "6M",
    "9m": "9M",
    "9 month": "9M",
    "9 months": "9M",
    "9-month": "9M",
    "九个月": "9M",
    "1y": "1Y",
    "1 year": "1Y",
    "1year": "1Y",
    "12m": "1Y",
    "12 month": "1Y",
    "12 months": "1Y",
    "12-month": "1Y",
    "十二个月": "1Y",
}


@dataclass
class DepositRow:
    term: str
    currency: str
    bid: Optional[Decimal]
    ask: Optional[Decimal] = None
    note: Optional[str] = None

    @property
    def term_sort(self) -> int:
        return TERM_SORT.get(self.term, 100)


@dataclass
class ParsedQuote:
    institution_key: str
    institution_name: str
    short_name: Optional[str]
    cn_name: Optional[str]
    sender_email: str
    quote_date: date
    subject: str
    rows: list[DepositRow] = field(default_factory=list)
    source: str = ""


def normalize_term(raw: str) -> Optional[str]:
    s = re.sub(r"\s+", " ", (raw or "").strip()).lower()
    s = s.replace("%", "").strip()
    if not s:
        return None
    if s in _TERM_ALIASES:
        return _TERM_ALIASES[s]
    # "1 Month" / "2 Weeks"
    s2 = re.sub(r"s$", "", s)
    if s2 in _TERM_ALIASES:
        return _TERM_ALIASES[s2]
    m = re.fullmatch(r"(\d+)\s*(w|m|y|week|weeks|month|months|year|years)", s)
    if m:
        n, u = m.group(1), m.group(2)[0]
        key = f"{n}{u.upper()}"
        if key == "12M":
            return "1Y"
        return key if key in TERM_SORT else key
    return None


def normalize_currency(raw: str, aliases: Optional[dict] = None) -> Optional[str]:
    s = (raw or "").strip().upper()
    if not s:
        return None
    aliases = aliases or {}
    if s in aliases:
        return aliases[s]
    if s in {"USD", "HKD", "CNY", "EUR", "GBP", "JPY", "AUD", "SGD"}:
        return s
    if s in {"CNH", "RMB"}:
        return aliases.get(s, "CNY")
    return None


def parse_rate(raw) -> Optional[Decimal]:
    if raw is None:
        return None
    if isinstance(raw, (int, float, Decimal)):
        try:
            return Decimal(str(raw))
        except InvalidOperation:
            return None
    s = str(raw).strip()
    if not s or s.upper() in {"TBC", "N/A", "NA", "-", "--", "NIL", "TBD"}:
        return None
    s = s.replace("%", "").replace(",", "").strip()
    try:
        return Decimal(s)
    except InvalidOperation:
        return None


def parse_date_from_text(*texts: str) -> Optional[date]:
    patterns = [
        (r"(\d{4})[/-](\d{1,2})[/-](\d{1,2})", "ymd"),
        (r"(\d{1,2})[/-](\d{1,2})[/-](\d{4})", "dmy"),
        (r"(\d{4})年(\d{1,2})月(\d{1,2})日", "ymd"),
        (r"(\d{1,2})([A-Za-z]{3})(\d{4})", "dMonY"),
        (r"(\d{8})", "ymd8"),
    ]
    months = {
        "jan": 1,
        "feb": 2,
        "mar": 3,
        "apr": 4,
        "may": 5,
        "jun": 6,
        "jul": 7,
        "aug": 8,
        "sep": 9,
        "oct": 10,
        "nov": 11,
        "dec": 12,
    }
    for text in texts:
        if not text:
            continue
        for pat, kind in patterns:
            m = re.search(pat, text)
            if not m:
                continue
            try:
                if kind == "ymd":
                    y, mo, d = int(m.group(1)), int(m.group(2)), int(m.group(3))
                    return date(y, mo, d)
                if kind == "dmy":
                    d, mo, y = int(m.group(1)), int(m.group(2)), int(m.group(3))
                    return date(y, mo, d)
                if kind == "dMonY":
                    d = int(m.group(1))
                    mo = months[m.group(2).lower()[:3]]
                    y = int(m.group(3))
                    return date(y, mo, d)
                if kind == "ymd8":
                    s = m.group(1)
                    return date(int(s[:4]), int(s[4:6]), int(s[6:8]))
            except ValueError:
                continue
    return None


def received_to_date(rt) -> date:
    if isinstance(rt, datetime):
        return rt.date()
    if isinstance(rt, date):
        return rt
    return date(rt.year, rt.month, rt.day)


def dedupe_rows(rows: Iterable[DepositRow]) -> list[DepositRow]:
    """Keep last row per (term, currency)."""
    out: dict[tuple[str, str], DepositRow] = {}
    for r in rows:
        if r.bid is None and r.ask is None:
            continue
        out[(r.term, r.currency)] = r
    return sorted(out.values(), key=lambda x: (x.currency, x.term_sort, x.term))
