#!/usr/bin/env bash
# scripts/check_gate_reachability.sh
#
# DOCTRINE `GATE-REACHABILITY` (structural) — CI-PARITY-GATE-ROT.2.
#
#   A check that nothing INVOKES is indistinguishable from a check that does not exist.
#
# ⭐ WHY THIS IS A DOCTRINE AND NOT A NOTE. This repository found three maintained checks that
# nothing ran, one per session, each BY ACCIDENT:
#   1. `ast_dump_contract_gate` — RED for four sessions, "referenced by no aggregate and no CI
#      workflow" (GENERATED-LINT-CORRECTNESS.1);
#   2. the generated-clippy strict stage — `PGEN_CLIPPY_GENERATED_STRICT` set by nothing at all, so
#      a 291 -> 0 correctness win was unguarded from the moment it landed
#      (GENERATED-LINT-CORRECTNESS.3);
#   3. `ci_workflow_local_gate` itself — the designated stand-in for the paused hosted workflows,
#      unable to complete for 1,371 commits (CI-PARITY-GATE-ROT).
# **Finding them by accident is the defect.** Priced against the whole corpus before adopting, as
# `GENERATED-LINT-CORRECTNESS.4` requires: 123 targets, 30 orphans + 1 policy-only at adoption.
#
# ⛔ THIS IS A RATCHET, NOT A REPORT — the distinction `DOCTRINE-GAP-OWNERSHIP.1` had to learn. A
# one-shot census is the same disease one level up: it gets written down and then nothing re-applies
# it. So the orphan set is re-derived every run and joined against a TRACKED register; anything
# UNTRIAGED fails, and a register entry that no longer names an orphan fails too, so the exemption
# list cannot accumulate dead weight.
#
# ⛔ EVERYTHING IS DERIVED FROM THE REPOSITORY, NEVER HAND-LISTED. The universe, the edges and the
# roots are re-read from `rust/Makefile`, `rust/scripts/*.sh`, `.github/workflows/*.yml`,
# `.githooks/*`, `scripts/check_*.sh` and `COMMIT.md` on every run. Only the DISPOSITIONS are
# written down, because a disposition is a human decision and nothing else can derive it.
#
# Usage:
#   bash scripts/check_gate_reachability.sh            # gate mode (exit nonzero on an untriaged orphan)
#   bash scripts/check_gate_reachability.sh --report   # the full human-readable inventory
#   bash scripts/check_gate_reachability.sh --json OUT # also emit the machine-readable rows
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"

exec python3 - "$@" <<'PYEOF'
import json, os, re, sys, glob

ROOT = os.getcwd()

def read(p):
    try:
        with open(os.path.join(ROOT, p), encoding="utf-8", errors="replace") as f:
            return f.read()
    except OSError:
        return ""

def strip_shell_comments(text):
    """Drop whole-line comments. Deliberately conservative: a trailing comment on a real command
    line is left alone rather than risk cutting a quoted '#'. The failure mode we are guarding
    against is a DOCUMENTATION line being counted as an invocation, and those are whole-line."""
    return "\n".join(l for l in text.splitlines() if not l.lstrip().startswith("#"))

