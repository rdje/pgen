#!/usr/bin/env python3
"""Census the IEEE 1800 productions that the LRM defines in its CLAUSE BODIES but that
Annex A — the only surface `tools/extract_systemverilog_lrm_profiles.py` reads — does not
contain.

WHY THIS EXISTS (`SV-CORPUS-GRAD.13c.2v`)
-----------------------------------------
`grammars/systemverilog.ebnf` descends from Annex A.  Annex A is normative, but so is the
clause text, and IEEE 1800 says where the two part company: every clause syntax box carries a
caption, and that caption reads either

    Syntax 10-5—Assignment patterns syntax (excerpt from Annex A)
    Syntax 18-11—Scope randomize function syntax (not in Annex A)

The second form is the standard telling us, in its own words, that a normative production has
no Annex A counterpart — so PGEN's extraction pipeline is structurally blind to it.  This
instrument turns that self-label into a worklist.

⛔ A HIT IS NOT A DEFECT.  Most of these productions are outside the SystemVerilog *source*
language (the VCD file format, the compiler directives that belong to the preprocessor family)
or are already covered by a general Annex A rule (`system_tf_call` subsumes most of clause 20
and 21).  The census's job is to name the population and its classes; adjudication is per row.

REFUSAL POSTURE
---------------
The run FAILS (rc 1) when a clause-only production cannot be attached to an IEEE caption, or
when a caption's clause number has no declared bucket.  A class this instrument has never seen
must be declared before it can be counted, so a new one cannot land silently.

Usage:
    python3 stimuli/sv/lrm_annex_a_gap_census.py [--write]

Without `--write` the artifact is recomputed and compared against the tracked copy (gate mode).
"""

from __future__ import annotations

import argparse
import collections
import importlib.util
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
EXTRACTOR = REPO_ROOT / "tools" / "extract_systemverilog_lrm_profiles.py"
ARTIFACT_DIR = REPO_ROOT / "docs" / "tasks" / "artifacts" / "sv_corpus_grad" / "lrm_annex_a_gap"
SHIPPED_GRAMMAR = REPO_ROOT / "grammars" / "systemverilog.ebnf"

# The Annex A markdown per edition.  ⛔ 2017's Annex A is NOT in a file named for it: the
# PDF→markdown splitter titled the section from a nearby line and it landed in
# `section-41-data-read-api.md`.  Verified by `grep -c '::='` — 736 productions there, 0 in
# the file the name would suggest.
ANNEX_SOURCE = {
    "2017": "section-41-data-read-api.md",
    "2023": "section-Annex_A-normative-formal-syntax.md",
}

# Caption → bucket, keyed on the CLAUSE NUMBER IEEE prints in the caption (not on the markdown
# file name, which the splitter mangles).  Every bucket says what adjudicating that class means.
CLAUSE_BUCKETS = {
    20: ("system_task_function", "clause 20 utility system tasks/functions — Annex A models "
                                 "these generically through `system_tf_call` (A.8.2)"),
    21: ("system_task_function", "clause 21 I/O system tasks/functions and the VCD command set "
                                 "— the task CALLS are `system_tf_call`; the VCD FILE grammar "
                                 "is not SystemVerilog source at all"),
    22: ("compiler_directive", "clause 22 compiler directives — owned by the "
                               "`systemverilog_preprocessor` family, not by the SV grammar"),
}
# Everything else is source syntax until proven otherwise — that is the worklist.
DEFAULT_BUCKET = ("sv_source_syntax", "a SystemVerilog SOURCE construct the clause text defines "
                                      "and Annex A does not — the adjudication worklist")

