#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2f slice 4 — the RED control for the four new `arm` claims.

⛔ WHY THIS EXISTS. `PATHPULSE$a$y = 3;` ACCEPTED both before and after slice 4 — before, because
`pulse_control_specparam` was unreachable and the text fell through to the ordinary
`specparam_assignment`; after, because that fall-through is still legal for a single limit value.
So a VERDICT proves nothing here, and the manifest rows therefore pin the AST ARM. An arm claim is
itself a claim, and `docs/CLAIM_VERIFICATION.md` §3 leg 2 says a control never seen RED is not known
to work — this probe runs each claim against an AST it MUST refuse as well as one it must accept.

It calls the runner's OWN checker (`arm_failures` / `_kind_chain_present`), never a re-implementation,
so a change to the arm semantics is measured here rather than mirrored.

    python3 docs/tasks/artifacts/sv_corpus_grad/es13c2f4_pathpulse/arm_red_control.py
    # ARM-RED-CONTROL: 8/8 as declared, 4 of them RED — the checker is proven able to fail

Exit 0 iff every case behaves as declared; the total is DERIVED from the cases that ran, never stored.
"""
import importlib.util
import os
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
os.chdir(ROOT)

RUNNER = "stimuli/sv/run_adjudication_repros.py"
REPROS = pathlib.Path("stimuli/sv/adjudication_repros")

spec = importlib.util.spec_from_file_location("rar", RUNNER)
if spec is None or spec.loader is None:
    sys.exit(f"arm-red-control: cannot load {RUNNER} — refusing to publish a verdict")
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)

# (repro, arm claim, must the checker REJECT it?, why)
CASES = [
    ("control_specparam_pathpulse_scalar.sv", "pulse>general", True,
     "the ordinary route produces no `pulse` node"),
    ("control_specparam_pathpulse_scalar.sv", "pulse>input_output", True,
     "same, for the path-specific alternative's kind"),
    ("control_specparam_pathpulse_scalar.sv", "!pulse", False,
     "the manifest's own claim for this control"),
    ("fixed_pathpulse_global.sv", "pulse>general", False,
     "A.7.5 alternative 1 fired; `general` is UNIQUE to it"),
    ("fixed_pathpulse_global.sv", "!pulse", True,
     "the negation must fail where the pulse route IS taken"),
    ("fixed_pathpulse_path_specific.sv", "pulse>input_output", False,
     "A.7.5 alternative 2 fired"),
    ("fixed_pathpulse_path_specific.sv", "pulse>general", True,
     "the two alternatives must not satisfy each other's claim"),
    ("fixed_pathpulse_lrm_30_7_1.sv", "pulse>input_output", False,
     "IEEE 1800-2023 §30.7.1's own example, on the pulse route"),
]

rows, red, as_declared = [], 0, 0
for name, arm, must_fail, why in CASES:
    path = REPROS / name
    if not path.is_file():
        sys.exit(f"arm-red-control: {path} is missing — refusing to report on a population it cannot read")
    ast = runner.parse_ast(path)
    if ast is None:
        sys.exit(f"arm-red-control: no AST for {name} — the probe or the parser is wrong, not the arm")
    bad = runner.arm_failures({"arm": arm}, ast)
    good = bool(bad) == must_fail
    red += bool(bad)
    as_declared += good
    rows.append((("RED " if bad else "GREEN"), name, arm, why, good))

for verdict, name, arm, why, good in rows:
    print(f"  {verdict:<5} {name:<40} arm={arm:<20} {why:<52} {'✓' if good else '✗ UNEXPECTED'}")

print(f"\nARM-RED-CONTROL: {as_declared}/{len(rows)} as declared, {red} of them RED — "
      "the checker is proven able to fail")
sys.exit(0 if as_declared == len(rows) else 1)
