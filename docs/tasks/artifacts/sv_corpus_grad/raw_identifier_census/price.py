#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2k (d) — PRICE THE FIX BEFORE IT IS WRITTEN, and make the price FALSIFIABLE.

⛔ WHY. `.13c.2j` wrote its fix first and `PARSE-COST-RATCHET` REFUSED it, after the correctness
was already verified — two-thirds of that change's cost bought nothing, and only a per-site
attribution found it. This leaf's (d) says the reading comes FIRST. ⭐ A price is only worth
taking if it can be WRONG, so this prints a numeric PREDICTION for each candidate spelling, which
the post-fix re-measure either confirms or refutes.

THE ECONOMICS, MEASURED NOT ASSUMED. `ast_based_generator.rs` memoizes EVERY rule
(`SV-EXH-PROOF.3.3.4.b.6.2.15`, "memoize ALL rules unconditionally"), and `rule_entry_counts`
counts CALLS while `rule_memo_hit_counts` counts the calls that never ran a body. So the cost of a
negative lookahead depends entirely on WHERE it is spelled:

  * inside `non_keyword_identifier` (today) — that rule takes **0** memo hits, so its guard runs on
    every single call;
  * inside `identifier` — that rule is ~89 % memo HITS, so a guard in its body runs only on the
    misses.

⇒ Moving the exclusion DOWN into `identifier` and reducing `non_keyword_identifier` to an alias is
predicted to make the parser CHEAPER while making it STRICTER. That is a claim, and this script is
where it is written down before the measurement that can kill it.

CANDIDATE SPELLINGS PRICED
  A  targeted   `identifier` -> `non_keyword_identifier` at the hierarchy-component loops only.
                Fixes the three measured rows; leaves the other 43 references of the class open.
                Priced as a BOUND, not a point: per-site call counts are not observable from a
                per-rule dump, so the loops' own entry counts bound it from below.
  B  universal  the exclusion moves into `identifier`'s simple branch and
                `non_keyword_identifier` becomes an alias. Fixes all 46 sites at once.
                ⭐ `escaped_identifier` is deliberately NOT guarded: IEEE 1800-2017 §5.6.1 makes
                `\\module ` a legal identifier distinct from the keyword, and leaving that branch
                unguarded is both correct and one fewer evaluation per miss.

