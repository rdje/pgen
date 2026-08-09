#!/usr/bin/env python3
"""`SV-CORPUS-GRAD.3.24` — HOW MUCH OF THE REMAINING BURN-DOWN IS ADJUDICATOR WORK?

The leaf's deliverable is **a number, not a fix**: of the `divergence:unexplained_rejects_valid`
rows still in the manifest, how many are mis-adjudicated expectations rather than parser defects?
Getting that wrong in either direction is expensive — burning adjudicator rows down as parser
defects injects OVER-ACCEPTANCE ([[feedback_sv_strict_lrm_compliance_default]]), while moving a
real defect out of the class makes it invisible ([[a-rising-pass-rate-is-not-evidence-of-correctness]]).

⛔⛔ **THE FILENAME IS NOT AN ORACLE, and that is the whole difficulty.** `.3.24` opened on the
observation that 71 of the then-296 rows sat in files whose PATH advertises invalidity (`_bad`, a
`test/error/` directory, an `_ILLEGAL` marker). The naive reading — "those are negatives, pin
them" — is WRONG: verilator's `_bad` overwhelmingly means *this test expects an ELABORATION
error*, and elaboration errors parse perfectly. `.3.23` proved it in both directions at once, on
files that are in that very population: `t_enum_bad_value.v`'s `_bad` names an out-of-range enum
VALUE (elaboration, so `must_accept` is right for the reason the name gives) — yet it still fails
to PARSE, for the completely unrelated `enum [N:M]` reason, which is what actually made its
expected verdict wrong. **The name explains a different defect from the one the parser hits.**

⇒ THE ONLY SOUND CHANNEL IS THE SUITES' OWN STAGE METADATA, which is what this census reads. Every
row's expected verdict was DERIVED from that metadata and the derivation is recorded verbatim in
the manifest's `basis` column, so classifying the basis classifies the actual reasoning rather than
a re-guess of it. Five classes:

  UPSTREAM_SAYS_VALID  upstream asserts the unit COMPILES (a verilator driver expecting success, a
                       clean Surelog golden, an ispras TYPE:POSITIVE, an sv-tests positive, an
                       sv2v conversion input, an ivtest `normal`). ⇒ PARSER work, or a genuine
                       upstream-vs-LRM disagreement. NOT adjudicator work.
  POST_PARSE_CLAIM     upstream names a failure at a LATER stage (elaboration, lint, type, name
                       resolution). The text parses by upstream's own testimony. ⇒ PARSER work.
  TOOL_LIMIT_CLAIM     upstream says "Unsupported" or hit an internal error. ⚠️ This is testimony
                       about the TOOL, not about the language — it supports NEITHER verdict, and
                       is broken out rather than being quietly folded into POST_PARSE.
  PARSE_STAGE_CLAIM    upstream names a lexical/syntactic failure. ⇒ ADJUDICATOR work. Expected to
                       be ~0 here BY CONSTRUCTION: such a row is already `must_reject`, hence a
                       `match`, hence not in this population at all. A NON-ZERO count means the
                       stage heuristic has a hole — which is exactly how `.3.24` found three.
  NO_CLAIM             no upstream metadata. ⇒ undecidable from metadata; needs per-construct LRM
                       adjudication, i.e. a `.3.x` construct leaf.

⭐ GROUND TRUTH ([[feedback_instrument_needs_ground_truth]]) — this instrument REFUSES rather than
guesses, in three ways, and any miss aborts before a number is printed:
  * an UNRECOGNIZED basis spelling is a REFUSAL, never an "other" bucket. Enumeration-with-a-
    fallback is what let one repo instrument measure the same population as 10, then 16, then 18,
    every miss silent in the passing direction (`LIVE-DOC-CURRENCY` instrument B). If a suite
    changes its wording, this census stops instead of under-reporting.
  * a POSITIVE control: a pinned row whose basis names a parse-stage golden must classify
    PARSE_STAGE_CLAIM.
  * a NEGATIVE control: a `%Error-UNSUPPORTED` row must classify TOOL_LIMIT_CLAIM and NOT
    POST_PARSE_CLAIM — the distinction the whole "is this parser work?" question turns on.

Usage (repo-root-relative, directive 12):
  python3 docs/tasks/artifacts/sv_corpus_grad/adjudicator_hole_sizing/stage_claim_census.py
  ... --manifest <tsv>   ... --show-rows PARSE_STAGE_CLAIM
"""

