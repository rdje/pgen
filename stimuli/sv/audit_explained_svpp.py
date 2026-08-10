#!/usr/bin/env python3
"""Audit the `divergence:explained_svpp_*` population (SV-CORPUS-GRAD.12).

WHY THIS EXISTS
---------------
`adjudicate_external_corpus.py:preproc_dependency()` decides the `explained_svpp_*`
label from a WHOLE-FILE existence test: "does this file contain a `include / an
unknown-macro use / an `ifdef / a protected envelope ANYWHERE?"  It never looks at
WHERE the parse actually failed.  The label is then read downstream as the much
stronger claim "this file fails BECAUSE it needs the preprocessor lane", and the row
is removed from the axis-2 burn-down by construction.

The two errors are not equally priced.  A wrong `unexplained` verdict costs a wasted
investigation; a wrong `explained` verdict SHIPS a parser defect, because nothing
ever looks at the row again.  Only one of them is being checked.

WHAT THE PARSER ACTUALLY TELLS US -- AND THE OFF-BY-A-GAP THAT NEARLY FAKED A FINDING
-------------------------------------------------------------------------------------
`furthest_position` is the deepest byte any branch CONSUMED, not the offset of the
token that defeated the parse.  A parse stuck ON a directive therefore reports the
byte just before the whitespace preceding it: `module top();` + `\n  \n  ` +
`` `define `` reports furthest_position=13 while the offending backtick sits at 19.

The first cut of this instrument compared `furthest_position` against the first
backtick offset directly and "proved" 504 of 1459 rows misclassified.  Every one of
the rows spot-checked was stuck EXACTLY on a backtick -- the 6-byte whitespace gap
was the entire finding.  So the audit resolves the gap first:

    stuck_offset := first non-whitespace, non-comment byte at or after
                    furthest_position          -- the token the parser choked on

THE SOUND FALSIFICATION (the load-bearing test)
-----------------------------------------------
A preprocessor's output is byte-identical to its input up to the first backtick token
it can ALTER.  Nothing before that offset can be added, removed, expanded or shifted by
svpp -- `define lines are deleted, macros expand, `ifdef arms vanish, but all of that
happens AT or AFTER that point, never before it.  (⛔ "alterable" is not "any backtick":
`timescale and its neighbours are handed through untouched and move nothing.)

    stuck_offset < first_alterable_tick
        =>  the parser choked on a token svpp provably cannot alter or move
        =>  it would choke identically on the fully expanded text
        =>  the `explained_svpp_*` label is FALSE for that row.

That is a proof, not a heuristic, and it needs no re-adjudication of the expected
verdict -- which this leaf is forbidden to touch anyway
([[feedback_corpus_expected_from_spec_not_fix]]).  This instrument audits whether the
OBSERVED -> `explained` mapping is sound, never what the expected verdict should be.

Note the test deliberately uses the first backtick of ANY kind, INCLUDING the
`KNOWN_DIRECTIVES` that `preproc_dependency()` ignores.  `define is not a
"dependency flag" there, but svpp still DELETES the line, so the expanded text
diverges from the raw text at that offset.  Using the flag-bearing constructs only
would have produced unsound "proofs".

THE CORROBORATION (what a HEALTHY explained row looks like)
-----------------------------------------------------------
    stuck_offset is a backtick token svpp CONSUMES
        =>  the parse died exactly on macro substitution / conditional resolution /
            `include inlining -- the three operations SVPP-EXPANSION is scoped to
            (docs/tasks/SVPP-EXPANSION.md).  The label is not merely unrefuted, it
            is positively corroborated.

⛔ The converse matters just as much.  svpp expands macros, resolves conditionals and
inlines `include -- and PASSES EVERYTHING ELSE THROUGH.  A parse stuck on `timescale,
`default_nettype, `celldefine, `resetall, `begin_keywords or `unconnected_drive is
stuck on a token that survives preprocessing unchanged, so running svpp cannot fix it.
Those rows are misclassified no matter where they sit in the file.

THE RESIDUE (a signal, explicitly NOT a proof)
----------------------------------------------
    stuck_offset > last_tick_offset, or stuck between two ticks on ordinary syntax

Suspicious but NOT falsifying: an expansion at an earlier offset can change the token
stream that follows it (a macro expanding to `module m;` unbalances an `endmodule` a
thousand lines later).  Rows are reported with the source text at the failure offset
so the residue can be adjudicated by eye, and they are counted separately from the
proven class.  ⛔ Do not promote these buckets to a defect count without reading them.

USAGE
-----
    python3 stimuli/sv/audit_explained_svpp.py            # full census, all classes
    python3 stimuli/sv/audit_explained_svpp.py --jobs 8

Outputs (repo-root-relative, deterministic -- rows sorted by (class, suite, relpath)):
    docs/tasks/artifacts/sv_corpus_grad/explained_svpp_audit/audit.tsv
    docs/tasks/artifacts/sv_corpus_grad/explained_svpp_audit/summary.md
"""

