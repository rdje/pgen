#!/usr/bin/env python3
"""Cluster a corpus rejection population by stuck-point signature (SV-CORPUS-GRAD.3.0;
family-parameterized by CORPUS-GRAD-ALL.2.1).

Read-only diagnosis pass: for every input row, re-run parseability_probe, read the
rejection's furthest_position, extract the source construct at that point, and cluster
rows by a normalized stuck-token signature. Output = the defect-class worklist the
burn-down leaves are cut from.

TWO INPUT MODES — pick by which one the family has:
  --manifest  an adjudication manifest, clustering only rows classed
              `divergence:unexplained_rejects_valid` (the SV lane: expected verdicts already
              adjudicated, so the population is defect signal by construction)
  --results   a raw run_external_corpus.sh results.tsv, clustering its `fail` rows (the lane
              for a family with no adjudication yet — VHDL today). Raw fails include
              intentionally-invalid corpus files, so a cluster here SIZES a candidate class
              and does not by itself claim a defect; expected-verdict adjudication is the
              separate step (feedback_corpus_expected_from_spec_not_fix).

⛔ WHY THIS TOOL IS FAMILY-PARAMETERIZED RATHER THAN FORKED (CORPUS-GRAD-ALL.2.1). The
engine — probe, read furthest_position, tokenize 3 tokens at the stuck point, rank by
member count — is family-neutral; only the KEYWORD SET and the multi-char operator set are
not. Copying it per family is the exact defect `.2.0` was opened to fix on the diagnostic it
depends on (see docs/knowledge/a-copied-diagnostic-covers-only-where-it-was-pasted.md): a
copied block has no single site to extend. Adding a family here is a dict entry.

⚠️ It still LIVES under stimuli/sv/ although it is no longer SV-only. Deliberate: several
historical task leaves cite this path, and relocating it would either dangle those citations
or require rewriting settled records. Discoverability is served by TOOLBOX.md naming it as
family-neutral. A relocation is a candidate for its own slice, not a side effect of this one.

Usage:
  python3 stimuli/sv/cluster_rejects_valid.py \
      [--manifest stimuli/sv/characterization/adjudication_manifest.tsv] \
      [--subs-root stimuli/sv/subs] \
      [--probe rust/target/debug/parseability_probe] \
      [--grammar systemverilog] [--profile sv_2017] [--keywords <family|auto>] \
      [--jobs 8] \
      [--out docs/tasks/artifacts/sv_corpus_grad/rejects_valid_clusters.tsv] \
      [--summary docs/tasks/artifacts/sv_corpus_grad/rejects_valid_clusters.md]

  # VHDL, raw-fail lane (no adjudication yet), profile-less grammar:
  python3 stimuli/sv/cluster_rejects_valid.py \
      --results stimuli/vhdl/characterization/results.tsv \
      --grammar vhdl --profile '' \
      --out docs/tasks/artifacts/corpus_grad_all/vhdl_fail_clusters.tsv \
      --summary docs/tasks/artifacts/corpus_grad_all/vhdl_fail_clusters.md
"""

import argparse
import concurrent.futures
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parents[2]

POS_RE = re.compile(
    r"position (\d+)(?: \[furthest_position=(\d+)[^\]]*\])?")
WORD_RE = re.compile(r"[A-Za-z_$][A-Za-z0-9_$]*")
NUM_RE = re.compile(r"[0-9][0-9_a-zA-Z]*")

