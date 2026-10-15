#!/usr/bin/env bash
# Create PostgreSQL database `lapa` and apply project schema.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

PGHOST="${PGHOST:-127.0.0.1}"
PGPORT="${PGPORT:-5432}"
PGUSER="${PGUSER:-postgres}"
PGDATABASE="${PGDATABASE:-lapa}"

find_psql() {
  if command -v psql >/dev/null 2>&1; then
    command -v psql
    return
  fi
  for c in \
    "/d/Program Files/PostgreSQL/18/bin/psql.exe" \
    "/c/Program Files/PostgreSQL/18/bin/psql.exe" \
    "D:/Program Files/PostgreSQL/18/bin/psql.exe" \
    "C:/Program Files/PostgreSQL/18/bin/psql.exe"
  do
    if [[ -x "$c" ]]; then
      echo "$c"
      return
    fi
  done
  echo "psql not found" >&2
  exit 1
}

PSQL="$(find_psql)"
echo "using psql: $PSQL"

if [[ -z "${PGPASSWORD:-}" ]]; then
  read -r -s -p "PostgreSQL password for ${PGUSER}: " PGPASSWORD
  echo
  export PGPASSWORD
fi

export PGHOST PGPORT PGUSER

echo ">> create database ${PGDATABASE} if missing..."
"$PSQL" -v ON_ERROR_STOP=1 -d postgres -f "$ROOT/scripts/db/01_create_database.sql"

echo ">> apply schema.sql..."
"$PSQL" -v ON_ERROR_STOP=1 -d "$PGDATABASE" -f "$ROOT/schema.sql"

echo ">> tables:"
"$PSQL" -v ON_ERROR_STOP=1 -d "$PGDATABASE" -c \
  "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename;"

ENV_FILE="$ROOT/.env"
PG_URL="postgresql://${PGUSER}:${PGPASSWORD}@${PGHOST}:${PGPORT}/${PGDATABASE}"
PG_LINE="PG__URL=${PG_URL}"

if [[ -f "$ENV_FILE" ]]; then
  if grep -q '^PG__URL=' "$ENV_FILE"; then
    # portable-ish replace
    tmp="$(mktemp)"
    sed "s|^PG__URL=.*|${PG_LINE}|" "$ENV_FILE" >"$tmp"
    mv "$tmp" "$ENV_FILE"
    echo "updated PG__URL in .env"
  else
    printf '\n%s\n' "$PG_LINE" >>"$ENV_FILE"
    echo "appended PG__URL to .env"
  fi
else
  printf '%s\n' "$PG_LINE" >"$ENV_FILE"
  echo "created .env with PG__URL"
fi

echo
echo "Done. Next:"
echo "  cargo cli settings-init"
echo "  cargo cli user-add <username> <password>"
