"""China Construction Bank Hong Kong — deposit indications (body)."""
from __future__ import annotations

import sys
from pathlib import Path

_INGEST = Path(__file__).resolve().parent.parent
if str(_INGEST) not in sys.path:
    sys.path.insert(0, str(_INGEST))

from banks._vertical_tenor import parse_vertical_tenor

SCRIPT = "ccbhk"


def parse(**kwargs):
    return parse_vertical_tenor(source="ccbhk/body", **kwargs)


if __name__ == "__main__":
    from banks._runner import run_bank

    raise SystemExit(run_bank(SCRIPT))
