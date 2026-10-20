"""Load / save Outlook watch settings (yaml + env overrides)."""
from __future__ import annotations

import os
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Optional

import yaml
from dotenv import load_dotenv

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
CONFIG_PATH = HERE / "watch_config.yaml"
PID_PATH = HERE / ".watch_state" / "watch.pid"

TASK_NAME = r"LapaQuotes\OutlookWatch"


@dataclass
class WatchConfig:
    enabled: bool = True
    auto_with_dev: bool = True
    register_on_logon: bool = False
    store: str = "fixed income"
    folder: str = "Inbox"
    poll_seconds: float = 60.0
    catch_up_minutes: float = 30.0
    no_events: bool = True
    sync_bi: bool = True
    bank: Optional[str] = None
    subject_contains: Optional[str] = None

    def sync_bi_effective(self) -> bool:
        return bool(self.sync_bi)


def _load_yaml() -> dict[str, Any]:
    if not CONFIG_PATH.is_file():
        return {}
    with CONFIG_PATH.open(encoding="utf-8") as f:
        data = yaml.safe_load(f) or {}
    if not isinstance(data, dict):
        return {}
    return data


def load_watch_config(apply_env: bool = True) -> WatchConfig:
    load_dotenv(ROOT / ".env")
    raw = _load_yaml()
    cfg = WatchConfig(
        enabled=bool(raw.get("enabled", True)),
        auto_with_dev=bool(raw.get("auto_with_dev", True)),
        register_on_logon=bool(raw.get("register_on_logon", False)),
        store=str(raw.get("store") or "fixed income"),
        folder=str(raw.get("folder") or "Inbox"),
        poll_seconds=float(raw.get("poll_seconds") or 60),
        catch_up_minutes=float(raw.get("catch_up_minutes") or 30),
        no_events=bool(raw.get("no_events", True)),
        sync_bi=bool(raw.get("sync_bi", True)),
        bank=raw.get("bank") or None,
        subject_contains=raw.get("subject_contains") or None,
    )
    if raw.get("no_bi"):
        cfg.sync_bi = False

    if apply_env:
        # WATCH_OUTLOOK=0|1|true|false
        env_on = os.environ.get("WATCH_OUTLOOK")
        if env_on is not None:
            cfg.enabled = env_on.strip().lower() in {"1", "true", "yes", "on"}
        if os.environ.get("WATCH_OUTLOOK_AUTO_DEV") is not None:
            cfg.auto_with_dev = os.environ["WATCH_OUTLOOK_AUTO_DEV"].strip().lower() in {
                "1",
                "true",
                "yes",
                "on",
            }
        if os.environ.get("WATCH_OUTLOOK_STORE"):
            cfg.store = os.environ["WATCH_OUTLOOK_STORE"]
        if os.environ.get("WATCH_OUTLOOK_FOLDER"):
            cfg.folder = os.environ["WATCH_OUTLOOK_FOLDER"]
        if os.environ.get("WATCH_OUTLOOK_POLL_SECONDS"):
            cfg.poll_seconds = float(os.environ["WATCH_OUTLOOK_POLL_SECONDS"])
        if os.environ.get("WATCH_OUTLOOK_NO_BI", "").strip().lower() in {
            "1",
            "true",
            "yes",
            "on",
        }:
            cfg.sync_bi = False
    return cfg


def save_watch_config(cfg: WatchConfig) -> Path:
    data = asdict(cfg)
    # Keep yaml friendly nulls
    for k in ("bank", "subject_contains"):
        if data.get(k) in ("", None):
            data[k] = None
    text = yaml.safe_dump(data, allow_unicode=True, sort_keys=False)
    CONFIG_PATH.write_text(
        "# Outlook watch — 项目内配置（非 Windows 服务）\n"
        "# 注册到项目：python scripts/ingest/register_watch.py register\n"
        "# 查看状态：  python scripts/ingest/register_watch.py status\n"
        "# 启动项目：  python scripts/dev.py  （auto_with_dev=true 时自动拉起 watch）\n\n"
        + text,
        encoding="utf-8",
    )
    return CONFIG_PATH


def update_watch_config(**kwargs: Any) -> WatchConfig:
    cfg = load_watch_config(apply_env=False)
    for k, v in kwargs.items():
        if not hasattr(cfg, k):
            raise KeyError(f"unknown watch config key: {k}")
        setattr(cfg, k, v)
    save_watch_config(cfg)
    return load_watch_config(apply_env=True)


def build_watch_argv(cfg: WatchConfig, *, python: Optional[str] = None) -> list[str]:
    """CLI args for watch_outlook.py (without interpreter / script path)."""
    args: list[str] = [
        "--store",
        cfg.store,
        "--folder",
        cfg.folder,
        "--poll-seconds",
        str(cfg.poll_seconds),
        "--catch-up-minutes",
        str(cfg.catch_up_minutes),
    ]
    if cfg.no_events:
        args.append("--no-events")
    if not cfg.sync_bi:
        args.append("--no-bi")
    if cfg.bank:
        args.extend(["--bank", cfg.bank])
    if cfg.subject_contains:
        args.extend(["--subject-contains", cfg.subject_contains])
    return args


def watch_command(cfg: Optional[WatchConfig] = None) -> list[str]:
    cfg = cfg or load_watch_config()
    py = os.environ.get("WATCH_OUTLOOK_PYTHON") or sys_executable()
    script = str(HERE / "watch_outlook.py")
    return [py, script, *build_watch_argv(cfg)]


def sys_executable() -> str:
    import sys

    return sys.executable


def read_pid() -> Optional[int]:
    if not PID_PATH.is_file():
        return None
    try:
        return int(PID_PATH.read_text(encoding="utf-8").strip())
    except (ValueError, OSError):
        return None


def write_pid(pid: int) -> None:
    PID_PATH.parent.mkdir(parents=True, exist_ok=True)
    PID_PATH.write_text(str(pid), encoding="utf-8")


def clear_pid() -> None:
    try:
        PID_PATH.unlink(missing_ok=True)
    except OSError:
        pass


def pid_running(pid: Optional[int] = None) -> bool:
    pid = pid if pid is not None else read_pid()
    if not pid:
        return False
    if os.name == "nt":
        import ctypes

        kernel32 = ctypes.windll.kernel32  # type: ignore[attr-defined]
        PROCESS_QUERY_LIMITED_INFORMATION = 0x1000
        handle = kernel32.OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, int(pid))
        if handle:
            kernel32.CloseHandle(handle)
            return True
        return False
    try:
        os.kill(pid, 0)
        return True
    except OSError:
        return False
