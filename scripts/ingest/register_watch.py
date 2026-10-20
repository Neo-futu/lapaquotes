#!/usr/bin/env python3
"""在 lapaquotes 项目内注册 / 配置 Outlook watch（写入 watch_config.yaml）。

主路径（项目内，非 Windows 服务）：
  python scripts/ingest/register_watch.py register     # enabled+auto_with_dev → 启动 dev.py 时自动 watch
  python scripts/ingest/register_watch.py status
  python scripts/ingest/register_watch.py enable|disable
  python scripts/ingest/register_watch.py configure --poll-seconds 45
  python scripts/ingest/register_watch.py start|stop

可选（Windows 登录计划任务，一般不需要）：
  python scripts/ingest/register_watch.py register-logon
  python scripts/ingest/register_watch.py unregister-logon

启动项目：
  python scripts/dev.py                # 若已 register，自动拉起 watch_outlook
  python scripts/dev.py --no-outlook-watch
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from watch_config import (  # noqa: E402
    CONFIG_PATH,
    TASK_NAME,
    WatchConfig,
    clear_pid,
    load_watch_config,
    pid_running,
    read_pid,
    save_watch_config,
    update_watch_config,
    watch_command,
    write_pid,
)


def cmd_status(_: argparse.Namespace) -> int:
    cfg = load_watch_config()
    pid = read_pid()
    running = pid_running(pid)
    print(f"config:      {CONFIG_PATH}")
    print(f"exists:      {CONFIG_PATH.is_file()}")
    print(f"enabled:     {cfg.enabled}")
    print(f"auto_dev:    {cfg.auto_with_dev}  ← python scripts/dev.py 时自动启动")
    print(f"store:       {cfg.store!r} / {cfg.folder!r}")
    print(f"poll:        {cfg.poll_seconds}s  catch_up={cfg.catch_up_minutes}m")
    print(f"no_events:   {cfg.no_events}  sync_bi={cfg.sync_bi}")
    print(f"process:     pid={pid} running={running}")
    print(f"logon_task:  {cfg.register_on_logon} ({_task_query()})")
    print(f"command:     {subprocess.list2cmdline(watch_command(cfg))}")
    if cfg.enabled and cfg.auto_with_dev:
        print("hint:        已在项目内注册；运行 python scripts/dev.py 会自动启动 watch")
    else:
        print("hint:        尚未完整注册；执行: python scripts/ingest/register_watch.py register")
    return 0


def cmd_register(_: argparse.Namespace) -> int:
    """项目内注册：写入 yaml，dev.py 启动时自动拉起 watch。"""
    if not CONFIG_PATH.is_file():
        save_watch_config(WatchConfig())
    cfg = update_watch_config(enabled=True, auto_with_dev=True)
    print(f"registered in project → {CONFIG_PATH}")
    print(f"  enabled={cfg.enabled}  auto_with_dev={cfg.auto_with_dev}")
    print("start project with:  python scripts/dev.py")
    print("or start watch now:  python scripts/ingest/register_watch.py start")
    return 0


def cmd_unregister_project(_: argparse.Namespace) -> int:
    """取消项目内自动启动（不删 yaml，只关开关）。"""
    update_watch_config(enabled=False, auto_with_dev=False)
    print("unregistered from project auto-start (enabled=false, auto_with_dev=false)")
    return 0


def cmd_enable(_: argparse.Namespace) -> int:
    update_watch_config(enabled=True)
    print("enabled=true")
    return 0


def cmd_disable(_: argparse.Namespace) -> int:
    update_watch_config(enabled=False)
    print("enabled=false（dev.py 将不再自动启动 watch）")
    return 0


def cmd_init(_: argparse.Namespace) -> int:
    if CONFIG_PATH.is_file():
        print(f"already exists: {CONFIG_PATH}")
        return 0
    path = save_watch_config(WatchConfig())
    print(f"wrote defaults → {path}")
    return 0


def cmd_configure(args: argparse.Namespace) -> int:
    updates = {}
    if args.enabled is not None:
        updates["enabled"] = args.enabled
    if args.auto_with_dev is not None:
        updates["auto_with_dev"] = args.auto_with_dev
    if args.register_on_logon is not None:
        updates["register_on_logon"] = args.register_on_logon
    if args.store:
        updates["store"] = args.store
    if args.folder:
        updates["folder"] = args.folder
    if args.poll_seconds is not None:
        updates["poll_seconds"] = args.poll_seconds
    if args.catch_up_minutes is not None:
        updates["catch_up_minutes"] = args.catch_up_minutes
    if args.no_events is not None:
        updates["no_events"] = args.no_events
    if args.sync_bi is not None:
        updates["sync_bi"] = args.sync_bi
    if args.bank is not None:
        updates["bank"] = args.bank or None
    if args.subject_contains is not None:
        updates["subject_contains"] = args.subject_contains or None
    if not updates:
        print("nothing to update; see configure --help")
        return 1
    if not CONFIG_PATH.is_file():
        save_watch_config(WatchConfig())
    cfg = update_watch_config(**updates)
    print(f"saved {CONFIG_PATH}")
    print(json.dumps(cfg.__dict__, ensure_ascii=False, indent=2, default=str))
    return 0


def _task_query() -> str:
    if os.name != "nt":
        return "n/a"
    r = subprocess.run(
        ["schtasks", "/Query", "/TN", TASK_NAME, "/FO", "LIST"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if r.returncode != 0:
        return "not registered"
    for line in (r.stdout or "").splitlines():
        if line.strip().startswith("Status:"):
            return line.split(":", 1)[-1].strip() or "registered"
    return "registered"


def cmd_register_logon(args: argparse.Namespace) -> int:
    """可选：Windows 登录计划任务（非项目主路径）。"""
    if os.name != "nt":
        print("register-logon is Windows-only; use project register + dev.py")
        return 1
    cfg = load_watch_config(apply_env=False)
    cfg.register_on_logon = True
    if args.enable:
        cfg.enabled = True
        cfg.auto_with_dev = True
    save_watch_config(cfg)
    cfg = load_watch_config()

    cmd = watch_command(cfg)
    tr = subprocess.list2cmdline(cmd)
    create = [
        "schtasks",
        "/Create",
        "/TN",
        TASK_NAME,
        "/TR",
        tr,
        "/SC",
        "ONLOGON",
        "/RL",
        "LIMITED",
        "/F",
    ]
    print("creating task:", " ".join(create))
    r = subprocess.run(create, capture_output=True, text=True, encoding="utf-8", errors="replace")
    if r.returncode != 0:
        print(r.stdout)
        print(r.stderr, file=sys.stderr)
        return r.returncode
    print(r.stdout.strip() or "OK")
    print(f"optional logon task {TASK_NAME} registered")
    print("project auto-start still uses: python scripts/dev.py")
    return 0


def cmd_unregister_logon(_: argparse.Namespace) -> int:
    if os.name != "nt":
        print("unregister-logon is Windows-only")
        return 1
    update_watch_config(register_on_logon=False)
    r = subprocess.run(
        ["schtasks", "/Delete", "/TN", TASK_NAME, "/F"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if r.returncode != 0 and "cannot find" not in (r.stderr or "").lower():
        print(r.stderr or r.stdout, file=sys.stderr)
        return r.returncode
    print(f"logon task {TASK_NAME} removed (if existed)")
    return 0


def cmd_start(args: argparse.Namespace) -> int:
    cfg = load_watch_config()
    if not cfg.enabled and not args.force:
        print("watch disabled in config; use register / enable or --force")
        return 1
    if pid_running() and not args.force:
        print(f"already running pid={read_pid()}")
        return 0
    if pid_running() and args.force:
        cmd_stop(args)
    cmd = watch_command(cfg)
    print("starting:", subprocess.list2cmdline(cmd))
    kwargs: dict = dict(
        cwd=str(HERE.parents[1]),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    if os.name == "nt":
        kwargs["creationflags"] = (
            subprocess.CREATE_NEW_PROCESS_GROUP | subprocess.DETACHED_PROCESS  # type: ignore[attr-defined]
        )
        kwargs["close_fds"] = True
    else:
        kwargs["start_new_session"] = True
    proc = subprocess.Popen(cmd, **kwargs)
    write_pid(proc.pid)
    time.sleep(0.4)
    print(f"started pid={proc.pid}")
    return 0


def cmd_stop(_: argparse.Namespace) -> int:
    pid = read_pid()
    if not pid:
        print("no pid file")
        return 0
    if not pid_running(pid):
        clear_pid()
        print(f"stale pid {pid} cleared")
        return 0
    if os.name == "nt":
        subprocess.run(["taskkill", "/PID", str(pid), "/T", "/F"], capture_output=True)
    else:
        try:
            os.kill(pid, 15)
        except OSError as e:
            print(e, file=sys.stderr)
            return 1
    clear_pid()
    print(f"stopped pid={pid}")
    return 0


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")

    p = argparse.ArgumentParser(
        description="在 lapaquotes 项目内注册/配置 Outlook watch（非 Windows 服务）"
    )
    sub = p.add_subparsers(dest="cmd", required=True)

    sub.add_parser("status", help="显示项目配置与进程状态")
    sub.add_parser(
        "register",
        help="项目内注册：enabled+auto_with_dev（dev.py 启动时自动 watch）",
    )
    sub.add_parser("unregister", help="取消项目内自动启动")
    sub.add_parser("init", help="若无则写入默认 watch_config.yaml")
    sub.add_parser("enable", help="enabled=true")
    sub.add_parser("disable", help="enabled=false")

    c = sub.add_parser("configure", help="修改 watch_config.yaml 字段")
    c.add_argument("--enabled", type=lambda s: s.lower() in {"1", "true", "yes"}, default=None)
    c.add_argument(
        "--auto-with-dev",
        dest="auto_with_dev",
        type=lambda s: s.lower() in {"1", "true", "yes"},
        default=None,
    )
    c.add_argument(
        "--register-on-logon",
        dest="register_on_logon",
        type=lambda s: s.lower() in {"1", "true", "yes"},
        default=None,
    )
    c.add_argument("--store")
    c.add_argument("--folder")
    c.add_argument("--poll-seconds", type=float)
    c.add_argument("--catch-up-minutes", type=float)
    c.add_argument(
        "--no-events",
        type=lambda s: s.lower() in {"1", "true", "yes"},
        default=None,
    )
    c.add_argument(
        "--sync-bi",
        dest="sync_bi",
        type=lambda s: s.lower() in {"1", "true", "yes"},
        default=None,
    )
    c.add_argument("--bank", default=None)
    c.add_argument("--subject-contains", dest="subject_contains", default=None)

    rl = sub.add_parser("register-logon", help="可选：Windows 登录计划任务")
    rl.add_argument("--enable", action="store_true")
    sub.add_parser("unregister-logon", help="删除登录计划任务")

    s = sub.add_parser("start", help="立即后台启动 watch")
    s.add_argument("--force", action="store_true")
    sub.add_parser("stop", help="停止后台 watch")

    args = p.parse_args()
    handlers = {
        "status": cmd_status,
        "register": cmd_register,
        "unregister": cmd_unregister_project,
        "init": cmd_init,
        "enable": cmd_enable,
        "disable": cmd_disable,
        "configure": cmd_configure,
        "register-logon": cmd_register_logon,
        "unregister-logon": cmd_unregister_logon,
        "start": cmd_start,
        "stop": cmd_stop,
    }
    return handlers[args.cmd](args)


if __name__ == "__main__":
    raise SystemExit(main())
