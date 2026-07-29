#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_doctrine_lane_census.sh
#
# CI-PARITY-GATE-ROT.15 — how many enforced doctrines have an AUTOMATIC lane?
#
# ⭐ THE BEFORE SIDE IS REPLAYED, NOT DESCRIBED. It reads the workflow as it existed at a given
# git revision (default HEAD) through the SAME derivation used for the live tree, so the two
# columns cannot be measuring different things — the failure mode `.5` and `.9` both hit was a
# comparison against a remembered number instead of a re-executed one.
#
# ⛔ DERIVED, NOT HAND-LISTED. The doctrine roster comes from the driver's own `DOCTRINES=(…)`
# array and the workflow roster from `git ls-files`, so a doctrine or workflow added later is
# counted here by construction — which is the very property this leaf exists to give CI.
#
# Usage:
#   bash docs/tasks/artifacts/ci_parity_gate_rot/run_doctrine_lane_census.sh [BEFORE_REV]
# Exit 0 always (a census, not a gate — `scripts/check_flow_integrity.sh` invariant (8) is the gate).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"
BEFORE_REV="${1:-HEAD}"

exec python3 - "$BEFORE_REV" <<'PYEOF'
import re, subprocess, sys, os

ROOT = os.getcwd()
BEFORE_REV = sys.argv[1]
DRIVER = "scripts/check_doctrines.sh"

def sh(cmd):
    return subprocess.run(["bash", "-c", cmd], cwd=ROOT, capture_output=True, text=True).stdout

def read(p):
    try:
        with open(os.path.join(ROOT, p), encoding="utf-8", errors="replace") as f:
            return f.read()
    except OSError:
        return ""

def uncommented(t):
    return "\n".join(l for l in t.splitlines() if not l.lstrip().startswith("#"))

def roster(src):
    m = re.search(r"(?ms)^DOCTRINES=\(\n(.*?)^\)", src)
    if not m:
        return []
    return [(l.strip().strip('"').split("|")[0], l.strip().strip('"').split("|")[-1])
            for l in m.group(1).splitlines() if l.strip().startswith('"')]

def auto(body):
    blk = re.search(r"(?ms)^on:\n(.*?)(?=^\S|\Z)", uncommented(body))
    return bool(blk) and bool(re.search(r"^\s{1,4}(push|pull_request|schedule)\s*:", blk.group(1), re.M))

workflows = sorted(sh("git ls-files '.github/workflows/*.yml'").split())
doctrines = roster(read(DRIVER))
if not workflows or not doctrines:
    print("census: derivation returned nothing — refusing to report a number it cannot back.")
    sys.exit(0)

def measure(label, wf_body_for):
    """Coverage of the doctrine roster by whatever auto-triggered workflows say."""
    auto_wfs = [w for w in workflows if auto(wf_body_for(w))]
    bodies = {w: uncommented(wf_body_for(w)) for w in auto_wfs}
    via_driver = [w for w, b in bodies.items() if DRIVER in b]
    by_name = {}
    for did, script in doctrines:
        hits = [w for w, b in bodies.items() if script in b]
        if hits:
            by_name[did] = hits
    covered = set(d for d, _ in doctrines) if via_driver else set(by_name)
    print(f"--- {label}")
    print(f"    tracked workflows                : {len(workflows)}")
    print(f"    auto-triggered (push/PR/schedule): {len(auto_wfs)}  {[os.path.basename(w) for w in auto_wfs]}")
    print(f"    registered doctrines             : {len(doctrines)}")
    print(f"    invoke the DRIVER                : {len(via_driver)}  -> roster inherited: {bool(via_driver)}")
    print(f"    named individually               : {len(by_name)}  {sorted(by_name)}")
    print(f"    ⇒ doctrines with an AUTOMATIC lane: {len(covered)} of {len(doctrines)}")
    missing = [d for d, _ in doctrines if d not in covered]
    print(f"    ⇒ doctrines with NO lane          : {len(missing)}  {missing}")
    return len(covered), len(doctrines)

def at_rev(rev):
    def get(p):
        r = subprocess.run(["git", "show", f"{rev}:{p}"], cwd=ROOT, capture_output=True, text=True)
        return r.stdout if r.returncode == 0 else ""
    return get

print("=" * 78)
print("CI-PARITY-GATE-ROT.15 — the doctrine AUTOMATIC lane (every figure derived this run)")
print("=" * 78)
# ⛔ Pin the RESOLVED sha, never the symbolic name: this capture is committed, and `HEAD` means
# something different the moment it lands — an artifact that silently re-points is the stale-evidence
# class `.7` fixed, in miniature.
resolved = sh(f"git rev-parse --short {BEFORE_REV}").strip() or BEFORE_REV
b_cov, b_tot = measure(f"BEFORE (workflows as of {BEFORE_REV} = {resolved})", at_rev(BEFORE_REV))
print()
a_cov, a_tot = measure("AFTER  (working tree)", read)
print("-" * 78)
print(f"before -> after : {b_cov}/{b_tot} -> {a_cov}/{a_tot} doctrines on the automatic lane")

# ⚠️ The honest half: which of them can actually EVALUATE anything on a hosted push.
staged = [d for d, s in doctrines
          if re.search(r"--cached|--staged|diff-index", read(s) or "")]
print(f"⚠️  of the {a_tot}, {len(staged)} are scoped to the STAGED diff and exit 0 vacuously on a")
print(f"    hosted push with an empty index: {staged}")
print(f"    ⇒ {a_tot - len(staged)} run MEANINGFULLY there. The driver now prints this itself, so a")
print( "      green tick cannot imply the staged-scope four were satisfied.")
PYEOF
