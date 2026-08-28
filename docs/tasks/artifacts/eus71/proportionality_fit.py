#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.71 — turn the two scaling axes into the claim they support,
and no wider.

`-0118`(1) retired "proved by proportionality" because it was measured on one shape, and
adding shapes one at a time only ever produces an INDUCTIVE claim that needs an Nth. This
script tests the STRUCTURAL statement instead, which is falsifiable on the data already in
hand:

    deepest_accepted(shape, R) = frame_ceiling(R) / frames_per_level(shape)

If that decomposition holds, then linearity in R for EVERY shape follows from two measured
facts rather than from a sample of shapes:

  (A) frame_ceiling is linear in R with a ~ZERO intercept — fitted here from the frame count
      the guard reports in its own rejection, which no bisection produces. A fixed ceiling is
      the intercept-only model (slope 0), so the fit separates the two hypotheses directly.
  (B) frames_per_level is a property of the SHAPE and does not move with R.

⛔ The fit is reported with its residuals. A two-point line always fits; the claim is that
FOUR points do, and the residual column is what makes that refutable.

Usage: python3 docs/tasks/artifacts/eus71/proportionality_fit.py <frame_readout.json> [<scaling.json> ...]
"""
import json, sys
from collections import defaultdict

# `ast_pipeline/mod.rs` STACK_HEADROOM_RESERVE_DIVISOR: the probe refuses to spend this
# fraction of the reservation, as an implementation margin for the frames between the last
# sample and the true edge. So the reservation-bytes-per-frame reported below is an UPPER
# bound on the real per-frame stack cost, by exactly this factor.
RESERVE_DIVISOR = 16

# ⛔ WITHDRAWN COMPARISON, left here as a warning rather than deleted. An earlier version of
# this script compared the implied cost against `SV-CORPUS-GRAD.8c.3`'s "~2 KB/frame release"
# and concluded the guard over-estimates ~1.6x and refuses with a third of the stack unspent.
# BOTH HALVES WERE WRONG. (1) POPULATION: 8c.3's figure is ~8 MB / 4096 frames from a ulimit
# bisection on a DEEP-PARENTHESIS shape, and that same leaf records ~6 frames/paren under
# verilog_2005 against ~11 under sv_2017 -- so it is neither this shape nor necessarily this
# profile. (2) MECHANISM: `extend_depth_probe` is ITERATIVE. It re-samples at every new
# threshold and refuses only when `used >= budget` or `affordable == 0`, so an over-estimated
# per-frame cost shortens each STRIDE and raises the sample count; it does not leave stack
# unspent. The near-zero intercept and the flat frames/MiB below are what a converging loop
# looks like. Deriving a utilisation number needs the spend INSTRUMENTED at refusal, which
# nothing here does.


def fit(xs, ys):
    """Least-squares y = m*x + b. Kept explicit rather than pulled from a library so the
    residuals below are computed from the same two numbers that are published."""
    n = len(xs)
    mx, my = sum(xs) / n, sum(ys) / n
    sxx = sum((x - mx) ** 2 for x in xs)
    m = sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / sxx
    return m, my - m * mx


# The OLD fixed ceiling, from `.63`(b): the guard fired at exactly 4096 frames, so each
# shape's boundary depth there is a (depth, frames) point measured under an UNRELATED
# regime — a different ceiling mechanism, a different binary, a different slice. It anchors
# the frames-vs-depth line at ~7x shallower depth than anything measured here.
OLD_CEILING_FRAMES = 4096
FIRST_BAD_63B = {"cast_paren": 250, "gen_mux_both_arms": 147}


def cross_oracle(by_shape):
    """Compare this slice's frames axis with `.63`(b) on the SLOPE, not on the quotient.

    frames/level = (a + b*depth)/depth still carries the intercept `a`, so it drifts with
    depth and two regimes measured at different depths disagree even when the per-level
    cost is identical. Fitting frames against depth separates `a` from `b` and compares
    the quantity that is actually a shape property.
    """
    print("\n=== cross-oracle: frames vs DEPTH, anchored on `.63`(b)'s 4096-frame regime ===")
    for shape, rs in sorted(by_shape.items()):
        rs = sorted(rs, key=lambda r: r["first_bad_depth"])
        xs = [r["first_bad_depth"] for r in rs]
        ys = [r["frames"] for r in rs]
        m_new, b_new = fit(xs, ys)
        anchor = FIRST_BAD_63B.get(shape)
        print(f"\n  {shape}")
        print(f"    this slice only ({len(xs)} pts, depth {xs[0]}..{xs[-1]}): "
              f"slope {m_new:.3f} frames/level, intercept {b_new:+.0f} frames")
        if anchor:
            m_all, b_all = fit(xs + [anchor], ys + [OLD_CEILING_FRAMES])
            pred = m_new * anchor + b_new
            print(f"    `.63`(b) anchor: depth {anchor} measured {OLD_CEILING_FRAMES} frames; "
                  f"this slice's line predicts {pred:.0f} ({100*(OLD_CEILING_FRAMES-pred)/pred:+.2f} %)")
            print(f"    with the anchor  ({len(xs)+1} pts, depth {anchor}..{xs[-1]}): "
                  f"slope {m_all:.3f} frames/level, intercept {b_all:+.0f} frames")
            print(f"    ⇒ the naive quotient 4096/{anchor} = {OLD_CEILING_FRAMES/anchor:.3f} "
                  f"differs from the deepest quotient {ys[-1]/xs[-1]:.3f} by "
                  f"{100*(ys[-1]/xs[-1]-OLD_CEILING_FRAMES/anchor)/(OLD_CEILING_FRAMES/anchor):+.2f} %, "
                  f"which is the INTERCEPT showing through a ratio — not a disagreement about slope.")


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    rows = [r for p in sys.argv[1:] for r in json.load(open(p)) if r.get("frames")]
    by_shape = defaultdict(list)
    for r in rows:
        by_shape[r["shape"]].append(r)

    for shape, rs in sorted(by_shape.items()):
        rs.sort(key=lambda r: r["reservation_MiB"])
        xs = [r["reservation_MiB"] for r in rs]
        ys = [r["frames"] for r in rs]
        m, b = fit(xs, ys)
        print(f"\n=== {shape} — axis A: frame ceiling vs reservation ({len(rs)} points) ===")
        print(f"{'MiB':>6} {'frames':>8} {'fitted':>9} {'residual':>9} {'resid %':>8} "
              f"{'frames/MiB':>11} {'frames/level':>13}")
        for r, x, y in zip(rs, xs, ys):
            f = m * x + b
            print(f"{x:>6} {y:>8} {f:>9.1f} {y-f:>9.1f} {100*(y-f)/y:>8.3f} "
                  f"{r['frames_per_MiB']:>11} {r['frames_per_level']:>13}")
        # A ceiling that is a CONSTANT is the slope-0 model: its whole value sits in b.
        print(f"  slope = {m:.2f} frames/MiB  intercept = {b:.1f} frames "
              f"({100*b/(m*xs[-1]+b):+.2f} % of the largest ceiling)")
        print(f"  reservation cost = {1024/m:.3f} KB of RESERVATION per frame "
              f"⇒ real per-frame stack cost <= {1024/m*(1-1/RESERVE_DIVISOR):.3f} KB "
              f"(the 1/{RESERVE_DIVISOR} margin is never spent). ⚠ NOT a measurement of the "
              f"real cost: that needs the spend instrumented at refusal.")
        fpl = [r["frames_per_level"] for r in rs]
        print(f"  axis B: frames/level over the same {len(rs)} reservations = "
              f"{min(fpl)}..{max(fpl)}, spread {100*(max(fpl)-min(fpl))/min(fpl):.3f} % "
              f"⇒ reservation-invariant FOR THIS SHAPE over this range. ⚠ That it holds for an "
              f"arbitrary shape is an empirical claim over shapes, measured here on two.")

    cross_oracle(by_shape)

    if len(by_shape) > 1:
        print("\n=== cross-shape ===")
        for shape, rs in sorted(by_shape.items()):
            fm = [r["frames_per_MiB"] for r in rs]
            print(f"  {shape:>18}: frames/MiB {min(fm)}..{max(fm)} "
                  f"(spread {100*(max(fm)-min(fm))/min(fm):.3f} %)")
        allfm = [r["frames_per_MiB"] for rs in by_shape.values() for r in rs]
        print(f"  ⇒ ACROSS shapes the frame ceiling spreads "
              f"{100*(max(allfm)-min(allfm))/min(allfm):.2f} % — near-, not exactly, "
              f"shape-independent: different rule mixes have different average frame sizes.")


if __name__ == "__main__":
    main()
