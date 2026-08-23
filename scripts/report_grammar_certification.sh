#!/usr/bin/env bash
# GRAMMAR CERTIFICATION STATUS — DERIVED, never hand-written.
#
# Answers, for every shipped grammar family: is it CERTIFIED, and if not, WHY.
#
# ⛔ WHY THIS EXISTS (GRAMMAR-CERT-STATUS.1, director 2026-08-23). Asked "is the SV parser fully
# certified?", the honest answer took a dozen commands and turned out to be *no, and the last yes
# was scored against a bar we wrote and never re-run*. Two failures made that possible and this
# report is aimed at both:
#   1. the roster of certified families was DOC-ASSERTED — `CHANGES.md` records rtl_const_expr's
#      "fully-certified-6 roster membership was doc-asserted only", i.e. a claim with no oracle;
#   2. a certification can be GREEN and STALE at once — SV's proof is dated a full day BEFORE the
#      parser it describes, and every doctrine still reported PASS.
# ⇒ a status line is worthless without its FRESHNESS, so every row carries both.
#
# ⛔ SCOPE, stated so the report is not read as more than it is. "Certified" here is this repo's
# CERTIFICATE-COVERAGE definition: can the stimuli generator reach every rule of the grammar? It is
# a claim about the grammar's own internal reachability. It is NOT a claim that the parser is
# correct for the LANGUAGE — that is the corpus axis, reported separately per family.
#
# usage: report_grammar_certification.sh [--check FILE] [--markdown]
#   --markdown  emit the book table
#   --check F   regenerate and DIFF against F; nonzero if they differ (the derive-and-diff pattern
#               KNOWLEDGE-MAP uses, so the published page cannot drift from the tree)
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2
MODE=text; CHECK=""
while [ $# -gt 0 ]; do
  case "$1" in
    --markdown) MODE=markdown ;;
    --check) CHECK="${2:-}"; MODE=markdown; shift ;;
    *) echo "report_grammar_certification: unknown arg '$1'" >&2; exit 2 ;;
  esac; shift
done

emit() {
python3 - <<'PY'
import glob, hashlib, json, os, pathlib, sys

ROOT = pathlib.Path.cwd()
MODE = os.environ.get("RGC_MODE", "text")

def sha(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()

# The POPULATION is the set of shipped generated parsers — derived, never a hand-kept list, because
# a hand-kept roster is the exact defect this report exists to retire.
fams = sorted(pathlib.Path(p).name[:-len("_parser.rs")]
              for p in glob.glob("generated/*_parser.rs"))
fams = [f for f in fams if f != "scratch"]   # the blessed throwaway slot, not a family

rows = []
for fam in fams:
    parser = pathlib.Path(f"generated/{fam}_parser.rs")
    contract = None
    for cand in glob.glob(f"rust/test_data/grammar_quality/{fam}*cert*contract*.json"):
        contract = pathlib.Path(cand); break
    gate = None
    for cand in glob.glob(f"rust/scripts/*{fam}*cert*gate.sh") + glob.glob("rust/scripts/sv_cert_recognized_union_gate.sh") if fam == "systemverilog" else glob.glob(f"rust/scripts/{fam}*cert*gate.sh"):
        gate = pathlib.Path(cand).name; break

    verdict, why, nums, fresh = "NO ORACLE", "", "", "—"
    if contract is None:
        why = "no certificate-coverage contract exists for this family, so nothing has ever scored its rule reachability"
    else:
        d = json.loads(contract.read_text())
        total = d.get("expected_total")
        proof = d.get("expected_proof")
        wit = d.get("expected_union_witness", d.get("expected_witness"))
        unk = d.get("expected_union_unknown", d.get("expected_unknown"))
        canon_unk = d.get("expected_canonical_unknown")
        nums = f"{total} rules · {wit} witness · {proof} proof · {unk} unknown"
        if canon_unk is not None:
            nums += f" (canonical unknown {canon_unk})"

        ident = d.get("identity") or {}
        pinned = (ident.get("inputs") or {}).get(str(parser))
        if pinned is None:
            fresh = "UNPINNED"
        else:
            live = sha(parser) if parser.exists() else ""
            fresh = "fresh" if live == pinned else "STALE"

        if unk == 0 and fresh == "fresh":
            verdict, why = "CERTIFIED", "every rule is positively covered, and the proof is pinned to the parser in the tree"
        elif unk == 0 and fresh == "STALE":
            verdict = "UNVERIFIED"
            why = ("the last run reached zero unknown, but the contract pins a different "
                   "`" + parser.name + "` than the one in the tree — the proof no longer describes this parser")
        elif unk == 0 and fresh == "UNPINNED":
            verdict = "UNVERIFIED"
            why = ("the last run reached zero unknown, but the contract carries no identity block, "
                   "so nothing can say which tree it was measured against")
        else:
            verdict = "NOT CERTIFIED"
            why = f"{unk} rule(s) are still unknown — no config produces a string that reaches them"
    rows.append((fam, verdict, fresh, nums, why, contract.name if contract else "—", gate or "—"))

if MODE == "markdown":
    print("<!-- BEGIN DERIVED: scripts/report_grammar_certification.sh --markdown -->")
    print()
    print("| grammar | certification | proof freshness | coverage | why |")
    print("|---|---|---|---|---|")
    ico = {"CERTIFIED": "✅", "UNVERIFIED": "⚠️", "NOT CERTIFIED": "⛔", "NO ORACLE": "⛔"}
    for fam, v, fr, n, why, c, g in rows:
        print(f"| `{fam}` | {ico.get(v,'')} **{v}** | {fr} | {n or '—'} | {why} |")
    print()
    print("| grammar | contract | standing gate |")
    print("|---|---|---|")
    for fam, v, fr, n, why, c, g in rows:
        print(f"| `{fam}` | `{c}` | `{g}` |")
    print()
    cert = sum(1 for r in rows if r[1] == "CERTIFIED")
    print(f"**{cert} of {len(rows)} shipped grammars are certified and fresh.**")
    print()
    print("<!-- END DERIVED -->")
else:
    print(f"GRAMMAR-CERTIFICATION: {len(rows)} shipped families")
    for fam, v, fr, n, why, c, g in rows:
        print(f"  {fam:28s} {v:15s} freshness={fr:9s} {n}")
        if why: print(f"  {'':28s}   why: {why}")
    cert = sum(1 for r in rows if r[1] == "CERTIFIED")
    print(f"GRAMMAR-CERTIFICATION: certified_and_fresh={cert}/{len(rows)}")
PY
}

if [ -n "$CHECK" ]; then
  RGC_MODE=markdown emit > "$ROOT/rust/target/rgc_fresh.md" || exit 2
  if [ ! -f "$CHECK" ]; then echo "grammar-certification: REFUSED — no published page at $CHECK" >&2; exit 2; fi
  python3 - "$CHECK" "$ROOT/rust/target/rgc_fresh.md" <<'PY'
import pathlib, sys, re
pub, fresh = pathlib.Path(sys.argv[1]).read_text(), pathlib.Path(sys.argv[2]).read_text()
m = re.search(r"<!-- BEGIN DERIVED.*?<!-- END DERIVED -->", pub, re.S)
if not m:
    sys.exit("grammar-certification: REFUSED — the published page carries no DERIVED block")
if m.group(0).strip() != fresh.strip():
    sys.exit("grammar-certification: ✗ the published table is OUT OF SYNC with the tree — "
             "re-run scripts/report_grammar_certification.sh --markdown and republish")
print("grammar-certification: OK (published table equals a fresh derivation)")
PY
  exit $?
fi
RGC_MODE="$MODE" emit
