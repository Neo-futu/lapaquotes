#!/usr/bin/env bash
# Watch-mode frontend site (SSR + hydrate).
set -euo pipefail
cd "$(dirname "$0")/.."

export RUST_LOG="${RUST_LOG:-info,site=debug}"
cargo leptos watch -p site