USAGE   python3 docs/tasks/artifacts/sv_corpus_grad/raw_identifier_census/price.py [--jobs N]
EXIT    0 = priced; 2 = it could not measure (missing probe, no dumps)
"""

from __future__ import annotations

import argparse
import concurrent.futures
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
PROBE = ROOT / "rust/target/release/parseability_probe"
SAMPLE = ROOT / "stimuli/sv/parse_cost_sample.tsv"
GRAMMAR_NAME = "systemverilog"
PROFILE = "sv_2017"
TIMEOUT_S = 120

if not (ROOT / "grammars").is_dir():
    raise SystemExit(f"price: not at the repo root (derived {ROOT}) — fix the parents[] depth")

# The rules whose counters carry the whole price. `identifier` and its two branches say how much
# work a guard in `identifier`'s body would do; the `reserved_*` chain says what the guard costs
# today, per call, at the one site that already spells it.
WATCH = (
    "identifier",
    "escaped_identifier",
    "simple_identifier",
    "non_keyword_identifier",
    "reserved_non_keyword_identifier",
    "reserved_non_keyword_identifier_sv",
    "reserved_non_keyword_identifier_v2005",
    "hierarchical_identifier",
    "hierarchical_tf_identifier",
    "split_hierarchical_callable_receiver",
)


def sample_files() -> list[str]:
    rows = []
    for line in SAMPLE.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) >= 2:
            rows.append(parts[1])
    return rows


def measure_one(rel: str) -> dict | None:
    fd, tmp = tempfile.mkstemp(suffix=".json", dir=str(ROOT / "rust" / "target"))
    os.close(fd)
    try:
        subprocess.run(
            [str(PROBE), "--parse", GRAMMAR_NAME, str(ROOT / rel), "--profile", PROFILE,
             "--dump-rule-outcome-counts-json", tmp],
            capture_output=True, timeout=TIMEOUT_S,
        )
        if os.path.getsize(tmp) == 0:
            return None
        with open(tmp, encoding="utf-8") as fh:
            return json.load(fh)
    except (subprocess.TimeoutExpired, json.JSONDecodeError, OSError):
        return None
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--jobs", type=int, default=max(1, (os.cpu_count() or 4) - 2))
    args = ap.parse_args()

    if not PROBE.exists():
        print(f"price: no probe at {PROBE} — build it first", file=sys.stderr)
        return 2

    files = sample_files()
    if not files:
        print("price: EMPTY sample — refusing", file=sys.stderr)
        return 2

    entries: dict[str, int] = dict.fromkeys(WATCH, 0)
    hits: dict[str, int] = dict.fromkeys(WATCH, 0)
    total_entries = total_hits = total_committed = 0
    measured = 0
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as ex:
        for d in ex.map(measure_one, files):
            if d is None:
                continue
            measured += 1
            total_entries += int(d.get("total_entries", 0))
            total_hits += int(d.get("total_memo_hits", 0))
            total_committed += int(d.get("total_committed", 0))
            e, m = d.get("rule_entry_counts", {}), d.get("rule_memo_hit_counts", {})
            for r in WATCH:
                entries[r] += int(e.get(r, 0))
                hits[r] += int(m.get(r, 0))

    if measured == 0:
        print("price: NO file produced a dump — refusing", file=sys.stderr)
        return 2

    print(f"PARSE-COST-PRICE: files={measured}/{len(files)} profile={PROFILE} "
          f"total_entries={total_entries} total_memo_hits={total_hits} "
          f"total_committed={total_committed}")
    print()
    print(f"  {'rule':<40} {'calls':>12} {'memo hits':>12} {'bodies run':>12}")
    for r in WATCH:
        print(f"  {r:<40} {entries[r]:>12,} {hits[r]:>12,} {entries[r] - hits[r]:>12,}")
    print()

    # ── the prediction ──────────────────────────────────────────────────────────────────────────
    ident_bodies = entries["identifier"] - hits["identifier"]
    guard_today = entries["reserved_non_keyword_identifier"]
    guard_chain_today = guard_today + entries["reserved_non_keyword_identifier_sv"] \
        + entries["reserved_non_keyword_identifier_v2005"]

    # Design B adds one `reserved_non_keyword_identifier` call per `identifier` BODY (the simple
    # branch), which itself calls exactly one profile-active child; and removes the whole chain
    # `non_keyword_identifier` spends today, since that rule becomes a bare alias.
    added_b = 2 * ident_bodies
    removed_b = guard_chain_today
    delta_b = added_b - removed_b

    print("── design B (universal: exclusion inside `identifier`, `non_keyword_identifier` an alias)")
    print(f"  guard evaluations today  : {guard_chain_today:>12,}  (chain, all from non_keyword_identifier)")
    print(f"  guard evaluations after  : {added_b:>12,}  (2 per `identifier` body = {ident_bodies:,} bodies)")
    print(f"  PREDICTED delta entries  : {delta_b:>+12,}  ({100.0 * delta_b / total_entries:+.3f} % of total)")
    print(f"  PREDICTED direction      : {'FALL — rebaseline territory' if delta_b < 0 else 'RISE — needs an accepted_rises attribution'}")
    print()

    # Design A cannot be priced to a point from a per-rule dump: the three loop rules' entry counts
    # bound the number of guarded positions from BELOW (each entry runs the loop >= 0 times), so
    # this is reported as a floor and explicitly not as an estimate.
    loop_entries = (entries["hierarchical_identifier"] + entries["hierarchical_tf_identifier"]
                    + entries["split_hierarchical_callable_receiver"])
    print("── design A (targeted: the three hierarchy-component loops only)")
    print(f"  loop-rule calls          : {loop_entries:>12,}  (a FLOOR on guarded positions, not a count)")
    print(f"  PREDICTED delta entries  : >= {3 * 0:>9,}  and <= 3 x (loop iterations) — NOT observable")
    print( "  ⛔ a per-rule dump cannot attribute per SITE, so design A is priced only as a RISE of")
    print( "     unknown size. `.13c.2j` measured exactly this shape and the ratchet refused it.")
    print()
    print("⭐ THE PREDICTION IS THE POINT: re-run this after the fix and compare. A design-B measured")
    print("   delta that is not close to the number above means this model of the engine is wrong.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
