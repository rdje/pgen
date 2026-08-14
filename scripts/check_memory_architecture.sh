#!/usr/bin/env bash
# scripts/check_memory_architecture.sh
# SV-EXH-PROOF/MEMORY-ARCH (PGEN-MEMORY-ARCH-0005): single source of truth for the
# durable-memory-architecture invariants (per MEMORY_ARCHITECTURE.md §9 E2).
# Exits NONZERO on any breach. Called by .githooks/pre-commit (E3) and CI (E4).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"
# Layer-A resume-pointer caps. BOTH are load-bearing — see the note below before touching either.
#
# ⛔ WHY TWO CAPS, MEASURED IN THIS REPOSITORY (README-POLICY.2, 2026-07-30):
#   From adoption until today this check enforced the LINE cap alone, exactly as the portable
#   standard's own reference script prescribes (MEMORY_ARCHITECTURE.md §9). Measured at the start
#   of the trim, MEMORY.md was:
#       60 lines  -> PASSING, exactly at the 60-line ceiling
#       138,403 bytes -> UNBOUNDED, 2,306 bytes per line, single longest line 18,816 bytes
#   ⇒ a file whose own header calls it a "bounded resume pointer … keep ≤ ~50 lines" was a 138 KB
#   document, and this guard was green the entire time. The line cap held the line COUNT and held
#   back nothing. The "Current state" block — whose heading literally reads "OVERWRITE this block
#   each update — do not append" — carried 18 distinct sessions and 81.3% of the file.
#   The same class was independently found at README.md:115 (4,369 bytes on ONE line, no shared
#   code path), which is why scripts/check_readme_stability.sh shipped with both caps on day one.
#   ⭐ Line and byte caps are COMPLEMENTS, not redundancy: neither wrapped prose nor very long
#   lines can bypass the budget.
#
# ⛔ NEVER raise a cap to land content. Layer A holds where-we-are-NOW; anything else belongs in
#   docs/tasks/ (layer B) or docs/decisions/ (layer C), both of which are durable and addressable.
#   A cap increase requires an explicit reviewed decision recorded in docs/tasks/README-POLICY.md.
#
# The line cap: chosen AFTER the README-POLICY.2 trim (39 lines / 5,720 bytes) with deliberately
# proportional headroom (~28%). It is lowered 60 -> 50 to match the standard this script enforces,
# which says "≤ ~50 lines" in both MEMORY_ARCHITECTURE.md §6 and MEMORY.md's own header; 60 was
# looser than the rule it was policing.
#
# ⭐⭐ THE BYTE CAP IS 32768 BY DIRECTOR RULING (2026-08-14, README-POLICY.8) — NOT to fit content.
#   README-POLICY.2 set it to 7168 = the post-trim 5,720 B plus ~25% headroom. That headroom is
#   GONE, and the measurement is unambiguous (`git rev-list` + `git cat-file -s` over layer A's
#   own history):
#       last 40 layer-A commits: min 6795  median 7079  mean 7063  max 7168 == THE CAP EXACTLY
#       36 of those 40 sat at >= 97% of the cap; 18 of 40 came within 68 bytes of it
#       headroom at the ruling: 117 bytes — and 3 of the last 20 updates changed the file by
#       more than 117 bytes on net alone
#   ⇒ the cap had stopped bounding the LAYER and started editing the PROSE: `git diff --numstat`
#   over nine consecutive commits reports `4 4` — the same four lines rewritten in place, never
#   grown — i.e. authors were shaving bytes to fit rather than deciding what belongs in layer A.
#   That is the failure this repository already named for itself in
#   docs/knowledge/a-cap-with-no-headroom-is-a-cap-about-to-be-raised.md: *"the moment a cap blocks
#   you is the worst possible moment to decide policy about it."* The ruling was taken at the calm
#   moment the card asks for — layer A was PASSING, not blocked.
#
# ⛔ THE DISCIPLINE IS UNCHANGED, AND THE RAISE IS NOT A LICENCE. Layer A still holds
#   where-we-are-NOW and nothing else; it is still OVERWRITE-only; anything else still belongs in
#   docs/tasks/ (layer B) or docs/decisions/ (layer C). The raise buys HEADROOM so the demotion
#   decision is never made under commit pressure — it does not buy room to accumulate.
#
# ⚠️ HONEST STRUCTURAL COST, stated rather than discovered later: at 50 lines / 32768 bytes the
#   byte axis permits ~655 B/line, so in the 7-32 KB band the LINE cap is the only binding axis and
#   the two-axis design degrades toward the single-axis form README-POLICY.2 replaced. The byte cap
#   keeps its original job — making the 138,403-byte outcome impossible — but it is no longer
#   co-binding at pointer shape. Mitigation, per the card: HEADROOM is now reported on every
#   passing run (below), so the metric that predicts the next failure is visible on every commit
#   instead of only when the gate finally fires.
#
# ⛔ A cap increase requires an explicit reviewed decision recorded in docs/tasks/README-POLICY.md.
#   Both changes to this value now have one (.2 introduced it, .8 raised it).
CAP="${MEMORY_POINTER_LINE_CAP:-50}"
BYTE_CAP="${MEMORY_POINTER_BYTE_CAP:-32768}"
fail=0
note(){ printf 'memory-arch: %s\n' "$1" >&2; fail=1; }

