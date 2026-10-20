#!/usr/bin/env python3
"""Launch the Lapa site (:4141) and admin (:4143) dev servers.

Binds 0.0.0.0 by default (see Cargo.toml site-addr) so LAN IPs work, e.g.
http://172.21.55.8:4141/ as well as http://127.0.0.1:4141/.

Windows-oriented: prepends %USERPROFILE%\\.cargo\\bin to PATH (same as
`$env:Path = "$env:USERPROFILE\\.cargo\\bin;" + $env:Path` in PowerShell),
sets RUST_LOG, then runs `cargo leptos watch` for both packages concurrently.

When Outlook watch is enabled (`scripts/ingest/watch_config.yaml` /
`WATCH_OUTLOOK=1` + `auto_with_dev`), also starts `watch_outlook.py`.

Usage:
    python scripts/dev.py              # watch mode + Outlook watch（若已配置）
    python scripts/dev.py --serve      # serve mode (no hot reload)
    python scripts/dev.py --only site  # just one of: site | admin
    python scripts/dev.py --no-outlook-watch
    python scripts/dev.py --with-outlook-watch
    python scripts/dev.py --log info,site=debug
"""

from __future__ import annotations

import argparse
import os
import signal
import subprocess
import sys
import threading
import time
from pathlib import Path

# --- constants -------------------------------------------------------------

REPO_ROOT = Path(__file__).resolve().parent.parent
CARGO_BIN = Path(os.environ.get("USERPROFILE", str(Path.home()))) / ".cargo" / "bin"
INGEST_DIR = REPO_ROOT / "scripts" / "ingest"

# (package, port, ANSI color, label)
TARGETS = {
    "site": ("site", "4141", "\033[36m", "SITE  "),   # cyan
    "admin": ("admin", "4143", "\033[35m", "ADMIN "),  # magenta
}
WATCH_COLOR = "\033[33m"  # yellow
WATCH_LABEL = "WATCH "

DEFAULT_RUST_LOG = "info"


def build_env(rust_log: str) -> dict:
    """cargo bin first in PATH + RUST_LOG, like the PowerShell snippet."""
    env = os.environ.copy()
    env["PATH"] = str(CARGO_BIN) + os.pathsep + env.get("PATH", "")
    env["RUST_LOG"] = rust_log
    return env


def stream_output(proc: subprocess.Popen, label: str, color: str) -> None:
    """Tag every output line with the process label (keeps colors from cargo)."""
    reset = "\033[0m"
    stream = proc.stdout
    assert stream is not None
    try:
        for raw in iter(stream.readline, b""):
            line = raw.decode(errors="replace").rstrip("\r\n")
            print(f"{color}[{label}]{reset} {line}", flush=True)
    except (ValueError, OSError):
        pass  # stream closed during shutdown


