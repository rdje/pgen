#!/usr/bin/env python3
"""Tree-walk attribution over a /usr/bin/sample call-graph dump.

For each target symbol substring, report: total cum samples, and the breakdown
of cum samples by nearest enclosing *interesting* caller frame (the attribution
frame), so populations can be assigned to grammar/runtime sites.
"""
import re
import sys
from collections import defaultdict

path = sys.argv[1]
targets = sys.argv[2:]

# Frames worth attributing to (parser rules / runtime entry points).
ATTR = re.compile(
    r"(cascade_match_\w+|cascade_build_\w+|parse_[a-z_]+|memoized_call|"
    r"with_semantic_runtime_rule_transaction|apply_semantic_runtime_effect_directive|"
    r"checkpoint|extract_delta_since|apply_delta|rollback_to_labeled)"
)

line_re = re.compile(r"^(\s*[+!:|\s]*)(\d+)\s+(.*?)(?:\s+\(in .*)?$")

stack = []  # list of (depth, symbol, count)
cum = defaultdict(int)                 # target -> cum samples
by_caller = defaultdict(lambda: defaultdict(int))  # target -> attribution frame -> cum

in_graph = False
for raw in open(path, errors="replace"):
    if raw.startswith("Call graph"):
        in_graph = True
        continue
    if raw.startswith("Total number in stack"):
        break
    if not in_graph:
        continue
    m = line_re.match(raw.rstrip())
    if not m:
        continue
    prefix, count, sym = m.group(1), int(m.group(2)), m.group(3)
    depth = len(prefix)
    while stack and stack[-1][0] >= depth:
        stack.pop()
    stack.append((depth, sym, count))
    for t in targets:
        if t in sym:
            # Only count if no ancestor already matched this target (avoid
            # double-counting recursive frames).
            if any(t in s for _, s, _ in stack[:-1]):
                continue
            cum[t] += count
            attr = "«root»"
            for _, s, _ in reversed(stack[:-1]):
                am = ATTR.search(s)
                if am:
                    attr = am.group(1)
                    break
            by_caller[t][attr] += count
            break

for t in targets:
    print(f"=== {t}: cum={cum[t]}")
    for caller, n in sorted(by_caller[t].items(), key=lambda kv: -kv[1])[:12]:
        print(f"    {n:6d}  {caller}")