# E2.1 — the standard itself must be present (system of record).
[ -f MEMORY_ARCHITECTURE.md ] || note "MEMORY_ARCHITECTURE.md is missing (the memory system of record)"

# E2.2 — layer A: bounded resume pointer present and within BOTH caps.
# ⚠️ DISTINCT NAMES, DELIBERATELY. These two values are read again at the very end of the script
# (the headroom report), so they must survive every loop in between. The obvious short names `n`
# and `b` do NOT: the layer-C reconcile loop below rebinds `b="$(basename "$f")"`, and a first cut
# of the report duly tried to divide a decision-record filename by the byte cap — printing a bash
# arithmetic error on stderr while still exiting 0, i.e. failing in the passing direction.
layer_a_lines=""; layer_a_bytes=""
if [ -f MEMORY.md ]; then
  layer_a_lines=$(wc -l < MEMORY.md | tr -d ' ')
  layer_a_bytes=$(wc -c < MEMORY.md | tr -d ' ')
  n="$layer_a_lines"; b="$layer_a_bytes"
  [ "$n" -le "$CAP" ] || note "MEMORY.md is $n lines (> cap $CAP) — it is layer A (resume pointer); demote content to docs/tasks/ (B) or docs/decisions/ (C)"
  # The byte cap is what makes the line cap mean something: 60 lines carried 138,403 bytes here.
  [ "$b" -le "$BYTE_CAP" ] || note "MEMORY.md is $b bytes (> cap $BYTE_CAP) — long lines bypass the line cap; demote content to docs/tasks/ (B) or docs/decisions/ (C). Do NOT raise the cap to fit the content."
else
  note "MEMORY.md (layer A resume pointer) is missing"
fi

# E2.3 — bootstrap pointers (E1): present and pointing at the system of record.
for f in AGENTS.md CLAUDE.md; do
  if [ -f "$f" ]; then
    grep -q "MEMORY_ARCHITECTURE.md" "$f" || note "$f does not point at MEMORY_ARCHITECTURE.md"
  else
    note "$f bootstrap pointer is missing"
  fi
done

# E2.4 — layer B: the task-tree system exists.
[ -f docs/TASK_TREE.md ] || note "docs/TASK_TREE.md (layer B index) is missing"
[ -d docs/tasks ] || note "docs/tasks/ (layer B task-trees) is missing"

