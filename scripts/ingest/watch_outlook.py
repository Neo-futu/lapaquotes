#!/usr/bin/env python3
"""
实时监测 Outlook 组邮箱：新邮件 → 判定业务/机构 → ingest 入库 → 同步飞书。

独立常驻进程（不依赖 site）。本地 DepositQuote 与飞书台账均为 upsert：
同机构 + 同报价日 + 同期限/币种已存在则更新，否则新增。

用法：
  python scripts/ingest/watch_outlook.py
  python scripts/ingest/watch_outlook.py --poll-seconds 30 --catch-up-minutes 120
  python scripts/ingest/watch_outlook.py --dry-run --no-bi
  python scripts/ingest/watch_outlook.py --once --catch-up-minutes 60

依赖：已登录桌面 Outlook（pywin32）、Postgres、可选 site :4141（飞书 HTTP）或 cargo bi_sync。
"""
from __future__ import annotations

import argparse
import datetime as dt
import logging
import sys
import tempfile
import threading
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from outlook_fetch import (  # noqa: E402
    _get_folder,
    fetch_deposit_mails,
    fetch_recent_mails,
    item_to_mail,
)
from db import backfill_ok_from_quotes, connect, ensure_day_roster  # noqa: E402
from pipeline import (  # noqa: E402
    mail_process_key,
    process_mail,
    result_to_dict,
    sync_feishu,
)
from registry import load_cfg  # noqa: E402
from watch_config import load_watch_config  # noqa: E402
from watch_state import WatchState, default_state_path  # noqa: E402

log = logging.getLogger("lapa.ingest.watch")


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description="Watch Outlook Inbox → ingest DepositQuote → Feishu BI sync"
    )
    p.add_argument(
        "--store",
        default=None,
        help="Outlook store（默认 watch_config.yaml / institutions.yaml）",
    )
    p.add_argument("--folder", default=None, help="文件夹（默认配置）")
    p.add_argument(
        "--poll-seconds",
        type=float,
        default=None,
        help="轮询间隔秒（默认读 watch_config.yaml）",
    )
    p.add_argument(
        "--catch-up-minutes",
        type=float,
        default=None,
        help="启动回扫分钟数（默认读 watch_config.yaml；0=不回扫）",
    )
    g = p.add_mutually_exclusive_group()
    g.add_argument(
        "--date",
        help="按报价日刷新 YYYY-MM-DD（扫描该日 Inbox 存款邮件并入库）",
    )
    g.add_argument(
        "--today",
        action="store_true",
        help="按今天刷新（默认一次性刷新模式；常驻时仍用 catch-up/轮询）",
    )
    p.add_argument(
        "--subject-contains",
        default=None,
        help="只处理主题包含该字符串的邮件（验证单封，如 today's indicative deposit rates）",
    )
    p.add_argument(
        "--state",
        type=Path,
        default=None,
        help="已处理 EntryID 状态库路径",
    )
    p.add_argument("--bank", default=None, help="只处理指定 institution key")
    p.add_argument("--dry-run", action="store_true", help="解析但不写库/不同步飞书")
    p.add_argument("--no-bi", action="store_true", help="入库后不同步飞书")
    p.add_argument(
        "--once",
        action="store_true",
        help="只做一轮后退出（指定 --date/--today 时默认 once）",
    )
    eg = p.add_mutually_exclusive_group()
    eg.add_argument(
        "--no-events",
        action="store_true",
        help="禁用 ItemAdd，仅轮询",
    )
    eg.add_argument(
        "--with-events",
        action="store_true",
        help="启用 ItemAdd 事件",
    )
    p.add_argument(
        "--force",
        action="store_true",
        help="忽略已处理状态，强制重跑匹配邮件",
    )
    p.add_argument("-v", "--verbose", action="store_true")
    return p.parse_args()


