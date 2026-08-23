#!/usr/bin/env python3
"""How much of the SV corpus actually carries a VERDICT? (SV-CORPUS-GRAD.13)

WHY THIS EXISTS
---------------
The graduation bar is a count of DIVERGENCES.  That number answers "how many defects do
we know about", and it is silent on the question a signoff claim actually rests on:
**what fraction of the corpus was asked a question it could answer at all?**

A row that never got a verdict is not evidence of correctness.  It is not evidence of
anything.  `deferred:chained_only` in particular is the honest thing to do with a
multi-file design when you can only parse one file in isolation -- and it means the most
realistic industry RTL in the corpus contributes NOTHING to the confidence claim.

`SV-CORPUS-GRAD.12` established the pattern for this whole family of question: an
`explained`/`deferred` label is a statement about the INPUT, and until someone checks it,
the burn-down's denominator is a guess.  This instrument makes the denominator explicit
and re-runnable so it cannot rot the way a number pasted into a doc does.

WHAT IT REPORTS
---------------
Every adjudication class, bucketed by whether the row carries a usable verdict:

  ADJUDICATED   the row was asked and answered - a match, or a known divergence
  ROUTED        answered in ANOTHER lane's manifest (verilog_2005), not lost
  NO VERDICT    the row contributes nothing to the confidence claim, for a named reason

⛔ NO VERDICT is not the same as "defect".  It is "unknown", and the whole point is that
unknown and clean are different words.

THE STRATIFICATION (SV-CORPUS-GRAD.13a) -- a class blob cannot be dispositioned
------------------------------------------------------------------------------
"38.7 % carries no verdict" is a headline, not a plan.  A disposition (fix / honest
permanent deferral / route) attaches to a REASON and to what the parser already did,
so this instrument splits NO VERDICT three further ways, each measured:

1. by what the parser DID on the row's own bytes, standalone:

     ONE-SIDED POSITIVE  the parse consumed the WHOLE file.  That forecloses a
                         rejects-valid defect for this row and says NOTHING about
                         accepts-invalid.  ⚠️ For a FRAGMENT-shaped row (an `.svh`
                         include payload, a `// verilog_syntax:` excerpt) an accept may
                         itself BE the over-acceptance, because the file is not a legal
                         standalone compilation unit.  Counted separately for that reason.
     DARK                the parse failed and the row's deferral reason is why nobody
                         looked.  Nothing at all is known here.

2. by the row's per-row BASIS REASON, not by its class.  `deferred:svpp_owned` is ten
   different arguments wearing one label; a disposition per class would be a guess.

3. within DARK, by whether the deferral reason can even REACH the failure.  A file with
   no `` ` `` anywhere cannot be altered by macro expansion, conditional resolution or
   `` `include `` inlining: the preprocessed text is byte-identical to the raw text, so a
   parse that fails on it fails identically after chaining.  For `deferred:chained_only`
   that REFUTES the textual half of the deferral -- what remains is the cross-file FACT
   channel (a `type_name` a sibling file declares), which is a different, narrower and
   separately testable claim.  ⛔ For the other classes the label rests on the test's
   PURPOSE or the key's existence, not on directive presence, so a zero-tick count there
   refutes NOTHING and is reported as information only.  Getting that distinction wrong
   in either direction is the whole risk this instrument exists to manage.

⛔ NONE of this moves a row into ADJUDICATED.  A row earns a verdict by being PARSED
under honest conditions, never by being reclassified (SV-CORPUS-GRAD.13, .3.13, .3.23).

GROUND TRUTH -- it refuses rather than guesses
----------------------------------------------
File paths are resolved through the tracked corpus results file, so the suite layout is
never re-encoded here (a second copy of `resolve_path()` would drift silently).  Each
manifest row must resolve to EXACTLY ONE results row, and that row's outcome must equal
the manifest's `observed`; 0 matches, >1 match, or any disagreement aborts before a
number is published.

USAGE
    python3 stimuli/sv/corpus_verdict_coverage.py
    python3 stimuli/sv/corpus_verdict_coverage.py --no-corpus-scan   # skip the tick census

Output: docs/tasks/artifacts/sv_corpus_grad/verdict_coverage/coverage.md
        + coverage.tsv (per class) + strata.tsv (per class x stratum)
"""