def start_target(name: str, mode: str, env: dict) -> subprocess.Popen:
    package, port, color, label = TARGETS[name]
    reset = "\033[0m"
    cmd = ["cargo", "leptos", mode, "-p", package]
    kwargs: dict = dict(
        cwd=REPO_ROOT,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if os.name == "nt":
        kwargs["creationflags"] = subprocess.CREATE_NEW_PROCESS_GROUP
    proc = subprocess.Popen(cmd, **kwargs)
    t = threading.Thread(target=stream_output, args=(proc, label, color), daemon=True)
    t.start()
    print(
        f"{color}[{label}]{reset} started `{' '.join(cmd)}`  ->  "
        f"http://127.0.0.1:{port}  (LAN: http://<本机IP>:{port})"
    )
    return proc


def _load_watch_helpers():
    if str(INGEST_DIR) not in sys.path:
        sys.path.insert(0, str(INGEST_DIR))
    from watch_config import (  # type: ignore
        clear_pid,
        load_watch_config,
        pid_running,
        read_pid,
        watch_command,
        write_pid,
    )

    return load_watch_config, watch_command, pid_running, read_pid, write_pid, clear_pid


def start_outlook_watch(env: dict) -> subprocess.Popen | None:
    """Start watch_outlook if config allows; return Popen or None."""
    reset = "\033[0m"
    try:
        load_watch_config, watch_command, pid_running, read_pid, write_pid, _clear = (
            _load_watch_helpers()
        )
    except Exception as e:
        print(f"{WATCH_COLOR}[{WATCH_LABEL}]{reset} config load failed: {e}")
        return None

    cfg = load_watch_config()
    if not cfg.enabled or not cfg.auto_with_dev:
        print(
            f"{WATCH_COLOR}[{WATCH_LABEL}]{reset} skipped "
            f"(enabled={cfg.enabled} auto_with_dev={cfg.auto_with_dev})"
        )
        return None

    if pid_running():
        print(
            f"{WATCH_COLOR}[{WATCH_LABEL}]{reset} already running pid={read_pid()} "
            f"(leave as-is)"
        )
        return None

    cmd = watch_command(cfg)
    kwargs: dict = dict(
        cwd=REPO_ROOT,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if os.name == "nt":
        kwargs["creationflags"] = subprocess.CREATE_NEW_PROCESS_GROUP
    proc = subprocess.Popen(cmd, **kwargs)
    write_pid(proc.pid)
    t = threading.Thread(
        target=stream_output, args=(proc, WATCH_LABEL, WATCH_COLOR), daemon=True
    )
    t.start()
    print(f"{WATCH_COLOR}[{WATCH_LABEL}]{reset} started `{' '.join(cmd)}`  pid={proc.pid}")
    return proc


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--serve",
        action="store_true",
        help="serve instead of watch (no hot reload)",
    )
    parser.add_argument(
        "--only",
        choices=sorted(TARGETS),
        help="start only site or admin",
    )
    parser.add_argument(
        "--log",
        default=DEFAULT_RUST_LOG,
        help='RUST_LOG value (default "info")',
    )
    g = parser.add_mutually_exclusive_group()
    g.add_argument(
        "--with-outlook-watch",
        action="store_true",
        help="强制启动 Outlook watch（忽略 auto_with_dev=false）",
    )
    g.add_argument(
        "--no-outlook-watch",
        action="store_true",
        help="不启动 Outlook watch",
    )
    args = parser.parse_args()

    if not CARGO_BIN.is_dir():
        print(f"warning: {CARGO_BIN} not found; relying on existing PATH")

    mode = "serve" if args.serve else "watch"
    env = build_env(args.log)
    names = [args.only] if args.only else ["site", "admin"]

    procs: dict[str, subprocess.Popen] = {}
    watch_proc: subprocess.Popen | None = None
    exit_code = 0
    try:
        for name in names:
            procs[name] = start_target(name, mode, env)

        if args.no_outlook_watch:
            print(f"{WATCH_COLOR}[{WATCH_LABEL}]\033[0m skipped (--no-outlook-watch)")
        elif args.with_outlook_watch:
            # Temporarily force enable flags via env for this process tree
            env = env.copy()
            env["WATCH_OUTLOOK"] = "1"
            env["WATCH_OUTLOOK_AUTO_DEV"] = "1"
            watch_proc = start_outlook_watch(env)
        else:
            watch_proc = start_outlook_watch(env)

        print("\nCtrl+C to stop servers (+ Outlook watch if started).\n")
        while True:
            time.sleep(1)
            for name, proc in procs.items():
                ret = proc.poll()
                if ret is not None:
                    print(f"\n[{TARGETS[name][3]}] exited with code {ret}")
                    raise KeyboardInterrupt
            if watch_proc is not None:
                ret = watch_proc.poll()
                if ret is not None:
                    print(f"\n[{WATCH_LABEL}] exited with code {ret}")
                    watch_proc = None
    except KeyboardInterrupt:
        print("\nshutting down...")
    finally:
        to_stop: list[tuple[str, subprocess.Popen]] = list(procs.items())
        if watch_proc is not None and watch_proc.poll() is None:
            to_stop.append(("outlook_watch", watch_proc))
        for name, proc in to_stop:
            if proc.poll() is None:
                try:
                    if os.name == "nt":
                        proc.send_signal(signal.CTRL_BREAK_EVENT)
                    else:
                        proc.terminate()
                except OSError:
                    pass
        deadline = time.time() + 10
        for name, proc in to_stop:
            label = TARGETS[name][3] if name in TARGETS else WATCH_LABEL
            try:
                proc.wait(timeout=max(0.5, deadline - time.time()))
            except subprocess.TimeoutExpired:
                proc.kill()
                print(f"[{label}] force-killed")
        try:
            _, _, _, _, _, clear_pid = _load_watch_helpers()
            clear_pid()
        except Exception:
            pass
        print("bye")

    return exit_code


if __name__ == "__main__":
    sys.exit(main())
