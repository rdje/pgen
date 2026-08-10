#!/usr/bin/env bash
# stimuli/run_external_corpus.sh <sv|vhdl> [per_file_timeout_s] [jobs] [max_files]
#
# EXTERNAL-CORPUS.3.1 / .3.2 — bulk characterization of the external TEST CORPORA
# vendored under stimuli/<fam>/subs/ (git submodules, sparse-checked-out to their
# test dirs). Parses EVERY corpus file with the PGEN parser and tallies
# parse-pass / parse-fail / timeout per sub-corpus, then writes a dated
# characterization report.
#
# Doctrine (characterize, don't game — EXTERNAL-CORPUS / project_external_corpus_doctrine):
#   - The number reported is the HONEST raw parse outcome, not a forced "conformance pass".
#   - Many corpus files are INTENTIONALLY invalid (e.g. GHDL `gna` bug-regressions,
#     VESTS `non_compliant/`, sv-tests files with `:should_fail_because:`), so a
#     parse-FAIL is often the CORRECT outcome, not a parser bug. This runner reports
#     raw outcomes per sub-corpus; deeper expected-vs-actual adjudication is a follow-up.
#   - These corpora exercise the FULL grammars (not simplified subsets), so genuine
#     divergences are real parser gaps — a bug-finding oracle.
#
# ⛔ THE RECORDED PARAMETERS BIND (SV-CORPUS-GRAD.3.27) — see "Provenance reconciliation" below.
# This script OVERWRITES a tracked graduation ORACLE. Its measurement parameters are therefore
# not free-floating defaults: they are read back from the artifact about to be replaced, adopted
# when the caller supplies nothing, and REFUSED when the caller supplies something different.
#
# Usage:
#   stimuli/run_external_corpus.sh sv            # all SV corpora at the TRACKED parameters
#   stimuli/run_external_corpus.sh vhdl 30 6     # VHDL, 30s/file, 6 jobs  (refused if those
#                                                #   differ from the tracked artifact's own)
#   PGEN_CORPUS_OUT_DIR=… stimuli/run_external_corpus.sh sv 20 8 500      # smoke, non-destructive
#
# Environment:
#   PGEN_PARSE_PROBE_BIN                  parse binary (a CALLER-SUPPLIED measurement parameter)
#   PGEN_CORPUS_OUT_DIR                   write the artifacts elsewhere (repo-relative or absolute,
#                                         resolved against the repo root) — nothing tracked is
#                                         touched, so a pending run can be DIFFED before promotion
#   PGEN_CORPUS_REBASELINE=1              proceed despite parameter drift, establishing a new
#                                         baseline; loud, never silent
#   ⛔ These three are the COMPLETE set. Since SV-CORPUS-GRAD.12c.3 any OTHER `PGEN_CORPUS_*`
#   variable is REFUSED (exit 6) rather than ignored: a near-miss like `PGEN_CORPUS_OUTDIR` used
#   to be silently unread, so the run wrote to the CANONICAL directory while the operator believed
#   it was sandboxed.
#   PGEN_CORPUS_TIMEOUT_RECONFIRM_MAX     cap on serial timeout re-confirmation (default 64;
#                                         the measured populations are 4 / 0 / 0 for sv / vhdl /
#                                         sv2005, so the cap is ~16x the observed maximum and any
#                                         truncation is REPORTED, never silent). Set 0 to skip the
#                                         pass entirely — the artifact then says so out loud.
#
# ✅ MEMORY BUDGET — RESOLVED 2026-08-10 by SV-CORPUS-GRAD.11a (`PGEN-SV-CORPUS-GRAD-0197`); the
# README's example 12288 MB is sufficient again. Measured tree peak for a full `sv` run is now
# **4 756 MB** (16 336 files in 71 s, timeout=0).
#
# The history, kept because it explains why a larger budget is still a safe choice: until `.11a`
# landed, a SINGLE release-binary parse of
# `stimuli/sv/subs/opentitan/hw/top_earlgrey/ip/xbar_main/rtl/autogen/xbar_main.sv` peaked at
# **12 362 MB** before the 60 s deadline cut it — the whole example budget in one process, so
# `--budget-mb 12288` killed the run (measured tree peak 12 468 MB) and `.3.27` raised the guidance
# to >= 16384. The cause was an O(2^n) `if/else if` blowup at `conditional_else_branch`, fixed by one
# `@branch_policy: ordered` annotation; that file now parses in **0.13 s at 118 MB**. Nothing here
# depends on the larger budget any more, but >= 16384 remains harmless headroom.
set -uo pipefail

# LC_ALL=C is load-bearing twice: `sort` must be locale-independent for the artifact to be
# byte-reproducible, and bash's EPOCHREALTIME uses the locale's decimal separator, which the
# duration arithmetic below splits on '.'.
export LC_ALL=C

FAM="${1:?usage: run_external_corpus.sh <sv|sv2005|vhdl> [timeout_s] [jobs] [max_files]}"