# ⚠️⚠️ THE DEFECT THIS INSTRUMENT HAD IN ITS OWN FIRST CUT — A MENTION IS NOT AN INVOCATION.
# The first version treated any occurrence of `rust/scripts/x.sh` or `make -C rust x` as an edge.
# That reported `ast_dump_contract_gate` and `clippy_on_rust_change` as REACHABLE, contradicting two
# facts this project had already measured the hard way: `ast_dump_contract_gate` sat RED for four
# sessions precisely because *nothing runs it*, and `clippy_on_rust_change` belongs to no aggregate
# and no CI workflow. The bogus edges came from the local parity gate's SURFACE AUDITS —
#   assert_tracked "rust/scripts/clippy_on_rust_change.sh"
#   assert_file_contains "rust/Makefile" 'cd $(RUST_DIR) && ./scripts/ast_dump_contract_gate.sh'
# — which READ those files and do not run them. An audit that mentions a gate is the opposite of an
# invoker; counting it as one would have certified the exact rot this inventory exists to find.
# ⛔ Same class as `.1`'s Makefile COMMENT and `DOCTRINE-GAP-OWNERSHIP.1`'s grammar terminals: an
# instrument that measures its own documentation is measuring itself.
#
# The fix is a COMMAND-POSITION rule. A reference counts only when the thing is being executed:
# it is the command word of its segment, or the segment's command word is a known runner. Segments
# are split on shell operators AFTER single-quoted literals are neutralised — assertion needles are
# single-quoted here, and it is exactly those that carry `&&`-joined command text as data.
RUNNERS = {
    "env", "bash", "sh", "source", ".", "exec", "eval", "time", "xargs",
    "run_logged", "run_logged_with_env_file", "run_check", "run_stage", "run_workflow",
}
# ⚠️ SECOND CALIBRATION DEFECT, ALSO CAUGHT BY THE GROUND-TRUTH CONTROLS. Tightening to
# command-position swung the instrument the other way: `mdbook_docs_gate` and
# `branch_protection_contract_gate` came out ORPHAN even though a tracked workflow runs each of
# them. Cause: in a workflow YAML the line is `run: make -C rust ... <target>`, so the command word
# was `run:`; in COMMIT.md it is a markdown bullet with backticks. Both are step SYNTAX around a
# real invocation, not evidence against one. LEADERS therefore also skips the host syntax, and
# backticks are stripped from tokens.
# ⭐ Neither swing would have been visible without the two controls at the bottom of this file:
# `ast_dump_contract_gate` (measured: nothing runs it) and `mdbook_docs_gate` (measured: a workflow
# does). A reachability instrument with no ground truth is a confident number nobody can check.
LEADERS = {"if", "then", "else", "elif", "do", "done", "!", "(", "{", "&&", "||", "|", ";",
           "run:", "-", "*", "+", "shell:", "$"}
SQ = re.compile(r"'[^']*'")
VAR_ASSIGN = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*)=.*?scripts/([a-z0-9_]+)\.sh", re.M)

def command_segments(text):
    """Yield (command_word, segment) for each executable segment of a shell/make body."""
    # ⚠️ FOURTH calibration defect, again caught by a ground-truth control. A backslash-continued
    # call is ONE command, but scanning line-by-line made every continuation line look like its own
    # command — so the second line of
    #     assert_file_contains \
    #       "rust/scripts/clippy_on_rust_change.sh" \
    #       '...'
    # had the SCRIPT PATH as its command word and was counted as an execution. That reported
    # `clippy_on_rust_change` reachable from an aggregate, contradicting the measured fact that it
    # belongs to no aggregate and no CI workflow. Join continuations first, as the shell does.
    text = re.sub(r"\\\n\s*", " ", text)
    for line in text.splitlines():
        line = SQ.sub("''", line)            # neutralise single-quoted literals (assertion needles)
        for seg in re.split(r"&&|\|\||[|;]", line):
            toks = [t.strip("`") for t in seg.strip().split()]
            # ⚠️ THIRD calibration defect: make recipe prefixes. `@$(MAKE) -C $(RUST_DIR) x` is an
            # invocation, but the command word is `@$(MAKE)`. `annotation_contract_gate` composes
            # twenty sub-gates exactly that way, so missing this reported every one of them ORPHAN.
            if toks and toks[0][:1] in "@+":
                toks[0] = toks[0].lstrip("@+")
            while toks and (toks[0] in LEADERS or re.match(r"^[A-Za-z_][A-Za-z0-9_]*=", toks[0])):
                toks.pop(0)
            if toks:
                yield toks[0], toks, seg

MAKEFILE = "rust/Makefile"
mk_raw = read(MAKEFILE)

# ---------------------------------------------------------------- the Makefile target table
# A target block is `name: prereqs` at column 0 followed by TAB-indented recipe lines.
targets = {}          # name -> {"prereqs": [...], "recipe": "..."}
cur = None
for line in mk_raw.splitlines():
    m = re.match(r"^([A-Za-z0-9_.-]+):(?!=)\s*(.*)$", line)
    if m and not line.startswith("\t"):
        cur = m.group(1)
        targets.setdefault(cur, {"prereqs": m.group(2).split(), "recipe": []})
        continue
    if cur is not None and (line.startswith("\t") or line.strip() == ""):
        if line.startswith("\t"):
            targets[cur]["recipe"].append(line)
        continue
    cur = None
