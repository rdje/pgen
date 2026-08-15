#!/usr/bin/env bash
# scripts/make_freshness_guard.sh — repair GNU Make 3.81's WHOLE-SECOND blind spot, at the exact
# moment it is about to make a wrong decision, and never otherwise.
#
# CI-PARITY-GATE-ROT.32 (d).
#
# ⛔⛔ THE DEFECT THIS EXISTS FOR. This host's only `make` is Apple's GNU Make **3.81** (2006), which
# truncates file mtimes to whole seconds; sub-second comparison arrived in make 4.x. It skips a rule
# iff `floor(mtime(target)) >= floor(mtime(prereq))`. So a prerequisite rewritten in the SAME
# WALL-CLOCK SECOND as its target is invisible, and the rule is skipped at exit 0 — silently, and in
# the PASSING direction. Measured on this tree: with `grammars/scratch/scratch.ebnf` written 0.8 s
# AFTER `generated/scratch.json`, the frontend ran **0** times and the regenerated parser still
# declared the PREVIOUS grammar's rules.
#
# ⭐ THE CONDITION HERE IS EXACT, NOT CONSERVATIVE. Make's decision is WRONG iff
#
#     floor(mtime(target)) == floor(mtime(prereq))   AND   mtime(prereq) > mtime(target)
#
# — the second clause is what makes it exact. Same-second-but-target-newer is make being RIGHT (the
# artifact really is up to date), and this guard leaves it alone. So the guard forces work only in
# the window where make would otherwise serve a stale artifact: **zero cost in the common case, and
# no false forcing at all.** That is what makes it safe to put on every family, including the
# 28-second SystemVerilog regeneration.
#
# ⛔ NANOSECOND INTEGERS, NOT FLOAT SECONDS. `st_mtime` as a float is a double: an epoch near 1.79e9
# leaves ~6 significant digits below the decimal point, which is not enough to order two writes a
# few hundred microseconds apart. `st_mtime_ns` is an exact integer and `ns // 10**9` is exactly the
# whole second make itself compares. A guard for a precision defect must not have a precision defect.
#
# ⛔ WHY A GUARD AND NOT THE TWO OBVIOUS ALTERNATIVES — both were priced, neither was adopted:
#   - `rm -f` the artifacts unconditionally in every `focus_*` (the acute `.32` fix, kept for
#     `focus_scratch`). Correct, but it makes every entry point unconditional, and THREE TRACKED
#     GATES call `focus_*` purely to ENSURE an artifact exists — `sv_cert_recognized_union_gate.sh`
#     and `verilog_2005_conformance_gate.sh` (`focus_systemverilog`, measured 28.5 s of generation)
#     and `rtl_const_expr_cert_gate.sh`. Those would pay a full regeneration every run for a hazard
#     window they are not in.
#   - require `make >= 4.0`. Fixes all 65 file rules at once and costs every contributor an install
#     plus a `gmake`-vs-`make` rename through the docs and workflows; GitHub's ubuntu runners ship
#     make 4.x while its macOS runners ship 3.81, so it also buys a CI-parity split. Recorded as
#     PRICED, not adopted (`CI-PARITY-GATE-ROT.32` acceptance (c)).
#   A third — key freshness on a content hash of the emitted json — is IMPOSSIBLE here and the census
#   proved it: `generated/<fam>.json` embeds a wall-clock `"generated_at"`, so two runs of the same
#   frontend on the same grammar never hash the same. Any content-addressed scheme must key on the
#   GRAMMAR.
#
# ⚠️ HONEST BOUND. This closes the window for a SEQUENTIAL driver — one that waits for `make` to
# return before touching the grammar again, which is every script, gate and agent loop in this
# repository. A driver that edits a grammar WHILE a build of it is running is a data race on the
# input and no mtime scheme can fix it.
#
# Usage:
#   scripts/make_freshness_guard.sh --chain GRAMMAR JSON PARSER --tools FRONTEND GENERATOR
#   scripts/make_freshness_guard.sh --chain … --tools … --json-only # edge A only — for the two
#     annotation flows, whose recipe regenerates the parser UNCONDITIONALLY (the generator runs as a
#     recipe line, not as a file rule), so edge B does not exist there and removing their parser
#     would only make `$(RUST_AST_PIPELINE)`'s placeholder logic recreate a stub in its place.
#   scripts/make_freshness_guard.sh --chain … --tools … --dry-run   # verdict only; rc 3 = would force
#   scripts/make_freshness_guard.sh --self-test                     # RED/GREEN/CONTROL arms
#
# Exit: 0 nothing to do, or the removal succeeded · 2 usage · 3 --dry-run and it WOULD force.
set -uo pipefail
# ⛔ THE CALLER'S CWD IS PART OF THE INTERFACE, AND GETTING THIS WRONG MAKES THE GUARD A SILENT NO-OP.
# `make` runs from `rust/` and hands over paths like `../generated/json.json`. Every other script here
# cds to the repository root; doing that here without re-anchoring would resolve those paths OUTSIDE
# the repository, every stat would come back absent, `make_would_wrongly_skip` would answer False for
# every edge, and the guard would exit 0 having inspected nothing. Caught by `make -n` before it
# shipped. The invocation cwd is therefore captured and the chain re-anchored against it, and a
# missing GRAMMAR is a hard refusal (exit 2) precisely so this class cannot come back quietly.
export PGEN_GUARD_CALLER_PWD="$PWD"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"

