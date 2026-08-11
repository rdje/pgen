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

Run:
    python3 stimuli/sv/run_adjudication_repros.py            # exit 0 = every claim still holds
    python3 stimuli/sv/run_adjudication_repros.py --verbose
"""
from __future__ import annotations

import argparse
import csv
import subprocess
import sys
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


def parses(path: Path) -> bool:
    proc = subprocess.run(
        [str(PROBE), "--parse", "systemverilog", str(path), "--profile", "sv_2017"],
        capture_output=True, text=True, timeout=TIMEOUT_S)
    return proc.returncode == 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--verbose", action="store_true")
    args = ap.parse_args()

    if not PROBE.exists():
        raise SystemExit(f"⛔ REFUSING: {PROBE} is missing. Build it first:\n"
                         "   (cd rust && cargo build --release --features generated_parsers "
                         "--bin parseability_probe)")

    with MANIFEST.open(encoding="utf-8") as fh:
        rows = list(csv.DictReader(fh, delimiter="\t"))

    listed = {r["id"] for r in rows}
    on_disk = {p.name for p in REPROS.glob("*.sv")}
    failures = []
    for missing in sorted(listed - on_disk):
        failures.append(f"{missing}: listed in the manifest, absent on disk")
    for orphan in sorted(on_disk - listed):
        failures.append(f"{orphan}: on disk, absent from the manifest — every reproducer must "
                        "carry its claim")

    checked = 0
    for row in sorted(rows, key=lambda r: r["id"]):
        path = REPROS / row["id"]
        if not path.exists():
            continue
        got = "ACCEPT" if parses(path) else "REJECT"
        checked += 1
        ok = got == row["expect"]
        if args.verbose or not ok:
            print(f"{'ok  ' if ok else 'FAIL'} {row['id']:<44} "
                  f"{row['class']:<8} expect={row['expect']} got={got}")
        if not ok:
            failures.append(f"{row['id']}: expected {row['expect']}, got {got}\n"
                            f"      {FIX_HINT.get((row['class'], got), '')}\n"
                            f"      construct: {row['construct']}  (LRM {row['lrm']})")

    print(f"\nADJUDICATION-REPROS: checked={checked} listed={len(rows)} "
          f"failures={len(failures)}")
    if failures:
        print("\n".join(f"  ⛔ {f}" for f in failures))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
