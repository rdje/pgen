#!/usr/bin/env python3
"""Cluster the rejects-valid population by stuck-point signature (SV-CORPUS-GRAD.3.0).

Read-only diagnosis pass: for every adjudication_manifest.tsv row classed
divergence:unexplained_rejects_valid, re-run parseability_probe, parse the
rejection's furthest_position, extract the source construct at that point, and
cluster rows by a normalized stuck-token signature. Output = the defect-class
worklist the .3 burn-down leaves are cut from.

Usage:
  python3 stimuli/sv/cluster_rejects_valid.py \
      [--manifest stimuli/sv/characterization/adjudication_manifest.tsv] \
      [--subs-root stimuli/sv/subs] \
      [--probe rust/target/debug/parseability_probe] \
      [--jobs 8] \
      [--out docs/tasks/artifacts/sv_corpus_grad/rejects_valid_clusters.tsv] \
      [--summary docs/tasks/artifacts/sv_corpus_grad/rejects_valid_clusters.md]
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


def signature_at(text: str, pos: int) -> str:
    """Normalized 3-token signature starting at the stuck point."""
    tail = text[pos:pos + 200]
    tokens = []
    i = 0
    while i < len(tail) and len(tokens) < 3:
        ch = tail[i]
        if ch.isspace():
            i += 1
            continue
        m = WORD_RE.match(tail, i)
        if m:
            w = m.group(0)
            tokens.append(w if w in SV_KEYWORDS else "ID")
            i = m.end()
            continue
        m = NUM_RE.match(tail, i)
        if m:
            tokens.append("NUM")
            i = m.end()
            continue
        # operator/punct: take up to 2 punct chars as one token
        j = i + 1
        if j < len(tail) and tail[i:j + 1] in {
                "##", "|->", "|=>", "::", "++", "--", "<<", ">>", "==", "!=",
                "<=", ">=", "&&", "||", "->", "'{"}:
            j += 1
        tokens.append(tail[i:j])
        i = j
    return " ".join(tokens) if tokens else "<EOF>"


def line_at(text: str, pos: int) -> str:
    start = text.rfind("\n", 0, pos) + 1
    end = text.find("\n", pos)
    if end == -1:
        end = len(text)
    return text[start:end].strip()[:120]


def probe_one(task):
    probe, subs_root, suite, rel = task
    fpath = subs_root / suite / rel
    try:
        cp = subprocess.run(
            [str(probe), "--parse", "systemverilog", str(fpath),
             "--profile", "sv_2017"],
            capture_output=True, text=True, timeout=30)
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
    furthest = int(m.group(2)) if m.group(2) else surface
    text = fpath.read_text(encoding="utf-8", errors="replace")
    return (suite, rel, surface, furthest,
            signature_at(text, furthest), line_at(text, furthest))


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
    args = ap.parse_args()

    tasks = []
    for line in args.manifest.read_text(encoding="utf-8").splitlines()[1:]:
        cols = line.split("\t")
        if len(cols) >= 5 and cols[4] == "divergence:unexplained_rejects_valid":
            tasks.append((args.probe, args.subs_root, cols[0], cols[1]))
    tasks.sort(key=lambda t: (t[2], t[3]))

    results = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as ex:
        for res in ex.map(probe_one, tasks):
            results.append(res)
    results.sort()

    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w", encoding="utf-8") as fh:
        fh.write("suite\trelpath\tsurface_pos\tfurthest_pos\tsignature\tstuck_line\n")
        for suite, rel, sp, fp, sig, ln in results:
            fh.write(f"{suite}\t{rel}\t{sp}\t{fp}\t{sig}\t{ln}\n")

    clusters = defaultdict(list)
    for suite, rel, _sp, _fp, sig, ln in results:
        clusters[sig].append((suite, rel, ln))

    lines = ["# rejects-valid stuck-point clusters (SV-CORPUS-GRAD.3.0)", "",
             f"{len(results)} rows probed from the adjudication manifest; "
             f"{len(clusters)} distinct 3-token stuck signatures.", "",
             "| # | signature | rows | example (stuck line) |", "|---|---|---|---|"]
    ranked = sorted(clusters.items(), key=lambda kv: (-len(kv[1]), kv[0]))
    for i, (sig, members) in enumerate(ranked, 1):
        ex_suite, ex_rel, ex_line = members[0]
        lines.append(f"| {i} | `{sig}` | {len(members)} | "
                     f"`{ex_line}` ({ex_suite}/{ex_rel}) |")
    lines.append("")
    args.summary.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(f"rows: {len(results)}  clusters: {len(clusters)}")
    for sig, members in ranked[:15]:
        print(f"{len(members):5d}  {sig}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