for t in targets:
    targets[t]["recipe"] = strip_shell_comments("\n".join(targets[t]["recipe"]))

# ---------------------------------------------------------------- the universe under inventory
# Derived, two ways in, union reported: a `*_gate` target, or any target whose recipe runs a
# rust/scripts/*.sh proof script. The second arm is what catches lanes like `clippy_on_rust_change`
# that are gates in everything but name — a name-only filter would have silently excluded them.
SCRIPT_RE = re.compile(r"(?:\./|\$\(RUST_DIR\)/|rust/)scripts/([a-z0-9_]+)\.sh")
universe, owns_script = {}, {}
for name, blk in targets.items():
    scripts = set(SCRIPT_RE.findall(blk["recipe"]))
    is_gate = name.endswith("_gate")
    if is_gate or scripts:
        universe[name] = "gate-named" if is_gate else "script-backed"
        owns_script[name] = scripts

script_owner = {}     # script basename -> set(targets whose recipe runs it)
for t, ss in owns_script.items():
    for s in ss:
        script_owner.setdefault(s, set()).add(t)

# ---------------------------------------------------------------- edges: who invokes whom
edges = {}            # target -> set(targets it invokes)

def add_edge(a, b):
    if b in universe and a != b:
        edges.setdefault(a, set()).add(b)

