"""Load per-bank scripts from banks/ and match mail → institution config."""
from __future__ import annotations

import importlib
import sys
from pathlib import Path
from typing import Any, Callable, Optional

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

ParseFn = Callable[..., Any]


def load_cfg() -> dict:
    import yaml

    with (HERE / "institutions.yaml").open(encoding="utf-8") as f:
        return yaml.safe_load(f)


def match_institution(subject: str, sender: str, cfg: dict) -> Optional[dict]:
    sender_l = (sender or "").lower()
    subj = subject or ""
    scored: list[tuple[int, dict]] = []
    for inst in cfg.get("institutions") or []:
        score = 0
        for s in inst.get("subject_contains") or []:
            if s.lower() in subj.lower():
                score += 10
        for e in inst.get("emails") or []:
            if e.lower() == sender_l:
                score += 5
        for d in inst.get("domains") or []:
            if sender_l.endswith("@" + d.lower()) or sender_l.endswith(d.lower()):
                score += 2
        if score:
            scored.append((score, inst))
    if not scored:
        return None
    scored.sort(key=lambda x: x[0], reverse=True)
    return scored[0][1]


def script_module_name(inst: dict) -> str:
    """institutions.yaml: script: abchk  →  banks.abchk"""
    name = inst.get("script") or inst.get("parser") or inst.get("key")
    if not name:
        raise ValueError(f"institution missing script/key: {inst}")
    name = str(name).removesuffix(".py")
    if name.startswith("banks."):
        return name
    if name.startswith("banks/"):
        return name.replace("/", ".")
    return f"banks.{name}"


def load_parse_fn(inst: dict) -> ParseFn:
    mod_name = script_module_name(inst)
    mod = importlib.import_module(mod_name)
    fn = getattr(mod, "parse", None)
    if not callable(fn):
        raise AttributeError(f"{mod_name} must define parse(...)")
    return fn


def list_bank_scripts() -> list[str]:
    bank_dir = HERE / "banks"
    return sorted(
        p.stem
        for p in bank_dir.glob("*.py")
        if p.stem not in {"__init__", "_base", "_vertical_tenor"}
        and not p.stem.startswith("_")
    )
