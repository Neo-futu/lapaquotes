#!/usr/bin/env python3
"""Clean legacy commodity views on OTC报价台账; keep one「全部报价」view."""

from __future__ import annotations

import json
import os
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def load_dotenv() -> None:
    p = ROOT / ".env"
    if not p.is_file():
        return
    for line in p.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        os.environ.setdefault(k.strip(), v.strip())


def api(method: str, url: str, body: dict | None = None) -> dict:
    data = None if body is None else json.dumps(body).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=data,
        method=method,
        headers={
            "Authorization": f"Bearer {os.environ['FEISHU_BITABLE_ACCESS_TOKEN']}",
            "Content-Type": "application/json",
        },
    )
    with urllib.request.urlopen(req, timeout=60) as resp:
        out = json.loads(resp.read().decode("utf-8"))
    if out.get("code") != 0:
        raise RuntimeError(f"{method} {url}: {out}")
    return out.get("data") or {}


def main() -> None:
    load_dotenv()
    base = os.environ.get("FEISHU_BITABLE_API_BASE", "https://base-api.feishu.cn").rstrip("/")
    app = os.environ["FEISHU_BITABLE_APP_TOKEN"]
    tables = api("GET", f"{base}/open-apis/bitable/v1/apps/{app}/tables?page_size=50")["items"]

    for t in tables:
        tid, name = t["table_id"], t["name"]
        print(f"== {name} ({tid})")
        views = api("GET", f"{base}/open-apis/bitable/v1/apps/{app}/tables/{tid}/views?page_size=50")[
            "items"
        ]
        for v in views:
            print(f"  before: {v.get('view_name')} {v.get('view_id')}")

        keep_name = "全部报价" if "台账" in name or "ledger" in name.lower() else "消息内容"
        # Prefer create a clean view first so we never delete the last one.
        existing_keep = next((v for v in views if v.get("view_name") == keep_name), None)
        if existing_keep is None:
            created = api(
                "POST",
                f"{base}/open-apis/bitable/v1/apps/{app}/tables/{tid}/views",
                {"view_name": keep_name, "view_type": "grid"},
            )
            keep_id = (created.get("view") or {}).get("view_id") or created.get("view_id")
            print(f"  created view {keep_name} {keep_id}")
            time.sleep(0.25)
            views = api(
                "GET", f"{base}/open-apis/bitable/v1/apps/{app}/tables/{tid}/views?page_size=50"
            )["items"]
            existing_keep = next(v for v in views if v.get("view_id") == keep_id or v.get("view_name") == keep_name)
        keep_id = existing_keep["view_id"]

        for v in views:
            vid, vname = v["view_id"], v.get("view_name")
            if vid == keep_id:
                continue
            api(
                "DELETE",
                f"{base}/open-apis/bitable/v1/apps/{app}/tables/{tid}/views/{vid}",
            )
            print(f"  deleted view {vname} {vid}")
            time.sleep(0.25)

        # Rename keep view if needed
        if existing_keep.get("view_name") != keep_name:
            api(
                "PATCH",
                f"{base}/open-apis/bitable/v1/apps/{app}/tables/{tid}/views/{keep_id}",
                {"view_name": keep_name},
            )
            print(f"  renamed keep → {keep_name}")

        views = api("GET", f"{base}/open-apis/bitable/v1/apps/{app}/tables/{tid}/views?page_size=50")[
            "items"
        ]
        for v in views:
            print(f"  after: {v.get('view_name')} {v.get('view_id')}")


if __name__ == "__main__":
    main()
