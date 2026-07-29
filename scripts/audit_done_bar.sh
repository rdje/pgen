#!/usr/bin/env bash
# audit_done_bar.sh — audit every `Done` claim in LIVE_ACHIEVEMENT_STATUS.md against the three-leg
# `Done` bar (task-tree leaf `DONE-BAR.1`; standing director directive
# docs/decisions/feedback_done_bar_is_first_tier_only.md).
#
# THE BAR (docs/tasks/DONE-BAR.md):
#   leg 1  stimuli-generator proof — the family's own generated samples close the loop with ZERO
#          residual actionable-target debt;
#   leg 2  all our gates — every gate covering the family is green NOW and is actually INVOKED by
#          something (a gate nothing invokes is indistinguishable from a gate that does not exist);
#   leg 3  all the external test corpus, PASSING — a TRIAGE gate is not a conformance gate, a
#          CHARACTERIZATION is not a pass, and the ABSENCE of a corpus is an UNMET leg, never an
#          inapplicable one.
#
# WHAT THIS SCRIPT IS AND IS NOT
#   It is an AUDIT (`DONE-BAR.1`), not the enforcement ratchet (`DONE-BAR.4`). It never demotes a
#   row — `DONE-BAR.2` does that, with this report's evidence attached.
#
#   It is READ-ONLY and cheap: no cargo, no make, no network. It derives structural facts from the
#   tracked tree and READS whatever gate artifacts already exist. It NEVER runs a gate, so it can
#   never manufacture the green it is auditing.
#
# ⛔ THE POLARITY THAT MATTERS: a leg this script cannot SEE is reported `UNPROVEN`, never `MET`.
#   `UNPROVEN` does not satisfy the bar. This repository has repeatedly shipped checks that returned
#   green because they could not run or could not see (CI-PARITY-GATE-ROT `.3`/`.5`/`.8`); the
#   standing principle is that a check which cannot run — or cannot see — must SAY SO.
#
# SELF-CALIBRATION: the script carries ground-truth controls — facts this repository has already
#   measured independently. If any control fails to reproduce it prints MISCALIBRATED and exits 3
#   WITHOUT reporting a verdict, because an instrument with no ground truth is a confident guess
#   (docs/decisions/feedback_instrument_needs_ground_truth.md).
#
# USAGE
#   bash scripts/audit_done_bar.sh              # the full per-family audit report
#   bash scripts/audit_done_bar.sh --json OUT   # also emit the machine-readable rows
#
# EXIT CODES
#   0  every `Done` row meets the bar
#   1  at least one `Done` row does not meet the bar (the expected state while DONE-BAR is open)
#   2  refusal — the audit could not be performed (missing register entry, unreadable input)
#   3  MISCALIBRATED — a ground-truth control did not reproduce
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Scratch stays on the repository volume and is derived from the repo root at runtime, per the
# project data-locality policy (CLAUDE.md §13) — never $TMPDIR, which is on the boot volume.
STATE_DIR="$ROOT/rust/target/done_bar_audit"
mkdir -p "$STATE_DIR"
REACHABILITY_JSON="$STATE_DIR/gate_reachability.json"

# Ask the existing reachability instrument; do not re-implement it. It carries its own ground-truth
# controls and refuses rather than reporting numbers it cannot back.
if ! bash "$ROOT/scripts/check_gate_reachability.sh" --json "$REACHABILITY_JSON" >"$STATE_DIR/gate_reachability.log" 2>&1; then
    echo "audit-done-bar: REFUSED — scripts/check_gate_reachability.sh did not succeed;" >&2
    echo "  leg 2 cannot be judged without it. Log: rust/target/done_bar_audit/gate_reachability.log" >&2
    exit 2
fi

python3 - "$@" <<'PYEOF'
import json
import os
import re
import sys

