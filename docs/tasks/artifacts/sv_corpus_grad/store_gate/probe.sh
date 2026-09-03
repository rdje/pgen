#!/usr/bin/env bash
# SV-CORPUS-GRAD.13w bank — the store-gate explanation class. Every arm asserts an OUTCOME the code
# produces (a verdict, a refusal text, a count), never that a file exists. RED arms drive the
# soundness conditions that keep a rejection in the bar; GREEN arms prove a real explanation on the
# real parser; the conservation arm ties the tracked manifests to the frozen before-census.
# Run: bash docs/tasks/artifacts/sv_corpus_grad/store_gate/probe.sh
# Run BY: scripts/check_sv_store_gate_bank.sh — the SV-STORE-GATE-BANK doctrine (SV-CORPUS-GRAD.15), on every
# commit and in CI; before it, this bank was run by nothing and sat RED for six commits.
set -u
cd "$(dirname "$0")/../../../../.." || exit 2
DECLARED=10
pass=0; fail=0
arm() { # arm <name> <rc>  (rc 0 = PASS)
  if [ "$2" -eq 0 ]; then pass=$((pass+1)); echo "  PASS  $1"; else fail=$((fail+1)); echo "  FAIL  $1"; fi
}
SG=stimuli/sv/store_gate_explanations.py
ADJ=stimuli/sv/adjudicate_external_corpus.py
HERE=docs/tasks/artifacts/sv_corpus_grad/store_gate

# A1 the module's own self-test (memo logic + live parse)
out=$(python3 "$SG" --self-test 2>&1); rc=$?
[ "$rc" -eq 0 ] && grep -q 'self-test: PASS (0 failing)' <<<"$out"; arm "A1 module self-test passes on the memo logic and the live parse" $?

# A2 the adjudicator's self-test (extension guard + store-gate post-pass arms)
out=$(python3 "$ADJ" --self-test 2>&1); rc=$?
[ "$rc" -eq 0 ] && grep -q 'store-gate self-test: PASS (0 failing)' <<<"$out"; arm "A2 adjudicator self-test passes, store-gate arms included" $?

# A3 RED: an unresolved wildcard import keeps the row in the bar (the boundary says ACCEPT there)
out=$(python3 - <<'PY'
import importlib.util
spec = importlib.util.spec_from_file_location("sg", "stimuli/sv/store_gate_explanations.py"); sg = importlib.util.module_from_spec(spec); spec.loader.exec_module(sg)
m = sg.machinery()
print(sg.soundness_reason(m.blank_comments_and_strings("import ext_pkg::*;\nmodule t; T x; endmodule\n"), ("T",)))
PY
); [ "$out" = "residual:unresolved_wildcard_import" ]; arm "A3 RED: unresolved wildcard import => residual:unresolved_wildcard_import (got: $out)" $?

# A4 RED: an LRM-predefined type name is a grammar defect, never a missing declaration
out=$(python3 - <<'PY'
import importlib.util
spec = importlib.util.spec_from_file_location("sg", "stimuli/sv/store_gate_explanations.py"); sg = importlib.util.module_from_spec(spec); spec.loader.exec_module(sg)
print(sg.soundness_reason("covergroup cg; endgroup\n", ("CrossQueueType",)))
PY
); [ "$out" = "residual:predefined_type_name" ]; arm "A4 RED: CrossQueueType demanded => residual:predefined_type_name (got: $out)" $?

# A5 RED: a binary that is not the report's refuses to measure
out=$(python3 - <<'PY'
import importlib.util, tempfile
from pathlib import Path
spec = importlib.util.spec_from_file_location("sg", "stimuli/sv/store_gate_explanations.py"); sg = importlib.util.module_from_spec(spec); spec.loader.exec_module(sg)
with tempfile.TemporaryDirectory(dir="rust/target") as td:
    f = Path(td) / "x.sv"; f.write_text("module x; T t; endmodule\n"); b = Path(td) / "probe"; b.write_bytes(b"not it")
    try:
        sg.explain_rows("sv_2017", [("s", "x.sv", f)], "0" * 64, b, {}, log=lambda *_: None); print("measured")
    except SystemExit as e:
        print("REFUSED" if "REFUSED" in str(e) else str(e))
PY
); [ "$out" = "REFUSED" ]; arm "A5 RED: binary != report's parse binary => measurement REFUSED (got: $out)" $?