# SV keywords kept verbatim in signatures; other identifiers normalize to ID.
SV_KEYWORDS = {
    "module", "endmodule", "class", "endclass", "function", "endfunction",
    "task", "endtask", "package", "endpackage", "interface", "endinterface",
    "program", "endprogram", "property", "endproperty", "sequence",
    "endsequence", "checker", "endchecker", "covergroup", "endgroup",
    "begin", "end", "initial", "always", "always_comb", "always_ff",
    "always_latch", "final", "assign", "typedef", "enum", "struct", "union",
    "packed", "logic", "reg", "wire", "bit", "byte", "int", "integer",
    "shortint", "longint", "real", "shortreal", "realtime", "time", "string",
    "chandle", "event", "void", "signed", "unsigned", "const", "static",
    "automatic", "local", "protected", "rand", "randc", "virtual", "extends",
    "implements", "import", "export", "parameter", "localparam", "specparam",
    "defparam", "generate", "endgenerate", "genvar", "if", "else", "case",
    "casex", "casez", "endcase", "unique", "unique0", "priority", "for",
    "foreach", "while", "do", "repeat", "forever", "return", "break",
    "continue", "fork", "join", "join_any", "join_none", "disable", "wait",
    "assert", "assume", "cover", "expect", "restrict", "constraint", "solve",
    "before", "dist", "inside", "with", "randomize", "new", "this", "super",
    "null", "posedge", "negedge", "edge", "or", "and", "not", "iff",
    "intersect", "throughout", "within", "first_match", "until", "until_with",
    "s_until", "s_until_with", "eventually", "s_eventually", "nexttime",
    "s_nexttime", "always", "strong", "weak", "accept_on", "reject_on",
    "sync_accept_on", "sync_reject_on", "input", "output", "inout", "ref",
    "modport", "clocking", "endclocking", "default", "type", "var", "extern",
    "pure", "context", "bind", "alias", "force", "release", "deassign",
    "wait_order", "cell", "config", "endconfig", "design", "instance",
    "liblist", "library", "use", "primitive", "endprimitive", "table",
    "endtable", "specify", "endspecify", "pulsestyle_onevent", "randsequence",
    "randcase", "std", "tagged", "matches", "wildcard", "timeprecision",
    "timeunit", "nettype", "let", "untyped", "supply0", "supply1", "tri",
    "triand", "trior", "trireg", "tri0", "tri1", "uwire", "wand", "wor",
    "highz0", "highz1", "strong0", "strong1", "pull0", "pull1", "weak0",
    "weak1", "small", "medium", "large", "vectored", "scalared",
}

# VHDL reserved words (IEEE 1076-2019 §15.10) kept verbatim in signatures; every other
# identifier normalizes to ID. CORPUS-GRAD-ALL.2.1. Case-insensitive: VHDL is a
# case-insensitive language, so `ENTITY`/`Entity`/`entity` must land in ONE cluster —
# lookups lower-case the token, and the signature emits the lower-case spelling, or a
# corpus that shouts its keywords would fragment every class by capitalization.
VHDL_KEYWORDS = {
    "abs", "access", "after", "alias", "all", "and", "architecture", "array",
    "assert", "assume", "attribute", "begin", "block", "body", "buffer", "bus",
    "case", "component", "configuration", "constant", "context", "cover",
    "default", "disconnect", "downto", "else", "elsif", "end", "entity",
    "exit", "fairness", "file", "for", "force", "function", "generate",
    "generic", "group", "guarded", "if", "impure", "in", "inertial", "inout",
    "is", "label", "library", "linkage", "literal", "loop", "map", "mod",
    "nand", "new", "next", "nor", "not", "null", "of", "on", "open", "or",
    "others", "out", "package", "parameter", "port", "postponed", "procedure",
    "process", "property", "protected", "private", "pure", "range", "record",
    "register", "reject", "release", "rem", "report", "restrict", "return",
    "rol", "ror", "select", "sequence", "severity", "shared", "signal", "sla",
    "sll", "sra", "srl", "strong", "subtype", "then", "to", "transport",
    "type", "unaffected", "units", "until", "use", "variable", "view", "vmode",
    "vpkg", "vprop", "vunit", "wait", "when", "while", "with", "xnor", "xor",
}