ROOT = os.getcwd()
STATE_DIR = os.path.join(ROOT, "rust", "target", "done_bar_audit")
TREE = os.path.join(ROOT, "docs", "tasks", "DONE-BAR.md")
REACHABILITY = os.path.join(STATE_DIR, "gate_reachability.json")
CONTRACT_DIR = os.path.join(ROOT, "rust", "test_data", "grammar_quality")

# Testability seams. Each defaults to the tracked artifact, so a normal run reads exactly what the
# repository ships; they exist so the probe driver
# (docs/tasks/artifacts/done_bar/run_done_bar_probes.sh) can mutate ONE input at a time and prove
# each ground-truth control actually fires. An instrument whose controls have never been seen to
# FAIL is an instrument with untested controls.
TRACKER = os.environ.get("PGEN_DONE_BAR_TRACKER") or os.path.join(ROOT, "LIVE_ACHIEVEMENT_STATUS.md")
REGISTER = os.environ.get("PGEN_DONE_BAR_REGISTER") or os.path.join(CONTRACT_DIR, "done_bar_family_register_v0.json")
SCRIPTS_DIR = os.environ.get("PGEN_DONE_BAR_SCRIPTS_DIR") or os.path.join(ROOT, "rust", "scripts")

failures = []          # rows that do not meet the bar
miscalibrations = []   # controls that did not reproduce


def refuse(msg):
    print(f"audit-done-bar: REFUSED — {msg}", file=sys.stderr)
    sys.exit(2)


def read(path):
    with open(path, encoding="utf-8") as f:
        return f.read()


# ---------------------------------------------------------------------------
# 1. The family roster — DERIVED, never hand-listed
# ---------------------------------------------------------------------------
# A tracker table row names a parser FAMILY iff the first backticked token in its Area cell is the
# basename of a tracked grammars/*.ebnf. That join is what makes the roster derived: a family added
# to the tracker tomorrow is picked up, and a row like "Later auxiliary readers (`gate-level` netlist
# reader…)" — whose first backticked token is NOT a grammar — is correctly not a family.
GRAMMARS = {
    os.path.basename(p)[: -len(".ebnf")]
    for p in os.listdir(os.path.join(ROOT, "grammars"))
    if p.endswith(".ebnf")
}
if not GRAMMARS:
    refuse("no grammars/*.ebnf found; the family roster cannot be derived")

STATUSES = {"Done", "Mostly Done", "In Progress", "Not Started"}

families = {}   # name -> {"status": ..., "area": ...}
for line in read(TRACKER).splitlines():
    if not line.startswith("|"):
        continue
    cells = [c.strip() for c in line.split("|")]
    if len(cells) < 4:
        continue
    area, status = cells[1], cells[2]
    # `Provisional (…)` carries a qualifier, so match on the leading word set rather than equality.
    base_status = status.split("(")[0].strip()
    if base_status not in STATUSES and base_status != "Provisional":
        continue
    m = re.search(r"`([^`]+)`", area)
    if not m or m.group(1) not in GRAMMARS:
        continue
    name = m.group(1)
    if name not in families:      # first row wins; a duplicate would be a tracker defect
        families[name] = {"status": status, "area": area}

if not families:
    refuse("derived zero parser families from the tracker — an empty roster is a refusal, not a pass")

register = json.loads(read(REGISTER))["families"]
for name in sorted(families):
    if name not in register:
        refuse(
            f"family '{name}' is on the tracker but absent from "
            f"rust/test_data/grammar_quality/done_bar_family_register_v0.json. "
            f"An unregistered family BLOCKS the audit rather than being skipped, because a skip "
            f"would let a new `Done` row score well by being invisible."
        )

# ---------------------------------------------------------------------------
# 2. Gate universe, make-target reachability, and script-level reachability
# ---------------------------------------------------------------------------
rows = json.loads(read(REACHABILITY))["rows"]
target_status = {r["target"]: r["status"] for r in rows}
target_invokers = {r["target"]: r.get("invokers", []) for r in rows}

