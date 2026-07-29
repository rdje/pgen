#!/usr/bin/env bash
# scripts/check_flow_integrity.sh
#
# DOCTRINE `FLOW-INTEGRITY` (structural) — CI-PARITY-GATE-ROT.8.
#
#   The gate flow must not drift or rot again — so the invariants that hold it together are
#   enforced on EVERY COMMIT, not only when somebody runs the gate that contains them.
#
# ⭐⭐ WHY THIS EXISTS, AND IT IS AN UNCOMFORTABLE MEASUREMENT. The `CI-PARITY-GATE-ROT` campaign
# repaired the flow and mechanized each repair — but MEASURED afterwards, **four of the five new
# invariants lived inside `ci_workflow_local_gate`**, which the reachability inventory classifies as
# OPERATOR tier: it runs when a human asks. The flow was fixed with checks that could themselves
# rot, which is the very failure the campaign existed to end. Only `GATE-REACHABILITY` was in the
# AUTOMATIC tier (the pre-commit driver).
#
# ⇒ This check moves those invariants into the automatic tier. It is deliberately cheap — file reads
# and greps, no cargo, no build, no network — so it can run on every commit without anyone weighing
# whether it is worth it. A check nobody minds running is a check that keeps running.
#
# ⛔ DERIVED, NOT HAND-LISTED. The workflow roster, the hand-off consumer set, and both forbidden
# shapes are re-read from the repository on every run. Exactly TWO inputs are written down —
# `rust/test_data/grammar_quality/flow_integrity_register_v0.json` — because they are human
# DECISIONS nothing can re-derive: which workflows were MEASURED not to need generated parsers, and
# which hand-off consumers do not yet verify what they are handed.
#
# ⛔ ONE SOURCE FOR THE RULES. `ci_workflow_local_gate.sh` reads the exemption set from that same
# register rather than keeping its own copy. Two lists that must agree are two lists that can
# disagree — `CI-PARITY-GATE-ROT.1` found twelve assertions rotted exactly that way.
#
# What it enforces, each traced to an incident:
#   1. regeneration coverage      — 14 of 15 workflows could not build (`.4`)
#   2. timeout floor              — the flagship budgeted 60 min for a 143-min job (`.4`/`.6`)
#   3. one home for the recipe    — it was about to be copy-pasted into ten more files (`.4`)
#   4. PREPARE stays on           — the identical default eroded once before (`GENERATED-LINT-CORRECTNESS.3`)
#   5. no standalone-default hand-offs — consumed a 3-day-old artifact as current proof (`.7`)
#   6. no requires-a-defect assertions — a gate that passed only when the parser FAILED (`.5`)
#   7. hand-off provenance ratchet — coverage may only improve (`.7`)
#   8. the doctrine roster keeps an AUTOMATIC lane, through the DRIVER — the one auto-running
#      workflow named 5 enforcers individually, so 8 of 13 doctrines had no automatic lane and a
#      doctrine added tomorrow silently got none (`.15`)
#
# Usage:
#   bash scripts/check_flow_integrity.sh            # gate mode
#   bash scripts/check_flow_integrity.sh --report   # verbose, shows every invariant's state
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"

exec python3 - "$@" <<'PYEOF'
import json, os, re, subprocess, sys, glob

ROOT = os.getcwd()
REGISTER = "rust/test_data/grammar_quality/flow_integrity_register_v0.json"
REPORT = "--report" in sys.argv
ACTION = ".github/actions/regenerate-parsers"
MAKE_TARGET = "regenerate_generated_parsers"
PARITY_GATE = "rust/scripts/ci_workflow_local_gate.sh"

def read(p):
    try:
        with open(os.path.join(ROOT, p), encoding="utf-8", errors="replace") as f:
            return f.read()
    except OSError:
        return ""

def sh(cmd):
    return subprocess.run(["bash", "-c", cmd], cwd=ROOT, capture_output=True, text=True).stdout

def uncommented(text):
    return "\n".join(l for l in text.splitlines() if not l.lstrip().startswith("#"))

fails, notes = [], []
def bad(msg): fails.append(msg)
def note(msg):
    notes.append(msg)

try:
    reg = json.loads(read(REGISTER))
except Exception as exc:
    print(f"flow-integrity: cannot read {REGISTER}: {exc}", file=sys.stderr)
    sys.exit(2)

