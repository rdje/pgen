#!/usr/bin/env bash
# audit_done_bar.sh — audit every `Done` CLAIM against the three-leg `Done` bar (task-tree leaf
# `DONE-BAR.1`; standing director directive docs/decisions/feedback_done_bar_is_first_tier_only.md).
#
# The claim is the register's hand-authored `claimed_status` (moved out of LIVE_ACHIEVEMENT_STATUS.md
# by LIVE-MEANS-LIVE.1a), and the family ROSTER is derived from grammars/*.ebnf (`.1b`) — so this
# script no longer reads that tracker at all. See §1 for why the roster is derived that way round.
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
#   2  refusal — the audit could not be performed: a tracked grammar adjudicated NEITHER as a family
#      NOR with a recorded disposition, one adjudicated BOTH ways, a register entry naming a grammar
#      that does not exist, a family with no `claimed_status`, or an unreadable input
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
#
# ⛔ LIVE_ACHIEVEMENT_STATUS.md is deliberately ABSENT from this list. The family-status CLAIM moved
# into the register's `claimed_status` field (LIVE-MEANS-LIVE.1a) and the family ROSTER is now
# derived from grammars/*.ebnf (`.1b`), so this audit reads the tracker for nothing at all.
REGISTER = os.environ.get("PGEN_DONE_BAR_REGISTER") or os.path.join(CONTRACT_DIR, "done_bar_family_register_v0.json")
SCRIPTS_DIR = os.environ.get("PGEN_DONE_BAR_SCRIPTS_DIR") or os.path.join(ROOT, "rust", "scripts")
GRAMMARS_DIR = os.environ.get("PGEN_DONE_BAR_GRAMMARS_DIR") or os.path.join(ROOT, "grammars")
# The build-output root holding gate artifacts + logs. A seam because a ground-truth control that
# pins UNTRACKED state decays silently: CTRL-4a pinned a failure recorded in rust/target/ and went
# red the moment that gate was re-run and passed (`.1b`). A control must pin a TRACKED fact or
# CONSTRUCT the state it observes; this seam is what lets it construct.
TARGET_DIR = os.environ.get("PGEN_DONE_BAR_TARGET_DIR") or os.path.join(ROOT, "rust", "target")

failures = []          # rows that do not meet the bar
miscalibrations = []   # controls that did not reproduce


def refuse(msg):
    print(f"audit-done-bar: REFUSED — {msg}", file=sys.stderr)
    sys.exit(2)


def read(path):
    with open(path, encoding="utf-8") as f:
        return f.read()


# ---------------------------------------------------------------------------
# 1. The family roster — DERIVED FROM THE PRODUCT, never hand-listed
# ---------------------------------------------------------------------------
# ⭐ The candidate set is `grammars/*.ebnf` — the product itself, which cannot lie about what
# exists. Every tracked grammar must then be adjudicated EXACTLY ONCE, as either a parser family
# (`families`) or a recorded non-family (`grammar_dispositions`). This is the GATE-REACHABILITY
# pattern — invoked, or a deliberate disposition — applied to families.
#
# ⛔ WHY IT IS THIS WAY ROUND (LIVE-MEANS-LIVE.1b). The roster used to be
# `LIVE_ACHIEVEMENT_STATUS.md rows ∩ grammars/*.ebnf`, whose LEFT side was the tracker: a grammar
# with no tracker row contributed nothing and the derivation never saw it. The only refusal fired on
# the converse arm (on the tracker, absent from the register), so every check guarding the roster
# was on the side that could not fail. Measured, it hid THREE shipped parsers with registered
# generated parsers — `ebnf`, `json` and `semantic_annotation`, the last with a published downstream
# integration contract — plus the 8 gate targets attributed to them. Deriving from the grammars
# makes the check two-sided, which is the whole point: it can now fail.
GRAMMARS = {
    os.path.basename(p)[: -len(".ebnf")]
    for p in os.listdir(GRAMMARS_DIR)
    if p.endswith(".ebnf")
}
if not GRAMMARS:
    refuse("no grammars/*.ebnf found — an empty roster is a refusal, not a pass")

