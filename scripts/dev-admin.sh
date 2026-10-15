#!/usr/bin/env bash
# Watch-mode admin dashboard (SSR + hydrate).
set -euo pipefail
cd "$(dirname "$0")/.."

export RUST_LOG="${RUST_LOG:-info,admin=debug}"
cargo leptos watch -p admin
