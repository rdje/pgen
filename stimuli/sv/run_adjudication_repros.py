#!/usr/bin/env python3
"""The SV-CORPUS-GRAD.13c.2 adjudication ORACLE — a two-sided ratchet over minimal reproducers.

WHY THIS EXISTS
---------------
`.13c.2` adjudicated the 57-row candidate worklist and reduced its structural residue to a
handful of NAMED constructs. Every one of those adjudications is a claim about the parser, and
a claim that is only written down rots. Each is therefore pinned here as a MINIMAL reproducer
whose verdict this runner re-executes, in both directions:

  * `class=defect`  — valid SV that PGEN rejects today. Expected `REJECT`, and the manifest
    says so. ⭐ When the owning fix lands, this runner FAILS with "flip it to ACCEPT" — so a
    fix cannot land silently and the manifest cannot drift away from the parser.
  * `class=invalid` — SV that is NOT legal, which PGEN correctly rejects. Expected `REJECT`
    FOREVER. ⛔ If one of these ever ACCEPTs, PGEN has acquired an OVER-ACCEPTANCE defect —
    the failure mode the strict-LRM default exists to prevent, and the one that is invisible
    without a negative oracle ([[feedback_sv_strict_lrm_compliance_default]]).
  * `class=control` — the neighbouring construct that DOES parse, so each defect/invalid
    reproducer is pinned to exactly one difference. A control that stops parsing means the
    reproducer no longer isolates what its comment claims.

⛔ THE ORACLE MUST BE ABLE TO FAIL AND ABLE TO RUN
([[a-check-whose-inputs-all-pass-has-not-been-tested]]). Both directions above have teeth:
`defect` fails on an unannounced FIX, `invalid`/`control` fail on a regression. A manifest row
whose file is missing, or a file with no manifest row, is a hard error — not a skip.

⭐⭐ THE `arm` COLUMN — WHY A VERDICT IS NOT ENOUGH (SV-CORPUS-GRAD.13c.2a.2)
---------------------------------------------------------------------------
An ACCEPT says the text parsed. It does NOT say WHICH alternative parsed it, and where a rule
ends in a catch-all arm reaching the general expression hierarchy, the answer is routinely "not
the one under test". This oracle shipped a control (`control_select_expression_and.sv`) whose
whole claim — *"both operands carry the KEYWORD `intersect`, so it cannot be passing as one
plain expression"* — was FALSE: it parsed with the `&&` continuation dead, because the seed
`select_condition` swallowed the entire remainder through a `covergroup_range_list*` whose
literal braces had been lost. A verdict-only oracle called that green.

So a row may pin the ARM as well as the verdict. `arm` is a `,`-separated list of claims; each
claim is a `>`-separated ancestor→descendant chain of AST `kind` values that must appear nested
(each step is any descendant, not necessarily a child), e.g. `select_chain>with_matches`.
Prefix a chain with `!` to require its ABSENCE. Choose `kind` values that are UNIQUE to the
alternative under test — verify with `grep -c 'kind: "X"' grammars/systemverilog.ebnf — and
prove the claim can fail by running it against the input it must reject.

Only ACCEPT rows may carry an `arm`: a REJECT produces no AST, so an `arm` there is an
incoherent manifest, and this runner treats it as a hard error rather than a skip.

Run:
    python3 stimuli/sv/run_adjudication_repros.py            # exit 0 = every claim still holds
    python3 stimuli/sv/run_adjudication_repros.py --verbose
"""
from __future__ import annotations

import argparse
import csv
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REPROS = ROOT / "stimuli/sv/adjudication_repros"
MANIFEST = REPROS / "MANIFEST.tsv"
PROBE = ROOT / "rust/target/release/parseability_probe"
TIMEOUT_S = 60