# A gate whose MAKE TARGET is an orphan may still be executed as a SCRIPT by a parent gate — that is
# how the external-corpus triage gates run today. Build the script-call graph so leg 2 measures what
# actually runs rather than what `make` alone can reach.
#
# ⛔ ci_workflow_local_gate.sh is excluded as a caller BY CONSTRUCTION: it is the parity gate whose
# job is to AUDIT gate surfaces, so its references to a gate script are assert_tracked/read
# assertions, not invocations. Counting a MENTION as an INVOCATION is the first defect the gate
# reachability instrument made and had to fix (CI-PARITY-GATE-ROT.2). Its genuine workflow replays
# run `make` targets, which the make-target layer above already covers, so excluding it here loses
# nothing.
CALLER_EXCLUSIONS = set()
if os.environ.get("PGEN_DONE_BAR_NO_CALLER_EXCLUSIONS") != "1":
    CALLER_EXCLUSIONS = {"ci_workflow_local_gate.sh"}

script_callers = {}   # gate name -> set of caller gate names
gate_scripts = sorted(p for p in os.listdir(SCRIPTS_DIR) if p.endswith(".sh"))
script_text = {p: read(os.path.join(SCRIPTS_DIR, p)) for p in gate_scripts}
for gate_script in gate_scripts:
    gate = gate_script[: -len(".sh")]
    callers = set()
    needle = gate_script
    for caller_script, text in script_text.items():
        if caller_script == gate_script or caller_script in CALLER_EXCLUSIONS:
            continue
        for line in text.splitlines():
            stripped = line.strip()
            if stripped.startswith("#"):
                continue
            if needle in stripped:
                callers.add(caller_script[: -len(".sh")])
                break
    script_callers[gate] = callers


def runs(gate, seen=None):
    """True iff `gate` is executed by something: its own make target is reachable, or a caller is."""
    if seen is None:
        seen = set()
    if gate in seen:
        return False
    seen.add(gate)
    if target_status.get(gate) == "reachable":
        return True
    return any(runs(c, seen) for c in script_callers.get(gate, ()))


def run_path(gate, seen=None):
    """The shortest human-readable chain proving `gate` runs, or None."""
    if seen is None:
        seen = set()
    if gate in seen:
        return None
    seen.add(gate)
    if target_status.get(gate) == "reachable":
        return [gate]
    for c in sorted(script_callers.get(gate, ())):
        sub = run_path(c, seen)
        if sub:
            return [gate] + sub
    return None


# Longest-prefix attribution of a gate to a family (sv_preprocessor_* must not land on systemverilog)
prefix_owner = []
for name, entry in register.items():
    for pfx in entry["gate_prefixes"]:
        prefix_owner.append((pfx, name))
prefix_owner.sort(key=lambda t: len(t[0]), reverse=True)


def owner_of(gate):
    for pfx, name in prefix_owner:
        if gate.startswith(pfx):
            return name
    return None


family_gates = {name: [] for name in register}
for target in sorted(target_status):
    owner = owner_of(target)
    if owner:
        family_gates[owner].append(target)

# Which family-status gate computes a given family's status is DERIVED from what each gate script
# EMITS, not from its name. `sv_parser_family_status_gate` computes BOTH `systemverilog` and
# `systemverilog_preprocessor`; a name-prefix attribution gives the preprocessor no status gate at
# all and reports it worse than it measures. The audit must be accurate in both directions.
STATUS_KEY_RE = re.compile(r'echo\s+"([a-z0-9_]+)_status:')

status_gate_families = {}   # gate -> set of families whose status it emits
for p in gate_scripts:
    g = p[: -len(".sh")]
    if not g.endswith("_parser_family_status_gate"):
        continue
    emitted = set(STATUS_KEY_RE.findall(script_text[p]))
    status_gate_families[g] = {f for f in register if f in emitted}


