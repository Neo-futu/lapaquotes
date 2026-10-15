#!/usr/bin/env python3
"""Apply repo schema.sql (incl. BI tables) to PG__URL."""

from __future__ import annotations

import os
import re
from pathlib import Path

import psycopg2

ROOT = Path(__file__).resolve().parents[2]


def load_dotenv() -> None:
    env_path = ROOT / ".env"
    if not env_path.is_file():
        return
    for line in env_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        os.environ.setdefault(k.strip(), v.strip())


def main() -> None:
    load_dotenv()
    url = os.environ.get("PG__URL") or "postgresql://postgres:postgres@127.0.0.1:5432/lapa"
    sql_path = ROOT / "schema.sql"
    raw = sql_path.read_text(encoding="utf-8")
    # Strip psql meta if any; schema.sql is plain SQL.
    statements = [s.strip() for s in re.split(r";\s*\n", raw) if s.strip() and not s.strip().startswith("--")]
    conn = psycopg2.connect(url)
    conn.autocommit = True
    cur = conn.cursor()
    cur.execute(raw)
    cur.execute(
        """
        SELECT table_name FROM information_schema.tables
        WHERE table_schema='public' AND table_name LIKE 'Bi%'
        ORDER BY 1
        """
    )
    print("BI tables:", [r[0] for r in cur.fetchall()])
    conn.close()


if __name__ == "__main__":
    main()