# ⛔ The hint is keyed by (class, WHICH WAY it broke). A single per-class message told an
# operator "a defect reproducer now parses" when the manifest had simply been edited to expect
# ACCEPT before the fix landed — the exact opposite instruction. A gate's message is part of
# the gate.
FIX_HINT = {
    ("defect", "ACCEPT"): ("⭐ A DEFECT REPRODUCER NOW PARSES. If the owning fix landed, that "
                           "is the good news — flip its `expect` to ACCEPT and its class to "
                           "`fixed` in stimuli/sv/adjudication_repros/MANIFEST.tsv, citing the "
                           "fixing work-unit."),
    ("defect", "REJECT"): ("⛔ The manifest expects this defect to be FIXED and it is not. "
                           "Either the fix has not landed, or it regressed."),
    ("invalid", "ACCEPT"): ("⛔ AN INVALID-SV REPRODUCER NOW PARSES. PGEN has acquired an "
                            "OVER-ACCEPTANCE defect: it accepts text the LRM does not admit. "
                            "This is a REGRESSION, not a manifest update."),
    ("invalid", "REJECT"): "⛔ The manifest expects this invalid text to be ACCEPTED. It must not be.",
    ("control", "REJECT"): ("⛔ A CONTROL STOPPED PARSING. Its reproducer no longer isolates "
                            "one difference, so the neighbouring claim is no longer pinned."),
    ("control", "ACCEPT"): "⛔ A control is expected to REJECT — controls exist to parse.",
    ("fixed", "REJECT"): ("⛔ A FIXED REPRODUCER REGRESSED — valid SV that used to parse is "
                          "rejected again."),
    ("fixed", "ACCEPT"): "⛔ The manifest expects this fixed reproducer to REJECT.",
}


ARM_HINT = ("⛔ THE VERDICT IS RIGHT AND THE ARM IS WRONG. The text parsed, but not through the "
            "alternative this row claims — the classic accidental route under a catch-all arm. "
            "An ACCEPT alone would have reported this as green.")


DEFAULT_PROFILE = "sv_2017"
KNOWN_PROFILES = ("sv_2017", "sv_2023", "verilog_2005")


def row_profiles(row: dict) -> list[str]:
    """Which dialect profiles this row binds on.

    ⛔ THE PARSER SHIPS THREE PROFILES AND THIS RUNNER CHECKED ONE (SV-CORPUS-GRAD.13c.2h,
    director 2026-08-18). The compliance goal is IEEE 1800-2017 + 1800-2023 + **1364-2005**, and a
    grammar rule with no `@profiles` gate governs all three identically — so an over-acceptance
    guard pinned only on `sv_2017` cannot see a relaxation that shows up under `verilog_2005`.
    ⭐ Default stays `sv_2017` so every historical row keeps its exact meaning; a row DECLARES the
    wider set when its construct is ungated. Fixing this by silently running all three everywhere
    would have re-adjudicated 29 rows nobody measured on the other two.
    """
    spec = (row.get("profiles") or "").strip()
    if not spec:
        return [DEFAULT_PROFILE]
    out = []
    for name in (x.strip() for x in spec.split(",") if x.strip()):
        if name not in KNOWN_PROFILES:
            raise SystemExit(f"⛔ REFUSING: {row['id']} declares unknown profile '{name}'. "
                             f"Known: {', '.join(KNOWN_PROFILES)}. A silently-ignored profile "
                             "would make this ratchet report coverage it does not have.")
        out.append(name)
    return out


def parses(path: Path, profile: str = DEFAULT_PROFILE) -> bool:
    proc = subprocess.run(
        [str(PROBE), "--parse", "systemverilog", str(path), "--profile", profile],
        capture_output=True, text=True, timeout=TIMEOUT_S)
    return proc.returncode == 0


def parse_ast(path: Path, profile: str = DEFAULT_PROFILE):
    """The typed AST for `path`, or None when the probe declines to produce one."""
    with tempfile.TemporaryDirectory(dir=str(ROOT / "tmp")) as workdir:
        out = Path(workdir) / "ast.json"
        proc = subprocess.run(
            [str(PROBE), "--parse-dump-ast", "systemverilog", str(path), str(out),
             "--profile", profile],
            capture_output=True, text=True, timeout=TIMEOUT_S)
        if proc.returncode != 0 or not out.exists():
            return None
        return json.loads(out.read_text(encoding="utf-8"))


def _kind_chain_present(node, chain: list[str]) -> bool:
    """True when `chain` occurs as a nested ancestor→descendant sequence of `kind` values.

    Each step matches any DESCENDANT, not only a direct child, because the shape between two
    annotated rules is an implementation detail of the intervening un-annotated rules.
    """
    if not chain:
        return True
    head, rest = chain[0], chain[1:]
    if isinstance(node, dict):
        if node.get("kind") == head and any(
                _kind_chain_present(v, rest) for v in node.values()):
            return True
        return any(_kind_chain_present(v, chain) for v in node.values())
    if isinstance(node, list):
        return any(_kind_chain_present(v, chain) for v in node)
    return False


