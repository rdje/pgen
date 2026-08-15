#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_make_freshness_probes.sh
#
# CI-PARITY-GATE-ROT.32 (d) — RED/GREEN/CONTROL arms for `scripts/make_freshness_guard.sh`, run
# against the REPOSITORY'S OWN RULES, on a family that is not the one the acute fix already repaired.
#
# ⭐⭐ THE ORACLE IS `make -q`, WHICH IS TO SAY: MAKE ITSELF. Every arm asks make what IT believes
# about a target (`make -q` exits 0 for "up to date", 1 for "would rebuild") and never asks this
# guard's own model of make. That matters more than usual here — the whole defect is that make's
# belief is wrong, so a probe built on a re-implementation of make's comparison would agree with the
# bug. `-q` executes nothing, so reading the belief costs nothing and changes nothing.
#
# ⭐ EVERY ARM PINS MTIMES EXPLICITLY. The defect is a timestamp RELATIONSHIP, so a probe that had to
# race a fast machine to reproduce it would report green on a slow day and prove nothing on a fast
# one. `.32`'s acute proof used the same technique.
#
# ⛔ THE WORKING TREE IS RESTORED BY AN EXIT TRAP: the grammar's mtime, both artifacts' bytes, and
# both artifacts' mtimes. The grammar's CONTENT is never touched, and the git index never is.
#
# Usage: bash docs/tasks/artifacts/ci_parity_gate_rot/run_make_freshness_probes.sh
# Exit 0 iff every arm reaches its expected verdict.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

exec python3 - "$@" <<'PYEOF'
import hashlib, os, shutil, subprocess, sys

ROOT = os.getcwd()
FAM = "json"                       # cheapest family: 0.014 s frontend, 0.139 s generator
GRAMMAR = "grammars/json.ebnf"
JSON = "generated/json.json"
PARSER = "generated/json_parser.rs"
FRONTEND = "rust/target/ebnf_frontend_build/debug/ast_pipeline"
GENERATOR = "rust/target/debug/ast_pipeline"
GUARD = "scripts/make_freshness_guard.sh"
SAVE = "rust/target/ci_parity_gate_rot_probe/freshness_arms"

os.makedirs(SAVE, exist_ok=True)
orig = {}
for p in (GRAMMAR, JSON, PARSER):
    if os.path.exists(p):
        shutil.copy2(p, os.path.join(SAVE, os.path.basename(p)))
        orig[p] = os.stat(p).st_mtime_ns


def restore():
    for p, ns in orig.items():
        src = os.path.join(SAVE, os.path.basename(p))
        if os.path.exists(src):
            shutil.copy2(src, p)
            os.utime(p, ns=(ns, ns))


def pin(path, ns):
    os.utime(path, ns=(ns, ns))


def make_q(target):
    """make's OWN verdict: 0 = it believes the target is up to date, 1 = it would rebuild."""
    return subprocess.run(["make", "-C", "rust", "SHELL=/bin/bash", "-q", "../" + target],
                          cwd=ROOT, capture_output=True, text=True).returncode


def guard(*extra):
    """Run the guard exactly as rust/Makefile does — from `rust/`, with caller-relative paths."""
    cmd = ["bash", "../" + GUARD, "--chain", "../" + GRAMMAR, "../" + JSON, "../" + PARSER,
           "--tools", "./" + FRONTEND[len("rust/"):], "./" + GENERATOR[len("rust/"):], *extra]
    r = subprocess.run(cmd, cwd=os.path.join(ROOT, "rust"), capture_output=True, text=True)
    return r.returncode, r.stdout + r.stderr


def focus():
    r = subprocess.run(["make", "-C", "rust", "SHELL=/bin/bash", "focus_" + FAM],
                       cwd=ROOT, capture_output=True, text=True)
    out = r.stdout + r.stderr
    return (r.returncode,
            out.count("Generating json JSON"),            # frontend step ran N times
            out.count("Generating final json parser"))    # generator step ran N times


def sha(path):
    try:
        with open(path, "rb") as f:
            return hashlib.sha256(f.read()).hexdigest()[:16]
    except OSError:
        return "<absent>"


passed = failed = 0


def arm(label, ok, detail=""):
    global passed, failed
    if ok:
        passed += 1
        print(f"  ✓ {label}" + (f"   [{detail}]" if detail else ""))
    else:
        failed += 1
        print(f"  ✗ {label}   [{detail}]")


print("=" * 96)
print("CI-PARITY-GATE-ROT.32 (d) — make-freshness-guard probes (oracle: `make -q`, i.e. make itself)")
print("=" * 96)
print(subprocess.run(["make", "--version"], capture_output=True, text=True).stdout.splitlines()[0])
print(f"family under test: {FAM}   (the acute `.32` fix covered `scratch` ONLY — this is one of the")
print( "                    nine it did not reach)")
print("-" * 96)

parser_sha_before = sha(PARSER)

