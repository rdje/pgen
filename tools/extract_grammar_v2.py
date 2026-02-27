#!/usr/bin/env python3
"""
Deduplicate and normalize grammar rules extracted from markdown.

Produces:
- JSON report with variants/sources
- EBNF file with first-seen normalized definitions
"""

from __future__ import annotations

import argparse
import json
import re
from collections import OrderedDict, defaultdict
from pathlib import Path


CATALOG_RE = re.compile(r"^([^|]+)\|([A-Za-z_][A-Za-z0-9_]*)\s*::=\s*(.+?)\s*$")


def normalize(defn: str) -> str:
    defn = re.sub(r"\s+", " ", defn).strip()
    return defn


def main() -> int:
    ap = argparse.ArgumentParser(description="Normalize/dedupe grammar catalog into EBNF + JSON report.")
    ap.add_argument("--catalog", required=True, help="Input catalog from extract_grammar.py")
    ap.add_argument("--output-ebnf", required=True, help="Output normalized EBNF path")
    ap.add_argument("--output-json", required=True, help="Output JSON report path")
    args = ap.parse_args()

    catalog_path = Path(args.catalog).expanduser().resolve()
    out_ebnf = Path(args.output_ebnf).expanduser().resolve()
    out_json = Path(args.output_json).expanduser().resolve()
    if not catalog_path.is_file():
        raise SystemExit(f"error: catalog not found: {catalog_path}")

    merged = OrderedDict()
    variants = defaultdict(set)
    sources = defaultdict(set)

    for raw in catalog_path.read_text(encoding="utf-8", errors="replace").splitlines():
        raw = raw.strip()
        if not raw:
            continue
        m = CATALOG_RE.match(raw)
        if not m:
            continue
        source = m.group(1)
        rule = m.group(2)
        definition = normalize(m.group(3))
        if rule not in merged:
            merged[rule] = definition
        variants[rule].add(definition)
        sources[rule].add(source)

    out_ebnf.parent.mkdir(parents=True, exist_ok=True)
    lines = []
    lines.append("(* Auto-generated normalized grammar (extract_grammar_v2.py) *)")
    lines.append("")
    for rule in sorted(merged.keys()):
        lines.append(f"{rule} ::=")
        lines.append(f"    {merged[rule]} ;")
        lines.append("")
    out_ebnf.write_text("\n".join(lines), encoding="utf-8")

    report = {
        "rule_count": len(merged),
        "rules": {
            rule: {
                "selected_definition": merged[rule],
                "variant_count": len(variants[rule]),
                "variants": sorted(variants[rule]),
                "sources": sorted(sources[rule]),
            }
            for rule in sorted(merged.keys())
        },
    }
    out_json.parent.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps(report, indent=2), encoding="utf-8")

    print(f"catalog: {catalog_path}")
    print(f"rule_count: {len(merged)}")
    print(f"output_ebnf: {out_ebnf}")
    print(f"output_json: {out_json}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

