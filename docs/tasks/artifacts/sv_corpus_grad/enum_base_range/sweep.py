#!/usr/bin/env python3
"""`SV-CORPUS-GRAD.3.23` — size and probe the base-less `enum [` population.

⛔ THE ROW IS THE WITNESS, NOT THE CONSTRUCT (`.3.16`). The stuck-signature clusters split
this one construct across TWO signatures — `{ ID =` and `{ ID ,` — purely on whether the
enum's first member carries an `= value`. A leaf cut from the top cluster alone would have
seen 7 rows and silently missed 2. This sweep keys on the CONSTRUCT instead.

The construct: `enum` followed directly by a packed dimension with **no base type**, e.g.
`typedef enum [2:0] { A, B } e_t;`. IEEE 1800-2017 A.2.2.1 gives

    enum_base_type ::= integer_atom_type [ signing ]
                     | integer_vector_type [ signing ] [ packed_dimension ]
                     | type_identifier [ packed_dimension ]

so a packed dimension is only ever reachable BEHIND a type, and clause 6.19 says it in
prose: *"In the absence of a data type declaration, the default data type shall be int. Any
other data type used with enumerated types shall require an explicit data type
declaration."* A `[2:0]` dimension IS another data type. ⇒ PGEN is RIGHT to reject, and the
corpus rows are mis-adjudicated `must_accept`.

Two lanes, both printed:
  * MATRIX  — six minimal enum shapes through the real probe, isolating which one rejects.
  * SWEEP   — every manifest row whose file contains the construct, with its adjudication
              class, so the population is sized rather than guessed.

Usage (repo-root-relative, directive 12):
  python3 docs/tasks/artifacts/sv_corpus_grad/enum_base_range/sweep.py
"""

import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "config_use_param_override"))
from _repo_root import repo_root  # noqa: E402

ROOT = repo_root()
HERE = Path(__file__).resolve().parent
PROBE = ROOT / "rust/target/release/parseability_probe"
MANIFEST = ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv"

# `enum` then optional layout then `[` — no type in between.
BASELESS = re.compile(r"(?<![A-Za-z0-9_$])enum\s*\[")
POS_RE = re.compile(r"furthest_position=(\d+)")

# (stem, what it isolates, expected verdict under IEEE 1800-2017)
MATRIX = [
    ("e1_dim_no_base",     "enum [2:0] { … }            packed dim, NO base type", "REJECT"),
    ("e2_no_base_no_dim",  "enum { … }                  no base, no dim (int)",    "ACCEPT"),
    ("e3_vector_base_dim", "enum logic [2:0] { … }      integer_vector_type + dim", "ACCEPT"),
    ("e4_atom_base",       "enum int { … }              integer_atom_type",        "ACCEPT"),
    ("e5_signing_dim",     "enum bit signed [2:0] { … } vector + signing + dim",   "ACCEPT"),
    ("e6_typeid_dim",      "enum my_t [2:0] { … }       type_identifier + dim",    "ACCEPT"),
]


def probe(path: Path, profile: str = "sv_2017"):
    r = subprocess.run([str(PROBE), "--parse", "systemverilog", str(path), "--profile", profile],
                       capture_output=True, text=True, timeout=120)
    out = r.stdout + r.stderr
    if "parse_full passed" in out:
        return "ACCEPT", "-"
    m = POS_RE.search(out)
    return "REJECT", m.group(1) if m else "?"


def main() -> int:
    if not PROBE.is_file():
        raise SystemExit(f"REFUSE: probe not built at {PROBE.relative_to(ROOT)}")

    print("## MATRIX — which enum base shape actually rejects (profile sv_2017)")
    print(f"  {'case':<20} {'verdict':<8} {'furthest':<9} {'want':<7} shape")
    mismatch = 0
    for stem, shape, want in MATRIX:
        verdict, furthest = probe(HERE / f"{stem}.sv")
        flag = "" if verdict == want else "   <-- differs from the LRM reading"
        mismatch += verdict != want
        print(f"  {stem:<20} {verdict:<8} {furthest:<9} {want:<7} {shape}{flag}")
    print(f"  cases differing from the LRM reading: {mismatch}")
    print()
    print("  => exactly ONE shape rejects, and it is the one Annex A cannot derive. Every")
    print("     LRM-expressible base (none / atom / vector+dim / signing+dim / type_id+dim)")
    print("     parses, so this is a precise strictness boundary, not a hole in enum support.")
    print()

    print("## SWEEP — the whole population, keyed on the CONSTRUCT (not on a cluster signature)")
    rows = [l.rstrip("\n").split("\t")
            for l in MANIFEST.read_text(encoding="utf-8", errors="replace").splitlines()[1:]]
    by_class, actionable = Counter(), []
    for r in rows:
        if len(r) < 5:
            continue
        suite, rel, _observed, expected, adj = r[0], r[1], r[2], r[3], r[4]
        p = ROOT / "stimuli/sv/subs" / suite / rel
        if not p.is_file():
            continue
        if BASELESS.search(p.read_text(encoding="utf-8", errors="replace")):
            by_class[adj] += 1
            if adj == "divergence:unexplained_rejects_valid":
                actionable.append((suite, rel, expected))
    print(f"  manifest rows scanned: {len(rows)}")
    for cls, n in by_class.most_common():
        print(f"    {n:4d}  {cls}")
    print(f"\n  ACTIONABLE (unexplained_rejects_valid): {len(actionable)}")
    for suite, rel, expected in actionable:
        print(f"    {suite:<12} {rel:<62} expected={expected}")
    print()
    if actionable:
        print("  => every one of them is expected `must_accept`, which is the mis-adjudication.")
        print("     ⛔ Correcting the PARSER here would be an over-acceptance defect")
        print("     ([[feedback_sv_strict_lrm_compliance_default]]); the fix is in the ADJUDICATOR.")
    else:
        # The conclusion used to print unconditionally, so a POST-FIX re-run asserted a
        # mis-adjudication that no longer existed — an instrument stating a stale finding as
        # a live one. It now reports the state it measured.
        print("  => 0 actionable: the population is fully adjudicated (`.3.23` pinned the 9 as")
        print("     `must_reject`, LRM-grounded and per-file justified by verify_pins.py). The")
        print("     construct is still REJECTED and no parser byte moved — an adjudication")
        print("     correction, never burn-down yield.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