# ⛔ THE BASELINE SECOND MUST CLEAR THE TOOL BINARIES, NOT JUST THE JSON — and the probe's first cut
# did not, so six arms failed with `make -q` answering 1 for a reason that had nothing to do with the
# arm. `$(X_JSON)` also depends on `$(RUST_EBNF_FRONTEND_BIN)` and `$(X_PARSER)` on
# `$(RUST_AST_PIPELINE)`; on this tree both binaries were rebuilt 17 h AFTER the last regeneration,
# so make wanted to remake everything regardless of the grammar. Anchor on the whole chain's maximum.
S = (max(os.stat(p).st_mtime_ns for p in (JSON, PARSER, FRONTEND, GENERATOR)) // 10**9) * 10**9 \
    + 10 * 10**9

try:
    # ---------------------------------------------------------------- GREEN baseline
    pin(GRAMMAR, S - 5 * 10**9); pin(JSON, S); pin(PARSER, S + 500_000_000)
    arm("GREEN-0  artifacts newer than the grammar → make says up to date",
        make_q(JSON) == 0 and make_q(PARSER) == 0,
        f"json rc={make_q(JSON)} parser rc={make_q(PARSER)}")
    rc, out = guard()
    arm("GREEN-1  guard does NOTHING on an up-to-date tree", rc == 0 and out.strip() == "",
        f"rc={rc} output={len(out)}B")
    rc, front, gen = focus()
    arm("GREEN-2  `make focus_json` is still a no-op (zero cost preserved)",
        rc == 0 and front == 0 and gen == 0, f"rc={rc} frontend×{front} generator×{gen}")

    # ---------------------------------------------------------------- EDGE A — the verbatim defect
    pin(JSON, S + 100_000_000); pin(PARSER, S + 100_000_000); pin(GRAMMAR, S + 900_000_000)
    arm("RED-A1   ⛔ grammar 0.8 s NEWER, same second → MAKE BELIEVES THE JSON IS UP TO DATE",
        make_q(JSON) == 0, f"make -q rc={make_q(JSON)} (0 = the defect: it will skip the rule)")
    rc, out = guard("--dry-run")
    arm("RED-A2   guard --dry-run reports it WOULD force", rc == 3 and "WRONGLY skip" in out,
        f"rc={rc}")
    rc, out = guard()
    arm("RED-A3   guard removes the json AND cascades to the parser",
        rc == 0 and not os.path.exists(JSON) and not os.path.exists(PARSER), f"rc={rc}")
    arm("RED-A4   make now agrees the json must be rebuilt", make_q(JSON) == 1,
        f"make -q rc={make_q(JSON)}")
    rc, front, gen = focus()
    arm("RED-A5   `make focus_json` regenerates BOTH steps exactly once",
        rc == 0 and front == 1 and gen == 1, f"rc={rc} frontend×{front} generator×{gen}")
    arm("RED-A6   the regenerated parser is byte-identical to the one we started with",
        sha(PARSER) == parser_sha_before, f"{sha(PARSER)} vs {parser_sha_before}")

    # ---------------------------------------------------------------- EDGE B — 10 of 10 exposed
    pin(GRAMMAR, S - 5 * 10**9); pin(PARSER, S + 100_000_000); pin(JSON, S + 900_000_000)
    arm("RED-B1   ⛔ json 0.8 s NEWER, same second → MAKE BELIEVES THE PARSER IS UP TO DATE",
        make_q(PARSER) == 0, f"make -q rc={make_q(PARSER)} (0 = fresh json, STALE parser)")
    rc, out = guard()
    arm("RED-B2   guard removes the parser and LEAVES the json (edge B is not edge A)",
        rc == 0 and os.path.exists(JSON) and not os.path.exists(PARSER), f"rc={rc}")
    arm("RED-B3   make now agrees the parser must be rebuilt", make_q(PARSER) == 1,
        f"make -q rc={make_q(PARSER)}")
    rc, front, gen = focus()
    arm("RED-B4   only the GENERATOR re-runs — the fresh json is not thrown away",
        rc == 0 and front == 0 and gen == 1, f"rc={rc} frontend×{front} generator×{gen}")

    # ---------------------------------------------------------------- CONTROLS: it must not over-fire
    # ⛔ WITHOUT THESE, "always force" would pass every arm above — and always-force is exactly the
    # option this fix rejected, because three tracked gates call `focus_*` merely to ensure an
    # artifact exists.
    pin(JSON, S + 100_000_000); pin(PARSER, S + 200_000_000); pin(GRAMMAR, S + 1_900_000_000)
    arm("CTRL-1   grammar newer by a WHOLE second → make already sees it",
        make_q(JSON) == 1, f"make -q rc={make_q(JSON)}")
    rc, out = guard()
    arm("CTRL-1b  …so the guard stays out of the way (make is right; let it work)",
        rc == 0 and out.strip() == "" and os.path.exists(JSON), f"rc={rc} output={len(out)}B")

    pin(GRAMMAR, S + 100_000_000); pin(JSON, S + 900_000_000); pin(PARSER, S + 950_000_000)
    arm("CTRL-2   same second but the TARGET is newer → make is RIGHT to skip",
        make_q(JSON) == 0, f"make -q rc={make_q(JSON)}")
    rc, out = guard()
    arm("CTRL-2b  …guard does not fire (the exact clause that makes it exact)",
        rc == 0 and out.strip() == "" and os.path.exists(JSON), f"rc={rc} output={len(out)}B")

    # ---------------------------------------------------------------- the guard's own blind spot
    rc = subprocess.run(["bash", "../" + GUARD, "--chain", "grammars/json.ebnf", JSON, PARSER,
                         "--tools", FRONTEND, GENERATOR],
                        cwd=os.path.join(ROOT, "rust"), capture_output=True, text=True).returncode
    arm("CTRL-3   a wrong-cwd invocation REFUSES instead of silently inspecting nothing", rc == 2,
        f"rc={rc} (this is the defect `make -n` caught in the guard's first cut)")
finally:
    restore()

print("-" * 96)
print(f"arms={passed + failed}  PASS={passed}  FAIL={failed}")
print(f"tree restored: {GRAMMAR} mtime + {JSON}/{PARSER} bytes and mtimes")
sys.exit(0 if failed == 0 else 1)
PYEOF