import argparse
import hashlib
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent

# ⛔ SINGLE SOURCE OF TRUTH.  The svpp reach model -- which directives the preprocessor
# CONSUMES, which it hands through, and how a failure position is resolved against them
# -- is owned by the ADJUDICATOR, because that is what actually labels the corpus.  This
# instrument imports it rather than restating it, so the audit re-verifies the SHIPPED
# predicate instead of a copy that can drift away from it (SV-CORPUS-GRAD.12a).
sys.path.insert(0, str(Path(__file__).resolve().parent))
from adjudicate_external_corpus import (           # noqa: E402
    SVPP_PASSTHROUGH,
    blank_comments_and_strings,
    first_alterable_tick,
    svpp_can_explain_failure,
)

ANY_TICK_RE = re.compile(r"`[A-Za-z_][A-Za-z0-9_$]*")
TICK_NAME_RE = re.compile(r"`([A-Za-z_][A-Za-z0-9_$]*)")

# The three operations SVPP-EXPANSION is scoped to deliver (docs/tasks/SVPP-EXPANSION.md:
# "macro substitution + conditional resolution + `include inlining").  A directive in
# this set is CONSUMED by the preprocessor, so a parse stuck on it is genuinely waiting
# for that lane.  Reporting-only: the adjudicator needs a boolean, this needs a REASON.
SVPP_CONSUMED = {
    "define", "undef", "undefineall",                       # macro substitution
    "ifdef", "ifndef", "elsif", "else", "endif",            # conditional resolution
    "include",                                              # `include inlining
}
# `line and `pragma are in neither SVPP_CONSUMED nor the imported SVPP_PASSTHROUGH.
# `line is preprocessor-adjacent (it rewrites reported line numbers) but is not one of
# the three scoped operations; `pragma protect is owned by the §34 protected-envelope
# lane.  Calling either a defect would be a guess, so they get their own bucket.

FURTHEST_RE = re.compile(r"furthest_position=(\d+)")
SURFACE_RE = re.compile(r"did not consume full input at position (\d+)")

UVM_FOLD_MARKER = "stimuli/sv/uvm/"


def tick_offsets(stripped: str):
    """(first, last) byte offset of a backtick token svpp can ALTER, or (None, None).

    ⛔ Not every backtick moves a byte.  svpp substitutes macros, resolves
    conditionals and inlines `include; it hands `timescale, `default_nettype,
    `celldefine, `resetall, `begin_keywords and `unconnected_drive straight through.
    A file whose only early backtick is a `timescale is byte-identical to its
    expansion well past that directive, so the first ALTERABLE tick -- not the first
    tick of any kind -- is where svpp's reach begins.

    `line and `pragma are counted as alterable here on purpose: the audit refuses to
    judge them (see SVPP_PASSTHROUGH's neighbours above), and counting them alterable
    is the choice that cannot manufacture a false falsification.
    """
    hits = [m.start() for m in ANY_TICK_RE.finditer(stripped)
            if TICK_NAME_RE.match(stripped, m.start()).group(1)
            not in SVPP_PASSTHROUGH]
    if not hits:
        return (None, None)
    return (hits[0], hits[-1])


def line_of(raw: str, off: int) -> int:
    return raw.count("\n", 0, off) + 1


