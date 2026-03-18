#!/usr/bin/env python3
"""
Convert section-*.txt files produced by split_sections.py into markdown files.

Adapted from the 1800-2017 converter flow, generalized for any IEEE LRM text slice set.
"""

from __future__ import annotations

import argparse
import re
from pathlib import Path
from typing import List


SECTION_FILE_RE = re.compile(r"^section-([^-]+)-(.+)\.txt$")
HEADING_RE = re.compile(r"^(\d+(?:\.\d+)*)(?:\.)?\s+(.+)$")


def parse_section_filename(path: Path) -> tuple[str, str]:
    m = SECTION_FILE_RE.match(path.name)
    if not m:
        return "unknown", path.stem
    section_id = m.group(1).replace("_", ".")
    title_slug = m.group(2).replace("-", " ")
    return section_id, title_slug


def fence_grammar_blocks(lines: List[str]) -> List[str]:
    out: List[str] = []
    i = 0
    while i < len(lines):
        line = lines[i]
        if "::=" in line:
            out.append("```ebnf")
            out.append(line)
            i += 1
            while i < len(lines):
                nxt = lines[i]
                if not nxt.strip():
                    break
                if "::=" in nxt or nxt.lstrip().startswith("|") or nxt.startswith(" "):
                    out.append(nxt)
                    i += 1
                    continue
                break
            out.append("```")
            out.append("")
            continue
        out.append(line)
        i += 1
    return out


def structure_text(raw: str) -> str:
    lines = [ln.rstrip() for ln in raw.replace("\r\n", "\n").replace("\r", "\n").split("\n")]
    lines = fence_grammar_blocks(lines)
    result: List[str] = []
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("```") or (result and result[-1].startswith("```") and stripped):
            result.append(line)
            continue
        if not stripped:
            result.append("")
            continue
        hm = HEADING_RE.match(stripped)
        if hm:
            depth = hm.group(1).count(".") + 2
            hashes = "#" * min(depth, 6)
            result.append(f"{hashes} {stripped}")
            result.append("")
            continue
        result.append(line)

    text = "\n".join(result)
    text = re.sub(r"\n{3,}", "\n\n", text).strip() + "\n"
    return text


def convert_file(
    txt_file: Path,
    md_file: Path,
    document: str,
    standard: str,
    domain: str,
    source_pdf: str,
) -> None:
    section_id, fallback_title = parse_section_filename(txt_file)
    raw = txt_file.read_text(encoding="utf-8", errors="replace")

    # Try first meaningful line as the best title.
    title = fallback_title
    lines = [line.strip() for line in raw.splitlines()]
    for s in lines:
        if not s:
            continue
        m = HEADING_RE.match(s)
        if m and m.group(1).replace(".", "_") == section_id.replace(".", "_"):
            title = m.group(2)
            break
    frontmatter = (
        "---\n"
        f'title: "Section {section_id}: {title}"\n'
        f'document: "{document}"\n'
        f'standard: "{standard}"\n'
        f'domain: "{domain}"\n'
        f'section: "{section_id}"\n'
        f'source_txt: "{txt_file.name}"\n'
        f'source_pdf: "{source_pdf}"\n'
        "---\n\n"
        f"# Section {section_id}: {title}\n\n"
    )
    body = structure_text(raw)
    md_file.write_text(frontmatter + body, encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser(description="Convert section text files to markdown.")
    ap.add_argument("--txt-dir", required=True, help="Input directory containing section-*.txt files.")
    ap.add_argument("--md-dir", required=True, help="Output directory for section-*.md files.")
    ap.add_argument("--document", required=True, help="Document title.")
    ap.add_argument("--standard", required=True, help="Standard label (for example IEEE 1800-2023).")
    ap.add_argument("--domain", default="HDL", help="Domain label (default: HDL).")
    ap.add_argument("--source-pdf", default="", help="Optional source PDF path for metadata.")
    args = ap.parse_args()

    txt_dir = Path(args.txt_dir).expanduser().resolve()
    md_dir = Path(args.md_dir).expanduser().resolve()
    if not txt_dir.is_dir():
        raise SystemExit(f"error: txt directory not found: {txt_dir}")
    md_dir.mkdir(parents=True, exist_ok=True)

    files = sorted(p for p in txt_dir.glob("section-*.txt") if p.is_file())
    if not files:
        raise SystemExit(f"error: no section-*.txt files found under {txt_dir}")

    for txt_file in files:
        md_file = md_dir / (txt_file.stem + ".md")
        convert_file(
            txt_file=txt_file,
            md_file=md_file,
            document=args.document,
            standard=args.standard,
            domain=args.domain,
            source_pdf=args.source_pdf,
        )

    print(f"txt_dir: {txt_dir}")
    print(f"md_dir: {md_dir}")
    print(f"files_converted: {len(files)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
