#!/usr/bin/env python3
"""Compute MSCI-style crowding scores for top US stocks by realized market cap."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import pandas as pd

sys.path.insert(0, str(Path(__file__).resolve().parent))

from msci_crowding import (  # noqa: E402
    compute_crowding,
    fetch_top_us_by_mcap,
    results_to_frame,
)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--top", type=int, default=20, help="number of mega-caps to score")
    ap.add_argument(
        "--universe",
        type=int,
        default=40,
        help="broader peer set size for cross-sectional z-scores",
    )
    ap.add_argument("--years", type=int, default=3, help="history length for TS z-scores")
    ap.add_argument(
        "--out",
        type=Path,
        default=Path(__file__).resolve().parent / "top20_crowding.csv",
    )
    args = ap.parse_args()

    print(
        "Ranking US names by realized market cap = "
        "last close × impliedSharesOutstanding (dual-class aware)..."
    )
    peer, peer_meta = fetch_top_us_by_mcap(n=max(args.top, args.universe))
    focus = peer[: args.top]
    as_ofs = sorted({peer_meta[t]["as_of"] for t in focus if t in peer_meta})
    print("Price as-of (focus):", ", ".join(as_ofs) if as_ofs else "n/a")
    print("Focus (top by realized mcap):", ", ".join(focus))
    print("Peer universe for XS z-scores:", len(peer), "names")

    print("Downloading prices / fundamentals and computing crowding...")
    # Enrich meta for full peer set (focus already has meta; fill rest)
    mcap_meta = dict(peer_meta)
    all_results = compute_crowding(peer, years=args.years, mcap_meta=mcap_meta)
    focus_set = set(focus)
    focus_results = [r for r in all_results if r.ticker in focus_set]
    order = {t: i for i, t in enumerate(focus)}
    focus_results.sort(key=lambda r: order.get(r.ticker, 999))

    df = results_to_frame(focus_results)
    cols = [
        "ticker",
        "name",
        "as_of",
        "price",
        "market_cap_b",
        "integrated",
        "time_series",
        "cross_section",
        "label",
    ]
    view = df[cols].copy()
    view["price"] = view["price"].map(lambda x: f"{x:,.2f}" if pd.notna(x) else "")
    view["market_cap_b"] = view["market_cap_b"].map(
        lambda x: f"{x:,.0f}" if pd.notna(x) else ""
    )
    for c in ("integrated", "time_series", "cross_section"):
        view[c] = view[c].map(lambda x: f"{x:+.2f}" if pd.notna(x) else "n/a")

    print("\n=== MSCI-style Security Crowding (open-data approx) ===")
    print("Market cap = last close × implied shares (yfinance); price as-of shown.")
    print("Thresholds: >=1.0 Crowded, >=2.0 Very Crowded (MSCI commentary).\n")
    try:
        from tabulate import tabulate

        print(tabulate(view, headers="keys", tablefmt="github", showindex=False))
    except Exception:
        print(view.to_string(index=False))

    df.to_csv(args.out, index=False)
    print(f"\nFull component detail written to: {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