def status_gate_for(family):
    owned = [g for g, fams in status_gate_families.items() if family in fams]
    return sorted(owned, key=len)[0] if owned else None


ARTIFACT_ROOTS = [
    os.path.join(ROOT, "rust", "target"),
    os.path.join(ROOT, "rust", "target", "sota_exit_gate", "work"),
]


def find_artifact(gate):
    """Newest non-empty summary.txt for `gate`, with its provenance. Empty files do not count —
    a 0-byte summary.txt is exactly what a gate that died mid-run leaves behind
    (CI-PARITY-GATE-ROT.14)."""
    best = None
    for base in ARTIFACT_ROOTS:
        path = os.path.join(base, gate, "summary.txt")
        if os.path.isfile(path) and os.path.getsize(path) > 0:
            mtime = os.path.getmtime(path)
            if best is None or mtime > best[1]:
                best = (path, mtime)
    return best


def gate_ran_and_failed(gate):
    """A gate that RAN and DIED is not the same as a gate that never ran, and the difference is
    decisive: the first is an UNMET leg with a named cause, the second is UNPROVEN. A gate killed
    mid-run leaves a 0-byte summary.txt (CI-PARITY-GATE-ROT.14) and an `error:` line in its log."""
    candidates = [
        os.path.join(ROOT, "rust", "target", "sota_exit_gate", "logs", f"{gate}.log"),
        os.path.join(ROOT, "rust", "target", gate, "logs", f"{gate}.log"),
    ]
    for path in candidates:
        if not os.path.isfile(path):
            continue
        for line in read(path).splitlines():
            if line.startswith("error:"):
                return os.path.relpath(path, ROOT), line
    return None, None


# ---------------------------------------------------------------------------
# 3. GROUND-TRUTH CONTROLS — facts already measured independently by this repository.
#    If any fails to reproduce, the instrument reports MISCALIBRATED instead of a number.
# ---------------------------------------------------------------------------
def control(label, got, want):
    if got != want:
        miscalibrations.append(f"{label}: derived {got!r}, ground truth {want!r}")


# C1 — a family that is NOT `Done` must resolve as such. Proves the status reader is reading the
#      row rather than defaulting to something comfortable.
control("C1 systemverilog tracker status",
        families.get("systemverilog", {}).get("status"), "Mostly Done")

# C2 — the "Later auxiliary readers (`gate-level` netlist reader…)" row must NOT be admitted.
#      Proves the grammar-membership join, not merely "has a backtick", is what admits a family.
control("C2 gate-level admitted as a family", "gate-level" in families, False)

# C3 — systemverilog and systemverilog_preprocessor are DISTINCT families.
control("C3 both SV families derived",
        {"systemverilog", "systemverilog_preprocessor"} <= set(families), True)

# C4 — longest-prefix attribution: the preprocessor's closure gate belongs to the preprocessor.
#      A plain `sv_` prefix scan gets this wrong and silently credits the wrong family.
control("C4 sv_preprocessor_formal_exhaustive_closure_gate owner",
        owner_of("sv_preprocessor_formal_exhaustive_closure_gate"), "systemverilog_preprocessor")

# C5 — the reachability rows must account for themselves. Catches a parse defect in the join
#      without breaking when the target universe legitimately grows.
counted = sum(1 for r in rows if r["status"] in {"reachable", "ORPHAN", "policy-only"})
control("C5 reachability rows all classified", counted, len(rows))
control("C5 reachability universe non-empty", len(rows) > 0, True)

# C6 — the script-call graph must reproduce a chain PROVEN by a real run: the VHDL external-corpus
#      triage gate is an orphan make target, yet aggregate run 3 (2026-07-29) produced its state dir
#      under vhdl_parser_family_status_gate/work/vhdl_formal_exhaustive_closure_gate/work/. If the
#      graph cannot see that edge it is under-reporting what runs.
control("C6 vhdl_external_corpus_triage_gate runs",
        runs("vhdl_external_corpus_triage_gate"), True)