# ── SV-CORPUS-GRAD.12c.3 (finding F2) — REFUSE an unrecognised `PGEN_CORPUS_*` spelling ────────
#
# ⛔ THE DEFECT THIS CLOSES. This script reads `PGEN_CORPUS_OUT_DIR`. A near-miss —
# `PGEN_CORPUS_OUTDIR`, `PGEN_CORPUS_OUT`, a typo, a half-remembered name — was simply never read,
# so the run fell through to the CANONICAL directory and OVERWROTE tracked graduation oracles with
# whatever it produced. `.12a` hit exactly this with a 200-file capped run and caught it only
# because `git status` happened to be checked immediately afterwards.
#
# ⭐ A redirect that does not redirect is worse than no redirect at all, because the operator
# believes they are sandboxed and therefore does NOT check.
#
# ⛔ It classifies TOTALLY and refuses on the unmatched, rather than enumerating misspellings —
# the discipline `LIVE-DOC-CURRENCY` instrument B already applies, and for the same reason:
# enumeration misses silently, and silently in the PASSING direction
# (docs/decisions/feedback_enumerating_instrument_must_refuse.md). The recognised set is small,
# closed and authoritative; the set of ways to get it wrong is not.
#
# ⚠️ The whitelist is deliberately NOT named `PGEN_CORPUS_KNOWN_VARS`. The first cut was, and the
# guard's very first run refused the guard's own state variable — the holder of the whitelist sat
# inside the namespace the whitelist polices. Special-casing it would have been the enumerate-the-
# exceptions anti-pattern this guard exists to avoid; renaming it out of the namespace is the fix.
KNOWN_CORPUS_ENV_VARS="PGEN_CORPUS_OUT_DIR PGEN_CORPUS_REBASELINE PGEN_CORPUS_TIMEOUT_RECONFIRM_MAX"

# The decision as a pure function, so it can carry ground truth (an instrument with no ground truth
# is a confident guess — docs/decisions/feedback_instrument_needs_ground_truth.md).
# $1 = variable name; echoes `known` or `unknown`.
corpus_var_verdict() {
  case " $KNOWN_CORPUS_ENV_VARS " in
    *" $1 "*) printf 'known\n' ;;
    *)        printf 'unknown\n' ;;
  esac
}

# GROUND TRUTH, re-run on every invocation (microseconds): every recognised name must classify
# `known`, and the near-miss that provoked this guard must classify `unknown`. A miss REFUSES
# rather than quietly reporting a clean environment.
for _ctrl in $KNOWN_CORPUS_ENV_VARS; do
  [ "$(corpus_var_verdict "$_ctrl")" = known ] || {
    echo "external-corpus: MISCALIBRATED — the env guard does not recognise its own '$_ctrl'" >&2
    exit 7
  }
done
[ "$(corpus_var_verdict PGEN_CORPUS_OUTDIR)" = unknown ] || {
  echo "external-corpus: MISCALIBRATED — the env guard accepts the near-miss it exists to catch" >&2
  exit 7
}
unset _ctrl

# `${!PREFIX@}` lists the names of every set variable with that prefix — the environment is
# enumerated, not the misspellings.
_unknown_corpus_vars=""
for _name in ${!PGEN_CORPUS_@}; do
  [ "$(corpus_var_verdict "$_name")" = unknown ] && _unknown_corpus_vars="$_unknown_corpus_vars $_name"
done
if [ -n "$_unknown_corpus_vars" ]; then
  {
    echo "external-corpus: REFUSING — unrecognised PGEN_CORPUS_* variable(s):$_unknown_corpus_vars"
    echo
    echo "  This script reads ONLY these, and silently ignores anything else:"
    for _known in $KNOWN_CORPUS_ENV_VARS; do echo "    $_known"; done
    echo
    echo "  Refusing rather than ignoring, because the failure is invisible in the direction that"
    echo "  looks safe: an unread PGEN_CORPUS_OUT_DIR near-miss does not redirect anything, so the"
    echo "  run writes to the CANONICAL directory and overwrites tracked graduation oracles while"
    echo "  the operator believes the run is sandboxed."
    echo
    echo "  Fix the spelling, or 'unset' the variable if it was not meant for this script."
  } >&2
  exit 6
fi
unset _name _unknown_corpus_vars _known

# ⛔ Capture what the CALLER actually supplied BEFORE any default is applied. The whole
# reconciliation below turns on the difference between "the caller asked for 20s" and "nobody
# said, so the script's own default was 20s" — `${2:-20}` erases exactly that difference, and
# erasing it is how the tracked artifact came to be silently re-measured under parameters it
# does not record.
ARG_TIMEOUT="${2-}"
ARG_JOBS="${3-}"
ARG_MAX="${4-}"
ARG_PROBE="${PGEN_PARSE_PROBE_BIN-}"

# The script's own fallbacks — used ONLY when there is no tracked artifact to inherit from.
DEFAULT_TIMEOUT=20
DEFAULT_JOBS=8
DEFAULT_MAX=0            # 0 = no cap
DEFAULT_PROBE_REL="rust/target/debug/parseability_probe"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

ROOT_EARLY="$ROOT"
case "$FAM" in
  sv)   GRAMMAR=systemverilog; export PROFILE_ARGS="--profile sv_2017";
        FIND_EXTS=( -name '*.sv' -o -name '*.svh' -o -name '*.v' );
        # SV-CORPUS-GRAD.8: fold the pre-submodule-vintage uvm-core vendoring
        # (plain tracked files) into the bulk universe as sub-corpus `uvm-core`.
        EXTRA_DIRS=( "$ROOT_EARLY/stimuli/sv/uvm" );;
  # SV-CORPUS-GRAD.8c: the verilog_2005 profile lane — re-parses ONLY the
  # adjudicator-derived lane files (single source of lane membership:
  # `v2005_lane_files.tsv`, emitted by adjudicate_external_corpus.py) under
  # the strict verilog_2005 profile.
  sv2005) GRAMMAR=systemverilog; export PROFILE_ARGS="--profile verilog_2005";
        FIND_EXTS=( -name '*.v' );  # unused (list-driven)
        EXTRA_DIRS=();
        LANE_LIST="$ROOT_EARLY/stimuli/sv/characterization/v2005_lane_files.tsv";;
  vhdl) GRAMMAR=vhdl;          export PROFILE_ARGS="";
        FIND_EXTS=( -name '*.vhd' -o -name '*.vhdl' );
        EXTRA_DIRS=();;
  *) echo "unknown family '$FAM' (expected sv|sv2005|vhdl)" >&2; exit 2;;