def arm_failures(row: dict, ast) -> list[str]:
    """Which of the row's `arm` claims the produced AST does not support."""
    spec = (row.get("arm") or "").strip()
    if not spec:
        return []
    if ast is None:
        return [f"arm '{spec}' cannot be checked — the probe produced no AST"]
    bad = []
    for claim in (c.strip() for c in spec.split(",") if c.strip()):
        negated = claim.startswith("!")
        chain = [step.strip() for step in claim.lstrip("!").split(">") if step.strip()]
        present = _kind_chain_present(ast, chain)
        if present and negated:
            bad.append(f"arm claim '{claim}': the chain IS present in the AST, and the manifest "
                       "requires its absence")
        elif not present and not negated:
            bad.append(f"arm claim '{claim}': the chain is ABSENT from the AST, and the manifest "
                       "requires it — the input parsed down some other route")
    return bad


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--verbose", action="store_true")
    args = ap.parse_args()

    if not PROBE.exists():
        raise SystemExit(f"⛔ REFUSING: {PROBE} is missing. Build it first:\n"
                         "   (cd rust && cargo build --release --features generated_parsers "
                         "--bin parseability_probe)")

    (ROOT / "tmp").mkdir(exist_ok=True)

    with MANIFEST.open(encoding="utf-8") as fh:
        reader = csv.DictReader(fh, delimiter="\t")
        if "arm" not in (reader.fieldnames or []):
            raise SystemExit("⛔ REFUSING: MANIFEST.tsv has no `arm` column. The column is how a "
                             "row pins WHICH alternative parsed it; a manifest without it silently "
                             "downgrades every claim to a verdict.")
        rows = list(reader)

    listed = {r["id"] for r in rows}
    on_disk = {p.name for p in REPROS.glob("*.sv")}
    failures = []
    for missing in sorted(listed - on_disk):
        failures.append(f"{missing}: listed in the manifest, absent on disk")
    for orphan in sorted(on_disk - listed):
        failures.append(f"{orphan}: on disk, absent from the manifest — every reproducer must "
                        "carry its claim")

    for row in rows:
        if (row.get("arm") or "").strip() and row["expect"] != "ACCEPT":
            failures.append(f"{row['id']}: carries an `arm` claim but expects "
                            f"{row['expect']} — a rejected input produces no AST, so the claim "
                            "can never be checked. Drop the arm or fix the expectation.")

    checked = 0
    armed = 0
    multi = 0
    for row in sorted(rows, key=lambda r: r["id"]):
        path = REPROS / row["id"]
        if not path.exists():
            continue
        profiles = row_profiles(row)
        if len(profiles) > 1:
            multi += 1
        for profile in profiles:
            got = "ACCEPT" if parses(path, profile) else "REJECT"
            checked += 1
            ok = got == row["expect"]
            bad_arms: list[str] = []
            if ok and got == "ACCEPT" and (row.get("arm") or "").strip():
                bad_arms = arm_failures(row, parse_ast(path, profile))
                armed += 1
            if args.verbose or not ok or bad_arms:
                status = "ok  " if ok and not bad_arms else "FAIL"
                arm = f" arm={row['arm']}" if (row.get("arm") or "").strip() else ""
                print(f"{status} {row['id']:<44} {row['class']:<8} "
                      f"profile={profile:<12} expect={row['expect']} got={got}{arm}")
            if not ok:
                failures.append(f"{row['id']} [{profile}]: expected {row['expect']}, got {got}\n"
                                f"      {FIX_HINT.get((row['class'], got), '')}\n"
                                f"      construct: {row['construct']}  (LRM {row['lrm']})")
            for bad in bad_arms:
                failures.append(f"{row['id']} [{profile}]: {bad}\n      {ARM_HINT}\n"
                                f"      construct: {row['construct']}  (LRM {row['lrm']})")

    print(f"\nADJUDICATION-REPROS: checked={checked} armed={armed} listed={len(rows)} "
          f"multi_profile_rows={multi} failures={len(failures)}")
    if failures:
        print("\n".join(f"  ⛔ {f}" for f in failures))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
