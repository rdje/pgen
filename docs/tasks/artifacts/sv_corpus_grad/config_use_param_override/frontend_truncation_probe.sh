#!/usr/bin/env bash
# Does a `#` comment inside a rule body silently discard the following alternatives?
# (`SV-CORPUS-GRAD.3.19` diagnosis → routed to `EBNF-FRONTEND-SILENT-TRUNCATION`)
#
# ⭐ This exists because the OBVIOUS root cause was WRONG. When `.3.19`'s repro matrix went
# red, the natural reading was "`#` is mishandled" — and the new alternative did contain a
# `#` both in its `@probe_sample` string AND in the comment block above it. Guessing either
# one would have produced a plausible, wrong finding. The discriminator below varies exactly
# one thing per case, so the answer is measured rather than argued.
#
# Every case declares THREE alternatives, so the expected IR answer is 3 every time.
# Anything less is language the frontend silently deleted.
#
# Usage (from anywhere; paths resolve against the repository root, directive 12):
#   bash docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/frontend_truncation_probe.sh
set -uo pipefail

# ⛔ TERMINATION GUARD (`SV-CORPUS-GRAD.13c.2b`, 2026-08-23). Directive 12 forbids a hard-coded
# depth, so the root is walked up from this script at run time. The walk MUST have its own
# terminator: `cd ..` at `/` SUCCEEDS and is a no-op, so a `cd .. || exit` escape can NEVER fire and
# the loop spins forever outside a checkout (measured: 201+ iterations, `pwd=/`, no error). The
# Python sibling `_repo_root.py` never had this — `Path.parents` is finite and it REFUSES by name.
find_repo_root() {
  local d
  d="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)" || return 1
  while [ ! -f "$d/CLAUDE.md" ] || [ ! -d "$d/grammars" ]; do
    [ "$d" = "/" ] && return 1
    d="$(dirname "$d")"
  done
  printf '%s\n' "$d"
}
root="$(find_repo_root)" || { echo "REFUSE: no repository root (CLAUDE.md + grammars/) above ${BASH_SOURCE[0]}" >&2; exit 2; }
pipeline="$root/rust/target/debug/ast_pipeline"
work="$root/rust/target/ebnf_truncation_probe"
[ -x "$pipeline" ] || { echo "REFUSE: build ast_pipeline first (cargo build --features ebnf_dual_run --bin ast_pipeline)" >&2; exit 2; }
mkdir -p "$work"

emit() { # emit <name> <body-lines...>
  local name="$1"; shift
  { echo '@entry: true'; printf '%s\n' "$@"; } > "$work/$name.ebnf"
}

emit C_baseline \
  'scratch := "a" "b"' \
  '        | "c" "d"' \
  '        | "e" "f"'

emit D_indented_comment \
  'scratch := "a" "b"' \
  '        # an INDENTED comment line between two alternatives' \
  '        | "c" "d"' \
  '        | "e" "f"'

emit B_hash_in_string \
  'scratch := @probe_sample: "x #(y)" "a" "b"' \
  '        | "c" "d"' \
  '        | "e" "f"'

emit A_col0_comment_after_alt1 \
  'scratch := "a" "b"' \
  '# a COLUMN-0 comment line between two alternatives' \
  '        | "c" "d"' \
  '        | "e" "f"'

emit E_col0_comment_after_alt2 \
  'scratch := "a" "b"' \
  '        | "c" "d"' \
  '# a COLUMN-0 comment line after the second alternative' \
  '        | "e" "f"'

printf '%-28s %-10s %s\n' case 'IR alts' 'expected 3'
for f in C_baseline D_indented_comment B_hash_in_string A_col0_comment_after_alt1 E_col0_comment_after_alt2; do
  "$pipeline" "$work/$f.ebnf" --generate-stimuli --count 1 --seed 0 \
      --dump-gen-ast "$work/$f.json" >/dev/null 2>&1
  python3 - "$work/$f.json" "$f" <<'PY'
import json, sys
node = json.load(open(sys.argv[1]))['grammar_tree']['scratch']
alts = node.get('Or', {}).get('alternatives')
n = len(alts) if alts else 1
print(f"{sys.argv[2]:<28} {n:<10} {'OK' if n == 3 else f'*** {3-n} ALTERNATIVE(S) SILENTLY LOST (top node {list(node)[0]}) ***'}")
PY
done
