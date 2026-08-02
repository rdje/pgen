#!/usr/bin/env python3
"""SV-EXH-PROOF.7.4.6.13 -- census every `max_depth=N` value recorded in a coverage
artifact's per-branch `failure_reasons`, so the cumulative +4 depth-slack ladder is
measured rather than inferred.

`branch_groups[key].failure_reasons` is a LIST indexed by branch index; each element is a
map reason-string -> count.  (A map-shaped read returns a spurious clean zero -- the
instrument-honesty trap recorded in the leaf.)

GROUND TRUTH (refuse rather than guess): the script asserts it saw at least one
`max_depth=` reason at all; a run with none is reported as REFUSED, not as "no ladder".
"""
import json
import re
import sys
from collections import Counter, defaultdict

MAX_DEPTH_RE = re.compile(r"max_depth=(\d+)")


def census(path):
    doc = json.load(open(path))
    per_value = Counter()
    per_value_groups = defaultdict(set)
    groups_escalated = set()
    for key, group in doc["branch_groups"].items():
        reasons = group.get("failure_reasons") or []
        if isinstance(reasons, dict):  # defensive: shape drift
            reasons = list(reasons.values())
        for entry in reasons:
            if not isinstance(entry, dict):
                continue
            for reason, count in entry.items():
                m = MAX_DEPTH_RE.search(reason)
                if not m:
                    continue
                value = int(m.group(1))
                per_value[value] += count
                per_value_groups[value].add(key)
    return doc, per_value, per_value_groups


def report(path, base_depth):
    doc, per_value, per_value_groups = census(path)
    if not per_value:
        print(f"REFUSED {path}: no `max_depth=` reason found at all "
              f"(shape drift or a genuinely ladder-free run -- do NOT read this as zero)")
        return 1
    values = sorted(per_value)
    steps = sorted({b - a for a, b in zip(values, values[1:])})
    total = sum(per_value.values())
    escalated = {v: c for v, c in per_value.items() if v > base_depth + 4}
    esc_groups = set()
    for v in escalated:
        esc_groups |= per_value_groups[v]
    print(f"== {path}")
    print(f"   grammar={doc['grammar_name']} branch_groups={len(doc['branch_groups'])}")
    print(f"   distinct max_depth values : {len(values)}  min={values[0]} max={values[-1]}"
          f"  ({values[-1] / base_depth:.1f}x the configured --max-depth {base_depth})")
    print(f"   distinct step set         : {steps}")
    print(f"   recorded failures total   : {total}")
    print(f"   above base+4 (>{base_depth + 4})       : {sum(escalated.values())}"
          f" ({100.0 * sum(escalated.values()) / total:.1f}%) in {len(esc_groups)} branch groups")
    print("   top 6 values by failure count:")
    for value, count in per_value.most_common(6):
        print(f"      max_depth={value:<5} {count:>9}")
    print(f"   first 12 values: {values[:12]}")
    return 0


if __name__ == "__main__":
    base = int(sys.argv[1])
    rc = 0
    for p in sys.argv[2:]:
        rc |= report(p, base)
        print()
    sys.exit(rc)
