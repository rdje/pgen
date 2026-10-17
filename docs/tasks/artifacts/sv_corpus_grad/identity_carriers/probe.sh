#!/usr/bin/env bash
# docs/tasks/artifacts/sv_corpus_grad/identity_carriers/probe.sh — SV-CORPUS-GRAD.13i's RED/GREEN bank for
# IDENTITY-CARRIER-CURRENCY, driven through the `PGEN_IDENTITY_CARRIERS` seam on FIXTURES built under
# rust/target (copies of a real un-owned carrier with rows rewritten). Nothing tracked is mutated.
# ⭐ Arm 5 is `.13i`(c)'s named control: the SV report as it stood at 0994c3c0, when four of six carriers were stale.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
G=scripts/check_identity_carriers.sh; SRC=docs/tasks/artifacts/sv_corpus_grad/dark_chained_only/summary.md
W=rust/target/identity_carriers_probe; rm -rf "$W"; mkdir -p "$W/before" "$W/live"; trap 'rm -rf "$ROOT/$W"' EXIT INT TERM HUP
Z=$(printf '0%.0s' $(seq 64)); pass=0; fail=0
arm(){ local label="$1" files="$2" want_rc="$3" token="$4"; local out rc; out="$(PGEN_IDENTITY_CARRIERS="$files" bash "$G" 2>&1)"; rc=$?
  if [ "$rc" -eq "$want_rc" ] && printf '%s' "$out" | grep -qF -- "$token"; then pass=$((pass+1)); printf 'PASS  %-70s rc=%d\n' "$label" "$rc"; else fail=$((fail+1)); printf 'FAIL  %-70s rc=%d (want %d, token %q)\n%s\n' "$label" "$rc" "$want_rc" "$token" "$(printf '%s' "$out" | grep -E 'NOTE|✗|OK|REFUSE' | head -4)"; fi; }
# 1 RED: the grammar row zeroed on an un-owned carrier
sed "s/^| grammar | \`grammars\/systemverilog.ebnf\` | \`[0-9a-f]*\` |/| grammar | \`grammars\/systemverilog.ebnf\` | \`$Z\` |/" "$SRC" > "$W/live/bad_grammar.md"
arm "1  RED: un-owned carrier, grammar row zeroed -> FAIL" "$W/live/bad_grammar.md" 1 "MOVED"
# 2 NOTE: a zeroed parse-binary row is provenance, not staleness
sed "s/^| grammar |/| parse binary | \`rust\/target\/release\/parseability_probe\` | \`$Z\` |\n| grammar |/" "$SRC" > "$W/live/probe_moved.md"
arm "2  NOTE: parse binary row zeroed -> rc 0 with a NOTE" "$W/live/probe_moved.md" 0 "provenance, not staleness"
# 3 frozen: the same bad carrier under before/ is reported frozen and never judged
cp "$W/live/bad_grammar.md" "$W/before/snapshot.md"
arm "3  FROZEN: a bad carrier under before/ is excluded (frozen=1)" "$W/before/snapshot.md"$'\n'"$SRC" 0 "frozen=1"
# 4 semantic: an annotated 2-col (ebnf_raw_ast) row with a wrong digest fails SEMANTICALLY
printf '# f\n\n## Instrument identity\n\n| input | sha256 |\n| --- | --- |\n| `grammars/systemverilog.ebnf` (ebnf_raw_ast) | `%s` |\n\n## next\n' "$Z" > "$W/live/sem_bad.md"
arm "4  RED: annotated (ebnf_raw_ast) row wrong -> FAIL semantically" "$W/live/sem_bad.md" 1 "MOVED semantically"
LIVE_S=$(bash scripts/check_baseline_identity.sh --digest ebnf_raw_ast grammars/systemverilog.ebnf | cut -d' ' -f1)
printf '# f\n\n## Instrument identity\n\n| input | sha256 |\n| --- | --- |\n| `grammars/systemverilog.ebnf` (ebnf_raw_ast) | `%s` |\n\n## next\n' "$LIVE_S" > "$W/live/sem_ok.md"
arm "4b GREEN: annotated (ebnf_raw_ast) row equal -> pass" "$W/live/sem_ok.md" 0 "rows_equal=1"
# 5 .13i(c)'s named control: the SV report at 0994c3c0 — grammar and parser both moved since
git show 0994c3c0:stimuli/sv/characterization/characterization.md > "$W/live/hist_0994c3c0.md" 2>/dev/null
arm "5  RED (history): the SV report as of 0994c3c0 -> FAIL on its grammar row" "$W/live/hist_0994c3c0.md" 1 "grammar\` MOVED"
# 6 GREEN: the derived population on the real tree
out="$(bash "$G" 2>&1)"; rc=$?; if [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q '^IDENTITY-CARRIERS: OK'; then pass=$((pass+1)); printf 'PASS  %-70s rc=0\n' "6  GREEN: the derived population on the real tree"; printf '      %s\n' "$(printf '%s' "$out" | grep '^IDENTITY-CARRIERS')"; else fail=$((fail+1)); printf 'FAIL  6 GREEN rc=%d\n%s\n' "$rc" "$out"; fi
# 7 REFUSE: an empty population is never a pass
out="$(PGEN_IDENTITY_CARRIERS="" bash "$G" 2>&1)"; rc=$?; if [ "$rc" -eq 2 ] && printf '%s' "$out" | grep -q 'EMPTY'; then pass=$((pass+1)); printf 'PASS  %-70s rc=2\n' "7  REFUSE: an empty population"; else fail=$((fail+1)); printf 'FAIL  7 rc=%d\n%s\n' "$rc" "$out"; fi
# 8 .58c: the two-edition pipeline census as -0446 carried it (its answer-key row three re-derivations stale), copied under
#   a basename two oracles name (`census.md`) in a directory none names — un-owned, so the stale row FAILS instead of
#   being "deferred to check_corpus_key_integrity.sh, which judges this carrier" (the note the basename rule issued)
git show 9e1ce4890:docs/tasks/artifacts/sv_corpus_grad/pipeline_sv2023_lane/census.md > "$W/live/census.md" 2>/dev/null
arm "8  RED (.58c): a stale census.md in an un-owned directory -> FAIL, not a deferral" "$W/live/census.md" 1 "sv_2023 answer key\` MOVED"
printf 'IDENTITY-CARRIERS-BANK: %d/%d as declared\n' "$pass" "$((pass+fail))"; [ "$fail" -eq 0 ]
