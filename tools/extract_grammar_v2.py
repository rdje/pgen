#!/usr/bin/env python3
"""
Deduplicate and normalize grammar rules extracted from markdown.

Produces:
- JSON report with variants/sources
- EBNF file with normalized definitions chosen from the strongest recovered
  variant, with light-weight placeholder alias repair for extracted LRM
  artifacts
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


def is_placeholder(defn: str) -> bool:
    return "(From " in defn


def looks_structurally_complete(defn: str) -> bool:
    if not defn:
        return False
    if defn.rstrip().endswith(("|", "::=", "{", ",", "(")):
        return False
    return (
        defn.count("(") == defn.count(")")
        and defn.count("[") == defn.count("]")
        and defn.count("{") == defn.count("}")
    )


def alias_repair_candidates(rule: str, all_variants: dict[str, set[str]]) -> list[str]:
    if not rule.endswith("a"):
        return []

    stem = rule[:-1]
    repaired = set()
    for candidate_rule, definitions in all_variants.items():
        if candidate_rule == rule or not candidate_rule.startswith(stem):
            continue
        suffix = candidate_rule[len(stem) :]
        if not suffix.isdigit():
            continue
        for definition in definitions:
            if not is_placeholder(definition) and looks_structurally_complete(definition):
                repaired.add(definition)

    return sorted(repaired)


def choose_best_variant(rule: str, definitions: set[str], all_variants: dict[str, set[str]]) -> str:
    variants = sorted(definitions)

    complete_non_placeholder = [
        variant
        for variant in variants
        if not is_placeholder(variant) and looks_structurally_complete(variant)
    ]
    if complete_non_placeholder:
        return max(complete_non_placeholder, key=len)

    non_placeholder = [variant for variant in variants if not is_placeholder(variant)]
    if non_placeholder:
        return max(non_placeholder, key=len)

    selected = max(variants, key=len)
    if is_placeholder(selected):
        repaired = alias_repair_candidates(rule, all_variants)
        if len(repaired) == 1:
            return repaired[0]

    complete_placeholder = [
        variant for variant in variants if looks_structurally_complete(variant)
    ]
    if complete_placeholder:
        return max(complete_placeholder, key=len)

    return selected


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
        variants[rule].add(definition)
        sources[rule].add(source)

    for rule in variants.keys():
        merged[rule] = choose_best_variant(rule, variants[rule], variants)

    out_ebnf.parent.mkdir(parents=True, exist_ok=True)
    lines = []
    lines.append("# Auto-generated normalized grammar (extract_grammar_v2.py)")
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
