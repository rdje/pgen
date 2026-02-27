#!/usr/bin/env python3
"""
Extract raw grammar-like rules from markdown section files.

Output format is a simple catalog text file:
source_file|rule_name ::= rule_definition
"""

from __future__ import annotations

import argparse
import re
from pathlib import Path
from typing import List, Tuple


RULE_RE = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*::=\s*(.+?)\s*$")


def extract_rules_from_text(text: str) -> List[Tuple[str, str]]:
    rules: List[Tuple[str, str]] = []
    lines = text.splitlines()
    i = 0
    while i < len(lines):
        line = lines[i]
        m = RULE_RE.match(line)
        if not m:
            i += 1
            continue
        name = m.group(1)
        parts = [m.group(2).strip()]
        i += 1
        while i < len(lines):
            nxt = lines[i].strip()
            if not nxt:
                break
            if RULE_RE.match(lines[i]):
                break
            if nxt.startswith("|") or lines[i].startswith(" "):
                parts.append(nxt)
                i += 1
                continue
            break
        definition = " ".join(parts)
        definition = re.sub(r"\s+", " ", definition).strip()
        rules.append((name, definition))
    return rules


def main() -> int:
    ap = argparse.ArgumentParser(description="Extract grammar rules from markdown files.")
    ap.add_argument("--md-dir", required=True, help="Directory containing section-*.md files.")
    ap.add_argument(
        "--output",
        required=True,
        help="Output catalog path (text).",
    )
    args = ap.parse_args()

    md_dir = Path(args.md_dir).expanduser().resolve()
    out_path = Path(args.output).expanduser().resolve()
    if not md_dir.is_dir():
        raise SystemExit(f"error: md directory not found: {md_dir}")

    rows: List[str] = []
    file_count = 0
    rule_count = 0
    for md_file in sorted(md_dir.glob("section-*.md")):
        file_count += 1
        text = md_file.read_text(encoding="utf-8", errors="replace")
        rules = extract_rules_from_text(text)
        for name, definition in rules:
            rows.append(f"{md_file.name}|{name} ::= {definition}")
        rule_count += len(rules)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(rows) + ("\n" if rows else ""), encoding="utf-8")

    print(f"md_dir: {md_dir}")
    print(f"files_scanned: {file_count}")
    print(f"rules_extracted: {rule_count}")
    print(f"output: {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

