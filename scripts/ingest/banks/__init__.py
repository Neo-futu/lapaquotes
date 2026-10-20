"""Per-institution Outlook deposit quote parsers.

Add a new bank:
  1. Create banks/<key>.py with parse(...) and SCRIPT = "<key>"
  2. Register in institutions.yaml (emails / subject_contains / script)
  3. Optional: python -m banks.<key> --date YYYY-MM-DD --dry-run
"""
