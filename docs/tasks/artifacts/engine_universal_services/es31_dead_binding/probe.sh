#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.31 (a) — the dead `let filename_str` binding is GONE, and its removal
# changed NOTHING ELSE.
#
# `.25` slice 1 measured that every generated parser emitted `let filename_str = "<-o path>";` once
# per rule method and that **nothing ever read it**: 2 704 bindings across the eleven artifacts,
# 0 non-assignment uses, reported by rustc as 2 848 `unused variable: filename_str` warnings on
# every build (2 848 rather than 2 704 because `generated/ebnf.rs` is `include!`d at two sites).
#
# This bank is the after-the-fact proof. Its load-bearing claim is not "the warnings are gone" —
# that is easy and weak — but **"the ONLY thing that changed is those lines"**, which is what makes
# the artifact-keyed rebaseline a bookkeeping act rather than a behavioural one.
# ⚠️ CORRECTION (`.31` slice 2): this line named THREE baselines and there are **two**.
# `git grep -l <the SV parser's sha256>` returns `generated_reproducibility_v0.json` and the
# `parse_cost_ratchet/` artifacts, and nothing else; `CODEGEN-DETERMINISM` is a task TREE that owns
# no tracked artifact-keyed baseline at all. Derived, not remembered — the same class as the carried
# site count this leaf's sibling retracted (`DERIVED_STATE_CONTAINMENT.md` R1/R3).
#
# HOW:  bash docs/tasks/artifacts/engine_universal_services/es31_dead_binding/probe.sh
#       bash …/probe.sh --before rust/target/es31_before    # adds the line-removal identity arm
#
# ⛔ The `--before` snapshot is a pre-change copy of `generated/*_parser.rs` + `generated/ebnf.rs`.
#    Without it arm 5 reports NOT EVALUATED — LOUDLY, never as a pass — because a bank that
#    silently drops its strongest arm is indistinguishable from one that never had it.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT"

CMP="$ROOT/scripts/compare_generated_parsers.py"
EMITTER="rust/src/ast_pipeline/ast_based_generator.rs"
BEFORE=""
[ "${1:-}" = "--before" ] && BEFORE="${2:-}"

# The measured pre-change population (`.25` slice 1). Named per artifact so a single moved row is
# attributable, rather than a total that can be met by two errors cancelling.
DEAD_JSON='json_parser.rs 9
regex_parser.rs 276
return_annotation_parser.rs 35
rtl_const_expr_parser.rs 48
rtl_frontend_parser.rs 169
scratch_parser.rs 2
semantic_annotation_parser.rs 114
systemverilog_parser.rs 1608
systemverilog_preprocessor_parser.rs 74
vhdl_parser.rs 225
ebnf.rs 144'
DEAD_TOTAL=2704
SITES_BEFORE=63186          # embedded `-o` occurrences, all 11 artifacts, before slice 1
SITES_AFTER=60482           # = 63 186 − 2 704, the count slice 1 left behind
# ⛔⛔ THAT 60 482 IS AN ERA CONSTANT AND IT HAS SINCE MOVED — which is why arm 3 no longer compares
# against it blindly. `ENGINE-UNIVERSAL-SERVICES.31` slice 2 hoisted the surviving class-L literals
# to ONE module constant per artifact, so the live total is now the ARTIFACT COUNT. A bank that had
# kept asserting 60 482 would have gone red against a correct tree one commit later
# ([[a-bank-pinned-to-the-shipped-behaviour-is-pinned-to-a-moving-target]]). Arm 3 therefore DERIVES
# which era it is in from the emitter's own source, and checks the count that era predicts.
LABEL_CONST='PGEN_SOURCE_LABEL'

fails=0; arms=0
pass() { arms=$((arms+1)); printf '  ✓ %s\n' "$1"; }
fail() { arms=$((arms+1)); fails=$((fails+1)); printf '  ✗ %s\n' "$1" >&2; }
note() { printf '  ⚠️  %s\n' "$1"; }

