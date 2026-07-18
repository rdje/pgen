#!/usr/bin/env python3
"""RGX-0078.5.j.1 REPRESENTATION-ROAD STEP-0 — static construction census
over the canonical regex artifact (36e1f5fb).

Per cascade_build_<rule>: fold class (shaped-object / shaped-other / plain),
fixed element-wrapper allocs, quantified (per-iteration) element allocs,
$N-extraction clones, to_shaped_value calls, alloc_shaped_pairs calls,
Sequence-Vec constructions.
Per cascade_match_<rule>: boundary call-out sites (deriv_boundary.push) and
which protocol parse_R they invoke.
"""
import re
import sys
from collections import defaultdict

path = sys.argv[1] if len(sys.argv) > 1 else "generated/regex_parser.rs"
src = open(path).read().splitlines()

# Partition into functions (top-level "    fn " / "    pub fn " inside impl).
fn_re = re.compile(r"^    (?:pub )?fn ([a-zA-Z_0-9]+)")
funcs = {}  # name -> (start, end)
cur = None
for i, line in enumerate(src):
    m = fn_re.match(line)
    if m:
        if cur:
            funcs[cur[0]] = (cur[1], i)
        cur = (m.group(1), i)
if cur:
    funcs[cur[0]] = (cur[1], len(src))

build_rows = []
match_rows = []
for name, (a, b) in funcs.items():
    body = src[a:b]
    text = "\n".join(body)
    if name.startswith("cascade_build_"):
        rule = name[len("cascade_build_"):]
        elem_allocs = len(re.findall(r'rule_name: "element_', text))
        # element allocs inside a quantifier loop (per-iteration): heuristic =
        # "element_" allocs textually inside a `while`/`loop` block. Count
        # QuantCount consumers instead (exact: one loop per QuantCount read).
        quant_loops = len(re.findall(r"DerivEvent::QuantCount", text))
        opt_reads = len(re.findall(r"DerivEvent::OptPresent", text))
        or_reads = len(re.findall(r"DerivEvent::OrWinner", text))
        clones = len(re.findall(r"\.content\.clone\(\)", text))
        tsv = len(re.findall(r"to_shaped_value", text))
        pairs_calls = len(re.findall(r"alloc_shaped_pairs", text))
        pair_items = len(re.findall(r'^\s*\(\s*$|\("[a-zA-Z_]+", ', text))
        seq_vecs = len(re.findall(r"ParseContent::Sequence\(", text))
        shaped = len(re.findall(r"ParseContent::Shaped\(", text))
        node_allocs = len(re.findall(r"\.alloc\(ParseNode", text)) + len(
            re.findall(r"\.alloc\(__pgen", text)
        )
        boundary_reads = len(re.findall(r"deriv_next_boundary", text))
        fold = "shaped-object" if pairs_calls else ("shaped-other" if shaped else "plain")
        build_rows.append(
            (rule, fold, elem_allocs, quant_loops, opt_reads, or_reads, clones,
             tsv, pairs_calls, seq_vecs, node_allocs, boundary_reads)
        )
    elif name.startswith("cascade_match_"):
        rule = name[len("cascade_match_"):]
        pushes = len(re.findall(r"deriv_boundary\.push", text))
        callees = sorted(set(re.findall(r"parser\.parse_([a-zA-Z_0-9]+)\(\)", text)))
        if pushes:
            match_rows.append((rule, pushes, ",".join(callees)))

print("== cascade_build census (per-invocation static counts) ==")
print(f"{'rule':<42} {'fold':<14} {'elem':>4} {'qnt':>3} {'opt':>3} {'or':>3} "
      f"{'clon':>4} {'tsv':>3} {'objs':>4} {'seqV':>4} {'allocN':>6} {'bnd':>3}")
hist = defaultdict(int)
tot = defaultdict(int)
for r in sorted(build_rows):
    print(f"{r[0]:<42} {r[1]:<14} {r[2]:>4} {r[3]:>3} {r[4]:>3} {r[5]:>3} "
          f"{r[6]:>4} {r[7]:>3} {r[8]:>4} {r[9]:>4} {r[10]:>6} {r[11]:>3}")
    hist[r[1]] += 1
    tot["elem"] += r[2]; tot["clones"] += r[6]; tot["tsv"] += r[7]
    tot["objs"] += r[8]; tot["seqV"] += r[9]; tot["allocN"] += r[10]

print()
print("fold-class histogram:", dict(hist))
print("static totals:", dict(tot))
print()
print("== cascade_match boundary call-outs ==")
for r in sorted(match_rows):
    print(f"{r[0]:<42} pushes={r[1]:>2}  callees: {r[2]}")
