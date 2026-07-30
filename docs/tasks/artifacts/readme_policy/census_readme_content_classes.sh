#!/usr/bin/env bash
# docs/tasks/artifacts/readme_policy/census_readme_content_classes.sh
# README-POLICY.1 — measured census of README.md against the README Stability Policy's
# content contract. Read-only. Prints a per-section line/byte census and classifies each
# top-level section as KEEP (landing page) or ROUTE (changing detail with a canonical home).
#
# WHY this exists: the policy's adoption checklist step 1 is "remove duplicated status,
# history, inventories, and deep reference prose". That requires knowing WHICH sections
# carry which class and HOW BIG each is — eyeballing a 510-line file is exactly the
# failure mode this repo's toolbox doctrine forbids.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"
TARGET="${1:-README.md}"

[ -f "$TARGET" ] || { printf 'census: %s not found (ROOT=%s)\n' "$TARGET" "$ROOT" >&2; exit 2; }

printf '== census: %s ==\n' "$TARGET"
printf 'total: %s lines, %s bytes\n\n' \
  "$(wc -l < "$TARGET" | tr -d ' ')" "$(wc -c < "$TARGET" | tr -d ' ')"

printf '%-6s %-6s %-8s  %s\n' 'start' 'lines' 'bytes' 'section'
printf '%-6s %-6s %-8s  %s\n' '-----' '-----' '-----' '-------'

awk -v file="$TARGET" '
  /^## / {
    if (name != "") printf "%-6d %-6d %-8d  %s\n", start, NR-start, bytes, name
    name = substr($0, 4); start = NR; bytes = 0
  }
  { if (name != "") bytes += length($0) + 1 }
  END { if (name != "") printf "%-6d %-6d %-8d  %s\n", start, NR-start+1, bytes, name }
' "$TARGET"

printf '\n== growth-guard reading ==\n'
printf 'lines=%s bytes=%s\n' \
  "$(wc -l < "$TARGET" | tr -d ' ')" "$(wc -c < "$TARGET" | tr -d ' ')"