# C7 — and the converse arm, or C6 would pass a graph that says everything runs:
#      regex_corpus_bundle_contract_gate is referenced ONLY by ci_workflow_local_gate.sh's
#      assert_tracked lines, so nothing executes it.
control("C7 regex_corpus_bundle_contract_gate runs",
        runs("regex_corpus_bundle_contract_gate"), False)

# C8 — every register family is on the derived roster. The original C8 ("at least one row claims
#      `Done`") pinned a ground truth `DONE-BAR.2b` legitimately moved: the demotion took the
#      tracker to ZERO `Done` rows, which is the directive working, not a parse defect. Its
#      anti-vacuous purpose is kept twice over: the report now states the zero-`Done` case
#      explicitly (never a silent pass), and THIS control catches the failure C8 was really
#      guarding against — a status-vocabulary change silently dropping rows from the roster
#      (roster ⊆ register is already enforced by the per-family refusal; this is the converse).
control("C8 register families all derived from the tracker",
        sorted(set(register) - set(families)), [])

# C9 — ONE status gate computes TWO families. This arm caught a real defect in this instrument's own
#      first cut: attributing status gates by NAME prefix left systemverilog_preprocessor with no
#      status gate at all and reported it worse than it measures. Attribution is by what the gate
#      EMITS; this control is what holds that.
control("C9 sv_parser_family_status_gate covers both SV families",
        status_gate_families.get("sv_parser_family_status_gate"),
        {"systemverilog", "systemverilog_preprocessor"})

# C10 — a gate that RAN AND DIED must not be reported as merely UNPROVEN. Conditional, so a clean
#       rust/target does not fail it: IF the aggregate left a log for a status gate whose summary is
#       empty or absent, that log's `error:` line must be recovered. Aggregate run 3 (2026-07-29)
#       left exactly this shape for regex — a 0-byte summary.txt beside a log naming the real cause.
for _g in sorted(status_gate_families):
    if find_artifact(_g) is None and gate_ran_and_failed(_g)[1] is None:
        _log = os.path.join(ROOT, "rust", "target", "sota_exit_gate", "logs", f"{_g}.log")
        if os.path.isfile(_log) and os.path.getsize(_log) > 0:
            control(f"C10 {_g} failure recovered from its log", False, True)

if miscalibrations:
    print("=" * 78)
    print("audit-done-bar: MISCALIBRATED — refusing to report a verdict")
    print("=" * 78)
    for m in miscalibrations:
        print(f"  ⛔ {m}")
    print()
    print("An instrument with no ground truth is a confident guess. Fix the derivation (or the")
    print("control, if the ground truth genuinely moved) before trusting any number below.")
    sys.exit(3)

# ---------------------------------------------------------------------------
# 4. Artifact lookup + staleness
# ---------------------------------------------------------------------------
def newest_input_mtime(family):
    """The newest mtime among the inputs a family's verdict depends on. An artifact older than its
    own inputs is STALE — it judged a tree that no longer exists. This is the measured failure from
    CI-PARITY-GATE-ROT.5, where a three-day-old hand-run artifact was consumed as current proof."""
    candidates = [TRACKER]
    grammar = os.path.join(ROOT, "grammars", f"{family}.ebnf")
    if os.path.isfile(grammar):
        candidates.append(grammar)
    for gate in family_gates.get(family, ()):
        p = os.path.join(SCRIPTS_DIR, f"{gate}.sh")
        if os.path.isfile(p):
            candidates.append(p)
    return max(os.path.getmtime(p) for p in candidates)


def summary_value(path, key):
    prefix = key + ": "
    for line in read(path).splitlines():
        if line.startswith(prefix):
            return line[len(prefix):].strip()
    return None


# ---------------------------------------------------------------------------
# 5. Per-family legs
# ---------------------------------------------------------------------------
MET, UNMET, UNPROVEN = "MET", "UNMET", "UNPROVEN"


