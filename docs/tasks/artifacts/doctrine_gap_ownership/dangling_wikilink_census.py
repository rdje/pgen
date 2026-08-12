#!/usr/bin/env python3
"""`DOCTRINE-GAP-OWNERSHIP.7` — do the repository's `[[wikilink]]` pointers actually RESOLVE?

Layer C (`docs/decisions/`) and the Knowledge Map (`docs/knowledge/`) are addressed by `[[name]]`
from every other tracked surface. Nothing checks that a `[[name]]` names anything: `KNOWLEDGE-MAP`
verifies the derived map matches its fact SOURCES, never that a reference REACHED a source. A
pointer that resolves to nothing fails silently, in the passing direction, and reads as a citation.

⛔ THE NUMBER THIS PRINTS IS A CLASSIFICATION, NOT A DEFECT COUNT
([[feedback_classify_referents_by_requirement]] — the repo has been wrong by 96x here before).
A raw dangling total is meaningless because several classes are not broken links at all:

  NOT-A-WIKILINK   `[[bin]]` (Cargo TOML sections), `[[link]]`, `[[example]]` — prose, false positives
  SPELLING         hyphenated spelling of a record that EXISTS under underscores — mechanical fix
  TASK-TREE        names a real `docs/tasks/<X>.md` — a real file in the wrong namespace
  DOC-REFERENT     names a tracked document (`MEMORY_ARCHITECTURE.md`, …) — likewise a namespace slip
  NEAR-MISS        one edit away from an existing record — probably a typo
  MISSING          nothing in the repository answers to this name — the only bucket that is a defect

Exit is 0 iff the MISSING bucket is empty, and the MISSING count is printed on the verdict line —
so a future ratchet keys on the bucket that is a defect instead of on the raw dangling total.

  python3 docs/tasks/artifacts/doctrine_gap_ownership/dangling_wikilink_census.py
  python3 …/dangling_wikilink_census.py --list missing      # every MISSING target, by occurrence
  python3 …/dangling_wikilink_census.py --list all          # every bucket, with the files
"""
from __future__ import annotations

import collections
import os
import re
import sys
from pathlib import Path

LINK = re.compile(r"\[\[([A-Za-z0-9_\-]+)\]\]")

# Tokens that are `[[…]]`-shaped but are not references: Cargo manifest sections and prose
# placeholders. Listed rather than pattern-matched, so adding one is a visible decision.
NOT_A_WIKILINK = {"bin", "link", "example", "name", "wikilink", "wiki-links"}

SCAN_ROOTS = (
    "docs",
    "MEMORY.md",
    "CHANGES.md",
    "DEVELOPMENT_NOTES.md",
    "TOOLBOX.md",
    "README.md",
    "KNOWLEDGE_MAP.md",
    "MEMORY_ARCHITECTURE.md",
    "DOCTRINE_ENFORCEMENT.md",
)


def repo_root() -> Path:
    here = Path(__file__).resolve()
    for parent in here.parents:
        if (parent / "CLAUDE.md").is_file() and (parent / "grammars").is_dir():
            return parent
    raise SystemExit(f"REFUSE: no repository root above {here}")


def markdown_files(root: Path) -> list[Path]:
    out: list[Path] = []
    for entry in SCAN_ROOTS:
        path = root / entry
        if path.is_file():
            out.append(path)
        elif path.is_dir():
            for dirpath, _, filenames in os.walk(path):
                out += [Path(dirpath) / f for f in filenames if f.endswith(".md")]
    if not out:
        raise SystemExit("REFUSE: no markdown scanned — the scan roots no longer exist")
    return out


