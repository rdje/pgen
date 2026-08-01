#!/usr/bin/env bash
# docs/tasks/artifacts/generated_lint_correctness/run_diag_evidence_root_kw_probes.sh
#
# GENERATED-LINT-CORRECTNESS.9 — RED / GREEN / CONTROL probes for the ROOT_KW NARROWING in
# scripts/check_diagnosis_evidence.sh (`\bwhy\b` dropped; `why ?[-+/&] ?where` kept).
#
# ⭐ THE MISBEHAVIOUR, AND WHY IT IS THE WORST CLASS. `ROOT_KW`'s third alternative matched the
# BARE WORD "why". A `**FIX**` box routinely writes prose such as "Why no lower tier: …" and
# routinely quotes a command, so its header matched the ROOT CAUSE keyword and its body carried a
# diagnosis signature — and a leaf holding NO ROOT CAUSE box at all passed box 1 on its FIX box.
# That is FAILS-OPEN, on one of the director's four named steps. RED-W1 is that hole, reproduced.
#
# ⭐ THE SAME ALTERNATIVE ALSO FAILED CLOSED, on the other polarity. `unchecked()` blocks on an
# UNTICKED box matching the keyword wherever it sits, so an unticked `**FIX**` box mentioning
# "why" blocked a leaf whose real ROOT CAUSE / ADDRESSED / NO REGRESSION boxes were all complete
# and backed. RED-W2 is that false block. One narrowing closes both polarities.
#
# ⛔ THE POINT OF THE GREEN ARMS. A narrowing must not lose a spelling authors actually use. The
# template spelling `**ROOT CAUSE (WHY + WHERE)**` is the dominant one (backed by the corpus
# census: 409 of 413 ticked headers already match the candidate), but the hand-written connector
# variants `/`, `-`, `&` are equally honest and are kept deliberately.
#
# Arms (verdicts asserted are the POST-fix ones):
#   RED-W1  FIX box says "why" + quotes a tool; NO ROOT CAUSE box    -> BLOCK (PASSED before)
#   RED-W2  complete backed checklist + an UNTICKED FIX box saying "why"
#                                                                    -> ALLOW (BLOCKED before)
#   GREEN-1 the template spelling `**ROOT CAUSE (WHY + WHERE)**`     -> ALLOW
#   GREEN-2 `**ROOT CAUSE**` alone (the `root cause` alternative)    -> ALLOW
#   GREEN-3 connector `/`  — `**ROOT CAUSE (WHY / WHERE)**`          -> ALLOW
#   GREEN-4 connector `-`  — `**ROOT CAUSE (WHY - WHERE)**`          -> ALLOW
#   GREEN-5 connector `&`  — `**WHY & WHERE**` with no "root cause"  -> ALLOW
#   CTRL-1  real ROOT CAUSE box AND a FIX box saying "why"           -> ALLOW (not punitive)
#   CTRL-2  the real ROOT CAUSE box is UNTICKED                      -> BLOCK (unchanged)
#   CTRL-3  ROOT CAUSE ticked but the signature sits outside the box  -> BLOCK (unchanged, `.3`)
#
# Before->after replay (produced, not asserted):
#   git show HEAD:scripts/check_diagnosis_evidence.sh > rust/target/head_check.sh
#   PGEN_DIAG_CHECK_OVERRIDE=rust/target/head_check.sh bash <this script>
# Under the pre-`.9` enforcer RED-W1 MUST pass (the fails-open hole) and RED-W2 MUST block (the
# fails-closed twin), while every GREEN/CTRL arm keeps its verdict; if a RED arm already had its
# post-fix verdict there, the narrowing bought nothing.
#
# Scratch lives under rust/target/ (repo volume) per the project data-locality policy.
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
CHECK="${PGEN_DIAG_CHECK_OVERRIDE:-${ROOT_DIR}/scripts/check_diagnosis_evidence.sh}"
[ "${CHECK#/}" = "$CHECK" ] && CHECK="${ROOT_DIR}/${CHECK}"
WORK="${ROOT_DIR}/rust/target/doctrine_checks/diag_evidence_root_kw_probes"

