#!/usr/bin/env python3
"""SV-EXH-PROOF.7.4.6.19 — is the replay stage replaying a LIVE universe, or a STALE one?

⭐ WHY THIS EXISTS.  `run_closed_loop_replay_stage.sh` is the cheap, faithful oracle every
generator A/B in this tree has used since `-0172`.  It is faithful *for the replay stage* —
but it does not GENERATE the universe it replays.  Two inputs are read verbatim out of the
canonical gate's work directory and are whatever the LAST canonical gate run happened to
leave there:

    rust/target/sv_stimuli_quality_gate/work/systemverilog_gen_ast.json
    rust/target/sv_stimuli_quality_gate/work/profile_<P>_initial_gap.json

So a residual of 0 measured on the stage is a statement about *that frozen universe*.  If a
generator change moves the INITIAL stage — the diverse pass that writes the gap report — the
stage keeps replaying the old target set and keeps reporting the old number, and nothing
refuses.  This script answers the question directly: are the two universes the same?

⛔ IT IS A COMPARATOR, NOT A GATE.  It never regenerates anything and never decides whether a
difference is acceptable.  It reports, at three sharpening levels, so a difference can be read
as what it actually is rather than as a scare:

  1. BYTES        — `cmp`-equivalence of the whole artifact.
  2. UNIVERSE     — the SET of replay target ids, which is the only part of the gap report the
                    replay stage consumes as targets.  This is the load-bearing comparison: a
                    serialization-ordering change (e.g. `-0180`'s sorted emitters) moves bytes
                    and leaves the universe untouched, and those two outcomes must never be
                    reported as the same thing.
  3. SUMMARY      — the per-profile coverage counters, field by field, so a universe that IS
                    equal can still surface a moved denominator.

⭐ GROUND TRUTH — it refuses rather than guesses ([[feedback_instrument_needs_ground_truth]]).
Before publishing any verdict it runs a POSITIVE control (an artifact compared with itself
must come out identical at all three levels) and a NEGATIVE control (a planted mutation —
one target dropped, one summary counter moved — must be caught at exactly the two levels that
can see it, and must NOT be reported at the level that cannot).  A miss on either aborts with
exit 2 before a number is published: a comparator that cannot see a planted difference is
reporting a broken instrument, not a finding.

Usage (from the repository root):
    python3 docs/tasks/artifacts/sv_exh_proof/replay_universe_staleness_diff.py \
        rust/target/sv_exh_proof_readjudication/baseline_0170 \
        rust/target/sv_stimuli_quality_gate/work

Exit codes:  0 = every compared universe is IDENTICAL · 1 = at least one DIFFERS
             2 = a ground-truth control missed (nothing published) · 3 = usage/IO error
"""

from __future__ import annotations

import copy
import hashlib
import json
import sys
from pathlib import Path

PROFILES = ("2017", "2023")
GEN_AST = "systemverilog_gen_ast.json"


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _target_ids(report: dict) -> list[str]:
    """The replay stage's target universe: the `targets` list's ids, order-insensitively."""
    return sorted(str(t.get("id", "")) for t in report.get("targets", []))


def compare_reports(baseline: dict, fresh: dict) -> dict:
    """The three-level comparison, as data (no printing, so the controls can assert on it)."""
    base_ids, fresh_ids = _target_ids(baseline), _target_ids(fresh)
    base_set, fresh_set = set(base_ids), set(fresh_ids)

    base_summary = baseline.get("summary", {}) or {}
    fresh_summary = fresh.get("summary", {}) or {}
    summary_moved = {
        key: (base_summary.get(key), fresh_summary.get(key))
        for key in sorted(set(base_summary) | set(fresh_summary))
        if base_summary.get(key) != fresh_summary.get(key)
    }

    return {
        "universe_equal": base_set == fresh_set,
        "universe_size": (len(base_ids), len(fresh_ids)),
        "targets_only_in_baseline": sorted(base_set - fresh_set),
        "targets_only_in_fresh": sorted(fresh_set - base_set),
        "summary_moved": summary_moved,
    }


