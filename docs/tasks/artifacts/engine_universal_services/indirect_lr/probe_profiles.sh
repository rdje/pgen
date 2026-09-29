#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.50 (c) — the PROFILE-AWARE dual-oracle driver for P6
# (`p6_crossing_profile_gates.ebnf`: two CROSSING dialect gates on ONE indirect-LR route).
#
# One row per (input, profile): the verdict on the generated parser (GEN — the scratch slot,
# authoritative by construction) and on the interpreter (INTERP — `ast_pipeline --interpret-parse
# --grammar-profile`), whether the two agree, whether their TYPED ASTs are byte-identical after
# canonical JSON, the OUTER `kind` the AST carries, and whether the row matches the expectation the
# grammar's header pre-registers. The `kind` column is the load-bearing one: the two suffix routes
# iterate a byte-identical suffix and differ only in the AST they declare, so a route gate copied
# from the wrong donor shows up as a wrong `kind` under one profile, never as a wrong verdict.
#
# ⛔ WHY BOTH ORACLES — `probe.sh`'s lesson, restated: on a SURVIVING cycle the interpreter and the
# generated parser disagree in both directions (`.13`, tracked as `.14`). The point of P6 is that the
# cycle no longer survives, and the only proof of that on the interpreter is agreement with GEN.
#
# Usage (from the repository root):
#   docs/tasks/artifacts/engine_universal_services/indirect_lr/probe_profiles.sh                # both oracles
#   docs/tasks/artifacts/engine_universal_services/indirect_lr/probe_profiles.sh --interp-only   # fast column
#
# Exit code: 0 iff every row matches the pre-registered expectation on every oracle that ran AND (with
# both oracles) every GEN/INTERP cell agrees on verdict and AST; 1 on any disagreement; 2 on a tool
# error. `--interp-only` can never be the basis of a claim about the engine on its own.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../../../../.." && pwd)"
cd "$REPO_ROOT" || exit 2

AST_PIPELINE="rust/target/debug/ast_pipeline"
PROBE_BIN="rust/target/debug/parseability_probe"
SCRATCH="grammars/scratch/scratch.ebnf"
WORK="rust/target/indirect_lr_probe/p6"
PROBE="p6_crossing_profile_gates"
GRAMMAR="$HERE/$PROBE.ebnf"
INTERP_ONLY=0
[[ "${1:-}" == "--interp-only" ]] && INTERP_ONLY=1

INPUTS=("n" "m" "n'(n)" "m'(n)" "n'(n)'(n)" "m'(n)'(n)")
PROFILES=(x y z)

# The pre-registered matrix (input | profile | verdict[:outer kind]) — see the grammar's header.
EXPECT="$(cat <<'TABLE'
n          x accept:a
n          y accept:a
n          z reject
m          x accept:b
m          y accept:b
m          z accept:b
n'(n)      x accept:a
n'(n)      y reject
n'(n)      z reject
m'(n)      x accept:a
m'(n)      y reject
m'(n)      z accept:b
n'(n)'(n)  x accept:a
n'(n)'(n)  y reject
n'(n)'(n)  z reject
m'(n)'(n)  x accept:a
m'(n)'(n)  y reject
m'(n)'(n)  z accept:b
TABLE
)"
expected_for() {  # $1 input, $2 profile
  awk -v i="$1" -v p="$2" '$1==i && $2==p { print $3; exit }' <<<"$EXPECT"
}

mkdir -p "$WORK" || exit 2
[[ -x "$AST_PIPELINE" ]] || {
  echo "missing $AST_PIPELINE — build it: (cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline)" >&2
  exit 2
}

