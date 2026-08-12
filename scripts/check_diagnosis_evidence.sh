#!/usr/bin/env bash
# scripts/check_diagnosis_evidence.sh
# DIAG-TOOLBOX-ENFORCE (PGEN-DIAG-TOOLBOX-0002/0003): make a fix PROVABLY follow the
# task-acceptance procedure from start to finish — not "trust me". Per the director directive
# (2026-06-22): "every task tree shall check some boxes, [go] through certain steps to analyse
# an issue, and made sure the issue it wanted to address was clearly addressed with no
# regression." This gate enforces a REQUIRED ACCEPTANCE CHECKLIST in the owning task leaf for any
# code change: each required box must be TICKED ([x]) and backed by real tool-output evidence; an
# unticked or missing required box BLOCKS the commit. Exits NONZERO on any breach.
#
# The required checklist (label keywords are flexible; the [x] and the keyword are what matter):
#   - [x] ROOT CAUSE (WHY + WHERE) ........ backed by a DIAGNOSIS tool signature, from any of the
#                                           FIVE families: (1) correctness — cert/probe/trace/
#                                           reach/lint; (2) performance/SPEED — a profiler
#                                           (`/usr/bin/sample`, `otool -tV`, flamegraph self-time
#                                           + call-graph); (3) build-integrity — a rustc
#                                           `error[EXXXX]`; (4) codegen-emission — a real
#                                           `clippy::<lint>` over the generated artifacts;
#                                           (5) ops/build-flow — a verbatim shell/git/make/errno
#                                           invocation for a defect in the repo's own scripts,
#                                           Makefiles, hooks or tracking state.
#   - [x] ADDRESSED (verified)  ........... the issue is resolved (before->after on the symptom)
#   - [x] NO REGRESSION ................... backed by a global-gate signature (seeds 0/7/42, etc.)
# (REPRODUCE/FIX/LOCKSTEP boxes are recommended by the template but not hard-required here, to
# avoid false-blocking; the three above are the director's named steps and ARE required.)
#
# Why "not trust-me-bro": (1) a ticked box must co-occur with the real tool-output signature that
# only the debug tools / deterministic gates emit; (2) the project's DETERMINISTIC gates
# (cert-coverage at seeds 0/7/42, ast_shape_contract, syntax-closure, external-corpus) re-RUN the
# real tools in the make gates / CI, so a fabricated "NO REGRESSION" claim does not reproduce and
# fails there. Honest limit: a hook cannot prove the author reasoned — only that the boxes are
# ticked, the evidence is present, and (via the oracle gates) the numbers reproduce.
#
# Called by scripts/check_doctrines.sh (the general enforcer) via .githooks/pre-commit (E3) + CI
# (E4). Recorded exception (loud, never silent): PGEN_DIAG_EVIDENCE_WAIVER="<reason>".
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"

if [ -n "${PGEN_DIAG_EVIDENCE_RANGE:-}" ]; then
  mapfile -t staged < <(git diff --name-only --diff-filter=ACMR "$PGEN_DIAG_EVIDENCE_RANGE")
else
  mapfile -t staged < <(git diff --cached --name-only --diff-filter=ACMR)
fi

