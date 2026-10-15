#!/usr/bin/env bash
# Build Slate.js rich text editor bundle used by admin.
set -euo pipefail
cd "$(dirname "$0")/../slate"

if [[ ! -d node_modules ]]; then
  npm install
fi

npm run build
echo "Slate editor built."
