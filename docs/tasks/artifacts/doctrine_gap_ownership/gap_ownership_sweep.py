#!/usr/bin/env python3
"""DOCTRINE-GAP-OWNERSHIP.1 — the gap-ownership census, with a triage RATCHET.

WHY THIS EXISTS
---------------
The founding case: a real defect was found on 2026-07-22, written into a decision
record's prose, and then sat for 58 commits because `docs/decisions/` is a MEMORY
surface, not a WORK QUEUE. Nothing pointed at it; nothing asked about it.

A one-shot census does not fix that — it just moves the inertness up one level. The
census itself becomes the thing nobody re-runs, and every future session re-triages
143 hits from zero. So this driver is a RATCHET, not a report:

  sweep the repo  ->  mechanically resolve what can be resolved  ->  join the
  residue against a TRACKED register  ->  exit nonzero on anything UNTRIAGED.

⇒ a newly-recorded gap phrase must be classified (in the register) or mechanically
resolvable, or the check fails. Nothing can slide by being merely written down.

THE THREE MECHANICAL RESOLUTIONS (re-derived from the tree every run, never stored)
----------------------------------------------------------------------------------
1. FENCE       — the hit is inside a ``` fenced code block: it is example code, not
                 a claim about the repo. (4 real instances; all `walking-the-ast.md`
                 walker examples whose comments say "so future slices don't break".)
2. PROVENANCE  — the hit sits under a DATED or VERSIONED ancestor heading
                 (`## 2026-04-30 …`, `## Release 1.1.44 …`, `### … slice 14`).
                 An append-only historical record describes what a PAST commit did;
                 "not fixed here" in a changelog entry is PROVENANCE of that commit's
                 scope, not open work. Verified: CHANGES.md / DEVELOPMENT_NOTES.md
                 have a deletion:addition ratio of 0.004 over their last 40 commits.
3. OWNED       — an owner token appears within +/-12 lines. The window is 12, not the
                 original 3, because a real closure annotation was measured sitting 5
                 lines above its hit (`project_every_parser_per_parser_book.md`) and
                 was being reported as an orphan.

Everything else is RESIDUE and must appear in the register with a terminal
disposition. Register rows are keyed by content hash, NOT by line number, so an
append above a hit does not orphan its triage — but EDITING the gap sentence does
(correctly: the claim changed, so re-triage it).

HONEST LIMITS (state them; do not over-claim)
---------------------------------------------
- This proves an owner was NAMED, not that the owner is real, active, or correct.
- PROVENANCE trusts the heading structure. A gap written under a dated heading but
  genuinely still open reads as provenance. That is the deliberate trade: the
  alternative (treat 5 years of changelog as backlog) buries the live signal.
- The register is hand-authored. It records judgement, and judgement can be wrong;
  what it cannot be is ABSENT, which was the actual failure mode.
"""

from __future__ import annotations

import hashlib
import os
import re
import subprocess
import sys

# Repo-root-relative by construction (repo may be moved/relocated at any time).
# PGEN_GAP_SWEEP_ROOT is a TEST seam only: it lets the probe driver run every arm
# against a hermetic synthetic repo instead of mutating the real tree. Production
# runs never set it.
ROOT = os.environ.get("PGEN_GAP_SWEEP_ROOT") or os.path.abspath(
    os.path.join(os.path.dirname(__file__), "..", "..", "..", "..")
)
REGISTER = os.path.join(
    ROOT, "docs", "tasks", "artifacts", "doctrine_gap_ownership", "gap_triage_register.tsv"
)

# --- what counts as a recorded gap -------------------------------------------------
# Prose gap language (docs). Deliberately excludes forward-looking doctrine prose like
# "is its own task-tree leaf", which states a RULE rather than recording a gap.
GAP_DOC = re.compile(
    r"watch item|known gap|known limitation|known soundness gap|worth a future|"
    r"hardening slice|future slice|not fixed here|not fixed in this leaf|left as-is|"
    r"left as is|re-open if|reopen if|residual recorded|deliberately not fixed|"
    r"out of scope for this leaf|follow-up slice|still missing|deferred to a future",
    re.I,
)
# Code markers.
#  - \b matters: without it "whack-a-mole" matches HACK — a measured false positive at
#    grammars/systemverilog.ebnf:426.
#  - XXX keeps its trailing colon (as the original driver had it). Bare \bXXX\b was
#    tried and measured: it matches the combinator suite's own test literals
#    `("xxx", true)` in 4 places. A marker convention needs the colon to be a marker.
GAP_CODE = re.compile(r"\b(TODO|FIXME|HACK)\b|XXX:|@todo", re.I)