# Is this a CODE change? (grammars, rust sources, generated parsers, AST shape-contract manifests)
#
# ⭐⭐ THE PROOF SURFACE IS PART OF THE CODE (GENERATED-LINT-CORRECTNESS.5, 2026-07-27, on a direct
# director order: *"You are the guarantor of the integrity of the repository ... that all rules,
# doctrines of the project are strictly followed to the T"*).
#
# MEASURED across all 2,618 commits before widening: **521 touched the proof surface** (the gates,
# the doctrine enforcers, the Makefiles that wire them, the git hooks, the CI workflows) and
# **397 of those 521 (76%) staged no path from the old list**, so this check answered "no code
# change staged" and required NOTHING of them.
#
# ⛔ That is not a small hole; it is the hole the last three sessions kept falling into. EVERY rot
# class found in sessions #214-#216 lives on that surface and on no other:
#   - `ast_dump_contract_gate` RED since #212, belonging to no aggregate  (a gate script)
#   - `PGEN_CLIPPY_GENERATED_STRICT` set by NOTHING, defaulting to 0      (gate + Makefile wiring)
#   - `ci_workflow_local_gate` unable to complete for 1,371 commits       (a gate script)
# A check that guards the parser but not the machinery deciding whether the parser is ever checked
# is guarding the wrong thing. **A change to a gate is a change to what "verified" MEANS.**
#
# ⚠️ SCOPED DELIBERATELY NARROWER THAN "anything under scripts/". Only the machinery that decides
# whether other checks run is in scope; ordinary tooling, corpora and helper scripts are not. The
# ops/build-flow diagnosis family added by `.4` is what makes this satisfiable rather than
# punitive: such a defect can now be root-caused with `git ls-files` / `make -n` / `shellcheck` /
# an errno, so an author is never forced to waive an otherwise-correct gate.
code_changed=0
for f in "${staged[@]:-}"; do
  case "$f" in
    # the parser itself
    grammars/*.ebnf|rust/src/*|generated/*|rust/test_data/ast_shape_contract/*.json) code_changed=1 ;;
    # the proof surface: the doctrine enforcers, the gates, their wiring, the hooks, CI
    scripts/check_*.sh|rust/scripts/*.sh|.githooks/*|.github/workflows/*.yml|rust/build.rs) code_changed=1 ;;
    Makefile|rust/Makefile) code_changed=1 ;;
  esac
done

if [ "$code_changed" -eq 0 ]; then
  echo "diag-evidence: OK (no code change staged; task-acceptance checklist not required)"
  exit 0
fi

if [ -n "${PGEN_DIAG_EVIDENCE_WAIVER:-}" ]; then
  printf 'diag-evidence: ⚠️ WAIVED by PGEN_DIAG_EVIDENCE_WAIVER="%s" — exception recorded; CI still re-checks.\n' \
    "$PGEN_DIAG_EVIDENCE_WAIVER" >&2
  exit 0
fi

# A code change must be owned by a staged task-tree leaf carrying the acceptance checklist.
mapfile -t staged_tasks < <(printf '%s\n' "${staged[@]:-}" | grep -E '^docs/tasks/.*\.md$' || true)
if [ "${#staged_tasks[@]}" -eq 0 ]; then
  cat >&2 <<'MSG'
diag-evidence: ✗ a CODE change is staged but NO owning task-tree leaf (docs/tasks/*.md) is staged.
  Stage the owning docs/tasks/<TREE>.md carrying the ACCEPTANCE CHECKLIST (TOOLBOX.md template).
MSG
  exit 1
fi

# Scan the staged leaves DIRECTLY (grep reads the files itself). Do NOT do
# `printf '%s\n' "$leaf_text" | grep -q` — under `set -o pipefail`, `grep -q` exits on the first
# match and closes the pipe, the upstream `printf` then takes SIGPIPE (rc 141), and pipefail
# propagates that as the pipeline's status, so a genuinely-present box is reported as MISSING once
# the concatenated leaf text exceeds the pipe buffer (~64 KB) and the match is early. Grepping the
# files directly has no upstream writer to kill, so the match is deterministic at any file size.
# `grep -q` over multiple files returns 0 iff ANY line in ANY file matches — identical to scanning
# the concatenation. (STORE-AWARE-GEN.4b.12: this race began false-failing once the owning task
# leaf grew past ~64 KB.)
#
# ⭐ BOX-SCOPED EVIDENCE (GENERATED-LINT-CORRECTNESS.3, 2026-07-27). The signature that BACKS a
# ticked box is now required to sit INSIDE THAT BOX'S OWN BULLET, not merely somewhere in the
# staged task files. This closes two MEASURED soundness holes that made the check weaker than it
# reads:
#   (1) cross-FILE leakage — the greps ran over ALL staged `docs/tasks/*.md`, so a co-staged,
#       unrelated tree file could supply the signature for a leaf that carried none. Measured:
#       that is exactly how `GENERATED-LINT-CORRECTNESS.1` passed, on tokens belonging to
#       `docs/tasks/QUANT-PLUS-ITER.md`.
#   (2) incidental-PROSE leakage — a whole-file grep matched a token mentioned anywhere in the
#       leaf rather than in the ticked box. Recorded as a known gap and deferred in
#       docs/decisions/project_build_integrity_compiler_root_cause_signature.md ("Watch item":
#       *"the underlying looseness (whole-file grep, not box-scoped) is a known soundness gap in
#       all three signature groups and is worth a future hardening slice — scope the grep to the
#       ticked box's own bullet"*). This IS that hardening slice; one mechanism closes both.
# Rationale + the RED/GREEN evidence: docs/decisions/project_codegen_emission_root_cause_signature.md.
#
# A box's BODY runs from its `- [x] …` line up to (but not including) the next checklist box at
# the same-or-shallower indent, or the next markdown heading, or EOF. More-indented nested boxes
# belong to the parent body — they are the author's own elaboration of that box.

# Repo-volume scratch (project data-locality policy: never /tmp when a repo-derived path works).
WORK=""
for cand in "$ROOT/rust/target/doctrine_checks" "${TMPDIR:-/tmp}"; do
  if mkdir -p "$cand" 2>/dev/null && [ -w "$cand" ]; then
    WORK="$(mktemp -d "$cand/diag_evidence.XXXXXX" 2>/dev/null || true)"
    [ -n "$WORK" ] && break
  fi
done
[ -n "$WORK" ] || { echo "diag-evidence: ✗ could not create a scratch dir" >&2; exit 1; }
trap 'rm -rf "$WORK"' EXIT

# box_body <start-line> <file> — print the box that STARTS at <start-line>, body included.
# The body ends at the next checklist box at the same-or-shallower indent, at the next markdown
# heading, or at EOF. More-indented nested boxes stay in the body: they are the author's own
# elaboration of that box.
box_body() {
  awk -v start="$1" '
    function isbox(l)     { return match(l, /^[ \t]*[-*][ \t]*\[[ xX]\][ \t]/) }
    function indent(l, i) { i = match(l, /[^ \t]/); return (i == 0 ? 0 : i - 1) }
    NR <  start { next }
    NR == start { boxind = indent($0); print; next }
    {
      if (isbox($0) && indent($0) <= boxind) exit
      if ($0 ~ /^#/) exit
      print
    }
  ' "$2"
}

# ⭐⭐ THE BOX MUST BE ONE THIS COMMIT ACTUALLY WROTE (GENERATED-LINT-CORRECTNESS.7, 2026-08-01).
#
# `.3` scoped the SIGNATURE to its own box, but never scoped the BOX to the change. The check
# still passed when ANY ticked box in ANY staged task file satisfied a requirement — so for every
# tree that already holds one compliant leaf, box-scoping was VACUOUS: a new leaf could carry no
# checklist at all and inherit a finished leaf's boxes.
#
# MEASURED, not inferred (probes H1/H2 in run_diag_evidence_leaf_scope_probes.sh):
#   H1  a new leaf with NO checklist, in the same file as one old completed leaf ....... PASSED
#   H2  the owning leaf in file B, the only backed boxes in an unrelated co-staged A ... PASSED
#   33 of the tracked task files carry >=1 backed ROOT CAUSE box, i.e. a free pass; and
#   THIS leaf's own commit would have passed box 1 on four historical boxes (.2/.3/.4/.5)
#   without writing a single line of checklist.
# Replayed over the last 400 commits: 138 code-change commits pass today, and **7 of them pass
# ONLY by borrowing** — 5 whose own NO REGRESSION box carries no gate signature and 2 that wrote
# no qualifying box at all. Under the rule below those 7 fail and the other 131 are untouched, so
# the strengthening has **0 false positives** on the measured corpus.
#
# THE RULE. A box may satisfy a requirement only if it sits inside a LEAF SECTION (bounded by
# markdown headings of level <= 3, so a `#### Acceptance Checklist` block belongs to its `###`
# leaf) that the staged change TOUCHES, and all three requirements must be met within ONE file.
# That is deliberately permissive INSIDE a leaf — a follow-up commit editing any part of the same
# leaf keeps its checklist — while closing the cross-leaf and cross-file borrow.
# ⚠️ Honest bound: the enforcer cannot know which leaf OWNS a change. "Written by this commit" is
# a proxy, not a proof; editing inside an unrelated leaf's own checklist section still satisfies it.

# added_ranges <file> — line ranges in the NEW file that the staged change adds or modifies.
# A pure-deletion hunk yields the insertion point, so deleting inside a leaf still counts as
# touching it (otherwise a docs-trimming commit would be blocked for the wrong reason).
added_ranges() {
  { if [ -n "${PGEN_DIAG_EVIDENCE_RANGE:-}" ]; then
      git diff -U0 "$PGEN_DIAG_EVIDENCE_RANGE" -- "$1"
    else
      git diff --cached -U0 -- "$1"
    fi; } 2>/dev/null | awk '
    /^@@/ { if (match($0, /\+[0-9]+(,[0-9]+)?/)) {
              s = substr($0, RSTART + 1, RLENGTH - 1); n = split(s, a, ",")
              st = a[1] + 0; ln = (n > 1 ? a[2] + 0 : 1)
              if (ln > 0) print st, st + ln - 1
              else { p = (st < 1 ? 1 : st); print p, p } } }'
}

# box_matches_in_file <file> <state> <box-keyword-regex> <required-signature-regex> <require-touched>
# True iff THIS file has a box in <state> whose HEADER LINE matches the keyword, whose own BODY
# matches the signature (when one is given), and — when <require-touched> is 1 — whose enclosing
# leaf section is touched by the staged change.
#
# The three stages are separable ON PURPOSE: the caller probes present / signature-backed /
# change-owned independently so a breach is reported at the stage it actually failed, instead of
# a ticked-but-untouched box being misreported as "MISSING".
#
# The keyword is matched against the header line only, using the same proven regex the pre-
# box-scoping version used. Matching it against the body too would let a box that merely MENTIONS
# "root cause" in its prose stand in for the real ROOT CAUSE box (found by `.3`'s own RED-2
# probe, which the first implementation of this function failed).
box_matches_in_file() {
  local f="$1" state="$2" kw="$3" sig="${4:-}" want_touch="${5:-0}" ln hdr_re rf
  [ -f "$f" ] || return 1
  if [ "$state" = "x" ]; then
    hdr_re="^[[:space:]]*[-*][[:space:]]*\[[xX]\][[:space:]].*($kw)"
  else
    hdr_re="^[[:space:]]*[-*][[:space:]]*\[[[:space:]]\][[:space:]].*($kw)"
  fi
  grep -nEi -- "$hdr_re" "$f" >"$WORK/hdr.txt" 2>/dev/null || return 1
  [ -s "$WORK/hdr.txt" ] || return 1
  # An UNTICKED box blocks wherever it is: it says the step is unfinished, and that verdict must
  # not depend on whether this commit happened to edit that part of the file.
  if [ "$state" != "x" ]; then return 0; fi
  rf="$WORK/ranges.${f//\//_}"
  [ -f "$rf" ] || added_ranges "$f" >"$rf"
  while IFS= read -r ln; do
    [ -n "$ln" ] || continue
    if [ -n "$sig" ]; then
      box_body "$ln" "$f" >"$WORK/body.txt" 2>/dev/null || continue
      grep -Eiq -- "$sig" "$WORK/body.txt" || continue
    fi
    [ "$want_touch" -eq 1 ] || return 0
    section_touched "$f" "$ln" "$rf" && return 0
  done < <(cut -d: -f1 "$WORK/hdr.txt")
  return 1
}

# section_touched <file> <box-line> <ranges-file> — is the box's enclosing leaf section (headings
# of level <= 3) intersected by any added/modified range?
#
# ⚠️ TRAILING BLANK LINES ARE TRIMMED OFF THE SECTION, and that is load-bearing, not tidiness.
# A new leaf is APPENDED, and an append begins with the blank separator line that syntactically
# still belongs to the PREVIOUS section — so without the trim, writing a brand-new leaf with no
# checklist at all "touches" the finished leaf above it and inherits its boxes. That is precisely
# the borrow this fix exists to stop, and the probe RED-H1 caught the rule being vacuous against
# the most common edit in the repository.
section_touched() {
  awk -v box="$2" -v rf="$3" '
    { L[NR] = $0; if ($0 ~ /^#{1,3}[ \t]/) H[++hn] = NR }
    END {
      s = 1; e = NR
      for (k = 1; k <= hn; k++) { if (H[k] <= box) s = H[k]; else { e = H[k] - 1; break } }
      while (e > s && L[e] ~ /^[ \t]*$/) e--
      while ((getline line < rf) > 0) {
        split(line, r, " ")
        if (r[1] + 0 <= e && r[2] + 0 >= s) exit 0
      }
      exit 1
    }' "$1"
}

# A checked / unchecked checklist box mentioning a category keyword, anywhere in the staged set.
# `unchecked` stays deliberately file-wide and section-blind — see box_matches_in_file.
unchecked() {
  local f
  for f in "${staged_tasks[@]}"; do box_matches_in_file "$f" ' ' "$1" && return 0; done
  return 1
}

# Evidence signatures that must BACK the ticked boxes.
# The first group is the CORRECTNESS-defect diagnosis toolbox (cert/probe/trace/reach/lint — "why
# does this parse WRONG / where"). The second group (added for the SPEED phase, director 2026-07-11
# — speed is now a first-class, continuously-tracked deliverable) is the PERFORMANCE-defect diagnosis
# toolbox: a slowness defect is legitimately root-caused by a PROFILER (macOS `sample`, `cargo
# flamegraph`) via self-time / call-graph attribution, NOT by a correctness tool. Tokens are kept
# tight so they cannot match unrelated text (e.g. `self-time`/`call-graph`/`flamegraph` never appear
# inside `sample_parse_failures`). Rationale + the case that motivated it (RGX-0078.3):
# docs/decisions/project_speed_phase_profiler_root_cause_signature.md.
# The third group is the BUILD-INTEGRITY diagnosis toolbox (BIN-BUILD-INTEGRITY.1, 2026-07-22): a
# target that no longer COMPILES is root-caused by the COMPILER - its error code and the exact
# file:line it names ARE the WHY+WHERE - not by a correctness tool (there is no parse to trace) and
# not by a profiler (there is no run to sample). Tokens are verbatim rustc output, so they cannot
# match unrelated prose; quoting them means a real compiler diagnostic was actually read. Rationale:
# docs/decisions/project_build_integrity_compiler_root_cause_signature.md.
# The fourth group is the CODEGEN-EMISSION diagnosis toolbox (GENERATED-LINT-CORRECTNESS.3,
# 2026-07-27): a defect where the GENERATOR emits the wrong CODE has no parse to trace (the parser
# is correct), no run to sample (it is not a slowness defect) and no compiler error (the emission
# compiles fine) - the WHY+WHERE is the emission site in the generator plus a census of the emitted
# artifacts, and the instrument that reports it is the generated-parser lint lane. Tokens are
# verbatim tool output on the same footing as `error[EXXXX]`: the correctness gate's own signature
# line, a real `clippy::<lint>` path, and the generated stage's strict switch. Rationale:
# docs/decisions/project_codegen_emission_root_cause_signature.md.
#
# ⭐ GROUP 2 VOCABULARY CORRECTED (GENERATED-LINT-CORRECTNESS.4, 2026-07-27). The performance group
# was written from a GENERIC Rust-profiler vocabulary (`cargo flamegraph`, `self-time`) that this
# repository's SPEED campaign does not actually use. MEASURED over every ticked ROOT CAUSE box in
# docs/tasks/: group 2 backed exactly TWO boxes repo-wide, while `docs/tasks/RGX-0078.md` - the
# SPEED tree itself, 153 such boxes - root-causes with macOS `/usr/bin/sample`, `otool -tV`
# annotated disassembly, `spindump`/`filtercalltree`, a process-local `ITIMER_PROF` sampler, and
# PGEN's own `--dump-rule-outcome-counts-json`. Those are verbatim tool invocations on exactly the
# same footing as `cargo flamegraph`, so naming them is a CORRECTION of a bar aimed at the wrong
# tools, NOT a relaxation of it. Without it a SPEED leaf using this repo's real profiler is forced
# to waive - which is how a gate teaches authors to bypass it.
#
# The fifth group is the OPS / BUILD-FLOW diagnosis toolbox (GENERATED-LINT-CORRECTNESS.4,
# 2026-07-27). A defect in the repository's OWN operational surface - a Makefile recipe that
# swallows a nonzero exit, a version gate that passes vacuously, an `execve` argument list that
# overflows ARG_MAX, awk array auto-vivification in the memory guard, untracked-residue hygiene -
# has NO parse to trace, NO run to sample, NO compiler error (it is shell/make, not Rust) and NO
# codegen emission. That is the IDENTICAL argument the record already used to admit groups 3 and 4.
# ⭐ This family was requested BY THE CORPUS ITSELF and nobody read it: docs/tasks/RGX-0090.md:131
# carries a hand-written waiver note inside the ticked box - verbatim "like RGX-0091 this is a
# BUILD-FLOW defect - the parse/perf diagnosis-toolbox signatures do not apply" - and
# docs/tasks/RGX-0091.md:119 wrote "Diagnosis tool signatures: grep -n ..." and got no credit.
# Tokens are verbatim shell/git/OS invocations and errno names, so quoting one means the tool was
# actually run. ⛔ A bare `grep` mention is DELIBERATELY EXCLUDED: it matches 16 boxes on prose such
# as "verified by grep", i.e. it is a claim, not tool output. Rationale + the measured case:
# docs/decisions/project_ops_build_flow_root_cause_signature.md.
#
# ⭐ REGISTERING A NEW INSTRUMENT IS A LOCKSTEP OBLIGATION, NOT A SIXTH FAMILY
# (ENGINE-UNIVERSAL-SERVICES.11 slice 1, 2026-08-12). Group 1 is a VOCABULARY OF TOOLS, so a
# family-1 instrument that lands without its token joining this list is invisible here: every leaf
# root-caused with it is blocked, and the only ways out are to cite a tool that did NOT produce the
# diagnosis or to waive - both dishonest, and the second is how a gate teaches bypass. That is
# exactly what happened when `PGEN_REACH_FORCED_OVERRIDE_DUMP` (TOOLBOX.md 6.4) landed: the gate
# CORRECTLY refused the very leaf that built the tool. `PGEN_REACH_PATH_DUMP` is already registered
# on identical footing, so this is the same correction group 2 made for the SPEED profilers - the
# bar was aimed at a stale tool list, not lowered. ⛔ The obligation runs the OTHER way too: a token
# may only be added here when a real, runnable instrument emits it, and TOOLBOX.md's five-family
# table must list it in the same commit. Adding a token for prose is the `grep` exclusion above.
DIAGNOSIS_SIG='CERTIFICATE-COVERAGE:|\[plannable-probe\]|rejected by post predicate|furthest_position=|witnessed_target=(true|false)|PGEN_CERT_COVERAGE_(DUMP_ALL|DEBUG_PROBES)|PGEN_LINT_DUMP_ALL|PGEN_REACH_PATH_DUMP|PGEN_REACH_FORCED_OVERRIDE_DUMP|\[forced-override\]|--report-certificate-coverage|--trace-rules|--dump-rule-call-counts|--dump-rule-outcome-counts|--lint-grammar|--parse-dump-ast|self-time|call-graph attribution|call-graph samples|cargo flamegraph|flamegraph|/usr/bin/sample|\botool\b|\bspindump\b|\bfiltercalltree\b|\bITIMER_PROF\b|error\[E[0-9]{4}\]|could not compile|GENERATED-CLIPPY-CORRECTNESS:|clippy::[a-z_]{3,}|PGEN_CLIPPY_GENERATED_STRICT|git (ls-files|log -S|log --all -S|rev-list|fsck|reflog|diff-tree|merge-base|cat-file)|\bshellcheck\b|bash -n |sh -n |make -n |make --dry-run|\bE2BIG\b|\bENOSPC\b|\bEACCES\b|\bARG_MAX\b|guard\.[0-9]+\.marker|reason=(none|rss-budget|free-floor|disk-floor|timeout)'
NOREGRESS_SIG='seeds? *0/7/42|byte-identical|external corpus *1[0-9]/1[0-9]|corpus *1[0-9]/1[0-9]|shape.?contract|spf=0|sample_parse_failures=0|fully_certified|clippy'

fails=()

# ⭐ ROOT_KW NARROWED (GENERATED-LINT-CORRECTNESS.9, 2026-08-01). It used to carry a third
# alternative `\bwhy\b` matching the BARE WORD, and that alternative was FAILS-OPEN on one of the
# director's own named steps: a `**FIX**` box routinely writes prose such as "Why no lower tier: …"
# and routinely quotes a command, so a leaf carrying NO ROOT CAUSE box at all could satisfy the
# ROOT CAUSE requirement on its FIX box. That is categorically worse than the two candidates `.7`
# disproved (cross-leaf `unchecked` blocking and `box_body` `^#` truncation), which both fail CLOSED.
# The same alternative had a fails-CLOSED twin: an UNTICKED `**FIX**` box mentioning "why" tripped
# `unchecked()` and blocked a leaf whose real checklist was complete.
# ⛔ PRICED BEFORE ADOPTING, as a NARROWING — so the bar is "does it break anything", not `.4`/`.7`'s
# "is there corpus pressure" (which is the bar a WIDENING must clear). Measured over every ticked
# ROOT CAUSE box in docs/tasks/: 4 boxes drop, 0 of them BACKED, 0 files lose their last backed box,
# and the unticked false-block surface stays 0 -> 0. The connector class `[-+/&]` keeps the
# template spelling `**ROOT CAUSE (WHY + WHERE)**` plus the obvious hand-written variants.
# Probes: docs/tasks/artifacts/generated_lint_correctness/run_diag_evidence_root_kw_probes.sh.
ROOT_KW='root cause|why ?[-+/&] ?where'
ADDR_KW='addressed|verified|resolved|before.{0,5}after|reject.{0,6}pass'
NOREG_KW='no.?regress|regression'

# Per requirement, three escalating facts across the staged task files, so a breach can be
# reported at the precise stage it failed rather than as one opaque "checklist incomplete":
#   present — a ticked box with that keyword exists
#   sig     — ... and the required signature sits inside that box's own bullet   (`.3`)
#   own     — ... and that box's leaf section is one this change touched         (`.7`)
declare -A present=() sig_ok=() own=()
declare -A file_all=()
for req in ROOT ADDR NOREG; do present[$req]=0; sig_ok[$req]=0; own[$req]=0; done

for f in "${staged_tasks[@]}"; do
  n_full=0
  for req in ROOT ADDR NOREG; do
    case "$req" in
      ROOT)  kw="$ROOT_KW";  s="$DIAGNOSIS_SIG" ;;
      ADDR)  kw="$ADDR_KW";  s="" ;;
      NOREG) kw="$NOREG_KW"; s="$NOREGRESS_SIG" ;;
    esac
    box_matches_in_file "$f" x "$kw" ""   0 && present[$req]=1
    box_matches_in_file "$f" x "$kw" "$s" 0 || continue
    sig_ok[$req]=1
    box_matches_in_file "$f" x "$kw" "$s" 1 || continue
    own[$req]=1; n_full=$((n_full + 1))
  done
  [ "$n_full" -eq 3 ] && file_all["$f"]=1
done

# An UNTICKED required box blocks outright, wherever it sits — the step is unfinished.
if   unchecked "$ROOT_KW";  then fails+=("ROOT CAUSE box is present but UNTICKED ([ ]) — the cause is not yet established.")
elif [ "${present[ROOT]}" -eq 0 ]; then fails+=("ROOT CAUSE (WHY+WHERE) box is MISSING/unticked from the acceptance checklist.")
elif [ "${sig_ok[ROOT]}" -eq 0 ]; then
  fails+=("ROOT CAUSE box is ticked but the diagnosis-tool signature is NOT INSIDE THAT BOX (cert/probe/trace/furthest_position; profiler self-time/flamegraph//usr/bin/sample/otool; rustc error[EXXXX]; a codegen-emission signature such as clippy::<lint>; or an ops/build-flow signature such as git ls-files / make -n / E2BIG). A token elsewhere in the file — or in a co-staged tree file — no longer counts.")
fi

if   unchecked "$ADDR_KW"; then fails+=("ADDRESSED/VERIFIED box is present but UNTICKED — the fix is not yet confirmed to resolve the issue.")
elif [ "${present[ADDR]}" -eq 0 ]; then fails+=("ADDRESSED (verified the issue is resolved) box is MISSING/unticked.")
fi

if   unchecked "$NOREG_KW"; then fails+=("NO REGRESSION box is present but UNTICKED — regressions are not yet cleared.")
elif [ "${present[NOREG]}" -eq 0 ]; then fails+=("NO REGRESSION box is MISSING/unticked from the acceptance checklist.")
elif [ "${sig_ok[NOREG]}" -eq 0 ]; then
  fails+=("NO REGRESSION box is ticked but the global-gate signature is NOT INSIDE THAT BOX (seeds 0/7/42, byte-identical, external corpus, shape-contract, spf=0, fully_certified, clippy). A token elsewhere in the file — or in a co-staged tree file — no longer counts.")
fi

# `.7` — the box must be one THIS change wrote, and all three must live in ONE leaf's file.
if [ "${#fails[@]}" -eq 0 ]; then
  for req in ROOT ADDR NOREG; do
    [ "${own[$req]}" -eq 1 ] && continue
    case "$req" in
      ROOT)  label="ROOT CAUSE" ;;
      ADDR)  label="ADDRESSED" ;;
      NOREG) label="NO REGRESSION" ;;
    esac
    fails+=("$label box is ticked and backed, but it belongs to a LEAF THIS CHANGE DID NOT TOUCH — an already-finished leaf cannot supply the checklist for new work. Write the box in the leaf that owns this change.")
  done
  if [ "${#fails[@]}" -eq 0 ] && [ "${#file_all[@]}" -eq 0 ]; then
    fails+=("the three required boxes are satisfied only ACROSS DIFFERENT task files — one owning leaf must carry ROOT CAUSE + ADDRESSED + NO REGRESSION together.")
  fi
fi

if [ "${#fails[@]}" -gt 0 ]; then
  cat >&2 <<'MSG'
diag-evidence: ✗ the staged task leaf does NOT pass the required ACCEPTANCE CHECKLIST for a code
  change. A fix must be PROVABLY taken through the procedure (analyse -> root cause -> fix ->
  addressed -> no regression), not "trust me". Add/complete the checklist (template in TOOLBOX.md):
    - [x] ROOT CAUSE (WHY + WHERE)  — backed by a debug-tool signature from ONE of the five families:
                                      correctness (cert / [plannable-probe] / predicate-rejection trace / furthest_position=),
                                      performance (/usr/bin/sample / otool -tV / flamegraph self-time),
                                      build-integrity (rustc error[EXXXX]), codegen-emission (clippy::<lint>),
                                      ops/build-flow (git ls-files / make -n / shellcheck / E2BIG / guard marker reason=)
    - [x] ADDRESSED (verified)      — the issue is resolved (before->after on the symptom)
    - [x] NO REGRESSION             — global metrics (cert seeds 0/7/42 spf=0, 6 grammars byte-identical, external corpus 14/14, ast_shape_contract GREEN, clippy)
  An UNTICKED required box means the task is not done — finish the step, do not bypass. The
  deterministic gates re-run the oracles in CI, so the NO-REGRESSION numbers you cite are re-verified.
  Breaches:
MSG
  for m in "${fails[@]}"; do printf '  - %s\n' "$m" >&2; done
  exit 1
fi

echo "diag-evidence: OK (task leaf passes the acceptance checklist: ROOT CAUSE + ADDRESSED + NO REGRESSION, evidence-backed)"
exit 0
