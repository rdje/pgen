#!/usr/bin/env python3
"""Rebaseline `systemverilog_recognized_cert_union_contract.json` FROM THE GATE'S OWN summary.json.

WHY THIS EXISTS
---------------
`sv_cert_recognized_union_gate` refuses to be re-stamped green by hand, and it is right to:
a stamp asserts the expectations were RE-DERIVED and matched, which is exactly what has not
happened when the gate is red. But the legitimate case — a grammar change that ADDS rules, so the
totals move and nothing else does — still needs the `expected_*` fields updated, and the failure
mode there is a human RE-TYPING a number they read in a log.

`SV-CORPUS-GRAD.13e.5` recorded that its numbers were *"parsed out of the gates' own logs by
scripts that REFUSE on seed disagreement or a nonzero failure counter — never re-typed"*. No such
script was tracked, so the next leaf (`.13e.4`, `PGEN-SV-CORPUS-GRAD-0288`) had to write one. This
is it, tracked so the third leaf does not write a third.

⛔ IT REFUSES RATHER THAN REPORTS. Every guard below exits non-zero and changes nothing:

  1. the three seeds disagree on ANY quantity          -> a non-deterministic baseline is not a baseline
  2. any seed's `sample_parse_failures` is nonzero     -> the witnesses are not trustworthy
  3. canonical UNKNOWN moved                           -> a rule fell OUT of covered; adjudicate, don't stamp
  4. union UNKNOWN moved, or the residual set changed  -> `fully_certified_via_union` is the whole claim
  5. either accounting fails to close
     (proof + witness + unknown != total)              -> the numbers are internally inconsistent
  6. `--expect-delta N` given and total moved by != N  -> you must SAY what you expect to move

Guards 3 and 4 are the load-bearing ones: they are precisely the two quantities a
"just make it green" edit would silently take with it.

⭐ It does NOT stamp. After it writes, RE-RUN the gate — a green run stamps its own identity block,
which is the only signature that means the numbers were re-derived against this tree.

Usage:
    python3 docs/tasks/artifacts/sv_corpus_grad/cert_union_rebaseline/rebaseline_cert_union_from_summary.py \\
        --expect-delta 1 --note "SV-CORPUS-GRAD.13e.4 (...): why the total moved and why none of it is a defect"
    # then:
    make -C rust SHELL=/bin/bash sv_cert_recognized_union_gate
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

# docs/tasks/artifacts/sv_corpus_grad/cert_union_rebaseline/<this file> -> five levels to the root.
# ⛔ Measured, not counted: the first cut said parents[4] and resolved to `docs/`, so the tool
# REFUSED with "no gate summary" instead of silently reading the wrong tree. Asserted below.
ROOT = Path(__file__).resolve().parents[5]
SUMMARY = ROOT / "rust/target/sv_cert_recognized_union_gate/summary.json"
CONTRACT = ROOT / "rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json"
assert (ROOT / "grammars/systemverilog.ebnf").is_file(), f"ROOT resolved wrong: {ROOT}"


def _stamper_style(text: str) -> str:
    """Re-compact the `{ "kind": …, "digest": … }` input entries onto one line.

    ⛔ NOT cosmetic. `identity` is written and re-written by
    `scripts/check_baseline_identity.sh --stamp`, which emits that entry inline. A plain
    `json.dumps(indent=2)` expands it to four lines, so this tool would show four spurious diff
    lines in a block it has no business touching — and a reviewer reading a rebaseline commit would
    see the IDENTITY block move. Matching the stamper's style keeps the diff to the fields this
    tool actually owns.
    """
    return re.sub(
        r'\{\n\s*"kind": ("(?:[^"\\]|\\.)*"),\n\s*"digest": ("(?:[^"\\]|\\.)*")\n(\s*)\}',
        lambda m: '{ "kind": %s, "digest": %s }' % (m.group(1), m.group(2)),
        text,
    )


def refuse(msg: str) -> "None":
    print(f"cert-union-rebaseline: REFUSED — {msg}", file=sys.stderr)
    raise SystemExit(2)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--summary", default=SUMMARY, type=Path)
    ap.add_argument("--contract", default=CONTRACT, type=Path)
    ap.add_argument("--expect-delta", type=int, required=True,
                    help="how many rules you expect `total` to move by (may be 0 or negative)")
    ap.add_argument("--note", required=True, help="prepended to `rebaseline_note`; say WHY it moved")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    if not args.summary.is_file():
        refuse(f"no gate summary at {args.summary} — run the gate first, this tool never measures")
    summary = json.loads(args.summary.read_text(encoding="utf-8"))
    contract = json.loads(args.contract.read_text(encoding="utf-8"))

    per_seed = summary.get("per_seed") or []
    if len(per_seed) < 2:
        refuse(f"summary carries {len(per_seed)} seed(s); determinism cannot be checked from one")

    # --- guard 1 + 2: determinism, and witnesses that actually parsed -------------------------
    shape = [(s["canonical"]["total"], s["canonical"]["proof"], s["canonical"]["witness"],
              s["canonical"]["unknown"], s["union"]["witness"], s["union"]["unknown"],
              tuple(sorted(s["union"]["residual_rules"]))) for s in per_seed]
    if len(set(shape)) != 1:
        refuse("the seeds DISAGREE — " + " | ".join(
            f"seed={s['seed']}:{sh}" for s, sh in zip(per_seed, shape)))
    for s in per_seed:
        spf = s["canonical"].get("sample_parse_failures")
        if spf:
            refuse(f"seed={s['seed']} has sample_parse_failures={spf}; witnesses are not trustworthy")

    total, proof, witness, unknown, u_witness, u_unknown, residual = shape[0]

    # --- guard 5: both accountings must close -------------------------------------------------
    if proof + witness + unknown != total:
        refuse(f"canonical accounting does not close: {proof}+{witness}+{unknown} != {total}")
    if proof + u_witness + u_unknown != total:
        refuse(f"union accounting does not close: {proof}+{u_witness}+{u_unknown} != {total}")

    # --- guard 3 + 4: the two quantities a 'make it green' edit would take with it -------------
    if unknown != contract["expected_canonical_unknown"]:
        refuse(f"canonical UNKNOWN moved {contract['expected_canonical_unknown']} -> {unknown}. "
               "A rule left the covered set; that is an adjudication, not a rebaseline.")
    if u_unknown != contract["expected_union_unknown"]:
        refuse(f"union UNKNOWN moved {contract['expected_union_unknown']} -> {u_unknown}. "
               "`fully_certified_via_union` is the whole claim; adjudicate it.")
    if sorted(residual) != sorted(contract["expected_union_residual_rules"]):
        refuse(f"union residual set changed {contract['expected_union_residual_rules']} -> {list(residual)}")

    # --- guard 6: you must say what you expect to move -----------------------------------------
    delta = total - contract["expected_total"]
    if delta != args.expect_delta:
        refuse(f"total moved by {delta} ({contract['expected_total']} -> {total}) but "
               f"--expect-delta said {args.expect_delta}")

    before = {k: contract[k] for k in ("expected_total", "expected_proof",
                                       "expected_canonical_witness", "expected_union_witness")}
    contract["expected_total"] = total
    contract["expected_proof"] = proof
    contract["expected_canonical_witness"] = witness
    contract["expected_union_witness"] = u_witness
    moves = ", ".join(f"{k} {before[k]} -> {contract[k]}" for k in before if before[k] != contract[k])
    contract["rebaseline_note"] = (
        f"{args.note} — DERIVED, never re-typed: parsed from the gate's own summary.json by "
        f"docs/tasks/artifacts/sv_corpus_grad/cert_union_rebaseline/rebaseline_cert_union_from_summary.py, "
        f"which REFUSES on seed disagreement, a nonzero sample_parse_failures, a moved canonical or union "
        f"UNKNOWN, a changed union residual, an accounting that does not close, or a total delta other than "
        f"the declared --expect-delta {args.expect_delta}. Moves: {moves}; "
        f"expected_canonical_unknown UNCHANGED at {unknown}, expected_union_unknown UNCHANGED at {u_unknown}, "
        f"expected_union_residual_rules UNCHANGED at {list(residual)}. "
        f"--- Prior note retained: {contract['rebaseline_note']}")

    if args.dry_run:
        print(f"cert-union-rebaseline: DRY RUN — would write {moves}")
        return 0
    args.contract.write_text(_stamper_style(json.dumps(contract, indent=2, ensure_ascii=False)) + "\n",
                             encoding="utf-8")
    print(f"cert-union-rebaseline: wrote {args.contract.relative_to(ROOT)} — {moves}")
    print("cert-union-rebaseline: now RE-RUN the gate; a green run stamps its own identity block, "
          "which is the only signature meaning the numbers were re-derived against this tree.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
