#!/usr/bin/env python3
"""RGX-0078.5.j.1 STEP-0 — join the STATIC per-rule build census (what one
cascade_build_R invocation allocates/clones/folds) with the DYNAMIC per-rule
committed counts (the banked -0121 outcome dumps, byte-identical to -0116)
to estimate per-class construction populations on the 8-pattern bench corpus.

Committed counts are derivation-determined (path-independent between the
observability twin and the bare cascade), so the join weight is sound; the
known undercount is per-ITERATION element wrappers at quantified sites inside
a parent rule's build (attributed here to the parent as its static count only
once). The dynamic arena census (regex_construction_census_probe) is the
ground truth the join is cross-checked against.
"""
import json
import re
import sys
from collections import defaultdict

ARTIFACT = "generated/regex_parser.rs"
DUMP_DIR = "rust/target/generated_logs/spine_dispatch_step0"
PATTERNS = [
    "literal_simple", "digit_sequence", "character_class", "alternation",
    "capture_groups", "url_simple", "email_basic", "anchor_complex",
]

src = open(ARTIFACT).read().splitlines()
fn_re = re.compile(r"^    (?:pub )?fn ([a-zA-Z_0-9]+)")
funcs = {}
cur = None
for i, line in enumerate(src):
    m = fn_re.match(line)
    if m:
        if cur:
            funcs[cur[0]] = (cur[1], i)
        cur = (m.group(1), i)
if cur:
    funcs[cur[0]] = (cur[1], len(src))

static = {}
for name, (a, b) in funcs.items():
    if not name.startswith("cascade_build_"):
        continue
    rule = name[len("cascade_build_"):]
    text = "\n".join(src[a:b])
    static[rule] = {
        "elem": len(re.findall(r'rule_name: "element_', text)),
        "clones": len(re.findall(r"\.content\.clone\(\)", text)),
        "tsv": len(re.findall(r"to_shaped_value", text)),
        "objs": len(re.findall(r"alloc_shaped_pairs", text)),
        "seqV": len(re.findall(r"ParseContent::Sequence\(", text)),
        "allocN": len(re.findall(r"\.alloc\(ParseNode", text))
        + len(re.findall(r"\.alloc\(__pgen", text)),
    }

# Per-rule static census for PROTOCOL rule methods (the boundary-called zone):
proto = {}
for name, (a, b) in funcs.items():
    if not name.startswith("parse_") or name in ("parse_full_regex",):
        continue
    rule = name[len("parse_"):]
    text = "\n".join(src[a:b])
    proto[rule] = {
        "allocN": len(re.findall(r"\.alloc\(ParseNode", text))
        + len(re.findall(r"\.alloc\(__pgen", text)),
        "clones": len(re.findall(r"\.content\.clone\(\)", text)),
        "seqV": len(re.findall(r"ParseContent::Sequence\(", text)),
    }

grand = defaultdict(int)
print(f"{'pattern':<18} {'commit':>6} {'elem':>6} {'clones':>7} {'tsv':>6} "
      f"{'objs':>6} {'seqV':>6} {'allocN':>7}")
print("-" * 70)
for pat in PATTERNS:
    d = json.load(open(f"{DUMP_DIR}/{pat}.outcome.json"))
    cc = d["rule_committed_counts"]
    tot = defaultdict(int)
    for rule, n in cc.items():
        s = static.get(rule)
        if not s:
            continue
        for k, v in s.items():
            tot[k] += n * v
    print(f"{pat:<18} {d['total_committed']:>6} {tot['elem']:>6} "
          f"{tot['clones']:>7} {tot['tsv']:>6} {tot['objs']:>6} "
          f"{tot['seqV']:>6} {tot['allocN']:>7}")
    for k, v in tot.items():
        grand[k] += v
    grand["commit"] += d["total_committed"]
print("-" * 70)
print(f"{'TOTAL':<18} {grand['commit']:>6} {grand['elem']:>6} "
      f"{grand['clones']:>7} {grand['tsv']:>6} {grand['objs']:>6} "
      f"{grand['seqV']:>6} {grand['allocN']:>7}")

# Top rules by joined clone/alloc population across the corpus:
rule_tot = defaultdict(lambda: defaultdict(int))
for pat in PATTERNS:
    d = json.load(open(f"{DUMP_DIR}/{pat}.outcome.json"))
    for rule, n in d["rule_committed_counts"].items():
        s = static.get(rule)
        if not s:
            continue
        for k, v in s.items():
            rule_tot[rule][k] += n * v
        rule_tot[rule]["commit"] += n

print()
print("== top 20 rules by joined build-zone node allocs (corpus-wide) ==")
rows = sorted(rule_tot.items(), key=lambda kv: -kv[1]["allocN"])[:20]
print(f"{'rule':<38} {'commit':>6} {'allocN':>7} {'clones':>7} {'tsv':>5} {'objs':>5} {'seqV':>5}")
for rule, t in rows:
    print(f"{rule:<38} {t['commit']:>6} {t['allocN']:>7} {t['clones']:>7} "
          f"{t['tsv']:>5} {t['objs']:>5} {t['seqV']:>5}")
print()
print("== top 20 rules by joined build-zone content clones (corpus-wide) ==")
rows = sorted(rule_tot.items(), key=lambda kv: -kv[1]["clones"])[:20]
for rule, t in rows:
    print(f"{rule:<38} {t['commit']:>6} {t['allocN']:>7} {t['clones']:>7} "
          f"{t['tsv']:>5} {t['objs']:>5} {t['seqV']:>5}")
