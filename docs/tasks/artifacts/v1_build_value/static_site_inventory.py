#!/usr/bin/env python3
"""V1 STEP-0 (PGEN-RGX-0078-0152) — static build-value site inventory.

Maps every clone/convert site class in the generated regex artifact to its
enclosing emitted function, so the dynamic counter populations (the
shaped_conversion_census numbers) can be attributed to emitter mechanisms.

Site classes:
  seq_peel_clone   elements[N].content.clone()            ($N property access, Sequence peel)
  alt_hop_clone    ParseContent::Alternative(node) => node.content.clone()
  other_clone      other => other.clone()                  ($N fallback arm)
  expr_clone       (expr).clone() via let __pgen_content   (multi-capture $N)
  peel_walk_clone  current = node.content.clone()          (nested-access peel walk)
  peel_fn_clone    __pgen_peel_alternative(node.content.clone())
  raw_best_clone   best_raw_content = Some(raw_content.clone())   (tournament raw carrier)
  raw_use_clone    let content = raw_content.clone()       (protocol raw-content reuse)
  direct_fold      .content.to_shaped_value(parser.arena)  (NO clone — boundary/child fold)
  pgen_content     let __pgen_content =                    (clone-first conversion marker)
"""

import re
import sys
from collections import defaultdict

ARTIFACT = sys.argv[1] if len(sys.argv) > 1 else "generated/regex_parser.rs"

CLASSES = [
    ("seq_peel_clone", re.compile(r"elements\[\d+usize\]\.content\.clone\(\)")),
    ("alt_hop_clone", re.compile(r"ParseContent::Alternative\(node\) => node\.content\.clone\(\)")),
    ("other_clone", re.compile(r"other => other\.clone\(\)")),
    ("peel_walk_clone", re.compile(r"current = node\.content\.clone\(\)")),
    ("peel_fn_clone", re.compile(r"__pgen_peel_alternative\(node\.content\.clone\(\)\)")),
    ("raw_best_clone", re.compile(r"best_raw_content = Some\(raw_content\.clone\(\)\)")),
    ("raw_use_clone", re.compile(r"let content = raw_content\.clone\(\)")),
    ("direct_fold", re.compile(r"\.content\.to_shaped_value\(parser\.arena\)")),
    ("pgen_content", re.compile(r"let __pgen_content = ")),
]

FN_RE = re.compile(r"^    (?:pub )?fn ([a-zA-Z0-9_]+)")

fn_name = "<preamble>"
per_fn = defaultdict(lambda: defaultdict(int))
totals = defaultdict(int)

with open(ARTIFACT, encoding="utf-8") as f:
    for line in f:
        m = FN_RE.match(line)
        if m:
            fn_name = m.group(1)
        for cls, rx in CLASSES:
            n = len(rx.findall(line))
            if n:
                per_fn[fn_name][cls] += n
                totals[cls] += n

print(f"# static build-value site inventory — {ARTIFACT}")
print(f"# totals: " + " ".join(f"{k}={v}" for k, v in sorted(totals.items())))
print()
hdr = ["fn"] + [c for c, _ in CLASSES]
print("\t".join(hdr))
for fn in sorted(per_fn, key=lambda f: -sum(per_fn[f].values())):
    row = [fn] + [str(per_fn[fn].get(c, 0)) for c, _ in CLASSES]
    print("\t".join(row))