import argparse
import hashlib
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent

# How each adjudication class contributes to a confidence claim, and WHY.
# ⛔ The `why` strings are the load-bearing part: a bucket assignment nobody can argue
# with is a bucket assignment nobody checked.
CLASSES = {
    "match": (
        "ADJUDICATED", "expected and observed agree - the row testifies FOR the parser"),
    "divergence:unexplained_rejects_valid": (
        "ADJUDICATED", "a known defect: valid SV the parser refuses (the axis-2 bar)"),
    "divergence:unexplained_accepts_invalid": (
        "ADJUDICATED", "a known defect: invalid SV the parser accepts (the axis-2 bar)"),
    "divergence:explained_svpp_macro_use": (
        "ADJUDICATED", "parse stops on a macro use - positionally gated since .12a"),
    "divergence:explained_svpp_conditional": (
        "ADJUDICATED", "parse stops on a conditional - positionally gated since .12a"),
    "divergence:explained_svpp_include": (
        "ADJUDICATED", "parse stops on an `include - positionally gated since .12a"),
    "divergence:explained_svpp_protected_envelope": (
        "ADJUDICATED", "IEEE 1800-2017 §34 encrypted envelope - not source text yet"),
    "deferred:v2005_profile_lane": (
        "ROUTED", "adjudicated in adjudication_manifest_v2005.tsv, not lost"),
    "deferred:chained_only": (
        "NO VERDICT",
        "multi-file design: needs `include/`define chaining to parse honestly - the "
        "MOST realistic RTL in the corpus, contributing nothing to the claim"),
    "deferred:no_sv_key": (
        "NO VERDICT", "no upstream answer key exists - expectation underivable"),
    "deferred:svpp_owned": (
        "NO VERDICT", "conformance owned by the preprocessor lane, by design"),
    "deferred:impl_varying": (
        "NO VERDICT", "LRM leaves the behaviour implementation-defined"),
    "deferred:verilog_ams_lane": (
        "NO VERDICT", "Verilog-AMS, a different language family"),
    "deferred:ni_unimplemented": (
        "NO VERDICT", "upstream marks the construct not-implemented"),
    "deferred:vendor_extension_enabled": (
        "NO VERDICT", "the descriptor switches a NON-STANDARD vendor extension ON, so "
        "'it compiled' is testimony about a superset language (SV-CORPUS-GRAD.13e.3)"),
}
BUCKETS = ["ADJUDICATED", "ROUTED", "NO VERDICT"]

# What "this file contains no `` ` `` byte at all" MEANS for each NO-VERDICT class.
# ⛔ THE DISTINCTION *IS* THE FINDING. The same measurement refutes one label and says
# nothing whatever about another, so publishing one number for all six would be wrong
# five times over. Every claim below that says "refutes" was checked by hand first.
TICK_MEANING = {
    "deferred:chained_only":
        "⭐ **REFUTES the TEXTUAL half of the deferral** — no chaining can alter one "
        "byte of these files, so the parse fails identically expanded. What remains is "
        "the cross-file FACT channel (a `type_name` a sibling file declares), which is "
        "narrower, is not expansion, and is separately testable → `.13c`",
    "deferred:no_sv_key":
        "information only — the label is the absence of an upstream ANSWER KEY, which "
        "no directive could supply",
    "deferred:svpp_owned":
        "⛔ refutes NOTHING — read them: verible excerpt-mode fragments and verilator "
        "`t_preproc_*_bad` EOF/string cases. Preprocessor relevance is the test's "
        "PURPOSE, not a directive in its text",
    "deferred:impl_varying":
        "information only — the LRM leaves the verdict implementation-defined "
        "regardless of directives",
    "deferred:verilog_ams_lane":
        "information only — the label is about the DIALECT (Verilog-AMS), not directives",
    "deferred:ni_unimplemented":
        "information only — upstream marks the construct not-implemented",
    "deferred:vendor_extension_enabled":
        "⛔ **IRRELEVANT to this deferral, and saying so is the point** — the row is "
        "parked because of what the ANSWER KEY tested, never because of what the TEXT "
        "needs. A backtick census measures the text; it cannot make a compile under "
        "`-gxtypes` into testimony about IEEE 1364-2005. The disposition these rows "
        "need is a per-file LRM adjudication with a clause cite, exactly like the "
        "`V2005_LRM_PINNED` rows that already carry one",
}