exempt = {k: v for k, v in reg["regeneration_exempt_workflows"].items() if not k.startswith("_")}
floor = int(reg["workflow_regeneration_timeout_floor_minutes"])
unverified_registered = set(reg["handoff_provenance_ratchet"]["unverified_consumers"])

# --------------------------------------------------------------- the derived workflow roster
workflows = sorted(sh("git ls-files '.github/workflows/*.yml'").split())
if not workflows:
    print("flow-integrity: the workflow roster derivation returned NOTHING — a check that inspects\n"
          "  nothing must refuse, not pass.", file=sys.stderr)
    sys.exit(1)

# --------------------------------------------------------------- (1)(2) regeneration + timeout
need, exempt_seen = [], []
for wf in workflows:
    body = uncommented(read(wf))
    has_step = ACTION in body
    if wf in exempt:
        exempt_seen.append(wf)
        if has_step:
            bad(f"(1) {wf} is a MEASURED-EXEMPT workflow but declares the regeneration step.\n"
                f"    Reason it is exempt: {exempt[wf]}\n"
                f"    A ~236s regeneration bolted onto a job that needs none is a cost regression;\n"
                f"    exemption is asserted in BOTH directions on purpose.")
        continue
    if "make -C rust" not in body:
        continue                       # runs no make gate at all (e.g. memory-architecture-gate)
    need.append(wf)
    if not has_step:
        bad(f"(1) {wf} runs a 'make -C rust' gate but declares no regeneration step.\n"
            f"    generated/ is untracked, so actions/checkout hands this job a tree with no\n"
            f"    generated parsers and any command compiling the crate dies on\n"
            f"      error: couldn't read `src/../../generated/return_annotation_parser.rs`\n"
            f"    fix:  add   uses: {ACTION}   before the gate step, or MEASURE that it needs\n"
            f"          nothing against a tracked-files-only tree and register the exemption.")
        continue
    m = re.search(r"^\s*timeout-minutes:\s*(\d+)", read(wf), re.M)
    if not m:
        bad(f"(2) {wf} declares the regeneration step but no timeout-minutes; price the job.")
    elif int(m.group(1)) < floor:
        bad(f"(2) {wf} budgets timeout-minutes: {m.group(1)}, below the {floor}-minute floor for a\n"
            f"    job that regenerates generated/ first (the step alone measured 236s locally,\n"
            f"    warm, on a fast volume; a hosted runner starts cold).")

for wf in exempt:
    if wf not in workflows:
        bad(f"(1) the register exempts {wf}, which is not a tracked workflow — remove the entry so\n"
            f"    the exemption list cannot hold dead weight.")

# --------------------------------------------------------------- (3) one home for the recipe
if not os.path.isfile(os.path.join(ROOT, f"{ACTION}/action.yml")):
    bad(f"(3) the composite action {ACTION}/action.yml is missing — the regeneration STEP has no home.")
elif f"make -C rust SHELL=/bin/bash {MAKE_TARGET}" not in read(f"{ACTION}/action.yml"):
    bad(f"(3) {ACTION}/action.yml no longer delegates to the '{MAKE_TARGET}' make target.")
if f"{MAKE_TARGET}:" not in read("rust/Makefile"):
    bad(f"(3) rust/Makefile no longer defines '{MAKE_TARGET}' — the RECIPE has no home.")
for wf in workflows:
    if "regex_parser_bootstrap" in uncommented(read(wf)):
        bad(f"(3) {wf} re-inlines the regeneration recipe (it names regex_parser_bootstrap).\n"
            f"    The sequence has exactly one definition, in rust/Makefile's {MAKE_TARGET};\n"
            f"    a copy in a workflow is a second home whose drift nothing can detect.")

# --------------------------------------------------------------- (4) PREPARE stays on by default
# ⚠️ Compare the declared VALUE, never match the literal: an assertion about a file cannot live
# inside that file as a literal, and this check lives outside the file precisely so it can.
m = re.search(r'^PREPARE_RAW="\$\{PGEN_CI_WORKFLOW_LOCAL_PREPARE:-([a-z]+)\}"', read(PARITY_GATE), re.M)
if not m:
    bad("(4) cannot find the PGEN_CI_WORKFLOW_LOCAL_PREPARE default declaration in the parity gate.")
elif m.group(1) != "true":
    bad(f"(4) PGEN_CI_WORKFLOW_LOCAL_PREPARE defaults to '{m.group(1)}', expected 'true'. Without\n"
        f"    preparation 8 of the 11 replays cannot compile the crate, so the parity gate would\n"
        f"    certify a parity it is not testing.")

