"""
MSCI Security Crowding Model — open-data approximation.

Source hierarchy / weights: MSCI Security Crowding Model Factsheet
https://www.msci.com/documents/1296102/25918384/MSCI-Security-Crowding-Model-Factsheet.pdf

Integrated Crowding
├── Time-Series Crowding (50%)
│   ├── Value TS (40%) = 0.5*EY + 0.5*BP   [expensive => crowded => use -EY, -BP]
│   ├── Short Interest TS (30%)
│   ├── Liquidity / Turnover TS (10%)
│   ├── Residual Volatility TS (10%)
│   └── Momentum TS (10%)
└── Cross-Sectional Crowding (50%)
    ├── Value XS (40%)
    ├── Short Interest XS (30%)
    ├── Liquidity XS (10%)
    ├── Residual Volatility XS (10%)
    └── Momentum XS (10%)

Each leaf is a z-score. MSCI commentary: scores > ~2 very elevated (~2% of names).

This is NOT the commercial MSCI feed. Public yfinance data approximates inputs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
from typing import Dict, Iterable, List, Optional, Tuple
from urllib.request import Request, urlopen

import numpy as np
import pandas as pd
import yfinance as yf

W_TS = 0.50
W_XS = 0.50

W_VALUE = 0.40
W_SHORT = 0.30
W_LIQ = 0.10
W_RVOL = 0.10
W_MOM = 0.10

W_EY = 0.50
W_BP = 0.50


@dataclass
class CrowdingResult:
    ticker: str
    name: str
    market_cap: float
    price: float
    shares_outstanding: float
    as_of: str
    integrated: float
    time_series: float
    cross_section: float
    components: Dict[str, float] = field(default_factory=dict)
    label: str = ""


def _http_get(url: str, timeout: int = 30) -> str:
    req = Request(
        url,
        headers={
            "User-Agent": (
                "Mozilla/5.0 (compatible; lapaquotes-crowding/1.0; "
                "+https://github.com/)"
            )
        },
    )
    with urlopen(req, timeout=timeout) as resp:
        return resp.read().decode("utf-8", errors="replace")


_CURRENCY_NOISE = {
    "EN", "EUR", "INR", "GBP", "CAD", "AUD", "NZD", "HKD", "SGD", "USD",
    "JPY", "CNY", "KRW", "CHF", "BRL", "MXN", "ZAR", "SEK", "NOK", "DKK",
    "USA", "ETF", "ADR", "CEO", "IPO", "X",
}


def fetch_us_mcap_ranking(limit: int = 40) -> List[str]:
    """Top US tickers by market cap from companiesmarketcap.com (live ranking)."""
    import re

    html = _http_get(
        "https://companiesmarketcap.com/usa/largest-companies-in-the-usa-by-market-cap/"
    )
    tickers: List[str] = []
    for m in re.finditer(
        r'href="/[^"]+/marketcap/"[^>]*>.*?</a>\s*([A-Z][A-Z0-9.-]{0,6})\b',
        html,
        flags=re.S,
    ):
        t = m.group(1).strip()
        if t in _CURRENCY_NOISE:
            continue
        if t not in tickers:
            tickers.append(t)
        if len(tickers) >= limit:
            break
    if len(tickers) < 10:
        for m in re.finditer(r">([A-Z]{1,5}(?:-[A-Z])?)</", html):
            t = m.group(1)
            if t in _CURRENCY_NOISE:
                continue
            if t not in tickers:
                tickers.append(t)
            if len(tickers) >= limit:
                break
    return tickers


def realized_market_cap(ticker: str) -> Tuple[float, float, float, str, str, dict]:
    """
    Realized mcap = last close × impliedSharesOutstanding
    (falls back to sharesOutstanding / Yahoo marketCap).

    Prefer implied shares so dual-class names (GOOGL/BRK/SPCX etc.)
    match total equity value, not a single share class.
    Returns (mcap, price, shares, as_of_date, name, info).
    """
    tk = yf.Ticker(ticker)
    info = tk.info or {}
    shares = info.get("impliedSharesOutstanding") or info.get("sharesOutstanding")
    name = str(info.get("shortName") or info.get("longName") or ticker)

    hist = tk.history(period="10d", auto_adjust=True)
    if hist is None or hist.empty:
        price = info.get("regularMarketPrice") or info.get("currentPrice")
        as_of = datetime.utcnow().strftime("%Y-%m-%d") + " (info)"
        if price and shares:
            return float(price) * float(shares), float(price), float(shares), as_of, name, info
        mcap = info.get("marketCap")
        return (
            float(mcap) if mcap else np.nan,
            float(price) if price else np.nan,
            float(shares) if shares else np.nan,
            as_of,
            name,
            info,
        )

    last_ts = hist.index[-1]
    price = float(hist["Close"].iloc[-1])
    as_of = (
        pd.Timestamp(last_ts).tz_convert(None).strftime("%Y-%m-%d")
        if getattr(last_ts, "tzinfo", None)
        else pd.Timestamp(last_ts).strftime("%Y-%m-%d")
    )
    if shares and float(shares) > 0:
        mcap = price * float(shares)
    else:
        mcap = float(info.get("marketCap") or np.nan)
        shares = float(shares) if shares else np.nan
    # Sanity: if Yahoo marketCap is materially larger, keep it (rare stale shares)
    y_mcap = info.get("marketCap")
    if y_mcap and np.isfinite(mcap) and float(y_mcap) > 1.15 * mcap:
        mcap = float(y_mcap)
    return mcap, price, float(shares) if shares else np.nan, as_of, name, info


def zscore(series: pd.Series) -> pd.Series:
    s = series.astype(float)
    mu = s.mean(skipna=True)
    sd = s.std(skipna=True, ddof=0)
    if sd is None or not np.isfinite(sd) or sd < 1e-12:
        return pd.Series(np.zeros(len(s)), index=s.index)
    return (s - mu) / sd


def last_z_vs_history(x: pd.Series, min_obs: int = 60) -> float:
    s = x.dropna().astype(float)
    if len(s) < min_obs:
        return np.nan
    # if nearly constant, TS z ~ 0
    if float(s.std(ddof=0)) < 1e-12:
        return 0.0
    return float(zscore(s).iloc[-1])


def cross_z(values: Dict[str, float]) -> Dict[str, float]:
    arr = pd.Series(values, dtype=float)
    zs = zscore(arr)
    return {k: float(zs[k]) if np.isfinite(zs[k]) else np.nan for k in values}


def short_interest_pct(info: dict) -> float:
    for key in ("shortPercentOfFloat", "shortPercentOfSharesOutStanding"):
        v = info.get(key)
        if v is not None and np.isfinite(v):
            return float(v)
    ss = info.get("sharesShort")
    fl = info.get("floatShares") or info.get("sharesOutstanding")
    if ss and fl and fl > 0:
        return float(ss) / float(fl)
    return np.nan


def fetch_top_us_by_mcap(
    n: int = 20,
    candidate_pool: int = 50,
) -> Tuple[List[str], Dict[str, dict]]:
    """
    Rank US names by realized market cap (last close × shares outstanding).
    Universe seed from live companiesmarketcap.com US ranking.
    Returns (tickers, meta_by_ticker) where meta has mcap/price/shares/as_of/name/info.
    """
    seeds_fallback = [
        "NVDA", "AAPL", "GOOGL", "GOOG", "MSFT", "AMZN", "SPCX", "META", "AVGO",
        "TSLA", "MU", "BRK-B", "LLY", "AMD", "JPM", "WMT", "V", "XOM", "JNJ",
        "INTC", "MA", "ABBV", "PLTR", "CSCO", "ORCL", "COST", "AMAT", "LRCX",
        "CVX", "BAC", "CAT", "KO", "DELL", "MRK", "PG", "UNH", "PANW", "GE",
        "PM", "MS", "NFLX", "HD", "ANET", "CRWD", "GEV", "TXN", "GS", "KLAC",
        "MRVL", "TMO",
    ]
    try:
        seeds = fetch_us_mcap_ranking(limit=max(candidate_pool, n))
        if len(seeds) < n:
            seeds = list(dict.fromkeys(seeds + seeds_fallback))
    except Exception:
        seeds = seeds_fallback

    # Prefer GOOGL over GOOG for Alphabet share class consolidation
    seeds = [t for t in seeds if t != "GOOG"] + (
        ["GOOGL"] if "GOOG" in seeds or "GOOGL" in seeds else []
    )
    seeds = list(dict.fromkeys(seeds))[: max(candidate_pool, n + 5)]

    rows = []
    meta: Dict[str, dict] = {}
    for t in seeds:
        try:
            mcap, price, shares, as_of, name, info = realized_market_cap(t)
            if not np.isfinite(mcap) or mcap <= 0:
                continue
            meta[t] = {
                "market_cap": mcap,
                "price": price,
                "shares_outstanding": shares,
                "as_of": as_of,
                "name": name,
                "info": info,
            }
            rows.append((t, mcap))
        except Exception:
            continue

    rows.sort(key=lambda x: x[1], reverse=True)
    out = [t for t, _ in rows[:n]]
    # keep meta for the full ranked candidate set (needed for peer universe)
    return out, {t: meta[t] for t, _ in rows}


def download_panel(
    tickers: Iterable[str],
    years: int = 3,
    benchmark: str = "SPY",
) -> Tuple[pd.DataFrame, pd.DataFrame, Dict[str, dict]]:
    tickers = list(dict.fromkeys(list(tickers) + [benchmark]))
    px = yf.download(
        tickers,
        period=f"{years}y",
        interval="1d",
        auto_adjust=True,
        progress=False,
        threads=True,
        group_by="column",
    )
    if isinstance(px.columns, pd.MultiIndex):
        close = px["Close"].copy()
        volume = px["Volume"].copy()
    else:
        close = px[["Close"]].copy()
        close.columns = [tickers[0]]
        volume = px[["Volume"]].copy()
        volume.columns = [tickers[0]]

    infos: Dict[str, dict] = {}
    for t in tickers:
        if t == benchmark:
            continue
        try:
            infos[t] = yf.Ticker(t).info or {}
        except Exception:
            infos[t] = {}
    return close, volume, infos


def residual_vol_series(
    stock_ret: pd.Series, mkt_ret: pd.Series, window: int = 60
) -> pd.Series:
    df = pd.concat([stock_ret, mkt_ret], axis=1).dropna()
    df.columns = ["r", "m"]
    out = []
    idx = []
    for i in range(window, len(df) + 1):
        sl = df.iloc[i - window : i]
        x = sl["m"].to_numpy(dtype=float)
        y = sl["r"].to_numpy(dtype=float)
        if np.std(x) < 1e-12:
            resid_std = float(np.std(y))
        else:
            b = float(np.cov(y, x, ddof=0)[0, 1] / np.var(x, ddof=0))
            a = float(y.mean() - b * x.mean())
            resid = y - (a + b * x)
            resid_std = float(np.std(resid, ddof=0))
        out.append(resid_std * np.sqrt(252))
        idx.append(df.index[i - 1])
    return pd.Series(out, index=idx)


def momentum_12_1(close: pd.Series) -> pd.Series:
    r_12 = close / close.shift(252) - 1.0
    r_1 = close / close.shift(21) - 1.0
    return r_12 - r_1


def turnover_series(volume: pd.Series, shares: Optional[float]) -> pd.Series:
    if shares is None or not np.isfinite(shares) or shares <= 0:
        return volume.rolling(20).mean()
    return (volume / float(shares)).rolling(20).mean()


def build_raw_features(
    close: pd.DataFrame,
    volume: pd.DataFrame,
    infos: Dict[str, dict],
    benchmark: str = "SPY",
) -> Dict[str, Dict[str, pd.Series]]:
    mkt_ret = close[benchmark].pct_change()
    features: Dict[str, Dict[str, pd.Series]] = {}

    for t, info in infos.items():
        if t not in close.columns:
            continue
        px = close[t].dropna()
        vol = volume[t].reindex(px.index)
        ret = px.pct_change()
        shares = info.get("sharesOutstanding") or info.get("floatShares")

        pe = info.get("trailingPE") or info.get("forwardPE")
        pb = info.get("priceToBook")
        ey_series = pd.Series(np.nan, index=px.index)
        bp_series = pd.Series(np.nan, index=px.index)
        if pe and np.isfinite(pe) and pe > 0:
            eps = float(px.iloc[-1]) / float(pe)
            ey_series = eps / px.replace(0, np.nan)
        if pb and np.isfinite(pb) and pb > 0:
            book = float(px.iloc[-1]) / float(pb)
            bp_series = book / px.replace(0, np.nan)

        si = short_interest_pct(info)
        si_series = pd.Series(si, index=px.index)

        features[t] = {
            "value_expensive": -(W_EY * ey_series + W_BP * bp_series),
            "short_interest": si_series,
            "liquidity": turnover_series(vol, float(shares) if shares else None),
            "resid_vol": residual_vol_series(ret, mkt_ret.reindex(ret.index)),
            "momentum": momentum_12_1(px),
        }
    return features


def combine_level3(parts: Dict[str, float]) -> float:
    w = {
        "value_expensive": W_VALUE,
        "short_interest": W_SHORT,
        "liquidity": W_LIQ,
        "resid_vol": W_RVOL,
        "momentum": W_MOM,
    }
    num = 0.0
    den = 0.0
    for k, wt in w.items():
        v = parts.get(k, np.nan)
        if v is not None and np.isfinite(v):
            num += wt * float(v)
            den += wt
    if den < 1e-12:
        return np.nan
    return num / den


def crowding_label(score: float) -> str:
    if not np.isfinite(score):
        return "n/a"
    if score >= 2.0:
        return "VERY CROWDED"
    if score >= 1.0:
        return "CROWDED"
    if score >= 0.5:
        return "ELEVATED"
    if score > -0.5:
        return "NEUTRAL"
    if score > -1.0:
        return "OUT OF FAVOR"
    return "UNCROWDED"


def compute_crowding(
    tickers: List[str],
    years: int = 3,
    benchmark: str = "SPY",
    mcap_meta: Optional[Dict[str, dict]] = None,
) -> List[CrowdingResult]:
    close, volume, infos = download_panel(tickers, years=years, benchmark=benchmark)
    feats = build_raw_features(close, volume, infos, benchmark=benchmark)

    ts_leaf: Dict[str, Dict[str, float]] = {}
    for t, f in feats.items():
        ts_leaf[t] = {
            k: last_z_vs_history(s, min_obs=40 if k == "short_interest" else 60)
            for k, s in f.items()
        }

    latest_raw: Dict[str, Dict[str, float]] = {k: {} for k in feats}
    for t, f in feats.items():
        for k, s in f.items():
            s2 = s.dropna()
            latest_raw[t][k] = float(s2.iloc[-1]) if len(s2) else np.nan

    xs_leaf: Dict[str, Dict[str, float]] = {t: {} for t in feats}
    for metric in [
        "value_expensive",
        "short_interest",
        "liquidity",
        "resid_vol",
        "momentum",
    ]:
        cross = {t: latest_raw[t].get(metric, np.nan) for t in feats}
        zmap = cross_z(cross)
        for t in feats:
            xs_leaf[t][metric] = zmap[t]

    results: List[CrowdingResult] = []
    for t in tickers:
        if t not in feats:
            continue
        info = infos.get(t, {})
        meta = (mcap_meta or {}).get(t)
        if meta is None:
            mcap, price, shares, as_of, name, _ = realized_market_cap(t)
        else:
            mcap = float(meta["market_cap"])
            price = float(meta["price"])
            shares = float(meta["shares_outstanding"]) if meta.get("shares_outstanding") is not None else np.nan
            as_of = str(meta["as_of"])
            name = str(meta.get("name") or t)
        # Prefer panel last close for as_of / price consistency with feature series
        if t in close.columns:
            s = close[t].dropna()
            if len(s):
                price = float(s.iloc[-1])
                ts = s.index[-1]
                as_of = (
                    pd.Timestamp(ts).tz_convert(None).strftime("%Y-%m-%d")
                    if getattr(ts, "tzinfo", None)
                    else pd.Timestamp(ts).strftime("%Y-%m-%d")
                )
                sh = info.get("impliedSharesOutstanding") or info.get("sharesOutstanding")
                if sh and float(sh) > 0:
                    shares = float(sh)
                    mcap = price * shares
                    y_mcap = info.get("marketCap")
                    if y_mcap and float(y_mcap) > 1.15 * mcap:
                        mcap = float(y_mcap)

        ts_score = combine_level3(ts_leaf[t])
        xs_score = combine_level3(xs_leaf[t])
        parts = []
        wts = []
        if np.isfinite(ts_score):
            parts.append(W_TS * ts_score)
            wts.append(W_TS)
        if np.isfinite(xs_score):
            parts.append(W_XS * xs_score)
            wts.append(W_XS)
        integrated = float(sum(parts) / sum(wts)) if wts else np.nan

        comps = {}
        for k in ts_leaf[t]:
            comps[f"ts_{k}"] = ts_leaf[t][k]
            comps[f"xs_{k}"] = xs_leaf[t][k]

        results.append(
            CrowdingResult(
                ticker=t,
                name=name or str(info.get("shortName") or info.get("longName") or t),
                market_cap=float(mcap) if np.isfinite(mcap) else np.nan,
                price=float(price) if np.isfinite(price) else np.nan,
                shares_outstanding=float(shares) if np.isfinite(shares) else np.nan,
                as_of=as_of,
                integrated=integrated,
                time_series=float(ts_score) if np.isfinite(ts_score) else np.nan,
                cross_section=float(xs_score) if np.isfinite(xs_score) else np.nan,
                components=comps,
                label=crowding_label(integrated),
            )
        )

    results.sort(
        key=lambda r: r.market_cap if np.isfinite(r.market_cap) else -1.0,
        reverse=True,
    )
    return results


def results_to_frame(results: List[CrowdingResult]) -> pd.DataFrame:
    rows = []
    for r in results:
        row = {
            "ticker": r.ticker,
            "name": r.name,
            "as_of": r.as_of,
            "price": r.price,
            "shares_outstanding": r.shares_outstanding,
            "market_cap_b": r.market_cap / 1e9 if np.isfinite(r.market_cap) else np.nan,
            "integrated": r.integrated,
            "time_series": r.time_series,
            "cross_section": r.cross_section,
            "label": r.label,
        }
        row.update(r.components)
        rows.append(row)
    return pd.DataFrame(rows)