# Restore the scratch slot on ANY exit — fixture AND generated parser (probe.sh's trap, verbatim in
# intent: `git checkout` alone leaves generated/scratch_parser.rs holding the probe's grammar).
restore_scratch() {
  [[ $INTERP_ONLY -eq 1 ]] && return 0
  git checkout -- "$SCRATCH" 2>/dev/null || return 0
  make -C rust SHELL=/bin/bash focus_scratch >"$WORK/restore_focus.log" 2>&1 || {
    echo "⛔ scratch slot RESTORE-REGENERATE failed — generated/scratch_parser.rs still holds the" >&2
    echo "   probe's grammar. Re-run by hand before committing anything:" >&2
    echo "     make -C rust SHELL=/bin/bash focus_scratch    # log: $WORK/restore_focus.log" >&2
  }
  # ⛔ THE THIRD HALF OF THE SLOT: the two BINARIES. `focus_scratch` builds ast_pipeline BEFORE it
  # regenerates (TOOLBOX §1.3's ordering trap), so the restore's own run links ast_pipeline against
  # the PROBE's parser, and the debug probe this driver built embeds it too. Rebuild both against the
  # restored fixture so no binary on disk is a P6 binary once the run is over (~2-3 min).
  (cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline \
          && cargo build --features generated_parsers --bin parseability_probe) >"$WORK/restore_build.log" 2>&1 || {
    echo "⛔ post-restore rebuild failed — rust/target/debug/{ast_pipeline,parseability_probe} may still embed" >&2
    echo "   the probe's scratch parser; rebuild by hand (log: $WORK/restore_build.log)" >&2
  }
}
trap restore_scratch EXIT

canon_json() {  # $1 file -> canonical JSON on stdout (sorted keys, no whitespace); "" if unreadable
  python3 -c 'import json,sys
try:
    print(json.dumps(json.load(open(sys.argv[1])), sort_keys=True, separators=(",",":")))
except Exception:
    print("")' "$1"
}
outer_kind() {  # $1 file -> the OUTER `kind` of the typed AST: the envelope is
                #   {"rule_name": "scratch", "span": …, "content": {"Json": <the declared value>}}
                # (TOOLBOX §1.2), so the value both oracles declare sits at content.Json.
  python3 -c 'import json,sys
try:
    v = json.load(open(sys.argv[1]))
except Exception:
    print("?"); sys.exit(0)
if isinstance(v, dict) and isinstance(v.get("content"), dict):
    v = v["content"].get("Json", v["content"])
print(v["kind"] if isinstance(v, dict) and "kind" in v else "?")' "$1"
}

interp_cell() {  # $1 input file, $2 profile, $3 ast out -> "accept:<kind>" | "reject@<pos>" | TOOL-ERROR
  local line
  rm -f "$3"
  line="$("$AST_PIPELINE" "$GRAMMAR" --interpret-parse "$1" --grammar-profile "$2" \
            --interpret-parse-ast-json "$3" 2>/dev/null | grep -m1 '^INTERPRET-PARSE: grammar=')"
  [[ -z "$line" ]] && { echo "TOOL-ERROR"; return; }
  if [[ "$line" == *"accepted=true"* ]]; then echo "accept:$(outer_kind "$3")"; else
    echo "reject@$(sed -E 's/.*furthest_position=([0-9]+).*/\1/' <<<"$line")"
  fi
}

gen_cell() {  # $1 input file, $2 profile, $3 ast out
  # The verdict comes from `--parse` (its reject line carries `furthest_position=`; the dump mode's
  # does not — measured, the first run read TOOL-ERROR on every reject); the AST is dumped on accept.
  local out
  rm -f "$3"
  out="$("$PROBE_BIN" --parse scratch "$1" --profile "$2" 2>&1)"
  if [[ "$out" == *"parse_full passed"* ]]; then
    "$PROBE_BIN" --parse-dump-ast-pretty scratch "$1" "$3" --profile "$2" >/dev/null 2>&1
    [[ -s "$3" ]] && echo "accept:$(outer_kind "$3")" || echo "TOOL-ERROR"
  elif [[ "$out" == *furthest_position=* ]]; then
    echo "reject@$(sed -E 's/.*furthest_position=([0-9]+).*/\1/' <<<"$out")"
  else echo "TOOL-ERROR"; fi
}

