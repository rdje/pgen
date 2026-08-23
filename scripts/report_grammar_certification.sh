#!/usr/bin/env bash
# GRAMMAR CERTIFICATION STATUS — MEASURED, never inferred from which files happen to exist.
#
# THE DEFINITION (PGEN's own, GRAMMAR-WELLFORMED.G.4 — the linter<->generator duality capstone,
# quoted from `ast_pipeline --report-certificate-coverage`'s own help text):
#
#   "For every rule, is it covered by a verified unreachability PROOF or a verified reachability
#    WITNESS (a clean diverse --count sample that parses through the real parser and exercises it)?
#    UNKNOWN=0 with no failures = the objective 'trustworthy on this grammar' number."
#
# So a grammar is CERTIFIED iff, for every rule, either a verified PROOF says no input can reach it
# or a generated sample WITNESSES it through the real generated parser — with UNKNOWN=0,
# sample_parse_failures=0 and proof_reverify_failures=0.
#
# ⛔⛔ THIS SCRIPT'S FIRST VERSION WAS WRONG AND PUBLISHED A FALSE TABLE TO THE MAIN BOOK
# (GRAMMAR-CERT-STATUS.1, corrected same day under a director challenge). It asked
# "does a *cert*contract*.json exist for this family?" and reported NO ORACLE for five families that
# are demonstrably `fully_certified=true` when you simply RUN the oracle. The certificate-coverage
# report IS the oracle; a tracked contract is only a PIN on top of it. ⇒ ask the instrument, never
# the filesystem. This is the third time in one session that a verdict was taken under MY chosen
# parameters instead of the subject's own.
#
# usage: report_grammar_certification.sh [--markdown] [--check FILE] [--seed N] [--count N]
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2
MODE=text; CHECK=""; SEED=0; COUNT=40
while [ $# -gt 0 ]; do
  case "$1" in
    --markdown) MODE=markdown ;;
    --check) CHECK="${2:-}"; MODE=markdown; shift ;;
    --seed) SEED="${2:-0}"; shift ;;
    --count) COUNT="${2:-40}"; shift ;;
    *) echo "report_grammar_certification: unknown arg '$1'" >&2; exit 2 ;;
  esac; shift
done
BIN="rust/target/debug/ast_pipeline"
[ -x "$BIN" ] || { echo "grammar-certification: REFUSED — no ast_pipeline at $BIN" >&2; exit 2; }

# Per-family generation parameters that are NOT the CLI default. ⛔ Reading these is load-bearing:
# judging rtl_const_expr at the default --max-depth 24 reports a family that generates NOTHING,
# while its own cert contract declares 32 and it certifies cleanly there (GRAMMAR-WELLFORMED.H.23).
declare_extra() { case "$1" in rtl_const_expr) echo "--max-depth 32" ;; *) echo "" ;; esac; }