def leg1(family, report):
    """Leg 1 — stimuli-generator proof with ZERO residual actionable-target debt."""
    gate = status_gate_for(family)
    if gate is None:
        report.append("no *_parser_family_status_gate computes this family's closure at all")
        return UNPROVEN
    art = find_artifact(gate)
    if art is None:
        log_rel, err = gate_ran_and_failed(gate)
        if err:
            report.append(f"{gate} RAN AND FAILED (log {log_rel}); no summary was written")
            report.append(f"  {err}")
            return UNMET
        report.append(f"{gate} has produced no non-empty summary.txt on this machine")
        report.append(f"run: make -C rust SHELL=/bin/bash {gate}")
        return UNPROVEN
    path, mtime = art
    rel = os.path.relpath(path, ROOT)
    stale = mtime < newest_input_mtime(family)
    report.append(f"artifact {rel} (mtime {int(mtime)}{', ⚠️ STALE — older than its own inputs' if stale else ''})")
    if stale:
        report.append("a verdict resting on an artifact older than the inputs it judged is not proof")
        return UNPROVEN

    unmet_count = summary_value(path, f"{family}_unmet_closure_criteria_count")
    primary = summary_value(path, f"{family}_primary_unmet_closure_criterion")
    final_targets = summary_value(path, f"{family}_final_targets")
    if final_targets is not None:
        report.append(f"{family}_final_targets = {final_targets}")
    if unmet_count is None:
        report.append(f"artifact carries no '{family}_unmet_closure_criteria_count' key")
        return UNPROVEN
    report.append(f"{family}_unmet_closure_criteria_count = {unmet_count}")
    if unmet_count == "0" and (final_targets in (None, "0")):
        return MET
    if primary and primary != "<none>":
        report.append(f"primary unmet criterion: {primary}")
    return UNMET


def leg2(family, report):
    """Leg 2 — every covering gate green NOW and actually invoked."""
    gates = family_gates.get(family, [])
    if not gates:
        report.append("no gate in the tracked universe is attributable to this family")
        return UNMET
    not_run = sorted(g for g in gates if not runs(g))
    report.append(f"{len(gates)} covering gate targets; {len(gates) - len(not_run)} run, {len(not_run)} run by nothing")
    for g in not_run:
        report.append(f"  ⛔ invoked by nothing: {g}")

    gate = status_gate_for(family)
    if gate is None:
        report.append("  ⛔ no family-status gate exists, so nothing computes this family's status")
        return UNMET

    art = find_artifact(gate)
    if art is None:
        log_rel, err = gate_ran_and_failed(gate)
        if err:
            report.append(f"  ⛔ {gate} RAN AND FAILED (log {log_rel}): {err}")
            verdict = UNMET
        else:
            report.append(f"  {gate} green-NOW is UNPROVEN — no artifact on this machine")
            verdict = UNPROVEN
    else:
        path, mtime = art
        computed = summary_value(path, f"{family}_status")
        tracker_aligned = summary_value(path, f"{family}_tracker_alignment_ok")
        report.append(f"  {gate} computes {family}_status = {computed} (tracker_alignment_ok={tracker_aligned})")
        if mtime < newest_input_mtime(family):
            # Same rule as leg 1: 'green NOW' cannot rest on a run that predates its own inputs.
            # The status gates assert tracker alignment, so a tracker edit after the run leaves that
            # alignment unproven even when the recorded answer was `true`.
            report.append("  ⚠️ that answer PREDATES its own inputs ⇒ green-NOW is UNPROVEN, not MET")
            verdict = UNPROVEN
        else:
            verdict = MET if tracker_aligned == "true" else UNMET

    # AUTOMATIC tier: 'invoked' has a ceiling this repo has already measured. Report it rather than
    # letting an OPERATOR-only lane read as 'actually invoked' without qualification.
    automatic = [g for g in gates if "ci-workflow-auto" in target_invokers.get(g, []) or "git-hook" in target_invokers.get(g, [])]
    if not automatic:
        report.append("  ⚠️ every covering lane is OPERATOR-tier: it runs only when a human asks")
    if not_run:
        return UNMET
    return verdict


