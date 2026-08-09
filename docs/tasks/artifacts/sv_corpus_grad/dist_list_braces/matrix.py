#!/usr/bin/env python3
"""The `dist { dist_list }` repro matrix (`SV-CORPUS-GRAD.3.18`) — one command, both lanes.

Every `d*` case is copied VERBATIM from the IEEE 1800-2017 normative text (the line numbers
are cited in the leaf), so "expected ACCEPT" is a clause reading and not a preference. Every
`c*` case is a regression tripwire that already parsed before the fix. The single `n*` case
is the LRM-ILLEGAL spelling the pre-fix grammar wrongly accepted, so it is the STRICTNESS
control ([[feedback_sv_strict_lrm_compliance_default]]): it must flip ACCEPT -> REJECT.

⭐ `d4` is why this matrix dumps an AST rather than only an exit code. `soft x dist {5, 8};`
PARSES before the fix — but as ONE dist_item whose value is the concatenation expression
`{5, 8}`, not as the TWO items the LRM defines. A pass/fail probe reports that row as
healthy; only the emitted `"kind"` shows the corruption. Hence `--check-d4-shape`, which
names the kind found at the dist item's value.

Usage (from anywhere; paths resolve against the repository root, directive 12):
  python3 docs/tasks/artifacts/sv_corpus_grad/dist_list_braces/matrix.py
  python3 docs/tasks/artifacts/sv_corpus_grad/dist_list_braces/matrix.py --profile verilog_2005
"""

import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _repo_root import repo_root  # noqa: E402

ROOT = repo_root()
PROBE = ROOT / "rust/target/release/parseability_probe"
REPRO = Path(__file__).resolve().parent / "repro"
POS_RE = re.compile(r"furthest_position=(\d+)")

# (file stem, LRM citation, expected verdict AFTER the fix under sv_2017)
CASES = [
    ("d1_dist_weighted", "§18.5.4 :503", "ACCEPT"),
    ("d2_dist_range_eq", "§18.5.4 :518", "ACCEPT"),
    ("d3_dist_range_prop", "§18.5.4 :520", "ACCEPT"),
    ("d4_dist_unweighted_soft", "§18.5.11 :1406", "ACCEPT"),
    ("d5_dist_in_property", "A.2.10 expression_or_dist", "ACCEPT"),
    ("c1_constraint_no_dist", "control — no dist", "ACCEPT"),
    ("c2_concat_expression", "control — real concatenation", "ACCEPT"),
    ("n1_dist_no_braces_ILLEGAL", "A.2.10 — NO such production", "REJECT"),
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


def d4_value_kind(profile: str):
    """The `kind` the parser gives the VALUE of d4's single/first dist item.

    `concat` == the mis-parse (one item holding `{5, 8}`); `number` == the LRM reading
    (two items, the first being `5`). Returns None when d4 does not parse at all.
    """
    src = REPRO / "d4_dist_unweighted_soft.sv"
    with tempfile.TemporaryDirectory(dir=ROOT / "rust/target") as td:
        out = Path(td) / "d4_ast.json"
        r = subprocess.run(
            [str(PROBE), "--parse-dump-ast-pretty", "systemverilog", str(src), str(out),
             "--profile", profile], capture_output=True, text=True, timeout=120)
        if "parse_full passed" not in (r.stdout + r.stderr) or not out.is_file():
            return None
        tree = json.loads(out.read_text())

    def find(node, key):
        """Breadth-first, so the SHALLOWEST match wins.

        ⛔ Depth-first is wrong here and quietly so: the mis-parse's discriminator
        (`"kind": "concat"`) sits in the SAME dict as the `"body"` list holding the two
        operands, and each operand carries its own `"kind": "number"`. A depth-first walk
        descends into `body` first and returns `number` — i.e. it reports the LRM-correct
        answer on the corrupted tree, which is the one wrong answer this check exists to
        rule out.
        """
        queue = [node]
        while queue:
            nxt = []
            for cur in queue:
                if isinstance(cur, dict):
                    if key in cur:
                        yield cur[key]
                    nxt.extend(cur.values())
                elif isinstance(cur, list):
                    nxt.extend(cur)
            queue = nxt

    for dist in find(tree, "dist"):
        for value in find(dist, "value"):
            # Only the two kinds that DISCRIMINATE. The wrapper kinds on the way down
            # (`sv_2017`, `expression`, `base`, `operand_chain`, `primary`) are identical
            # in both readings, so matching them would answer a different question.
            for kind in find(value, "kind"):
                if kind in ("concat", "number"):
                    return kind
        return "(no value node)"
    return "(no dist node)"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--profile", default="sv_2017")
    ap.add_argument("--check-d4-shape", action="store_true", default=True)
    args = ap.parse_args()

    if not PROBE.is_file():
        raise SystemExit(f"REFUSE: probe not built at {PROBE.relative_to(ROOT)}")

    print(f"# dist-list repro matrix — profile {args.profile}")
    print(f"{'case':<28} {'verdict':<8} {'furthest':<9} LRM")
    for stem, cite, _expected in CASES:
        verdict, detail = probe(REPRO / f"{stem}.sv", args.profile)
        print(f"{stem:<28} {verdict:<8} {str(detail if detail is not None else '-'):<9} {cite}")

    if args.check_d4_shape:
        kind = d4_value_kind(args.profile)
        print(f"\nd4 dist-item value kind: {kind}   "
              f"({'MIS-PARSE — LRM says two items' if kind == 'concat' else 'per LRM'})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