import argparse
import re
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "config_use_param_override"))
from _repo_root import repo_root  # noqa: E402

ROOT = repo_root()
VERILATOR_T = ROOT / "stimuli/sv/subs/verilator/test_regress/t"
TARGET_CLASS = "divergence:unexplained_rejects_valid"

VALID, POST, TOOL, PARSE, NONE = (
    "UPSTREAM_SAYS_VALID", "POST_PARSE_CLAIM", "TOOL_LIMIT_CLAIM",
    "PARSE_STAGE_CLAIM", "NO_CLAIM")

# (basis substring, class). Ordered: first match wins, so the specific precede the generic.
# ⛔ EVERY row must match one of these. There is deliberately NO fallback.
BASIS_RULES = [
    ("golden .out reports a PARSE-STAGE error",            PARSE),
    ("golden .out is\n a post-parse tool error",           POST),   # (never matches; see below)
    ("but golden .out is a post-parse tool error",         POST),   # refined per-row by the golden
    (".py expects success",                                VALID),
    ("TYPE: POSITIVE",                                     VALID),
    ("conversion-input .sv (valid SV by suite contract)",  VALID),
    ("sv-tests: positive test",                            VALID),
    ("kythe single-file indexing fixture",                 VALID),
    ("lint-rule fixture (style findings, valid syntax)",   VALID),
    ("fixture (positive-dominant default)",                VALID),
    ("unittest data fixture (positive-dominant)",          VALID),
    ("no metadata header (positive-dominant default)",     NONE),
    ("completes with no [SNT:]/[FTL:]",                    VALID),
    ("type normal - compiles under the iverilog SV dialect", VALID),
    ("runs under an explicit SV gen",                      VALID),
    ("golden",                                             POST),   # ivtest "…golden X shows only post-parse errors"
    ("at post-parse stage(s)",                             POST),
    ("post-parse",                                         POST),
    ("semantic",                                           POST),
    ("elaboration",                                        POST),
    ("prose",                                              POST),
    ("BNF-parseable",                                      POST),
    ("declaration-ordering",                               POST),
    ("resolution failure",                                 POST),
    ("binding semantics",                                  POST),
    ("constant-value legality",                            POST),
    ("coherence rules",                                    POST),
    ("regression/driver fixture",                          VALID),
]

UNSUPPORTED_RE = re.compile(r"%Error-UNSUPPORTED|%Error: Internal Error")
FAILS_TRUE_RE = re.compile(r"fails\s*=\s*(True|test\.vlt_all)\b")
TOP_FILENAME_RE = re.compile(r"top_filename\s*=\s*[\"']([^\"']+)[\"']")

# --- LEG 1: the verilator LEXICAL-VOCABULARY guard -------------------------------------------
#
# ⛔ WHY A SECOND LEG EXISTS AT ALL — the honest bound on the census above. A basis census reads
# the RECORDED DERIVATION, so it inherits whatever the adjudicator was blind to: a row derived
# WRONG is classified confidently and correctly *as what the adjudicator thought*. That is exactly
# how three rows hid. They were not found by classifying bases; they were found by enumerating
# verilator's own lexer message set across all tracked goldens and noticing that one construct
# family had two verdicts. So the census's "0 remaining" is conditional on this vocabulary being
# COMPLETE — and this leg is what keeps it so as the submodule is re-vendored.
LEXICALish_RE = re.compile(
    r"(EOF in [^\r\n]*|Unterminated [^\r\n]*|Version control conflict[^\r\n]*)")
# Every spelling verilator emits, each with an adjudicated lane. A spelling in NEITHER set is a
# REFUSAL: it means upstream grew a message nobody has ruled on, which is the moment a stage
# heuristic silently goes stale.
LEXICAL_PARSE_STAGE = {
    "EOF in unterminated string",             # 5.9 string literal running to EOF
    'EOF in unterminated """ string',         # ditto, triple-quoted spelling
    "Unterminated string",                    # 5.9, reported at the quote
    "EOF in (*",                              # A.9.1 attribute_instance with no `*)`
    "EOF in 'X' block comment",               # 5.4 /* with no */
    "Version control conflict marker in file",  # `<<<<<<<` has no lexical derivation
}
LEXICAL_OTHER_LANE = {
    "EOF in define argument list": "preprocessor (svpp lane)",
    "Unterminated ( in define formal arguments.": "preprocessor (svpp lane)",
    "EOF in unterminated preprocessor expression": "preprocessor (svpp lane)",
    "Unterminated /* comment inside -f file.": "a `-f` command file, not source text",
}


