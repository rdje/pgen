#!/usr/bin/env python3
"""`ENGINE-UNIVERSAL-SERVICES.13` acceptance (a) — the PER-CYCLE adjudication, re-runnable.

For each of the 10 distinct surviving left-recursive cycles (see `canonicalize_lint_cycles.py`),
this drives a spec-grounded input that NEEDS the recursion and records the probe verdict. The leaf
opened by saying *"the 30 are not 30 defects — for many the LRM may never put a legal string on that
path"*. Measured, **8 of the 10 do**.

⛔ TWO VERDICTS ARE NOT "PASS": `DEAD-BUT-COVERED` means the probe ACCEPTS while the cycle's own
alternative stays unreachable, the text being served by a sibling rule. That is the
`SV-CORPUS-GRAD.13c.2b` trap in the other direction, and it is why every accepting row here carries
the trace/AST evidence naming which branch actually won. A row that merely parses proves nothing.

Two probe families, because the two grammars are reached by different tools:

  * SystemVerilog — `parseability_probe --parse systemverilog --profile <p>`.
  * `ebnf` — no parseability adapter exists, so each probe is run through BOTH frontends:
    `ast_pipeline --lint-grammar` (arm 1, the hand-written `ebnf_frontend`) and
    `ebnf_dual_run_diff` (arm 2, the parser GENERATED from `grammars/ebnf.ebnf`). A row where arm 1
    accepts and arm 2 rejects is a live frontend-replacement blocker (`LANG-CAPABILITY-AUDIT.10.6`).

  python3 docs/tasks/artifacts/engine_universal_services/lr_cycle_adjudication/adjudicate.py
"""
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from _repo_root import repo_root  # noqa: E402  (path is set immediately above)

PROBES = Path(__file__).resolve().parent / "probes"
REPROS = "stimuli/sv/adjudication_repros"

# (cycle id, knot, probe path, profile, expected verdict, what the input needs the recursion for)
SV_ROWS = [
    ("SV-1", "cast/call", PROBES / "sv_cast_call_chain.sv", "sv_2017", "REJECT",
     "size cast whose casting_type is a constant_function_call"),
    ("SV-5", "cast/call", PROBES / "sv_cast_call_chain.sv", "sv_2023", "REJECT",
     "same, through the sv_2023 method-call receiver"),
    ("SV-2", "cast/const", f"{REPROS}/defect_constant_size_cast.sv", "sv_2017", "REJECT",
     "numeric size cast in a constant expression"),
    ("SV-3", "cast/const", f"{REPROS}/defect_constant_size_cast.sv", "sv_2023", "REJECT",
     "same, through constant_primary_sv_2023"),
    ("SV-6", "property", PROBES / "sv_property_implication.sv", "sv_2017", "REJECT",
     "property_expr implies property_expr with a non-expression left operand"),
    ("SV-7", "property", PROBES / "sv_property_implication.sv", "sv_2023", "REJECT",
     "same, through the sv_2023 property cascade"),
    ("SV-4", "class-scope", PROBES / "sv_incomplete_class_scoped_type.sv", "sv_2023", "ACCEPT",
     "3+ segment incomplete class scope — DEAD-BUT-COVERED by data_type"),
]

# (cycle id, probe path, expected arm-1 rc, expected arm-2 rc, note)
EBNF_ROWS = [
    ("EBNF-1", PROBES / "ebnf_arithmetic_return.ebnf", 0, 1, "-> $1 + $2"),
    ("EBNF-2", PROBES / "ebnf_conditional_return.ebnf", 0, 1, "-> $1 ? $2 : $3"),
    ("EBNF-3", PROBES / "ebnf_member_access_return.ebnf", 0, 0, "-> $1.name — DEAD-BUT-COVERED"),
]


def require(path: Path, build_hint: str) -> Path:
    if not path.is_file():
        raise SystemExit(f"REFUSE: {path} is missing — build it with:\n  {build_hint}")
    return path


def probe_sv(root: Path, source: Path, profile: str) -> tuple[str, str]:
    binary = require(
        root / "rust/target/release/parseability_probe",
        "(cd rust && cargo build --release --features generated_parsers --bin parseability_probe)",
    )
    result = subprocess.run(
        [str(binary), "--parse", "systemverilog", str(source), "--profile", profile],
        capture_output=True,
        text=True,
    )
    tail = (result.stdout + result.stderr).strip().splitlines()[-1]
    if tail.startswith("parse_full passed"):
        return "ACCEPT", ""
    detail = tail.split("[")[-1].rstrip("]").split(",")[0]
    return "REJECT", detail


