#!/usr/bin/env python3
"""Launch the Lapa site (127.0.0.1:4141) and admin (127.0.0.1:4143) dev servers.

Windows-oriented: prepends %USERPROFILE%\\.cargo\\bin to PATH (same as
`$env:Path = "$env:USERPROFILE\\.cargo\\bin;" + $env:Path` in PowerShell),
sets RUST_LOG, then runs `cargo leptos watch` for both packages concurrently.

Usage:
    python scripts/dev.py              # watch mode (hot reload), site + admin
    python scripts/dev.py --serve      # serve mode (no hot reload)
    python scripts/dev.py --only site  # just one of: site | admin
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

# (package, port, ANSI color, label)
TARGETS = {
    "site": ("site", "4141", "\033[36m", "SITE  "),   # cyan
    "admin": ("admin", "4143", "\033[35m", "ADMIN "),  # magenta
}

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
    print(f"{color}[{label}]{reset} started `{' '.join(cmd)}`  ->  http://127.0.0.1:{port}")
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
    args = parser.parse_args()

    if not CARGO_BIN.is_dir():
        print(f"warning: {CARGO_BIN} not found; relying on existing PATH")

    mode = "serve" if args.serve else "watch"
    env = build_env(args.log)
    names = [args.only] if args.only else ["site", "admin"]

    procs: dict[str, subprocess.Popen] = {}
    exit_code = 0
    try:
        for name in names:
            procs[name] = start_target(name, mode, env)

        print("\nCtrl+C to stop both servers.\n")
        while True:
            time.sleep(1)
            for name, proc in procs.items():
                ret = proc.poll()
                if ret is not None:
                    print(f"\n[{TARGETS[name][3]}] exited with code {ret}")
                    # A crash of one target should not keep the other hanging.
                    raise KeyboardInterrupt
    except KeyboardInterrupt:
        print("\nshutting down...")
    finally:
        for name, proc in procs.items():
            if proc.poll() is None:
                try:
                    if os.name == "nt":
                        proc.send_signal(signal.CTRL_BREAK_EVENT)
                    else:
                        proc.terminate()
                except OSError:
                    pass
        deadline = time.time() + 10
        for name, proc in procs.items():
            try:
                proc.wait(timeout=max(0.5, deadline - time.time()))
            except subprocess.TimeoutExpired:
                proc.kill()
                print(f"[{TARGETS[name][3]}] force-killed")
        print("bye")

    return exit_code


if __name__ == "__main__":
    sys.exit(main())