if [[ $INTERP_ONLY -eq 0 ]]; then
  # ⛔ Compose the slot as the COMMITTED HEADER + the probe's rules, never a whole-file copy: the slot's
  # 32-line header is its operating manual and `make focus_scratch` refuses a slot whose header is not
  # byte-identical to HEAD's (SCRATCH-SLOT-HEADER, TOOLBOX §1.3). The probe's own comment header is a
  # description, not the manual, so only its rule lines go in.
  {
    git show "HEAD:$SCRATCH" | sed -n '1,/^# =*$/p' | sed '$d'   # the manual, up to its closing rule
    git show "HEAD:$SCRATCH" | grep -m2 '^# =*$' | tail -1        # the closing rule of the header box
    echo ""
    echo "# ENGINE-UNIVERSAL-SERVICES.50 (c) — P6, loaded by probe_profiles.sh; see docs/tasks/artifacts/engine_universal_services/indirect_lr/$PROBE.ebnf"
    grep -v '^#' "$GRAMMAR"
  } >"$SCRATCH" || exit 2
  bash scripts/check_scratch_slot_header.sh --probe-time || { echo "the composed slot failed the header tier" >&2; exit 2; }
  make -C rust SHELL=/bin/bash focus_scratch >"$WORK/focus.log" 2>&1 ||
    { echo "focus_scratch FAILED — see $WORK/focus.log" >&2; exit 2; }
  grep -c 'Generating' "$WORK/focus.log" >/dev/null || { echo "focus_scratch did no generation — see $WORK/focus.log" >&2; exit 2; }
  (cd rust && cargo build --features generated_parsers --bin parseability_probe) >"$WORK/build.log" 2>&1 ||
    { echo "probe rebuild FAILED — see $WORK/build.log" >&2; exit 2; }
  "$PROBE_BIN" --parser-fingerprint >/dev/null 2>&1   # warm the fresh binary once (TOOLBOX: the two binaries)
fi

fail=0
printf '%-12s %-8s %-12s %-12s %-8s %-6s %-8s %s\n' INPUT PROFILE GEN INTERP AGREE AST EXPECT ROW
for input in "${INPUTS[@]}"; do
  printf '%s' "$input" >"$WORK/in.txt"
  for profile in "${PROFILES[@]}"; do
    exp="$(expected_for "$input" "$profile")"
    interp="$(interp_cell "$WORK/in.txt" "$profile" "$WORK/interp.json")"
    [[ "$interp" == TOOL-ERROR ]] && fail=2
    if [[ $INTERP_ONLY -eq 1 ]]; then
      gen="(skipped)"; agree="-"; ast="-"
      [[ "${interp%%@*}" == "$exp" ]] && row="ok" || { row="MISMATCH"; [[ $fail -eq 0 ]] && fail=1; }
    else
      gen="$(gen_cell "$WORK/in.txt" "$profile" "$WORK/gen.json")"
      [[ "$gen" == TOOL-ERROR ]] && fail=2
      # Verdict AND kind must agree; furthest_position legitimately differs on a reject.
      [[ "${gen%%@*}" == "${interp%%@*}" ]] && agree="yes" || { agree="DIVERGE"; [[ $fail -eq 0 ]] && fail=1; }
      if [[ "$gen" == accept:* && "$interp" == accept:* ]]; then
        [[ "$(canon_json "$WORK/gen.json")" == "$(canon_json "$WORK/interp.json")" && -n "$(canon_json "$WORK/gen.json")" ]] \
          && ast="same" || { ast="DIFF"; [[ $fail -eq 0 ]] && fail=1; }
        cp "$WORK/gen.json" "$WORK/ast_${profile}_$(tr -c 'a-z0-9' '_' <<<"$input" | sed 's/_*$//').json"
      else ast="-"; fi
      [[ "${gen%%@*}" == "$exp" && "${interp%%@*}" == "$exp" ]] && row="ok" || { row="MISMATCH"; [[ $fail -eq 0 ]] && fail=1; }
    fi
    printf '%-12s %-8s %-12s %-12s %-8s %-6s %-8s %s\n' "$input" "$profile" "$gen" "$interp" "$agree" "$ast" "$exp" "$row"
  done
done
if [[ $fail -eq 0 ]]; then
  echo "P6-CROSSING-GATES: OK (rows=$(( ${#INPUTS[@]} * ${#PROFILES[@]} )) oracles=$([[ $INTERP_ONLY -eq 1 ]] && echo interp || echo gen+interp) every row as pre-registered$([[ $INTERP_ONLY -eq 0 ]] && echo ', every GEN/INTERP cell agrees on verdict and AST'))"
else
  echo "P6-CROSSING-GATES: FAIL (rc=$fail) — a row above reads MISMATCH / DIVERGE / DIFF / TOOL-ERROR" >&2
fi
exit $fail
