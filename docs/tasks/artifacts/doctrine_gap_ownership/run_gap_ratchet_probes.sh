#!/usr/bin/env bash
# docs/tasks/artifacts/doctrine_gap_ownership/run_gap_ratchet_probes.sh
# DOCTRINE-GAP-OWNERSHIP.1 — RED / GREEN / CONTROL arms for the triage ratchet.
#
# ⛔ WHY THIS EXISTS: a gate nobody has watched FAIL is not a gate. `.1`'s deliverable
# is "a tightened driver whose ORPHAN count is MEANINGFUL" — meaningful requires that
# (a) an untriaged gap BLOCKS, and (b) the tightenings did not silently start
# swallowing real gaps. RED arms prove (a); CONTROL arms prove (b).
#
# Every arm runs against a HERMETIC synthetic repo (PGEN_GAP_SWEEP_ROOT), so no arm
# can mutate the real tree — the failure mode where a probe's own edits get committed.
#
# CONTROL arms 8 and 9 are regression pins for two false positives MEASURED while
# building this driver, each of which had silently marked a real defect as clean:
#   8. "whack-a-mole" matching HACK (grammars/systemverilog.ebnf:426)
#   9. a test literal ("xxx", true) matching a bare XXX marker (4 sites)
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
SWEEP="$ROOT/docs/tasks/artifacts/doctrine_gap_ownership/gap_ownership_sweep.py"
WORK="$ROOT/rust/target/doctrine_checks/gap_probe"
PASS=0; FAIL=0

# build a hermetic synthetic repo; $1 = arm name
mk_repo() {
  local d="$WORK/$1"
  rm -rf "$d"; mkdir -p "$d/docs/tasks/artifacts/doctrine_gap_ownership" "$d/grammars" "$d/rust/src"
  git -C "$d" init -q
  git -C "$d" config user.email probe@local
  git -C "$d" config user.name probe
  : >"$d/docs/tasks/artifacts/doctrine_gap_ownership/gap_triage_register.tsv"
  echo "$d"
}
commit_all() { git -C "$1" add -A >/dev/null 2>&1; }

# $1 arm, $2 expected exit, $3 description
run_arm() {
  local d="$1" want="$2" desc="$3"
  commit_all "$d"
  PGEN_GAP_SWEEP_ROOT="$d" python3 "$SWEEP" --check >"$d/out.txt" 2>&1
  local got=$?
  if [ "$got" = "$want" ]; then
    printf '  ✅ %-58s exit=%s (expected %s)\n' "$desc" "$got" "$want"; PASS=$((PASS+1))
  else
    printf '  ❌ %-58s exit=%s (expected %s)\n' "$desc" "$got" "$want"; FAIL=$((FAIL+1))
    sed 's/^/       /' "$d/out.txt" | head -14
  fi
}

echo "=== DOCTRINE-GAP-OWNERSHIP.1 — triage-ratchet probes ==="

# ---------------------------------------------------------------- RED 1
d=$(mk_repo red1)
printf '# Notes\n\nSome prose.\n\nThis is a known limitation nobody owns.\n' >"$d/NOTES.md"
run_arm "$d" 1 "RED 1: a new unowned, undated gap phrase BLOCKS"

# ---------------------------------------------------------------- RED 2
d=$(mk_repo red2)
printf '# Notes\n\nThis is a known limitation nobody owns.\n' >"$d/NOTES.md"
commit_all "$d"
key=$(PGEN_GAP_SWEEP_ROOT="$d" python3 "$SWEEP" 2>/dev/null | awk '/^  [0-9a-f]{12}  /{print $1; exit}')
printf '%s\tNOT-A-REAL-DISPOSITION\troot-docs\tsomething\tnote\n' "$key" \
  >"$d/docs/tasks/artifacts/doctrine_gap_ownership/gap_triage_register.tsv"
run_arm "$d" 1 "RED 2: a non-terminal disposition BLOCKS"

# ---------------------------------------------------------------- RED 3
d=$(mk_repo red3)
printf '# Notes\n\nThis is a known limitation nobody owns.\nAnd here is a second known gap, also unowned.\n' >"$d/NOTES.md"
commit_all "$d"
PGEN_GAP_SWEEP_ROOT="$d" python3 "$SWEEP" 2>/dev/null \
  | awk '/^  [0-9a-f]{12}  /{print $1}' | head -1 \
  | while read -r k; do printf '%s\tFALSE-POSITIVE\troot-docs\treason\tnote\n' "$k"; done \
  >"$d/docs/tasks/artifacts/doctrine_gap_ownership/gap_triage_register.tsv"
run_arm "$d" 1 "RED 3: register covering only SOME residue still BLOCKS"

# ---------------------------------------------------------------- GREEN 4
d=$(mk_repo green4)
printf '# Notes\n\nThis is a known limitation nobody owns.\n' >"$d/NOTES.md"
commit_all "$d"
PGEN_GAP_SWEEP_ROOT="$d" python3 "$SWEEP" 2>/dev/null \
  | awk '/^  [0-9a-f]{12}  /{printf "%s\tFALSE-POSITIVE\troot-docs\treason\tnote\n",$1}' \
  >"$d/docs/tasks/artifacts/doctrine_gap_ownership/gap_triage_register.tsv"
run_arm "$d" 0 "GREEN 4: every residue row triaged PASSES"

# ---------------------------------------------------------------- CONTROL 5
d=$(mk_repo ctrl5)
printf '# Changelog\n\n## 2026-04-30 — some release\n\nThis is a known limitation, not fixed here.\n' >"$d/CHANGES.md"
run_arm "$d" 0 "CTRL 5: gap under a DATED heading = PROVENANCE, no block"

# ---------------------------------------------------------------- CONTROL 6
d=$(mk_repo ctrl6)
printf '# Notes\n\nThis is a known limitation, routed to SOMETREE.4 for the fix.\n' >"$d/NOTES.md"
run_arm "$d" 0 "CTRL 6: gap WITH a named owner = OWNED, no block"

# ---------------------------------------------------------------- CONTROL 7
d=$(mk_repo ctrl7)
printf '# Walking\n\n```rust\n// handle them so future slices do not break the walker\n```\n' >"$d/NOTES.md"
run_arm "$d" 0 "CTRL 7: gap phrase inside a fenced block = FENCE, no block"

# ---------------------------------------------------------------- CONTROL 8
d=$(mk_repo ctrl8)
printf 'rule := "a"\n# patch each corrupt token is unbounded whack-a-mole. Strategy 1\n' >"$d/grammars/g.ebnf"
run_arm "$d" 0 "CTRL 8: 'whack-a-mole' is NOT a HACK marker (regression pin)"

# ---------------------------------------------------------------- CONTROL 9
d=$(mk_repo ctrl9)
printf 'fn t() { let inputs = &[("", true), ("xxx", true), ("xy", false)]; }\n' >"$d/rust/src/t.rs"
run_arm "$d" 0 "CTRL 9: test literal \"xxx\" is NOT an XXX marker (regression pin)"

# ---------------------------------------------------------------- CONTROL 10
d=$(mk_repo ctrl10)
printf 'rule := "todo" | "fixme" | "bug"\n' >"$d/grammars/semantic.ebnf"
run_arm "$d" 0 "CTRL 10: quoted grammar TERMINALS are not gap markers"

# ---------------------------------------------------------------- RED 11
d=$(mk_repo red11)
printf 'fn f() { /* TODO: implement the thing */ }\n' >"$d/rust/src/t.rs"
run_arm "$d" 1 "RED 11: a real unowned TODO in Rust source BLOCKS"

echo
echo "probes: PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