# E2.5 — layer C: decisions dir + index present, and the index RECONCILES with the records.
#
# ⛔ WHAT THIS REPLACED, AND WHY (README-POLICY.7): the previous form asserted only that
# INDEX.md had MORE THAN ZERO rows while records existed. It therefore passed at
# 135 records / 133 index rows — two records invisible to layer C's own index with the
# doctrine fully green. That is the same disease as the layer-A line-only cap one check
# above: a bound satisfied without binding.
#
# ⭐ PROVENANCE — ADOPTED, NOT DESIGNED, AND THE FLOW RAN BACKWARDS. The working
# implementation already existed in the portable spine repo this project extracted its
# discipline INTO, whose documented transfer direction is one-way (reference deployment ->
# spine). Generalizing a check is a REWRITE, not a copy, so the neutral version can come out
# stronger — it did. Found only because a port happened to open that file.
#
# ⭐⭐ Two deliberate strengthenings over the adopted version, both measured here:
#   (1) ROW-ANCHORED, not a bare basename grep. A basename match anywhere in INDEX.md
#       false-passes a record that is merely MENTIONED in another row's prose. Measured:
#       `project_json_full_standard_proof.md` already occurs TWICE (its own row + a prose
#       link in a neighbouring row), so the shape that hides a missing row is present today
#       even though no record currently exploits it.
#   (2) BOTH DIRECTIONS. The adopted version is honest that it is one-directional (a record
#       with no row, never a row with no record). A row pointing at a deleted record is an
#       index that lies in the other direction, so it is checked too.
#
# ⚠️ Not self-referential (reference_self_referential_assertion_is_unsound.md): every
# assertion here is about a file OTHER than the one the assertion lives in, and INDEX.md is
# excluded from the record loop. ⚠️ grep reads the FILE directly — never
# `printf "$var" | grep -q`, which returns failure ON SUCCESS past the pipe buffer under
# `pipefail` (README-POLICY.6).
if [ -d docs/decisions ]; then
  if [ ! -f docs/decisions/INDEX.md ]; then
    note "docs/decisions/INDEX.md (layer C index) is missing"
  else
    # Forward: every record must have its OWN ROW in the index.
    for f in docs/decisions/*.md; do
      [ -e "$f" ] || continue
      b="$(basename "$f")"
      case "$b" in INDEX.md|TEMPLATE.md) continue ;; esac
      b_re="$(printf '%s' "$b" | sed 's/\./\\./g')"
      grep -qE "^\| \[$b_re\]\($b_re\)" docs/decisions/INDEX.md \
        || note "layer C: record $b has NO row in docs/decisions/INDEX.md (a record nothing indexes is unreachable by topic)"
    done
    # Reverse: every indexed row must point at a record that exists.
    while IFS= read -r row; do
      [ -n "$row" ] || continue
      [ -f "docs/decisions/$row" ] \
        || note "layer C: INDEX.md has a row for $row, which does not exist (the index names a record that is gone)"
    done < <(sed -nE 's/^\| \[([A-Za-z0-9_.-]+\.md)\]\(\1\).*/\1/p' docs/decisions/INDEX.md)
  fi
else
  note "docs/decisions/ (layer C) is missing"
fi

# E2.6 — DERIVED-STATE CONTAINMENT on layer A (docs/DERIVED_STATE_CONTAINMENT.md R1/R3).
#
# ⛔ THE DEFECT THIS EXISTS FOR, MEASURED (LIVE-DOC-CONTAINMENT.2, 2026-08-08): layer A carried
# `**push**: 185 ahead`. `git rev-list --count origin/main..HEAD` said 187 when the repair began
# and 189 when it landed — it drifted TWICE during its own repair. That field is not merely
# drift-PRONE, it is wrong BY CONSTRUCTION: recording "N ahead" requires a commit, which makes it
# N+1. FOUR commits' subject lines were that counter. No cadence or reminder can fix arithmetic;
# only deletion can, so this check enforces the deletion rather than trusting the discipline.
#
# ⭐ DECLARED, NEVER INFERRED (§6): the pattern list is explicit and reviewable. A heuristic that
# tried to guess which prose is a commit count would fail OPEN — silently, in the passing
# direction — which is the failure mode every other bound in this file was written to escape.
#
# ⭐ R3/R5: the fix is never "delete the field". It is "replace the field with its DERIVATION", so
# a reader still gets the answer — and gets one that is correct at the moment they ask.
DERIVED_STATE_SURFACES="${DERIVED_STATE_SURFACES:-MEMORY.md}"
derived_state_patterns=(
  # (1) field-name form: a layer-A bullet whose KEY names a git-owned quantity.
  '^[[:space:]]*[-*][[:space:]]*\*\*(push|unpushed|commits_ahead|commits_behind|latest_commit|current_commit|head_commit)\*\*'
  # (2) value form: a stored commit DISTANCE, caught even if the field is renamed.
  '[0-9]+[[:space:]]+(commits?[[:space:]]+)?(ahead|behind)([[:space:]]|[.,;)]|$)'
)
derived_state_hits_in_file(){ # $1=file -> prints matching "pattern|line" pairs
  local f="$1" p
  for p in "${derived_state_patterns[@]}"; do
    grep -nE "$p" "$f" 2>/dev/null | while IFS= read -r l; do printf '%s\n' "$l"; done
  done
}
derived_state_hits_in_text(){ # $1=text -> 0 if any pattern matches (herestring: no SIGPIPE, cf. README-POLICY.6)
  local s="$1" p
  for p in "${derived_state_patterns[@]}"; do
    grep -qE "$p" <<< "$s" && return 0
  done
  return 1
}