# Multi-character operator tokens taken as ONE signature token, per family. A family whose
# operators are absent here silently splits `<=` into `<` `=`, which merges distinct classes.
SV_OPERATORS = {
    "##", "|->", "|=>", "::", "++", "--", "<<", ">>", "==", "!=",
    "<=", ">=", "&&", "||", "->", "'{",
}
VHDL_OPERATORS = {
    "=>", "<=", ":=", "**", "/=", ">=", "<>", "??", "?=", "?<", "?>",
    "<<", ">>", "&", "|", "--",
}

# family name -> (grammar, default profile, keyword set, operator set, case-insensitive?)
FAMILIES = {
    "systemverilog": ("systemverilog", "sv_2017", SV_KEYWORDS, SV_OPERATORS, False),
    "vhdl": ("vhdl", "", VHDL_KEYWORDS, VHDL_OPERATORS, True),
}


def signature_at(text: str, pos: int, keywords=SV_KEYWORDS, operators=SV_OPERATORS,
                 fold_case: bool = False):
    """Normalized 3-token signature starting at the stuck point.

    Returns `(signature, first_token_pos)`.

    `fold_case` is required for a case-insensitive language (VHDL): without it `ENTITY` and
    `entity` are two clusters and neither shows its true size.

    ⭐ CORPUS-GRAD-ALL.2.1 — `first_token_pos` exists because the report was CONTRADICTING
    ITSELF. `furthest_position` frequently lands on trailing whitespace or a newline, so the
    tokenizer (which skips whitespace, newlines included) took its first token from the NEXT
    line while the example line was read at the raw byte — i.e. the signature named one
    construct and the example line showed another. Measured in the first VHDL run: cluster
    `alias ID :` illustrated by `constant USER_RIGHT : integer := 1 ;`, and `shared variable
    ID` by a `subtype` declaration. Anchoring the excerpt to the first token the signature
    actually consumed makes key and illustration name the same construct.
    """
    tail = text[pos:pos + 200]
    tokens = []
    first = None
    i = 0
    while i < len(tail) and len(tokens) < 3:
        ch = tail[i]
        if ch.isspace():
            i += 1
            continue
        if first is None:
            first = i
        m = WORD_RE.match(tail, i)
        if m:
            w = m.group(0)
            probe = w.lower() if fold_case else w
            tokens.append(probe if probe in keywords else "ID")
            i = m.end()
            continue
        m = NUM_RE.match(tail, i)
        if m:
            tokens.append("NUM")
            i = m.end()
            continue
        # operator/punct: take up to 2 punct chars as one token
        j = i + 1
        if j < len(tail) and tail[i:j + 1] in operators:
            j += 1
        tokens.append(tail[i:j])
        i = j
    if not tokens:
        return "<EOF>", pos
    return " ".join(tokens), pos + (first or 0)


def tsv_cell(value) -> str:
    """One TSV cell: never a tab, never a line break, so one record is one LINE.

    ⛔ `SV-CORPUS-GRAD.3.21`. The normal `stuck_line` path can only ever produce a single
    source line, but the `<NO-POSITION>` path (`probe_one`) writes the probe's raw
    stdout+stderr, and a probe error is MULTI-LINE:

        Error: failed to read input file 'stimuli/sv/subs/sv2v/test/lex/latin1.sv'

        Caused by:
            stream did not contain valid UTF-8

    Written unescaped, that one row became FOUR physical lines in
    `rejects_valid_clusters.tsv`. The downstream classifier drops any line with fewer than
    six columns, so the three orphan fragments vanished silently and the totals still
    looked right — the corruption was visible only as a `wc -l` of 299 over a 296-row
    population. A per-cell escape is the fix at the source; the classifier now REFUSES a
    malformed line rather than skipping it, so the pair cannot drift back.

    ⚠️ Deliberately narrow: it replaces ONLY the characters that break the format, one for
    one, and leaves internal spacing alone. The first cut collapsed whitespace RUNS as well
    (`" ".join(value.split())`) and rewrote **79 lines** of a 296-row artifact that has one
    real defect. A repair whose diff is forty times the size of the bug is a second change
    smuggled in beside the first, and it makes the before/after unreadable as evidence.
    """
    return re.sub(r"[\r\n\t]", " ", str(value))