rm -rf "${WORK}"; mkdir -p "${WORK}"
pass_count=0; fail_count=0

# The two boxes that are NOT under test here, written compliant so every arm isolates ROOT_KW.
tail_boxes() {
  printf -- '- [x] **ADDRESSED (verified)** — UNKNOWN 3 -> 0 on the same command.\n'
  printf -- '- [x] **NO REGRESSION** — cert deterministic at seeds 0/7/42, fully_certified.\n'
}

new_repo() {
  local d="${WORK}/$1"
  rm -rf "$d"; mkdir -p "$d/docs/tasks" "$d/rust/src" "$d/scripts"
  git -C "$d" init -q
  git -C "$d" config user.email probe@pgen.local
  git -C "$d" config user.name probe
  cp "${CHECK}" "$d/scripts/check_diagnosis_evidence.sh"
  # Committed, not staged: `.5` counts scripts/check_*.sh as a code change, so a staged copy would
  # make every arm look like a code change and break the "no code change staged" reasoning.
  git -C "$d" add scripts/check_diagnosis_evidence.sh >/dev/null 2>&1
  git -C "$d" commit -qm "probe base: the enforcer itself" >/dev/null 2>&1
  printf '%s' "$d"
}

code_change() { printf 'fn main() {}\n' >"$1/rust/src/lib.rs"; }

run_arm() {
  # run_arm <name> <expected_exit> <expected_substring> <repo_dir>
  local name="$1" expect_rc="$2" expect_msg="$3" d="$4" rc=0
  local log="${WORK}/${name}.log"
  ( cd "$d" && bash scripts/check_diagnosis_evidence.sh ) >"$log" 2>&1 || rc=$?
  local ok_msg="yes"
  [ -n "$expect_msg" ] && { grep -qF -- "$expect_msg" "$log" || ok_msg="no"; }
  if [ "$rc" = "$expect_rc" ] && [ "$ok_msg" = "yes" ]; then
    printf 'PROBE %-8s PASS  exit=%s (expected %s)\n' "$name" "$rc" "$expect_rc"
    pass_count=$((pass_count + 1))
  else
    printf 'PROBE %-8s FAIL  exit=%s (expected %s, msg_match=%s)\n' "$name" "$rc" "$expect_rc" "$ok_msg"
    sed 's/^/  /' "$log" | tail -n 12
    fail_count=$((fail_count + 1))
  fi
}

MISSING="ROOT CAUSE (WHY+WHERE) box is MISSING"
UNTICKED="UNTICKED"
NOT_IN_BOX="NOT INSIDE THAT BOX"
OK_MSG="OK (task leaf passes"

# ------------------------------------ RED ARMS ------------------------------------------------
# W1 — the fails-open hole, verbatim in the shape the corpus actually carries: a FIX box whose
# prose says "Why no lower tier" and whose body quotes a real tool. There is no ROOT CAUSE box.
d="$(new_repo redw1)"
{ printf '# TREE-A\n\n### `.1` — the owning leaf\n\n#### Acceptance Checklist (enforced)\n\n'
  printf -- '- [x] **FIX** — grammar tier: the branch predicate moved to the rule. Why no lower\n'
  printf -- '      tier: the declarative surface cannot express it. Confirmed with `--trace-rules`\n'
  printf -- '      that the rejection disappears.\n'
  tail_boxes; } >"$d/docs/tasks/TREE-A.md"
code_change "$d"; git -C "$d" add -A >/dev/null 2>&1
run_arm RED-W1 1 "$MISSING" "$d"

# W2 — the fails-CLOSED twin. Everything required is present, ticked and backed; the only
# "unticked" box is a FIX box that happens to write the word "why".
d="$(new_repo redw2)"
{ printf '# TREE-A\n\n### `.1` — the owning leaf\n\n#### Acceptance Checklist (enforced)\n\n'
  printf -- '- [x] **ROOT CAUSE (WHY + WHERE)** — `--report-certificate-coverage` UNKNOWN=3; the\n'
  printf -- '      [plannable-probe] verdict named the rule.\n'
  tail_boxes
  printf -- '- [ ] **FIX** — not yet chosen. Why the engine tier is likely: the grammar tier\n'
  printf -- '      cannot express the constraint.\n'; } >"$d/docs/tasks/TREE-A.md"
