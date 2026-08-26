#!/usr/bin/env bash
# SAMPLE-PROFILE-PARITY — every `@sample` in a grammar still parses under the profiles its own
# BRANCH is live in. Tree: docs/tasks/SV-CORPUS-GRAD.md (leaf `.13e.11`). Exits NONZERO on breach.
# Called by scripts/check_doctrines.sh via .githooks/pre-commit (E3) and CI (E4).
#
# ⭐⭐ WHY THIS EXISTS — measured, not assumed.
#   An `@sample` is a SINGLE PROFILE-BLIND STRING. Declaring a `@profiles` dialect NARROWS the
#   accepted language, so every sample already written against the wider language becomes a
#   candidate defect the moment the narrower profile exists. `SV-CORPUS-GRAD.13e.9` measured FOUR
#   such samples on `systemverilog`, costing TEN certificate rules, written 2026-04-22 and latent
#   since the profile was registered on 2026-07-02 — 54 days — because NOTHING in the repository
#   read `@sample` at all: no gate, no workflow, no hook, no lint. The defect class is created by
#   an edit that is otherwise correct (adding a dialect), which is why nobody looks.
#
# ⛔⛔ THE JOIN IS THE WHOLE DESIGN, AND THE OBVIOUS ONE IS 100 % FALSE POSITIVES.
#   `@profiles` is a RULE-level annotation while 43 of SystemVerilog's 52 samples are BRANCH-level.
#   A rule stays live under a profile as long as SOME alternative survives, while other alternatives
#   are dead there and their samples are then honestly unparseable. Measured on the real grammar:
#     rule-level join   -> 115 checks, 8 reported, 0 true defects  (every row a false positive)
#     branch-level join -> 101 checks, 0 reported, 0 true defects
#   The 8 are exactly the 7 `.13e.9` adjudicated BY HAND as "branch dead under v2005 — sample
#   honest", plus the 1 that `.13e.10`(c4) created. A gate on the rule-level join would have shipped
#   the 91 %-false-positive shape `GENERATED-LINT-CORRECTNESS.6`/`.12` refused.
#
# ARCHETYPE: oracle (DOCTRINE_ENFORCEMENT.md §3) — it RE-EXECUTES the census, so a fabricated claim
#   does not reproduce. Not a currency proxy: it gates whether the samples PARSE, never whether the
#   census was re-run.
#
# ⭐ SCOPE — STAGED-DIFF-SCOPED, and the reason is cadence rather than taste (§4 rule 5).
#   The census is 33 s on `systemverilog` (101 checks, one grammar load). That is affordable when a
#   grammar changes and unaffordable on every commit, so this check is INERT unless the staged set
#   contains a `grammars/**.ebnf`. ⛔ The alternative — a tracked baseline plus an identity block,
#   cheap tier 1 hashing inputs and heavy tier 2 on demand — was considered and REFUSED: it gates
#   "was the census re-run since the grammar moved", a currency PROXY, and `ENGINE-UNIVERSAL-SERVICES.53`
#   measured in the same week what a proxy costs when the question moves out from under it. Making
#   the census cheap (`-0308`: 2 m 43 s -> 33 s) is what made the proxy unnecessary.
#
# ⛔ A GRAMMAR WITH NO `@profiles` IS `NOT EVALUATED`, LOUDLY — never a pass and never a block. A
#   reader that treats what it cannot evaluate as "no findings" fails in the PASSING direction (the
#   `CORPUS-KEY-INTEGRITY` A1 shape). The instrument prints the sample count beside that refusal, so
#   a grammar with samples and no profile universe is visible rather than silent.
#
# HONEST LIMITS, stated rather than hidden:
#   - The census reads the grammar's OWN `@sample` annotations. A construct with no sample is not
#     covered by it, and never was — this doctrine bounds sample ROT, not sample COVERAGE.
#   - `branch_satisfiable_under` is BOTTOM-UP satisfiability, not reachability (TOOLBOX 5.11's
#     standing trap). That is the right predicate here because a sample is parsed AT its owning rule
#     as the entry, so the question genuinely is bottom-up; it would be wrong for an entry-relative
#     claim.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict; explains on stderr;
# deterministic (no clock, no network, no RNG — the interpreter is a pure function of grammar +
# input); read-only; staged-scope-aware; path-agnostic; inert when out of scope.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

HEADLINE="SAMPLE-PROFILE-PARITY"
BIN="rust/target/debug/ast_pipeline"
LOG_DIR="rust/target/sample_profile_parity"

note() { printf '%s: %s\n' "$HEADLINE" "$1" >&2; }
ok()   { printf '%s: %s\n' "$HEADLINE" "$1"; }

# ── the population in scope ─────────────────────────────────────────────────────────────────────
# Staged grammars only. No staged set (a manual run) => census the whole tracked population, which
# is what `--all` is for and what the self-test drives.
staged_grammars() {
  git diff --cached --name-only --diff-filter=ACM 2>/dev/null \
    | grep -E '^grammars/.*\.ebnf$' || true
}