# A6 GREEN: the real parser explains `T x;` by one class declaration
out=$(python3 - <<'PY'
import importlib.util
spec = importlib.util.spec_from_file_location("sg", "stimuli/sv/store_gate_explanations.py"); sg = importlib.util.module_from_spec(spec); spec.loader.exec_module(sg)
print(sg.measure("module t;\n  T x;\nendmodule\n", "sv_2017"))
PY
); [ "$out" = "('explained', ('T',), ('class',))" ]; arm "A6 GREEN live: 'T x;' => explained, names=T, forms=class (got: $out)" $?

# A7 conservation: per lane, explained + still-unexplained == the frozen before-count; the cache is exactly that population
out=$(python3 - <<'PY'
import csv, json
before = json.load(open("docs/tasks/artifacts/sv_corpus_grad/store_gate/before_census.json"))
cache = [r for r in csv.DictReader((l for l in open("stimuli/sv/characterization/store_gate_cache.tsv") if not l.startswith("#")), delimiter="\t")]
known = {"explained", "residual:parses_bare", "residual:not_parsed_with_declarations", "residual:unresolved_wildcard_import", "residual:package_export", "residual:predefined_type_name"}
ok = True; parts = []
for lane, mf in (("sv_2017", "stimuli/sv/characterization/adjudication_manifest.tsv"), ("verilog_2005", "stimuli/sv/characterization/adjudication_manifest_v2005.tsv")):
    rows = list(csv.DictReader(open(mf), delimiter="\t"))
    ex = sum(r["adjudication"] == "divergence:explained_store_gate" for r in rows)
    un = sum(r["adjudication"] == "divergence:unexplained_rejects_valid" for r in rows)
    crow = [r for r in cache if r["lane"] == lane]
    cex = sum(r["verdict"] == "explained" for r in crow)
    # SV-CORPUS-GRAD.13x.e: the population can only SHRINK after the freeze — a row leaves it by being
    # explained (counted here) or by a ruling / grammar fix that makes it `match` (.3.29, .3.30, .13x.b-d
    # moved 50 sv_2017 rows and 6 v2005 rows that way) — and never gains a member without a re-run of
    # the corpus lane. The first cut asserted EQUALITY to the frozen count and went RED, silently, at the
    # first `match` (-0347), for six commits: a conservation law that forbids the OTHER legitimate exit
    # is a law about the wrong quantity. What must hold exactly is the PARTITION: the memo holds exactly
    # the explained + still-unexplained rows, and its explained rows are the manifest's.
    good = (ex + un <= before[lane]["divergence:unexplained_rejects_valid"] and len(crow) == ex + un and cex == ex
            and all(r["verdict"] in known for r in crow) and all(r["names"] and r["forms"] for r in crow if r["verdict"] == "explained"))
    ok &= good; parts.append(f"{lane}: explained={ex} unexplained={un} <= before={before[lane]['divergence:unexplained_rejects_valid']} cache_rows={len(crow)} {'OK' if good else 'MISMATCH'}")
print(("CONSERVED " if ok else "BROKEN ") + "; ".join(parts))
PY
); grep -q '^CONSERVED' <<<"$out"; arm "A7 conservation (partition exact, population monotone): $out" $?

# A8 the instrument buckets the class ADJUDICATED and the conformance compare treats it as explained
out=$(python3 - <<'PY'
import importlib.util
def load(n, p):
    spec = importlib.util.spec_from_file_location(n, p); m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m); return m