def _ground_truth_or_refuse(sample: dict) -> None:
    """Positive + negative controls.  Abort (exit 2) before publishing on any miss."""
    # POSITIVE — an artifact against itself must be identical at every level.
    positive = compare_reports(sample, copy.deepcopy(sample))
    if not positive["universe_equal"] or positive["summary_moved"]:
        print("REFUSING: positive control MISSED — the comparator reports a difference "
              "between an artifact and itself.", file=sys.stderr)
        raise SystemExit(2)

    # NEGATIVE — a planted mutation must be caught at exactly the levels that can see it.
    mutated = copy.deepcopy(sample)
    if not mutated.get("targets"):
        print("REFUSING: the control sample carries no targets, so the negative control "
              "cannot be planted.", file=sys.stderr)
        raise SystemExit(2)
    dropped = str(mutated["targets"].pop()["id"])
    mutated.setdefault("summary", {})
    counter = "reachable_branches" if "reachable_branches" in mutated["summary"] else None
    if counter is None:
        print("REFUSING: the control sample carries no `reachable_branches` counter, so the "
              "summary half of the negative control cannot be planted.", file=sys.stderr)
        raise SystemExit(2)
    original = mutated["summary"][counter]
    mutated["summary"][counter] = original + 1

    negative = compare_reports(sample, mutated)
    if negative["universe_equal"]:
        print("REFUSING: negative control MISSED — a dropped target was not seen at the "
              "UNIVERSE level.", file=sys.stderr)
        raise SystemExit(2)
    if negative["targets_only_in_baseline"] != [dropped]:
        print(f"REFUSING: negative control IMPRECISE — expected exactly ['{dropped}'] missing, "
              f"got {negative['targets_only_in_baseline']}.", file=sys.stderr)
        raise SystemExit(2)
    if negative["summary_moved"].get(counter) != (original, original + 1):
        print("REFUSING: negative control MISSED — a moved summary counter was not seen at "
              "the SUMMARY level.", file=sys.stderr)
        raise SystemExit(2)

    print(f"[ground-truth] positive 3/3 levels identical · negative caught the dropped target "
          f"'{dropped}' and the moved counter '{counter}' {original}->{original + 1} — "
          f"instrument TRUSTED")


def main(argv: list[str]) -> int:
    if len(argv) < 3:
        print(__doc__, file=sys.stderr)
        return 3
    baseline_dir, fresh_dir = Path(argv[1]), Path(argv[2])
    profiles = argv[3:] or list(PROFILES)

    for directory in (baseline_dir, fresh_dir):
        if not directory.is_dir():
            print(f"error: not a directory: {directory}", file=sys.stderr)
            return 3

    # Ground truth is established on the FIRST readable baseline report, before any verdict.
    control_source = None
    for profile in profiles:
        candidate = baseline_dir / f"profile_{profile}_initial_gap.json"
        if candidate.is_file():
            control_source = json.loads(candidate.read_text())
            break
    if control_source is None:
        print(f"error: no baseline initial-gap report found under {baseline_dir}", file=sys.stderr)
        return 3
    _ground_truth_or_refuse(control_source)

    all_equal = True

    # Level 1 on the generation IR — it has no target list, so bytes are the whole story.
    base_ast, fresh_ast = baseline_dir / GEN_AST, fresh_dir / GEN_AST
    if base_ast.is_file() and fresh_ast.is_file():
        base_hash, fresh_hash = _sha256(base_ast), _sha256(fresh_ast)
        verdict = "IDENTICAL" if base_hash == fresh_hash else "DIFFERS"
        all_equal &= base_hash == fresh_hash
        print(f"\n{GEN_AST}: BYTES {verdict}")
        print(f"  baseline sha256 {base_hash}")
        print(f"  fresh    sha256 {fresh_hash}")
    else:
        print(f"\n{GEN_AST}: SKIPPED (absent on one side)")

    for profile in profiles:
        name = f"profile_{profile}_initial_gap.json"
        base_path, fresh_path = baseline_dir / name, fresh_dir / name
        if not (base_path.is_file() and fresh_path.is_file()):
            print(f"\n{name}: SKIPPED (absent on one side)")
            continue

        base_hash, fresh_hash = _sha256(base_path), _sha256(fresh_path)
        report = compare_reports(json.loads(base_path.read_text()),
                                 json.loads(fresh_path.read_text()))
        base_n, fresh_n = report["universe_size"]

        print(f"\n{name}")
        print(f"  BYTES:    {'IDENTICAL' if base_hash == fresh_hash else 'DIFFER'}"
              f"  ({base_hash[:16]}… vs {fresh_hash[:16]}…)")
        print(f"  UNIVERSE: {'IDENTICAL' if report['universe_equal'] else 'DIFFERS'}"
              f"  (targets {base_n} vs {fresh_n})")
        if not report["universe_equal"]:
            only_base = report["targets_only_in_baseline"]
            only_fresh = report["targets_only_in_fresh"]
            print(f"    only in baseline ({len(only_base)}): {only_base[:20]}"
                  f"{' …' if len(only_base) > 20 else ''}")
            print(f"    only in fresh    ({len(only_fresh)}): {only_fresh[:20]}"
                  f"{' …' if len(only_fresh) > 20 else ''}")
        if report["summary_moved"]:
            print(f"  SUMMARY:  {len(report['summary_moved'])} counter(s) MOVED")
            for key, (was, now) in report["summary_moved"].items():
                print(f"    {key}: {was} -> {now}")
        else:
            print("  SUMMARY:  every counter IDENTICAL")

        all_equal &= report["universe_equal"] and not report["summary_moved"]

    print(f"\nVERDICT: {'the replay universe is LIVE (identical on every compared artifact)' if all_equal else 'the replay universe MOVED — the stage was replaying a STALE universe'}")
    return 0 if all_equal else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
