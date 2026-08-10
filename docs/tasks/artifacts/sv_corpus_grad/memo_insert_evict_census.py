#!/usr/bin/env python3
"""Per-rule packrat memo INSERT / EVICT / REPLAY census (SV-CORPUS-GRAD.11a).

`--dump-rule-outcome-counts-json` reports a single `rule_memo_hit_counts` number
per rule, which FUSES three different memo paths -- a replayed SUCCESS, a cached
clean FAILURE and a cached tainted FAILURE. That fusion is why a rule can look
well-served by the memo while its expensive success path is never replayed even
once: `conditional_statement` reports 208 hits at n=6 and 208 of them are cached
failures.

This census splits them. It needs no new engine code: the generated
`memoized_call` ALREADY logs every memo transition under
`PGEN_TRACE_VERBOSITY=debug`, keyed by numeric rule id. This script joins those
lines against the parser's own `RULE_NAMES` table and reports, per rule:

    miss          body executed (no usable entry)
    ins_success   a successful entry was inserted
    evict_succ    a lookup found a TAINTED success whose store epoch had moved
                  (MEMO-STORE-SOUNDNESS.2) and removed it -- the body re-executes
    hit_success   a successful entry was REPLAYED  <-- the packrat guarantee
    ins_fail / ins_tfail / evict_fail / hit_fail / hit_tfail
                  the failure-side equivalents

GROUND TRUTH (checked on every run, `--verify FILE.json`): for every rule,
`miss + hit_success + hit_fail + hit_tfail` must equal that rule's
`rule_entry_counts` from `--dump-rule-outcome-counts-json`. The two instruments
are independent -- one reads the trace log, the other reads atomic counters --
so agreement is a real cross-check and a mismatch aborts rather than publishing.

⛔ The control is PARTITIONED, because not every rule is memoized. A rule reached
through the generated `inlined_frame_call` helper gets a full observable frame --
entry counter, coverage push, enter/exit trace -- but NO `memoized_call` at all:
it is inlined into its caller. On `systemverilog.ebnf` that is **663 of 1481
rules across 2871 call sites**, so a flat "every rule must balance" control fails
on ~305 live rules and says nothing. The partition:

  STRICT  rules with ZERO inlined call sites -- fully memoized, so the identity
          must hold EXACTLY. A mismatch aborts.
  MIXED   rules with >=1 inlined call site -- reachable both ways, so only
          `census_total <= entries` is assertable, and the shortfall IS the
          inlined-entry count. Reported, never silently dropped.

The partition is derived from the parser source (the `inlined_frame_call(Self::RULE_*`
sites), not hand-listed, so it cannot drift from the parser it is measuring.

Usage:
  PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe \
      --parse systemverilog IN.sv --profile sv_2017 --trace --trace-log-file T.log
  ./rust/target/release/parseability_probe --parse systemverilog IN.sv \
      --profile sv_2017 --dump-rule-outcome-counts-json OC.json
  memo_insert_evict_census.py T.log --parser generated/systemverilog_parser.rs \
      [--verify OC.json] [--rules r1,r2,...] [--top N]

⚠️ Both runs take the PROTOCOL graph, not the fused `cascade_*` graph (tracing and
counters each set `bare_parse = false`) -- see TOOLBOX.md 2.1. That is the correct
graph to reason about here: it is the one whose `memoized_call` is under study.
"""
import argparse
import collections
import json
import re
import sys

# The generated `memoized_call` log lines, one per memo transition.
PATTERNS = {
    "hit_success": r"💾 Memo hit for rule (\d+) at position \d+ - reusing cached result",
    "hit_fail":    r"💾 Memo hit for rule (\d+) at position \d+ - cached failure",
    "hit_tfail":   r"💾 Memo hit for rule (\d+) at position \d+ - cached tainted failure",
    "miss":        r"💾 Memo miss for rule (\d+)",
    "ins_success": r"💾 Memoized successful result for rule (\d+)",
    "ins_tfail":   r"💾 Memoized TAINTED failed result for rule (\d+)",
    "ins_fail":    r"💾 Memoized failed result for rule (\d+)",
    "evict_succ":  r"💾 Evicting STALE tainted success for rule (\d+)",
    "evict_fail":  r"💾 Evicting STALE tainted failure for rule (\d+)",
}
# `ins_fail` is a prefix of `ins_tfail`'s text only after the tainted variant is
# ruled out, so order matters: most specific first.
ORDER = ["hit_success", "hit_fail", "hit_tfail", "miss", "ins_success",
         "ins_tfail", "ins_fail", "evict_succ", "evict_fail"]
COLUMNS = ["miss", "ins_success", "evict_succ", "hit_success",
           "ins_fail", "ins_tfail", "evict_fail", "hit_fail", "hit_tfail"]