code_change "$d"; git -C "$d" add -A >/dev/null 2>&1
run_arm RED-W2 0 "$OK_MSG" "$d"

# ----------------------------------- GREEN ARMS -----------------------------------------------
green_arm() {
  # green_arm <name> <root-cause-header-text>
  local name="$1" hdr="$2" d
  d="$(new_repo "${name,,}")"
  { printf '# TREE-A\n\n### `.1` — the owning leaf\n\n#### Acceptance Checklist (enforced)\n\n'
    printf -- '- [x] %s — `--report-certificate-coverage` UNKNOWN=3; the [plannable-probe]\n' "$hdr"
    printf -- '      verdict named the rule.\n'
    tail_boxes; } >"$d/docs/tasks/TREE-A.md"
  code_change "$d"; git -C "$d" add -A >/dev/null 2>&1
  run_arm "$name" 0 "$OK_MSG" "$d"
}

green_arm GREEN-1 '**ROOT CAUSE (WHY + WHERE)**'
green_arm GREEN-2 '**ROOT CAUSE**'
green_arm GREEN-3 '**ROOT CAUSE (WHY / WHERE)**'
green_arm GREEN-4 '**ROOT CAUSE (WHY - WHERE)**'
green_arm GREEN-5 '**WHY & WHERE**'

# ---------------------------------- CONTROL ARMS ----------------------------------------------
# CTRL-1 — the narrowing must not be punitive: the everyday leaf carries BOTH a real ROOT CAUSE
# box and a FIX box whose prose says "why". This passed before and must still pass.
d="$(new_repo ctrl1)"
{ printf '# TREE-A\n\n### `.1` — the owning leaf\n\n#### Acceptance Checklist (enforced)\n\n'
  printf -- '- [x] **ROOT CAUSE (WHY + WHERE)** — `--trace-rules` named the rejecting predicate\n'
  printf -- '      in `foo_rule`.\n'
  printf -- '- [x] **FIX** — declarative tier. Why no lower tier: there is none below declarative.\n'
  tail_boxes; } >"$d/docs/tasks/TREE-A.md"
code_change "$d"; git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-1 0 "$OK_MSG" "$d"

# CTRL-2 — an UNTICKED real ROOT CAUSE box must still block (the narrowing touches the keyword,
# never the unticked polarity for boxes that genuinely are ROOT CAUSE boxes).
d="$(new_repo ctrl2)"
{ printf '# TREE-A\n\n### `.1` — the owning leaf\n\n#### Acceptance Checklist (enforced)\n\n'
  printf -- '- [ ] **ROOT CAUSE (WHY + WHERE)** — pending; `--trace-rules` not yet run.\n'
  tail_boxes; } >"$d/docs/tasks/TREE-A.md"
code_change "$d"; git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-2 1 "$UNTICKED" "$d"

# CTRL-3 — `.3`'s box-scoping is untouched: the signature must still sit inside the ticked box.
d="$(new_repo ctrl3)"
{ printf '# TREE-A\n\n### `.1` — the owning leaf\n\n#### Acceptance Checklist (enforced)\n\n'
  printf -- '- [x] **ROOT CAUSE (WHY + WHERE)** — we investigated and are confident.\n'
  tail_boxes
  printf '\nNarrative outside the box: `--report-certificate-coverage` UNKNOWN=0.\n'; } >"$d/docs/tasks/TREE-A.md"
code_change "$d"; git -C "$d" add -A >/dev/null 2>&1
run_arm CTRL-3 1 "$NOT_IN_BOX" "$d"

printf '\nprobes: %d passed, %d failed  (enforcer: %s)\n' "$pass_count" "$fail_count" "$CHECK"
[ "$fail_count" -eq 0 ] || exit 1
exit 0