# Total classification, asserted at import: a NO-VERDICT class added to CLASSES without
# an interpretation here would otherwise publish a bare number with no reading.
_UNMEANT = sorted(c for c, (bucket, _why) in CLASSES.items()
                  if bucket == "NO VERDICT" and c not in TICK_MEANING)
if _UNMEANT:
    raise SystemExit(
        f"⛔ REFUSING: NO-VERDICT class(es) with no TICK_MEANING entry: {_UNMEANT}. "
        "State what a zero-backtick count means for them — the same measurement "
        "refutes some labels and is irrelevant to others.")


# A row whose file is not a legal standalone compilation unit: an accept on it is not
# one-sided-positive testimony, because accepting a fragment may BE the over-acceptance.
FRAGMENT_EXTS = (".svh", ".vh", ".h")
FRAGMENT_BASIS_MARKERS = ("fragment", "include payload")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_manifest(path: Path):
    """Rows as dicts. `basis` is load-bearing here: a disposition attaches to the
    per-row REASON, never to the class blob the reason sits under."""
    rows = []
    with path.open(encoding="utf-8") as fh:
        header = fh.readline().rstrip("\n").split("\t")
        idx = {name: header.index(name) for name in
               ("suite", "relpath", "observed", "adjudication", "basis")}
        for line in fh:
            if not line.strip():
                continue
            c = line.rstrip("\n").split("\t")
            rows.append({k: c[i] for k, i in idx.items()})
    return rows


def read_results(path: Path):
    """The corpus runner's raw outcomes: suite, verdict, REPO-ROOT-RELATIVE path.
    This is the path oracle -- the suite layout (uvm-core does not live under
    `subs/`) is stated once, upstream, and never re-encoded in this file."""
    out = []
    with path.open(encoding="utf-8") as fh:
        for line in fh:
            if not line.strip():
                continue
            c = line.rstrip("\n").split("\t")
            if len(c) != 3:
                raise SystemExit(f"⛔ REFUSING: malformed results row: {line!r}")
            out.append((c[0], c[1], c[2]))
    return out


def index_results(results):
    """(suite, basename) -> [(observed, path)]. Basename first so the suffix match
    below stays linear rather than scanning every path per row."""
    idx = {}
    for suite, observed, path in results:
        idx.setdefault((suite, path.rsplit("/", 1)[-1]), []).append((observed, path))
    return idx


def resolve_row(row, idx):
    """The manifest's `relpath` is suite-root-relative; the results file carries the
    full path. Resolve by unique suffix match and REFUSE on ambiguity or absence --
    a wrong path would silently read the wrong file's bytes."""
    rel = row["relpath"]
    cands = [(o, p) for o, p in idx.get((row["suite"], rel.rsplit("/", 1)[-1]), [])
             if p.endswith("/" + rel)]
    if len(cands) != 1:
        raise SystemExit(
            f"⛔ REFUSING: {row['suite']}/{rel} resolved to {len(cands)} corpus "
            "results rows (want exactly 1). The manifest and the results file "
            "disagree about the corpus - re-run the corpus adjudication.")
    observed, path = cands[0]
    if observed != row["observed"]:
        raise SystemExit(
            f"⛔ REFUSING: {row['suite']}/{rel} is `{row['observed']}` in the "
            f"manifest and `{observed}` in the results file. One of the two is "
            "stale; a stratification built on the stale one would be wrong.")
    return path


def is_fragment_shaped(row) -> bool:
    rel, basis = row["relpath"].lower(), row["basis"].lower()
    return (rel.endswith(FRAGMENT_EXTS)
            or any(m in basis for m in FRAGMENT_BASIS_MARKERS))