esac

if [ "$FAM" = sv2005 ]; then
  SUBS="$ROOT/stimuli/sv/subs"
  CANON_OUTDIR="$ROOT/stimuli/sv/characterization"
  RESULTS_NAME="results_v2005.tsv"
  DURATIONS_NAME="durations_v2005.tsv"
  POSITIONS_NAME="positions_v2005.tsv"
  REPORT_NAME="characterization_v2005.md"
else
  SUBS="$ROOT/stimuli/$FAM/subs"
  CANON_OUTDIR="$ROOT/stimuli/$FAM/characterization"
  RESULTS_NAME="results.tsv"
  DURATIONS_NAME="durations.tsv"
  POSITIONS_NAME="positions.tsv"
  REPORT_NAME="characterization.md"
fi
CANON_REPORT="$CANON_OUTDIR/$REPORT_NAME"

# ---- provenance reconciliation (SV-CORPUS-GRAD.3.27) ----
#
# ⛔ THE DEFECT THIS CLOSES. `characterization.md` faithfully records the parameters it was
# produced with — that is the FLOW-INTEGRITY hand-off-provenance work — but the record was
# WRITE-ONLY: nothing read it back and compared it against the run about to overwrite it. All
# three tracked artifacts (sv / vhdl / sv2005) were produced at `60 8 0` against the RELEASE
# probe, while this script's own defaults were `20 8 0` against the DEBUG probe, and
# `stimuli/sv/subs/.../mm_ram.sv` parses in 12 s release vs 127 s debug. So the documented bare
# invocation re-measured a graduation oracle under a ~10x slower binary at a 3x tighter deadline
# and published the result with no complaint: measured in `.3.25`, 6 rows moved into
# `divergence:explained_timeout` (timeouts 4 -> 10) on a parser change that provably could not
# touch them. Landing on `must_accept` rows instead, the same drift would have REMOVED files from
# `unexplained_rejects_valid` — the burn-down number improving because the machine was busy
# (a-rising-pass-rate-is-not-evidence-of-correctness). Nothing guarded that direction.
#
# The remedy is DOCTRINE_ENFORCEMENT.md §3's *structural* archetype: the invariant is re-derived
# from the artifact itself on every run, so it cannot rot. Adopt what the artifact records when
# the caller says nothing; REFUSE when the caller says something else.
REC_TIMEOUT=""; REC_JOBS=""; REC_MAX=""; REC_PROBE_REL=""
if [ -f "$CANON_REPORT" ]; then
  # "Generated by `stimuli/run_external_corpus.sh sv 60 8 0` against `parseability_probe`."
  _inv="$(awk -F'`' '/^Generated by `stimuli\/run_external_corpus\.sh /{print $2; exit}' "$CANON_REPORT")"
  if [ -n "${_inv:-}" ]; then
    read -r _ _f REC_TIMEOUT REC_JOBS REC_MAX <<<"$_inv"
  fi
  # "| parse binary | `rust/target/release/parseability_probe` | `<sha256>` |"
  REC_PROBE_REL="$(awk -F'`' '/^\| parse binary \|/{print $2; exit}' "$CANON_REPORT")"
fi