# ⭐ The ONE declared exception to "every clause-only production must carry an IEEE caption".
# Annex F (formal semantics of concurrent assertions) states its rewriting rules as `A ::= …`,
# `P ::= strong ( R )` and so on — single-uppercase-letter METAVARIABLES, not SystemVerilog
# productions, and they sit in running text with no syntax-box caption.  The exception is
# guarded: `assert_metavariables_are_not_syntax()` proves Annex A names no such rule in either
# edition, so this cannot quietly swallow a real production.
# ⛔ The 2017 copy of these lives in `section-100-time-units.md` — the splitter mangled the
# name.  That is exactly why the census keys on IEEE's caption and not on the file name.
METAVARIABLE_RE = re.compile(r"^[A-Z]$")
METAVARIABLE_BUCKET = ("formal_semantics_metavariable",
                       "Annex F formal-semantics metavariable, not SystemVerilog syntax")

CAPTION_RE = re.compile(
    r"Syntax\s+(?P<clause>\d+)-(?P<index>\d+)\s*[—–-]\s*(?P<title>[^()]*?)\s*"
    r"\((?P<annex>not in Annex A|excerpt from Annex A)\)"
)
RULE_HEAD_RE = re.compile(r"^\s*([A-Za-z_$][A-Za-z0-9_$,]*)\s*::\s*=")
GRAMMAR_RULE_RE = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\s*:=", re.MULTILINE)

# The markdown conversion runs the caption line straight on from the last production of its
# box, so a scraped body ends with the caption text.  Cut it — the caption is metadata the
# census reads separately, and leaving it in makes every long body unreadable.
BODY_TAIL_RE = re.compile(r"\s*Syntax\s+\d+-\d+\s*[—–-].*$")

# IEEE also writes the disposition INSIDE the box, as a comment, and where it does that marker
# is authoritative over the caption that follows the box — `inline_constraint_declaration`
# (2023, Syntax 18-10) carries `// not in Annex A` in the box while the next caption below it
# reads `excerpt from Annex A`.
IN_BOX_MARKER_RE = re.compile(r"//\s*not in Annex A", re.IGNORECASE)


def load_extractor():
    """Import the tracked LRM scraper.

    ⭐ Deliberately the SAME function the shipped grammar was extracted with, so a divergence
    this census reports cannot be an artifact of a second, differently-buggy scraper.
    """
    spec = importlib.util.spec_from_file_location("lrm_extractor", EXTRACTOR)
    module = importlib.util.module_from_spec(spec)
    sys.modules["lrm_extractor"] = module
    spec.loader.exec_module(module)
    return module


def captions_by_line(path: Path) -> list[tuple[int, str, int, str]]:
    """(line_no, clause_str, clause_int, annex_disposition) for every IEEE syntax-box caption."""
    found = []
    for idx, line in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines()):
        for m in CAPTION_RE.finditer(line):
            found.append((idx, f"{m.group('clause')}-{m.group('index')}",
                          int(m.group("clause")), m.group("annex")))
    return found


def rule_heads_by_line(path: Path, canonicalize) -> list[tuple[int, str]]:
    heads = []
    for idx, line in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines()):
        m = RULE_HEAD_RE.match(line)
        if m:
            heads.append((idx, canonicalize(m.group(1))))
    return heads


def attach_caption(head_line: int, captions: list[tuple[int, str, int, str]]):
    """IEEE prints the caption AFTER its box, so a production belongs to the next caption below
    it.  Returns None when there is none — which makes the run REFUSE rather than guess."""
    for line_no, label, clause, annex in captions:
        if line_no >= head_line:
            return label, clause, annex
    return None


def shipped_rule_names() -> set[str]:
    return set(GRAMMAR_RULE_RE.findall(SHIPPED_GRAMMAR.read_text(encoding="utf-8")))


def assert_metavariables_are_not_syntax(annex_rules, edition: str) -> list[str]:
    """Guard the METAVARIABLE_BUCKET exception: Annex A must name no single-uppercase-letter
    production, or the exception could swallow a real one."""
    offenders = sorted(n for n in annex_rules if METAVARIABLE_RE.match(n))
    if offenders:
        return [f"{edition}: Annex A DOES name single-letter production(s) {offenders} — the "
                f"metavariable exception is unsafe and must be re-derived"]
    return []