# ⭐ THE GENERATOR MUST BE PROVEN CURRENT, NOT MERELY PRESENT. A stale binary answers a question
# about a tree that no longer exists, and this check's whole verdict is that binary's output. Cargo
# is asked whether it is current: ~0.8 s when it is (the `ENGINE-UNIVERSAL-SERVICES.32` measurement),
# a rebuild when it is not — which is exactly when you want one. Feature set mirrors rust/Makefile.
ensure_binary() {
  if ! ( cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline ) \
       >"$LOG_DIR/build.log" 2>&1; then
    return 1
  fi
  [ -x "$BIN" ]
}

# Verdicts: 0 = censused clean · 1 = parity DEFECT · 2 = NOT EVALUATED (loudly, never a pass) ·
# 3 = the instrument itself misbehaved (a clean exit with no verdict line).
census_one() {
  local grammar="$1" log="$2" rc=0
  "$BIN" "$grammar" --report-sample-parity >"$log" 2>&1 || rc=$?
  # ⛔ THE EXIT CODE ALONE IS NOT THE VERDICT, and the two nonzero causes need OPPOSITE dispositions.
  # A grammar whose samples do not parse is a DEFECT; a grammar that cannot be LOADED at all is a
  # BUILD/EXTRACTION finding this doctrine does not measure, and converting one into the other is
  # the `-0293` failure (a load failure read as 160 phantom moves in the flattering direction).
  # The instrument's own headline is what separates them: it is printed only after the grammar
  # loaded and the census ran.
  local headline
  headline="$(grep -m1 "^${HEADLINE}: " "$log" || true)"
  if [ -z "$headline" ]; then
    if [ "$rc" != 0 ]; then
      # ⭐ MEASURED, and this is why the disposition is NOT EVALUATED rather than a block: the
      # closed-population sweep (`--all`) found **4 of the 17** tracked grammars do not load —
      # `verilog_2005_lrm_extracted` and the two `systemverilog_*_lrm_extracted` files carry raw
      # LRM text the loader rejects (*"unterminated regex literal"*), and
      # `systemverilog_lrm_profiled_generated` is an include fragment with no `@entry`. They have
      # never generated a parser. Blocking a commit that touches one would be a false positive on
      # nearly a quarter of the population — the shape `GENERATED-LINT-CORRECTNESS.6`/`.12` refused.
      note "NOT EVALUATED — '$grammar' does not LOAD (rc=$rc), so there is no parity verdict to give."
      note "  That is a grammar/extraction finding, not a parity one, and this gate refuses to convert one into the other. See $log"
      return 2
    fi
    # ⛔ A CLEAN EXIT WITH NO VERDICT LINE IS THE INSTRUMENT MISBEHAVING, and it is the one case that
    # must block: it is indistinguishable from "checked everything and found nothing" to any reader.
    note "REFUSING to score '$grammar' — exit 0 with no '${HEADLINE}:' headline. A missing verdict is NOT a pass. See $log"
    return 3
  fi
  printf '  %s\n' "${headline#"$HEADLINE": }"
  case "$headline" in
    *"NOT EVALUATED"*) return 2 ;;
  esac
  if [ "$rc" != 0 ]; then
    grep "^${HEADLINE}: DEFECT" "$log" >&2 || true
    grep "^${HEADLINE}: UNJOINABLE" "$log" >&2 || true
    note "'$grammar' has \`@sample\` annotations that no longer parse under a profile their own BRANCH is live in."
    note "  Re-run it directly:  ./$BIN $grammar --report-sample-parity"
    note "  Then either FIX the sample for that profile, or — if the branch is genuinely dead there —"
    note "  the census already excludes it, so a reported row means the branch IS live and the sample is stale."
    return 1
  fi
  return 0
}

mode="${1:-}"

