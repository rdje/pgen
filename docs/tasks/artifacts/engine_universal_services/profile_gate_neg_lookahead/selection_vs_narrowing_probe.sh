#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.46 (c) — THE PROBE THAT SEPARATES THE TWO USES OF A `@profiles` GATE.
#
# WHY THIS FILE EXISTS
# --------------------
# `.46`(c)'s first cut bypassed the profile gate inside EVERY negative lookahead whose body could
# reach a `@profiles`-gated rule. That is the wrong condition, and it would have shipped a
# regression in the REJECTING direction. A gate is used two ways:
#
#   NARROWING  the rule is simply absent from the narrow dialect and nothing replaces it. Gate it
#              out and `!X` has nothing to match, so the guard is DELETED and the narrow profile
#              accepts strings the wide one rejects. ⇒ THE DEFECT. The bypass must apply.
#
#   SELECTION  sibling rules, one per dialect, behind a dispatcher. `grammars/systemverilog.ebnf`:
#                reserved_non_keyword_identifier := reserved_non_keyword_identifier_sv       @profiles ["sv_2017","sv_2023"]
#                                                 | reserved_non_keyword_identifier_v2005    @profiles ["verilog_2005"]
#              Under any declared profile ONE alternative is live, so the guard is SWITCHED, never
#              deleted. ⇒ NOT a defect. The bypass must NOT apply — applying it would make the guard
#              see BOTH dialects' keyword lists at once.
#
# ⛔ THE COST OF GETTING IT WRONG, MEASURED: `class` is a legal `verilog_2005` identifier and an
# SV-2017 keyword. Under the blanket bypass, `reg class;` would have stopped parsing under
# `verilog_2005` — a rejection introduced by a fix for an over-acceptance. Arm [2] is that exact
# input, and it is the arm that decides whether the repair is aimed correctly.
#
# The condition that separates them is SATISFIABILITY of the lookahead BODY under the profile, not
# reachability of a gated rule from it. SELECTION stays satisfiable; NARROWING does not.
#
# Usage: bash docs/tasks/artifacts/engine_universal_services/profile_gate_neg_lookahead/selection_vs_narrowing_probe.sh

set -uo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT_DIR" || exit 2

PIPE="rust/target/debug/ast_pipeline"
SV="grammars/systemverilog.ebnf"
SCRATCH="rust/target/es46_selection_probe"

if [[ ! -x "$PIPE" ]]; then
    printf 'REFUSED: %s is absent — build it first:\n' "$PIPE" >&2
    printf '  cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline\n' >&2
    exit 2
fi

rm -rf "$SCRATCH"; mkdir -p "$SCRATCH"
trap 'rm -rf "$SCRATCH"' EXIT
pass=0; fail=0

verdict() { # verdict <text> <profile>
    printf '%s' "$1" > "$SCRATCH/in.sv"
    "$PIPE" "$SV" --interpret-parse "$SCRATCH/in.sv" --grammar-profile "$2" 2>&1 \
        | grep -o 'accepted=[a-z]*' | head -1
}
expect() { # expect <label> <actual> <wanted>
    if [[ "$2" == "$3" ]]; then printf '  ✅ %-62s %s\n' "$1" "$2"; pass=$((pass+1))
    else printf '  ❌ %-62s %s (wanted %s)\n' "$1" "$2" "$3"; fail=$((fail+1)); fi
}

echo "SELECTION-vs-NARROWING PROBE — ENGINE-UNIVERSAL-SERVICES.46 (c)"
echo
echo "[1] ⭐⭐⭐ SELECTION MUST BE UNTOUCHED — the arm that catches an over-applied bypass"
# `class` is NOT reserved in IEEE 1364-2005 and IS reserved in IEEE 1800. The dispatcher's per-
# dialect alternatives are what make this work, and the guard must keep SWITCHING rather than union.
CLASS_IDENT=$'module m;\n  reg class;\nendmodule\n'
expect "'reg class;' is a legal verilog_2005 identifier -> accept" "$(verdict "$CLASS_IDENT" verilog_2005)" accepted=true
expect "'reg class;' is an SV-2017 KEYWORD            -> reject" "$(verdict "$CLASS_IDENT" sv_2017)"       accepted=false

echo
echo "[2] the mirror: a v2005-only keyword used as an identifier under SV"
# `wait` is reserved in BOTH, so it is not a discriminator; `macromodule` is v2005-reserved and not
# an SV-2017 keyword, which is the opposite asymmetry from `class`.
MACRO_IDENT=$'module m;\n  reg macromodule;\nendmodule\n'
expect "'reg macromodule;' is a v2005 KEYWORD          -> reject" "$(verdict "$MACRO_IDENT" verilog_2005)" accepted=false

echo
echo "[3] NARROWING — the guarded token cannot appear under the narrow dialect either way"
# `!scope_resolution` IS the narrowing shape (one gated rule, no sibling), so the bypass DOES apply
# here. Nothing under verilog_2005 can consume `::`, so the verdict is REJECT before and after — the
# repair restores the constraint without moving this verdict, which is why the corpus does not move.
SCOPED=$'module m;\n  assign x = p::b;\nendmodule\n'
expect "'p::b' rejects under verilog_2005 (no :: token) -> reject" "$(verdict "$SCOPED" verilog_2005)" accepted=false
expect "'p::b' parses under sv_2017                     -> accept" "$(verdict "$SCOPED" sv_2017)"       accepted=true

echo
printf 'RESULT: pass=%d fail=%d\n' "$pass" "$fail"
[[ $fail -eq 0 ]] || exit 1
exit 0