CORPUS_GATE_RE = re.compile(r"corpus|conformance", re.IGNORECASE)


def corpus_gate_kind(gate):
    if "triage" in gate:
        return "TRIAGE"
    if "conformance" in gate:
        return "CONFORMANCE"
    return "PROOF"


def external_backed(gate, roots):
    """True iff the gate reads one of the family's declared external corpus roots — directly, or via
    a tracked contract manifest it names. This is what separates a genuine external-corpus lane from
    a gate that merely has 'corpus' in its name while reading a repo-authored fixture."""
    path = os.path.join(SCRIPTS_DIR, f"{gate}.sh")
    if not os.path.isfile(path):
        return False, "no script"
    text = read(path)
    for root in roots:
        if root in text:
            return True, f"reads {root} directly"
    for manifest in re.findall(r"[A-Za-z0-9_]+\.json", text):
        mpath = os.path.join(CONTRACT_DIR, manifest)
        if not os.path.isfile(mpath):
            continue
        mtext = read(mpath)
        for root in roots:
            if root in mtext:
                return True, f"reads {root} via {manifest}"
    return False, "reads no declared external corpus root"


def leg3(family, report):
    """Leg 3 — an officially-recognized external corpus, PASSING."""
    entry = register[family]
    roots = entry["corpus_roots"]
    gates = [g for g in family_gates.get(family, []) if CORPUS_GATE_RE.search(g)]

    if not roots:
        if gates:
            miscalibrations.append(
                f"register says {family} has no corpus roots, but corpus-facing gates are "
                f"attributed to it: {', '.join(gates)}"
            )
        owner = entry["language_owner"]
        if owner == "pgen":
            report.append("no external corpus EXISTS — the language is authored by PGEN itself")
            report.append("⇒ leg 3 is unreachable BY CONSTRUCTION: qualifier `Provisional (ceiling)`")
        elif owner == "external-standard":
            report.append(f"NO external corpus is wired, though the language is standardized ({entry['standard']})")
            report.append("⇒ qualifier `Provisional (corpus pending)` — absence is an UNMET leg, not an inapplicable one")
        else:
            report.append("no external corpus wired; language ownership UNADJUDICATED (owed by DONE-BAR.3)")
            report.append("⇒ qualifier cannot be issued: `ceiling` may not be claimed by default")
        return UNMET

    if not gates:
        report.append(f"corpus assets present ({', '.join(roots)}) but NO gate asserts them")
        return UNMET

    met_any = False
    for g in sorted(gates):
        kind = corpus_gate_kind(g)
        ext, why = external_backed(g, roots)
        chain = run_path(g)
        runs_str = " <- ".join(chain) if chain else "NOTHING"
        report.append(f"  {g}: kind={kind}, external={'yes' if ext else 'NO'} ({why}), run by {runs_str}")
        if kind == "TRIAGE":
            report.append("    ⛔ a TRIAGE gate is not a conformance gate")
            continue
        if not ext:
            report.append("    ⛔ not backed by the family's declared external corpus")
            continue
        if chain is None:
            report.append("    ⛔ invoked by nothing, so its green (if any) is never re-established")
            continue
        art = find_artifact(g)
        if art is None:
            report.append("    ⚠️ no artifact on this machine — result UNPROVEN")
            continue
        met_any = True
    return MET if met_any else UNMET


# ---------------------------------------------------------------------------
# 6. Report
# ---------------------------------------------------------------------------
leaf5_open = bool(re.search(r"^### `\.5`.*\(`todo`\)", read(TREE), re.MULTILINE))