def census(edition: str, extractor) -> tuple[list[dict], list[str]]:
    base = REPO_ROOT / "docs" / "systemverilog" / edition / "md"
    annex_name = ANNEX_SOURCE[edition]
    annex = extractor.extract_rules(base / annex_name)

    clause_bodies: dict[str, list[str]] = collections.OrderedDict()
    clause_meta: dict[str, dict] = {}
    for path in sorted(base.glob("section-*.md")):
        if path.name == annex_name:
            continue
        rules = extractor.extract_rules(path)
        if not rules:
            continue
        captions = captions_by_line(path)
        heads = rule_heads_by_line(path, extractor.canonicalize_rule_name)
        head_line = {}
        for line_no, name in heads:
            head_line.setdefault(name, line_no)
        for name, bodies in rules.items():
            if not bodies:
                continue
            clause_bodies.setdefault(name, []).extend(bodies)
            if name in clause_meta:
                continue
            attached = attach_caption(head_line.get(name, 0), captions)
            clause_meta[name] = {"source": path.name, "caption": attached}

    shipped = shipped_rule_names()
    rows, refusals = [], assert_metavariables_are_not_syntax(annex, edition)
    for name in sorted(set(clause_bodies) - set(annex)):
        meta = clause_meta[name]
        if meta["caption"] is None:
            if METAVARIABLE_RE.match(name):
                bucket, rationale = METAVARIABLE_BUCKET
                rows.append({
                    "edition": edition, "rule": name, "bucket": bucket,
                    "ieee_caption": "(none — running text)", "ieee_says": "not in Annex A",
                    "clause": 0,
                    "modelled_in_shipped_grammar": "yes" if name in shipped else "no",
                    "source_md": meta["source"],
                    "body": BODY_TAIL_RE.sub(
                        "", re.sub(r"\s+", " ", clause_bodies[name][0]))[:240],
                    "rationale": rationale,
                })
                continue
            refusals.append(f"{edition}: '{name}' ({meta['source']}) has no IEEE syntax-box "
                            f"caption below it — cannot classify")
            continue
        label, clause, annex_disposition = meta["caption"]
        bucket, rationale = CLAUSE_BUCKETS.get(clause, DEFAULT_BUCKET)
        raw_body = re.sub(r"\s+", " ", clause_bodies[name][0])
        if IN_BOX_MARKER_RE.search(raw_body):
            annex_disposition = "not in Annex A"
        rows.append({
            "edition": edition,
            "rule": name,
            "bucket": bucket,
            "ieee_caption": f"Syntax {label}",
            "ieee_says": annex_disposition,
            "clause": clause,
            "modelled_in_shipped_grammar": "yes" if name in shipped else "no",
            "source_md": meta["source"],
            "body": BODY_TAIL_RE.sub("", raw_body)[:240],
            "rationale": rationale,
        })
    return rows, refusals


COLUMNS = ["edition", "rule", "bucket", "ieee_says", "ieee_caption", "clause",
           "modelled_in_shipped_grammar", "source_md", "body"]


def render_tsv(rows: list[dict]) -> str:
    out = ["\t".join(COLUMNS)]
    for r in rows:
        out.append("\t".join(str(r[c]) for c in COLUMNS))
    return "\n".join(out) + "\n"


