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


RULE_RE = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*::=\s*(.*?)\s*$")
SECTION_HEADING_RE = re.compile(r"^(?:Annex\s+[A-Z]|[A-Z]\.\d+(?:\.\d+)*|\d+(?:\.\d+)*)\b")


def is_fence_line(line: str) -> bool:
    return line.strip().startswith("```")


def is_page_noise_line(line: str) -> bool:
    stripped = line.strip()
    if not stripped:
        return False
    if stripped.startswith("Authorized licensed use limited to:"):
        return True
    if stripped in {"IEEE", "HARDWARE DESCRIPTION LANGUAGE", "IEEE STANDARD FOR VERILOG®"}:
        return True
    if stripped.startswith("Std 1364-2005"):
        return True
    if stripped.startswith("Copyright ©"):
        return True
    if stripped.isdigit():
        return True
    return False


def looks_incomplete(parts: List[str]) -> bool:
    if not parts:
        return True
    text = " ".join(parts).rstrip()
    if text.endswith(("::=", "|", "{", ",", "(")):
        return True
    return (
        text.count("(") != text.count(")")
        or text.count("[") != text.count("]")
        or text.count("{") != text.count("}")
    )


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
        parts = []
        first_rhs = m.group(2).strip()
        if first_rhs:
            parts.append(first_rhs)
        i += 1
        while i < len(lines):
            nxt = lines[i].strip()
            if is_fence_line(lines[i]) or is_page_noise_line(lines[i]):
                if looks_incomplete(parts):
                    i += 1
                    continue
                break
            if not nxt:
                if looks_incomplete(parts):
                    i += 1
                    continue
                break
            if SECTION_HEADING_RE.match(nxt):
                break
            if RULE_RE.match(lines[i]):
                break
            if nxt.startswith("|") or lines[i].startswith(" "):
                parts.append(nxt)
                i += 1
                continue
            if looks_incomplete(parts):
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
