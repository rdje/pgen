#!/usr/bin/env bash
# scripts/check_sv_store_gate_bank.sh
#
# SV-STORE-GATE-BANK — the `.13w` store-gate explanation bank is RUN, by a gate, every arm it declares.
#
# THE FINDING (SV-CORPUS-GRAD.15, 2026-09-03): the bank docs/tasks/artifacts/sv_corpus_grad/store_gate/probe.sh
# (9 arms then, ~4 s) was RED from PGEN-SV-CORPUS-GRAD-0347 to -0354 — six commits — and only an unrelated
# edit found it, because NOTHING invoked it: git ls-files 'scripts/*.sh' 'rust/scripts/*.sh' 'rust/Makefile'
# '.github/workflows/*.yml' '.githooks/*' | xargs grep -ln 'store_gate/probe.sh' | wc -l → 0. A check nothing
# invokes is indistinguishable from no check (the GATE-REACHABILITY argument). This gate is the invoker.
#
# THE RULE: the bank exits 0 AND its summary line reads `store_gate bank: N/N as declared (fail=0)` with N
# equal to the bank's own DECLARED count, AND no arm line reads FAIL. Three conditions because each can fail
# alone: a bank that exits 0 with a FAIL line (a script whose LAST command succeeded), a bank whose declared
# count drifted from its arm count, a bank that crashed before its summary.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit 0 = the doctrine holds; 1 = a breach (the bank's own output on
# stderr); 2 = REFUSED (the bank is missing). Deterministic: the bank reads tracked artifacts and the release
# probe; it mutates nothing outside rust/target. SCOPE: the bank's LIVE arms parse with
# rust/target/release/parseability_probe; where it is absent (a hosted CI checkout builds nothing) this gate
# evaluates NOTHING, exits 0 SAYING SO on stdout, and the driver's scope note names it — the doctrine binds on
# every clone that has built the probe (E3, the pre-commit hook), which is where the bank went red unseen.
#
# USAGE:
#   bash scripts/check_sv_store_gate_bank.sh              # the doctrine (~10 s with the probe)
#   bash scripts/check_sv_store_gate_bank.sh --self-test  # prove the judge refuses each failure shape
set -u
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 2
BANK="docs/tasks/artifacts/sv_corpus_grad/store_gate/probe.sh"
# PGEN_RELEASE_PROBE overrides the probe path — a testability seam so the "not built" path is PROVEN, not asserted.
PROBE="${PGEN_RELEASE_PROBE:-rust/target/release/parseability_probe}"
SUMMARY_RE='^store_gate bank: ([0-9]+)/([0-9]+) as declared \(fail=([0-9]+)\)$'

# judge <bank-path> : 0 = the bank passes every arm it declares; 1 = breach; 2 = cannot inspect
judge() {
  local bank="$1" out rc summary declared n m f
  [ -f "$bank" ] || { echo "sv-store-gate-bank: REFUSED (2) — no bank at $bank" >&2; return 2; }
  out="$(bash "$bank" 2>&1)"; rc=$?
  declared="$(grep -oE '^DECLARED=[0-9]+' "$bank" | head -1 | cut -d= -f2)"
  summary="$(printf '%s\n' "$out" | grep -E '^store_gate bank: ' | tail -1)"
  if [ -z "$summary" ]; then
    printf 'sv-store-gate-bank: FAIL — the bank printed NO summary line (crashed before its last arm?). Output:\n%s\n' "$out" >&2
    return 1
  fi
  n="$(printf '%s' "$summary" | sed -E "s|${SUMMARY_RE}|\\1|")"
  m="$(printf '%s' "$summary" | sed -E "s|${SUMMARY_RE}|\\2|")"
  f="$(printf '%s' "$summary" | sed -E "s|${SUMMARY_RE}|\\3|")"
  if [ "$rc" -ne 0 ] || [ "$f" != "0" ] || [ "$n" != "$m" ] || [ -z "$declared" ] || [ "$m" != "$declared" ] \
     || printf '%s\n' "$out" | grep -qE '^  FAIL  '; then
    printf 'sv-store-gate-bank: FAIL — the bank does not pass every arm it declares (rc=%s, DECLARED=%s, summary: %s). Output:\n%s\n' \
      "$rc" "${declared:-?}" "$summary" "$out" >&2
    printf '  Fix the arm, or the instrument it judges — never the count. See docs/tasks/SV-CORPUS-GRAD.md .13w / .15.\n' >&2
    return 1
  fi
  echo "sv-store-gate-bank: OK ($summary)"
  return 0
}

