#!/usr/bin/env python3
"""
Split an IEEE LRM PDF into clause-based text files using the embedded PDF TOC.

This is a generalized/adapted version of the earlier 1800-2017 split flow:
- no hardcoded line numbers,
- works with different IEEE LRMs (for example 1800-2023 and 1076-2019),
- emits a machine-readable manifest for downstream processing.
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Iterable, List, Tuple

import fitz  # PyMuPDF


CLAUSE_RE = re.compile(r"^\s*(\d+(?:\.\d+)*)(?:\.)?\s+(.+?)\s*$")
ANNEX_RE = re.compile(r"^\s*(Annex\s+[A-Z])\s+(.+?)\s*$", re.IGNORECASE)


def slugify(value: str) -> str:
    value = re.sub(r"\s+", " ", value.replace("\r", " ").replace("\n", " ")).strip()
    value = re.sub(r"[^A-Za-z0-9]+", "-", value)
    value = re.sub(r"-{2,}", "-", value).strip("-")
    return value.lower() or "untitled"


@dataclass
class Section:
    section_id: str
    title: str
    page_start: int
    page_end: int
    toc_level: int
    source_title: str

    @property
    def filename(self) -> str:
        safe_id = self.section_id.replace(".", "_").replace(" ", "_").replace("/", "_")
        return f"section-{safe_id}-{slugify(self.title)}.txt"


def is_clause_heading(title: str, clause_depth: int, include_annex: bool) -> Tuple[bool, str, str]:
    m = CLAUSE_RE.match(title)
    if m:
        section_id = m.group(1)
        depth = section_id.count(".") + 1
        if depth <= clause_depth:
            return True, section_id, m.group(2).strip()
        return False, "", ""

    if include_annex:
        am = ANNEX_RE.match(title)
        if am:
            return True, am.group(1).strip(), am.group(2).strip()
    return False, "", ""


def collect_sections(
    toc: Iterable[Tuple[int, str, int]],
    total_pages: int,
    clause_depth: int,
    include_annex: bool,
    toc_max_level: int,
    limit: int | None,
) -> List[Section]:
    candidates: List[Tuple[int, str, str, int, str]] = []
    seen = set()

    for level, raw_title, page in toc:
        if level > toc_max_level:
            continue
        title = re.sub(r"\s+", " ", raw_title.replace("\r", " ").strip())
        keep, section_id, short_title = is_clause_heading(title, clause_depth, include_annex)
        if not keep:
            continue
        # TOC pages are 1-based.
        page_start = int(page)
        dedupe_key = (section_id, page_start)
        if dedupe_key in seen:
            continue
        seen.add(dedupe_key)
        candidates.append((page_start, section_id, short_title, level, title))

    candidates.sort(key=lambda x: x[0])
    if limit is not None:
        candidates = candidates[:limit]

    sections: List[Section] = []
    for idx, (page_start, section_id, short_title, level, source_title) in enumerate(candidates):
        next_start = candidates[idx + 1][0] if idx + 1 < len(candidates) else total_pages + 1
        page_end = max(page_start, next_start - 1)
        sections.append(
            Section(
                section_id=section_id,
                title=short_title,
                page_start=page_start,
                page_end=page_end,
                toc_level=level,
                source_title=source_title,
            )
        )
    return sections


def heading_from_page_text(
    text: str,
    clause_depth: int,
    include_annex: bool,
) -> Tuple[str, str] | None:
    lines = [re.sub(r"\s+", " ", line).strip() for line in text.splitlines()]
    lines = [line for line in lines if line]

    # Only inspect the first chunk of each page where section headings usually live.
    for line in lines[:40]:
        keep, section_id, title = is_clause_heading(line, clause_depth, include_annex)
        if not keep:
            continue
        # Reject obvious false positives from inline references.
        if len(title) < 3 or len(title) > 160:
            continue
        if not re.search(r"[A-Za-z]", title):
            continue
        return section_id, title
    return None


def collect_sections_without_toc(
    doc: fitz.Document,
    clause_depth: int,
    include_annex: bool,
    limit: int | None,
) -> List[Section]:
    total_pages = len(doc)
    candidates: List[Tuple[int, str, str, int, str]] = []
    seen_ids: set[str] = set()

    for page_idx in range(total_pages):
        page = doc.load_page(page_idx)
        text = page.get_text("text")
        heading = heading_from_page_text(text, clause_depth, include_annex)
        if heading is None:
            continue
        section_id, short_title = heading
        if section_id in seen_ids:
            continue
        seen_ids.add(section_id)
        page_start = page_idx + 1
        source_title = f"{section_id} {short_title}"
        candidates.append((page_start, section_id, short_title, 1, source_title))

    candidates.sort(key=lambda x: x[0])
    if limit is not None:
        candidates = candidates[:limit]

    sections: List[Section] = []
    for idx, (page_start, section_id, short_title, level, source_title) in enumerate(candidates):
        next_start = candidates[idx + 1][0] if idx + 1 < len(candidates) else total_pages + 1
        page_end = max(page_start, next_start - 1)
        sections.append(
            Section(
                section_id=section_id,
                title=short_title,
                page_start=page_start,
                page_end=page_end,
                toc_level=level,
                source_title=source_title,
            )
        )
    return sections


def extract_text_range(doc: fitz.Document, page_start: int, page_end: int) -> str:
    # Input pages are 1-based inclusive.
    chunks = []
    for page_no in range(page_start - 1, page_end):
        page = doc.load_page(page_no)
        text = page.get_text("text")
        chunks.append(text.rstrip() + "\n")
    return "".join(chunks)


def main() -> int:
    ap = argparse.ArgumentParser(description="Split IEEE LRM PDF into clause text files.")
    ap.add_argument("--pdf", required=True, help="Path to source PDF.")
    ap.add_argument("--out-dir", required=True, help="Output directory for section-*.txt files.")
    ap.add_argument(
        "--manifest-out",
        default=None,
        help="Output JSON manifest path (default: <out-dir>/sections_manifest.json).",
    )
    ap.add_argument(
        "--clause-depth",
        type=int,
        default=1,
        help="Maximum numeric clause depth to extract (1=top clauses, 2=subclauses, ...).",
    )
    ap.add_argument(
        "--include-annex",
        action="store_true",
        help="Also include 'Annex X ...' headings when present in TOC.",
    )
    ap.add_argument(
        "--toc-max-level",
        type=int,
        default=3,
        help="Maximum PDF TOC level to inspect.",
    )
    ap.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Optional cap on number of sections (useful for smoke tests).",
    )
    ap.add_argument(
        "--standard",
        default="",
        help="Optional standard label to include in manifest metadata (for example 'IEEE 1800-2023').",
    )
    args = ap.parse_args()

    pdf_path = Path(args.pdf).expanduser().resolve()
    out_dir = Path(args.out_dir).expanduser().resolve()
    out_dir.mkdir(parents=True, exist_ok=True)
    manifest_path = (
        Path(args.manifest_out).expanduser().resolve()
        if args.manifest_out
        else out_dir / "sections_manifest.json"
    )

    if not pdf_path.is_file():
        raise SystemExit(f"error: PDF not found: {pdf_path}")
    if args.clause_depth < 1:
        raise SystemExit("error: --clause-depth must be >= 1")
    if args.toc_max_level < 1:
        raise SystemExit("error: --toc-max-level must be >= 1")

    doc = fitz.open(str(pdf_path))
    toc = doc.get_toc(simple=True)
    if toc:
        sections = collect_sections(
            toc=toc,
            total_pages=len(doc),
            clause_depth=args.clause_depth,
            include_annex=args.include_annex,
            toc_max_level=args.toc_max_level,
            limit=args.limit,
        )
        detection_mode = "pdf_toc"
    else:
        sections = collect_sections_without_toc(
            doc=doc,
            clause_depth=args.clause_depth,
            include_annex=args.include_annex,
            limit=args.limit,
        )
        detection_mode = "page_heading_fallback"
    if not sections:
        raise SystemExit(
            "error: no matching sections extracted; try adjusting --clause-depth / --toc-max-level"
        )

    for section in sections:
        text = extract_text_range(doc, section.page_start, section.page_end)
        (out_dir / section.filename).write_text(text, encoding="utf-8")

    manifest = {
        "source_pdf": str(pdf_path),
        "standard": args.standard,
        "detection_mode": detection_mode,
        "total_pages": len(doc),
        "toc_entries": len(toc),
        "clause_depth": args.clause_depth,
        "include_annex": bool(args.include_annex),
        "toc_max_level": args.toc_max_level,
        "section_count": len(sections),
        "sections": [asdict(s) | {"filename": s.filename} for s in sections],
    }
    manifest_path.write_text(json.dumps(manifest, indent=2), encoding="utf-8")

    print(f"source_pdf: {pdf_path}")
    print(f"out_dir: {out_dir}")
    print(f"manifest: {manifest_path}")
    print(f"sections_written: {len(sections)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
