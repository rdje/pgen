#!/usr/bin/env bash
# scripts/check_sv_contract_currency.sh — doctrine SV-CONTRACT-CURRENCY (SV-CORPUS-GRAD.13c.2l).
#
# ⛔ WHAT THIS PROVES. The SystemVerilog downstream integration contract describes the grammar that
# is actually in the tree. Every revision of `grammars/systemverilog.ebnf` since the register's
# declared genesis is ACCOUNTED FOR: either it took a release number and a bug-ledger row, or it is
# recorded as ACCEPT-SET-NEUTRAL — and a neutrality claim is CHECKED, not believed.
#
# ⛔ WHY IT EXISTS, measured rather than imagined (`SV-CORPUS-GRAD.13c.2l`). Between 2026-08-12 and
# 2026-08-19 the SV grammar shipped SEVEN semantically-distinct revisions — four widenings, three
# narrowings and an AST-shape replacement — while the contract kept saying `1.0.183`. A consumer
# reading it was told the parser accepts text it now REFUSES. Nothing in the repository compared the
# grammar's last-modified commit with the contract's; the check that would have caught it on day one
# was one `git log` away and did not exist. ⭐ The leaf that opened to fix that debt then tracked
# the population BY HAND, and its hand-kept table went stale twice more inside the same week — so
# the register is not a second hand-kept list: every row carries the digest that makes its own claim
# falsifiable, and this script re-derives the newest one from the producer.
#
# THE IDENTITY IS THE FRONTEND'S OWN VIEW OF THE GRAMMAR, NOT ITS BYTES. `ast_pipeline
# --emit-raw-ast-json` prints the EBNF frontend's `raw_ast` envelope, which is what the code
# generator consumes; comments never reach it. So the digest is stable across a comment-only rewrite
# (measured: `-0234` and `-0238` leave it byte-identical) and moves on any change the generated
# parser can observe. Deriving the classifier from the PRODUCER rather than from a description of it
# is `docs/CLAIM_VERIFICATION.md` leg 2.
#
# FOUR TIERS, each catching what the others structurally cannot:
#   A — HISTORY   every grammar commit since the register's genesis has a register row.
#   B — STAGED    a staged grammar edit must stage the register too (tier A cannot see an
#                 uncommitted commit, which is exactly when the habit fails).
#   C — NEUTRALITY a `NEUTRAL` row's digest must equal its predecessor's. The claim refutes itself
#                 when false, from the register alone, with no binary.
#   D — IDENTITY  the working tree's digest re-derived from the producer must equal the newest row's
#                 and the contract's declared digest. Needs `ast_pipeline`; when it is absent the
#                 tier is reported NOT EVALUATED and tier B is what still binds.
#
# Contract: exit 0 = the doctrine holds; nonzero = a breach, explained on stderr (DOCTRINE_ENFORCEMENT.md §4).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 2

GRAMMAR="grammars/systemverilog.ebnf"
CONTRACT="docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md"
REGISTER="docs/contracts/PGEN_SV_GRAMMAR_REVISION_REGISTER.tsv"
PIPELINE="rust/target/debug/ast_pipeline"
fail=0
note() { printf 'sv-contract-currency: %s\n' "$1" >&2; fail=1; }

for f in "$GRAMMAR" "$CONTRACT" "$REGISTER"; do
  [ -f "$f" ] || { note "$f is missing — the doctrine cannot inspect its subject"; exit 1; }
done

# The genesis is DECLARED BY the register, so this script carries no commit literal of its own.
GENESIS="$(sed -n 's/^# genesis: *\([0-9a-f]\{7,40\}\).*/\1/p' "$REGISTER" | head -1)"
[ -n "$GENESIS" ] || { note "$REGISTER declares no '# genesis: <sha>' line — without it tier A has no range"; exit 1; }

# Rows, in file order: commit<TAB>date<TAB>slice<TAB>digest<TAB>disposition<TAB>release<TAB>notes
mapfile -t ROWS < <(grep -v '^#' "$REGISTER" | grep -v '^[[:space:]]*$' | tail -n +2)
[ "${#ROWS[@]}" -gt 0 ] || { note "$REGISTER carries no rows — a register that lists nothing cannot pass"; exit 1; }

row_field() { printf '%s' "$1" | cut -f"$2"; }

# ── tier A — HISTORY: every grammar commit since genesis is accounted for ────────────────────────
missing=""
while read -r sha; do
  [ -n "$sha" ] || continue
  short="${sha:0:8}"
  printf '%s\n' "${ROWS[@]}" | cut -f1 | grep -qx "$short" || missing="$missing $short"
done < <(git rev-list --reverse "$GENESIS..HEAD" -- "$GRAMMAR")
if [ -n "$missing" ]; then
  note "the SV grammar moved in commit(s)$missing with NO row in $REGISTER."
  note "  Each grammar revision owes a row: a RELEASE (contract section + bug-ledger row) or a"
  note "  NEUTRAL claim whose digest equals its predecessor's. Re-derive the digest with:"
  note "    $PIPELINE $GRAMMAR --emit-raw-ast-json raw.json && python3 -c \"import json,sys,hashlib; print(hashlib.sha256(json.dumps(json.load(open('raw.json'))['raw_ast'],sort_keys=True,separators=(',',':')).encode()).hexdigest())\""
