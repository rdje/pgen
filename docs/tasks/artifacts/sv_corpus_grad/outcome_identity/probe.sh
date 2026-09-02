#!/usr/bin/env bash
# docs/tasks/artifacts/sv_corpus_grad/outcome_identity/probe.sh
#
# SV-CORPUS-GRAD.13e.6(a) — the RED/GREEN bank for the OUTCOME-IDENTITY refusal.
#
# ⛔ WHAT IT GUARDS. `results.tsv` / `positions.tsv` are `.gitignore`d, so they carry neither an
# identity block nor a history — nothing on disk could say which run produced the copy you are
# adjudicating. `.13e.5` measured the cost: one SV-lane row sat `fail` for THREE DAYS after the
# construct was fixed, and the axis-2 bar counted a defect that no longer existed (275 -> 274).
# The runner now digests the dumps it wrote into the TRACKED report's identity table, and the
# adjudicator REFUSES when the copy on disk is a different one.
#
# ⛔ NOTHING TRACKED IS MUTATED. Every arm builds its report FIXTURE and its dumps under
# `rust/target/`, and the one end-to-end arm redirects all five adjudicator outputs there too.
#
# Deterministic, ~5 s. rc 0 = every arm as declared.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2

ADJ=stimuli/sv/adjudicate_external_corpus.py
WORK=rust/target/outcome_identity_probe
rm -rf "$WORK"; mkdir -p "$WORK"
trap 'rm -rf "$ROOT/$WORK"' EXIT INT TERM HUP

pass=0; fail=0
say() { # say <ok|no> <label> [detail]
  if [ "$1" = ok ]; then pass=$((pass+1)); printf 'PASS  %-56s\n' "$2"
  else fail=$((fail+1)); printf 'FAIL  %-56s\n' "$2"; [ -n "${3-}" ] && printf '%s\n' "$3" | sed 's/^/        /'
  fi
}

# ---- the five function-level arms, driven through the REAL function ------------------------------
# ⛔ Imported, never re-implemented: a bank that restates the predicate tests its own copy.
python3 - "$WORK" <<'PY'
import importlib.util, io, os, sys, contextlib
from pathlib import Path

work = Path(sys.argv[1]); work.mkdir(parents=True, exist_ok=True)
spec = importlib.util.spec_from_file_location(
    "adj", "stimuli/sv/adjudicate_external_corpus.py")
adj = importlib.util.module_from_spec(spec); spec.loader.exec_module(adj)

dump = work / "results.tsv"
dump.write_text("suiteX\tpass\tstimuli/sv/subs/suiteX/a.sv\n", encoding="utf-8")
good = adj.sha256_of(dump)
bad = "0" * 64
rel = adj._rel(dump)

def report(rows):
    p = work / "report.md"
    body = ["# r", "", "| input | repo-root-relative path | sha256 |", "|---|---|---|"]
    body += [f"| {label} | `{path}` | `{d}` |" for label, path, d in rows]
    p.write_text("\n".join(body) + "\n", encoding="utf-8")
    return p

def drive(rep, dumps):
    err = io.StringIO()
    try:
        with contextlib.redirect_stderr(err):
            adj.verify_outcome_identity(rep, dumps, rerun="RERUN")
        return "OK", err.getvalue()
    except SystemExit as e:
        return "REFUSED", str(e)

misses = []
def want(label, cond, got):
    if not cond:
        misses.append(f"{label}: got {got!r}")
    print(("PASS  " if cond else "FAIL  ") + f"{label:<56s}")

v, out = drive(report([("results (output)", rel, good)]), [dump])
want("A  matching digest -> OK, and it SAYS so",
     v == "OK" and "outcome-identity: OK" in out and rel in out, (v, out))

v, out = drive(report([("results (output)", rel, bad)]), [dump])
want("B  RED: the dump on disk is a different one",
     v == "REFUSED" and "is NOT the dump" in out and "RERUN" in out, (v, out))

v, out = drive(report([("grammar", "grammars/systemverilog.ebnf", good)]), [dump])
want("C  no output row -> NOT EVALUATED, loudly, never a pass",
     v == "OK" and "NOT EVALUATED" in out and "13e.6" in out, (v, out))

v, out = drive(report([("results (output)", rel, bad)]), [work / "absent.tsv"])
want("D  an ABSENT dump cannot be the WRONG dump",
     v == "OK" and "is NOT the dump" not in out, (v, out))

v, out = drive(work / "no-such-report.md", [dump])
want("E  an absent REPORT is NOT EVALUATED, never a pass",
     v == "OK" and "NOT EVALUATED" in out, (v, out))

# ⭐ The arm that proves the reader is not fooled by the row it is NOT looking at: a report that
# names a DIFFERENT dump must still report this one unevaluated rather than silently matching.
other = work / "positions.tsv"; other.write_text("x\n", encoding="utf-8")
v, out = drive(report([("positions (output)", adj._rel(other), adj.sha256_of(other))]), [dump])
want("F  a digest for another dump does not vouch for this one",
     v == "OK" and "NOT EVALUATED" in out, (v, out))

sys.exit(1 if misses else 0)
PY
if [ $? -eq 0 ]; then pass=$((pass+6)); else fail=$((fail+1)); fi

# ---- the END-TO-END arm — the WIRING, not just the function --------------------------------------
# A correct function that nothing calls is `GATE-REACHABILITY`'s subject. This drives the real
# adjudicator with a REPORT FIXTURE carrying a wrong digest for the real `results.tsv`, and every
# output redirected under rust/target/.
if [ -f stimuli/sv/characterization/results.tsv ]; then
  sed 's/^| generated parser |.*/&\n| results (output) | `stimuli\/sv\/characterization\/results.tsv` | `0000000000000000000000000000000000000000000000000000000000000000` |/' \
      stimuli/sv/characterization/characterization.md > "$WORK/bad_report.md"
  out="$(python3 "$ADJ" --report "$WORK/bad_report.md" \
          --out-manifest "$WORK/m.tsv" --out-summary "$WORK/s.md" \
          --out-lane-list "$WORK/lane.tsv" \
          --out-manifest-v2005 "$WORK/mv.tsv" --out-summary-v2005 "$WORK/sv.md" 2>&1)"; rc=$?
  if [ "$rc" -ne 0 ] && printf '%s' "$out" | grep -q 'is NOT the dump'; then
    say ok "G  END-TO-END: the adjudicator REFUSES a stale dump"
  else
    say no "G  END-TO-END: the adjudicator REFUSES a stale dump" "rc=$rc
$out"
  fi
  # ...and the GREEN control on the same wiring: the tracked report, whatever it records today.
  out="$(python3 "$ADJ" --out-manifest "$WORK/m2.tsv" --out-summary "$WORK/s2.md" \
          --out-lane-list "$WORK/lane2.tsv" \
          --out-manifest-v2005 "$WORK/mv2.tsv" --out-summary-v2005 "$WORK/sv2.md" 2>&1)"; rc=$?
  if [ "$rc" -eq 0 ]; then
    say ok "H  END-TO-END GREEN control: the real tree adjudicates"
  else
    say no "H  END-TO-END GREEN control: the real tree adjudicates" "rc=$rc
$out"
  fi
else
  printf 'SKIP  %-56s (results.tsv absent — it is .gitignore'"'"'d)\n' "G/H  END-TO-END arms"
fi

printf 'OUTCOME-IDENTITY: %d/%d as declared\n' "$pass" "$((pass + fail))"
[ "$fail" -eq 0 ]