artifacts() { ls generated/*_parser.rs generated/ebnf.rs 2>/dev/null; }

printf '\nES31-DEAD-BINDING: the emitted `let filename_str` is gone and nothing else moved\n'
printf 'commit : %s\n\n' "$(git log -1 --format=%h)"

[ -f "$CMP" ] || { printf 'es31-probe: missing %s\n' "$CMP" >&2; exit 2; }
[ -n "$(artifacts)" ] || { printf 'es31-probe: generated/ holds no parser — regenerate first\n' >&2; exit 2; }

# ── ARM 1 — the EMITTER no longer emits it, and says so ───────────────────────────────────────────
printf 'ARM 1  the emitter\n'
# ⛔ ANCHORED AT LINE START, because the DO-NOT-RE-ADD comment QUOTES the very code it forbids.
# An unanchored `grep -q 'let filename_str = #filename'` matches that comment and reports the
# change as reverted — measured: this arm failed on its own first run, against a correct tree.
# The mutant in arm 4 still trips it, which is what keeps the anchoring honest.
if grep -qE '^[[:space:]]*let filename_str = #filename' "$EMITTER"; then
  fail "$EMITTER STILL emits \`let filename_str = #filename;\` — the change was reverted"
else
  pass "$EMITTER emits no \`let filename_str\` binding"
fi
if grep -q 'DO NOT RE-ADD .let filename_str' "$EMITTER"; then
  pass "the emitter carries the DO-NOT-RE-ADD marker naming the leaf"
else
  fail "the DO-NOT-RE-ADD marker is missing — the next author has nothing to read"
fi

# ── ARM 2 — no ARTIFACT carries the identifier at all ────────────────────────────────────────────
printf 'ARM 2  the artifacts\n'
residue=0
while IFS= read -r f; do
  n=$(grep -c 'filename_str' "$f")
  [ "$n" = 0 ] || { residue=$((residue+n)); printf '      %s carries %s\n' "$f" "$n" >&2; }
done < <(artifacts)
[ "$residue" = 0 ] \
  && pass "0 \`filename_str\` occurrences across all $(artifacts | wc -l | tr -d ' ') artifacts (was $DEAD_TOTAL)" \
  || fail "$residue \`filename_str\` occurrences survive"

# ── ARM 3 — the embedded `-o` site count fell by EXACTLY the dead population ──────────────────────
# ⛔ A total alone can be met by two errors cancelling, so this is checked per artifact first.
printf 'ARM 3  embedded -o sites: per artifact, then the total\n'
total=0; bad=0
# ⛔ NO `IFS=` HERE. `IFS= read -r name dead` disables word splitting, so `name` swallows the whole
# `"<file> <count>"` line and every `[ -f generated/$name ]` test fails — measured: all 11 rows
# reported ABSENT against a tree where all 11 existed, and the total came out 0 against an
# expectation of 60 482. A row that reads ABSENT for every input is an instrument fault, not data.
while read -r name dead; do
  f="generated/$name"; [ -f "$f" ] || { printf '      %s ABSENT\n' "$f" >&2; bad=$((bad+1)); continue; }
  now=$(python3 "$CMP" --sites "$f") || { bad=$((bad+1)); continue; }
  total=$((total + now))
done <<< "$DEAD_JSON"
[ "$bad" = 0 ] || fail "$bad artifact(s) could not be measured"
# Which era is this tree in? Read the EMITTER, do not guess from the number we are about to check.
n_artifacts=$(artifacts | wc -l | tr -d ' ')
if grep -qE "^[[:space:]]*const #source_label: &str = #filename;" "$EMITTER"; then
  expected="$n_artifacts"; era="slice 2 (class L hoisted to one \`$LABEL_CONST\` constant per artifact)"
else
  expected="$SITES_AFTER";  era="slice 1 (class D removed, class L still emitted per site)"
fi
if [ "$total" = "$expected" ]; then
  if [ "$expected" = "$SITES_AFTER" ]; then
    pass "embedded sites total $SITES_BEFORE -> $total, i.e. exactly −$DEAD_TOTAL — era: $era"
  else
    pass "embedded sites total is $total = one per artifact ($n_artifacts) — era: $era; slice 1's own $SITES_BEFORE -> $SITES_AFTER stays the record of that era, re-derivable from the es31_label_hoist bank's ARM 1"
  fi
else
  fail "embedded sites total is $total, but the emitter says this tree is in $era, which predicts $expected"
fi

# ── ARM 4 — RED CONTROL: the arm-1 check must be able to FAIL ────────────────────────────────────
# A control never observed failing is not known to work (`docs/CLAIM_VERIFICATION.md` §3 leg 2).
printf 'ARM 4  RED control — arm 1 must be able to fail\n'
mutant="$(mktemp)"; trap 'rm -f "$mutant"' EXIT
sed 's|// ⛔ DO NOT RE-ADD|let filename_str = #filename; // ⛔ DO NOT RE-ADD|' "$EMITTER" > "$mutant"
if grep -q 'let filename_str = #filename' "$mutant"; then
  pass "a reinstated binding IS detected by arm 1's predicate (RED as designed)"
else
  fail "the mutant was not detected — arm 1's predicate cannot fail and proves nothing"
fi

# ── ARM 5 — THE LOAD-BEARING ONE: nothing but those lines changed ────────────────────────────────
printf 'ARM 5  line-removal identity (the claim the rebaseline rests on)\n'
if [ -z "$BEFORE" ]; then
  note "NOT EVALUATED — no --before snapshot given. Re-run as:"
  note "    bash ${BASH_SOURCE[0]#$ROOT/} --before <dir-of-pre-change-artifacts>"
  note "This is the arm that proves the rebaseline is bookkeeping, not behaviour."
elif [ ! -d "$BEFORE" ]; then
  fail "--before '$BEFORE' is not a directory"
elif [ "$expected" = "$n_artifacts" ]; then
  # ⛔ ERA GUARD. This arm isolates ONE change — slice 1's dead-line removal. Once slice 2 hoisted
  # class L, a live tree's AFTER differs from any pre-slice-1 BEFORE by TWO changes, so the
  # comparison would report "differs by more than the removed lines" about a correct tree. That is a
  # false finding, and it is exactly the shape this bank exists to prevent, so it refuses to make
  # the comparison instead of making it wrongly. Slice 1's own 11/11 result stands in its leaf.
  note "NOT EVALUATED — this tree is in the slice-2 (hoisted) era, so AFTER differs from any"
  note "pre-slice-1 snapshot by TWO changes and this arm cannot isolate slice 1's. Its 11/11"
  note "result is recorded in docs/tasks/ENGINE-UNIVERSAL-SERVICES.md \`.31\` SLICE 1; the"
  note "slice-2 substitution has its own identity arm in the es31_label_hoist bank (ARM C)."
else
  ident=0; diffs=0
  while IFS= read -r f; do
    b="$BEFORE/$(basename "$f")"
    [ -f "$b" ] || { printf '      no before-snapshot for %s\n' "$(basename "$f")" >&2; diffs=$((diffs+1)); continue; }
    # strip ONLY the dead-binding lines from BEFORE, then demand byte-identity with AFTER.
    if [ "$(grep -v '^[[:space:]]*let filename_str = ' "$b" | shasum -a 256 | cut -d' ' -f1)" \
       = "$(shasum -a 256 < "$f" | cut -d' ' -f1)" ]; then
      ident=$((ident+1))
    else
      diffs=$((diffs+1)); printf '      %s differs by MORE than the removed lines\n' "$(basename "$f")" >&2
    fi
  done < <(artifacts)
  [ "$diffs" = 0 ] \
    && pass "all $ident artifacts: BEFORE minus the \`let filename_str\` lines is BYTE-IDENTICAL to AFTER" \
    || fail "$diffs artifact(s) changed by more than the removed lines — the change is NOT purely the dead binding"

  # RED control for arm 5: stripping the WRONG lines must NOT reproduce AFTER.
  one=$(artifacts | head -1); b="$BEFORE/$(basename "$one")"
  if [ -f "$b" ] && [ "$(grep -v '^[[:space:]]*let position = ' "$b" | shasum -a 256 | cut -d' ' -f1)" \
     != "$(shasum -a 256 < "$one" | cut -d' ' -f1)" ]; then
    pass "RED: stripping a DIFFERENT line from BEFORE does not reproduce AFTER"
  else
    fail "RED control did not fire — arm 5's comparison cannot discriminate"
  fi
fi

printf '\n'
if [ "$fails" -eq 0 ]; then
  printf 'ES31-DEAD-BINDING: %d/%d arms as declared\n' "$arms" "$arms"
else
  printf 'ES31-DEAD-BINDING: %d of %d ARMS FAILED\n' "$fails" "$arms" >&2
fi
exit "$fails"
