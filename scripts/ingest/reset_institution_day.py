#!/usr/bin/env python3
"""Clear one institution's DepositQuote + IngestDayCheck for a date; optionally unmark watch state.

Usage:
  python scripts/ingest/reset_institution_day.py --date 2026-10-20 --bank abchk --unmark-watch
  python scripts/ingest/reset_institution_day.py --today --bank hxb --unmark-watch
"""
from __future__ import annotations

import argparse
import datetime as dt
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from db import clear_institution_day, connect  # noqa: E402
from watch_state import WatchState, default_state_path  # noqa: E402


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    p = argparse.ArgumentParser(description="Reset institution day for re-ingest tests")
    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument("--date", help="YYYY-MM-DD")
    g.add_argument("--today", action="store_true")
    p.add_argument("--bank", required=True, help="institution key, e.g. abchk / hxb")
    p.add_argument(
        "--unmark-watch",
        action="store_true",
        help="also delete EntryIDs from watch processed.sqlite3",
    )
    p.add_argument("--state", type=Path, default=None)
    args = p.parse_args()

    qd = dt.date.today() if args.today else dt.date.fromisoformat(args.date)
    conn = connect()
    try:
        report = clear_institution_day(
            conn, institution_key=args.bank, quote_date=qd
        )
    finally:
        conn.close()
    print("cleared:", report)

    if args.unmark_watch:
        state = WatchState(args.state or default_state_path())
        try:
            n = state.unmark_institution(args.bank, quote_date=str(qd))
        finally:
            state.close()
        print(f"unmarked watch entries: {n}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
