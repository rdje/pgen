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
#   A — HISTORY   every grammar commit since the register's genesis has a register row, matched by
#                 the row's DIGEST rather than by its commit sha. ⛔ Sha-keying was the first design
#                 and it had a chicken-and-egg the very next grammar change exposed: the row for the
#                 commit being made cannot carry that commit's own sha, so tier D forced the row to
#                 exist while tier A then rejected it on the NEXT commit and demanded a fixup. The
#                 digest is known before the commit is, and it is also the identity the contract
#                 actually describes; the `commit` column stays as documentation.
#   B — STAGED    a staged grammar edit must stage the register too (tier A cannot see an
#                 uncommitted commit, which is exactly when the habit fails).
#   C — NEUTRALITY a `NEUTRAL` row's digest must equal its predecessor's. The claim refutes itself
#                 when false, from the register alone, with no binary.
#   D — IDENTITY  the working tree's digest re-derived from the producer must equal the newest row's
#                 and the contract's declared digest. Needs `ast_pipeline`; when it is absent the
#                 tier is reported NOT EVALUATED and tier B is what still binds.
#   E — PARSER    a `GENERATOR-ONLY` row claims the digest MOVED while the generated parser did not,
#                 and that claim is re-derived rather than believed: the row's `parser_sha256=<hex>`
#                 must equal the sha256 of the shipped `generated/systemverilog_parser.rs`.
#                 `generated/` is untracked, so on a fresh clone this tier reports NOT EVALUATED,
#                 loudly — never a pass.
#
# ⛔ WHY A THIRD DISPOSITION EXISTS (SV-CORPUS-GRAD.13e.9, 2026-08-25). The first two were RELEASE
# (contract section + bug-ledger row) and NEUTRAL (comment-only; digest EQUALS its predecessor's).
# A `@sample` repair fits NEITHER: the annotation IS in the raw_ast, so the digest moves and NEUTRAL
# refutes itself — while the generated parser is byte-identical, so the accept set cannot have moved
# and a bug-ledger row would describe a defect no consumer of the parser can observe. Forcing such a
# change into RELEASE buys a currency check with a false ledger row. ⛔ THE HOLE IS ONE-SIDED, AND
# THAT IS THE SHARP PART: mislabelling it NEUTRAL is REFUSED by tier C below (measured, rc=1), so
# the only wrong label that PASSES is RELEASE -- the single remaining slot is the silent one.
# The honest third slot states a STRONGER claim than
# NEUTRAL's and is checked more cheaply than tier D: not "the grammar text barely changed" but
# "whatever changed, the code generator's output did not".
# ⚠️ HONEST BOUND, stated before the tier is trusted: tier E compares the row against the parser
# that is PRESENT in generated/. That it is the parser HEAD's grammar actually produces is the
# separate GENERATED-REPRODUCIBILITY doctrine's job. The two compose; neither alone is the argument.
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

# The semantic digest of one revision of the grammar, from the PRODUCER. Empty when unavailable.
sv_semantic_digest() {  # $1 = a file holding the grammar text
  [ -x "$PIPELINE" ] || return 0
  # ⛔ ON-VOLUME BY POLICY (CLAUDE.md §13): project-owned scratch is derived from the repository
  # root, never from $TMPDIR, which can sit on a different filesystem. Fixed 2026-08-20 by
  # SV-CORPUS-GRAD.13c.2x.4, which adopts this function's own digest definition and read it closely.
  local scratch="$ROOT/rust/target/sv_contract_currency"; mkdir -p "$scratch" 2>/dev/null
  local out; out="$(mktemp "$scratch/digest.XXXXXX.json")"
  "$PIPELINE" "$1" --emit-raw-ast-json "$out" >/dev/null 2>&1
  python3 -c '
import hashlib, json, sys
try:
    raw = json.load(open(sys.argv[1]))["raw_ast"]
except Exception:
    sys.exit(0)
print(hashlib.sha256(json.dumps(raw, sort_keys=True, separators=(",", ":")).encode()).hexdigest())
' "$out"
  rm -f "$out"
}