fi

# ── tier B — STAGED: tier A cannot see the commit being made; this can ───────────────────────────
staged="$(git diff --cached --name-only 2>/dev/null)"
if printf '%s\n' "$staged" | grep -qx "$GRAMMAR"; then
  printf '%s\n' "$staged" | grep -qx "$REGISTER" \
    || note "$GRAMMAR is staged and $REGISTER is not. A grammar revision that reaches a commit
  without a register row is exactly the drift this doctrine exists to stop — add its row (RELEASE or
  NEUTRAL) in the SAME commit."
fi

# ── tier C — NEUTRALITY: a NEUTRAL row's digest must equal its predecessor's ─────────────────────
prev_digest=""
for row in "${ROWS[@]}"; do
  commit="$(row_field "$row" 1)"; digest="$(row_field "$row" 4)"; disp="$(row_field "$row" 5)"
  case "$disp" in
    RELEASE) ;;
    NEUTRAL)
      if [ -z "$prev_digest" ]; then
        note "row $commit claims NEUTRAL with no predecessor row to be neutral against"
      elif [ "$digest" != "$prev_digest" ]; then
        note "row $commit claims ACCEPT-SET-NEUTRAL but its semantic digest ($digest) differs from
  its predecessor's ($prev_digest). A neutrality claim is a claim; this one is refuted by the
  register's own numbers."
      fi
      ;;
    *) note "row $commit carries disposition '$disp' — only RELEASE and NEUTRAL are defined, and an
  unreadable disposition must never read as a green one" ;;
  esac
  [ -n "$digest" ] || note "row $commit carries no semantic digest, so nothing about it is checkable"
  prev_digest="$digest"
done

newest_row="${ROWS[${#ROWS[@]}-1]}"
newest_digest="$(row_field "$newest_row" 4)"

# The contract must DECLARE the identity it describes, and it must be the newest row's.
declared="$(sed -n 's/^ *- *`\([0-9a-f]\{64\}\)` *(SV grammar semantic digest.*/\1/p' "$CONTRACT" | head -1)"
if [ -z "$declared" ]; then
  note "$CONTRACT declares no 'SV grammar semantic digest' in its Contract Identity block — without
  it the contract names no grammar and tier D has nothing to compare against"
elif [ "$declared" != "$newest_digest" ]; then
  note "the contract declares grammar digest $declared but the register's newest row
  ($(row_field "$newest_row" 1)) carries $newest_digest — the contract describes a different grammar
  than the one the register says shipped last"
fi

# ── tier D — IDENTITY: re-derive the working tree's digest from the producer ─────────────────────
if [ -x "$PIPELINE" ]; then
  # ⛔ To a FILE, never `/dev/stdout`: the tool also writes progress to stdout, so the stream
  # carries valid JSON followed by more text and `json.load` fails with "Extra data" — measured
  # while writing this check, which is why the recipe printed on breach names a file too.
  raw_json="$(mktemp "${TMPDIR:-/tmp}/sv_contract_currency.XXXXXX.json")"
  "$PIPELINE" "$GRAMMAR" --emit-raw-ast-json "$raw_json" >/dev/null 2>&1
  live="$(python3 -c '
import hashlib, json, sys
try:
    raw = json.load(open(sys.argv[1]))["raw_ast"]
except Exception:
    sys.exit(0)
print(hashlib.sha256(json.dumps(raw, sort_keys=True, separators=(",", ":")).encode()).hexdigest())
' "$raw_json")"
  rm -f "$raw_json"
  if [ -z "$live" ]; then
    note "tier D could not re-derive the grammar's semantic digest — $PIPELINE ran but produced no
  raw_ast envelope. A tier that cannot inspect its subject must say so, not pass."
  elif [ "$live" != "$newest_digest" ]; then
    note "the WORKING TREE grammar's semantic digest is $live, and the register's newest row
  ($(row_field "$newest_row" 1)) records $newest_digest. The grammar has changed in a way the
  generated parser can observe and no register row describes it yet."
  fi
else
  printf 'sv-contract-currency: tier D NOT EVALUATED — %s is not built, so the working-tree digest
  was not re-derived from the producer. Tiers A-C ran. Build it to close the loop:
    cargo build --features "generated_parsers ebnf_dual_run" --manifest-path rust/Cargo.toml\n' "$PIPELINE"
fi

if [ "$fail" = 0 ]; then
  printf 'SV-CONTRACT-CURRENCY: rows=%d genesis=%s newest=%s digest=%s\n' \
    "${#ROWS[@]}" "${GENESIS:0:8}" "$(row_field "$newest_row" 1)" "${newest_digest:0:16}"
fi
exit $fail