# The repo-root-relative spelling of a path, whatever it arrived as (repo-relative-paths policy:
# a comparison against a recorded relative path must not depend on where the clone lives).
rel_to_root() { case "$1" in "$ROOT"/*) printf '%s' "${1#"$ROOT"/}" ;; *) printf '%s' "$1" ;; esac; }

DRIFT_LINES=()
RECONCILED_VALUE=""; RECONCILED_SOURCE=""
# reconcile <label> <caller-value> <recorded-value> <script-default>
# Sets RECONCILED_VALUE/RECONCILED_SOURCE and appends to DRIFT_LINES. ⛔ NOT a command
# substitution on purpose: `$(...)` would run the array append in a subshell and every drift
# would be silently forgotten — the exact failure shape this leaf exists to remove.
reconcile() {
  local label="$1" caller="$2" recorded="$3" dflt="$4"
  if [ -n "$caller" ]; then
    RECONCILED_VALUE="$caller"; RECONCILED_SOURCE="caller"
    if [ -n "$recorded" ] && [ "$caller" != "$recorded" ]; then
      DRIFT_LINES+=("$(printf '  %-22s pending run: %-46s tracked artifact: %s' "$label" "$caller" "$recorded")")
    fi
  elif [ -n "$recorded" ]; then
    RECONCILED_VALUE="$recorded"; RECONCILED_SOURCE="provenance"
  else
    RECONCILED_VALUE="$dflt"; RECONCILED_SOURCE="default"
  fi
}

reconcile "per-file timeout" "$ARG_TIMEOUT" "$REC_TIMEOUT" "$DEFAULT_TIMEOUT"
TIMEOUT_S="$RECONCILED_VALUE"; SRC_TIMEOUT="$RECONCILED_SOURCE"
reconcile "parallel jobs"    "$ARG_JOBS"    "$REC_JOBS"    "$DEFAULT_JOBS"
JOBS="$RECONCILED_VALUE";     SRC_JOBS="$RECONCILED_SOURCE"
reconcile "max files"        "$ARG_MAX"     "$REC_MAX"     "$DEFAULT_MAX"
MAX_FILES="$RECONCILED_VALUE"; SRC_MAX="$RECONCILED_SOURCE"
reconcile "parse binary"     "$([ -n "$ARG_PROBE" ] && rel_to_root "$ARG_PROBE")" \
                             "$REC_PROBE_REL" "$DEFAULT_PROBE_REL"
PROBE_REL="$RECONCILED_VALUE"; SRC_PROBE="$RECONCILED_SOURCE"
case "$PROBE_REL" in /*) PROBE="$PROBE_REL" ;; *) PROBE="$ROOT/$PROBE_REL" ;; esac

if [ "${#DRIFT_LINES[@]}" -gt 0 ]; then
  if [ "${PGEN_CORPUS_REBASELINE:-0}" = 1 ]; then
    {
      echo "external-corpus[$FAM]: ⚠️  PARAMETER DRIFT ACCEPTED (PGEN_CORPUS_REBASELINE=1) — this"
      echo "  run establishes a NEW baseline; the artifact it replaces was measured differently:"
      printf '%s\n' "${DRIFT_LINES[@]}"
    } >&2
  else
    {
      echo "external-corpus[$FAM]: ⛔ REFUSING — the pending run's measurement parameters differ"
      echo "  from the ones \`${CANON_REPORT#"$ROOT"/}\` records, so it would silently re-baseline"
      echo "  a graduation oracle under conditions the artifact does not describe."
      echo
      printf '%s\n' "${DRIFT_LINES[@]}"
      echo
      echo "  Fix, in order of preference:"
      echo "    1. omit the differing argument(s) — they are ADOPTED from the artifact's provenance;"
      echo "    2. PGEN_CORPUS_OUT_DIR=<dir> …  — measure elsewhere and DIFF before promoting;"
      echo "    3. PGEN_CORPUS_REBASELINE=1 …   — deliberately establish a new baseline."
    } >&2
    exit 5
  fi
fi

# Output location. A non-canonical directory touches nothing tracked, so a pending run can be
# compared against the artifact it would replace instead of overwriting it first and asking later.
if [ -n "${PGEN_CORPUS_OUT_DIR:-}" ]; then
  case "$PGEN_CORPUS_OUT_DIR" in /*) OUTDIR="$PGEN_CORPUS_OUT_DIR" ;; *) OUTDIR="$ROOT/$PGEN_CORPUS_OUT_DIR" ;; esac
else
  OUTDIR="$CANON_OUTDIR"
fi
RESULTS="$OUTDIR/$RESULTS_NAME"
DURATIONS="$OUTDIR/$DURATIONS_NAME"
POSITIONS="$OUTDIR/$POSITIONS_NAME"
REPORT="$OUTDIR/$REPORT_NAME"

if [ ! -x "$PROBE" ]; then
  {
    echo "parseability_probe not found/executable at ${PROBE#"$ROOT"/}"
    if [ "$SRC_PROBE" = provenance ]; then
      echo "  — that path came from \`${CANON_REPORT#"$ROOT"/}\`'s provenance block, i.e. it is the"
      echo "    binary the tracked number was measured with. Build it rather than falling back:"
      echo "      (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)"
      echo "    To measure with a different binary deliberately:"
      echo "      PGEN_PARSE_PROBE_BIN=<path> PGEN_CORPUS_REBASELINE=1 $0 $FAM"
    fi
  } >&2
  exit 3
fi

mkdir -p "$OUTDIR"

{
  echo "external-corpus[$FAM]: effective measurement parameters"
  printf '  %-18s %-52s (%s)\n' "per-file timeout" "${TIMEOUT_S}s"        "$SRC_TIMEOUT"
  printf '  %-18s %-52s (%s)\n' "parallel jobs"    "$JOBS"                "$SRC_JOBS"
  printf '  %-18s %-52s (%s)\n' "max files"        "$MAX_FILES"           "$SRC_MAX"
  printf '  %-18s %-52s (%s)\n' "parse binary"     "$PROBE_REL"           "$SRC_PROBE"
  [ -f "$CANON_REPORT" ] || echo "  (no tracked artifact for this family yet — script defaults are in force)"
  [ "$OUTDIR" = "$CANON_OUTDIR" ] || echo "  output redirected to ${OUTDIR#"$ROOT"/} — nothing tracked is overwritten"
} >&2

export PROBE GRAMMAR TIMEOUT_S

# Parse a single file; emit "<subcorpus>\t<status>\t<repo-root-relative path>\t<wall seconds>".
# Columns 1-3 are the tracked `results.tsv` contract; column 4 goes only to the durations sidecar.
#
# CORPUS-GRAD-ALL.2.1 — column 3 is REPO-ROOT-RELATIVE, never absolute. An absolute path bakes
# the checkout root into the artifact, which made the tracked VHDL characterization report
# unreproducible: every one of its 13 720 rows pointed at a DIFFERENT clone (a home-directory copy
# on another volume), so the report could be neither re-run nor diffed against a later one, and it
# breached both the repo-root-relative-paths rule and the same-volume data-locality policy. A
# relative column makes the artifact portable and diffable, which is the precondition for TRACKING
# it — an untracked measurement is the staleness defect class
# (project_all_parsers_fully_pass_stimuli_and_external_corpora clause 4).
#
# Backward-compatible with the consumers BY CONSTRUCTION: both `adjudicate_external_corpus.py` and
# the stuck-point clusterer key on the `/subs/<suite>/` (or `/stimuli/sv/uvm/`) INFIX and split
# there, which a relative path still contains — verified before changing this.
#
# ⭐ Column 4 is the per-file WALL DURATION in seconds (SV-CORPUS-GRAD.3.27). It answers the one
# question the pass/fail/timeout triple cannot: how many files sit close enough to the deadline
# that ordinary machine contention can flip them? Without it the timeout population is a surprise
# rather than a known number, and a `must_accept` row flipping to `explained_timeout` silently
# IMPROVES the burn-down.
#
# ⛔ It is emitted to a SIDECAR (`durations.tsv`), never as a 4th column of `results.tsv`, and
# that is a measured decision rather than a stylistic one: three consumers hard-unpack exactly
# three fields (`adjudicate_external_corpus.py` at both its results readers,
# `corpus_rule_coverage.py`), so a 4th column raises `ValueError: too many values to unpack` —
# and `cluster_rejects_valid.py` skips any row with `len(cols) != 3`, so it would have dropped
# EVERY row and reported an empty worklist. One loud break and one silent one; the silent one is
# the worklist generator.
_now_ms() {
  # bash >= 5 exposes EPOCHREALTIME (microseconds, no subprocess). Older bashes fall back to the
  # integer SECONDS counter — 1 s resolution, which is ample for the only question durations are
  # asked here (proximity to a deadline measured in tens of seconds).
  if [ -n "${EPOCHREALTIME:-}" ]; then
    local e="${EPOCHREALTIME}"
    printf '%s' "$(( ${e%.*} * 1000 + 10#${e#*.} / 1000 ))"
  else
    printf '%s' "$(( SECONDS * 1000 ))"
  fi
}
export -f _now_ms

parse_one() {
  local f="$1" sub rc status rel t0 t1 ms
  sub="$(printf '%s' "$f" | sed -E 's#.*/subs/([^/]+)/.*#\1#; s#.*/stimuli/sv/uvm/.*#uvm-core#')"
  rel="${f#"$ROOT"/}"
  t0="$(_now_ms)"
  # shellcheck disable=SC2086
  # ⭐ SV-CORPUS-GRAD.12a — stderr is CAPTURED, not discarded. The probe prints
  # `furthest_position=N` on every reject, and discarding it is why the adjudicator had
  # to decide "is this file preprocessor-blocked?" from a whole-file regex instead of
  # from where the parse actually stopped. Measured cost of the capture: none that the
  # run's own wall clock can resolve; measured cost of NOT having it: 26 corpus rows
  # mislabelled `explained_svpp_*` and removed from the burn-down (leaf .12).
  local err fp
  err="$(timeout "$TIMEOUT_S" "$PROBE" --parse "$GRAMMAR" "$f" $PROFILE_ARGS 2>&1 >/dev/null)"
  rc=$?
  t1="$(_now_ms)"
  ms=$(( t1 - t0 )); [ "$ms" -ge 0 ] || ms=0
  if [ "$rc" -eq 0 ]; then status=pass
  elif [ "$rc" -eq 124 ]; then status=timeout
  elif [ "$rc" -ge 128 ]; then status=crash   # signal death (e.g. stack
                                              # overflow abort) is NOT a
                                              # graceful reject - own status
  else status=fail; fi
  # Pure-bash extraction (no subprocess in the hot loop): take the text after the last
  # `furthest_position=` and keep its leading digits. Empty when the probe printed none
  # (a pass, a timeout, or a non-parse error) — column 5 is then empty, never 0, because
  # 0 is a REAL position and would read as "stopped at byte 0".
  fp=""
  case "$err" in
    *furthest_position=*) fp="${err##*furthest_position=}"; fp="${fp%%[!0-9]*}" ;;
  esac
  printf '%s\t%s\t%s\t%d.%02d\t%s\n' "$sub" "$status" "$rel" "$(( ms / 1000 ))" "$(( (ms % 1000) / 10 ))" "$fp"
}
export -f parse_one
export ROOT

