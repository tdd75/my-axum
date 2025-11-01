#!/usr/bin/env sh
set -eu

if [ -z "${DATABASE_URL:-}" ]; then
    echo "DATABASE_URL is not set."
    exit 1
fi

DB_HOST=$(printf '%s\n' "$DATABASE_URL" | sed -E 's|postgresql(\+[^:]+)?://[^@]*@([^:/]+).*|\2|')
DB_PORT=$(printf '%s\n' "$DATABASE_URL" | sed -E 's|.*:([0-9]+)/.*|\1|' | grep -E '^[0-9]+$' || true)

if [ -z "$DB_PORT" ]; then
    DB_PORT=5432
fi

echo "Waiting for PostgreSQL to be ready..."
until nc -z -w 1 "$DB_HOST" "$DB_PORT" > /dev/null 2>&1; do
    echo "PostgreSQL is unavailable - sleeping"
    sleep 1
done

echo "Port is open, waiting for app layer to stabilize..."
sleep 2
echo "PostgreSQL is up and ready!"
