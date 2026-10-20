"""Persist processed Outlook EntryIDs so the watcher is idempotent across restarts."""
from __future__ import annotations

import sqlite3
import threading
from datetime import datetime, timezone
from pathlib import Path


class WatchState:
    def __init__(self, path: Path):
        self.path = path
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self._lock = threading.Lock()
        self._conn = sqlite3.connect(str(path), check_same_thread=False)
        self._conn.execute(
            """
            CREATE TABLE IF NOT EXISTS processed (
              entry_id TEXT PRIMARY KEY,
              processed_at TEXT NOT NULL,
              status TEXT NOT NULL,
              institution TEXT,
              quote_date TEXT,
              business TEXT,
              message TEXT
            )
            """
        )
        self._conn.commit()

    def seen(self, entry_id: str) -> bool:
        if not entry_id:
            return False
        with self._lock:
            cur = self._conn.execute(
                "SELECT 1 FROM processed WHERE entry_id = ?", (entry_id,)
            )
            return cur.fetchone() is not None

    def mark(
        self,
        entry_id: str,
        *,
        status: str,
        institution: str = "",
        quote_date: str = "",
        business: str = "",
        message: str = "",
    ) -> None:
        if not entry_id:
            return
        now = datetime.now(timezone.utc).isoformat()
        with self._lock:
            self._conn.execute(
                """
                INSERT INTO processed
                  (entry_id, processed_at, status, institution, quote_date, business, message)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(entry_id) DO UPDATE SET
                  processed_at = excluded.processed_at,
                  status = excluded.status,
                  institution = excluded.institution,
                  quote_date = excluded.quote_date,
                  business = excluded.business,
                  message = excluded.message
                """,
                (
                    entry_id,
                    now,
                    status,
                    institution,
                    quote_date,
                    business,
                    message[:500],
                ),
            )
            self._conn.commit()

    def unmark_entry(self, entry_id: str) -> bool:
        """Forget one EntryID so watch can re-ingest after DB reset."""
        if not entry_id:
            return False
        with self._lock:
            cur = self._conn.execute(
                "DELETE FROM processed WHERE entry_id = ?", (entry_id,)
            )
            self._conn.commit()
            return cur.rowcount > 0

    def unmark_institution(
        self, institution: str, *, quote_date: str | None = None
    ) -> int:
        """Forget processed mails for an institution (optional quote_date filter)."""
        if not institution:
            return 0
        with self._lock:
            if quote_date:
                cur = self._conn.execute(
                    "DELETE FROM processed WHERE institution = ? AND quote_date = ?",
                    (institution, quote_date),
                )
            else:
                cur = self._conn.execute(
                    "DELETE FROM processed WHERE institution = ?",
                    (institution,),
                )
            self._conn.commit()
            return int(cur.rowcount)

    def close(self) -> None:
        with self._lock:
            self._conn.close()


def default_state_path() -> Path:
    return Path(__file__).resolve().parent / ".watch_state" / "processed.sqlite3"


def export_recent(path: Path, limit: int = 20) -> list[dict]:
    if not path.is_file():
        return []
    conn = sqlite3.connect(str(path))
    rows = conn.execute(
        """
        SELECT entry_id, processed_at, status, institution, quote_date, business, message
        FROM processed ORDER BY processed_at DESC LIMIT ?
        """,
        (limit,),
    ).fetchall()
    conn.close()
    keys = [
        "entry_id",
        "processed_at",
        "status",
        "institution",
        "quote_date",
        "business",
        "message",
    ]
    return [dict(zip(keys, r)) for r in rows]
