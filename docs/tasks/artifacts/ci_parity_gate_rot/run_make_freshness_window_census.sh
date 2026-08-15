#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_make_freshness_window_census.sh
#
# CI-PARITY-GATE-ROT.32 (d) — PRICE THE TEN. Which `$(X_JSON): $(X_EBNF) $(FRONTEND)` instances can
# actually be hit by GNU Make 3.81's WHOLE-SECOND timestamp comparison, derived rather than felt.
#
# ⭐ THE EXPOSURE CRITERION IS EXACT, NOT A HEURISTIC — and it is NOT "how long the family takes".
# Make 3.81 skips a rule iff `floor(mtime(target)) >= floor(mtime(prereq))`. In a driver loop
# (edit → build → … → edit → build) the decision is WRONG iff the next prerequisite edit lands in
# the same wall-clock second as the previous build's write of the target. A sequential driver cannot
# issue that edit before `make` returns, so:
#
#     minimum achievable gap  =  the build work that happens AFTER the target is written
#     gap >= 1.000 s          ⇒  floor(prereq) > floor(target) always  ⇒  PROVABLY IMMUNE
#
# ⛔ Two edges per family, not one — the leaf's population table only named the first:
#   EDGE A  grammar → json    target `$(X_JSON)` written, then the grammar is edited.
#           The build's remaining work after the json write is the GENERATOR step, so
#           gap_A >= t_generator. Immune iff t_generator >= 1 s.
#   EDGE B  json → parser     target `$(X_PARSER)` written (essentially at the end of the build),
#           then the NEXT build rewrites the json. That rewrite is the FRONTEND step, so
#           gap_B >= t_frontend. Immune iff t_frontend >= 1 s.
#           ⇒ a family can pass edge A and still serve a STALE PARSER over a fresh json.
#
# ⚠️ HONEST BOUND. Both bounds assume a SEQUENTIAL driver — one that waits for `make` to return
# before touching the grammar again. A driver that edits a grammar while a build of it is running
# defeats any mtime reasoning and is out of scope here (it is a data race on the input, not a make
# defect).
#
# ⛔ THIS CENSUS NEVER TOUCHES `generated/`. Both steps write into `rust/target/`, and each output is
# hash-compared against the shipped artifact — a free determinism CONTROL proving the timings were
# paid on the real inputs and not on a degenerate stand-in.
#
# ⚠️ THE CONTROL'S FIRST CUT REPORTED 10/10 FAILURES, AND IT WAS THE CONTROL THAT WAS WRONG. Both
# artifacts embed WHERE AND WHEN they were produced, so a byte-for-byte comparison across two
# different output paths can never hold:
#   - `generated/<fam>.json` carries `"generated_at": "<wall clock>"` and `"source_file"` as the
#     path GIVEN to the frontend (make runs from `rust/`, so the shipped value is `../grammars/…`);
#   - `generated/<fam>_parser.rs` embeds its own `-o` path as a diagnostic string (5 sites in the
#     json family, more in the larger ones).
# The comparison therefore normalises exactly those three volatile fields and nothing else. ⭐ The
# by-product is worth more than the control: **a `generated/*.json` cannot be byte-compared across
# runs at all**, so any content-addressed freshness scheme in this repository must key on the
# GRAMMAR, never on the emitted json.
#
# Usage:
#   bash docs/tasks/artifacts/ci_parity_gate_rot/run_make_freshness_window_census.sh [--reps N]
# Exit 0 iff every instance was measured and every identity control held.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

exec python3 - "$@" <<'PYEOF'
import hashlib, os, subprocess, sys, time

ROOT = os.getcwd()
REPS = 3
if "--reps" in sys.argv:
    REPS = int(sys.argv[sys.argv.index("--reps") + 1])

WORK = "rust/target/make_freshness_window_census"
FRONTEND = "rust/target/ebnf_frontend_build/debug/ast_pipeline"
GENERATOR = "rust/target/debug/ast_pipeline"
GENERATOR_BOOTSTRAP = "rust/target/debug/ast_pipeline_bootstrap"