# --- what counts as an owner -------------------------------------------------------
# Case-SENSITIVE by construction: lowercasing turns ordinary Rust method calls
# (`code.push`, `base.as`, `self.generate`) into "task-leaf IDs" and silently marks
# real TODOs as owned. That exact unsoundness was measured while building this.
OWNER = re.compile(
    r"[A-Z][A-Z0-9]{2,}[A-Z0-9-]*\.[0-9a-z]+"  # TREE.4 / RGX-0078.5.i.7
    r"|\b[A-Z]{2,}[A-Z0-9]*-[0-9]{3,}\b"       # VHDL-0001 / PGEN-RGX-0082 (defect IDs)
    r"|`\.[0-9a-z][0-9a-z.]*`"                 # `.b.6.2.36.2` (leading-dot leaf IDs)
    r"|routed to|owned by|tracked by|OWNER|new tree|new leaf"
    r"|\bRouted\b|\bCLOSED\b|closed by|no longer missing|see docs/tasks/"
)

# A heading is PROVENANCE-bearing when it carries a date, a version, or a slice/release id.
DATED = re.compile(
    r"\d{4}-\d{2}-\d{2}|\b\d+\.\d+\.\d+\b|\brelease\b|\bslice\s*\d|\bschema\s*\d"
    r"|\bv\d+\b|PGEN-[A-Z]|^Post\b|^After\b|^Before\b",
    re.I,
)
HEADING = re.compile(r"^(#{1,6})\s+(.*)$")

OWNER_WINDOW = 12


def surface_of(path: str) -> str:
    if path.startswith("docs/decisions/"):
        return "decisions"
    if path.startswith("docs/tasks/"):
        return "task-trees"
    if path.startswith("docs/book/"):
        return "book"
    if path.startswith("docs/contracts/"):
        return "contracts"
    if path.startswith("docs/reference/"):
        return "reference"
    if path.endswith(".rs"):
        return "rust-src"
    if path.endswith(".ebnf"):
        return "grammars"
    if path.endswith(".sh"):
        return "scripts"
    if path in ("CHANGES.md", "DEVELOPMENT_NOTES.md") or path.endswith("/changelog-index.md"):
        return "history-narrative"
    return "root-docs"


def key_of(path: str, line_text: str) -> str:
    """Content-addressed key: survives line movement, changes when the claim changes."""
    norm = " ".join(line_text.split())
    return hashlib.sha1(f"{path}\0{norm}".encode("utf-8", "replace")).hexdigest()[:12]


def tracked_files() -> list[str]:
    globs = [
        "docs/decisions/*.md", "docs/tasks/*.md", "docs/book/src/*.md",
        "docs/contracts/*.md", "docs/reference/*.md", "*.md",
        "rust/src/*.rs", "rust/scripts/*.sh", "scripts/*.sh", "grammars/*.ebnf",
    ]
    out = subprocess.run(
        ["git", "ls-files", "--", *globs], cwd=ROOT, capture_output=True, text=True, check=True
    )
    return [p for p in out.stdout.splitlines() if p]


def fence_mask(lines: list[str]) -> list[bool]:
    """True where a line sits inside a ``` fenced block."""
    inside, mask = False, []
    for ln in lines:
        if ln.lstrip().startswith("```"):
            inside = not inside
            mask.append(True)  # the fence marker itself is never a claim
        else:
            mask.append(inside)
    return mask


def ancestor_headings(lines: list[str], idx: int) -> list[str]:
    """Heading chain above idx, keeping strictly-decreasing levels (h3 < h2 < h1)."""
    chain, lvl = [], 99
    for i in range(min(idx, len(lines) - 1), -1, -1):
        m = HEADING.match(lines[i])
        if m and len(m.group(1)) < lvl:
            lvl = len(m.group(1))
            chain.append(m.group(2))
            if lvl == 1:
                break
    return chain


def classify(path: str, lines: list[str], idx: int, is_md: bool, fences: list[bool]) -> str:
    if is_md and fences[idx]:
        return "FENCE"
    if is_md and any(DATED.search(h) for h in ancestor_headings(lines, idx)):
        return "PROVENANCE"
    lo = max(0, idx - OWNER_WINDOW)
    hi = min(len(lines), idx + OWNER_WINDOW + 1)
    if OWNER.search("\n".join(lines[lo:hi])):
        return "OWNED"
    return "RESIDUE"