_register_doc = json.loads(read(REGISTER))
register = _register_doc["families"]
dispositions = _register_doc.get("grammar_dispositions", {})

# Arm 1 — a grammar adjudicated BOTH ways. Checked first: it makes the two arms below ambiguous.
_both = sorted(set(register) & set(dispositions))
if _both:
    refuse(
        f"grammar(s) {', '.join(_both)} appear in BOTH `families` and `grammar_dispositions` of "
        f"rust/test_data/grammar_quality/done_bar_family_register_v0.json. A grammar is a family "
        f"or it is not; a contradictory register cannot be audited."
    )

# Arm 2 — THE NEW REFUSAL. A tracked grammar adjudicated NEITHER way blocks the audit. A skip here
# is exactly the silence this derivation replaced: it would let a shipped parser stay invisible.
_undisposed = sorted(g for g in GRAMMARS if g not in register and g not in dispositions)
if _undisposed:
    refuse(
        f"tracked grammar(s) {', '.join(_undisposed)} have no register entry and no recorded "
        f"disposition in rust/test_data/grammar_quality/done_bar_family_register_v0.json. Add a "
        f"`families` entry (if the grammar ships a registered generated parser) or a "
        f"`grammar_dispositions` entry with a reason. An unadjudicated grammar BLOCKS the audit "
        f"rather than being skipped, because a skip is how a shipped parser scores well by being "
        f"invisible."
    )

# Arm 3 — the converse. An entry naming a grammar that does not exist is a STALE adjudication: the
# grammar was deleted or renamed and the register still claims to cover it. Without this, the
# register could drift into describing a tree that no longer exists — the same staleness class
# CI-PARITY-GATE-ROT.5 measured on gate artifacts.
_stale = sorted((set(register) | set(dispositions)) - GRAMMARS)
if _stale:
    refuse(
        f"register entr(y|ies) {', '.join(_stale)} name no tracked grammars/*.ebnf file. A stale "
        f"adjudication describes a tree that no longer exists; remove it or restore the grammar."
    )

# The roster is the adjudicated families, and their status is the HAND-AUTHORED claim the register
# carries (`claimed_status`, moved here from the tracker by LIVE-MEANS-LIVE.1a). ⛔ It is never
# defaulted: a family whose claim is missing or empty blocks the audit, because a family with no
# claim would otherwise be audited against nothing and pass.
families = {}   # name -> {"status": ...}
for name in sorted(register):
    claim = register[name].get("claimed_status")
    if not claim or not str(claim).strip():
        refuse(
            f"family '{name}' carries no `claimed_status` in the register. That field is the "
            f"HAND-AUTHORED arm of the status check and is never defaulted — a missing claim is a "
            f"refusal, not a family with no opinion."
        )
    families[name] = {"status": str(claim).strip()}

if not families:
    refuse("derived zero parser families — an empty roster is a refusal, not a pass")

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
    TARGET_DIR,
    os.path.join(TARGET_DIR, "sota_exit_gate", "work"),
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
    mid-run leaves a 0-byte summary.txt (CI-PARITY-GATE-ROT.14) and an `error:` line in its log.

    Returns (rel_path, error_line, mtime) so the CALLER can apply the same staleness rule the
    artifact path applies. ⛔ DONE-BAR.1a: this branch used to return no vintage at all, and it is
    the branch MOST exposed to staleness — a gate that failed wrote no summary, so the only evidence
    left is a log of arbitrary age. Measured instance: `regex_parser_family_status_gate.log` (mtime
    2026-07-29 13:49) carried `error: regex tracker alignment mismatch: computed 'In Progress' but
    tracker says 'Done'`, and `DONE-BAR.2b` moved that tracker row to `In Progress` 4h48m LATER
    (b704e1ab, 18:37:42) — so the audit printed `tracker: In Progress` and, two lines below, a ⛔
    quoting an error about the tracker saying `Done`. The complaint was true when written and false
    when read."""
    candidates = [
        os.path.join(TARGET_DIR, "sota_exit_gate", "logs", f"{gate}.log"),
        os.path.join(TARGET_DIR, gate, "logs", f"{gate}.log"),
    ]
    for path in candidates:
        if not os.path.isfile(path):
            continue
        for line in read(path).splitlines():
            if line.startswith("error:"):
                return os.path.relpath(path, ROOT), line, os.path.getmtime(path)
    return None, None, None


# ---------------------------------------------------------------------------
# 3. GROUND-TRUTH CONTROLS — facts already measured independently by this repository.
#    If any fails to reproduce, the instrument reports MISCALIBRATED instead of a number.
# ---------------------------------------------------------------------------
CONTROL_COUNT = 0


def control(label, got, want):
    """Assert a ground truth this repository measured independently of this script.

    ⚠️ The count is DERIVED from the calls, not typed into the report. The header used to carry the
    literal `10`, which is a second place to forget: adding a control without touching the string
    would have under-reported the instrument's own calibration."""
    global CONTROL_COUNT
    CONTROL_COUNT += 1
    if got != want:
        miscalibrations.append(f"{label}: derived {got!r}, ground truth {want!r}")


