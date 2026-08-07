#!/usr/bin/env python3
"""SV-EXH-PROOF.7.4.6.15 — the WHOLE-SUMMARY diff between two closed-loop replay arms.

The residual ratchet reads ONE number (`targets`). A generator change that keeps that number and
still moves coverage would pass the ratchet and be wrong, so every A/B in this sub-tree also diffs
the full `summary` block and all four debt lists. This script is that comparison, written down once
instead of re-typed per leaf.

⛔ Debt rows are compared as SETS OF IDENTITIES, not as JSON blobs: a row carries per-run counters
(`selected_hits`, `failure_reasons`) that move with any generation change without the debt itself
moving. `resolved` = in A and not in B; `new` = in B and not in A. `new` is the number that must be
zero — it is the only direction that loses coverage.

Usage (from the repository root):
  python3 docs/tasks/artifacts/sv_exh_proof/whole_summary_diff.py \\
      rust/target/<arm_a>/profile_2017_replay_gap.json \\
      rust/target/<arm_b>/profile_2017_replay_gap.json
Exit code: 0 when nothing is NEW in either arm's debt lists, 1 otherwise.
"""
import json
import sys

DEBT_LISTS = (
    "reachable_rule_debt",
    "unreachable_rule_debt",
    "reachable_branch_debt",
    "unreachable_branch_debt",
)


def identity(row):
    """The row's IDENTITY — what makes it a distinct piece of debt, minus per-run counters."""
    for key in ("branch_id", "rule_name", "id", "name"):
        if key in row:
            return f"{key}={row[key]}"
    return json.dumps(row, sort_keys=True)


def main(argv):
    if len(argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    a = json.load(open(argv[1]))
    b = json.load(open(argv[2]))

    print(f"A = {argv[1]}\nB = {argv[2]}\n")

    print("summary:")
    keys = sorted(set(a["summary"]) | set(b["summary"]))
    width = max(len(k) for k in keys)
    for key in keys:
        va, vb = a["summary"].get(key), b["summary"].get(key)
        flag = "  " if va == vb else "->"
        print(f"  {flag} {key:<{width}}  {va}  {vb}")

    print("\ndebt lists (identity sets):")
    failed = False
    for name in DEBT_LISTS:
        ida = {identity(r) for r in a.get(name, [])}
        idb = {identity(r) for r in b.get(name, [])}
        resolved, new = sorted(ida - idb), sorted(idb - ida)
        print(f"  {name}: A={len(ida)} B={len(idb)} resolved={len(resolved)} new={len(new)}")
        for row in resolved:
            print(f"      resolved  {row}")
        for row in new:
            print(f"      NEW       {row}")
        if new:
            failed = True

    ta, tb = len(a.get("targets", [])), len(b.get("targets", []))
    print(f"\nresidual targets: A={ta} B={tb}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
