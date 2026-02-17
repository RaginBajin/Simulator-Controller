#!/bin/bash
set -e

echo "Checking migration files for destructive statements..."

FORBIDDEN_PATTERNS="DROP TABLE|DROP COLUMN|DROP INDEX|DELETE FROM|TRUNCATE"
MIGRATION_DIR="crates/storage/migrations"

if [ ! -d "$MIGRATION_DIR" ]; then
  echo "Migration directory not found: $MIGRATION_DIR"
  exit 1
fi

FOUND=0
for file in "$MIGRATION_DIR"/*.sql; do
  # Skip files with safety:allow-drop comment
  if head -1 "$file" | grep -q "safety:allow-drop"; then
    echo "ALLOWED: $file has safety:allow-drop annotation, skipping"
    continue
  fi

  if grep -iE "$FORBIDDEN_PATTERNS" "$file" > /dev/null 2>&1; then
    echo "FORBIDDEN: Destructive SQL found in $file"
    grep -inE "$FORBIDDEN_PATTERNS" "$file"
    FOUND=1
  fi
done

if [ $FOUND -eq 1 ]; then
  echo "Migration safety check FAILED. Destructive SQL statements are not allowed at MVP."
  echo "If a migration legitimately requires destructive SQL (e.g., SQLite table recreation),"
  echo "add '-- safety:allow-drop' as the first line of the migration file."
  exit 1
fi

echo "Migration safety check passed."