def has_no_backtick(path: Path) -> bool:
    """True when the file contains no `` ` `` byte at all -- so no macro expansion,
    conditional resolution or `` `include `` inlining can alter, delete or shift a
    single byte of it. Read as bytes: the question is about the file, not its text."""
    return b"`" not in path.read_bytes()


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--manifest",
                    default=ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv",
                    type=Path)
    ap.add_argument("--manifest-v2005",
                    default=ROOT / "stimuli/sv/characterization/adjudication_manifest_v2005.tsv",
                    type=Path)
    ap.add_argument("--audit",
                    default=ROOT / "docs/tasks/artifacts/sv_corpus_grad/"
                                   "explained_svpp_audit/after/audit.tsv",
                    type=Path)
    ap.add_argument("--results",
                    default=ROOT / "stimuli/sv/characterization/results.tsv",
                    type=Path,
                    help="the corpus runner's raw outcomes - the PATH oracle")
    ap.add_argument("--outdir",
                    default=ROOT / "docs/tasks/artifacts/sv_corpus_grad/verdict_coverage",
                    type=Path)
    ap.add_argument("--no-corpus-scan", action="store_true",
                    help="skip the DARK-half backtick census (no corpus file reads)")
    args = ap.parse_args()

    rows = read_manifest(args.manifest)
    counts = Counter(r["adjudication"] for r in rows)
    unknown = sorted(set(counts) - set(CLASSES))
    if unknown:
        raise SystemExit(
            "⛔ REFUSING: adjudication class(es) this census has no bucket for: "
            f"{unknown}. Classify them explicitly - an unclassified class silently "
            "lands in no bucket, which is the failure mode this instrument exists to "
            "prevent.")

    total = sum(counts.values())
    by_bucket = Counter()
    for cls, n in counts.items():
        by_bucket[CLASSES[cls][0]] += n

    # The .12 audit splits the `explained` rows further: corroborated vs undecidable.
    corroborated = undecidable = 0
    if args.audit.is_file():
        with args.audit.open(encoding="utf-8") as fh:
            h = fh.readline().rstrip("\n").split("\t")
            iv = h.index("verdict")
            for line in fh:
                if not line.strip():
                    continue
                v = line.rstrip("\n").split("\t")[iv]
                if v == "STUCK-ON-SVPP-TOKEN":
                    corroborated += 1
                else:
                    undecidable += 1

    # --- THE STRATIFICATION (SV-CORPUS-GRAD.13a) --------------------------------
    # Split NO VERDICT by what the parser already did on the row's OWN bytes. An
    # accept forecloses a rejects-valid defect for that row; it forecloses nothing
    # about accepts-invalid, and on a fragment it may BE the over-acceptance.
    nv_rows = [r for r in rows if CLASSES[r["adjudication"]][0] == "NO VERDICT"]
    strata = {}
    for r in nv_rows:
        s = strata.setdefault(r["adjudication"], Counter())
        if r["observed"] == "pass":
            s["one_sided_fragment" if is_fragment_shaped(r) else "one_sided_unit"] += 1
        else:
            s["dark"] += 1

    # GROUND-TRUTH CONTROL, run over EVERY row before any corpus byte is read: each
    # manifest row must resolve to exactly one results row whose outcome agrees.
    # A stratification built on a stale half would be wrong without being visible.
    no_tick, dark_scanned = Counter(), 0
    if not args.no_corpus_scan:
        idx = index_results(read_results(args.results))
        paths = [resolve_row(r, idx) for r in rows]
        for r, rel in zip(rows, paths):
            if (CLASSES[r["adjudication"]][0] == "NO VERDICT"
                    and r["observed"] == "fail"):
                dark_scanned += 1
                if has_no_backtick(ROOT / rel):
                    no_tick[r["adjudication"]] += 1

    L = []
    L.append("# SV corpus VERDICT COVERAGE — what fraction was asked a question it could "
             "answer? (SV-CORPUS-GRAD.13)\n")
    L.append("> Generated by `stimuli/sv/corpus_verdict_coverage.py`. ⛔ A row with no "
             "verdict is **unknown**, not clean — the two are different words, and only "
             "one of them supports a signoff claim.\n")
    L.append("## Instrument identity\n")
    L.append("| input | repo-root-relative path | sha256 |")
    L.append("|---|---|---|")
    for label, p in (("sv manifest", args.manifest),
                     ("v2005 manifest", args.manifest_v2005)):
        L.append(f"| {label} | `{p.relative_to(ROOT)}` | `{sha256(p)}` |")
    L.append("")
    L.append("## The headline\n")
    for b in BUCKETS:
        L.append(f"- **{b}: {by_bucket[b]:,} rows ({100*by_bucket[b]/total:.1f} %)**")
    L.append("")
    L.append("## Every class, bucketed\n")
    L.append("| bucket | adjudication class | rows | % | why |")
    L.append("|---|---|---:|---:|---|")
    for b in BUCKETS:
        for cls, n in sorted(counts.items(), key=lambda kv: -kv[1]):
            if CLASSES[cls][0] != b:
                continue
            L.append(f"| {b} | `{cls}` | {n:,} | {100*n/total:.1f} % | "
                     f"{CLASSES[cls][1]} |")
    L.append(f"| | **TOTAL** | **{total:,}** | **100.0 %** | |")
    L.append("")
    if corroborated or undecidable:
        L.append("## Inside `ADJUDICATED`, the `explained_svpp_*` rows split again "
                 "(SV-CORPUS-GRAD.12)\n")
        L.append(f"- **{corroborated:,} corroborated** — the parse stops *on* the "
                 "preprocessor construct. The label is positively verified.")
        L.append(f"- **{undecidable:,} undecidable by position** — the label is neither "
                 "corroborated nor refuted; only running an expander settles them "
                 "(`.12b`).")
        L.append("")
        L.append(f"⇒ Rows contributing nothing to the confidence claim, counting these: "
                 f"**{by_bucket['NO VERDICT'] + undecidable:,} "
                 f"({100*(by_bucket['NO VERDICT'] + undecidable)/total:.1f} %)**.")
        L.append("")
    L.append("## `NO VERDICT`, by suite — where the silence actually is\n")
    L.append("| suite | class | rows |")
    L.append("|---|---|---:|")
    per = Counter((r["suite"], r["adjudication"]) for r in nv_rows)
    for (s, c), n in per.most_common():
        L.append(f"| {s} | `{c}` | {n:,} |")
    L.append("")

    # --- the stratification, published (SV-CORPUS-GRAD.13a) ----------------------
    tot_unit = sum(s["one_sided_unit"] for s in strata.values())
    tot_frag = sum(s["one_sided_fragment"] for s in strata.values())
    tot_dark = sum(s["dark"] for s in strata.values())
    L.append("## Inside `NO VERDICT` — what the parser ALREADY did on the row's own "
             "bytes (SV-CORPUS-GRAD.13a)\n")
    L.append(f"- **ONE-SIDED POSITIVE — {tot_unit + tot_frag:,} rows "
             f"({100*(tot_unit + tot_frag)/total:.1f} % of the corpus)**: the parse "
             "consumed the WHOLE file standalone. ⇒ no *rejects-valid* defect hides "
             "behind these rows. ⛔ It says **nothing** about accepts-invalid, and it "
             "is **not** a verdict — there is still no expectation to compare against.")
    L.append(f"  - {tot_unit:,} are compilation-unit-shaped; ⚠️ **{tot_frag:,} are "
             "FRAGMENT-shaped** (`.svh` include payload / excerpt-mode fixture), where "
             "accepting is not testimony FOR the parser at all — a fragment is not a "
             "legal standalone unit, so the accept may itself BE the over-acceptance.")
    L.append(f"- ⛔ **DARK — {tot_dark:,} rows ({100*tot_dark/total:.1f} % of the "
             "corpus)**: the parse failed and the deferral is why nobody looked. "
             "**This — not the headline 38.7 % — is the population a disposition has "
             "to burn down.**")
    L.append("")
    L.append("| class | rows | one-sided (unit) | ⚠️ one-sided (fragment) | DARK |")
    L.append("|---|---:|---:|---:|---:|")
    for cls, n in sorted(counts.items(), key=lambda kv: -kv[1]):
        if CLASSES[cls][0] != "NO VERDICT":
            continue
        s = strata.get(cls, Counter())
        L.append(f"| `{cls}` | {n:,} | {s['one_sided_unit']:,} | "
                 f"{s['one_sided_fragment']:,} | {s['dark']:,} |")
    L.append(f"| **TOTAL** | **{tot_unit + tot_frag + tot_dark:,}** | "
             f"**{tot_unit:,}** | **{tot_frag:,}** | **{tot_dark:,}** |")
    L.append("")

    if not args.no_corpus_scan:
        L.append("## The DARK half — can the row's own deferral reason even REACH the "
                 "failure?\n")
        L.append(f"A file containing no `` ` `` byte anywhere cannot be altered by "
                 "macro expansion, conditional resolution or `` `include `` inlining: "
                 "the preprocessed text is byte-identical to the raw text, so the parse "
                 f"fails identically after chaining. Measured over all {dark_scanned:,} "
                 "DARK rows (path oracle: the tracked corpus results file; every "
                 f"manifest row resolved to exactly one results row, {total:,}/{total:,}, "
                 "with agreeing outcomes — the census aborts otherwise).\n")
        L.append("| class | DARK rows | of which NO `` ` `` anywhere | what that means |")
        L.append("|---|---:|---:|---|")
        for cls, n in sorted(counts.items(), key=lambda kv: -kv[1]):
            if CLASSES[cls][0] != "NO VERDICT":
                continue
            dark = strata.get(cls, Counter())["dark"]
            L.append(f"| `{cls}` | {dark:,} | {no_tick[cls]:,} | "
                     f"{TICK_MEANING[cls]} |")
        L.append("")

    L.append("## Every `NO VERDICT` class, by per-row REASON — what a disposition must "
             "attach to\n")
    L.append("⛔ A class is not an argument. `deferred:svpp_owned` is ten different "
             "arguments wearing one label, and a per-class disposition would be a guess "
             "about most of them.\n")
    L.append("| class | rows | reason (the row's own `basis`) |")
    L.append("|---|---:|---|")
    for cls, n in sorted(counts.items(), key=lambda kv: -kv[1]):
        if CLASSES[cls][0] != "NO VERDICT":
            continue
        reasons = Counter(" ".join(r["basis"].split()) for r in nv_rows
                          if r["adjudication"] == cls)
        for reason, rn in sorted(reasons.items(), key=lambda kv: (-kv[1], kv[0])):
            shown = reason if len(reason) <= 130 else reason[:127] + "…"
            L.append(f"| `{cls}` | {rn:,} | {shown} |")
    L.append("")

    args.outdir.mkdir(parents=True, exist_ok=True)
    (args.outdir / "coverage.md").write_text("\n".join(L) + "\n", encoding="utf-8")
    with (args.outdir / "coverage.tsv").open("w", encoding="utf-8") as fh:
        fh.write("bucket\tclass\trows\tpct\n")
        for cls, n in sorted(counts.items(), key=lambda kv: -kv[1]):
            fh.write(f"{CLASSES[cls][0]}\t{cls}\t{n}\t{100*n/total:.2f}\n")
    with (args.outdir / "strata.tsv").open("w", encoding="utf-8") as fh:
        fh.write("class\trows\tone_sided_unit\tone_sided_fragment\tdark\t"
                 "dark_no_backtick\n")
        for cls, n in sorted(counts.items(), key=lambda kv: -kv[1]):
            if CLASSES[cls][0] != "NO VERDICT":
                continue
            s = strata.get(cls, Counter())
            fh.write(f"{cls}\t{n}\t{s['one_sided_unit']}\t"
                     f"{s['one_sided_fragment']}\t{s['dark']}\t"
                     f"{'' if args.no_corpus_scan else no_tick[cls]}\n")

    for b in BUCKETS:
        print(f"{b:>12}: {by_bucket[b]:6,}  ({100*by_bucket[b]/total:.1f} %)")
    if undecidable:
        print(f"{'+undecidable':>12}: {undecidable:6,}  (inside ADJUDICATED)")
    print(f"{'one-sided':>12}: {tot_unit + tot_frag:6,}  (of which "
          f"{tot_frag:,} fragment-shaped — an accept there is NOT testimony)")
    print(f"{'DARK':>12}: {tot_dark:6,}  ({100*tot_dark/total:.1f} % — the burn-down "
          "population)")
    if not args.no_corpus_scan:
        print(f"{'no-backtick':>12}: {sum(no_tick.values()):6,}  (DARK rows no "
              "preprocessing can alter)")
    # Repo-root-relative when it can be (the project's path policy), absolute when the
    # caller pointed --outdir elsewhere — reporting where the file went must not raise.
    out_md = args.outdir / "coverage.md"
    print(f"wrote {out_md.relative_to(ROOT) if out_md.is_relative_to(ROOT) else out_md}")


if __name__ == "__main__":
    main()
