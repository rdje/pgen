#!/usr/bin/env python3
"""SV external-corpus LRM-CLAUSE-COVERAGE + NEGATIVES-DENSITY instrument
(SV-CORPUS-GRAD.7b).

The complementary lens to the rule-coverage instrument
(`corpus_rule_coverage.py`, leaf `.7a`). Where `.7a` measures which *grammar
rules* the corpus exercises — the authoritative parseable-surface denominator
(91.1% / 120 gaps for `sv_2017`) — this instrument measures which *LRM clauses*
the KEYED corpus suites carry a case for, grouped by LRM chapter, and reports
the density of keyed NEGATIVES (`must_reject`) per chapter (the thinnest axis
per the session-#191 corpus-sufficiency assessment).

This is a STRUCTURAL lens, not a second competing coverage percentage: only
three suites carry clause metadata (sv-tests `:tags:`, ispras clause-encoded
filenames), so the clause matrix speaks for a ~14% keyed slice of the vendored
universe. The authoritative "100% of the parseable surface" denominator remains
`.7a`'s rule inventory; this view answers a different question — *which LRM
chapters/clauses does the corpus deliberately target, and where is the keyed
gap (a whole chapter with no case, or a chapter with no negatives)?* — feeding
the `.9` gap-driven acquisition loop with an LRM-structure worklist.

KEYED SUITES (numerator; edition kept distinct — clause numbers are NOT
comparable across editions, only chapter numbers are broadly stable within the
1800 family):
  sv-tests             `:tags:` clause numbers          edition 1800-2017
  ispras (1800-2012)   `NN/NN.MM.KK_v.sv`  (dotted)     edition 1800-2012
  ispras (1364-2005)   `test_NN_MM_KK_v.v` (underscore) edition 1364-2005 (Verilog)

UNKEYED suites (opentitan, iverilog/ivtest, verilator, sv2v, Surelog, the
real-design corpora, uvm-core) carry no clause metadata — they are counted in
the totals but contribute no clause key. ivtest's `.list` `CE` (compile-error)
rows are negatives, but not clause-mapped; that unkeyed-negative mass is
reported alongside the keyed negatives so the thin keyed axis is not mistaken
for the whole negative surface.

Deterministic: reads only the committed adjudication manifest + static suite
metadata; sorted, content-derived outputs; the TSV + report are tracked and
diffable across sessions. No parser or codegen run — pure measurement.

Two manifests are read so each edition's verdict is authoritative: the main
`adjudication_manifest.tsv` (sv_2017 profile) owns the 1800-family keyed rows
(sv-tests 2017 + ispras 2012), while `adjudication_manifest_v2005.tsv`
(verilog_2005 profile) owns the 1364-2005 keyed rows — the same ispras
`ieee-1364-2005/` files are DEFERRED (`out_of_scope_with_cause:v2005_profile_lane`)
in the main manifest, so their real accept/reject verdict only exists in the
v2005 manifest. Reading both avoids both double-counting and a false all-zero
Verilog lane.

Usage:
  python3 stimuli/sv/corpus_clause_coverage.py \
      [--manifest stimuli/sv/characterization/adjudication_manifest.tsv] \
      [--v2005-manifest stimuli/sv/characterization/adjudication_manifest_v2005.tsv] \
      [--subs-root stimuli/sv/subs] \
      [--out-prefix stimuli/sv/characterization/clause_coverage]
"""

import argparse
import re
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent  # repo root