def classify(name: str, known: set[str], trees: set[str], docs: dict[str, str]) -> tuple[str, str]:
    if name in trees:
        return "TASK-TREE", f"docs/tasks/{name}.md exists — wrong namespace for a [[…]] reference"
    if name.lower() in NOT_A_WIKILINK:
        return "NOT-A-WIKILINK", "prose or Cargo-manifest syntax, not a reference"
    if name in docs:
        return "DOC-REFERENT", f"{docs[name]} exists — a tracked document, not a memory record"
    underscored = name.replace("-", "_")
    if underscored in known:
        return "SPELLING", f"exists as {underscored}"
    squashed = name.replace("_", "").replace("-", "")
    for candidate in known:
        if candidate.replace("_", "") == squashed or candidate.endswith(name) or name.endswith(candidate):
            return "NEAR-MISS", f"one edit from {candidate}"
    return "MISSING", "nothing in docs/decisions/ or docs/knowledge/ answers to this name"


def main(argv: list[str]) -> int:
    root = repo_root()
    known = {
        f[:-3]
        for d in ("docs/decisions", "docs/knowledge")
        for f in os.listdir(root / d)
        if f.endswith(".md")
    }
    trees = {f[:-3] for f in os.listdir(root / "docs/tasks") if f.endswith(".md")}
    # Tracked documents that are NOT memory records — a [[…]] naming one is a namespace slip, not a
    # missing record, and conflating the two inflates the defect bucket.
    docs: dict[str, str] = {}
    for dirpath, _, filenames in os.walk(root / "docs"):
        rel = os.path.relpath(dirpath, root)
        if rel.startswith(("docs/decisions", "docs/knowledge", "docs/tasks")):
            continue
        for filename in filenames:
            if filename.endswith(".md"):
                docs.setdefault(filename[:-3], os.path.join(rel, filename))
    for filename in os.listdir(root):
        if filename.endswith(".md"):
            docs.setdefault(filename[:-3], filename)
    if not known:
        raise SystemExit("REFUSE: no records found in docs/decisions|knowledge — every link would 'dangle'")

    occurrences = collections.Counter()
    files_of: dict[str, set[str]] = collections.defaultdict(set)
    total = 0
    for path in markdown_files(root):
        text = path.read_text(errors="replace")
        for match in LINK.finditer(text):
            name = match.group(1)
            total += 1
            if name in known:
                continue
            occurrences[name] += 1
            files_of[name].add(str(path.relative_to(root)))

    buckets: dict[str, list[tuple[str, int, str]]] = collections.defaultdict(list)
    for name, count in occurrences.items():
        bucket, why = classify(name, known, trees, docs)
        buckets[bucket].append((name, count, why))

    print(f"scanned {total:,} [[…]] occurrences against {len(known)} records\n")
    print(f"  {'bucket':<16} {'targets':>8} {'occurrences':>12}   meaning")
    order = ["MISSING", "SPELLING", "TASK-TREE", "DOC-REFERENT", "NEAR-MISS", "NOT-A-WIKILINK"]
    meaning = {
        "MISSING": "⛔ the defect bucket — a citation that reaches nothing",
        "SPELLING": "mechanical: hyphens where the record uses underscores",
        "TASK-TREE": "a real file, referenced in the wrong namespace",
        "DOC-REFERENT": "a tracked document, referenced as if it were a record",
        "NEAR-MISS": "probable typo, one edit from a real record",
        "NOT-A-WIKILINK": "false positive by construction — not a reference",
    }
    for bucket in order:
        rows = buckets.get(bucket, [])
        print(f"  {bucket:<16} {len(rows):>8} {sum(c for _, c, _ in rows):>12}   {meaning[bucket]}")

    wanted = argv[argv.index("--list") + 1] if "--list" in argv else None
    if wanted:
        for bucket in order:
            if wanted != "all" and bucket.lower() != wanted.lower():
                continue
            rows = sorted(buckets.get(bucket, []), key=lambda r: -r[1])
            if not rows:
                continue
            print(f"\n{bucket}")
            for name, count, why in rows:
                print(f"  {name:<62} x{count:<4} {why}")
                if wanted == "all":
                    print(f"      files: {', '.join(sorted(files_of[name])[:4])}")

    missing = len(buckets.get("MISSING", []))
    print(f"\nVERDICT: {missing} MISSING target(s) — the only bucket that is a defect.")
    return missing


if __name__ == "__main__":
    raise SystemExit(1 if main(sys.argv) else 0)
