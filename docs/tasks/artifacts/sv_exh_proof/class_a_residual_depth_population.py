#!/usr/bin/env python3
"""SV-EXH-PROOF.7.4.6.9 — the WHOLE residual population against the depth model.

Extends `min_derivation_depth.py` from one rule to EVERY branch target in the
canonical run's residual, both profiles.  Same engine-mirrored metrics; same
refuse-on-control-miss discipline.

THE CLAIM UNDER TEST (one-directional, which is what the model actually supports):

    purdom_depth(B) > max_depth  =>  construct_mode cannot generate B from B's own
    rule, because it commits to the min-TERMINAL-LENGTH derivation with no fallback
    (:10087) and that derivation is deeper than the budget.

The search fallback (`construct_fell_back_to_search`) does NOT commit — it keeps every
sibling — so it can still rescue a branch the model predicts construct_mode loses.
The controls are therefore stated against the observable the model really predicts:
the branch's own recorded `depth exceeded max_depth=40` failure, read from the run's
coverage artifact (per-branch `failure_reasons`, untruncated).

  NEGATIVE control  every residual branch has purdom_depth > max_depth
  POSITIVE control  every covered branch of those same rules that NEVER recorded a
                    `max_depth=40` failure has purdom_depth <= max_depth
  REPORTED, not hidden: covered branches that DID record such a failure and have
                    purdom_depth > max_depth — predicted to lose construct_mode and
                    demonstrably rescued by the search fallback.  Consistent with the
                    model, listed by name so the exception is never silent.

Verdict per residual branch:
  BUDGET      purdom_depth > max_depth >= min_depth — a shallower derivation exists
  STRUCTURAL  both exceed max_depth — no derivation of any shape fits
"""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from class_a_min_derivation_depth import (  # noqa: E402
    GEN_AST,
    WITNESS_MAX_DEPTH,
    compute_min_terminal_lengths,
    compute_rule_depths,
    forced_branch_depth,
    fmt,
)

ROOT = Path(__file__).resolve().parents[4]
GATE_WORK = ROOT / "rust/target/sv_stimuli_quality_gate/work"
DEPTH_SIGNATURE = f"depth exceeded max_depth={WITNESS_MAX_DEPTH} "

# The class-C store-gate pair is a DIFFERENT mechanism (.7.4.6.12) — excluded by name
# from the depth population, and named here so the exclusion is visible, not silent.
CLASS_C_RULES = {"net_declaration_sv_2017", "net_declaration_sv_2023"}


def load_profile(profile):
    with (GATE_WORK / f"profile_{profile}_replay_gap.json").open() as handle:
        report = json.load(handle)
    with (GATE_WORK / f"profile_{profile}_replay_coverage.json").open() as handle:
        coverage = json.load(handle)
    residual = {}
    for debt in report["reachable_branch_debt"]:
        residual.setdefault(debt["rule_name"], set()).add(debt["branch_index"])
    return residual, coverage["branch_groups"]


def recorded_depth_failure(group, index):
    reasons = group.get("failure_reasons") or []
    if index >= len(reasons):
        return False
    return any(DEPTH_SIGNATURE in reason for reason in reasons[index])


def main():
    with GEN_AST.open() as handle:
        tree = json.load(handle)["grammar_tree"]
    lengths = compute_min_terminal_lengths(tree)
    purdom_depths = compute_rule_depths(tree, lengths, purdom=True)
    min_depths = compute_rule_depths(tree, lengths, purdom=False)

    def depth_pair(rule, index):
        return (
            forced_branch_depth(tree, purdom_depths, lengths, True, rule, index),
            forced_branch_depth(tree, min_depths, lengths, False, rule, index),
        )

    rows, clean_controls, rescued = [], [], []
    for profile in ("2017", "2023"):
        residual, groups = load_profile(profile)
        for rule, indices in sorted(residual.items()):
            if rule in CLASS_C_RULES or "Or" not in tree.get(rule, {}):
                continue
            group = groups.get(f"{rule}::root", {})
            for index in sorted(indices):
                rows.append((profile, rule, index) + depth_pair(rule, index))
            for index in range(len(tree[rule]["Or"]["alternatives"])):
                if index in indices:
                    continue
                entry = (profile, rule, index) + depth_pair(rule, index)
                if recorded_depth_failure(group, index):
                    rescued.append(entry)
                else:
                    clean_controls.append(entry)

    failures = []
    for profile, rule, index, purdom, _ in rows:
        if purdom <= WITNESS_MAX_DEPTH:
            failures.append(
                f"NEGATIVE CONTROL MISS: residual {rule}#{index} ({profile}) has "
                f"purdom_depth={fmt(purdom)} <= {WITNESS_MAX_DEPTH}"
            )
    for profile, rule, index, purdom, _ in clean_controls:
        if purdom > WITNESS_MAX_DEPTH:
            failures.append(
                f"POSITIVE CONTROL MISS: covered {rule}#{index} ({profile}) never recorded a "
                f"max_depth={WITNESS_MAX_DEPTH} failure yet has purdom_depth={fmt(purdom)}"
            )
    if failures:
        print("RESIDUAL-DEPTH-POPULATION: REFUSING — ground-truth controls failed", file=sys.stderr)
        for line in failures[:40]:
            print("  " + line, file=sys.stderr)
        print(f"  ({len(failures)} control misses total)", file=sys.stderr)
        return 2

    budget = sum(1 for *_, purdom, minimum in rows if minimum <= WITNESS_MAX_DEPTH < purdom)
    print(
        f"RESIDUAL-DEPTH-POPULATION: max_depth={WITNESS_MAX_DEPTH} residual_branches={len(rows)} "
        f"controls=PASS (negative {len(rows)}/{len(rows)}, positive "
        f"{len(clean_controls)}/{len(clean_controls)}) BUDGET={budget} "
        f"STRUCTURAL={len(rows) - budget} search_rescued={len(rescued)}"
    )
    print("\nprofile  rule#branch                                       purdom  min   class")
    for profile, rule, index, purdom, minimum in rows:
        verdict = "BUDGET" if minimum <= WITNESS_MAX_DEPTH < purdom else "STRUCTURAL"
        print(
            f"{profile:<8} {rule}#{index}".ljust(56)
            + f"{fmt(purdom):>6}  {fmt(minimum):>4}  {verdict}"
        )
    if rescued:
        print(
            "\nCOVERED but predicted to lose construct_mode — rescued by the search fallback "
            "(model-consistent, listed so the exception is never silent):"
        )
        for profile, rule, index, purdom, minimum in rescued:
            print(
                f"  {profile}  {rule}#{index}".ljust(56)
                + f"purdom={fmt(purdom)} min={fmt(minimum)}"
            )
    return 0


if __name__ == "__main__":
    sys.exit(main())