def sweep() -> list[dict]:
    hits = []
    for path in tracked_files():
        full = os.path.join(ROOT, path)
        try:
            with open(full, encoding="utf-8", errors="replace") as fh:
                text = fh.read()
        except OSError:
            continue
        lines = text.split("\n")
        is_md = path.endswith(".md")
        is_ebnf = path.endswith(".ebnf")
        # Same split as the original driver: prose surfaces get the gap-language
        # pattern; code surfaces (.rs/.sh/.ebnf) get the marker pattern.
        pattern = GAP_DOC if is_md else GAP_CODE
        fences = fence_mask(lines) if is_md else [False] * len(lines)
        for idx, raw in enumerate(lines):
            probe = raw
            if is_ebnf:
                # A grammar's quoted TERMINALS are the language it accepts, not a gap
                # note: semantic_annotation.ebnf legitimately defines the annotation
                # names "todo" | "fixme" | "bug". Strip quoted literals before matching.
                probe = re.sub(r'"[^"]*"', '""', probe)
            if not pattern.search(probe):
                continue
            hits.append(
                {
                    "path": path,
                    "line": idx + 1,
                    "surface": surface_of(path),
                    "verdict": classify(path, lines, idx, is_md, fences),
                    "key": key_of(path, raw),
                    "text": " ".join(raw.split())[:140],
                }
            )
    return hits


def load_register() -> dict[str, dict]:
    reg = {}
    if not os.path.exists(REGISTER):
        return reg
    with open(REGISTER, encoding="utf-8") as fh:
        for raw in fh:
            if not raw.strip() or raw.startswith("#"):
                continue
            parts = raw.rstrip("\n").split("\t")
            if len(parts) < 4:
                continue
            reg[parts[0]] = {
                "disposition": parts[1],
                "surface": parts[2],
                "owner": parts[3],
                "note": parts[4] if len(parts) > 4 else "",
            }
    return reg


TERMINAL = {
    "FALSE-POSITIVE",  # the regex matched non-gap text
    "PROVENANCE",      # historical record, accurate as of its entry
    "BOUNDARY",        # a published/deliberate support boundary, not a defect
    "DELIBERATE",      # a considered decision with a stated rationale
    "STALE-CLOSED",    # record read as open; work is DONE (owner = closing commit)
    "OWNED",           # a real gap already owned by a leaf (owner = leaf id)
    "ROUTED",          # a real gap; a leaf was opened for it (owner = leaf id)
}


def main() -> int:
    check = "--check" in sys.argv
    hits = sweep()
    reg = load_register()

    counts: dict[str, int] = {}
    for h in hits:
        counts[h["verdict"]] = counts.get(h["verdict"], 0) + 1

    residue = [h for h in hits if h["verdict"] == "RESIDUE"]
    untriaged = [h for h in residue if h["key"] not in reg]
    live_keys = {h["key"] for h in residue}
    stale_rows = [k for k in reg if k not in live_keys]
    bad_disp = [(k, v) for k, v in reg.items() if v["disposition"] not in TERMINAL]

    print("=== GAP-OWNERSHIP SWEEP + TRIAGE RATCHET (whole repository) ===")
    print(f"total gap hits            : {len(hits)}")
    for verdict in ("FENCE", "PROVENANCE", "OWNED", "RESIDUE"):
        print(f"  {verdict:<22}: {counts.get(verdict, 0)}")
    print()
    print(f"register rows             : {len(reg)}")
    print(f"  UNTRIAGED (must be 0)   : {len(untriaged)}")
    print(f"  stale rows (prune these): {len(stale_rows)}")
    print(f"  bad dispositions        : {len(bad_disp)}")

    if residue:
        by_disp: dict[str, int] = {}
        for h in residue:
            d = reg.get(h["key"], {}).get("disposition", "UNTRIAGED")
            by_disp[d] = by_disp.get(d, 0) + 1
        print("\n--- residue by disposition ---")
        for d in sorted(by_disp):
            print(f"  {d:<16} {by_disp[d]}")

    if untriaged:
        print("\n⛔ UNTRIAGED — classify each in gap_triage_register.tsv, or the gate fails:")
        for h in untriaged:
            print(f"  {h['key']}  {h['surface']:<11} {h['path']}:{h['line']}")
            print(f"               {h['text'][:110]}")
    if stale_rows:
        print("\n⚠️  STALE register rows (their gap text no longer exists — prune):")
        for k in stale_rows:
            print(f"  {k}  {reg[k]['disposition']:<16} {reg[k]['note'][:80]}")
    if bad_disp:
        print(f"\n⛔ dispositions must be one of {sorted(TERMINAL)}:")
        for k, v in bad_disp:
            print(f"  {k}  {v['disposition']}")

    if check and (untriaged or bad_disp):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