if [ "$FAM" = sv2005 ]; then
  [ -f "$LANE_LIST" ] || { echo "lane list not found: $LANE_LIST (run the adjudicator first)" >&2; exit 4; }
  echo "external-corpus[$FAM]: reading lane files from $LANE_LIST ..." >&2
  mapfile -t FILES < <(tail -n +2 "$LANE_LIST" | cut -f3 | sed "s#^#$ROOT/#" | sort)
else
  echo "external-corpus[$FAM]: collecting files under $SUBS ..." >&2
  mapfile -t FILES < <(find "$SUBS" ${EXTRA_DIRS[@]+"${EXTRA_DIRS[@]}"} -type f \( "${FIND_EXTS[@]}" \) 2>/dev/null | sort)
fi
TOTAL_FOUND=${#FILES[@]}
if [ "$MAX_FILES" -gt 0 ] && [ "$TOTAL_FOUND" -gt "$MAX_FILES" ]; then
  FILES=( "${FILES[@]:0:$MAX_FILES}" )
  echo "external-corpus[$FAM]: capping at $MAX_FILES of $TOTAL_FOUND files (smoke run)" >&2
fi
echo "external-corpus[$FAM]: parsing ${#FILES[@]} files (timeout=${TIMEOUT_S}s, jobs=$JOBS) ..." >&2

# ⛔ SV-CORPUS-GRAD.12c.3 — the scratch temps are removed on EXIT, not only on the success path.
# They live beside their output on purpose (same filesystem), but until now the only `rm` was at
# the very end of the script, so a run killed by a timeout, a Ctrl-C or the memory guard left a
# ~124 KB `.durations.tsv.parallel` sitting in the TRACKED artifact directory — untracked churn
# that reads like an artifact. Measured by killing a canonical run at 6 s while exercising the env
# guard above. `${VAR:-}` because `set -u` is on and the trap can fire before either is assigned.
#
# ⭐⭐ THE SIGNAL TRAPS ARE NOT DECORATION, and the first cut of this fix proved it by being INERT.
# `trap … EXIT` alone does NOT run when the shell dies on an UNTRAPPED signal, and `timeout` sends
# SIGTERM — which is precisely the case that leaves residue. The re-measure after adding the EXIT
# trap alone still found the stray file; only trapping the signals fixed it. Each handler just
# calls `exit`, which then runs the EXIT trap (`rm -f` is idempotent, so the double call is free).
# ⚠️ Honest limit: SIGKILL cannot be trapped, so a hard `kill -9` (or an OOM kill) still leaves the
# temps. That is unavoidable, not overlooked.
_cleanup_corpus_temps() {
  [ -n "${RAW:-}" ] && rm -f "$RAW" "$RAW.sorted"
  [ -n "${RECONFIRM_FILE:-}" ] && rm -f "$RECONFIRM_FILE"
  return 0
}
trap _cleanup_corpus_temps EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

RAW="$OUTDIR/.${DURATIONS_NAME}.parallel"
: > "$RAW"
printf '%s\0' "${FILES[@]}" | xargs -0 -P "$JOBS" -I{} bash -c 'parse_one "$@"' _ {} >> "$RAW"

# ---- serial re-confirmation of the timeout population (SV-CORPUS-GRAD.3.27) ----
#
# ⛔ A timeout is a statement about the MACHINE as much as about the parser: under `-P 8` a file
# a few seconds inside the deadline can cross it because seven siblings were competing for the
# same cores. Recorded as `timeout`, that file is later adjudicated `divergence:explained_timeout`
# and leaves `unexplained_rejects_valid` — i.e. the burn-down number improves for a reason that
# has nothing to do with the parser. So every timeout is re-run ALONE at the same deadline before
# it is believed. Cheap by measurement: the tracked populations are 4 (sv) / 0 (vhdl) / 0 (sv2005).
RECONFIRM_MAX="${PGEN_CORPUS_TIMEOUT_RECONFIRM_MAX:-64}"
RECONFIRM_FILE="$OUTDIR/.${DURATIONS_NAME}.reconfirm"
: > "$RECONFIRM_FILE"
mapfile -t TMO_ROWS < <(awk -F'\t' '$2=="timeout"{print $3}' "$RAW" | sort)
TMO_TOTAL=${#TMO_ROWS[@]}
RECONFIRM_DONE=0; RECONFIRM_CHANGED=0; RECONFIRM_SKIPPED=0
if [ "$TMO_TOTAL" -gt 0 ]; then
  echo "external-corpus[$FAM]: re-confirming $TMO_TOTAL timeout row(s) serially (no contention) ..." >&2
  for _rel in "${TMO_ROWS[@]}"; do
    if [ "$RECONFIRM_DONE" -ge "$RECONFIRM_MAX" ]; then
      RECONFIRM_SKIPPED=$(( RECONFIRM_SKIPPED + 1 )); continue
    fi
    RECONFIRM_DONE=$(( RECONFIRM_DONE + 1 ))
    _line="$(parse_one "$ROOT/$_rel")"
    printf '%s\n' "$_line" >> "$RECONFIRM_FILE"
    case "$_line" in *$'\t'timeout$'\t'*) ;; *) RECONFIRM_CHANGED=$(( RECONFIRM_CHANGED + 1 )) ;; esac
  done
  echo "external-corpus[$FAM]: re-confirmed $RECONFIRM_DONE, reclassified $RECONFIRM_CHANGED, not re-confirmed $RECONFIRM_SKIPPED" >&2
