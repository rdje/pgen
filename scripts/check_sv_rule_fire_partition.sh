#!/usr/bin/env bash
# SV-RULE-FIRE-PARTITION (SV-CORPUS-GRAD.13c.2u) — the rules that never fired on the SV corpus are
# CLASSIFIED, and the classification is CONSUMED.
#
# ⛔⛔ WHY THIS DOCTRINE EXISTS. `stimuli/sv/characterization/rule_coverage_sv_2017.tsv` has recorded,
# per grammar rule, how many accepted corpus files ever caused it to fire — and NOTHING read it.
# `.13c.2u` opened on its headline, *"104 grammar rules have NEVER fired on 16 336 real files"*, as
# the highest-yield rejects-valid hunt on the board. Measured, that headline was wrong twice over:
# the 104 was stale (137 today — the grammar gained 135 rules), and a large part of the list cannot
# fire for a reason that is not a defect at all, because PGEN's own left-recursion eliminator AUTHORS
# rules (`X_lr_*`) and REPLACES the body of the rule it eliminates. `casting_type` is listed as never
# fired, yet on a file that ACCEPTS through `8'(1)` its nineteen `casting_type_lr_*` replacements
# fire and `casting_type` itself never appears.
# ⇒ **a tracked number nobody consumes is indistinguishable from a number nobody computed**
# (`CI-PARITY-GATE-ROT.2`'s thesis) — and worse here, an unclassified one reads as a defect surface
# and sends a reader hunting phantoms. This gate makes the number classified AND read.
#
# WHAT IT BINDS (tier 1, seconds — it reads tracked artifacts, it does not re-parse the corpus):
#   1. the tracked partition artifact is CONSISTENT with the tracked coverage artifact it derives
#      from — same gap population, same profile. A stale partition is refused, not reported.
#   2. every `unwitnessed` rule carries a hand adjudication with a measured construct behind it.
#   3. the `unadjudicated` count is ZERO and the `unwitnessed`-class ratchet may only go DOWN.
# ⛔ NOT bound here: re-deriving the partition needs a ~5-minute certificate pass and a ~3-minute
# corpus census. That is tier 2 (`make -C rust sv_rule_fire_partition`), and this tier says so
# rather than pretending a cheap check proved an expensive property.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"

COV="stimuli/sv/characterization/rule_coverage_sv_2017.tsv"
PART_TSV="stimuli/sv/characterization/rule_fire_partition_sv_2017.tsv"
PART_JSON="stimuli/sv/characterization/rule_fire_partition_sv_2017.json"
INSTRUMENT="stimuli/sv/rule_fire_partition.py"

fail=0
note() { printf 'sv-rule-fire-partition: %s\n' "$1" >&2; }
breach() { note "✗ $1"; fail=1; }

for f in "$COV" "$PART_TSV" "$PART_JSON" "$INSTRUMENT"; do
  [ -f "$f" ] || { breach "$f is MISSING — the partition cannot be checked. Re-derive: make -C rust SHELL=/bin/bash sv_rule_fire_partition"; }
done
[ "$fail" = 0 ] || { note "1 breach(es)"; exit 1; }

exec python3 - "$COV" "$PART_TSV" "$PART_JSON" <<'PY'
import json, sys, pathlib
cov, part_tsv, part_json = (pathlib.Path(p) for p in sys.argv[1:4])
fail = []

status = {}
for line in cov.read_text(encoding="utf-8").splitlines()[1:]:
    f = line.split("\t")
    if len(f) >= 2:
        status[f[0]] = f[1]
gaps = {r for r, s in status.items() if s == "GAP"}

rows = {}
for line in part_tsv.read_text(encoding="utf-8").splitlines()[1:]:
    f = line.split("\t")
    if len(f) >= 2:
        rows[f[0]] = f[1]
summary = json.loads(part_json.read_text(encoding="utf-8"))

# 1 — the partition describes the coverage artifact it claims to derive from.
if set(rows) != gaps:
    only_p, only_c = sorted(set(rows) - gaps)[:6], sorted(gaps - set(rows))[:6]
    fail.append(f"the partition covers {len(rows)} rules but the coverage artifact reports "
                f"{len(gaps)} GAP rules — it is STALE relative to the artifact it derives from. "
                f"Only-in-partition {only_p}; only-in-coverage {only_c}. "
                f"Re-derive: make -C rust SHELL=/bin/bash sv_rule_fire_partition")
if summary.get("gap_rules") != len(gaps):
    fail.append(f"the partition summary records gap_rules={summary.get('gap_rules')} but the "
                f"coverage artifact holds {len(gaps)}")

# 2/3 — every unwitnessed rule adjudicated, and the class ratchet may only fall.
unadj = summary.get("unadjudicated_unwitnessed", [])
if unadj:
    fail.append(f"{len(unadj)} rule(s) the certificate pass could not witness carry NO "
                f"adjudication: {unadj[:8]}. Each needs a minimal construct from the TRACKED LRM "
                f"and a check that its own ARM fires — a verdict alone is not enough "
                f"(SV-CORPUS-GRAD.13c.2q). Record it in ADJUDICATED_UNWITNESSED.")
live = sum(1 for c in rows.values() if c == "unwitnessed")
if live:
    fail.append(f"{live} rule(s) are still classified `unwitnessed` in the tracked partition")

# The REAL contradiction (available since `.13c.2u.2` published the certificate proof NAMES): a
# verified proof of unreachability refuted by a rule that COMMITS in an accepted corpus file.
contra = summary.get("proof_refuted_by_corpus")
if contra is None:
    fail.append("the partition summary carries no `proof_refuted_by_corpus` field — it was produced "
                "before `.13c.2u.2` and cannot have checked the contradiction. Re-derive: "
                "make -C rust SHELL=/bin/bash sv_rule_fire_partition")
elif contra:
    fail.append(f"{len(contra)} rule(s) carry a certificate PROOF of unreachability AND commit in an "
                f"accepted corpus file — a proof refuted by real text: {contra[:8]}")

classes = summary.get("classes", {})
if not classes:
    fail.append("the partition summary carries no class histogram — it is malformed")

for m in fail:
    print(f"sv-rule-fire-partition: ✗ {m}", file=sys.stderr)
if fail:
    print(f"sv-rule-fire-partition: {len(fail)} breach(es)", file=sys.stderr)
    sys.exit(1)
print("sv-rule-fire-partition: OK (" + ", ".join(f"{k}={v}" for k, v in sorted(classes.items()))
      + f"; {len(gaps)} never-fired rules, 0 unadjudicated)")
PY