def read(p: Path) -> str:
    try:
        return p.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


class VerilatorGoldens:
    """Which verilator golden log belongs to a row, so a POST_PARSE basis can be refined
    into TOOL_LIMIT when the tool's own message says 'Unsupported' / 'Internal Error'."""

    def __init__(self):
        self.stems, self.top_ref = set(), {}
        if not VERILATOR_T.is_dir():
            return
        for py in sorted(VERILATOR_T.glob("*.py")):
            self.stems.add(py.stem)
            for ref in TOP_FILENAME_RE.findall(read(py)):
                self.top_ref.setdefault(Path(ref).stem, py.stem)

    def is_tool_limit(self, relpath: str) -> bool:
        stem = Path(relpath).stem
        driver = stem if stem in self.stems else self.top_ref.get(stem, stem)
        for cand in (VERILATOR_T / f"{driver}.out", VERILATOR_T / f"{stem}.out"):
            if cand.is_file():
                return bool(UNSUPPORTED_RE.search(read(cand)))
        return False


def classify(suite: str, relpath: str, basis: str, goldens: VerilatorGoldens):
    for needle, cls in BASIS_RULES:
        if needle in basis:
            if cls is POST and suite == "verilator" and goldens.is_tool_limit(relpath):
                return TOOL
            return cls
    return None            # ⛔ REFUSAL, never a fallback bucket