def snippet_at(raw: str, off: int, width: int = 60) -> str:
    """One-line source excerpt around a byte offset, for eyeball adjudication."""
    lo = max(0, off - width // 3)
    hi = min(len(raw), off + width)
    text = raw[lo:hi].replace("\t", " ")
    text = " ".join(text.split())
    return text


def resolve_path(suite: str, rel: str) -> Path:
    if suite == "uvm-core":
        return ROOT / "stimuli/sv/uvm" / rel
    return ROOT / "stimuli/sv/subs" / suite / rel


def read_raw(path: Path) -> str:
    """Read as latin-1 so 1 char == 1 byte and every offset is a BYTE offset,
    matching what the parser reports in `furthest_position`."""
    try:
        return path.read_bytes().decode("latin-1")
    except OSError:
        return ""


def probe(probe_bin: Path, path: Path, timeout_s: int):
    """Run the parse probe; return (furthest, surface, raw_stderr_tail)."""
    try:
        proc = subprocess.run(
            [str(probe_bin), "--parse", "systemverilog", str(path),
             "--profile", "sv_2017"],
            capture_output=True, text=True, timeout=timeout_s)
    except subprocess.TimeoutExpired:
        return (None, None, "TIMEOUT")
    err = (proc.stderr or "") + (proc.stdout or "")
    tail = err.strip().splitlines()[-1] if err.strip() else ""
    fm = FURTHEST_RE.search(err)
    sm = SURFACE_RE.search(err)
    return (int(fm.group(1)) if fm else None,
            int(sm.group(1)) if sm else None,
            tail)


# The verdicts that assert the label is FALSE -- must equal the shipped predicate.
DISPROVEN_VERDICTS = ("FALSIFIED", "STUCK-ON-PASSTHROUGH", "NO-TICK-AT-ALL")


def stuck_offset_of(stripped: str, furthest):
    """The first non-whitespace, non-comment byte at or after `furthest` -- i.e. the
    token the parser actually choked on.  `furthest_position` is the deepest byte
    CONSUMED, so it points at the whitespace gap before the offender, not at it.
    Returns None when the parser ran out of input."""
    if furthest is None:
        return None
    for i in range(furthest, len(stripped)):
        if not stripped[i].isspace():
            return i
    return None


def classify(stripped, furthest, stuck, first_tick, last_tick):
    """The audit verdict for one row.  See the module docstring."""
    if furthest is None:
        return ("NO-POSITION", "")        # probe gave no furthest_position
    if first_tick is None:
        # The label claims a preprocessor dependency and the file has no backtick
        # token at all -- the classifier and the file contradict each other.
        return ("NO-TICK-AT-ALL", "")
    if stuck is None:
        # Consumed every byte and still failed: the top-level rule was never
        # satisfied (a missing `endmodule`, say).  A removed `ifdef arm could
        # explain that, so this is not falsifiable by position.
        return ("STUCK-AT-EOF", "")

    m = TICK_NAME_RE.match(stripped, stuck)
    name = m.group(1) if m else ""
    if m:
        # Stuck exactly on a preprocessor construct -- now ask WHICH one, because
        # svpp only consumes three families and passes the rest straight through.
        if name in SVPP_PASSTHROUGH:
            return ("STUCK-ON-PASSTHROUGH", name)   # PROVEN misclassified
        if name in ("line", "pragma"):
            return ("STUCK-ON-AMBIGUOUS", name)     # deliberately not judged
        # SVPP_CONSUMED, or a name in no directive table at all == a macro use
        return ("STUCK-ON-SVPP-TOKEN", name)        # label CORROBORATED

    if stuck < first_tick:
        return ("FALSIFIED", "")          # PROVEN misclassified
    if stuck > last_tick:
        return ("PAST-LAST-TICK", "")     # suspicious, needs eyes
    return ("IN-WINDOW", "")              # not falsifiable by position


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    h.update(path.read_bytes())
    return h.hexdigest()


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--manifest",
                    default=ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv",
                    type=Path)
    ap.add_argument("--probe-bin",
                    default=ROOT / "rust/target/release/parseability_probe", type=Path)
    ap.add_argument("--outdir",
                    default=ROOT / "docs/tasks/artifacts/sv_corpus_grad/explained_svpp_audit",
                    type=Path)
    ap.add_argument("--jobs", type=int, default=8,
                    help="parallel probe invocations (corpus provenance uses 8)")
    ap.add_argument("--timeout", type=int, default=60,
                    help="per-file probe timeout (corpus provenance uses 60 s)")
    args = ap.parse_args()

    if not args.probe_bin.is_file():
        sys.exit(f"probe binary not found: {args.probe_bin}\n"
                 "  (cd rust && cargo build --release --features generated_parsers "
                 "--bin parseability_probe)")

    rows = []
    with args.manifest.open(encoding="utf-8") as fh:
        header = fh.readline().rstrip("\n").split("\t")
        icls = header.index("adjudication")
        for line in fh:
            if not line.strip():
                continue
            cols = line.rstrip("\n").split("\t")
            if cols[icls].startswith("divergence:explained_svpp_"):
                rows.append((cols[icls].split("divergence:explained_svpp_")[1],
                             cols[0], cols[1]))
    rows.sort()
    print(f"auditing {len(rows)} explained_svpp rows "
          f"at {args.jobs}-way parallelism ...", file=sys.stderr)

    def audit_one(row):
        cls, suite, rel = row
        path = resolve_path(suite, rel)
        raw = read_raw(path)
        stripped = blank_comments_and_strings(raw)
        first_tick, last_tick = tick_offsets(stripped)
        furthest, surface, tail = probe(args.probe_bin, path, args.timeout)
        stuck = stuck_offset_of(stripped, furthest)
        verdict, tick_name = classify(stripped, furthest, stuck, first_tick, last_tick)
        at = stuck if stuck is not None else furthest
        # ⭐ DIFFERENTIAL SELF-CHECK (SV-CORPUS-GRAD.12a). This instrument's rich
        # verdict and the adjudicator's boolean are two implementations of one model.
        # Assert they agree on every row, so a future edit to either surfaces here
        # instead of silently moving corpus rows.
        shipped_says_disproven = not svpp_can_explain_failure(raw, furthest)
        disagree = shipped_says_disproven != (verdict in DISPROVEN_VERDICTS)
        return {
            "class": cls, "suite": suite, "relpath": rel,
            "bytes": len(raw),
            "furthest": furthest, "surface": surface, "stuck": stuck,
            "stuck_tick": tick_name,
            "first_tick": first_tick, "last_tick": last_tick,
            "verdict": verdict,
            "fail_line": line_of(raw, at) if at is not None else "",
            "first_tick_line": line_of(raw, first_tick) if first_tick is not None else "",
            "snippet": snippet_at(raw, at) if at is not None else tail[:120],
            "disagree": disagree,
        }

    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        results = list(pool.map(audit_one, rows))

    args.outdir.mkdir(parents=True, exist_ok=True)
    cols = ["class", "suite", "relpath", "verdict", "stuck_tick", "furthest",
            "stuck", "surface", "first_tick", "last_tick", "fail_line",
            "first_tick_line", "bytes", "snippet"]
    tsv = args.outdir / "audit.tsv"
    with tsv.open("w", encoding="utf-8") as fh:
        fh.write("\t".join(cols) + "\n")
        for r in results:
            fh.write("\t".join(str(r[c]) if r[c] is not None else "" for c in cols) + "\n")

    # ---- summary ----
    classes = sorted({r["class"] for r in results})
    verdicts = ["STUCK-ON-SVPP-TOKEN", "FALSIFIED", "STUCK-ON-PASSTHROUGH",
                "STUCK-ON-AMBIGUOUS", "IN-WINDOW", "PAST-LAST-TICK",
                "STUCK-AT-EOF", "NO-TICK-AT-ALL", "NO-POSITION"]
    PROVEN = ("FALSIFIED", "STUCK-ON-PASSTHROUGH", "NO-TICK-AT-ALL")
    lines = []
    lines.append("# `explained_svpp_*` audit — is the label a CLAIM or a COINCIDENCE?"
                 " (SV-CORPUS-GRAD.12)\n")
    lines.append("> Generated by `stimuli/sv/audit_explained_svpp.py`. **Full census of "
                 f"all {len(results)} rows** — not a sample.\n")
    lines.append("## Instrument identity (what produced these numbers)\n")
    lines.append("| input | repo-root-relative path | sha256 |")
    lines.append("|---|---|---|")
    for label, p in (("parse binary", args.probe_bin),
                     ("grammar", ROOT / "grammars/systemverilog.ebnf"),
                     ("generated parser", ROOT / "generated/systemverilog_parser.rs")):
        lines.append(f"| {label} | `{p.relative_to(ROOT)}` | `{sha256(p)}` |")
    lines.append("")
    lines.append(f"Probe parameters: `--parse systemverilog <file> --profile sv_2017`, "
                 f"timeout {args.timeout} s, {args.jobs}-way parallel "
                 "(the corpus provenance parameters).\n")
    lines.append("## Verdict census, per class\n")
    lines.append("| class | rows | " + " | ".join(f"`{v}`" for v in verdicts) + " |")
    lines.append("|---|---:|" + "---:|" * len(verdicts))
    for cls in classes:
        sub = [r for r in results if r["class"] == cls]
        counts = [sum(1 for r in sub if r["verdict"] == v) for v in verdicts]
        lines.append(f"| `{cls}` | {len(sub)} | "
                     + " | ".join(str(c) for c in counts) + " |")
    tot = [sum(1 for r in results if r["verdict"] == v) for v in verdicts]
    lines.append(f"| **TOTAL** | **{len(results)}** | "
                 + " | ".join(f"**{c}**" for c in tot) + " |")
    lines.append("")
    lines.append("**The label is CORROBORATED for:**")
    lines.append("- **`STUCK-ON-SVPP-TOKEN`** — the parser choked exactly on a macro "
                 "use, an `` `include ``, or a conditional directive: one of the three "
                 "operations `SVPP-EXPANSION` is scoped to deliver. The row really is "
                 "waiting for that lane.")
    lines.append("")
    lines.append("**The label is DISPROVEN for:**")
    lines.append("- **`FALSIFIED`** — the choked-on token lies STRICTLY BEFORE the "
                 "file's first *alterable* backtick (passthrough directives excluded, "
                 "since they move nothing), in a region the preprocessor provably "
                 "cannot alter or shift. It would choke identically on the expanded "
                 "text.")
    lines.append("- **`STUCK-ON-PASSTHROUGH`** — the parser choked on a compiler "
                 "directive svpp hands through unchanged (`` `timescale ``, "
                 "`` `default_nettype ``, …). Running the preprocessor gives the parser "
                 "the same token back, so it cannot be what is missing.")
    lines.append("- **`NO-TICK-AT-ALL`** — the label claims a preprocessor dependency "
                 "and the file contains no backtick token at all. Classifier and file "
                 "contradict each other outright.")
    lines.append("")
    lines.append("**Undecided by this method (⛔ NOT defect counts):**")
    lines.append("- **`IN-WINDOW`** — choked on ordinary syntax between the first and "
                 "last backtick. Position cannot falsify the label; the explanation "
                 "stands unrefuted, which is not the same as verified.")
    lines.append("- **`PAST-LAST-TICK`** — choked past EVERY backtick in the file. "
                 "Suspicious, *not* falsifying: an earlier expansion can shift the "
                 "token stream that follows it.")
    lines.append("- **`STUCK-AT-EOF`** — consumed every byte and still failed, so the "
                 "top-level rule was never satisfied. A resolved `` `ifdef `` arm could "
                 "supply the missing tail.")
    lines.append("- **`STUCK-ON-AMBIGUOUS`** — choked on `` `line `` or `` `pragma ``, "
                 "which sit in neither the scoped-svpp set nor the passthrough set. "
                 "Judging them would be a guess.")
    lines.append("- **`NO-POSITION`** — the probe reported no `furthest_position` "
                 "(timeout, or a non-parse error). Not auditable by this method.")
    lines.append("")

    proven = [r for r in results if r["verdict"] in PROVEN]
    if proven:
        lines.append(f"## The {len(proven)} DISPROVEN rows — every one is a parser "
                     "finding wearing an svpp label\n")
        lines.append("| class | verdict | suite | file | fail line | stuck @ | "
                     "first tick @ | source at failure |")
        lines.append("|---|---|---|---|---:|---:|---:|---|")
        for r in sorted(proven, key=lambda r: (r["class"], r["verdict"],
                                               r["suite"], r["relpath"])):
            snip = r["snippet"].replace("|", "\\|")
            lines.append(
                f"| `{r['class']}` | `{r['verdict']}` | {r['suite']} | "
                f"`{r['relpath']}` | {r['fail_line']} | {r['stuck']} | "
                f"{r['first_tick'] if r['first_tick'] is not None else '—'} | "
                f"`{snip}` |")
        lines.append("")

    (args.outdir / "summary.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

    bad = [r for r in results if r["disagree"]]
    if bad:
        print(f"\n⛔ DIFFERENTIAL SELF-CHECK FAILED on {len(bad)} row(s): this audit and "
              "adjudicate_external_corpus.py:svpp_can_explain_failure() disagree. One of "
              "the two changed without the other.", file=sys.stderr)
        for r in bad[:10]:
            print(f"    {r['suite']}/{r['relpath']} verdict={r['verdict']}",
                  file=sys.stderr)
    else:
        print(f"differential self-check: {len(results)}/{len(results)} rows AGREE with "
              "the shipped adjudicator predicate", file=sys.stderr)

    for v, c in zip(verdicts, tot):
        print(f"{v:>16}: {c}", file=sys.stderr)
    print(f"\nwrote {tsv.relative_to(ROOT)}", file=sys.stderr)
    print(f"wrote {(args.outdir / 'summary.md').relative_to(ROOT)}", file=sys.stderr)


if __name__ == "__main__":
    main()