def parser_facts(parser_path):
    """The generated parser's id->name table (the join key) and the set of rule
    names reached through `inlined_frame_call` (the un-memoized population).

    Both are read from the parser being measured, so neither can drift from it.
    The id->name join is self-checking: every `const RULE_X: RuleId = N` must
    satisfy `RULE_NAMES[N] == x`, which pins the two tables to each other.
    """
    src = open(parser_path, encoding="utf-8", errors="replace").read()
    marker = "const RULE_NAMES: &'static [&'static str] = &["
    start = src.index(marker)
    names = re.findall(r'"((?:[^"\\]|\\.)*)"', src[start:src.index("];", start)])

    consts = {m.group(1): int(m.group(2))
              for m in re.finditer(r"const (RULE_[A-Z0-9_]+): RuleId = (\d+)", src)}
    mismatched = [c for c, v in consts.items()
                  if not (v < len(names) and names[v].upper() == c[len("RULE_"):])]
    if mismatched:
        raise SystemExit(f"id->name join is unsound: {len(mismatched)} const(s) do not "
                         f"index their own name (e.g. {mismatched[:3]})")

    inlined = {names[consts[c]]
               for c in set(re.findall(r"inlined_frame_call\(\s*Self::(RULE_[A-Z0-9_]+)\s*,", src))
               if c in consts}
    return names, inlined


def census(log_path):
    compiled = [(k, re.compile(PATTERNS[k])) for k in ORDER]
    counts = collections.defaultdict(collections.Counter)
    for line in open(log_path, encoding="utf-8", errors="replace"):
        if "💾" not in line:
            continue
        for key, pat in compiled:
            m = pat.search(line)
            if m:
                counts[int(m.group(1))][key] += 1
                break
    return counts


def verify(counts, names, inlined, oc_json):
    """Cross-check against the independent atomic counters. Abort on mismatch.

    Partitioned per the module docstring: STRICT rules must balance exactly,
    MIXED rules may only fall short (the shortfall being their inlined entries).
    """
    entries = json.load(open(oc_json))["rule_entry_counts"]
    idx = {n: i for i, n in enumerate(names)}
    bad, strict_n, mixed_n, mixed_shortfall = [], 0, 0, 0
    for rule, want in entries.items():
        c = counts[idx[rule]] if rule in idx else collections.Counter()
        got = c["miss"] + c["hit_success"] + c["hit_fail"] + c["hit_tfail"]
        if rule in inlined:
            mixed_n += 1
            mixed_shortfall += want - got
            if got > want:                      # more memo frames than entries
                bad.append((rule, want, got, "MIXED: census exceeds entries"))
        else:
            strict_n += 1
            if got != want:
                bad.append((rule, want, got, "STRICT: must balance exactly"))
    if bad:
        print(f"GROUND-TRUTH CONTROL FAILED on {len(bad)} rule(s):", file=sys.stderr)
        for rule, want, got, why in bad[:10]:
            print(f"  {rule}: counters={want} census={got}  [{why}]", file=sys.stderr)
        return False
    print(f"ground-truth control OK ({oc_json}): {strict_n} STRICT rules balance "
          f"exactly; {mixed_n} MIXED (inlined-reachable) rules account for "
          f"{mixed_shortfall} un-memoized entries")
    return True


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("log")
    ap.add_argument("--parser", default="generated/systemverilog_parser.rs")
    ap.add_argument("--verify", help="outcome-counts JSON to cross-check against")
    ap.add_argument("--rules", help="comma-separated rules to report")
    ap.add_argument("--top", type=int, default=0,
                    help="also report the N rules with the most evictions")
    args = ap.parse_args()

    names, inlined = parser_facts(args.parser)
    counts = census(args.log)
    idx = {n: i for i, n in enumerate(names)}
    print(f"parser: {len(names)} rules, {len(inlined)} of them reachable through "
          f"inlined_frame_call (NOT memoized)")

    if args.verify and not verify(counts, names, inlined, args.verify):
        return 2

    total = collections.Counter()
    for c in counts.values():
        total.update(c)
    # Replays per insert, NOT a percentage: one entry may be replayed many times,
    # so this legitimately exceeds 1.0 on a healthy parse. It is whole-parse
    # context for the per-rule rows below -- a memo that works globally can still
    # be serving a given rule zero times, which is the whole point of the census.
    per_insert = total["hit_success"] / max(total["ins_success"], 1)
    print(f"\nWHOLE PARSE  success_inserts={total['ins_success']} "
          f"stale_success_evictions={total['evict_succ']} "
          f"SUCCESS_REPLAYS={total['hit_success']} "
          f"({per_insert:.1f} replays per insert)")

    selected = []
    if args.rules:
        selected = [r.strip() for r in args.rules.split(",") if r.strip()]
    if args.top:
        ranked = sorted(counts.items(), key=lambda kv: -kv[1]["evict_succ"])
        for rid, _ in ranked[:args.top]:
            if rid < len(names) and names[rid] not in selected:
                selected.append(names[rid])
    if not selected:
        return 0

    print(f"\n{'rule':34s}" + "".join(f"{c:>12s}" for c in COLUMNS))
    for rule in selected:
        if rule not in idx:
            print(f"{rule:34s}  <not a rule of this grammar>")
            continue
        c = counts[idx[rule]]
        print(f"{rule:34s}" + "".join(f"{c[col]:12d}" for col in COLUMNS))
    return 0


if __name__ == "__main__":
    sys.exit(main())
