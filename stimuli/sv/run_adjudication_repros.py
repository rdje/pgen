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
  * `class=accepts_invalid` — ILLEGAL SV that PGEN accepts TODAY. Expected `ACCEPT`, because that
    is what the parser does; the row exists so the over-acceptance is WATCHED rather than merely
    written down. ⭐ When the owning fix lands this runner FAILS with "flip it to `invalid`", and
    the row then guards the fix against regression forever — the exact mirror of `defect`.
    ⛔ It exists because the four classes above could hold *valid text wrongly rejected* and could
    not hold *invalid text wrongly accepted*: filing such a row as `invalid` goes RED on the commit
    that files it, and filing it as anything else lies. A known over-acceptance could therefore live
    in prose and nowhere the runner could see it (`SV-CORPUS-GRAD.13c.2k`).
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
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REPROS = ROOT / "stimuli/sv/adjudication_repros"
MANIFEST = REPROS / "MANIFEST.tsv"
# ⛔ THE PROBE IS SELECTABLE, because an EXPERIMENTAL ARM has to be checkable by the same oracle
# that guards the shipped parser. `SV-CORPUS-GRAD.13c.2k` built three candidate spellings of one
# fix and could compare their VERDICTS by hand but not their AST ARMS — and the arm column is the
# only instrument that sees a construct parsing through the WRONG production (`.13c.2q`, where the
# verdict never moved). Without this, checking an arm meant a ~22-minute release build, so it was
# skipped. `stimuli/sv/corpus_parse_cost.py` already carries the same override for the same reason
# (`PGEN_PARSE_COST_PROBE`); this is that gap closed for the repro runner.
# ⚠️ The DEFAULT is unchanged — the release probe — so every gated invocation is unaffected, and
# the run PRINTS which binary and which generated parser it used, so a measurement taken on an
# experimental arm can never be mistaken for one taken on the shipped one.
# `.resolve()` so a RELATIVE override (the natural way to type it) still yields a path the
# report can print repo-root-relative — the first cut crashed on exactly that.
PROBE = Path(os.environ.get("PGEN_ADJUDICATION_PROBE")
             or ROOT / "rust/target/release/parseability_probe").resolve()
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
    ("accepts_invalid", "REJECT"): ("⭐ AN OVER-ACCEPTANCE IS GONE. If the owning fix landed, that "
                                    "is the good news — flip its `expect` to REJECT and its class "
                                    "to `invalid` in stimuli/sv/adjudication_repros/MANIFEST.tsv, "
                                    "citing the fixing work-unit. The row then guards the FIX "
                                    "forever, which is the whole point of the round trip."),
    ("accepts_invalid", "ACCEPT"): ("⛔ The manifest expects this KNOWN over-acceptance to still be "
                                    "accepted; a REJECT here would mean it was fixed."),
}

# ⭐⭐ THE FIVE CLASSES AND THE VERDICT EACH ONE MEANS (SV-CORPUS-GRAD.13c.2k).
#
# ⛔ `accepts_invalid` is the FIFTH, and it exists because the first four could not hold one
# direction of a claim. `defect` says *valid SV that PGEN rejects today* and goes RED the day the
# fix lands; there was no mirror for *ILLEGAL SV that PGEN accepts today*. Filing such a row as
# `invalid` expects REJECT and so fails on the commit that files it; filing it as anything else
# lies. ⇒ a known over-acceptance could be written in prose and NOWHERE the runner could see it,
# which is exactly the asymmetry that lets an accepts-invalid defect age quietly while every
# rejects-valid defect is ratcheted. Now it goes RED when it is FIXED, and the fixer flips it to
# `invalid` — so the round trip ends with the defect guarded against regression forever.
#
# ⛔ An UNKNOWN class is REFUSED rather than skipped. Before this, `FIX_HINT.get((class, got), "")`
# meant a typo'd class produced an empty hint and — worse — a row whose expectation nothing
# cross-checked. A manifest is an oracle; an unreadable row in it must never read as a green one.
EXPECTED_BY_CLASS = {
    "defect": "REJECT",           # valid SV that PGEN rejects today
    "fixed": "ACCEPT",            # ... once the owning fix lands
    "invalid": "REJECT",          # illegal SV that PGEN correctly rejects, forever
    "accepts_invalid": "ACCEPT",  # illegal SV that PGEN wrongly accepts today
    "control": "ACCEPT",          # the neighbouring construct that DOES parse
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

    # Which binary, and which generated parser inside it — never assumed
    # (`ENGINE-UNIVERSAL-SERVICES.24`: a probe built from one arm was measured as another).
    fp = subprocess.run([str(PROBE), "--parser-fingerprint"], capture_output=True,
                        text=True, timeout=TIMEOUT_S).stdout
    try:
        sv_parser = json.loads(fp)["parsers"]["systemverilog"]
    except Exception:
        sv_parser = "unknown (this probe predates --parser-fingerprint)"
    print(f"ADJUDICATION-REPROS: probe={os.path.relpath(PROBE, ROOT)} sv_parser={sv_parser}")

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
        cls = (row.get("class") or "").strip()
        if cls not in EXPECTED_BY_CLASS:
            failures.append(f"{row['id']}: class `{cls}` is not one of "
                            f"{sorted(EXPECTED_BY_CLASS)} — an unreadable row in an oracle must "
                            f"never read as a green one.")
        elif row["expect"] != EXPECTED_BY_CLASS[cls]:
            failures.append(f"{row['id']}: class `{cls}` means expect="
                            f"{EXPECTED_BY_CLASS[cls]}, but the row says {row['expect']}. The "
                            f"class IS the claim; an incoherent pair checks nothing.")
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