fi

# Substitute the re-confirmed verdicts, then emit BOTH artifacts in a deterministic order.
#
# ⛔ The sort is not cosmetic. `xargs -P` appends in COMPLETION order, so the tracked artifact's
# row order was nondeterministic: a re-run whose every verdict was identical still produced a
# diff of thousands of moved lines, which is precisely why nobody diffed it. An oracle that
# cannot be compared with the run replacing it is the defect this leaf owns.
awk -F'\t' -v OFS='\t' -v RF="$RECONFIRM_FILE" '
  BEGIN { while ((getline l < RF) > 0) { split(l, a, "\t"); r[a[3]] = l } }
  { if ($3 in r) print r[$3]; else print }
' "$RAW" | sort -t'	' -k1,1 -k3,3 > "$RAW.sorted"

# ⛔ THE TRACKED ARTIFACT SHAPES ARE HELD: results.tsv stays 3 columns and durations.tsv
# stays 4. The `furthest_position` column added in SV-CORPUS-GRAD.12a is projected into a
# THIRD sidecar rather than appended to either, for the reason already measured for the
# duration column: three consumers hard-unpack exactly three fields
# (`adjudicate_external_corpus.py` at both readers, `corpus_rule_coverage.py`) and
# `cluster_rejects_valid.py` DROPS any row with `len(cols) != 3` — one loud break and one
# silent one, the silent one being the worklist generator.
cut -f1-4 "$RAW.sorted" > "$DURATIONS"
cut -f1-3 "$RAW.sorted" > "$RESULTS"
# Only rows that actually carry a position: passes and timeouts have none, and an absent
# row is what tells the adjudicator "undecidable, keep today's answer".
awk -F'	' -v OFS='	' '$5 != "" { print $1, $3, $5 }' "$RAW.sorted" > "$POSITIONS"
rm -f "$RAW" "$RAW.sorted" "$RECONFIRM_FILE"