# The ten instances of the shape that bit `.32`, in rust/Makefile order. `bootstrap` marks the two
# whose parser rule runs $(RUST_GENERATOR_BOOTSTRAP) rather than $(RUST_GENERATOR).
FAMILIES = [
    # (family, grammar, json, parser, bootstrap, focus target or None)
    ("semantic_annotation", "grammars/semantic_annotation.ebnf", "generated/semantic_annotation.json",
     "generated/semantic_annotation_parser.rs", True, None),
    ("return_annotation", "grammars/return_annotation.ebnf", "generated/return_annotation.json",
     "generated/return_annotation_parser.rs", True, None),
    ("regex", "grammars/regex.ebnf", "generated/regex.json",
     "generated/regex_parser.rs", False, "focus_regex"),
    ("json", "grammars/json.ebnf", "generated/json.json",
     "generated/json_parser.rs", False, "focus_json"),
    ("scratch", "grammars/scratch/scratch.ebnf", "generated/scratch.json",
     "generated/scratch_parser.rs", False, "focus_scratch"),
    ("rtl_const_expr", "grammars/rtl_const_expr.ebnf", "generated/rtl_const_expr.json",
     "generated/rtl_const_expr_parser.rs", False, "focus_rtl_const_expr"),
    ("systemverilog_preprocessor", "grammars/systemverilog_preprocessor.ebnf",
     "generated/systemverilog_preprocessor.json",
     "generated/systemverilog_preprocessor_parser.rs", False, "focus_systemverilog_preprocessor"),
    ("rtl_frontend", "grammars/rtl_frontend.ebnf", "generated/rtl_frontend.json",
     "generated/rtl_frontend_parser.rs", False, "focus_rtl_frontend"),
    ("vhdl", "grammars/vhdl.ebnf", "generated/vhdl.json",
     "generated/vhdl_parser.rs", False, "focus_vhdl"),
    ("systemverilog", "grammars/systemverilog.ebnf", "generated/systemverilog.json",
     "generated/systemverilog_parser.rs", False, "focus_systemverilog"),
]

os.makedirs(WORK, exist_ok=True)


def digest(path, drop_paths=(), strip_generated_at=False):
    """sha256 with the two PROVENANCE fields normalised away — see the header. `drop_paths` are the
    two output paths (census and shipped) that the artifact embeds as a diagnostic string."""
    try:
        with open(path, "rb") as f:
            blob = f.read().decode("utf-8", errors="replace")
    except OSError:
        return "<absent>"
    for p in drop_paths:
        blob = blob.replace(p, "<OUTPUT-PATH>")
    if strip_generated_at:
        blob = "\n".join(l for l in blob.splitlines()
                         if '"generated_at"' not in l and '"source_file"' not in l)
    return hashlib.sha256(blob.encode("utf-8")).hexdigest()[:16]


def timed(cmd):
    """Wall time of one invocation, plus its exit code. Timed around the process, so it includes
    exec + dyld — the cost the make recipe genuinely pays."""
    t0 = time.monotonic()
    rc = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True).returncode
    return time.monotonic() - t0, rc


for tool in (FRONTEND, GENERATOR, GENERATOR_BOOTSTRAP):
    if not os.access(os.path.join(ROOT, tool), os.X_OK):
        print(f"census: required binary missing or not executable: {tool}\n"
              "  build it first (make -C rust SHELL=/bin/bash $(RUST_AST_PIPELINE)) — a census that\n"
              "  cannot run its subject must refuse, not report zeros.", file=sys.stderr)
        sys.exit(2)

print("=" * 100)
print("CI-PARITY-GATE-ROT.32 (d) — MAKE 3.81 STALE-WINDOW CENSUS (every figure measured this run)")
print("=" * 100)
print(f"make: {subprocess.run(['make','--version'],capture_output=True,text=True).stdout.splitlines()[0]}")
print(f"reps per step: {REPS}   (the reported figure is the MINIMUM — immunity must hold on the")
print( "                        fastest observed run, not on the average)")
print("-" * 100)
print(f"{'family':<28}{'gram B':>9}{'t_front':>10}{'t_gen':>10}  "
      f"{'edge A (grammar→json)':<24}{'edge B (json→parser)':<24}")
