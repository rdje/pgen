#!/usr/bin/env python3
"""SV-EXH-PROOF.7.4.6.13 defect (ii) — price the DECLARED CEILING on the depth-slack ladder.

Sibling of `depth_slack_cap_pricing.py`, which prices a NESTING cap in levels. This one prices
the knob the fix actually declares: a ceiling at a stated MULTIPLE of the configured
`--max-depth`, which is what `DEPTH_SLACK_RETRY_CEILING_MULTIPLE` is set from.

⭐ WHY THE TWO ARE THE SAME KNOB, verified rather than assumed. At nesting level `L` the retry's
budget is exactly `configured + SLACK * L`, because the retry adds its slack to the LIVE budget
and restores it on the way out — so a budget ceiling `C` is the nesting cap
`L_max = (C - configured) // SLACK`. That identity is this instrument's POSITIVE CONTROL: it is
checked at every level of every census line and the script REFUSES (exit 2) on a miss, because a
ladder that is not arithmetic would make every multiple below meaningless.

⚠️ HONEST BOUND, stated up front and inherited from the sibling: refusing a rung changes
generation downstream, so the attempt counts at levels <= the cap will NOT stay what they are
here. This curve prices CANDIDATES; only the A/B run measures the outcome. Both prior candidates
for this defect were refuted by exactly that gap between static pricing and a measured arm.

Usage (from the repository root):
  python3 docs/tasks/artifacts/sv_exh_proof/depth_slack_ceiling_pricing.py \
      rust/target/sv_exh_proof_7_4_6_13_explicit/stage_2017.log \
      rust/target/sv_exh_proof_7_4_6_13_explicit/stage_2023.log
"""
import re
import sys

LEVEL_RE = re.compile(r"L(\d+):(\d+)/(\d+)@(\d+)")
CENSUS_RE = re.compile(r"Depth-slack retry census: .*")
# Mirrors `TARGET_BRANCH_DEPTH_RETRY_SLACK` in rust/src/ast_pipeline/stimuli_generator.rs.
SLACK = 4
MULTIPLES = range(5, 41)


def census_line(path):
    """The LAST census line in the log (one per stage run)."""
    found = None
    with open(path, encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            match = CENSUS_RE.search(raw)
            if match:
                found = match.group(0).strip()
    return found


def levels_of(line, path):
    """Parse the per-level table, PROVING the arithmetic-ladder identity as we go."""
    levels = [(int(l), int(s), int(a), int(b)) for l, s, a, b in LEVEL_RE.findall(line)]
    if not levels:
        raise SystemExit(f"REFUSED {path}: census line carries no per-level table")
    # The configured depth is not in the log, so recover it from level 1 and then REQUIRE the
    # identity at every other level. This is the positive control, not a convenience.
    configured = levels[0][3] - SLACK
    for level, _successes, _attempts, budget in levels:
        expected = configured + SLACK * level
        if budget != expected:
            raise SystemExit(
                f"REFUSED {path}: the ladder is not arithmetic at level {level} "
                f"(budget {budget}, expected {expected} for configured={configured}). "
                "A ceiling multiple cannot be priced off a non-arithmetic ladder."
            )
    return configured, levels


def price(path):
    line = census_line(path)
    if line is None:
        print(f"REFUSED {path}: no census line (the instrument did not fire)")
        return 1, None
    configured, levels = levels_of(line, path)
    total_attempts = sum(a for _, _, a, _ in levels)
    total_successes = sum(s for _, s, _, _ in levels)
    if total_successes == 0:
        print(f"REFUSED {path}: the ladder bought nothing, so no multiple can be priced")
        return 2, None
    deepest_paying = max(l for l, s, _, _ in levels if s > 0)

    print(f"== {path}")
    print(
        f"   configured={configured} levels={len(levels)} attempts={total_attempts} "
        f"successes={total_successes} max_budget={levels[-1][3]} "
        f"({levels[-1][3] / configured:.1f}x) deepest_paying_level={deepest_paying} "
        f"deepest_paying_budget={configured + SLACK * deepest_paying} "
        f"({(configured + SLACK * deepest_paying) / configured:.1f}x)"
    )
    print(f"   {'k':>3} {'ceiling':>8} {'Lmax':>5} {'refused att':>12} {'%':>7} {'succ lost':>10} {'%':>7}")
    tightest_free = None
    for k in MULTIPLES:
        ceiling = k * configured
        lmax = (ceiling - configured) // SLACK
        refused = sum(a for l, _, a, _ in levels if l > lmax)
        lost = sum(s for l, s, _, _ in levels if l > lmax)
        if lost == 0 and tightest_free is None:
            tightest_free = k
        # Print the informative band only: every multiple that still costs something, plus the
        # first two that do not — a table of zeroes prices nothing.
        if lost > 0 or (tightest_free is not None and k <= tightest_free + 1):
            print(
                f"   {k:>3} {ceiling:>8} {lmax:>5} {refused:>12} "
                f"{100.0 * refused / total_attempts:>6.2f}% {lost:>10} "
                f"{100.0 * lost / total_successes:>6.2f}%"
            )
    print(f"   ⇒ tightest zero-cost multiple for this profile: {tightest_free}x")
    return 0, tightest_free


if __name__ == "__main__":
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    status = 0
    tightest = []
    for argument in sys.argv[1:]:
        code, free = price(argument)
        status |= code
        if free is not None:
            tightest.append(free)
        print()
    if tightest:
        # The shipped constant must clear EVERY profile priced, so the answer is the max.
        print(f"⇒ DECLARED CEILING MULTIPLE across {len(tightest)} profile(s): {max(tightest)}x")
    sys.exit(status)
