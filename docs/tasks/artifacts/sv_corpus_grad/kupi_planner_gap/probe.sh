#!/usr/bin/env bash
# probe.sh — SV-CORPUS-GRAD.13c.2x.9 (c)
#
# WHY the reach planner's own carrier cannot witness `known_unscoped_property_identifier`,
# SystemVerilog's last certificate-union `UNKNOWN` — and the ONE change that makes it witness.
#
# ⭐ THE PLAN IS CORRECT AND THE RENDER IS CORRECT. `reach_path.txt` pins the installed hop chain,
# and it is right hop for hop down to the branch INDEX: `bind_instantiation` at `root/o3` is
# `checker_instantiation`, `prop_primary_sv_2017` at `root/o25` is `property_instance`, and
# `ps_or_hierarchical_property_identifier` at `root/o1` is the target itself. Exactly ONE distinct
# chain is installed across the whole cert run — the store-free pass, whose entire purpose is to
# re-route around store-gated edges, installs the SAME one and reports `0 witnessed`.
#
# ⛔⛔ THE FAILURE IS A PARSE-TIME STORE GATE ON A **MANDATORY SIBLING** OF THE REACH PATH.
# `checker_instantiation := ps_checker_identifier name_of_instance lparen … rparen semi`. The reach
# plan steers the ARGUMENT (`list_of_checker_port_connections` onwards); nothing steers
# `ps_checker_identifier`, which is not on the path to the target but IS mandatory for the sample to
# parse as a checker instantiation at all. The generator draws its unscoped alternative, whose
# `has_fact(checker_name, …)` post-predicate then REJECTS — the prelude declares a package and a
# property, never a checker. With the checker branch dead, `bind_instantiation` commits
# `program_instantiation` instead (the two shapes are textually identical here), the whole
# property-expression subtree is never entered, and the target commits 0.
#
# ⭐⭐ AND THE SAMPLE STILL PARSES, WHICH IS WHY THIS WAS MISREAD FOR TWO SLICES. TOOLBOX's cause map
# gives a store-gate failure the signature `parsed=false`; that signature only catches a gate on the
# TARGET. A gate on an intermediate hop re-routes the parse to a different alternative and the file
# parses perfectly — as something else — so it presents as `parsed=true witnessed_target=false` and
# reads like a grammar defect.
#
# ⛔ RULED OUT BY MEASUREMENT, not by argument:
#   depth  — TOOLBOX 4.5's NOT-depth check at `--max-depth` 24 / 32 / 40 leaves the UNKNOWN set
#            unchanged (`total=1385 … UNKNOWN=1`, same single residual).
#   time   — `PGEN_WITNESS_TIMEOUT_FLOOR_MS` 2000 and 20000 (100x the 200 ms default) are identical.
#   The `depth exceeded max_depth=61` reasons in `PGEN_REACH_FORCED_OVERRIDE_DUMP=1` are a SYMPTOM of
#   forcing a 19-hop chain, not the cause.
#
# THE TWO ESCAPES, both proven below — either one makes the same plan witness:
#   scoped_checker.sv   `p::chk` — needs NO checker declaration, only the `package_name` fact the
#                       prelude ALREADY plants. The generator has everything it needs today.
#   declared_checker.sv `checker chk(…); endchecker` in the prelude, then the bare name.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT"
PROBE="$ROOT/rust/target/release/parseability_probe"
[[ -x "$PROBE" ]] || { echo "error: build it: make -C rust SHELL=/bin/bash focus_systemverilog && cargo build --release --features generated_parsers --bin parseability_probe" >&2; exit 1; }
HERE="$(dirname "${BASH_SOURCE[0]}")"
OUT="$ROOT/rust/target/kupi_diag"; mkdir -p "$OUT"

disk="$(shasum -a 256 "$ROOT/generated/systemverilog_parser.rs" | cut -d' ' -f1)"
emb="$("$PROBE" --parser-fingerprint | python3 -c 'import json,sys; print(json.load(sys.stdin)["parsers"]["systemverilog"])')"
[[ "$disk" == "$emb" ]] || { echo "error: probe embeds systemverilog $emb but disk carries $disk — rebuild before trusting any row" >&2; exit 1; }
echo "# parser sha256 (binary == disk): $disk"
echo "# committed counts (--dump-rule-outcome-counts-json, C3-B) for the reach chain"

RULES="checker_instantiation program_instantiation ordered_checker_port_connection property_actual_arg property_instance known_unscoped_property_identifier"
printf '%-20s' 'carrier'; for r in $RULES; do printf '%-14s' "$(echo "$r" | cut -c1-13)"; done; printf '%s\n' 'VERDICT'
for f in "$HERE"/*.sv; do
    b="$(basename "$f" .sv)"
    "$PROBE" --parse systemverilog "$f" --profile sv_2017 \
        --dump-rule-outcome-counts-json "$OUT/pg_$b.outcome.json" >/dev/null 2>&1
    python3 "$HERE/../kupi_shadowing/kupi_arms.py" outcome "$OUT/pg_$b.outcome.json" >/dev/null 2>&1 || true
    row="$(python3 - "$OUT/pg_$b.outcome.json" $RULES <<'PY'
import json, sys
try: d = json.load(open(sys.argv[1]))
except Exception: print("  ".join(["-"] * (len(sys.argv) - 2)) + "  NO-PARSE"); raise SystemExit
com = d.get("rule_committed_counts", {})
cells = ["".join("%-14s" % com.get(r, 0)) for r in sys.argv[2:]]
verdict = "WITNESSES" if com.get("known_unscoped_property_identifier", 0) > 0 else "does-not-witness"
print("".join(cells) + verdict)
PY
)"
    printf '%-20s%s\n' "$b" "$row"
done