def load(manifest: Path):
    return [l.split("\t") for l in
            read(manifest).splitlines()[1:]]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--manifest", type=Path,
                    default=ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv")
    ap.add_argument("--show-rows", default=None,
                    help="print every row of the named class")
    args = ap.parse_args()

    goldens = VerilatorGoldens()
    rows = [r for r in load(args.manifest) if len(r) > 5 and r[4] == TARGET_CLASS]

    print("## LEG 1 — is verilator's LEXICAL message vocabulary still fully adjudicated?")
    # ⭐ The refusal path is exercised BEFORE the real scan, on a planted spelling. Without this
    # the guard's only observed outcome is PASS, and a check whose only observed outcome is PASS
    # has not been tested — it has been run
    # ([[a-check-whose-inputs-all-pass-has-not-been-tested]]). This is the leg that proves the
    # guard would actually fire when upstream grows a message nobody has ruled on.
    planted = "EOF in a message no one has ruled on"
    if (LEXICALish_RE.search(f"%Error: t/x.v:1:1: {planted}") is None
            or planted in LEXICAL_PARSE_STAGE or planted in LEXICAL_OTHER_LANE):
        raise SystemExit("REFUSE: the planted-spelling control did not behave as a NEW "
                         "unruled spelling — LEG 1 cannot be trusted to fire.")
    print(f"  PLANTED CONTROL: {planted!r} -> matched by the detector and UNRULED, "
          "so the refusal path fires. PASS")

    seen = Counter()
    for out in sorted(VERILATOR_T.glob("*.out")):
        for m in LEXICALish_RE.findall(read(out)):
            seen[re.sub(r"'[^']*'", "'X'", m).rstrip()] += 1
    unruled = sorted(s for s in seen
                     if s not in LEXICAL_PARSE_STAGE and s not in LEXICAL_OTHER_LANE)
    print(f"  goldens scanned: {len(list(VERILATOR_T.glob('*.out')))}   "
          f"distinct lexical-ish spellings: {len(seen)}")
    for s in sorted(seen):
        lane = ("PARSE-STAGE" if s in LEXICAL_PARSE_STAGE
                else LEXICAL_OTHER_LANE.get(s, "⛔ UNRULED"))
        print(f"    {seen[s]:3d}  {s:<48} {lane}")
    if unruled:
        raise SystemExit(
            "REFUSE: verilator emits a lexical message nobody has ruled on: "
            f"{unruled}. A stage heuristic goes stale silently at exactly this moment — "
            "adjudicate the spelling into LEXICAL_PARSE_STAGE or LEXICAL_OTHER_LANE, and "
            "into VERILATOR_PARSE_STAGE_RE in stimuli/sv/adjudicate_external_corpus.py.")
    print("  ⇒ every spelling is adjudicated; the parse-stage subset is mirrored by "
          "VERILATOR_PARSE_STAGE_RE.\n")

    print("## GROUND TRUTH — controls (a miss ABORTS before any number is printed)")
    ok = True
    pos = classify("verilator", "test_regress/t/x.v",
                   "verilator: driver x.py fails=True and golden .out reports a PARSE-STAGE "
                   "error (.3.24: ...)", goldens)
    ok &= pos == PARSE
    print(f"  POSITIVE   a parse-stage golden basis     -> {pos} "
          f"{'PASS' if pos == PARSE else 'FAIL'}")

    unsup = next((r for r in load(args.manifest)
                  if len(r) > 5 and r[0] == "verilator"
                  and "post-parse tool error" in r[5]
                  and goldens.is_tool_limit(r[1])), None)
    if unsup is None:
        print("  NEGATIVE   no %Error-UNSUPPORTED row available -> FAIL")
        ok = False
    else:
        got = classify(unsup[0], unsup[1], unsup[5], goldens)
        ok &= got == TOOL
        print(f"  NEGATIVE   {Path(unsup[1]).name:<34} -> {got} "
              f"{'PASS' if got == TOOL else 'FAIL — a tool limitation is being counted as '
                                           'upstream testimony that the text parses'}")

    unknown = [r for r in rows if classify(r[0], r[1], r[5], goldens) is None]
    ok &= not unknown
    print(f"  EXHAUSTIVE every row's basis is recognized  -> {len(unknown)} unrecognized "
          f"{'PASS' if not unknown else 'FAIL'}")
    for r in unknown[:8]:
        print(f"               [{r[0]}] {r[1]}\n               basis: {r[5][:110]}")
    print()
    if not ok:
        raise SystemExit("REFUSE: a ground-truth control missed — no number is published.")

    counts, per_suite = Counter(), {}
    for r in rows:
        cls = classify(r[0], r[1], r[5], goldens)
        counts[cls] += 1
        per_suite.setdefault(cls, Counter())[r[0]] += 1

    n = len(rows)
    print(f"## CENSUS — {n} `{TARGET_CLASS}` rows, by UPSTREAM STAGE CLAIM")
    for cls in (PARSE, VALID, POST, TOOL, NONE):
        c = counts.get(cls, 0)
        split = " ".join(f"{k}:{v}" for k, v in sorted(per_suite.get(cls, {}).items()))
        print(f"  {c:4d}  ({c * 100.0 / n:5.1f} %)  {cls:<20} {split}")
    print()
    print(f"  ⇒ ADJUDICATOR WORK sizeable from metadata: {counts.get(PARSE, 0)} of {n} "
          f"({counts.get(PARSE, 0) * 100.0 / n:.1f} %).")
    print(f"  ⇒ PARSER WORK by upstream's own testimony:  "
          f"{counts.get(VALID, 0) + counts.get(POST, 0)} of {n} "
          f"({(counts.get(VALID, 0) + counts.get(POST, 0)) * 100.0 / n:.1f} %).")
    print(f"  ⇒ UNDECIDABLE from metadata (tool-limit or no claim): "
          f"{counts.get(TOOL, 0) + counts.get(NONE, 0)} of {n}.")
    print()
    print("  ⛔ A metadata census CANNOT see the `.3.23` class, and that bound is the point: a row")
    print("     can carry impeccable UPSTREAM_SAYS_VALID testimony and still be LRM-underivable,")
    print("     because upstream is a TOOL and the oracle is the STANDARD. Those rows are found")
    print("     only by reading the construct at the stuck point — i.e. by `.3.x` construct")
    print("     leaves. This number sizes what the METADATA channel can reach, and no more.")
    print("  ⛔ AND IT INHERITS THE ADJUDICATOR'S BLIND SPOT: it classifies the RECORDED")
    print("     derivation, so a row derived WRONG is classified confidently as what the")
    print("     adjudicator believed. The three rows `.3.24` reclassified were invisible to this")
    print("     leg and were found by LEG 1 above. A 0 here means 'nothing left that the recorded")
    print("     reasoning can surface' — never 'nothing left'.")

    if args.show_rows:
        print(f"\n## ROWS — {args.show_rows}")
        for r in rows:
            if classify(r[0], r[1], r[5], goldens) == args.show_rows:
                print(f"  {r[0]:<16} {r[1]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