def probe_ebnf(root: Path, source: Path) -> tuple[int, int, Path]:
    """Run BOTH ebnf frontends. Returns (arm1_rc, arm2_rc, arm2_ast_path).

    ⛔ `--emit-ast-json` is REQUIRED, not decorative: without it `ebnf_dual_run_diff` reports only
    arm 1 and exits 0 even when the generated meta-parser rejects the input. A first draft of this
    script omitted it and read three green rows where two are rejections.
    """
    pipeline = require(
        root / "rust/target/debug/ast_pipeline",
        '(cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline)',
    )
    differ = require(
        root / "rust/target/debug/ebnf_dual_run_diff",
        "(cd rust && cargo build --features ebnf_dual_run --bin ebnf_dual_run_diff)",
    )
    work = root / "rust/target/lr_cycle_adjudication"
    work.mkdir(parents=True, exist_ok=True)
    ast_path = work / f"{source.stem}.arm2.json"
    ast_path.unlink(missing_ok=True)
    arm1 = subprocess.run(
        [str(pipeline), str(source), "--lint-grammar"], capture_output=True, text=True
    ).returncode
    arm2 = subprocess.run(
        [
            str(differ),
            "--input", str(source),
            "--output", str(work / f"{source.stem}.json"),
            "--emit-ast-json", str(ast_path),
        ],
        capture_output=True,
        text=True,
    ).returncode
    return arm1, arm2, ast_path


def dead_but_covered_sv4(root: Path, source: Path) -> str:
    """EARN SV-4's DEAD-BUT-COVERED verdict instead of asserting it.

    An accepting probe proves nothing about the cycle (the SV-CORPUS-GRAD.13c.2b trap). The verdict
    holds only if BOTH are true in the same traced run: the guard rejected the recursive alternative,
    AND the accepting text was served by the sibling `data_type` branch.
    """
    binary = root / "rust/target/release/parseability_probe"
    work = root / "rust/target/lr_cycle_adjudication"
    work.mkdir(parents=True, exist_ok=True)
    log = work / "sv4_trace.log"
    subprocess.run(
        [
            str(binary), "--parse", "systemverilog", str(source), "--profile", "sv_2023",
            "--trace-rules", "data_type_or_incomplete_class_scoped_type",
            "--trace-log-file", str(log),
        ],
        capture_output=True,
        text=True,
        env={**__import__("os").environ, "PGEN_TRACE_VERBOSITY": "high"},
    )
    text = log.read_text(errors="replace") if log.is_file() else ""
    guarded = "Infinite recursion detected in rule 'incomplete_class_scoped_type'" in text
    sibling = "Rule 'data_type_or_incomplete_class_scoped_type_sv_2023' selected branch 1/2" in text
    if guarded and sibling:
        return "guard rejected the recursive alternative; data_type (branch 1/2) served the text"
    return (
        f"⛔ UNEARNED: guard_seen={guarded} sibling_branch_seen={sibling} — "
        "the DEAD-BUT-COVERED verdict no longer reproduces"
    )


def main() -> int:
    root = repo_root()
    status = 0

    print("SystemVerilog — 7 distinct cycles")
    print(f"  {'cycle':<6} {'knot':<12} {'profile':<8} {'verdict':<8} {'expected':<8} needs the recursion for")
    for cycle, knot, source, profile, expected, why in SV_ROWS:
        path = source if isinstance(source, Path) else root / source
        verdict, detail = probe_sv(root, path, profile)
        flag = "" if verdict == expected else "   ⛔ MOVED"
        print(f"  {cycle:<6} {knot:<12} {profile:<8} {verdict:<8} {expected:<8} {why}{flag}")
        if detail:
            print(f"  {'':<6} {'':<12} {'':<8} {detail}")
        if cycle == "SV-4":
            evidence = dead_but_covered_sv4(root, path)
            print(f"  {'':<6} {'':<12} {'':<8} {evidence}")
            if evidence.startswith("⛔"):
                status = 1
        if verdict != expected:
            status = 1

    print("\nebnf — 3 distinct cycles (arm 1 = hand-written frontend, arm 2 = generated meta-parser)")
    print(f"  {'cycle':<8} {'arm1':<6} {'arm2':<6} {'expected':<10} input")
    for cycle, source, want1, want2, note in EBNF_ROWS:
        arm1, arm2, ast_path = probe_ebnf(root, source)
        ok = (arm1, arm2) == (want1, want2)
        flag = "" if ok else "   ⛔ MOVED"
        print(f"  {cycle:<8} rc={arm1:<3} rc={arm2:<3} {want1}/{want2:<8} {note}{flag}")
        if not ok:
            status = 1
        if cycle == "EBNF-3" and ast_path.is_file():
            # EARN the DEAD-BUT-COVERED verdict: `$1.name` must parse as a positional reference
            # with a property-access suffix, NEVER through the left-recursive member_access_return.
            blob = ast_path.read_text()
            covered = "property_access_suffix" in blob and "member_access" not in blob
            note = (
                "arm 2 AST: positional_reference + property_access_suffix, no member_access node"
                if covered
                else "⛔ UNEARNED: the arm-2 AST no longer shows the covering shape"
            )
            print(f"  {'':<8} {note}")
            if not covered:
                status = 1

    print()
    if status == 0:
        print("VERDICT: 8 of 10 distinct cycles cost real text; 2 are DEAD-BUT-COVERED.")
        print("         The fix surface is 3 knots, not 30 cycles — cast/call+cast/const share the")
        print("         `casting_type -> constant_primary` edge, property is one alternative, and")
        print("         the ebnf pair is one `return_expression` knot.")
    else:
        print("VERDICT: a row moved. If REJECTs became ACCEPTs, .13 landed — re-baseline this table,")
        print("         the probe headers, and the two DEAD-BUT-COVERED rows' branch evidence.")
    return status


if __name__ == "__main__":
    raise SystemExit(main())