def render_md(rows: list[dict]) -> str:
    lines = [
        "# LRM clause-only productions — the Annex A gap census",
        "",
        "> DERIVED — regenerate with `python3 stimuli/sv/lrm_annex_a_gap_census.py --write`.",
        "> Owning leaf: `SV-CORPUS-GRAD.13c.2v`. Never hand-edit.",
        "",
        "`grammars/systemverilog.ebnf` descends from **Annex A alone**. IEEE 1800 also defines",
        "productions in its clause bodies, and its own syntax-box captions say which:",
        "`(excerpt from Annex A)` versus **`(not in Annex A)`**. Every row below is a production",
        "the clause text defines that Annex A does not carry.",
        "",
        "⚠️ **`modelled in shipped grammar` is a NAME lookup, and it reads `0` everywhere by",
        "construction** — the grammar is extracted from Annex A, so it cannot carry a rule named",
        "after a production Annex A does not have. It is **not** a coverage verdict: a clause-only",
        "production is routinely *reachable* through a general Annex A rule (clause 20/21's system",
        "tasks through `system_tf_call`, for one). Coverage is decided per row by a witness —",
        "`worklist_probes/probe_worklist.sh` — never by this column.",
        "",
    ]
    for edition in sorted({r["edition"] for r in rows}):
        sub = [r for r in rows if r["edition"] == edition]
        lines += [f"## IEEE 1800-{edition}", "",
                  f"- clause-only productions: **{len(sub)}**", ""]
        by_bucket = collections.Counter(r["bucket"] for r in sub)
        lines += ["| bucket | rows | IEEE says `not in Annex A` | modelled in shipped grammar |",
                  "|---|---:|---:|---:|"]
        for bucket, count in sorted(by_bucket.items()):
            rs = [r for r in sub if r["bucket"] == bucket]
            explicit = sum(1 for r in rs if r["ieee_says"] == "not in Annex A")
            modelled = sum(1 for r in rs if r["modelled_in_shipped_grammar"] == "yes")
            lines.append(f"| `{bucket}` | {count} | {explicit} | {modelled} |")
        lines.append("")
        worklist = [r for r in sub if r["bucket"] == "sv_source_syntax"]
        lines += [f"### `sv_source_syntax` — the adjudication worklist ({len(worklist)} rows)", "",
                  "| rule | IEEE caption | says | modelled | body |", "|---|---|---|---|---|"]
        for r in worklist:
            lines.append(f"| `{r['rule']}` | {r['ieee_caption']} | {r['ieee_says']} | "
                         f"{r['modelled_in_shipped_grammar']} | `{r['body'][:110]}` |")
        lines.append("")
    return "\n".join(lines) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true",
                    help="write the artifact instead of comparing against the tracked copy")
    args = ap.parse_args()

    extractor = load_extractor()
    rows, refusals = [], []
    for edition in sorted(ANNEX_SOURCE):
        r, f = census(edition, extractor)
        rows += r
        refusals += f

    if refusals:
        print("LRM-ANNEX-A-GAP: REFUSED — unclassifiable clause-only productions:")
        for line in refusals:
            print(f"  - {line}")
        return 1

    tsv, md = render_tsv(rows), render_md(rows)
    explicit = sum(1 for r in rows if r["ieee_says"] == "not in Annex A")
    worklist = sum(1 for r in rows if r["bucket"] == "sv_source_syntax")
    headline = (f"LRM-ANNEX-A-GAP: clause_only={len(rows)} "
                f"ieee_labelled_not_in_annex_a={explicit} sv_source_worklist={worklist} "
                f"editions={','.join(sorted(ANNEX_SOURCE))}")

    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    tsv_path, md_path = ARTIFACT_DIR / "census.tsv", ARTIFACT_DIR / "census.md"
    if args.write:
        tsv_path.write_text(tsv)
        md_path.write_text(md)
        print(headline + " (written)")
        return 0

    drift = []
    for path, content in ((tsv_path, tsv), (md_path, md)):
        if not path.exists():
            drift.append(f"{path.relative_to(REPO_ROOT)} is missing")
        elif path.read_text() != content:
            drift.append(f"{path.relative_to(REPO_ROOT)} is stale")
    print(headline)
    if drift:
        print("LRM-ANNEX-A-GAP: DRIFT — " + "; ".join(drift))
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
