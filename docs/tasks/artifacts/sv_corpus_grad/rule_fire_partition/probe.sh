#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2u — every refusal arm of `rule_fire_partition.py`, OBSERVED firing.
#
# ⛔ A control never seen RED is not known to work (`docs/CLAIM_VERIFICATION.md` §3 leg 2), and this
# instrument's FIRST control was wrong in the refusing direction — it called a corpus-covered,
# generator-hard rule a "contradiction" and went red on a correct tree. A guard that refuses correct
# trees teaches people to bypass it, so every arm here perturbs a real input in a SCRATCH copy and
# asserts the exact exit code, and the last arm is the CONTROL that must stay GREEN.
set -uo pipefail
# ⚠️ FIVE levels: docs/tasks/artifacts/sv_corpus_grad/rule_fire_partition. The root
# guard below caught this exact off-by-one for the FOURTH time in this directory tree
# — which is the argument for keeping the guard, not for trusting the arithmetic.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
[ -d "$ROOT/grammars" ] || { echo "probe: not at the repo root (derived $ROOT)" >&2; exit 2; }
cd "$ROOT"

T=rust/target/rule_fire_partition_probe; rm -rf "$T"; mkdir -p "$T"
COV=stimuli/sv/characterization/rule_coverage_sv_2017.tsv
CERT=rust/target/cert_sv2017.out
[ -f "$CERT" ] || { echo "probe: NOT EVALUATED — no certificate report at $CERT. Produce one with" \
  "PGEN_CERT_COVERAGE_DUMP_ALL=1 ast_pipeline … --report-certificate-coverage" >&2; exit 3; }
cp "$COV" "$T/cov.orig"; cp "$CERT" "$T/cert.orig"

arms=0; bad=0
arm() { # arm <name> <expected-rc>
  local name="$1" want="$2"; shift 2
  "$@" >"$T/out.txt" 2>&1; local rc=$?
  arms=$((arms+1))
  if [ "$rc" = "$want" ]; then printf '  ✓ %-46s rc=%s\n' "$name" "$rc"
  else printf '  ✗ %-46s rc=%s (wanted %s)\n' "$name" "$rc" "$want"; bad=$((bad+1)); sed -n '1,4p' "$T/out.txt"; fi
}
run() { python3 stimuli/sv/rule_fire_partition.py --cert-report "$1" --out "$T/part"; }

echo "RULE-FIRE-PARTITION PROBE"

# ARM 1 — DENOMINATOR MISMATCH: add a satisfiable rule the cert never counted.
printf 'zz_probe_injected_rule\tGAP\t0\tsv_2017\n' >> "$COV"
arm "denominator mismatch refuses" 1 run "$CERT"
cp "$T/cov.orig" "$COV"

# ARM 2 — STALE CERT: make the report name a rule the artifact does not have.
sed 's/"kw_eventually_5865020f"/"zz_rule_that_does_not_exist"/' "$T/cert.orig" > "$T/cert.stale"
arm "cert naming an unknown rule refuses" 1 run "$T/cert.stale"

# ARM 3 — UNADJUDICATED: drop one hand-adjudication and the rule must resurface as a candidate.
python3 - "$T/part_noadj.py" <<'PY'
import pathlib, sys
src = pathlib.Path("stimuli/sv/rule_fire_partition.py").read_text()
src = src.replace('    "kw_s_always_0f9aa900": (', '    "zz_disabled_kw_s_always": (', 1)
pathlib.Path(sys.argv[1]).write_text(src)
PY
arm "an unadjudicated unwitnessed rule refuses" 1 python3 "$T/part_noadj.py" --cert-report "$CERT" --out "$T/part"

# ARM 4 — NO PROOF LIST: a report from an ast_pipeline older than `-0240` withholds the proof
# names, and inferring `witnessed` from `not UNKNOWN` is what misclassified 18 proven-unreachable
# rules as "it can fire". The instrument must REFUSE such a report, never silently degrade.
grep -v "PROOF-COVERED rules" "$T/cert.orig" > "$T/cert.noproof"
arm "a report with no PROOF list refuses" 2 run "$T/cert.noproof"

# ARM 5 — THE REAL CONTRADICTION: a rule with a certificate PROOF of unreachability that also
# COMMITS in an accepted corpus file. Injected by flipping a proof-covered rule to `covered`.
python3 - "$COV" "$T/cov.contra" <<'PY'
import sys, pathlib
src, dst = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
out = []
for line in src.read_text(encoding='utf-8').splitlines():
    f = line.split(chr(9))
    if f and f[0] == 'union_modifier' and len(f) >= 2:
        f[1] = 'covered'; line = chr(9).join(f)
    out.append(line)
dst.write_text(chr(10).join(out) + chr(10), encoding='utf-8')
PY
cp "$T/cov.contra" "$COV"
arm "a PROOF refuted by the corpus refuses" 1 run "$CERT"
cp "$T/cov.orig" "$COV"

# ARM 6 — MISSING INPUT: the instrument must refuse (2), never report a partial partition.
arm "a missing certificate report refuses" 2 python3 stimuli/sv/rule_fire_partition.py \
    --cert-report "$T/nope.txt" --out "$T/part"

# ARM 7 — THE CONTROL: unperturbed, it must PASS. Without this, an instrument broken into
# always-refusing would score 4/4 above.
arm "CONTROL — the unperturbed tree passes" 0 run "$CERT"

cp "$T/cov.orig" "$COV"
echo "RULE-FIRE-PARTITION PROBE: $((arms-bad))/$arms arms behaved as specified"
[ "$bad" = 0 ] || exit 1