# C1 — a family that is NOT `Done` must resolve as such. Proves the claim reader is reading the
#      register field rather than defaulting to something comfortable. (Reads `claimed_status`
#      since LIVE-MEANS-LIVE.1a; the value itself is unchanged from the tracker row it replaced.)
control("C1 systemverilog claimed status",
        families.get("systemverilog", {}).get("status"), "Mostly Done")

# C2 — a grammar carrying an explicit non-family DISPOSITION must NOT be admitted as a family.
#      The disposition arm is what keeps the new refusal satisfiable rather than punitive, so it is
#      also what would silently swallow a real family if it ever leaked into the roster.
control("C2 dispositioned grammar admitted as a family",
        bool(set(dispositions) & set(families)), False)
control("C2b builtin bootstrap contract is dispositioned, not a family",
        dispositions.get("builtin_return_annotation", {}).get("disposition"), "bootstrap_contract")

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

# C8 — ⭐ THE ADMISSION RULE, PINNED AGAINST A DIFFERENT TRACKED SOURCE. C8 has now been rewritten
#      twice, and both rewrites were forced by the roster derivation changing under it:
#        - the original ("at least one row claims `Done`") was legitimately moved by DONE-BAR.2b,
#          whose demotion took the tracker to ZERO `Done` rows;
#        - its successor ("register families all derived from the tracker") became VACUOUS the
#          moment LIVE-MEANS-LIVE.1b made the roster register-derived — `families` is now BUILT
#          from `register`, so `register - families` is empty by construction and could never fire.
#          A control that cannot fail is the vacuous floor LANG-CAPABILITY-AUDIT.10.9 had to
#          repair; it is replaced rather than reworded.
#      What actually needs guarding is the ADMISSION RULE — which grammars are families — because
#      that is now a hand-maintained adjudication. So pin it against rust/src/parser_registry.rs,
#      an INDEPENDENT tracked source the register cannot edit: a grammar is a family iff PGEN ships
#      a registered generated parser for it, the sole exception being the two bootstrap contracts,
#      which are registered only to break the annotation-parser cycle and are EXCLUDED BY NAME from
#      parse_harness_equivalence_gate. Adding a disposition for a grammar that really does ship a
#      parser — the convenient way to silence the new refusal — fires this control.
REGISTRY_SRC = os.path.join(ROOT, "rust", "src", "parser_registry.rs")
registered = set()
if os.path.isfile(REGISTRY_SRC):
    registered = set(re.findall(r'grammar_name:\s*"([a-z0-9_]+)"', read(REGISTRY_SRC))) & GRAMMARS
control("C8 registry is readable at all", bool(registered), True)

BOOTSTRAP = {"builtin_return_annotation", "builtin_semantic_annotation"}
# Arm A — a grammar shipping a registered parser is a family, unless it is a bootstrap contract.
control("C8a every registered parser is a family (bootstrap contracts excepted)",
        sorted(g for g in registered - BOOTSTRAP if g not in families), [])
