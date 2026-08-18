#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2h — the adjudication ratchet now binds on all THREE dialect profiles.
#
# ⛔ WHY. The SV parser ships `sv_2017`, `sv_2023` and `verilog_2005`, so the compliance goal is
# IEEE 1800-2017 + IEEE 1800-2023 + **IEEE 1364-2005** (director, 2026-08-18). The repro runner
# hard-coded `--profile sv_2017`, and `pulse_control_specparam` and its terminals carry **no
# `@profiles` gate** — so `.13c.2f` slice 4 changed what `verilog_2005` accepts while every guard
# in this suite watched one profile. An over-acceptance that only shows under `verilog_2005` was
# invisible to a green ratchet.
#
#   bash docs/tasks/artifacts/sv_corpus_grad/es13c2h_three_profiles/probe.sh
#   # THREE-PROFILES: N/N as declared
#
# The totals are DERIVED from the arms that ran — never stored.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../../.." && pwd)"
cd "$ROOT"

RUNNER="stimuli/sv/run_adjudication_repros.py"
MANIFEST="stimuli/sv/adjudication_repros/MANIFEST.tsv"
W="rust/target/es13c2h_probe"; rm -rf "$W"; mkdir -p "$W"
pass=0; total=0
cp "$MANIFEST" "$W/manifest.orig"
restore() { cp "$W/manifest.orig" "$MANIFEST"; }
trap restore EXIT

ok() { total=$((total+1)); if [ "$2" = "$3" ]; then pass=$((pass+1)); printf '  ✓ %-52s %s\n' "$1" "$2";
       else printf '  ✗ %-52s got %s, wanted %s\n' "$1" "$2" "$3"; fi }

edit_profiles() {  # edit_profiles <row-id> <value>
  python3 - "$1" "$2" <<'PY'
import csv, sys
P = "stimuli/sv/adjudication_repros/MANIFEST.tsv"
rows = list(csv.DictReader(open(P, encoding="utf-8", newline=""), delimiter="\t"))
fields = list(rows[0].keys())
for r in rows:
    if r["id"] == sys.argv[1]:
        r["profiles"] = sys.argv[2]
w = csv.DictWriter(open(P, "w", encoding="utf-8", newline=""), fieldnames=fields,
                   delimiter="\t", lineterminator="\n")
w.writeheader(); w.writerows(rows)
PY
}

echo "THREE-PROFILES — SV-CORPUS-GRAD.13c.2h"
echo

# ── A1 GREEN: the tracked manifest passes, and the PATHPULSE rows really do run three times ──────
out="$(python3 "$RUNNER" 2>&1)"
ok "A1 tracked manifest is green"            "$(printf '%s' "$out" | sed -n 's/.*failures=\([0-9]*\).*/\1/p' | tail -1)" "0"
ok "A2 twelve rows declare all three profiles" "$(printf '%s' "$out" | sed -n 's/.*multi_profile_rows=\([0-9]*\).*/\1/p' | tail -1)" "12"
# checked = 41 rows + 12 rows x 2 extra profiles = 65; derived, not stored
rows="$(python3 -c "import csv;print(sum(1 for _ in csv.DictReader(open('$MANIFEST',encoding='utf-8',newline=''),delimiter='\t')))")"
ok "A3 checked = rows + 2 per multi-profile row" \
   "$(printf '%s' "$out" | sed -n 's/.*checked=\([0-9]*\).*/\1/p' | tail -1)" "$((rows + 24))"

# ── A4 RED: an unknown profile must REFUSE, never be silently ignored ────────────────────────────
edit_profiles "invalid_pathpulse_three_limits.sv" "sv_2017,sv_1066"
red="$(python3 "$RUNNER" 2>&1)"
total=$((total+1))
case "$red" in *"declares unknown profile 'sv_1066'"*)
  pass=$((pass+1)); printf '  ✓ %-52s\n' "A4 an unknown profile REFUSES" ;;
  *) printf '  ✗ %-52s — it was IGNORED\n' "A4 an unknown profile REFUSES" ;;
esac
restore

# ── A5 RED: a false per-profile claim must be caught, with the profile NAMED ─────────────────────
# A covergroup `select_expression … with ( … )` is SystemVerilog-only, so claiming it ACCEPTs under
# `verilog_2005` is false — if the runner still reported green, the loop would be decorative.
edit_profiles "fixed_select_expression_with.sv" "sv_2017,verilog_2005"
red2="$(python3 "$RUNNER" 2>&1)"
restore
ok "A5 a false per-profile claim FAILS" \
   "$(printf '%s' "$red2" | sed -n 's/.*failures=\([0-9]*\).*/\1/p' | tail -1)" "1"
total=$((total+1))
case "$red2" in *"[verilog_2005]: expected ACCEPT, got REJECT"*)
  pass=$((pass+1)); printf '  ✓ %-52s\n' "A6 the failure NAMES the offending profile" ;;
  *) printf '  ✗ %-52s\n' "A6 the failure NAMES the offending profile" ;;
esac

# ── A7 the restore is real — a probe that leaves the tracked manifest edited is a defect ─────────
ok "A7 the tracked manifest is byte-restored" \
   "$(shasum -a 256 "$MANIFEST" | cut -d' ' -f1)" "$(shasum -a 256 "$W/manifest.orig" | cut -d' ' -f1)"

echo
echo "THREE-PROFILES: $pass/$total as declared"
[ "$pass" = "$total" ]