# --------------------------------------------------------------- (5) no standalone-default hand-offs
# ⚠️ THIS PATTERN WAS TOO NARROW IN ITS FIRST CUT, AND RED-7 CAUGHT IT. It required the
# `${VAR:-$RUST_DIR/target/…}` form and therefore missed the BARE `="$RUST_DIR/target/…"` form —
# which is FOUR of the eight sites the original incident actually had. A check written for a defect
# that cannot see that defect's commonest shape is worse than none: it reports zero and reads as
# proof. Match any value that mentions a standalone target dir at all.
STANDALONE = r'EXISTING_[A-Z_]*STATE_DIR="[^"]*\$RUST_DIR/target/'
hits = [l for l in sh(f"grep -rnE '{STANDALONE}' rust/scripts/*.sh").splitlines() if l.strip()]
for h in hits:
    bad("(5) an EXISTING-artifact hand-off points at a gate's STANDALONE default state dir:\n"
        f"      {h.strip()}\n"
        "    That directory exists only if somebody once ran that gate BY HAND. On a clean tree the\n"
        "    value is non-empty but absent, so the consumer SKIPS the branch that would produce the\n"
        "    artifact; on a developer machine it exists and is STALE. Measured 2026-07-28: a\n"
        "    sota_exit_gate run consumed a THREE-DAY-OLD artifact as current proof.\n"
        "    fix:  pass an empty value (\"not supplied\") or a directory THIS RUN produced.")

# --------------------------------------------------------------- (6) no requires-a-defect assertions
DEFECT_REQ = "expected at least one"
hits = [l for l in sh(f"grep -rn '{DEFECT_REQ}' rust/scripts/*.sh").splitlines()
        if l.strip() and not re.match(r"^[^:]+:\d+:\s*#", l)]
for h in hits:
    bad("(6) an assertion requires a FAILURE to exist in order to pass:\n"
        f"      {h.strip()}\n"
        "    A gate must not depend on the thing it watches being broken. Measured: this shape made\n"
        "    sv_failure_context_contract_gate passable only when the SV parser REJECTED its own\n"
        "    generated sample.\n"
        "    fix:  require the zero to be EARNED — the surface was exercised, the zero is consistent\n"
        "          with zero rejections, and a present excerpt is well-formed.")

# --------------------------------------------------------------- (7) hand-off provenance ratchet
consumers = {os.path.splitext(os.path.basename(p))[0]
             for p in sh("grep -lE 'EXISTING_[A-Z_]+_STATE_DIR=' rust/scripts/*.sh").split()}
verifying = {os.path.splitext(os.path.basename(p))[0]
             for p in sh("grep -lE 'require_supplied_state_dir' rust/scripts/*.sh").split()}
unverified_now = consumers - verifying
new_unverified = sorted(unverified_now - unverified_registered)
graduated = sorted(unverified_registered - unverified_now)
if new_unverified:
    bad("(7) hand-off consumer(s) that do not verify what they are handed, and are not registered:\n"
        + "\n".join(f"      - {c}" for c in new_unverified) + "\n"
        "    A consumer told 'this artifact exists, do not produce it' must prove that hand-off:\n"
        "    present, non-empty, and not older than the run consuming it. Add\n"
        "    require_supplied_state_dir (see rust/scripts/sv_parser_family_status_gate.sh), or\n"
        f"    register the gap in {REGISTER} — the list may only SHRINK.")
if graduated:
    bad("(7) register entr(ies) that now DO verify — remove them so the ratchet keeps its teeth:\n"
        + "\n".join(f"      - {c}" for c in graduated))

# --------------------------------------------------------------- (8) the doctrine automatic lane
# ⭐ THE ROSTER MUST BE INHERITED, NOT RE-TYPED. `.15` measured that the ONE workflow still running
# on push/pull_request executed five registered enforcers BY NAME instead of the registry driver, so
# 8 of the 13 doctrines had no automatic lane at all — and, the part that matters, a doctrine added
# tomorrow silently got none (`ROUTING-EVIDENCE`, shipped hours earlier, was already in exactly that
# position). Naming enforcers freezes the roster at the moment somebody last edited the YAML.
DRIVER = "scripts/check_doctrines.sh"
reg_src = read(DRIVER)
m_reg = re.search(r"(?ms)^DOCTRINES=\(\n(.*?)^\)", reg_src)
enforcers = ([l.strip().strip('"').split("|")[-1]
              for l in m_reg.group(1).splitlines() if l.strip().startswith('"')] if m_reg else [])