# ── --self-test: every refusal arm, observed firing ─────────────────────────────────────────────
# A green instrument proves nothing until its refusals fire. Each arm builds a throwaway grammar
# copy under rust/target (repo volume, CLAUDE.md §13) and drives the REAL census.
if [ "$mode" = "--self-test" ]; then
  mkdir -p "$LOG_DIR/selftest"
  arms=0; passed=0
  work="$LOG_DIR/selftest"
  if [ ! -x "$BIN" ]; then
    note "--self-test needs $BIN; build it with: make -C rust SHELL=/bin/bash ast_pipeline"
    exit 2
  fi

  # ARM 1 — a clean copy is GREEN. Without this the RED arms could be red for the copy, not the edit.
  arms=$((arms + 1))
  cp grammars/regex.ebnf "$work/green.ebnf"
  if "$BIN" "$work/green.ebnf" --report-sample-parity >"$work/green.log" 2>&1; then
    passed=$((passed + 1)); ok "arm 1 GREEN control — a clean copy passes"
  else
    note "arm 1 FAILED — a clean copy must pass"
  fi

  # ARM 2 — a broken sample on a LIVE branch is RED, and rc is 1.
  arms=$((arms + 1))
  if [ -f grammars/regex.ebnf ]; then
    # Break the regex grammar's own `@sample` by appending a byte its own language cannot accept.
    sed 's/^\(@sample: *"\)\(.*\)\("\)$/\1\2(\3/' grammars/regex.ebnf > "$work/red.ebnf"
    if cmp -s grammars/regex.ebnf "$work/red.ebnf"; then
      note "arm 2 NOT EVALUATED — the fixture edit changed nothing; the \`@sample\` spelling moved"
    elif "$BIN" "$work/red.ebnf" --report-sample-parity >"$work/red.log" 2>&1; then
      note "arm 2 FAILED — a broken live-branch sample must be REPORTED, not passed"
    elif grep -q "^${HEADLINE}: DEFECT" "$work/red.log"; then
      passed=$((passed + 1)); ok "arm 2 RED control — a broken live-branch sample is reported and rc is nonzero"
    else
      note "arm 2 FAILED — nonzero exit with no DEFECT line: that is a LOAD failure, not a parity verdict"
    fi
  fi

  # ARM 3 — a grammar with no `@profiles` is NOT EVALUATED, never a pass.
  arms=$((arms + 1))
  if "$BIN" grammars/json.ebnf --report-sample-parity 2>&1 | grep -q "NOT EVALUATED"; then
    passed=$((passed + 1)); ok "arm 3 — a grammar with no \`@profiles\` reports NOT EVALUATED, not a pass"
  else
    note "arm 3 FAILED — a profile-less grammar must refuse rather than report clean"
  fi

  # ARM 4 — a grammar that does not LOAD is NOT EVALUATED, never scored clean and never a block.
  # ⭐ This arm exists because the first cut of this check got it wrong and the closed-population
  # sweep measured it: 4 of the 17 tracked grammars do not load (three raw LRM extractions and one
  # include fragment), so the first cut would have BLOCKED any commit staging one of them.
  arms=$((arms + 1))
  printf 'not an ebnf grammar at all\n' > "$work/bogus.ebnf"
  census_one "$work/bogus.ebnf" "$work/bogus.log" >/dev/null 2>&1
  if [ "$?" = 2 ]; then
    passed=$((passed + 1)); ok "arm 4 — a grammar that does not LOAD is NOT EVALUATED (never scored clean, never a block)"
  else
    note "arm 4 FAILED — a load failure must be NOT EVALUATED, neither a pass nor a parity verdict"
  fi

  # ARM 5 — and the one case that MUST block: a clean exit with no verdict line, which reads to any
  # consumer as "checked everything, found nothing". Injected by pointing the census at a stub that
  # exits 0 and says nothing, so the arm tests THIS script's disposition rather than the engine's.
  arms=$((arms + 1))
  printf '#!/usr/bin/env bash\nexit 0\n' > "$work/silent_bin"
  chmod +x "$work/silent_bin"
  ( BIN="$work/silent_bin"; census_one grammars/json.ebnf "$work/silent.log" ) >/dev/null 2>&1
  if [ "$?" = 3 ]; then
    passed=$((passed + 1)); ok "arm 5 — a SILENT clean exit is refused (exit 3) and blocks, never a pass"
  else
    note "arm 5 FAILED — exit 0 with no verdict line must block, not pass"
  fi

  ok "--self-test: arms=${passed}/${arms}"
  [ "$passed" = "$arms" ] || exit 1
  exit 0
fi

# ── the ordinary run ────────────────────────────────────────────────────────────────────────────
mkdir -p "$LOG_DIR"

if [ "$mode" = "--all" ]; then
  targets="$(git ls-files 'grammars/*.ebnf')"
else
  targets="$(staged_grammars)"
fi

if [ -z "$targets" ]; then
  # Out of scope, and that is the common case by design — not a silent pass over the population.
  exit 0
fi

if ! ensure_binary; then
  # ⛔ NOT EVALUATED, loudly, and NOT a block: a tree that cannot build the analyser is a larger
  # finding than this gate measures, and every other doctrine that needs the binary says the same.
  note "NOT EVALUATED — $BIN could not be built from this tree (see $LOG_DIR/build.log)."
  note "  That is a BUILD finding, not a parity verdict, and this gate refuses to convert one into the other."
  exit 0
fi

fail=0
misbehaved=0
censused=0
not_evaluated=0
for grammar in $targets; do
  [ -f "$grammar" ] || continue
  log="$LOG_DIR/$(printf '%s' "$grammar" | tr '/' '_').log"
  census_one "$grammar" "$log"
  case "$?" in
    0) censused=$((censused + 1)) ;;
    2) not_evaluated=$((not_evaluated + 1)) ;;
    3) misbehaved=1 ;;
    *) fail=1 ;;
  esac
done

if [ "$fail" != 0 ] || [ "$misbehaved" != 0 ]; then
  exit 1
fi
# ⛔ THE PASS LINE MUST NOT OVERCLAIM. When nothing was censused, saying "every staged grammar's
# samples parse" is true and useless — and reads as coverage that did not happen. Say which it was.
if [ "$censused" = 0 ]; then
  ok "NOT EVALUATED for all ${not_evaluated} staged grammar(s) — nothing was censused, and that is reported rather than passed as clean"
else
  ok "OK — ${censused} grammar(s) censused, every \`@sample\` parsing under the profiles its own branch is live in (${not_evaluated} NOT EVALUATED)"
fi
exit 0