# ---- instrument identity (SV-CORPUS-GRAD.10) ----
#
# ⛔ A characterization report that does not name WHICH parser produced it cannot be checked
# for staleness. The axis-2 FRESHNESS AUDIT (2026-08-08) had to prove the tracked SV report
# stale by comparing the GIT COMMIT DATES of two OTHER files, because the report itself said
# only "against `parseability_probe`" — not which build, not which grammar, not which HEAD.
# Content hashes make the artifact self-dating: a later reader re-hashes the same three
# inputs and knows in ONE command whether the number still describes their tree. This closes
# the staleness defect class named by project_all_parsers_fully_pass_stimuli_and_external_corpora
# ("the first honest act is to re-measure them rather than to quote them") at the source, so
# the next reader does not have to reconstruct vintage from unrelated commit dates.
sha256_of() {
  [ -f "$1" ] || { printf '(absent)'; return; }
  if command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | cut -d' ' -f1
  elif command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1
  else printf '(no sha256 tool)'; fi
}

GRAMMAR_FILE="$ROOT/grammars/$GRAMMAR.ebnf"
GEN_PARSER="$ROOT/generated/${GRAMMAR}_parser.rs"
HEAD_ID="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || printf 'unknown')"
HEAD_DATE="$(git -C "$ROOT" log -1 --date=short --format=%ad 2>/dev/null || printf 'unknown')"
if [ -n "$(git -C "$ROOT" status --porcelain 2>/dev/null)" ]; then HEAD_ID="$HEAD_ID+dirty"; fi

# ---- aggregate ----
TOTAL=$(wc -l < "$RESULTS" | tr -d ' ')
PASS=$(awk -F'\t' '$2=="pass"' "$RESULTS" | wc -l | tr -d ' ')
FAIL=$(awk -F'\t' '$2=="fail"' "$RESULTS" | wc -l | tr -d ' ')
TMO=$(awk -F'\t' '$2=="timeout"' "$RESULTS" | wc -l | tr -d ' ')
CRASH=$(awk -F'\t' '$2=="crash"' "$RESULTS" | wc -l | tr -d ' ')
PCT=$(awk -v p="$PASS" -v t="$TOTAL" 'BEGIN{ if(t>0) printf "%.1f", 100*p/t; else print "0.0" }')

# ---- deadline-proximity census (SV-CORPUS-GRAD.3.27) ----
# How many NON-timeout files finished within 2x of the deadline, i.e. how large is the population
# a slower machine or a busier run could flip? A known number beats a surprise.
NEAR_HALF=$(awk -F'\t' -v t="$TIMEOUT_S" '$2!="timeout" && $4*2 >= t' "$DURATIONS" | wc -l | tr -d ' ')
SLOWEST_LINE=$(awk -F'\t' '$2!="timeout"' "$DURATIONS" | sort -t'	' -k4,4gr | head -1)
SLOWEST_S=$(printf '%s' "$SLOWEST_LINE" | cut -f4); SLOWEST_S="${SLOWEST_S:-0.00}"
SLOWEST_F=$(printf '%s' "$SLOWEST_LINE" | cut -f3); SLOWEST_F="${SLOWEST_F:-(none)}"