emit() {
  echo "# meta seed=$SEED count=$COUNT"
  for g in $(ls generated/*_parser.rs 2>/dev/null | sed 's|generated/||; s|_parser\.rs||' | grep -v '^scratch$' | sort); do
    [ -f "grammars/$g.ebnf" ] || { echo "$g|NO GRAMMAR|||"; continue; }
    if [ "$g" = "systemverilog" ]; then echo "$g|SKIP|||"; continue; fi
    line=$(timeout 900 "$BIN" "grammars/$g.ebnf" --report-certificate-coverage \
             --count "$COUNT" --seed "$SEED" $(declare_extra "$g") 2>&1 \
           | grep -E '^CERTIFICATE-COVERAGE:' | head -1)
    echo "$g|$line"
  done
}
RAW="$(emit)"

RGC_RAW="$RAW" python3 - "$MODE" "$SEED" "$COUNT" <<'PY'
import json, os, pathlib, re, sys, hashlib
MODE, SEED, COUNT = sys.argv[1], sys.argv[2], sys.argv[3]
raw = os.environ["RGC_RAW"]
rows = []
for ln in raw.strip().split("\n"):
    if ln.startswith("#") or "|" not in ln: continue
    fam, rest = ln.split("|", 1)
    if rest.startswith("SKIP") or rest.startswith("NO GRAMMAR"):
        rows.append((fam, None)); continue
    d = {k: v for k, v in re.findall(r"(total|proof|witness|UNKNOWN)=(\d+)", rest)}
    fc = "fully_certified=true" in rest
    spf = re.search(r"sample_parse_failures=(\d+)", rest)
    rows.append((fam, {"total": d.get("total"), "proof": d.get("proof"), "witness": d.get("witness"),
                       "unknown": d.get("UNKNOWN"), "certified": fc,
                       "spf": spf.group(1) if spf else "?"}))

# SystemVerilog is read from its tracked contract, NOT re-measured here: its accounting is a UNION
# over four entry/profile configs at ~2 min/seed, which does not belong in a status command.
sv = json.loads(pathlib.Path("rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json").read_text())
pin = (sv.get("identity", {}).get("inputs") or {}).get("generated/systemverilog_parser.rs")
p = pathlib.Path("generated/systemverilog_parser.rs")
live = hashlib.sha256(p.read_bytes()).hexdigest() if p.exists() else ""
sv_fresh = "fresh" if (pin and live == pin) else ("STALE" if pin else "UNPINNED")

def fmt(r):
    return f"{r['total']} rules · {r['witness']} witness · {r['proof']} proof · {r['unknown']} unknown"

out = []
if MODE == "markdown":
    out.append("<!-- BEGIN DERIVED: scripts/report_grammar_certification.sh --markdown -->")
    out.append("")
    out.append(f"Measured at seed {SEED}, `--count {COUNT}`, against the generated parsers in the tree.")
    out.append("")
    out.append("| grammar | certified | coverage | notes |")
    out.append("|---|---|---|---|")
    for fam, r in rows:
        if r is None: continue
        out.append(f"| `{fam}` | {'✅ **yes**' if r['certified'] else '⛔ **no**'} | {fmt(r)} | "
                   + ("every rule proven or witnessed" if r['certified']
                      else f"**{r['unknown']} rule(s) UNKNOWN** — no generated sample reaches them and no proof covers them") + " |")
    out.append(f"| `systemverilog` | ⚠️ **unverified** | {sv['expected_total']} rules · {sv['expected_union_witness']} witness · "
               f"{sv['expected_proof']} proof · {sv['expected_union_unknown']} unknown (canonical unknown {sv['expected_canonical_unknown']}) | "
               f"from the tracked union contract, **not re-measured here** (~2 min/seed × 3 seeds × 4 configs). "
               f"Proof freshness vs the parser in the tree: **{sv_fresh}** |")
    n_cert = sum(1 for _, r in rows if r and r['certified'])
    n_tot = sum(1 for _, r in rows if r) + 1
    out.append("")
    out.append(f"**{n_cert} of {n_tot} shipped grammars are certified** — every rule proven or witnessed, UNKNOWN=0.")
    out.append("")
    out.append("<!-- END DERIVED -->")
else:
    out.append(f"GRAMMAR-CERTIFICATION (measured, seed={SEED} count={COUNT})")
    for fam, r in rows:
        if r is None: continue
        out.append(f"  {fam:28s} {'CERTIFIED    ' if r['certified'] else 'NOT CERTIFIED'} {fmt(r)}  spf={r['spf']}")
    out.append(f"  {'systemverilog':28s} UNVERIFIED    union {sv['expected_union_unknown']} unknown "
               f"(canonical {sv['expected_canonical_unknown']}) — from contract, freshness={sv_fresh}")
    n_cert = sum(1 for _, r in rows if r and r['certified'])
    out.append(f"GRAMMAR-CERTIFICATION: certified={n_cert}/{sum(1 for _, r in rows if r) + 1}")
print("\n".join(out))
PY
