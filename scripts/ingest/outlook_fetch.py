"""Fetch OTC deposit mails from local Outlook (win32com)."""
from __future__ import annotations

import datetime as dt
import re
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

import win32com.client  # type: ignore

from common import received_to_date

DEPOSIT_SUBJ = re.compile(
    r"deposit|定存|定期存款|Deposit Rate|Indicative Deposit|Time Deposit",
    re.I,
)


@dataclass
class MailMessage:
    entry_id: str
    subject: str
    sender: str
    sender_name: str
    received: dt.datetime
    body: str
    html: str = ""
    attachment_paths: list[Path] = field(default_factory=list)
    folder_path: str = ""


def _get_namespace():
    return win32com.client.Dispatch("Outlook.Application").GetNamespace("MAPI")


def _get_folder(store_name: str, folder_name: str):
    outlook = _get_namespace()
    store = None
    for i in range(1, outlook.Folders.Count + 1):
        f = outlook.Folders.Item(i)
        if f.Name.lower() == store_name.lower():
            store = f
            break
    if store is None:
        raise RuntimeError(f"Outlook store not found: {store_name!r}")
    folder = None
    for i in range(1, store.Folders.Count + 1):
        f = store.Folders.Item(i)
        if f.Name.lower() == folder_name.lower():
            folder = f
            break
    if folder is None:
        raise RuntimeError(
            f"Folder {folder_name!r} not found under store {store_name!r}"
        )
    return folder


def _smtp_sender(item) -> str:
    sender = getattr(item, "SenderEmailAddress", "") or ""
    if sender.startswith("/"):
        try:
            sender = item.Sender.GetExchangeUser().PrimarySmtpAddress
        except Exception:
            sender = getattr(item, "SenderName", "") or sender
    return sender


def item_to_mail(item, *, save_dir: Optional[Path] = None, folder_path: str = "") -> MailMessage:
    """Convert an Outlook MailItem COM object to MailMessage (saves attachments)."""
    save_dir = save_dir or Path(tempfile.mkdtemp(prefix="lapa_ingest_"))
    save_dir.mkdir(parents=True, exist_ok=True)
    rt = item.ReceivedTime
    recv = dt.datetime(rt.year, rt.month, rt.day, rt.hour, rt.minute, rt.second)
    subj = getattr(item, "Subject", "") or ""
    sender = _smtp_sender(item)
    body = getattr(item, "Body", "") or ""
    html = getattr(item, "HTMLBody", "") or ""
    att_paths: list[Path] = []
    entry = getattr(item, "EntryID", "") or ""
    try:
        for att in item.Attachments:
            name = att.FileName
            if not name:
                continue
            if Path(name).suffix.lower() in {".png", ".jpg", ".jpeg", ".gif"}:
                continue
            dest = save_dir / f"{entry[-12:]}_{name}"
            att.SaveAsFile(str(dest))
            att_paths.append(dest)
    except Exception:
        pass
    return MailMessage(
        entry_id=entry,
        subject=subj,
        sender=sender,
        sender_name=getattr(item, "SenderName", "") or "",
        received=recv,
        body=body,
        html=html,
        attachment_paths=att_paths,
        folder_path=folder_path,
    )


def fetch_recent_mails(
    *,
    store: str = "fixed income",
    folder: str = "Inbox",
    since: Optional[dt.datetime] = None,
    limit: int = 80,
    save_dir: Optional[Path] = None,
    deposit_only: bool = False,
) -> list[MailMessage]:
    """Newest-first scan; return mails with ReceivedTime >= since (oldest first)."""
    inbox = _get_folder(store, folder)
    items = inbox.Items
    items.Sort("[ReceivedTime]", True)
    save_dir = save_dir or Path(tempfile.mkdtemp(prefix="lapa_ingest_"))
    save_dir.mkdir(parents=True, exist_ok=True)
    folder_path = getattr(inbox, "FolderPath", "") or ""

    out: list[MailMessage] = []
    checked = 0
    for item in items:
        checked += 1
        if checked > limit * 4:
            break
        if len(out) >= limit:
            break
        try:
            rt = item.ReceivedTime
            recv = dt.datetime(rt.year, rt.month, rt.day, rt.hour, rt.minute, rt.second)
        except Exception:
            continue
        if since and recv < since:
            break
        subj = getattr(item, "Subject", "") or ""
        if deposit_only and not DEPOSIT_SUBJ.search(subj):
            continue
        try:
            out.append(item_to_mail(item, save_dir=save_dir, folder_path=folder_path))
        except Exception:
            continue
    out.sort(key=lambda m: m.received)
    return out


def last_friday(today: Optional[dt.date] = None) -> dt.date:
    """Today if Friday, else the most recent past Friday."""
    today = today or dt.date.today()
    return today - dt.timedelta(days=(today.weekday() - 4) % 7)


def fetch_deposit_mails(
    *,
    store: str = "fixed income",
    folder: str = "Inbox",
    on_date: Optional[dt.date] = None,
    since: Optional[dt.date] = None,
    until: Optional[dt.date] = None,
    save_dir: Optional[Path] = None,
    limit: int = 200,
) -> list[MailMessage]:
    """Return deposit-related mails, oldest first."""
    inbox = _get_folder(store, folder)
    items = inbox.Items
    items.Sort("[ReceivedTime]", True)
    save_dir = save_dir or Path(tempfile.mkdtemp(prefix="lapa_ingest_"))
    save_dir.mkdir(parents=True, exist_ok=True)

    out: list[MailMessage] = []
    checked = 0
    for item in items:
        checked += 1
        if checked > limit * 3 and not out:
            break
        if len(out) >= limit:
            break
        try:
            rt = item.ReceivedTime
            d = received_to_date(rt)
        except Exception:
            continue
        if on_date and d != on_date:
            if d < on_date:
                # sorted desc; can stop early when past target day
                if checked > 5:
                    break
            continue
        if since and d < since:
            break
        if until and d > until:
            continue
        subj = getattr(item, "Subject", "") or ""
        if not DEPOSIT_SUBJ.search(subj):
            continue
        try:
            out.append(
                item_to_mail(
                    item,
                    save_dir=save_dir,
                    folder_path=getattr(inbox, "FolderPath", "") or "",
                )
            )
        except Exception:
            continue
    out.sort(key=lambda m: m.received)
    return out