# GROUND TRUTH FIRST — an instrument with no ground truth is a confident guess, and this one is
# cheap enough to prove on every run rather than once in a throwaway probe.
ds_ctrl_ok=1
# POSITIVE control: a compliant pointer (derivation, not value) must NOT match.
derived_state_hits_in_text '- **active_work_unit**: `SOME-TREE` -> frontier `.2`
> unpushed count is DERIVED: `git rev-list --count origin/main..HEAD`' && ds_ctrl_ok=0
# NEGATIVE controls: both the real pre-fix shapes MUST match.
derived_state_hits_in_text '- **push**: 185 ahead - RE-DERIVE, never increment.' || ds_ctrl_ok=0
derived_state_hits_in_text '- **unpushed_commits**: the tree is 42 commits ahead of origin.' || ds_ctrl_ok=0
if [ "$ds_ctrl_ok" -ne 1 ]; then
  note "derived-state controls MISSED (positive matched, or a negative did not) — the scanner is broken; refusing to publish a verdict on it"
else
  for surface in $DERIVED_STATE_SURFACES; do
    [ -f "$surface" ] || continue
    while IFS= read -r hit; do
      [ -n "$hit" ] || continue
      note "derived-state: $surface:${hit%%:*} stores a field git owns EXACTLY — \"$(printf '%s' "${hit#*:}" | cut -c1-72)\". Delete it and leave the DERIVATION in its place (docs/DERIVED_STATE_CONTAINMENT.md R1/R3). Do NOT schedule periodic correction: writing the value invalidates it."
    done < <(derived_state_hits_in_file "$surface" | sort -t: -k1,1n -u)
  done
fi

# ⭐ REPORT HEADROOM, NOT JUST COMPLIANCE (README-POLICY.8, implementing this repository's own
# lesson card `a-cap-with-no-headroom-is-a-cap-about-to-be-raised`): "a cap answers 'am I
# compliant?', which is binary and lagging — the question that predicts the next failure is 'how
# much room is left?'". Layer A sat at >= 97% of its byte cap for 36 of 40 consecutive commits and
# nothing said so, because a passing gate printed the same three characters at 5,720 bytes as at
# 7,168. This is REPORTING ONLY — it adds no failure path and cannot change any verdict, which is
# why it is safe to run on every commit; the caps above remain the sole gate.
if [ "$fail" -eq 0 ]; then
  if [ -n "$layer_a_bytes" ] && [ -n "$layer_a_lines" ]; then
    printf 'memory-arch: OK (layer A %s/%s bytes = %d%% of cap, %s/%s lines = %d%% of cap)\n' \
      "$layer_a_bytes" "$BYTE_CAP" "$(( layer_a_bytes * 100 / BYTE_CAP ))" \
      "$layer_a_lines"  "$CAP"      "$(( layer_a_lines * 100 / CAP ))"
  else
    echo "memory-arch: OK"
  fi
fi
exit $fail
