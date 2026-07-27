#!/usr/bin/env bash
# docs/tasks/artifacts/doctrine_gap_ownership/run_gap_ownership_sweep.sh
# DOCTRINE-GAP-OWNERSHIP.1 — find every RECORDED-BUT-UNOWNED gap across the WHOLE repository.
#
# WHY: the box-scoping defect was found, written down on 2026-07-22, and then sat for 58 commits
# because it lived in a decision record's prose instead of a task-tree leaf. `docs/decisions/` is a
# MEMORY surface, not a WORK QUEUE. This driver makes the backlog countable instead of anecdotal.
#
# ⛔ A one-directory grep is NOT a census (director, 2026-07-27: "You should check everything!").
# Surfaces swept: decisions, task trees, root live docs, the book, contracts, reference docs,
# Rust sources, grammars, and shell gates.
#
# A hit is OWNED when a task-tree leaf ID or an explicit routing phrase appears near it; ORPHAN
# otherwise. Honest limit: this proves an owner was NAMED, not that the owner is real or active.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"
OUT="${1:-rust/target/doctrine_checks/gap_ownership_sweep.tsv}"
mkdir -p "$(dirname "$OUT")"

# Prose gap language (docs). Deliberately excludes forward-looking design prose like
# "is its own task-tree leaf", which is doctrine, not a recorded gap.
GAP_DOC='watch item|known gap|known limitation|known soundness gap|worth a future|hardening slice|future slice|not fixed here|not fixed in this leaf|left as-is|left as is|re-open if|reopen if|residual recorded|deliberately not fixed|out of scope for this leaf|follow-up slice|still missing|deferred to a future'
# Code markers.
GAP_CODE='TODO|FIXME|HACK|XXX:|@todo'
# An owner is a leaf ID (TREE.N / TREE.N.N / .b.6.2 style) or an explicit routing phrase.
OWNER='[A-Z][A-Z0-9]{2,}[A-Z0-9-]*\.[0-9a-z]+|routed to|owned by|tracked by|OWNER|new tree|new leaf|CLOSED|closed by|no longer missing|see docs/tasks/'

surface_of() {
  case "$1" in
    docs/decisions/*) echo decisions ;;
    docs/tasks/*)     echo task-trees ;;
    docs/book/*)      echo book ;;
    docs/contracts/*) echo contracts ;;
    docs/reference/*) echo reference ;;
    */*.rs)           echo rust-src ;;
    *.ebnf)           echo grammars ;;
    *.sh)             echo scripts ;;
    CHANGES.md|DEVELOPMENT_NOTES.md) echo history-narrative ;;
    */changelog-index.md)            echo history-narrative ;;
    *)                echo root-docs ;;
  esac
}

: >"$OUT"
scan() { # scan <file> <pattern>
  local f="$1" pat="$2" ln rest win
  grep -nEi -- "$pat" "$f" 2>/dev/null | while IFS=: read -r ln rest; do
    win=$(awk -v s=$((ln-3)) -v e=$((ln+8)) 'NR>=s && NR<=e' "$f")
    if printf '%s' "$win" | grep -Eq -- "$OWNER"; then
      printf 'OWNED\t%s\t%s\t%s\n' "$(surface_of "$f")" "$f:$ln" "$(printf '%s' "$rest" | cut -c1-120)"
    else
      printf 'ORPHAN\t%s\t%s\t%s\n' "$(surface_of "$f")" "$f:$ln" "$(printf '%s' "$rest" | cut -c1-120)"
    fi
  done
}

for f in $(git ls-files 'docs/decisions/*.md' 'docs/tasks/*.md' 'docs/book/src/*.md' \
                        'docs/contracts/*.md' 'docs/reference/*.md' '*.md'); do
  scan "$f" "$GAP_DOC"
done >>"$OUT"
for f in $(git ls-files 'rust/src/*.rs' 'rust/scripts/*.sh' 'scripts/*.sh' 'grammars/*.ebnf'); do
  scan "$f" "$GAP_CODE"
done >>"$OUT"

echo "=== GAP-OWNERSHIP SWEEP (whole repository) ==="
printf 'total hits : %s\n' "$(wc -l <"$OUT" | tr -d ' ')"
printf '  OWNED    : %s\n' "$(grep -ac '^OWNED' "$OUT" || true)"
printf '  ORPHAN   : %s\n' "$(grep -ac '^ORPHAN' "$OUT" || true)"
echo
echo "--- ORPHANS by surface ---"
grep -a '^ORPHAN' "$OUT" | cut -f2 | sort | uniq -c | sort -rn
echo
echo "(full detail: $OUT)"
