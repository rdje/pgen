#!/usr/bin/env bash
# scripts/check_corpus_key_integrity.sh
#
# DOCTRINE `CORPUS-KEY-INTEGRITY` — CORPUS-KEY-AUDIT.1(e).
#
#   The corpus ANSWER KEY is an instrument, so it is checked like one — and the instrument
#   that checks it is checked too.
#
# ⛔ THE DEFECT THIS CLOSES — MEASURED, NOT HYPOTHETICAL. The adjudication manifests decide what
# counts as a parser defect, and every published SV number rests on them. Two failures in three
# days showed the key itself can be wrong, in the direction nothing reports:
#   * `SV-CORPUS-GRAD.13e.3`(a) — two files using the SAME operator (`~&` as a binary) carried
#     OPPOSITE expectations; the difference was an `iverilog` flag the key never read.
#   * `SV-0068` — three rows keyed `must_accept` for text IEEE 1364-2005 cannot derive sat at
#     `match`, the STRONGEST verdict in the file, for the whole campaign. ⛔ An expectation error
#     in the ACCEPT direction produces no flag at all, so residual burn-down can never find it.
#
# ⛔⛔ AND THE FIRST CENSUS BUILT TO CATCH THAT WAS ITSELF WRONG, TWICE, INSIDE ONE COMMIT
# (CORPUS-KEY-AUDIT.1(a)): it attributed every golden message to a row's single verdict — a 100 %
# false-positive rate, 2 of 2 — and its message regex required a literal `error:`/`sorry:` tag while
# iverilog emits its bare parse refusal UNTAGGED (`./ivltests/br_gh79.v:6: syntax error`), so the
# most parse-relevant class was absent from the vocabulary its headline was measured over. ⇒ this
# doctrine checks the CENSUS before it trusts the census's verdict, which is what tier A1 is for.
#
# ⭐⭐ TWO TIERS, BECAUSE THEY NEED DIFFERENT THINGS TO BE PRESENT.
#
#   A1 INSTRUMENT ALIVE (always, corpus-INDEPENDENT, ~0.1 s) — the census's own `--self-test` must
#     run its full arm set with zero failures. The arms are synthetic, so this tier binds on every
#     commit including a clone with no submodules. It is the tier that would have caught the
#     founding extractor blindness, because one arm pins the untagged `syntax error` class.
#     ⛔ An arm COUNT floor is enforced: a self-test that runs zero arms and prints `failed=0` is a
#     control that cannot fail, which is worse than no control (see the `--self-test` arms below).
#
#   A2/A3 KEY CLEAN + PUBLISHED ARTIFACT CURRENT (corpus-DEPENDENT, 0.24 s) — the census must report
#     zero contradictory message classes and zero key-integrity findings, AND the tracked
#     `census.md` must be byte-identical to a fresh derivation. ⭐ A3 exists because a derived
#     artifact nobody re-derives rots silently: `SV-CORPUS-DENOMINATOR` was founded on exactly that,
#     its artifact measured stale ONE DAY after landing.
#
# ⛔ The vendored corpora are git submodules. When they are absent this reports A2/A3 as
# NOT EVALUATED, LOUDLY — never as a pass, and never as a failure that would block every commit in a
# fresh clone (the posture `GRAMMAR-CERT-CURRENCY` and `GENERATED-REPRODUCIBILITY` already take for
# the untracked `generated/` tree).
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict (0 holds / 1 breach / 2 cannot
# run); explains on stderr; deterministic; MUTATES NOTHING — the census is driven with `--md`
# pointing at a scratch file so the tracked artifact is never written by this gate.
#
# Usage:
#   bash scripts/check_corpus_key_integrity.sh              # the doctrine
#   bash scripts/check_corpus_key_integrity.sh --self-test  # prove every refusal fires
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2

CENSUS="docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py"
ARTIFACT="docs/tasks/artifacts/corpus_key_audit/census.md"
MIN_ARMS=5   # the arm set at adoption; a self-test that shrinks below it has lost coverage

say() { printf 'corpus-key-integrity: %s\n' "$1" >&2; }

# ---- verdict readers. Kept as pure functions of a SUMMARY LINE so the self-test can drive every
# refusal without fabricating a census. ⛔ Both REFUSE a line they cannot parse: a reader that
# treats an unrecognized line as "no findings" fails in the PASSING direction, which is the whole
# family of defect this lane exists to catch.
field() { printf '%s' "$2" | sed -n "s/.*[[:space:]]$1=\([0-9][0-9]*\).*/\1/p"; }

verdict_from_selftest() {  # <summary line> -> 0 holds / 1 breach
  local line="$1" arms failed
  arms="$(field arms "$line")"; failed="$(field failed "$line")"
  if [ -z "$arms" ] || [ -z "$failed" ]; then
    say "✗ (A1) the census self-test printed no parsable 'arms=N failed=M' summary:"
    say "       ${line:-<no output>}"
    return 1
  fi
  if [ "$failed" -ne 0 ]; then
    say "✗ (A1) the census self-test has $failed failing arm(s) — the instrument is not trustworthy,"
    say "       so its verdict about the key means nothing. Re-run: python3 $CENSUS --self-test"
    return 1
  fi
  if [ "$arms" -lt "$MIN_ARMS" ]; then
    say "✗ (A1) the census self-test runs $arms arm(s), below the $MIN_ARMS pinned at adoption."
    say "       A control set that shrank is coverage lost silently — restore the arm or move this"
    say "       floor deliberately in CORPUS-KEY-AUDIT.1(e)."
    return 1
  fi
  return 0
}

verdict_from_census() {  # <summary line> -> 0 holds / 1 breach
  local line="$1" contra integrity rows
  contra="$(field contradictory_classes "$line")"
  integrity="$(field key_integrity_findings "$line")"
  rows="$(field rows "$line")"
  if [ -z "$contra" ] || [ -z "$integrity" ] || [ -z "$rows" ]; then
    say "✗ (A2) the census printed no parsable summary (rows= / contradictory_classes= /"
    say "       key_integrity_findings=): ${line:-<no output>}"
    return 1
  fi
  if [ "$rows" -eq 0 ]; then
    say "✗ (A2) the census scanned ZERO keyed rows. A clean verdict over an empty population is not"
    say "       a clean key — it is an instrument that found nothing to look at."
    return 1
  fi
  local rc=0
  if [ "$contra" -ne 0 ]; then
    say "✗ (A2) $contra upstream message class(es) carry CONTRADICTORY expectations: two rows keyed"
    say "       from the same evidence disagree about must_accept vs must_reject. Adjudicate each"
    say "       against the LRM and route it to SV-CORPUS-GRAD with a clause cite — never relabel."
    rc=1
  fi
  if [ "$integrity" -ne 0 ]; then
    say "✗ (A2) $integrity row(s) quote evidence their own golden does not contain — the key"
    say "       contradicts itself. See the artifact's 'Key integrity' section."
    rc=1
  fi
  [ $rc -eq 0 ] && say "  (A2) key clean over $rows keyed rows (0 contradictions, 0 integrity findings)"
  return $rc
}

run_doctrine() {
  local rc=0 out line tmp

  # ---- A1 INSTRUMENT ALIVE (corpus-independent; binds on every commit).
  [ -f "$CENSUS" ] || { say "✗ the census is missing: $CENSUS"; return 1; }
  out="$(python3 "$CENSUS" --self-test 2>&1)"
  line="$(printf '%s\n' "$out" | grep -E 'SELF-TEST: .*arms=' | tail -1)"
  verdict_from_selftest "$line" || { printf '%s\n' "$out" >&2; rc=1; }

  # ---- A2/A3 (corpus-dependent). The census REFUSES with exit 2 when the submodules are absent.
  # ⛔ Scratch is REPO-DERIVED, never $TMPDIR: project-owned data stays on the repository's own
  # volume, and rust/target/ is git-ignored (the same choice every other self-test here makes).
  mkdir -p rust/target || return 2
  tmp="$(mktemp "$ROOT/rust/target/corpus_key_census.XXXXXX")" || return 2
  out="$(python3 "$CENSUS" --md "$tmp" 2>&1)"; local census_rc=$?
  if [ "$census_rc" -eq 2 ]; then
    say "⚠️ NOT EVALUATED (A2/A3) — the vendored corpora are absent (git submodules), so the key"
    say "   cannot be measured. This is a fresh clone, NOT a pass. Populate them with:"
    say "   git submodule update --init --recursive"
    rm -f "$tmp"
    [ $rc -eq 0 ] && say "  (A1) instrument alive — the corpus-independent tier still bound."
    return $rc
  fi
  if [ "$census_rc" -ne 0 ]; then
    say "✗ (A2) the census exited $census_rc:"; printf '%s\n' "$out" >&2; rm -f "$tmp"; return 1
  fi
  line="$(printf '%s\n' "$out" | grep -E '^KEY-CONTRADICTION-CENSUS: ' | tail -1)"
  verdict_from_census "$line" || rc=1

  # ---- A3 the PUBLISHED artifact is what a fresh run produces. A derived file nobody re-derives
  # rots, and the rot is invisible precisely because the file looks authoritative.
  if [ ! -f "$ARTIFACT" ]; then
    say "✗ (A3) the published census artifact is missing: $ARTIFACT"; rc=1
  elif ! diff -q "$ARTIFACT" "$tmp" >/dev/null 2>&1; then
    say "✗ (A3) $ARTIFACT is NOT what the census produces today — it is stale or hand-edited."
    say "       Re-derive and commit it: python3 $CENSUS"
    diff -u "$ARTIFACT" "$tmp" 2>&1 | head -30 >&2
    rc=1
  fi
  rm -f "$tmp"
  return $rc
}

self_test() {
  # GROUND TRUTH over trust: a guard nobody has seen refuse is indistinguishable from one that
  # cannot refuse. Every arm drives a real code path of this script.
  local pass=0 fail=0
  arm() { # arm <label> <expect-rc> <fn> <line>
    local label="$1" want="$2" fn="$3" line="$4" rc
    "$fn" "$line" >/dev/null 2>&1; rc=$?
    if [ "$rc" -eq "$want" ]; then printf '  ✅ %s (rc %d)\n' "$label" "$rc"; pass=$((pass+1))
    else printf '  ❌ %s: rc %d, wanted %d\n' "$label" "$rc" "$want"; fail=$((fail+1)); fi
  }
  printf 'corpus-key-integrity --self-test:\n'

  arm "A1 GREEN: a full, passing arm set" 0 verdict_from_selftest \
      "KEY-CONTRADICTION-CENSUS SELF-TEST: arms=$MIN_ARMS failed=0"
  arm "A1 RED: an arm is failing" 1 verdict_from_selftest \
      "KEY-CONTRADICTION-CENSUS SELF-TEST: arms=$MIN_ARMS failed=1"
  arm "A1 RED: the arm set SHRANK below the pinned floor" 1 verdict_from_selftest \
      "KEY-CONTRADICTION-CENSUS SELF-TEST: arms=$((MIN_ARMS-1)) failed=0"
  arm "A1 RED: zero arms reporting success is a control that cannot fail" 1 verdict_from_selftest \
      "KEY-CONTRADICTION-CENSUS SELF-TEST: arms=0 failed=0"
  arm "A1 RED: an unparsable summary REFUSES, never passes" 1 verdict_from_selftest ""

  arm "A2 GREEN: clean key over a real population" 0 verdict_from_census \
      "KEY-CONTRADICTION-CENSUS: rows=499 messages=176 contradictory_classes=0 (naive=2) key_integrity_findings=0"
  arm "A2 RED: a contradictory message class" 1 verdict_from_census \
      "KEY-CONTRADICTION-CENSUS: rows=499 messages=176 contradictory_classes=2 (naive=2) key_integrity_findings=0"
  arm "A2 RED: a basis quoting evidence its golden lacks" 1 verdict_from_census \
      "KEY-CONTRADICTION-CENSUS: rows=499 messages=176 contradictory_classes=0 (naive=2) key_integrity_findings=3"
  arm "A2 RED: a clean verdict over ZERO rows is not a clean key" 1 verdict_from_census \
      "KEY-CONTRADICTION-CENSUS: rows=0 messages=0 contradictory_classes=0 (naive=0) key_integrity_findings=0"
  arm "A2 RED: an unparsable summary REFUSES, never passes" 1 verdict_from_census \
      "KEY-CONTRADICTION-CENSUS: everything is fine"

  # A3 end-to-end: a corrupted published artifact must be caught by the real doctrine path.
  local rc saved
  mkdir -p rust/target || return 2
  saved="$(mktemp "$ROOT/rust/target/census_artifact.XXXXXX")"
  if [ -f "$ARTIFACT" ]; then
    cp "$ARTIFACT" "$saved"
    printf '\nHAND-EDITED LINE THAT NO RUN PRODUCES\n' >> "$ARTIFACT"
    run_doctrine >/dev/null 2>&1; rc=$?
    cp "$saved" "$ARTIFACT"
    if [ "$rc" -eq 1 ]; then printf '  ✅ A3 RED: a hand-edited published artifact is caught (rc 1)\n'; pass=$((pass+1))
    else printf '  ❌ A3: a hand-edited artifact scored rc %d, wanted 1\n' "$rc"; fail=$((fail+1)); fi
    run_doctrine >/dev/null 2>&1; rc=$?
    if [ "$rc" -eq 0 ]; then printf '  ✅ A3 GREEN: the restored artifact passes end-to-end (rc 0)\n'; pass=$((pass+1))
    else printf '  ❌ A3: the restored artifact scored rc %d, wanted 0\n' "$rc"; fail=$((fail+1)); fi
  else
    printf '  ⚠️ A3 arms SKIPPED — %s absent (corpora not populated). Not a pass.\n' "$ARTIFACT"
  fi
  rm -f "$saved"

  # The NOT EVALUATED branch rests on the census REFUSING with exit 2 when the corpora are absent.
  # ⛔ That claim is exercised, not assumed: a submodule-less clone is simulated by relocating a COPY
  # of the census so its own `parents[4]` repo-root walk lands somewhere with no vendored corpus.
  # Otherwise the fresh-clone path would be the one branch of this gate nobody has ever seen run.
  local fake
  fake="$(mktemp -d "$ROOT/rust/target/corpus_key_norepo.XXXXXX")" || return 2
  mkdir -p "$fake/a/b/c/d" && cp "$CENSUS" "$fake/a/b/c/d/census.py"
  python3 "$fake/a/b/c/d/census.py" --md "$fake/out.md" >/dev/null 2>&1; rc=$?
  rm -rf "$fake"
  if [ "$rc" -eq 2 ]; then
    printf '  ✅ REFUSAL: the census exits 2 with no vendored corpus (the NOT EVALUATED branch)\n'
    pass=$((pass+1))
  else
    printf '  ❌ REFUSAL: a corpus-less census exited %d, wanted 2 — the fresh-clone branch would\n' "$rc"
    printf '     read as a PASS instead of NOT EVALUATED\n'; fail=$((fail+1))
  fi

  printf 'corpus-key-integrity: self-test %d passed, %d failed\n' "$pass" "$fail"
  [ "$fail" -eq 0 ]
}

case "${1:-}" in
  --self-test) self_test; exit $? ;;
  "") run_doctrine || exit 1
      echo "corpus-key-integrity: OK (census self-test alive; key clean; published artifact current)"
      exit 0 ;;
  *)  say "unknown argument: $1"; exit 2 ;;
esac