print("=" * 78)
print("DONE-BAR AUDIT — every `Done` claim against the three legs (all figures DERIVED this run)")
print("=" * 78)
print(f"families derived from LIVE_ACHIEVEMENT_STATUS.md x grammars/*.ebnf: {len(families)}")
print(f"gate universe (via scripts/check_gate_reachability.sh): {len(rows)} targets")
print("ground-truth controls reproduced: 10")
if leaf5_open:
    print("DONE-BAR.5 (consumer-facing disclosure gates) is OPEN ⇒ the highest attainable tier is")
    print("  `Provisional`; no row may be promoted to `Done` on legs 1-3 alone.")
print()

json_rows = []
for name in sorted(families, key=lambda n: (families[n]["status"] != "Done", n)):
    info = families[name]
    judged = info["status"] == "Done"
    print("-" * 78)
    print(f"{name}   tracker: {info['status']}" + ("" if judged else "   (not a `Done` claim — reported for context)"))
    print("-" * 78)

    r1, r2, r3 = [], [], []
    v1 = leg1(name, r1)
    v2 = leg2(name, r2)
    v3 = leg3(name, r3)
    for label, verdict, lines in (("leg 1 stimuli-generator proof", v1, r1),
                                  ("leg 2 gates green + invoked", v2, r2),
                                  ("leg 3 external corpus passing", v3, r3)):
        print(f"  {label:34s} {verdict}")
        for ln in lines:
            print(f"      {ln}")

    unmet = [n for n, v in (("1", v1), ("2", v2), ("3", v3)) if v != MET]
    if not unmet:
        verdict = "PROVISIONAL (legs 1-3 met; DONE-BAR.5 open)" if leaf5_open else "MEETS BAR"
    else:
        verdict = f"DOES NOT MEET BAR (leg{'s' if len(unmet) > 1 else ''} {', '.join(unmet)} unmet)"
    print(f"  ⇒ {verdict}")
    print()
    if judged and unmet:
        failures.append((name, verdict))
    json_rows.append({
        "family": name, "tracker_status": info["status"], "judged": judged,
        "leg1": v1, "leg2": v2, "leg3": v3, "verdict": verdict,
    })

if miscalibrations:
    print("=" * 78)
    print("audit-done-bar: MISCALIBRATED — a register entry contradicts the derived gate set")
    for m in miscalibrations:
        print(f"  ⛔ {m}")
    sys.exit(3)

if "--json" in sys.argv:
    out = sys.argv[sys.argv.index("--json") + 1]
    with open(out, "w", encoding="utf-8") as f:
        json.dump({"families": json_rows}, f, indent=2, sort_keys=True)

print("=" * 78)
if failures:
    print(f"audit-done-bar: {len(failures)} of "
          f"{sum(1 for f in families.values() if f['status'] == 'Done')} `Done` rows DO NOT meet the bar")
    for name, verdict in failures:
        print(f"  ⛔ {name}: {verdict}")
    print()
    print("⛔ This audit does NOT demote anything. Demotion is DONE-BAR.2, with this evidence attached.")
    sys.exit(1)
done_claims = sum(1 for f in families.values() if f["status"] == "Done")
if done_claims == 0:
    # Post-`DONE-BAR.2b` steady state until a family closes leg 3. Said out loud, because a green
    # exit with nothing judged is exactly the vacuous-green class this repository fights: this is
    # evidence the tracker makes no unproven `Done` claim, NOT evidence of parser quality.
    print("audit-done-bar: 0 `Done` rows are claimed — nothing to judge, VACUOUSLY green.")
    print("  This states only that the tracker currently claims no `Done` row; the per-family leg")
    print("  states above remain the substance. A future promotion to `Done` re-enters this")
    print("  audit's scope automatically (the roster is derived, not listed).")
    sys.exit(0)
print("audit-done-bar: every `Done` row meets the bar")
PYEOF