if [ "${1:-}" = "--self-test" ]; then
  td="$(mktemp -d "$ROOT/rust/target/sv_store_gate_bank_selftest.XXXXXX")"; trap 'rm -rf "$td"' EXIT
  fails=0
  arm() { if [ "$2" -eq 0 ]; then echo "  PASS  $1"; else echo "  FAIL  $1"; fails=$((fails+1)); fi; }
  printf '#!/usr/bin/env bash\nDECLARED=2\necho "  PASS  a"\necho "  PASS  b"\necho "store_gate bank: 2/2 as declared (fail=0)"\nexit 0\n' > "$td/green.sh"
  judge "$td/green.sh" >/dev/null 2>&1; arm "a green bank (2/2, exit 0, no FAIL line) is accepted" $?
  printf '#!/usr/bin/env bash\nDECLARED=2\necho "  PASS  a"\necho "  FAIL  b"\necho "store_gate bank: 1/2 as declared (fail=1)"\nexit 1\n' > "$td/red.sh"
  judge "$td/red.sh" >/dev/null 2>&1; [ $? -eq 1 ]; arm "a bank with a FAIL arm is refused (1)" $?
  printf '#!/usr/bin/env bash\nDECLARED=2\necho "  FAIL  a"\necho "  PASS  b"\necho "store_gate bank: 2/2 as declared (fail=0)"\nexit 0\n' > "$td/lying.sh"
  judge "$td/lying.sh" >/dev/null 2>&1; [ $? -eq 1 ]; arm "a bank that exits 0 while an arm line reads FAIL is refused (1)" $?
  printf '#!/usr/bin/env bash\nDECLARED=3\necho "  PASS  a"\necho "  PASS  b"\necho "store_gate bank: 2/2 as declared (fail=0)"\nexit 0\n' > "$td/drift.sh"
  judge "$td/drift.sh" >/dev/null 2>&1; [ $? -eq 1 ]; arm "a summary whose denominator is not the bank's own DECLARED is refused (1)" $?
  printf '#!/usr/bin/env bash\nDECLARED=2\necho "  PASS  a"\nexit 0\n' > "$td/crash.sh"
  judge "$td/crash.sh" >/dev/null 2>&1; [ $? -eq 1 ]; arm "a bank with no summary line is refused (1)" $?
  judge "$td/none.sh" >/dev/null 2>&1; [ $? -eq 2 ]; arm "a missing bank is REFUSED (2), never passed" $?
  [ -f "$BANK" ] && grep -qE '^DECLARED=[0-9]+' "$BANK"; arm "the real bank exists and declares its arm count" $?
  ne="$(PGEN_RELEASE_PROBE="$td/no_such_probe" bash "$ROOT/scripts/check_sv_store_gate_bank.sh" 2>&1)"; rc=$?
  [ "$rc" -eq 0 ] && [[ "$ne" == *"NOT EVALUATED"* ]]; arm "without the release probe the gate exits 0 SAYING SO (NOT EVALUATED), never judging" $?
  echo "sv-store-gate-bank self-test: $((8-fails))/8 (fail=$fails)"
  [ "$fails" -eq 0 ] || exit 1
  exit 0
fi

if [ ! -x "$PROBE" ]; then
  echo "sv-store-gate-bank: NOT EVALUATED — $PROBE is absent and the bank's LIVE arms parse with it; nothing was judged here (the driver's scope note names this). The doctrine binds on every clone that has built the probe."
  exit 0
fi
judge "$BANK"