cov = load("cov", "stimuli/sv/corpus_verdict_coverage.py"); cmp_ = load("cmp_", "stimuli/sv/corpus_conformance_compare.py")
b = cov.CLASSES.get("divergence:explained_store_gate", ("?",))[0]
tracked = {("s", "a.sv"): {"observed": "fail", "adjudication": "divergence:explained_store_gate"}}
fresh = {("s", "a.sv"): {"observed": "fail", "adjudication": "divergence:unexplained_rejects_valid"}}
r = cmp_.compare(tracked, fresh, lane="sv_2017")
print(b, r.get("regressions"), r.get("explained"), r.get("unexplained"))
PY
); [ "$out" = "ADJUDICATED 1 0 1" ]; arm "A8 instrument bucket=ADJUDICATED, compare flags explained->unexplained as a regression (got: $out)" $?

# A9 a published explanation re-derives: the first explained sv_2017 cache row is measured again, live
out=$(python3 - <<'PY'
import csv, importlib.util
from pathlib import Path
spec = importlib.util.spec_from_file_location("sg", "stimuli/sv/store_gate_explanations.py"); sg = importlib.util.module_from_spec(spec); spec.loader.exec_module(sg)
rows = [r for r in csv.DictReader((l for l in open("stimuli/sv/characterization/store_gate_cache.tsv") if not l.startswith("#")), delimiter="\t") if r["lane"] == "sv_2017" and r["verdict"] == "explained"]
r = rows[0]
p = Path("stimuli/sv/subs") / r["suite"] / r["relpath"]
v = sg.measure(p.read_bytes().decode("latin-1"), "sv_2017")
print("REDERIVED" if v == ("explained", tuple(r["names"].split(",")), tuple(r["forms"].split("+"))) else f"DRIFT {v} vs {r}", r["suite"], r["relpath"])
PY
); grep -q '^REDERIVED' <<<"$out"; arm "A9 a tracked explanation re-derives live ($out)" $?

# A10 (SV-CORPUS-GRAD.15) the memo key carries the INSTRUMENT: an instrument edit re-measures every
# row, a no-op re-measures none, and every TRACKED row carries the live producers' digest.
out=$(python3 - <<'PY'
import csv, importlib.util, tempfile
from pathlib import Path
spec = importlib.util.spec_from_file_location("sg", "stimuli/sv/store_gate_explanations.py"); sg = importlib.util.module_from_spec(spec); spec.loader.exec_module(sg)
calls = []
def fake(raw, profile):
    calls.append(raw); return (sg.EXPLAINED, ("T",), ("class",))
with tempfile.TemporaryDirectory(dir="rust/target") as td:
    f = Path(td) / "a.sv"; f.write_text("module a; T x; endmodule\n"); nb = Path(td) / "nobinary"
    cache = {}
    sg.explain_rows("sv_2017", [("s", "a.sv", f)], "b" * 64, nb, cache, measurer=fake, log=lambda *_: None, instrument_sha="1" * 64); first = len(calls); calls.clear()
    sg.explain_rows("sv_2017", [("s", "a.sv", f)], "b" * 64, nb, cache, measurer=fake, log=lambda *_: None, instrument_sha="1" * 64); noop = len(calls); calls.clear()
    sg.explain_rows("sv_2017", [("s", "a.sv", f)], "b" * 64, nb, cache, measurer=fake, log=lambda *_: None, instrument_sha="2" * 64); edited = len(calls)
live = sg.instrument_sha256()
rows = list(csv.DictReader((l for l in open("stimuli/sv/characterization/store_gate_cache.tsv") if not l.startswith("#")), delimiter="\t"))
stale = sum(1 for r in rows if r.get("instrument_sha256") != live)
print(f"first={first} noop={noop} edited={edited} tracked_rows={len(rows)} tracked_stale={stale}")
PY
); grep -Eq '^first=1 noop=0 edited=1 tracked_rows=[0-9]+ tracked_stale=0$' <<<"$out"; arm "A10 the memo key carries the instrument digest: an edit re-measures, a no-op does not, every tracked row carries the live digest ($out)" $?

echo "store_gate bank: ${pass}/${DECLARED} as declared (fail=${fail})"
[ "$pass" -eq "$DECLARED" ] && [ "$fail" -eq 0 ]