# ── tier A — HISTORY: every grammar commit since genesis is accounted for, BY DIGEST ─────────────
if [ -x "$PIPELINE" ]; then
  # ⛔ ON-VOLUME BY POLICY (CLAUDE.md §13), same as sv_semantic_digest above. This line read
  # `${TMPDIR:-/tmp}` until SV-CORPUS-GRAD.13e.9 — the exact off-volume default the function 15 lines
  # up documents as fixed, left behind because that fix was made where the bug was REPORTED rather
  # than over the file's whole population. A per-site fix to a per-file defect is half a fix.
  hist_root="$ROOT/rust/target/sv_contract_currency"; mkdir -p "$hist_root" 2>/dev/null
  scratch="$(mktemp -d "$hist_root/hist.XXXXXX")"
  missing=""
  while read -r sha; do
    [ -n "$sha" ] || continue
    git show "$sha:$GRAMMAR" > "$scratch/g.ebnf" 2>/dev/null || { note "cannot read $GRAMMAR at $sha"; continue; }
    d="$(sv_semantic_digest "$scratch/g.ebnf")"
    if [ -z "$d" ]; then
      note "tier A could not derive the semantic digest of $GRAMMAR at ${sha:0:8} — a tier that cannot
  inspect its subject must say so, not pass"
    elif ! printf '%s\n' "${ROWS[@]}" | cut -f4 | grep -qx "$d"; then
      missing="$missing ${sha:0:8}"
    else
      # ⛔ A `(pending)` row is the price of digest-keying: the row for the commit being made cannot
      # name that commit. The moment the commit EXISTS, the placeholder is rot, so back-filling it is
      # gated rather than reminded — a reminder has already lost 1 592 times in this repository
      # (LESSON-RETRIEVAL.1). The digest is what identifies the row, so the fix is one edit.
      pending_row="$(printf '%s\n' "${ROWS[@]}" | awk -F'\t' -v d="$d" '$4 == d && $1 == "(pending)"')"
      [ -z "$pending_row" ] || note "the register row for semantic digest ${d:0:16}… still reads
  \`(pending)\` in its \`commit\` column, but that state IS committed — as ${sha:0:8}. Back-fill it:
  the digest is what identifies the row, so this is a one-word edit in $REGISTER."
    fi
  done < <(git rev-list --reverse "$GENESIS..HEAD" -- "$GRAMMAR")
  rm -rf "$scratch"
else
  printf 'sv-contract-currency: tier A NOT EVALUATED — %s is not built, so per-revision digests were
  not derived. Tier B (the staged diff) still binds, and it is the tier that catches the change
  being made. Build it to close the loop:
    cargo build --features "generated_parsers ebnf_dual_run" --manifest-path rust/Cargo.toml\n' "$PIPELINE"
  missing=""
fi
if [ -n "$missing" ]; then
  note "the SV grammar moved in commit(s)$missing to a semantic state NO row in $REGISTER records."
  note "  Each grammar revision owes a row: a RELEASE (contract section + bug-ledger row), a"
  note "  NEUTRAL claim whose digest equals its predecessors, or a GENERATOR-ONLY claim whose"
  note "  parser_sha256 equals the shipped parsers. Re-derive the digest with:"
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
    GENERATOR-ONLY)
      # The claim is "the digest moved, the PARSER did not". Both halves are checked: the digest
      # half here (a GENERATOR-ONLY row whose digest EQUALS its predecessor's is a NEUTRAL row
      # mislabelled, and the stronger label must not be used to dress up the weaker fact), the
      # parser half in tier E.
      if [ -z "$prev_digest" ]; then
        note "row $commit claims GENERATOR-ONLY with no predecessor row to have moved away from"
      elif [ "$digest" = "$prev_digest" ]; then
        note "row $commit claims GENERATOR-ONLY, but its semantic digest EQUALS its predecessor's
  ($digest). Nothing moved, so this row is NEUTRAL — a stronger disposition must not be used to
  describe a weaker fact."
      fi
      [ -z "$(row_field "$row" 6)" ] || note "row $commit claims GENERATOR-ONLY and also names a
  release ($(row_field "$row" 6)). A revision the generated parser cannot observe ships no release;
  if it does ship one, it is a RELEASE row."
      printf '%s' "$(row_field "$row" 7)" | grep -qE 'parser_sha256=[0-9a-f]{64}' \
        || note "row $commit claims GENERATOR-ONLY but carries no \`parser_sha256=<64 hex>\` in its
  notes. The whole point of this disposition is that the claim is re-derivable; without the digest
  of the parser it claims did not move, it is an assertion."
      ;;
    *) note "row $commit carries disposition '$disp' — only RELEASE, NEUTRAL and GENERATOR-ONLY are
  defined, and an unreadable disposition must never read as a green one" ;;
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
  live="$(sv_semantic_digest "$GRAMMAR")"
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

# ── tier E — PARSER: a GENERATOR-ONLY row's "the parser did not move" is RE-DERIVED ─────────────
# Scoped to the NEWEST row deliberately: generated/ holds exactly one parser, so only the newest
# row's claim is one this tree can still answer. An older row's claim was answered when it landed.
if [ "$(row_field "$newest_row" 5)" = "GENERATOR-ONLY" ]; then
  PARSER="generated/systemverilog_parser.rs"
  claimed="$(printf '%s' "$(row_field "$newest_row" 7)" | grep -oE 'parser_sha256=[0-9a-f]{64}' | head -1 | cut -d= -f2)"
  if [ ! -f "$PARSER" ]; then
    printf 'sv-contract-currency: tier E NOT EVALUATED — %s is absent (generated/ is untracked, so a
  fresh clone reaches here). The GENERATOR-ONLY claim of row %s is UNCHECKED until the tree is
  generated. Tiers A-D ran. Close the loop with:
    make -C rust SHELL=/bin/bash regenerate_generated_parsers\n' "$PARSER" "$(row_field "$newest_row" 1)"
  elif [ -z "$claimed" ]; then
    : # already reported by tier C — do not double-count one defect as two
  else
    live_parser="$(shasum -a 256 "$PARSER" 2>/dev/null | cut -d' ' -f1)"
    if [ -z "$live_parser" ]; then
      note "tier E could not hash $PARSER — a tier that cannot inspect its subject must say so, not pass"
    elif [ "$live_parser" != "$claimed" ]; then
      note "row $(row_field "$newest_row" 1) claims GENERATOR-ONLY with parser_sha256=$claimed, but
  $PARSER hashes to $live_parser. The claim is that this grammar revision left the generated parser
  untouched; the parser in the tree says otherwise, so the revision is a RELEASE and owes a contract
  section and a bug-ledger row."
    fi
  fi
fi

if [ "$fail" = 0 ]; then
  printf 'SV-CONTRACT-CURRENCY: rows=%d genesis=%s newest=%s digest=%s\n' \
    "${#ROWS[@]}" "${GENESIS:0:8}" "$(row_field "$newest_row" 1)" "${newest_digest:0:16}"
fi
exit $fail