if not enforcers:
    print(f"flow-integrity: could not derive the doctrine roster from {DRIVER} — a check that cannot\n"
          "  read its subject must refuse, not pass.", file=sys.stderr)
    sys.exit(1)

def auto_triggered(wf):
    """True iff the workflow's `on:` block declares a trigger no human has to press."""
    blk = re.search(r"(?ms)^on:\n(.*?)(?=^\S|\Z)", uncommented(read(wf)))
    return bool(blk) and bool(re.search(r"^\s{1,4}(push|pull_request|schedule)\s*:", blk.group(1), re.M))

auto_workflows = [wf for wf in workflows if auto_triggered(wf)]
driver_lane = [wf for wf in auto_workflows if DRIVER in uncommented(read(wf))]
for wf in auto_workflows:
    body = uncommented(read(wf))
    named = sorted({e for e in enforcers if e in body})
    if named:
        bad(f"(8) {wf} runs automatically and invokes registered doctrine enforcer(s) BY NAME:\n"
            + "\n".join(f"      - {n}" for n in named) + "\n"
            f"    That freezes the automatic lane at whatever was typed into the YAML. Measured\n"
            f"    2026-07-29: five were named, so 8 of 13 doctrines had NO automatic lane and every\n"
            f"    doctrine added afterwards inherited none.\n"
            f"    fix:  run `bash {DRIVER}` — the registry is the single source of the roster, so a\n"
            f"          new doctrine gets the lane by construction.")
if not driver_lane:
    bad(f"(8) NO automatically-triggered workflow invokes {DRIVER}, so the enforced doctrines have no\n"
        f"    automatic lane — only the local pre-commit hook, which `--no-verify` bypasses. E4 is\n"
        f"    the un-bypassable backstop by design (DOCTRINE_ENFORCEMENT.md §9).\n"
        f"    fix:  give one cheap workflow `on: push` + `pull_request` and have it run the driver\n"
        f"          (it costs ~2 s: no cargo, no build, no network).")

# --------------------------------------------------------------- report / verdict
if REPORT:
    print("=" * 78)
    print("FLOW INTEGRITY (every figure DERIVED this run)")
    print("=" * 78)
    print(f"tracked workflows                     : {len(workflows)}")
    print(f"  ├─ require the regeneration step    : {len(need)}")
    print(f"  ├─ measured-exempt                  : {len(exempt_seen)}")
    print(f"  └─ run no `make -C rust` gate       : {len(workflows) - len(need) - len(exempt_seen)}")
    print(f"regeneration timeout floor            : {floor} min")
    print(f"standalone-default hand-offs          : 0 required, {len(hits)} found")
    print(f"hand-off consumers                    : {len(consumers)}")
    print(f"  ├─ verify provenance                : {len(verifying & consumers)}")
    print(f"  └─ registered as not yet verifying  : {len(unverified_registered)}")
    print(f"registered doctrines                  : {len(enforcers)}")
    print(f"  ├─ auto-triggered workflows         : {len(auto_workflows)}")
    print(f"  └─ of those, invoking the driver    : {len(driver_lane)}"
          f"  {'(the whole roster has a lane)' if driver_lane else '⛔ NONE'}")
    print("-" * 78)
    if unverified_now:
        print("⚠️  ACCEPTED RISK — these are handed artifacts they do not verify. The ratchet stops")
        print("    the list growing; SHRINKING it is real remaining work, not a closed question:")
        for c in sorted(unverified_now):
            print(f"      - {c}")
        print("-" * 78)

if fails:
    print("flow-integrity: the gate flow has drifted —", file=sys.stderr)
    print("\n".join(fails), file=sys.stderr)
    print("\n  Reference: docs/book/src/gate-flow.md (\"The Gate Flow — Reference\").", file=sys.stderr)
    sys.exit(1)

print(f"flow-integrity: OK ({len(need)} workflow(s) regenerate, {len(exempt_seen)} measured-exempt, "
      f"recipe has one home, PREPARE on, 0 standalone-default hand-offs, 0 requires-a-defect "
      f"assertions, provenance ratchet {len(verifying & consumers)}/{len(consumers)}, "
      f"all {len(enforcers)} doctrines on the automatic lane via the driver)")
PYEOF