exec python3 - "$@" <<'PYEOF'
import os, sys

argv = sys.argv[1:]
DRY = "--dry-run" in argv
if DRY:
    argv.remove("--dry-run")
QUIET = "--quiet" in argv
if QUIET:
    argv.remove("--quiet")
JSON_ONLY = "--json-only" in argv
if JSON_ONLY:
    argv.remove("--json-only")


def mtime_ns(path):
    try:
        return os.stat(path).st_mtime_ns
    except OSError:
        return None


def make_would_wrongly_skip(target, prereq):
    """True iff make 3.81 would call `target` up to date while `prereq` is genuinely newer."""
    t, p = mtime_ns(target), mtime_ns(prereq)
    if t is None or p is None:            # make rebuilds (or errors) on its own — nothing to repair
        return False
    return (t // 10**9) == (p // 10**9) and p > t


def fmt(path):
    ns = mtime_ns(path)
    return "<absent>" if ns is None else f"{ns // 10**9}.{ns % 10**9:09d}"


def guard(edges, dry=False, quiet=False):
    """edges: [(target, [prereqs...], [cascade...])] in dependency order.
    Returns the list of files removed (or that WOULD be removed under `dry`)."""
    removed = []
    for target, prereqs, cascade in edges:
        hit = next((p for p in prereqs if make_would_wrongly_skip(target, p)), None)
        if hit is None:
            continue
        victims = [target] + [c for c in cascade if os.path.exists(c)]
        if not quiet:
            print(f"make-freshness-guard: GNU Make 3.81 would WRONGLY skip this rule —\n"
                  f"    target  {target}  mtime {fmt(target)}\n"
                  f"    prereq  {hit}  mtime {fmt(hit)}   (newer, but the SAME whole second)\n"
                  f"    → forcing the rebuild by removing: {' '.join(victims)}")
        removed.extend(victims)
        if not dry:
            for v in victims:
                try:
                    os.remove(v)
                except OSError as exc:
                    print(f"make-freshness-guard: cannot remove {v}: {exc}", file=sys.stderr)
                    sys.exit(1)
    return removed


# ------------------------------------------------------------------------------- self test
def self_test():
    """RED / GREEN / CONTROL arms. ⭐ Every arm pins mtimes EXPLICITLY, so nothing here depends on
    how fast the machine is — the defect is a timestamp relationship, and a probe that had to RACE
    to reproduce it would be a probe that reports green on a slow day."""
    work = os.path.join("rust", "target", "make_freshness_guard_selftest")
    os.makedirs(work, exist_ok=True)
    passed = failed = 0

    def arm(label, want_removed, t_ns, p_ns, target_exists=True, prereq_exists=True):
        nonlocal passed, failed
        tgt, pre, casc = (os.path.join(work, n) for n in ("target", "prereq", "cascade"))
        for f in (tgt, pre, casc):
            if os.path.exists(f):
                os.remove(f)
        if prereq_exists:
            open(pre, "w").write("p"); os.utime(pre, ns=(p_ns, p_ns))
        if target_exists:
            open(tgt, "w").write("t"); os.utime(tgt, ns=(t_ns, t_ns))
            open(casc, "w").write("c"); os.utime(casc, ns=(t_ns, t_ns))
        got = guard([(tgt, [pre], [casc])], quiet=True)
        ok = (len(got) > 0) == want_removed
        if ok and want_removed:                      # the cascade must travel with the target
            ok = casc in got
        if ok:
            passed += 1; print(f"  ✓ {label}")
        else:
            failed += 1
            print(f"  ✗ {label}: expected {'FORCE' if want_removed else 'no action'}, got {got}")

    S = 1786800000 * 10**9                                        # a fixed second boundary
    print("make-freshness-guard self-test (mtimes pinned; no race, no machine dependence)")
    arm("RED-1   same second, prereq 0.5s NEWER  → FORCE", True,  S + 100_000_000, S + 600_000_000)
    arm("RED-2   same second, prereq 1ns NEWER   → FORCE", True,  S, S + 1)
    arm("GREEN-1 prereq 2s OLDER                 → none",  False, S, S - 2 * 10**9)
    arm("CTRL-1  prereq 1s NEWER (make sees it)  → none",  False, S, S + 10**9)
    arm("CTRL-2  same second, TARGET newer       → none",  False, S + 600_000_000, S + 100_000_000)
    arm("CTRL-3  mtimes exactly equal            → none",  False, S, S)
    arm("CTRL-4  target absent                   → none",  False, S, S + 5, target_exists=False)
    arm("CTRL-5  prereq absent                   → none",  False, S, S + 5, prereq_exists=False)
    print(f"  arms={passed + failed}  PASS={passed}  FAIL={failed}")
    return 0 if failed == 0 else 1


if "--self-test" in argv:
    sys.exit(self_test())

# ------------------------------------------------------------------------------- chain mode
if "--chain" not in argv or "--tools" not in argv:
    print(__doc__ or "", file=sys.stderr)
    print("usage: make_freshness_guard.sh --chain GRAMMAR JSON PARSER --tools FRONTEND GENERATOR\n"
          "       make_freshness_guard.sh --self-test", file=sys.stderr)
    sys.exit(2)

ci, ti = argv.index("--chain"), argv.index("--tools")
chain, tools = argv[ci + 1:ti], argv[ti + 1:]
if len(chain) != 3 or len(tools) != 2:
    print(f"make_freshness_guard.sh: --chain takes 3 paths (got {len(chain)}), "
          f"--tools takes 2 (got {len(tools)})", file=sys.stderr)
    sys.exit(2)

CALLER = os.environ.get("PGEN_GUARD_CALLER_PWD", os.getcwd())


def anchor(p):
    """Re-anchor a caller-relative path — see the cwd note in the bash preamble."""
    return p if os.path.isabs(p) else os.path.normpath(os.path.join(CALLER, p))


chain = [anchor(p) for p in chain]
tools = [anchor(p) for p in tools]
grammar, json_path, parser_path = chain
frontend, generator = tools

# ⛔ A GUARD THAT CANNOT SEE ITS SUBJECT MUST REFUSE, NOT PASS. The json and the parser are legitimately
# absent on a cold clone; the GRAMMAR never is. If it is missing, the paths are wrong (or the family
# is), and every edge below would answer "nothing to do" for the wrong reason.
if not os.path.exists(grammar):
    print(f"make-freshness-guard: the grammar does not exist: {grammar}\n"
          f"    (invocation cwd {CALLER}) — refusing rather than inspecting nothing.", file=sys.stderr)
    sys.exit(2)

# ⛔ TWO EDGES, IN DEPENDENCY ORDER, AND THE SECOND ONE IS THE ONE THE CENSUS FOUND EVERYWHERE.
#   EDGE A  json ← grammar        the authored edit; 5 of the 10 families are exposed.
#   EDGE B  parser ← json         the MACHINE-written edge — and the frontend step that rewrites the
#                                 json takes 0.006-0.111 s for EVERY family, SystemVerilog included,
#                                 so all 10 are exposed here. A family can therefore pass edge A and
#                                 still hand you a FRESH json with a STALE parser, which reads worse
#                                 than the original defect: the artifact gates inspect is current
#                                 while the artifact that actually parses is not.
# The tool binaries are prerequisites of these same rules and get the identical treatment for free.
edges = [
    (json_path,   [grammar, frontend],      [] if JSON_ONLY else [parser_path]),
]
if not JSON_ONLY:
    edges.append((parser_path, [json_path, generator], []))
removed = guard(edges, dry=DRY, quiet=QUIET)
sys.exit(3 if (DRY and removed) else 0)
PYEOF