# Arm B — and the converse, or arm A would pass a register that calls everything a family.
control("C8b no family lacks a registered generated parser",
        sorted(g for g in families if g not in registered), [])

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
#       ⚠️ The log path MUST come from TARGET_DIR, like every other artifact lookup. Hardcoding
#       ROOT/rust/target here made C10 read the REAL tree while find_artifact/gate_ran_and_failed
#       read the seam, so any probe pointing at a synthetic target dir fired C10 for every OTHER
#       status gate — a control failing on a tree it was not looking at. Caught by running the probe
#       driver (`.1b`), not by review.
for _g in sorted(status_gate_families):
    if find_artifact(_g) is None and gate_ran_and_failed(_g)[1] is None:
        _log = os.path.join(TARGET_DIR, "sota_exit_gate", "logs", f"{_g}.log")
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
    CI-PARITY-GATE-ROT.5, where a three-day-old hand-run artifact was consumed as current proof.

    The REGISTER is an input because the status gates assert alignment against its `claimed_status`
    (LIVE-MEANS-LIVE.1a), so an artifact older than the register asserted alignment with a claim
    that may since have changed. It replaces LIVE_ACHIEVEMENT_STATUS.md here for exactly that
    reason — the claim moved, so the staleness input moved with it."""
    candidates = [REGISTER]
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
        log_rel, err, log_mtime = gate_ran_and_failed(gate)
        if err:
            # Same rule as the artifact path below: a failure recorded before its own inputs moved
            # describes a tree that no longer exists, so it cannot stand as a CURRENT verdict.
            if log_mtime < newest_input_mtime(family):
                report.append(f"{gate} RAN AND FAILED (log {log_rel}, mtime {int(log_mtime)}, ⚠️ STALE — older than its own inputs)")
                report.append(f"  the recorded failure was: {err}")
                report.append("that failure PREDATES its own inputs ⇒ UNPROVEN, not UNMET — re-run the gate")
                return UNPROVEN
            report.append(f"{gate} RAN AND FAILED (log {log_rel}, mtime {int(log_mtime)}); no summary was written")
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
        log_rel, err, log_mtime = gate_ran_and_failed(gate)
        if err and log_mtime < newest_input_mtime(family):
            # The status gates ASSERT tracker alignment, so a tracker edit after the run leaves that
            # assertion unproven — which is exactly as true of a recorded FAILURE as of a recorded
            # pass. Reporting a superseded mismatch as current is how this audit came to print
            # `tracker: In Progress` above a ⛔ complaining the tracker says `Done` (DONE-BAR.1a).
            report.append(f"  ⚠️ {gate} RAN AND FAILED (log {log_rel}, mtime {int(log_mtime)}) but that run PREDATES its own inputs")
            report.append(f"     the recorded failure was: {err}")
            report.append("     ⇒ green-NOW is UNPROVEN, not UNMET — the failure may already be fixed; re-run the gate")
            verdict = UNPROVEN
        elif err:
            report.append(f"  ⛔ {gate} RAN AND FAILED (log {log_rel}, mtime {int(log_mtime)}): {err}")
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
print(f"families derived from grammars/*.ebnf x the DONE-BAR register: {len(families)} "
      f"({len(GRAMMARS)} tracked grammars = {len(families)} families + {len(dispositions)} recorded non-families)")
print(f"gate universe (via scripts/check_gate_reachability.sh): {len(rows)} targets")
print(f"ground-truth controls reproduced: {CONTROL_COUNT}")
if leaf5_open:
    print("DONE-BAR.5 (consumer-facing disclosure gates) is OPEN ⇒ the highest attainable tier is")
    print("  `Provisional`; no row may be promoted to `Done` on legs 1-3 alone.")
print()

json_rows = []
for name in sorted(families, key=lambda n: (families[n]["status"] != "Done", n)):
    info = families[name]
    judged = info["status"] == "Done"
    print("-" * 78)
    print(f"{name}   claimed: {info['status']}" + ("" if judged else "   (not a `Done` claim — reported for context)"))
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
        "family": name, "claimed_status": info["status"], "judged": judged,
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