def invoked_targets(text):
    """Make targets INVOKED by this body (command position only), plus scripts it EXECUTES."""
    var_scripts = dict((m.group(1), m.group(2)) for m in VAR_ASSIGN.finditer(text))
    made, ran = set(), set()
    for cmd, toks, seg in command_segments(text):
        is_make = cmd in ("make", "$(MAKE)") or cmd.endswith("/make")
        runner = cmd in RUNNERS or bool(SCRIPT_RE.search(cmd))
        if not (is_make or runner):
            continue
        # ⚠️ FIFTH calibration defect — a RUNNER'S ARGV CAN CARRY A NESTED COMMAND, and this is how
        # the flagship aggregate actually calls its sub-gates:
        #     run_check \
        #         "fixed_point_gate" \
        #         "required" \
        #         "deterministic bootstrap artifacts" \
        #         make -C rust SHELL=/bin/bash fixed_point_gate
        # Once continuations are joined (correctly), the command word is `run_check`, not `make`.
        # Looking only at the command word found ZERO sub-gates for `sota_exit_gate` and reported
        # its whole required set orphaned. Scan the argv of a make-or-runner segment for a nested
        # `make`; do NOT scan other command words, or an assertion needle becomes an invocation.
        for i, tok in enumerate(toks):
            base = tok.strip('"').strip("`")
            if not (base in ("make", "$(MAKE)") or base.endswith("/make")):
                continue
            rest = list(toks[i + 1:])
            while rest:
                t = rest.pop(0).strip('"').strip("`")
                if t == "-C":
                    if rest:
                        rest.pop(0)
                    continue
                if t.startswith("-") or "=" in t or t in ("rust", "."):
                    continue
                made.add(t)
                break
        # a script is EXECUTED when it is the command word, or the command word is a runner
        for t in toks:
            t = t.strip('"')
            m = SCRIPT_RE.search(t)
            if m:
                ran.add(m.group(1))
            v = re.fullmatch(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?", t)
            if v and v.group(1) in var_scripts:
                ran.add(var_scripts[v.group(1)])
    return made, ran

# ⭐ ASK MAKE, DO NOT RE-IMPLEMENT MAKE. A prerequisite list can be a variable
# (`parser_books_gate: $(PARSER_BOOK_GATES)`), and reconstructing `$(wildcard)`/`$(patsubst)` here
# would be a second implementation of make's semantics that can drift from the real one. The
# Makefile exposes a read-only `print-%` target, so the value comes from make itself.
_var_cache = {}
def expand_make_var(token):
    m = re.fullmatch(r"\$[({]([A-Za-z_][A-Za-z0-9_]*)[)}]", token)
    if not m:
        return None
    name = m.group(1)
    if name not in _var_cache:
        # ⛔ THIS MUST NOT DEGRADE SILENTLY (`DONE-BAR.1b`). It used to be
        #     _var_cache[name] = out.stdout.split() if out.returncode == 0 else []
        #   wrapped in `except Exception: _var_cache[name] = []`, so a timeout, a missing `make`, or a
        #   real make error all produced an EMPTY expansion — which DROPS the prerequisite edges this
        #   function exists to supply, and a dropped edge turns a reachable target into an "orphan".
        #   That is calibration defect (6) from `CI-PARITY-GATE-ROT.2`'s six-wrong-answers list — the
        #   one fixed by "ASK MAKE, DO NOT RE-IMPLEMENT MAKE" — returning through the FAILURE PATH of
        #   its own fix. Today it would surface as a loud refusal only by luck: if the lost edges
        #   happened to touch already-dispositioned orphans, the census would come out quietly
        #   different and still exit 0, guarded by nothing but whichever ground-truth control noticed.
        # ⭐ MEASURED before changing the behaviour: `make -C rust print-<not-a-variable>` exits **0**
        #   with EMPTY stdout, so the legitimate "this token is not a make variable" case never raises
        #   and never returns nonzero. ⇒ an exception or a nonzero exit can ONLY mean a real failure,
        #   and there is no legitimate empty expansion to preserve. Refusing costs no false positives.
        try:
            import subprocess
            out = subprocess.run(["make", "-C", "rust", "--no-print-directory", "-s",
                                  f"print-{name}"], cwd=ROOT, capture_output=True, text=True,
                                 timeout=30)
        except Exception as exc:
            print(f"gate-reachability: cannot expand make variable '{name}' "
                  f"(`make -C rust print-{name}` failed: {exc!r}). The edge derivation would silently "
                  f"lose this prerequisite list and report reachable targets as orphans, so no "
                  f"inventory is offered. A check that cannot see must say so.", file=sys.stderr)
            sys.exit(2)
        if out.returncode != 0:
            print(f"gate-reachability: cannot expand make variable '{name}' "
                  f"(`make -C rust print-{name}` exited {out.returncode}: "
                  f"{out.stderr.strip()[:200]!r}). Refusing rather than deriving edges from an empty "
                  f"expansion — a non-variable exits 0 with empty output, so a nonzero exit is a real "
                  f"failure.", file=sys.stderr)
            sys.exit(2)
        _var_cache[name] = out.stdout.split()
    return _var_cache[name]

for name, blk in targets.items():
    if name not in universe:
        continue
    made, _ = invoked_targets(blk["recipe"])
    for callee in made:
        add_edge(name, callee)
    for pre in blk["prereqs"]:
        expanded = expand_make_var(pre)
        for p2 in (expanded if expanded is not None else [pre]):
            add_edge(name, p2)

# A gate's script is part of that gate: what the script EXECUTES, the gate executes.
for script, owners in script_owner.items():
    body = strip_shell_comments(read(f"rust/scripts/{script}.sh"))
    if not body:
        continue
    made, ran = invoked_targets(body)
    for owner in owners:
        for callee in made:
            add_edge(owner, callee)
        for sub in ran:
            for sub_owner in script_owner.get(sub, ()):
                add_edge(owner, sub_owner)

# ---------------------------------------------------------------- roots, by invoker class
roots = {}            # target -> set(root classes)

def add_root(t, cls):
    if t in universe:
        roots.setdefault(t, set()).add(cls)

# (R1) the two aggregates are roots by definition: they are what an operator runs.
add_root("sota_exit_gate", "aggregate")
add_root("ci_workflow_local_gate", "aggregate")
for tok in re.findall(r"[a-z0-9_]+_gate", read("rust/config/sota_exit_policy.env")):
    add_root(tok, "sota-policy")

# (R2) a tracked hosted CI workflow.
# ⭐⭐ A DISPATCH-ONLY WORKFLOW IS NOT AN AUTOMATIC INVOKER, AND CONFLATING THE TWO OVERSTATED THIS
# INSTRUMENT'S OWN HEADLINE. Hosted Actions have been paused since `af85a5fd` (2026-04-14) to
# conserve account minutes, so **14 of the 15 tracked workflows are `workflow_dispatch`-only** —
# they run when a human decides to, exactly like an aggregate. Only `memory-architecture-gate.yml`
# still carries `push`/`pull_request`. Reporting "reachable from something that RUNS" for a target
# whose sole invoker is a paused workflow is the same category error this doctrine exists to name:
# it can run, but nothing makes it. So the trigger set is READ from each workflow and the class is
# split — `ci-workflow-auto` vs `ci-workflow-manual` — and the report separates what runs
# CONTINUOUSLY from what runs when someone asks.
AUTO_TRIGGERS = ("push", "pull_request", "schedule", "merge_group")

def workflow_trigger_class(rel):
    """Read the `on:` block and decide whether this workflow runs without a human."""
    body, in_on = [], False
    for line in read(rel).splitlines():
        if re.match(r"^on:\s*$", line):
            in_on = True
            continue
        if in_on:
            if line and not line[0].isspace():
                break
            body.append(line)
    keys = re.findall(r"^\s{2,}([a-z_]+):", "\n".join(body), re.M)
    return "ci-workflow-auto" if any(k in AUTO_TRIGGERS for k in keys) else "ci-workflow-manual"

for wf in sorted(glob.glob(os.path.join(ROOT, ".github/workflows/*.yml"))):
    rel = os.path.relpath(wf, ROOT)
    cls = workflow_trigger_class(rel)
    made, _ = invoked_targets(strip_shell_comments(read(rel)))
    for callee in made:
        add_root(callee, cls)

# (R3) the git hooks + the doctrine enforcer — machine-enforced at commit time.
for p in sorted(glob.glob(os.path.join(ROOT, ".githooks/*"))) + \
         sorted(glob.glob(os.path.join(ROOT, "scripts/check_*.sh"))) + ["scripts/check_doctrines.sh"]:
    rel = os.path.relpath(p, ROOT) if os.path.isabs(p) else p
    made, _ = invoked_targets(strip_shell_comments(read(rel)))
    for callee in made:
        add_root(callee, "git-hook")

# (R4) the commit workflow — POLICY, executed by a human/agent reading COMMIT.md.
# ⭐ Tracked as its OWN class, not merged with the machine-enforced ones. A gate whose only
# invoker is a prose instruction is exactly the class this inventory exists to surface: nothing
# fails if it is skipped.
made, _ = invoked_targets(read("COMMIT.md"))
for callee in made:
    add_root(callee, "commit-workflow-policy")

# ---------------------------------------------------------------- transitive closure
reach = {}            # target -> set(root classes, transitively)
for r, classes in roots.items():
    stack = [(r, frozenset(classes))]
    seen = set()
    while stack:
        node, cls = stack.pop()
        if node in seen:
            continue
        seen.add(node)
        reach.setdefault(node, set()).update(cls)
        for nxt in sorted(edges.get(node, ())):
            stack.append((nxt, cls))

# ⭐ THREE HONEST TIERS, NOT TWO.
#   AUTOMATIC — runs without anyone deciding to: the git hooks (every commit) and the one workflow
#               still on push/pull_request. This is the only tier that holds while nobody is looking.
#   OPERATOR  — runs when someone invokes an aggregate or dispatches a paused workflow. Legitimate,
#               and NOT the same thing as automatic.
#   ORPHAN    — nothing invokes it at all.
# The ratchet binds the third tier (plus policy-only). The first two are REPORTED so the difference
# between "covered" and "covered when someone remembers" cannot be read as the same claim.
AUTOMATIC = {"git-hook", "ci-workflow-auto"}
OPERATOR = {"aggregate", "sota-policy", "ci-workflow-manual"}
MACHINE = AUTOMATIC | OPERATOR
rows = []
for name in sorted(universe):
    cls = sorted(reach.get(name, ()))
    machine = bool(set(cls) & MACHINE)
    rows.append({
        "target": name,
        "kind": universe[name],
        "invokers": cls,
        "status": "reachable" if machine else ("policy-only" if cls else "ORPHAN"),
    })

orphans = [r for r in rows if r["status"] == "ORPHAN"]
policy_only = [r for r in rows if r["status"] == "policy-only"]


REGISTER = "rust/test_data/grammar_quality/gate_reachability_register_v0.json"
MODE = "--report" if "--report" in sys.argv else "--check"

# ---------------------------------------------------------------- GROUND-TRUTH CONTROLS
# ⭐⭐ THE MOST IMPORTANT PART OF THIS FILE. A reachability number nobody can check is a confident
# guess, and this instrument produced SIX different confident guesses while being written —
# 97, 71, 75, 93, 53, 40 orphans — each from a real calibration defect (a mention counted as an
# invocation; YAML `run:` / markdown bullet syntax; make's `@` recipe prefix; backslash
# continuations; a nested `make` inside a runner's argv; a prerequisite list held in a make
# variable). Every one was caught by checking the output against facts this project had already
# established the expensive way, not by reading the code. So those facts are now assertions: if the
# instrument cannot reproduce them it REFUSES, because a miscalibrated reachability report is worse
# than none — it would certify the exact rot it exists to find.
status_of = {r["target"]: r["status"] for r in rows}
CONTROLS = [
    ("ast_dump_contract_gate", "ORPHAN",
     "GENERATED-LINT-CORRECTNESS.1 measured it RED for four sessions precisely because no aggregate "
     "and no CI workflow runs it; QUANT-PLUS-ITER.4 repaired the gate and gave it a SURFACE audit, "
     "which READS it and still does not RUN it"),
    ("clippy_on_rust_change", "policy-only",
     "this leaf's charter residual: it belongs to no aggregate and no CI workflow, and is invoked "
     "only by COMMIT.md step 2"),
    ("mdbook_docs_gate", "reachable",
     ".github/workflows/mdbook-docs-gate.yml runs it"),
    ("branch_protection_contract_gate", "reachable",
     ".github/workflows/branch-protection-contract-gate.yml runs it, and the parity gate replays it"),
    ("ci_workflow_local_gate", "reachable",
     "it is one of the two aggregates an operator runs"),
    ("json_parser_book_gate", "reachable",
     "CI-PARITY-GATE-ROT.2 wired the ten per-parser book gates onto mdbook_docs_gate; if this "
     "regresses, the make-variable expansion of PARSER_BOOK_GATES has broken"),
]
control_failures = []
for target, want, why in CONTROLS:
    got = status_of.get(target, "<not in universe>")
    if got != want:
        control_failures.append(f"  {target}: expected {want}, got {got}\n      known because: {why}")
# ⭐ AND THE TRIGGER CLASSIFIER GETS ITS OWN CONTROLS. The auto-vs-manual split is what stops this
# instrument overstating its headline, so a silent regression in it (a stray `on:` parse change, a
# workflow re-indented) must fail rather than quietly re-inflate the AUTOMATIC tier.
for _wf, _want in (("memory-architecture-gate.yml", "ci-workflow-auto"),
                   ("sota-exit-gate.yml", "ci-workflow-manual")):
    _got = workflow_trigger_class(f".github/workflows/{_wf}")
    if _got != _want:
        control_failures.append(
            f"  .github/workflows/{_wf}: trigger class {_got}, expected {_want}\n"
            f"      known because: hosted Actions were paused by af85a5fd (2026-04-14), leaving "
            f"memory-architecture-gate.yml as the only workflow on push/pull_request")

policy_required = re.search(r'PGEN_SOTA_POLICY_REQUIRED_CHECKS="([^"]*)"',
                            read("rust/config/sota_exit_policy.env"))
if policy_required:
    for chk in policy_required.group(1).split():
        if chk in universe and status_of.get(chk) != "reachable":
            control_failures.append(f"  {chk}: SOTA policy marks it REQUIRED but the inventory says "
                                    f"{status_of.get(chk)}")

# ---------------------------------------------------------------- the tracked register
try:
    with open(os.path.join(ROOT, REGISTER), encoding="utf-8") as f:
        register = json.load(f)
except Exception as exc:
    print(f"gate-reachability: cannot read {REGISTER}: {exc}", file=sys.stderr)
    sys.exit(2)
registered = {e["target"]: e for e in register.get("entries", [])}

orphan_names = sorted(r["target"] for r in rows if r["status"] == "ORPHAN")
policy_names = sorted(r["target"] for r in rows if r["status"] == "policy-only")
needs_entry = orphan_names + policy_names
untriaged = [t for t in needs_entry if t not in registered]
stale = [t for t in registered if t not in needs_entry]

if MODE == "--report":
    print("=" * 78)
    print("GATE REACHABILITY INVENTORY (all figures DERIVED this run)")
    print("=" * 78)
    print(f"universe: {len(rows)} targets  "
          f"({sum(1 for r in rows if r['kind']=='gate-named')} *_gate, "
          f"{sum(1 for r in rows if r['kind']=='script-backed')} script-backed)")
    auto = sum(1 for r in rows if set(r["invokers"]) & AUTOMATIC)
    oper = sum(1 for r in rows if r["status"] == "reachable" and not (set(r["invokers"]) & AUTOMATIC))
    print(f"reachable at all: {sum(1 for r in rows if r['status']=='reachable')}")
    print(f"  ├─ AUTOMATIC (git hook / auto-triggered workflow — holds while nobody looks): {auto}")
    print(f"  └─ OPERATOR  (an aggregate, or a workflow_dispatch-only workflow): {oper}")
    if auto == 0:
        print("     ⚠️⚠️ ZERO targets are automatically invoked. Hosted Actions are PAUSED, so 14 of")
        print("        the 15 tracked workflows are workflow_dispatch-only, and the one that still")
        print("        auto-runs (memory-architecture-gate.yml) executes scripts/check_*.sh directly")
        print("        and no `make` target at all. ⇒ the automatic layer covers the ENFORCED")
        print("        DOCTRINES and NONE of these gate targets: every proof lane below runs only")
        print("        when a human asks. Wiring an orphan into an aggregate or a paused workflow")
        print("        moves it from ORPHAN to OPERATOR — it does not make anything run.")
    else:
        print(f"     ⚠️ hosted Actions are PAUSED: most tracked workflows are dispatch-only, so")
        print(f"        'reachable from a CI workflow' does NOT mean 'runs'.")
    print(f"reachable only from COMMIT.md policy (nothing FAILS if skipped): {len(policy_names)}")
    print(f"ORPHANS (no invoker at all): {len(orphan_names)}")
    print("-" * 78)
    for label, names in (("POLICY-ONLY", policy_names), ("ORPHAN", orphan_names)):
        if not names:
            continue
        print(f"{label} — each must carry a disposition in {REGISTER}:")
        for n in names:
            e = registered.get(n)
            d = f"{e['disposition']}: {e['reason']}" if e else "⛔ UNTRIAGED"
            print(f"  {n:<44} {d}")
        print("-" * 78)
    print("REACHABLE (target <- invoker classes):")
    for r in rows:
        if r["status"] == "reachable":
            print(f"  {r['target']:<44} {','.join(r['invokers'])}")
    print("-" * 78)

if control_failures:
    print("gate-reachability: ✗ GROUND-TRUTH CONTROLS FAILED — the inventory is MISCALIBRATED and "
          "its numbers must not be trusted:", file=sys.stderr)
    print("\n".join(control_failures), file=sys.stderr)
    sys.exit(1)

fails = []
if untriaged:
    fails.append(
        "gate-reachability: %d gate target(s) are invoked by NOTHING and carry no disposition:\n%s\n"
        "  A check that nothing invokes is indistinguishable from a check that does not exist — this\n"
        "  repository found three such gates by accident, one per session, before this register\n"
        "  existed. Either wire the target into an aggregate / CI workflow / hook, or record a\n"
        "  deliberate disposition for it in %s." % (
            len(untriaged), "\n".join(f"    - {t}" for t in untriaged), REGISTER))
if stale:
    fails.append(
        "gate-reachability: %d register entr(ies) name a target that is no longer orphaned or no\n"
        "  longer exists — remove them so the register cannot accumulate dead exemptions:\n%s" % (
            len(stale), "\n".join(f"    - {t}" for t in stale)))

if fails:
    print("\n".join(fails), file=sys.stderr)
    sys.exit(1)

print(f"gate-reachability: OK ({len(rows)} targets; "
      f"{sum(1 for r in rows if r['status']=='reachable')} reachable, "
      f"{len(orphan_names)} orphan + {len(policy_names)} policy-only, all dispositioned; "
      f"{len(CONTROLS) + 2} ground-truth controls reproduced)")

if "--json" in sys.argv:
    out = sys.argv[sys.argv.index("--json") + 1]
    with open(out, "w", encoding="utf-8") as f:
        json.dump({"rows": rows}, f, indent=2, sort_keys=True)

PYEOF
