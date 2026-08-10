#!/usr/bin/env bash
# docs/tasks/artifacts/sv_corpus_grad/timeout_reconfirm_control/run_control.sh
#
# SV-CORPUS-GRAD.3.27 — GROUND-TRUTH CONTROLS for the serial timeout re-confirmation added to
# `stimuli/run_external_corpus.sh`.
#
# ⛔ WHY THIS EXISTS. The re-confirmation pass only fires when a file times out, and the tracked
# corpora time out 4 / 0 / 0 times at the recorded parameters — so an ordinary run exercises the
# reclassification counter ZERO times and a broken counter would read `0` exactly like a healthy
# one. A number produced by an instrument that has never been seen to fire is not evidence. Both
# directions are therefore forced here with a stub parse binary:
#
#   POSITIVE — the stub is slow on its FIRST call for a file and fast on every later call, which
#              is the contention shape the pass exists to catch: the parallel pass records
#              `timeout`, the serial re-run records `pass`, and the reclassification counter must
#              move by exactly the population size.
#   NEGATIVE — the stub is slow on EVERY call, i.e. a genuinely slow parse: the serial re-run
#              must confirm the timeout and the reclassification counter must stay at 0. This is
#              the arm that proves the counter is not simply always-on.
#
# Deterministic (fixed file list, fixed sleeps, single-job serial arm), repo-volume-local, and it
# touches nothing tracked — the corpus runner writes under `rust/target/` via PGEN_CORPUS_OUT_DIR.
#
# Usage:  bash docs/tasks/artifacts/sv_corpus_grad/timeout_reconfirm_control/run_control.sh
# Exit:   0 = both controls behaved as specified; nonzero = the instrument is not trustworthy.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
[ -f "$ROOT/stimuli/run_external_corpus.sh" ] || {
  echo "reconfirm-control: repo root resolved to '$ROOT', which holds no stimuli/run_external_corpus.sh" >&2
  exit 2
}
WORK="$ROOT/rust/target/corpus_3_27_reconfirm_control"
STUB="$WORK/stub_probe.sh"
FILES=3          # corpus files to feed the runner
DEADLINE=2       # per-file timeout handed to the runner, seconds
SLOW=5           # stub sleep, comfortably past the deadline

rm -rf "$WORK"; mkdir -p "$WORK/markers"

# The stub ignores every probe flag and keys only on the LAST argument that names an existing
# file, which is how the runner invokes it (`--parse <grammar> <file> [--profile X]`).
cat > "$STUB" <<STUBEOF
#!/usr/bin/env bash
set -uo pipefail
target=""
for a in "\$@"; do [ -f "\$a" ] && target="\$a"; done
key="\$(printf '%s' "\$target" | tr -c 'A-Za-z0-9' '_')"
marker="$WORK/markers/\$key"
if [ "\${STUB_ALWAYS_SLOW:-0}" = 1 ] || [ ! -e "\$marker" ]; then
  : > "\$marker"
  sleep $SLOW
fi
exit 0
STUBEOF
chmod +x "$STUB"

run_arm() {  # $1 = arm label, $2 = STUB_ALWAYS_SLOW value, $3 = out dir
  rm -rf "$3"; rm -f "$WORK"/markers/*
  STUB_ALWAYS_SLOW="$2" \
  PGEN_PARSE_PROBE_BIN="$STUB" \
  PGEN_CORPUS_OUT_DIR="${3#"$ROOT"/}" \
  PGEN_CORPUS_REBASELINE=1 \
    "$ROOT/stimuli/run_external_corpus.sh" sv "$DEADLINE" "$FILES" "$FILES" 2>&1 \
    | grep -E 're-confirm|parsed —' || true
}

report_field() {  # $1 = out dir, $2 = row label -> the count cell
  awk -F'|' -v k="$2" '$0 ~ k { gsub(/ /, "", $3); print $3; exit }' "$1/characterization.md"
}

fail=0
note() { printf 'reconfirm-control: %s\n' "$1" >&2; fail=1; }

echo "=== POSITIVE arm — slow once, then fast (the contention shape) ==="
run_arm positive 0 "$WORK/positive"
P_SEEN=$(report_field "$WORK/positive" 'timeouts seen in the parallel pass')
P_DONE=$(report_field "$WORK/positive" 're-confirmed serially')
P_CHANGED=$(report_field "$WORK/positive" 'reclassified by re-confirmation')
P_FINAL=$(report_field "$WORK/positive" 'after serial re-confirmation')
echo "    parallel timeouts=$P_SEEN  re-confirmed=$P_DONE  reclassified=$P_CHANGED  final timeouts=$P_FINAL"
[ "$P_SEEN"    = "$FILES" ] || note "POSITIVE: expected $FILES parallel timeouts, got '$P_SEEN' — the stub did not force the shape"
[ "$P_DONE"    = "$FILES" ] || note "POSITIVE: expected $FILES serial re-confirmations, got '$P_DONE' — the pass did not run"
[ "$P_CHANGED" = "$FILES" ] || note "POSITIVE: expected $FILES reclassifications, got '$P_CHANGED' — the counter is BLIND"
[ "$P_FINAL"   = "0" ]      || note "POSITIVE: expected 0 surviving timeouts, got '$P_FINAL' — the verdict was not substituted"

echo "=== NEGATIVE arm — slow every time (a genuinely slow parse) ==="
run_arm negative 1 "$WORK/negative"
N_SEEN=$(report_field "$WORK/negative" 'timeouts seen in the parallel pass')
N_DONE=$(report_field "$WORK/negative" 're-confirmed serially')
N_CHANGED=$(report_field "$WORK/negative" 'reclassified by re-confirmation')
N_FINAL=$(report_field "$WORK/negative" 'after serial re-confirmation')
echo "    parallel timeouts=$N_SEEN  re-confirmed=$N_DONE  reclassified=$N_CHANGED  final timeouts=$N_FINAL"
[ "$N_SEEN"    = "$FILES" ] || note "NEGATIVE: expected $FILES parallel timeouts, got '$N_SEEN'"
[ "$N_DONE"    = "$FILES" ] || note "NEGATIVE: expected $FILES serial re-confirmations, got '$N_DONE'"
[ "$N_CHANGED" = "0" ]      || note "NEGATIVE: expected 0 reclassifications, got '$N_CHANGED' — the counter fires SPURIOUSLY"
[ "$N_FINAL"   = "$FILES" ] || note "NEGATIVE: expected $FILES surviving timeouts, got '$N_FINAL' — a real timeout was erased"

if [ "$fail" -eq 0 ]; then
  echo "=== reconfirm-control: BOTH ARMS OK — the re-confirmation pass fires, reclassifies only"
  echo "    when the serial re-run disagrees, and never erases a reproducible timeout. ==="
else
  echo "=== reconfirm-control: FAILED — do not trust the re-confirmation counts ===" >&2
fi
exit "$fail"
