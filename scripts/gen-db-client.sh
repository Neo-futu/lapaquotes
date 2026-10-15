#!/usr/bin/env bash
# Generate Cornucopia typed Postgres client from schema + queries.
set -euo pipefail
cd "$(dirname "$0")/.."

command -v cornucopia >/dev/null 2>&1 || {
  echo "cornucopia not found. Install: cargo install cornucopia"
  echo "Requires Docker or Podman. See: https://cornucopia-rs.github.io/cornucopia/"
  exit 1
}

cornucopia schema ./schema.sql
echo "Cornucopia client generated under ./cornucopia"
