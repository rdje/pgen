#!/usr/bin/env python3
"""The config `use`-clause parameter-override repro matrix (`SV-CORPUS-GRAD.3.19`/`.3.20`).

Every case is a full, self-contained design + `config` block in which ONLY the single
config-rule line varies, so a verdict difference can only come from the `use_clause`.

Three groups, each answering a different question:

  a*  ANNEX-A forms — the four `use_clause` alternatives IEEE 1800-2017 A.1.5 writes out.
      These already parse; they are the regression tripwire proving the fix is additive.
  h*  CLAUSE-33 normative-EXAMPLE forms — `use #( ... )`, copied VERBATIM from the LRM's
      own examples in 33.4.3 (2017 :365/:366 and 2023 :413/:424), plus the two-parameter
      shape the external corpus uses. These are the defect: LRM text that does not parse.
  c*  CONTROLS — neighbouring surfaces (`liblist`, `cell ... use`, and the ORDINARY
      `parameter_value_assignment` on a module instantiation) that must not move.
  n*  STRICTNESS controls — POSITIONAL parameter notation inside a config, which
      LRM 33.4.3 :333 forbids in so many words ("Configurations may not use positional
      parameter notation to override parameters"). They must REJECT before AND after;
      a fix that reached for `parameter_value_assignment` would silently accept them,
      which is the over-acceptance this matrix exists to rule out
      ([[feedback_sv_strict_lrm_compliance_default]]).

⭐ THE EXPECTATION IS PER PROFILE, AND EVERY PROFILE IS JUDGED (`SV-CORPUS-GRAD.3.20`).
Until `.3.20` this matrix carried ONE expectation column — `sv_2017`'s — and printed it
beside a `verilog_2005` run "for reference without judging". So the v2005 arm had no
oracle at all: it could only ever agree with itself, which is exactly how the standing
v2005 over-acceptance (`use .W(8)` & co. under IEEE 1364-2005, which declares a single
`use [lib.]cell[:config]` production and contains zero occurrences of `use #(`) survived
inside a green matrix. EXPECT below is the LRM answer per (case, profile), and an
unrecognized profile is a REFUSAL rather than an unjudged run
([[feedback_instrument_needs_ground_truth]]).

Usage (from anywhere; paths resolve against the repository root, directive 12):
  python3 docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/matrix.py
  python3 docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/matrix.py --profile verilog_2005
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _repo_root import repo_root  # noqa: E402

ROOT = repo_root()
PROBE = ROOT / "rust/target/release/parseability_probe"
REPRO = Path(__file__).resolve().parent / "repro"
POS_RE = re.compile(r"furthest_position=(\d+)")

# The IEEE-1364-2005 use_clause is a SINGLE production — `use [library_identifier.]
# cell_identifier[:config]` (Annex A A.1.5 :119, restated 13.3 :271). Any parameter
# override in a config `use` clause, named or `#(…)`, is a SystemVerilog addition, so
# every override case below must REJECT under verilog_2005 and ACCEPT under sv_2017 /
# sv_2023. `c3` is deliberately NOT one of them: an ordinary module instantiation's
# `#( … )` parameter_value_assignment is legal Verilog-2001/2005 and must stay ACCEPT.
SV = ("sv_2017", "sv_2023")
V2005 = ("verilog_2005",)
KNOWN_PROFILES = SV + V2005

# (file stem, LRM citation, {profile-group: expected verdict AFTER the fix})
CASES = [
    ("a1_use_cell", "A.1.5 use_clause alt 1", "ACCEPT", "ACCEPT"),
    ("a2_use_lib_cell", "A.1.5 use_clause alt 1 (+lib)", "ACCEPT", "ACCEPT"),
    ("a3_use_lib_cell_config", "A.1.5 use_clause alt 1 (:config)", "ACCEPT", "ACCEPT"),
    ("a4_use_named_only", "A.1.5 use_clause alt 2", "ACCEPT", "REJECT"),
    ("a5_use_named_multi", "A.1.5 use_clause alt 2 (list)", "ACCEPT", "REJECT"),
    ("a6_use_cell_named", "A.1.5 use_clause alt 3", "ACCEPT", "REJECT"),
    ("a7_use_lib_cell_named", "A.1.5 use_clause alt 3 (+lib)", "ACCEPT", "REJECT"),
    ("h1_hash_named", "33.4.3 :365 (2017)", "ACCEPT", "REJECT"),
    ("h2_hash_named_hier", "33.4.3 :366 (2017)", "ACCEPT", "REJECT"),
    ("h3_hash_empty", "33.4.3 :424 (2023)", "ACCEPT", "REJECT"),
    ("h4_hash_named_default", "33.4.3 :413 (2023)", "ACCEPT", "REJECT"),
    ("h5_hash_named_multi", "corpus: verilator t_config_param", "ACCEPT", "REJECT"),
    ("c1_default_liblist", "control — liblist clause", "ACCEPT", "ACCEPT"),
    ("c2_cell_use_config", "control — cell_clause use_clause", "ACCEPT", "ACCEPT"),
    ("c3_module_param_inst", "control — ordinary #() instantiation", "ACCEPT", "ACCEPT"),
    ("n1_hash_positional_ILLEGAL", "33.4.3 :333 — FORBIDS positional", "REJECT", "REJECT"),
    ("n2_hash_mixed_ILLEGAL", "33.4.3 :333 — FORBIDS positional", "REJECT", "REJECT"),
]


def probe(path: Path, profile: str):
    """-> ('ACCEPT', None) | ('REJECT', furthest) | ('ERROR', text)."""
    r = subprocess.run(
        [str(PROBE), "--parse", "systemverilog", str(path), "--profile", profile],
        capture_output=True, text=True, timeout=120)
    out = r.stdout + r.stderr
    if "parse_full passed" in out:
        return "ACCEPT", None
    m = POS_RE.search(out)
    if m:
        return "REJECT", int(m.group(1))
    return "ERROR", out.strip().splitlines()[0] if out.strip() else "(no output)"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--profile", default="sv_2017")
    args = ap.parse_args()

    if not PROBE.is_file():
        raise SystemExit(f"REFUSE: probe not built at {PROBE.relative_to(ROOT)}")
    # An unknown profile has no LRM expectation here, and printing an unjudged column is
    # precisely the failure `.3.20` fixed. Refuse instead of reporting a green run.
    if args.profile not in KNOWN_PROFILES:
        raise SystemExit(f"REFUSE: no per-case expectation declared for profile "
                         f"'{args.profile}' (known: {', '.join(KNOWN_PROFILES)})")

    print(f"# config use-clause repro matrix — profile {args.profile}")
    print(f"{'case':<30} {'verdict':<8} {'furthest':<9} {'want':<7} LRM")
    mismatched = 0
    for stem, cite, want_sv, want_v2005 in CASES:
        expected = want_sv if args.profile in SV else want_v2005
        verdict, detail = probe(REPRO / f"{stem}.sv", args.profile)
        flag = ""
        if verdict != expected:
            flag = "  <-- differs from post-fix expectation"
            mismatched += 1
        print(f"{stem:<30} {verdict:<8} "
              f"{str(detail if detail is not None else '-'):<9} {expected:<7} {cite}{flag}")
    print(f"\ncases differing from the post-fix expectation: {mismatched}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