{
  echo "# External Test-Corpus Characterization — \`$FAM\` (EXTERNAL-CORPUS.3.$([ "$FAM" = sv ] && echo 1 || echo 2))"
  echo
  echo "Generated by \`stimuli/run_external_corpus.sh $FAM $TIMEOUT_S $JOBS $MAX_FILES\` against \`$(basename "$PROBE")\`."
  echo "Per-file parse via \`parseability_probe --parse $GRAMMAR <file> $PROFILE_ARGS\`, timeout ${TIMEOUT_S}s, $JOBS-way parallel."
  echo
  echo "> **Characterize, don't game.** Raw parse outcomes only. Many corpus files are"
  echo "> INTENTIONALLY invalid (GHDL \`gna\` regressions, VESTS \`non_compliant/\`, sv-tests"
  echo "> \`:should_fail_because:\`), so a parse-FAIL is frequently the CORRECT outcome — not a"
  echo "> parser bug. Expected-vs-actual adjudication per sub-corpus is the follow-up slice."
  echo
  echo "## Instrument identity (what produced this number)"
  echo
  echo "> Re-hash these three inputs; if any hash differs from the row below, **this report no"
  echo "> longer describes your tree** and the honest act is to re-measure, not to quote."
  echo
  echo "| input | repo-root-relative path | sha256 |"
  echo "|---|---|---|"
  echo "| parse binary | \`${PROBE#"$ROOT/"}\` | \`$(sha256_of "$PROBE")\` |"
  echo "| grammar | \`${GRAMMAR_FILE#"$ROOT/"}\` | \`$(sha256_of "$GRAMMAR_FILE")\` |"
  echo "| generated parser | \`${GEN_PARSER#"$ROOT/"}\` | \`$(sha256_of "$GEN_PARSER")\` |"
  echo
  echo "Measured at \`HEAD\` = \`$HEAD_ID\` ($HEAD_DATE)."
  echo
  echo "## Measurement parameters (BINDING — the next run is held to them)"
  echo
  echo "> These four values are re-read from this report by \`stimuli/run_external_corpus.sh\` on"
  echo "> every later run: an omitted argument is **adopted** from the row below, and a supplied"
  echo "> argument that **differs** is **refused** (exit 5) unless \`PGEN_CORPUS_REBASELINE=1\`."
  echo "> Before \`SV-CORPUS-GRAD.3.27\` this block was write-only, so the documented bare"
  echo "> invocation re-measured the whole corpus under a slower binary at a tighter deadline and"
  echo "> published over the artifact without a word."
  echo
  echo "| parameter | value | source for this run |"
  echo "|---|---|---|"
  echo "| per-file timeout | ${TIMEOUT_S} s | $SRC_TIMEOUT |"
  echo "| parallel jobs | $JOBS | $SRC_JOBS |"
  echo "| max files | $MAX_FILES$([ "$MAX_FILES" = 0 ] && echo ' (no cap)') | $SRC_MAX |"
  echo "| parse binary | \`$PROBE_REL\` | $SRC_PROBE |"
  echo
  echo "## Deadline proximity and serial re-confirmation"
  echo
  echo "> A timeout is a statement about the machine as much as about the parser. Every"
  echo "> \`timeout\` row is re-run **alone** at the same deadline before it is recorded, and the"
  echo "> population sitting within 2x of the deadline is published so the flip-risk set is a"
  echo "> known number. Per-file durations: \`${DURATIONS#"$ROOT/"}\`."
  echo
  echo "| population | files |"
  echo "|---|---|"
  echo "| \`timeout\` after serial re-confirmation | $TMO |"
  echo "| timeouts seen in the parallel pass | $TMO_TOTAL |"
  echo "| re-confirmed serially | $RECONFIRM_DONE |"
  echo "| reclassified by re-confirmation (contention, not the parser) | $RECONFIRM_CHANGED |"
  echo "| **not** re-confirmed (cap \`$RECONFIRM_MAX\`) | $RECONFIRM_SKIPPED |"
  echo "| completed within 2x of the ${TIMEOUT_S}s deadline | $NEAR_HALF |"
  echo
  echo "Slowest completing file: \`${SLOWEST_S}\` s — \`${SLOWEST_F}\`."
  [ "$RECONFIRM_SKIPPED" -eq 0 ] || {
    echo
    echo "⚠️ $RECONFIRM_SKIPPED timeout row(s) were NOT serially re-confirmed (cap"
    echo "\`PGEN_CORPUS_TIMEOUT_RECONFIRM_MAX=$RECONFIRM_MAX\`); those rows carry the contended"
    echo "parallel-pass verdict and are not evidence of a parser-side timeout."
  }
  [ "$OUTDIR" = "$CANON_OUTDIR" ] || {
    echo
    echo "⚠️ **NON-CANONICAL RUN** — written to \`${OUTDIR#"$ROOT/"}\` via \`PGEN_CORPUS_OUT_DIR\`,"
    echo "not to the tracked \`${CANON_OUTDIR#"$ROOT/"}\`. Diff it against the tracked artifact"
    echo "before promoting anything."
  }
  echo
  echo "## Totals"
  echo
  echo "| files parsed | pass | fail | timeout | crash | pass-rate |"
  echo "|---|---|---|---|---|---|"
  echo "| $TOTAL | $PASS | $FAIL | $TMO | $CRASH | ${PCT}% |"
  echo
  echo "## Per sub-corpus"
  echo
  echo "| sub-corpus | files | pass | fail | timeout | crash | pass-rate |"
  echo "|---|---|---|---|---|---|---|"
  awk -F'\t' '
    { tot[$1]++; if($2=="pass")p[$1]++; else if($2=="timeout")to[$1]++; else if($2=="crash")c[$1]++; else f[$1]++ }
    END { for (s in tot) {
            r = (tot[s]>0)? 100*p[s]/tot[s] : 0;
            printf "| %s | %d | %d | %d | %d | %d | %.1f%% |\n", s, tot[s], p[s]+0, f[s]+0, to[s]+0, c[s]+0, r
          } }
  ' "$RESULTS" | sort
  echo
  echo "_Raw per-file results: \`${RESULTS#"$ROOT/"}\` (3 columns: sub-corpus, status,"
  echo "repo-root-relative path — a stable contract three consumers unpack positionally)._"
  echo "_Per-file durations: \`${DURATIONS#"$ROOT/"}\` (the same rows plus a 4th wall-seconds column)._"
  echo "_Both are sorted by (sub-corpus, path), so two runs are directly diffable._"
} > "$REPORT"

echo "external-corpus[$FAM]: $TOTAL parsed — pass=$PASS fail=$FAIL timeout=$TMO crash=$CRASH (${PCT}% pass). Report: $REPORT" >&2
