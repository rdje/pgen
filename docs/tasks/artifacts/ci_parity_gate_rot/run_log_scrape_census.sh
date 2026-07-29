#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_log_scrape_census.sh
#
# CI-PARITY-GATE-ROT.11 — the log-scraping census, as a REPRODUCIBLE instrument with its own
# ground truth, because the ad-hoc version of this measurement was wrong three times.
#
#   attempt 1: required `BASH_REMATCH` too  -> 3 scripts   ("class closed, no exposure")
#   attempt 2: any grep/sed/awk over a *log path -> 17 scripts, but TWO were false positives
#              (`>"$parse_log"` is a redirect = WRITING; `systemveriLOG` contains "log") and
#              one script was MISSED
#   this file: redirect targets stripped, `systemverilog` excluded -> 53 sites / 16 scripts,
#              with the three `.9` sites asserted present and the two false positives asserted
#              absent, so a future edit that breaks the instrument SAYS SO instead of
#              reporting a smaller, comfortable number.
#
# ⛔ It reports; it does not gate. Promotion to a ratchet belongs with the fix (`.11` scope),
# once the per-site dispositions exist to ratchet against.
#
# Usage: bash docs/tasks/artifacts/ci_parity_gate_rot/run_log_scrape_census.sh
# Exit 0 iff the calibration assertions hold.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

python3 - <<'PY'
import re, glob, os, sys
from collections import Counter, defaultdict

var_re  = re.compile(r'"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?"')
read_re = re.compile(r'\b(grep|sed|awk)\b')
# a redirect target is the script WRITING a log, not reading one
redir_re = re.compile(r'[0-9]?>[>&]?\s*"\$\{?[A-Za-z_][A-Za-z0-9_]*\}?"')

def is_log_var(v):
    lv = v.lower()
    if 'systemverilog' in lv:          # `systemveriLOG` is not a log variable
        return False
    return re.search(r'(^|_)log(_|$)|log_file|_log$', lv) is not None

def classify(line):
    s = line.strip()
    if 'Witness pass: resolved' in s or 'Target-driven generation: resolved' in s: return 'FIXED'
    if re.search(r'\(\[0-9\]\+\)|\[0-9\]\+/\[0-9\]\+|grep -c ', s):                return 'METRIC'
    if re.search(r'summary_value_from_log', s):                                    return 'METRIC'
    if re.search(r'CERTIFICATE-COVERAGE|grammar lint: ', s):                       return 'METRIC'
    if re.search(r'^\s*(if|elif)\s+.*grep -q', s):                                 return 'VERDICT'
    if re.search(r'awk .*failed=stage', s):                                        return 'VERDICT'
    if re.search(r'grep -Fq|grep -q', s):                                          return 'VERDICT'
    if re.search(r"s/\^error: \(\.\*\)\$/", s):                                    return 'DIAGNOSTIC'
    if re.search(r'sed -n "1,\$\{?max_head_lines', s):                             return 'DIAGNOSTIC'
    return 'REVIEW'

rows = []
for path in sorted(glob.glob('rust/scripts/*.sh')):
    for i, line in enumerate(open(path, errors='replace'), 1):
        if not read_re.search(line):
            continue
        stripped = redir_re.sub(' ', line)
        for v in var_re.findall(stripped):
            if is_log_var(v):
                rows.append((os.path.basename(path), i, classify(line), line.strip()[:100]))
                break

by_script = Counter(p for p, _, _, _ in rows)
by_class  = Counter(c for _, _, c, _ in rows)

print("=== CI-PARITY-GATE-ROT.11 — log-scrape census ===")
print(f"sites: {len(rows)}   scripts: {len(by_script)}\n")
print("--- by script ---")
for p, n in sorted(by_script.items(), key=lambda x: (-x[1], x[0])):
    print(f"  {n:3d}  {p}")
print("\n--- by class ---")
for c in ('FIXED', 'METRIC', 'VERDICT', 'DIAGNOSTIC', 'REVIEW'):
    print(f"  {by_class.get(c,0):3d}  {c}")

print("\n--- METRIC sites (the triage targets) ---")
for p, i, c, t in rows:
    if c == 'METRIC':
        print(f"  {p}:{i}\n      {t[:94]}")

# ---- CALIBRATION. An instrument without ground truth is a confident guess.
print("\n--- calibration ---")
ok = True
for g in ('ebnf_stimuli_quality_gate.sh', 'annotation_stimuli_quality_gate.sh', 'sv_preprocessor_quality_gate.sh'):
    hit = g in by_script
    print(f"  ground truth  {g:38s} {'present ✓' if hit else 'MISSING ✗'}")
    ok &= hit
for b in ('sv_combined_telemetry_contract_gate.sh', 'sv_stimuli_quality_gate.sh'):
    hit = b in by_script
    print(f"  control       {b:38s} {'STILL PRESENT ✗' if hit else 'excluded ✓'}")
    ok &= not hit
if not ok:
    print("\nMISCALIBRATED — the instrument no longer reproduces known facts; do not trust the numbers above.")
    sys.exit(1)
print("\ncalibration OK — the numbers above reproduce every known fact.")
PY