print("-" * 100)

rows, control_failures = [], []
for fam, gram, js, parser, boot, focus in FAMILIES:
    tmp_json = os.path.join(WORK, f"{fam}.json")
    tmp_parser = os.path.join(WORK, f"{fam}_parser.rs")

    front_times = []
    for _ in range(REPS):
        dt, rc = timed([FRONTEND, gram, "--emit-raw-ast-json", tmp_json])
        if rc != 0:
            control_failures.append(f"{fam}: frontend exited {rc}")
            break
        front_times.append(dt)

    gen_cmd = [GENERATOR_BOOTSTRAP if boot else GENERATOR, "--generate-parser"]
    gen_cmd += ["--bootstrap-mode"] if boot else []
    gen_cmd += ["--eliminate-left-recursion", js, "-o", tmp_parser]
    gen_times = []
    for _ in range(REPS):
        dt, rc = timed(gen_cmd)
        if rc != 0:
            control_failures.append(f"{fam}: generator exited {rc}")
            break
        gen_times.append(dt)

    if not front_times or not gen_times:
        continue
    t_front, t_gen = min(front_times), min(gen_times)

    # ---- identity controls: did we pay the REAL work? -------------------------------------
    # `make` runs from `rust/`, so every path the shipped artifacts embed is one `../` deeper.
    embedded = (tmp_json, tmp_parser, "../" + js, "../" + parser, js, parser)
    if digest(tmp_json, embedded, True) != digest(js, embedded, True):
        control_failures.append(f"{fam}: census json differs from the shipped generated/ json")
    if digest(tmp_parser, embedded) != digest(parser, embedded):
        control_failures.append(f"{fam}: census parser differs from the shipped generated/ parser")

    a_ok, b_ok = t_gen >= 1.0, t_front >= 1.0
    verdict_a = ("IMMUNE  (gap>=%.2fs)" % t_gen) if a_ok else ("EXPOSED (gap~%.2fs)" % t_gen)
    verdict_b = ("IMMUNE  (gap>=%.2fs)" % t_front) if b_ok else ("EXPOSED (gap~%.2fs)" % t_front)
    print(f"{fam:<28}{os.path.getsize(gram):>9}{t_front:>10.3f}{t_gen:>10.3f}  "
          f"{verdict_a:<24}{verdict_b:<24}")
    rows.append((fam, focus, t_front, t_gen, a_ok, b_ok))

print("-" * 100)
exposed_a = [r[0] for r in rows if not r[4]]
exposed_b = [r[0] for r in rows if not r[5]]
either = sorted({*exposed_a, *exposed_b})
print(f"instances measured                 : {len(rows)}/{len(FAMILIES)}")
print(f"EDGE A exposed (stale JSON served) : {len(exposed_a)}  {' '.join(exposed_a) or '—'}")
print(f"EDGE B exposed (stale PARSER served): {len(exposed_b)}  {' '.join(exposed_b) or '—'}")
print(f"exposed on EITHER edge             : {len(either)}  {' '.join(either) or '—'}")
print("-" * 100)
print("⭐ READ THIS AS A COST MODEL, NOT A RANKING. A family is EXPOSED when its own build is fast")
print("   enough that a scripted driver can complete it and edit the grammar again inside one second.")
print("   Slowness is the only thing protecting the rest — which is a property of today's grammars")
print("   and today's machine, not a property of the rules. Both are free to change.")
if control_failures:
    print("-" * 100)
    print("⛔ CONTROL FAILURES — the timings above cannot be trusted:")
    for c in control_failures:
        print(f"     {c}")
    sys.exit(1)
print(f"✅ identity controls: {len(rows)}/{len(rows)} families reproduced their shipped json AND parser")
print("   byte-identically from the tracked grammar — the measured work was the real work.")
PYEOF