class WatchService:
    def __init__(
        self,
        *,
        store: str,
        folder: str,
        state: WatchState,
        dry_run: bool,
        sync_bi: bool,
        bank: str | None,
        poll_seconds: float,
        use_events: bool,
        subject_contains: str | None = None,
        force: bool = False,
    ):
        self.store = store
        self.folder = folder
        self.state = state
        self.dry_run = dry_run
        self.sync_bi = sync_bi
        self.bank = bank
        self.poll_seconds = max(5.0, poll_seconds)
        self.use_events = use_events
        self.subject_contains = (subject_contains or "").strip().lower() or None
        self.force = force
        self.cfg = load_cfg()
        self._save_dir = Path(tempfile.mkdtemp(prefix="lapa_watch_"))
        self._lock = threading.Lock()
        self._sink = None
        self._stop = threading.Event()

    def _subject_ok(self, subject: str) -> bool:
        if not self.subject_contains:
            return True
        return self.subject_contains in (subject or "").lower()

    def handle_mail_item(self, item) -> None:
        try:
            entry = getattr(item, "EntryID", "") or ""
        except Exception:
            return
        if not entry:
            return
        with self._lock:
            if not self.force and self.state.seen(entry):
                return
            try:
                mail = item_to_mail(
                    item,
                    save_dir=self._save_dir,
                    folder_path=f"{self.store}/{self.folder}",
                )
            except Exception as e:
                log.exception("item_to_mail failed")
                self.state.mark(entry, status="error", message=str(e))
                return
            if not self._subject_ok(mail.subject):
                return
            self._process_locked(mail)

    def handle_mail_message(self, mail) -> None:
        if not self._subject_ok(mail.subject):
            return
        with self._lock:
            if not self.force and self.state.seen(mail.entry_id):
                return
            self._process_locked(mail)

    def _process_locked(self, mail) -> None:
        log.info(
            "mail received=%s from=%s | %s",
            mail.received.isoformat(sep=" ", timespec="minutes"),
            mail.sender,
            (mail.subject or "")[:70],
        )
        result = process_mail(
            mail,
            cfg=self.cfg,
            dry_run=self.dry_run,
            sync_bi=self.sync_bi and not self.dry_run,
            bank_filter=self.bank,
            source="watch",
        )
        # Mark all outcomes (incl. empty/error) so restarts stay idempotent.
        # Re-ingest after DB clear: reset_institution_day + WatchState.unmark_*.
        if not self.dry_run:
            self.state.mark(
                mail.entry_id,
                status=result.status,
                institution=result.institution_key,
                quote_date=result.quote_date,
                business=result.business,
                message=result.message,
            )
        payload = result_to_dict(result)
        if result.status == "ok":
            bi = payload.get("bi_sync") or {}
            log.info(
                "OK %s %s date=%s rows=%s bi=%s",
                result.business,
                result.institution_key,
                result.quote_date,
                result.rows,
                bi.get("via") if bi else ("skipped" if not self.sync_bi else "?"),
            )
        elif result.status == "skip":
            log.info("SKIP %s: %s", result.business or "-", result.message)
        else:
            log.error("ERROR %s: %s", result.institution_key or "-", result.message)

    def catch_up(self, minutes: float) -> int:
        if minutes <= 0:
            return 0
        since = dt.datetime.now() - dt.timedelta(minutes=minutes)
        log.info("catch-up since %s", since.isoformat(sep=" ", timespec="minutes"))
        mails = fetch_recent_mails(
            store=self.store,
            folder=self.folder,
            since=since,
            limit=200,
            save_dir=self._save_dir,
            deposit_only=False,
        )
        n = 0
        for mail in mails:
            if not self.force and self.state.seen(mail.entry_id):
                continue
            self.handle_mail_message(mail)
            n += 1
        log.info("catch-up processed %s mail(s)", n)
        return n

    def refresh_date(self, on_date: dt.date) -> int:
        """Scan deposit mails for a calendar day (default path for --today/--date).

        Ingests all matching mails first, then syncs Feishu once for that date.
        """
        log.info("refresh date=%s store=%r", on_date, self.store)
        if not self.dry_run:
            try:
                conn = connect()
                ensure_day_roster(conn, on_date, self.cfg)
                conn.close()
            except Exception:
                log.exception("ensure_day_roster failed")
        mails = fetch_deposit_mails(
            store=self.store,
            folder=self.folder,
            on_date=on_date,
            limit=200,
            save_dir=self._save_dir,
        )
        # Older first; 「以此為準」 last so correction mail upserts over empty drafts.
        mails = sorted(mails, key=mail_process_key)
        # Temporarily disable per-mail BI; one sync at the end.
        prev_bi = self.sync_bi
        self.sync_bi = False
        n = 0
        try:
            for mail in mails:
                if not self._subject_ok(mail.subject):
                    continue
                if not self.force and self.state.seen(mail.entry_id):
                    log.debug("skip seen %s", (mail.subject or "")[:50])
                    continue
                self.handle_mail_message(mail)
                n += 1
        finally:
            self.sync_bi = prev_bi

        if not self.dry_run:
            try:
                conn = connect()
                backfill_ok_from_quotes(conn, on_date)
                conn.close()
            except Exception:
                log.exception("backfill_ok_from_quotes failed")

        if prev_bi and not self.dry_run and n > 0:
            try:
                bi = sync_feishu(str(on_date))
                log.info("feishu sync %s → %s", on_date, bi)
            except Exception:
                log.exception("feishu sync failed for %s", on_date)
        log.info("refresh date=%s handled=%s matched_inbox=%s", on_date, n, len(mails))
        return n

    def poll_once(self) -> int:
        # Short window so we don't re-scan the whole inbox every tick.
        since = dt.datetime.now() - dt.timedelta(minutes=max(self.poll_seconds / 60.0 * 3, 15))
        mails = fetch_recent_mails(
            store=self.store,
            folder=self.folder,
            since=since,
            limit=60,
            save_dir=self._save_dir,
            deposit_only=False,
        )
        n = 0
        for mail in mails:
            if self.state.seen(mail.entry_id):
                continue
            self.handle_mail_message(mail)
            n += 1
        return n

    def attach_events(self) -> bool:
        if not self.use_events:
            return False
        try:
            import pythoncom  # type: ignore
            import win32com.client  # type: ignore
        except ImportError:
            log.warning("pywin32/pythoncom unavailable; poll-only mode")
            return False

        inbox = _get_folder(self.store, self.folder)
        items = inbox.Items
        svc = self

        class ItemsEvents:
            def OnItemAdd(self, item):
                try:
                    # Restrict to mail items (Class 43)
                    if getattr(item, "Class", None) not in (None, 43):
                        return
                except Exception:
                    pass
                svc.handle_mail_item(item)

        try:
            pythoncom.CoInitialize()
            self._sink = win32com.client.DispatchWithEvents(items, ItemsEvents)
            log.info("Outlook ItemAdd sink attached on %s / %s", self.store, self.folder)
            return True
        except Exception as e:
            log.warning("ItemAdd attach failed (%s); poll-only mode", e)
            self._sink = None
            return False

    def run_forever(self) -> int:
        events_ok = self.attach_events()
        log.info(
            "watching store=%r folder=%r poll=%ss events=%s dry_run=%s sync_bi=%s",
            self.store,
            self.folder,
            self.poll_seconds,
            events_ok,
            self.dry_run,
            self.sync_bi,
        )
        try:
            import pythoncom  # type: ignore
        except ImportError:
            pythoncom = None  # type: ignore

        while not self._stop.is_set():
            try:
                if pythoncom is not None:
                    pythoncom.PumpWaitingMessages()
                n = self.poll_once()
                if n:
                    log.debug("poll ingested %s", n)
            except KeyboardInterrupt:
                break
            except Exception:
                log.exception("watch loop error")
            # Sleep in small slices so Ctrl+C / PumpWaitingMessages stay responsive.
            deadline = time.time() + self.poll_seconds
            while time.time() < deadline and not self._stop.is_set():
                if pythoncom is not None:
                    try:
                        pythoncom.PumpWaitingMessages()
                    except Exception:
                        pass
                time.sleep(0.25)
        return 0

    def stop(self) -> None:
        self._stop.set()


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")

    args = parse_args()
    logging.basicConfig(
        level=logging.DEBUG if args.verbose else logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s: %(message)s",
        datefmt="%H:%M:%S",
    )

    wcfg = load_watch_config()
    cfg = load_cfg()
    defaults = cfg.get("defaults") or {}
    store = args.store or wcfg.store or defaults.get("store") or "fixed income"
    folder = args.folder or wcfg.folder or defaults.get("folder") or "Inbox"
    poll_seconds = (
        args.poll_seconds if args.poll_seconds is not None else wcfg.poll_seconds
    )
    catch_up_minutes = (
        args.catch_up_minutes
        if args.catch_up_minutes is not None
        else wcfg.catch_up_minutes
    )
    if args.with_events:
        use_events = True
    elif args.no_events:
        use_events = False
    else:
        use_events = not wcfg.no_events
    sync_bi = (not args.no_bi) and wcfg.sync_bi
    bank = args.bank if args.bank is not None else wcfg.bank
    subject_contains = (
        args.subject_contains
        if args.subject_contains is not None
        else wcfg.subject_contains
    )

    state_path = args.state or default_state_path()
    state = WatchState(state_path)
    log.info("state db: %s", state_path)
    log.info("watch_config: enabled=%s auto_dev=%s", wcfg.enabled, wcfg.auto_with_dev)

    svc = WatchService(
        store=store,
        folder=folder,
        state=state,
        dry_run=args.dry_run,
        sync_bi=sync_bi,
        bank=bank,
        poll_seconds=poll_seconds,
        use_events=use_events,
        subject_contains=subject_contains,
        force=args.force,
    )

    refresh_date: dt.date | None = None
    if args.date:
        refresh_date = dt.date.fromisoformat(args.date)
    elif args.today:
        refresh_date = dt.date.today()

    # --date/--today implies one-shot refresh unless user wants daemon after.
    once = args.once or refresh_date is not None

    try:
        if refresh_date is not None:
            svc.refresh_date(refresh_date)
        else:
            svc.catch_up(catch_up_minutes)
        if once:
            if refresh_date is None:
                n = svc.poll_once()
                log.info("once mode done (poll new=%s)", n)
            else:
                log.info("date refresh done (%s)", refresh_date)
            return 0
        return svc.run_forever()
    finally:
        state.close()


if __name__ == "__main__":
    raise SystemExit(main())