# --- IEEE 1800-2017 clause (chapter) titles — the LRM structural universe.
# Reference data from the standard's table of contents (the in-repo markdown
# under docs/systemverilog/2017/md/ mangles a few section titles into body
# text, so the canonical titles are encoded here directly). Chapters 3-35 are
# the parse-bearing surface the SV grammar targets; 1-4 (overview / references
# / building blocks / scheduling) and 36-41 (PLI/VPI/API) carry no primary
# parse surface -> N/A-with-cause on the density view.
LRM_1800_CHAPTERS = {
    1: "Overview",
    2: "Normative references",
    3: "Design and verification building blocks",
    4: "Scheduling semantics",
    5: "Lexical conventions",
    6: "Data types",
    7: "Aggregate data types",
    8: "Classes",
    9: "Processes",
    10: "Assignment statements",
    11: "Operators and expressions",
    12: "Procedural programming statements",
    13: "Tasks and functions (subroutines)",
    14: "Clocking blocks",
    15: "Interprocess synchronization and communication",
    16: "Assertions",
    17: "Checkers",
    18: "Constrained random value generation",
    19: "Functional coverage",
    20: "Utility system tasks and system functions",
    21: "Input/output system tasks and system functions",
    22: "Compiler directives",
    23: "Modules and hierarchy",
    24: "Programs",
    25: "Interfaces",
    26: "Packages",
    27: "Generate constructs",
    28: "Gate-level and switch-level modeling",
    29: "User-defined primitives",
    30: "Specify blocks",
    31: "Timing checks",
    32: "Backannotation using the standard delay format (SDF)",
    33: "Configuring the contents of a design",
    34: "Protected envelopes",
    35: "Direct programming interface (DPI)",
    36: "Programming language interface (PLI/VPI) overview",
    37: "VPI object model diagrams",
    38: "VPI routine definitions",
    39: "Assertion API",
    40: "Code coverage control and API",
    41: "Data read API",
}
# Parse-bearing chapters (have SV syntax the grammar must accept); the rest are
# N/A-with-cause (no parse surface) so an empty chapter there is not a gap.
PARSE_BEARING_1800 = set(range(5, 36))  # 5..35 inclusive


def norm_clause(components):
    """Normalize a list of numeric components into a dotted clause key with
    leading zeros stripped per component: ['03','05','01'] -> '3.5.1'."""
    return ".".join(str(int(c)) for c in components)


def clause_from_sv_tests(subs_root, relpath):
    """sv-tests: authoritative clause keys are the in-file `:tags:`. Numeric
    tags (`^\\d+(\\.\\d+)*$`) are clause keys (edition 1800-2017); non-numeric
    tags (uvm, sanity, ...) are feature tags. Falls back to the `chapter-N/`
    directory clause if the file is unreadable or carries no numeric tag."""
    clauses, feature_tags = [], []
    fpath = subs_root / "sv-tests" / relpath
    try:
        for line in fpath.read_text(encoding="utf-8", errors="replace").splitlines():
            m = re.match(r"^:tags:\s*(.*)$", line)
            if not m:
                continue
            for tok in m.group(1).split():
                if re.fullmatch(r"\d+(\.\d+)*", tok):
                    clauses.append(norm_clause(tok.split(".")))
                else:
                    feature_tags.append(tok)
            break  # a file has a single :tags: line
    except OSError:
        pass
    if not clauses:
        # fallback: `tests/chapter-N/N.M--desc.sv` encodes the clause too
        m = re.search(r"/(\d+(?:\.\d+)*)[-.]", "/" + relpath.replace("chapter-", ""))
        if m:
            clauses.append(norm_clause(m.group(1).split(".")))
    return sorted(set(clauses)), sorted(set(feature_tags))


def clause_from_ispras(relpath):
    """ispras: the clause is encoded in the path. Two edition formats:
      1800-2012:  ieee-1800-2012/NN/<clause>_<variant>.sv   (dotted clause)
      1364-2005:  ieee-1364-2005/test_<c>_<s>_<ss>_<variant>.v (underscored)
    In both, the trailing `_<variant>` index is dropped; the remainder is the
    clause. `ieee-1364-2005/parts/*` are shared helper files with no clause."""
    parts = relpath.split("/")
    if parts[0] == "ieee-1800-2012":
        stem = Path(parts[-1]).stem            # e.g. "22.05.01_04"
        clause_txt = stem.split("_")[0]        # drop the _NN variant -> "22.05.01"
        comps = clause_txt.split(".")
        if all(c.isdigit() for c in comps) and comps:
            return "1800-2012", norm_clause(comps)
        return "1800-2012", None
    if parts[0] == "ieee-1364-2005":
        if len(parts) >= 2 and parts[1] == "parts":
            return "1364-2005", None           # shared helper, unkeyed
        stem = Path(parts[-1]).stem            # e.g. "test_03_05_01_1"
        m = re.fullmatch(r"test_(\d+(?:_\d+)*)", stem)
        if not m:
            return "1364-2005", None
        comps = m.group(1).split("_")
        if len(comps) >= 2:
            comps = comps[:-1]                 # drop the trailing variant index
        return "1364-2005", norm_clause(comps)
    return None, None