def line_at(text: str, pos: int) -> str:
    start = text.rfind("\n", 0, pos) + 1
    end = text.find("\n", pos)
    if end == -1:
        end = len(text)
    return text[start:end].strip()[:120]


def probe_one(task):
    probe, grammar, profile, keywords, operators, fold_case, suite, rel, fpath = task
    cmd = [str(probe), "--parse", grammar, str(fpath)]
    if profile:
        cmd += ["--profile", profile]
    try:
        cp = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
    except subprocess.TimeoutExpired:
        return (suite, rel, -1, -1, "<TIMEOUT>", "")
    # Strip the absolute checkout prefix so captured probe text (e.g. a
    # read-error echoing the file path) stays repo-root-relative in tracked
    # artifacts (the DOCPATH doctrine forbids absolute repo paths in docs).
    out = (cp.stdout + cp.stderr).replace(str(_REPO_ROOT) + "/", "")
    if cp.returncode == 0:
        return (suite, rel, -2, -2, "<NOW-PASSES>", "")
    m = POS_RE.search(out)
    if not m:
        return (suite, rel, -3, -3, "<NO-POSITION>", out.strip()[:120])
    surface = int(m.group(1))
    # CORPUS-GRAD-ALL.2.0/.2.1: the bracket is now emitted by EVERY family, so `furthest` is a
    # real deep locus here rather than a silent fallback to the shallow one. The `else surface`
    # branch is kept only for reading an artifact produced by an older binary.
    furthest = int(m.group(2)) if m.group(2) else surface
    text = fpath.read_text(encoding="utf-8", errors="replace")
    sig, sig_pos = signature_at(text, furthest, keywords, operators, fold_case)
    # Anchor the excerpt to the first token the SIGNATURE consumed, not the raw furthest byte —
    # otherwise the two can name different constructs (see signature_at's docstring).
    return (suite, rel, surface, furthest, sig, line_at(text, sig_pos))


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    root = Path(__file__).resolve().parent.parent.parent
    ap.add_argument("--manifest",
                    default=root / "stimuli/sv/characterization/adjudication_manifest.tsv",
                    type=Path)
    ap.add_argument("--subs-root", default=root / "stimuli/sv/subs", type=Path)
    ap.add_argument("--probe",
                    default=root / "rust/target/debug/parseability_probe", type=Path)
    ap.add_argument("--jobs", default=8, type=int)
    ap.add_argument("--out",
                    default=root / "docs/tasks/artifacts/sv_corpus_grad/rejects_valid_clusters.tsv",
                    type=Path)
    ap.add_argument("--summary",
                    default=root / "docs/tasks/artifacts/sv_corpus_grad/rejects_valid_clusters.md",
                    type=Path)
    # CORPUS-GRAD-ALL.2.1 — family parameterization + the raw-results input mode.
    ap.add_argument("--results", type=Path, default=None,
                    help="cluster the `fail` rows of a run_external_corpus.sh results.tsv "
                         "instead of an adjudication manifest (paths repo-root-relative)")
    ap.add_argument("--family", default=None, choices=sorted(FAMILIES),
                    help="keyword/operator/case profile; defaults from --grammar")
    ap.add_argument("--grammar", default="systemverilog",
                    help="grammar name passed to parseability_probe --parse")
    ap.add_argument("--profile", default=None,
                    help="--profile value; '' for a profile-less grammar (e.g. vhdl). "
                         "Unset = the family default.")
    args = ap.parse_args()

    family = args.family or args.grammar
    if family not in FAMILIES:
        raise SystemExit(
            f"no keyword/operator profile for family {family!r}; known: "
            f"{', '.join(sorted(FAMILIES))}. Add one to FAMILIES rather than clustering with "
            f"another family's keywords — a wrong keyword set silently normalizes real keywords "
            f"to ID and merges distinct defect classes.")
    _g, default_profile, keywords, operators, fold_case = FAMILIES[family]
    profile = default_profile if args.profile is None else args.profile

    prof = (args.probe, args.grammar, profile, keywords, operators, fold_case)
    tasks = []
    if args.results is not None:
        # RAW-FAIL lane: results.tsv columns are (sub-corpus, status, repo-root-relative path).
        source_label = f"the `fail` rows of `{args.results.name}`"
        for line in args.results.read_text(encoding="utf-8").splitlines():
            cols = line.split("\t")
            if len(cols) != 3 or cols[1] != "fail":
                continue
            rel = cols[2]
            if rel.startswith("/"):
                raise SystemExit(
                    f"results row carries an ABSOLUTE path ({rel!r}); regenerate with the "
                    f"current stimuli/run_external_corpus.sh, which emits repo-root-relative "
                    f"paths — an absolute row cannot be resolved from another checkout.")
            tasks.append(prof + (cols[0], rel, _REPO_ROOT / rel))
    else:
        source_label = "the adjudication manifest"
        for line in args.manifest.read_text(encoding="utf-8").splitlines()[1:]:
            cols = line.split("\t")
            if len(cols) >= 5 and cols[4] == "divergence:unexplained_rejects_valid":
                tasks.append(prof + (cols[0], cols[1],
                                     args.subs_root / cols[0] / cols[1]))
    if not tasks:
        raise SystemExit("no input rows selected — refusing to publish an empty cluster report")
    tasks.sort(key=lambda t: (t[6], t[7]))

    results = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as ex:
        for res in ex.map(probe_one, tasks):
            results.append(res)
    results.sort()

    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w", encoding="utf-8") as fh:
        fh.write("suite\trelpath\tsurface_pos\tfurthest_pos\tsignature\tstuck_line\n")
        for suite, rel, sp, fp, sig, ln in results:
            fh.write(f"{suite}\t{rel}\t{sp}\t{fp}\t{sig}\t{tsv_cell(ln)}\n")

    clusters = defaultdict(list)
    for suite, rel, _sp, _fp, sig, ln in results:
        clusters[sig].append((suite, rel, ln))

    lines = [f"# `{args.grammar}` stuck-point clusters "
             f"(SV-CORPUS-GRAD.3.0 engine; family lane CORPUS-GRAD-ALL.2.1)", "",
             f"{len(results)} rows probed from {source_label}; "
             f"{len(clusters)} distinct 3-token stuck signatures. "
             f"Keyed on `furthest_position` (the DEEP locus), family profile `{family}`"
             f"{', case-folded' if fold_case else ''}.", ""]
    if args.results is not None:
        lines += [
            "> ⚠️ **RAW-FAIL lane — a cluster SIZES a candidate class, it does not adjudicate "
            "one.** Some corpus files are intentionally invalid, so a fail can be the CORRECT "
            "outcome. Expected verdicts come from the LRM / suite metadata, never from what the "
            "parser does today.", ""]
    lines += ["| # | signature | rows | example (stuck line) |", "|---|---|---|---|"]
    ranked = sorted(clusters.items(), key=lambda kv: (-len(kv[1]), kv[0]))
    for i, (sig, members) in enumerate(ranked, 1):
        ex_suite, ex_rel, ex_line = members[0]
        # In the raw-results lane `rel` is ALREADY repo-root-relative, so prefixing the suite
        # produced a nonsense doubled path ("Compliance-Tests/stimuli/vhdl/subs/Compliance-Tests/…").
        # Emit one path that actually resolves, whichever lane produced the row.
        where = ex_rel if ex_rel.startswith("stimuli/") else f"{ex_suite}/{ex_rel}"
        lines.append(f"| {i} | `{sig}` | {len(members)} | `{ex_line}` ({where}) |")
    lines.append("")
    args.summary.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(f"rows: {len(results)}  clusters: {len(clusters)}")
    for sig, members in ranked[:15]:
        print(f"{len(members):5d}  {sig}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
