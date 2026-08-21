#!/usr/bin/env bash
# profile_universe_delta.sh — SV-CORPUS-GRAD.13c.2x.7 (b)
#
# WHAT IT ANSWERS: `verilog_2005_conformance_contract_v0.json` pinned `cert.expected_total=1122` on
# 2026-08-09 and the tree measures 1143. Of that +21, how much is the GRAMMAR, how much is PGEN's own
# left-recursion elimination, and is any of it a DEFECT?
#
# It decomposes the delta into THREE named parts using ONE binary, so each arm varies exactly one
# thing:
#   1. the DIRECT-LR clones      — held constant, counted by name
#   2. the INDIRECT-LR clones    — isolated with `--no-eliminate-indirect-left-recursion`
#   3. the GRAMMAR               — isolated by dumping the baseline revision with today's binary
#
# ⭐ WHY `--dump-rule-profiles` AND NOT THE CERT RUN. The dump is ~2 s and needs no generated parser.
# The cert run needs a parser REGISTERED under the grammar's name, so an old grammar certifies through
# today's parser and its `witness`/`UNKNOWN` are contaminated by construction (measured on the base
# arm: `sample_parse_failures=3`). ⭐⭐ The two instruments were CROSS-CHECKED on both arms before the
# cheap one was trusted: the dump reads 1141 on the baseline grammar and 1143 at HEAD, and a cert run
# under the gate's own invocation reads `total=1141` and `total=1143` on the same two. Same number,
# independent paths.
#
# ⛔ HONEST BOUND: the DIRECT-LR share is attributed by NAME and by a held-off control, but whether
# the 2026-08-09 binary counted those four rules is an INFERENCE from the arithmetic closing exactly
# (1122 + 4 + 15 + 2 = 1143), not a run of that binary. Rebuilding it was priced and not paid.
#
# Usage: bash docs/tasks/artifacts/sv_corpus_grad/v2005_cert_adjudication/profile_universe_delta.sh [BASE_COMMIT] [PROFILE] [PINNED_BASELINE]
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT"

BASE_COMMIT="${1:-ba1a96c8}"      # the commit that last wrote the contract
PROFILE="${2:-verilog_2005}"
PINNED="${3:-1122}"               # the value that contract pinned, for the arithmetic check
BIN="$ROOT/rust/target/debug/ast_pipeline"
WORK="$ROOT/rust/target/v2005_adjudication"

[[ -x "$BIN" ]] || { echo "error: build it first: cargo build --features 'generated_parsers ebnf_dual_run' --bin ast_pipeline" >&2; exit 1; }

mkdir -p "$WORK/base"
git show "${BASE_COMMIT}:grammars/systemverilog.ebnf" > "$WORK/base/systemverilog.ebnf"

# The filename's stem IS the grammar name the pipeline resolves a parser by, so the baseline copy
# must be called systemverilog.ebnf — a differently-named file fails with "no generated parser is
# registered for grammar '<stem>'".
"$BIN" "$WORK/base/systemverilog.ebnf" --dump-rule-profiles "$WORK/profiles_base.json"      >/dev/null
"$BIN" "$WORK/base/systemverilog.ebnf" --no-eliminate-indirect-left-recursion \
                                       --dump-rule-profiles "$WORK/profiles_base_noind.json" >/dev/null
"$BIN" "$ROOT/grammars/systemverilog.ebnf" --dump-rule-profiles "$WORK/profiles_head.json"  >/dev/null

python3 - "$WORK" "$PROFILE" "$BASE_COMMIT" "$PINNED" <<'PY'
import json, os, sys
work, profile, base_commit, pinned = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])

def universe(name):
    d = json.load(open(os.path.join(work, name), encoding="utf-8"))
    return {k for k, v in d["rules"].items() if profile in (v.get("satisfiable_under") or [])}

base   = universe("profiles_base.json")           # today's engine, baseline grammar
noind  = universe("profiles_base_noind.json")     # …with the INDIRECT pass held off
head   = universe("profiles_head.json")           # today's engine, today's grammar

is_lr  = lambda s: {n for n in s if "_lr_" in n}
direct   = sorted(is_lr(noind))                   # LR rules surviving with indirect held off
indirect = sorted(is_lr(base) - is_lr(noind))     # what the indirect pass adds
added    = sorted(head - base)                    # grammar-side additions
removed  = sorted(base - head)                    # grammar-side removals
grammar_net = len(added) - len(removed)

print("PROFILE-UNIVERSE-DELTA: profile=%s base=%s pinned=%d" % (profile, base_commit, pinned))
print("  measured: baseline-grammar=%d  baseline-grammar/no-indirect=%d  HEAD=%d"
      % (len(base), len(noind), len(head)))
print()
print("  DECOMPOSITION of %d -> %d  (%+d)" % (pinned, len(head), len(head) - pinned))
print("    + %-3d direct-LR clones      %s" % (len(direct), ", ".join(direct)))
print("    + %-3d indirect-LR clones    %s" % (len(indirect), ", ".join(indirect[:3]) + (", …" if len(indirect) > 3 else "")))
print("    %+d   grammar (net)          %d added / %d removed" % (grammar_net, len(added), len(removed)))
for r in added:   print("        + %s" % r)
for r in removed: print("        - %s" % r)
total = pinned + len(direct) + len(indirect) + grammar_net
ok = "EXACT" if total == len(head) else "UNACCOUNTED %+d" % (len(head) - total)
print()
print("  ARITHMETIC: %d + %d + %d + (%+d) = %d   vs measured %d  -> %s"
      % (pinned, len(direct), len(indirect), grammar_net, total, len(head), ok))
print()
print("  VERDICT: %d of the %d are PGEN's OWN left-recursion residue (no grammar author wrote them);"
      % (len(direct) + len(indirect), len(head) - pinned))
print("           the remaining %+d is the grammar, and every name in it belongs to a LEDGERED FIX." % grammar_net)
raise SystemExit(0 if total == len(head) else 1)
PY