POSITIVE = "must_accept"
NEGATIVE = "must_reject"


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--manifest",
                    default=ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv",
                    type=Path)
    ap.add_argument("--v2005-manifest",
                    default=ROOT / "stimuli/sv/characterization/adjudication_manifest_v2005.tsv",
                    type=Path)
    ap.add_argument("--subs-root", default=ROOT / "stimuli/sv/subs", type=Path)
    ap.add_argument("--out-prefix",
                    default=ROOT / "stimuli/sv/characterization/clause_coverage",
                    type=Path)
    args = ap.parse_args()

    # per (edition, clause): case/verdict tallies + contributing suites
    clause_stats = defaultdict(lambda: {
        "cases": 0, "positive": 0, "negative": 0,
        "obs_pass": 0, "obs_fail": 0, "suites": set()})
    feature_tag_counts = defaultdict(int)          # non-clause sv-tests tags
    unkeyed_total = 0                              # main-manifest unkeyed rows
    # unkeyed negatives kept per lane (sv_2017 vs verilog_2005 profiles differ)
    unkeyed_neg = {"sv_2017": defaultdict(int), "verilog_2005": defaultdict(int)}
    total_rows = 0

    def keyed_of(suite, relpath):
        """(edition, [clauses], [feature_tags]) for a row, or (None, [], [])."""
        if suite == "sv-tests":
            cl, feats = clause_from_sv_tests(args.subs_root, relpath)
            return ("1800-2017", cl, feats)
        if suite == "ispras-sv-tests":
            ed, clause = clause_from_ispras(relpath)
            return (ed, [clause] if clause is not None else [], [])
        return (None, [], [])

    def ingest(st, expected, observed, suite):
        st["cases"] += 1
        st["suites"].add(suite)
        if expected == POSITIVE:
            st["positive"] += 1
        elif expected == NEGATIVE:
            st["negative"] += 1
        if observed == "pass":
            st["obs_pass"] += 1
        elif observed in ("fail", "crash", "timeout"):
            st["obs_fail"] += 1

    def read_manifest(path):
        rows = path.read_text(encoding="utf-8").splitlines()
        ci = {name: i for i, name in enumerate(rows[0].split("\t"))}
        for line in rows[1:]:
            if line.strip():
                f = line.split("\t")
                yield (f[ci["suite"]], f[ci["relpath"]],
                       f[ci["expected"]], f[ci["observed"]])

    # --- main manifest (sv_2017 profile): 1800-family keyed rows are
    #     authoritative here; the 1364-2005 rows present here are DEFERRED to
    #     the v2005 lane, so their keyed verdict is taken from the v2005
    #     manifest below (skip them here to avoid a false all-zero verdict).
    for suite, relpath, expected, observed in read_manifest(args.manifest):
        total_rows += 1
        edition, clauses, feats = keyed_of(suite, relpath)
        for t in feats:
            feature_tag_counts[t] += 1
        if clauses and edition != "1364-2005":
            for clause in clauses:
                ingest(clause_stats[(edition, clause)], expected, observed, suite)
        else:
            unkeyed_total += 1
            if expected == NEGATIVE:
                unkeyed_neg["sv_2017"][suite] += 1

    # --- v2005 manifest (verilog_2005 profile): owns the 1364-2005 keyed
    #     verdicts + its own unkeyed negatives (ivtest CE, sv2v bad-goldens).
    v2005_rows = 0
    if args.v2005_manifest.exists():
        for suite, relpath, expected, observed in read_manifest(args.v2005_manifest):
            v2005_rows += 1
            edition, clauses, _ = keyed_of(suite, relpath)
            if clauses and edition == "1364-2005":
                for clause in clauses:
                    ingest(clause_stats[(edition, clause)], expected, observed, suite)
            elif expected == NEGATIVE:
                unkeyed_neg["verilog_2005"][suite] += 1

    # ---- tracked TSV (per (edition, clause), machine-diffable)
    def chapter_of(clause):
        return int(clause.split(".")[0])

    def clause_sortkey(item):
        (edition, clause), _ = item
        comps = tuple(int(x) for x in clause.split("."))
        return (edition, comps)

    tsv_path = Path(f"{args.out_prefix}.tsv")
    ordered = sorted(clause_stats.items(), key=clause_sortkey)
    with tsv_path.open("w", encoding="utf-8") as fh:
        fh.write("edition\tchapter\tclause\tcases\tpositive\tnegative\t"
                 "observed_pass\tobserved_fail\tsuites\n")
        for (edition, clause), st in ordered:
            fh.write(f"{edition}\t{chapter_of(clause)}\t{clause}\t{st['cases']}\t"
                     f"{st['positive']}\t{st['negative']}\t{st['obs_pass']}\t"
                     f"{st['obs_fail']}\t{','.join(sorted(st['suites']))}\n")

    # ---- per-chapter density aggregation
    # 1800 family (sv-tests 2017 + ispras 2012): chapter numbers align at the
    # top level, so aggregate them together under the 1800-2017 titles.
    ch_1800 = defaultdict(lambda: {"clauses": set(), "cases": 0,
                                   "positive": 0, "negative": 0})
    ch_v2005 = defaultdict(lambda: {"clauses": set(), "cases": 0,
                                    "positive": 0, "negative": 0})
    for (edition, clause), st in clause_stats.items():
        bucket = ch_v2005 if edition == "1364-2005" else ch_1800
        c = bucket[chapter_of(clause)]
        c["clauses"].add(clause)
        c["cases"] += st["cases"]
        c["positive"] += st["positive"]
        c["negative"] += st["negative"]

    keyed_rows = sum(st["cases"] for st in clause_stats.values())
    distinct_1800 = sum(1 for (e, _) in clause_stats if e != "1364-2005")
    distinct_v2005 = sum(1 for (e, _) in clause_stats if e == "1364-2005")
    total_negative_keyed = sum(st["negative"] for st in clause_stats.values())
    unkeyed_neg_sv2017 = sum(unkeyed_neg["sv_2017"].values())
    unkeyed_neg_v2005 = sum(unkeyed_neg["verilog_2005"].values())

    # ---- tracked report
    L = []
    L.append("# SV external-corpus LRM-clause coverage + negatives density "
             "(SV-CORPUS-GRAD.7b)")
    L.append("")
    L.append("Generated by `stimuli/sv/corpus_clause_coverage.py` (deterministic; "
             "reads the committed adjudication manifest + static suite metadata — "
             "no parser run). The **structural companion** to `.7a`'s rule "
             "coverage: `.7a` owns the authoritative parseable-surface "
             "denominator (grammar-rule participation, 91.1% / 120 gaps for "
             "`sv_2017`); this view maps the KEYED corpus onto the LRM's own "
             "clause/chapter structure and measures the negative-case density.")
    L.append("")
    L.append("## Scope + honest limits")
    L.append("")
    L.append("- Only **three keyed inputs** carry clause metadata: sv-tests "
             "`:tags:` (edition 1800-2017), ispras `ieee-1800-2012/` dotted "
             "filenames, ispras `ieee-1364-2005/` `test_*` filenames (Verilog). "
             "The other ~13k unkeyed rows (opentitan, iverilog/ivtest, "
             "verilator, sv2v, Surelog, real designs, uvm-core) carry NO clause "
             "key and are covered by `.7a`'s rule lens instead.")
    L.append("- This is a **structural lens, not a competing coverage %**: a "
             "clause is \"exercised\" iff >=1 keyed case names it. There is no "
             "fabricated all-clauses denominator (the LRM's sub-clause universe "
             "is semantic-heavy and not a clean parse denominator); the honest "
             "gaps here are **chapter-granularity** (a parse-bearing chapter "
             "with zero keyed cases) and **negative-axis** (a chapter with no "
             "keyed `must_reject`).")
    L.append("- **Edition boundary kept**: clause NUMBERS are not comparable "
             "across editions. The 1800 family (sv-tests 2017 + ispras 2012) is "
             "aggregated at CHAPTER granularity only (top-level chapters are "
             "stable 2012->2017); the 1364-2005 Verilog lane is reported "
             "separately. Clause-level detail in the TSV is tagged with its "
             "edition.")
    L.append("")
    L.append("| metric | value |")
    L.append("|---|---|")
    L.append(f"| main manifest rows total (sv_2017) | {total_rows} |")
    L.append(f"| v2005 manifest rows total (verilog_2005) | {v2005_rows} |")
    L.append(f"| clause-keyed cases (numerator) | {keyed_rows} |")
    L.append(f"| unkeyed cases, main manifest (rule-lens only) | {unkeyed_total} |")
    L.append(f"| distinct (edition,clause) keys — 1800 family | {distinct_1800} |")
    L.append(f"| distinct (edition,clause) keys — 1364-2005 | {distinct_v2005} |")
    L.append(f"| keyed negatives (`must_reject`) | {total_negative_keyed} |")
    L.append(f"| unkeyed negatives — sv_2017 lane | {unkeyed_neg_sv2017} |")
    L.append(f"| unkeyed negatives — verilog_2005 lane | {unkeyed_neg_v2005} |")
    L.append("")

    # ---- 1800-family per-chapter density (the primary SV view)
    L.append("## Per-chapter density — 1800 family (sv-tests 2017 + ispras 2012)")
    L.append("")
    L.append("`parse-bearing` = the chapter defines SV syntax the grammar must "
             "accept (clauses 5-35). Non-parse-bearing chapters (1-4 overview / "
             "references / building blocks / scheduling; 36-41 PLI/VPI/API) are "
             "**N/A-with-cause** — an empty cell there is not a gap. A "
             "parse-bearing chapter with **0 clauses** is a keyed-corpus GAP for "
             "the `.9` worklist.")
    L.append("")
    L.append("| ch | title | parse-bearing | clauses keyed | cases | positive | negative | note |")
    L.append("|---:|---|:---:|---:|---:|---:|---:|---|")
    for ch in range(1, 42):
        title = LRM_1800_CHAPTERS.get(ch, "?")
        pb = ch in PARSE_BEARING_1800
        d = ch_1800.get(ch)
        nclauses = len(d["clauses"]) if d else 0
        cases = d["cases"] if d else 0
        pos = d["positive"] if d else 0
        neg = d["negative"] if d else 0
        if not pb:
            # non-primary parse surface; ispras nonetheless keyed a few cases
            # against some of these chapters — don't claim "no parse surface"
            # when cases exist.
            note = (f"N/A surface — {cases} keyed case(s)" if cases
                    else "N/A (no parse surface)")
        elif nclauses == 0:
            note = "**GAP — no keyed case**"
        elif neg == 0:
            note = "no keyed negative"
        else:
            note = ""
        L.append(f"| {ch} | {title} | {'yes' if pb else 'no'} | {nclauses} | "
                 f"{cases} | {pos} | {neg} | {note} |")
    L.append("")

    parse_bearing_gaps = sorted(
        ch for ch in PARSE_BEARING_1800 if len(ch_1800.get(ch, {}).get("clauses", ())) == 0)
    L.append(f"**Parse-bearing chapters with ZERO keyed cases ({len(parse_bearing_gaps)}):** "
             + (", ".join(f"{ch} ({LRM_1800_CHAPTERS[ch]})" for ch in parse_bearing_gaps)
                if parse_bearing_gaps else "none — every parse-bearing chapter has >=1 keyed case."))
    L.append("")

    # ---- negatives density (the thinnest axis)
    L.append("## Negatives density (keyed `must_reject` per chapter — the thinnest axis)")
    L.append("")
    L.append("Per the session-#191 corpus-sufficiency assessment, the "
             "parse-level negative axis is the thinnest in the whole corpus. "
             "Keyed negatives (clause-mappable) below; the bulk of the corpus's "
             "negatives are **unkeyed** (ivtest `CE` rows, sv2v bad-goldens) and "
             "cannot be attributed to an LRM chapter without per-file "
             "adjudication.")
    L.append("")
    L.append("| ch | title | keyed negatives |")
    L.append("|---:|---|---:|")
    any_neg = False
    for ch in range(1, 42):
        d = ch_1800.get(ch)
        neg = d["negative"] if d else 0
        if neg:
            any_neg = True
            L.append(f"| {ch} | {LRM_1800_CHAPTERS.get(ch, '?')} | {neg} |")
    if not any_neg:
        L.append("| — | (no keyed negatives in the 1800 family) | 0 |")
    L.append("")
    L.append(f"- keyed 1800-family negatives total: "
             f"{sum(d['negative'] for d in ch_1800.values())}")
    L.append(f"- keyed 1364-2005 negatives total: "
             f"{sum(d['negative'] for d in ch_v2005.values())} "
             f"(the verilog_2005 negative axis is entirely UNKEYED — see below)")
    L.append(f"- **unkeyed negatives — sv_2017 lane** (`must_reject`, not "
             f"clause-mapped): {unkeyed_neg_sv2017}")
    for suite in sorted(unkeyed_neg["sv_2017"]):
        L.append(f"  - {suite}: {unkeyed_neg['sv_2017'][suite]}")
    L.append(f"- **unkeyed negatives — verilog_2005 lane**: {unkeyed_neg_v2005}")
    for suite in sorted(unkeyed_neg["verilog_2005"]):
        L.append(f"  - {suite}: {unkeyed_neg['verilog_2005'][suite]}")
    L.append("")

    # ---- 1364-2005 Verilog lane (chapter numbers only; titles per the 1364 ToC)
    L.append("## Per-chapter density — 1364-2005 Verilog lane (ispras `test_*`)")
    L.append("")
    L.append("Chapter numbers follow the IEEE 1364-2005 ToC (distinct from the "
             "1800 numbering); titles are intentionally omitted to avoid a "
             "cross-edition title merge. Verdicts here are taken from the "
             "`verilog_2005`-profile manifest (`adjudication_manifest_v2005.tsv`) "
             "where these ispras `test_*` files are adjudicated — in the main "
             "sv_2017 manifest they are deferred to this lane.")
    L.append("")
    L.append("| ch (1364-2005) | clauses keyed | cases | positive | negative |")
    L.append("|---:|---:|---:|---:|---:|")
    for ch in sorted(ch_v2005):
        d = ch_v2005[ch]
        L.append(f"| {ch} | {len(d['clauses'])} | {d['cases']} | "
                 f"{d['positive']} | {d['negative']} |")
    L.append("")

    # ---- feature (non-clause) tags
    L.append("## sv-tests feature tags (non-clause `:tags:`)")
    L.append("")
    L.append("Non-numeric sv-tests tags label a construct FAMILY rather than an "
             "LRM clause; reported for completeness (not part of the clause "
             "matrix).")
    L.append("")
    L.append("| tag | files |")
    L.append("|---|---:|")
    for tag in sorted(feature_tag_counts, key=lambda t: (-feature_tag_counts[t], t)):
        L.append(f"| {tag} | {feature_tag_counts[tag]} |")
    L.append("")

    # ---- join with .7a
    L.append("## Join with `.7a` (rule coverage) — how `.9` uses both")
    L.append("")
    L.append("- `.7a` is the **authoritative** parseable-surface measure: "
             "grammar-rule participation, 1,223/1,343 = **91.1%** for `sv_2017`, "
             "**120 uncovered rules** = the primary `.9` worklist "
             "(`rule_coverage_sv_2017.tsv`).")
    L.append("- `.7b` (this view) is the **LRM-structure lens**: it says which "
             "LRM chapters/clauses the keyed corpus deliberately targets, and "
             "surfaces two structural gaps `.7a` cannot express — a parse-bearing "
             "chapter with no keyed case, and a chapter with no keyed negative.")
    L.append("- `.9` closes gaps from BOTH: for each `.7a` uncovered rule, source "
             "or craft a case (LRM clause cited); for each `.7b` chapter/negative "
             "gap, add a clause-keyed positive/negative. The two lenses are "
             "complementary, not redundant — a rule can be covered while its "
             "chapter has no keyed negative, and vice versa.")
    L.append("")

    md_path = Path(f"{args.out_prefix}.md")
    md_path.write_text("\n".join(L), encoding="utf-8")

    print(f"clause-coverage: {keyed_rows} keyed cases over "
          f"{distinct_1800} 1800-family + {distinct_v2005} 1364-2005 clauses; "
          f"{len(parse_bearing_gaps)} parse-bearing chapter gaps; "
          f"{total_negative_keyed} keyed negatives; unkeyed negatives "
          f"sv_2017={unkeyed_neg_sv2017} verilog_2005={unkeyed_neg_v2005}")
    print(f"report:  {md_path}")
    print(f"per-clause: {tsv_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
