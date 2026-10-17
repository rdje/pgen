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
import sys
import json
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
    "divergence:explained_svpp_keywords_region": (
        "ADJUDICATED", "parse stops ON a word a begin_keywords region unreserves - IEEE 1800-2017 "
                       "§22.14, applied by the expansion stage (SVPP-EXPANSION.8); token-exact tell "
                       "(SV-CORPUS-GRAD.18)"),
    "divergence:explained_svpp_macro_definition": (
        "ADJUDICATED", "parse stops ON a text-macro definition directive inside a design element - "
                       "text the preprocessor consumes (IEEE 1800-2017 §22.5.1); token-exact tell "
                       "(SV-CORPUS-GRAD.20)"),
    "divergence:explained_store_gate": (
        "ADJUDICATED", "the sound store gate refuses a provably-undeclared type identifier "
                       "(SV-PARSE-STRICT boundary); proven per row: the file parses once the "
                       "demanded name is declared (.13w)"),
    "deferred:v2005_profile_lane": (
        "ROUTED", "adjudicated in adjudication_manifest_v2005.tsv, not lost"),
    "deferred:chained_only": (
        "NO VERDICT",
        "multi-file design: needs `include/`define chaining to parse honestly - the "
        "MOST realistic RTL in the corpus, contributing nothing to the claim"),
    "deferred:no_sv_key": (
        "NO VERDICT", "no upstream answer key exists - expectation underivable"),
    "deferred:negative_stage_triage": (
        "NO VERDICT", "an ivtest CE whose failure STAGE is unresolved - a regress-vlog95.list CE "
        "(the -t vlog95 flow: its compile error may be the Verilog-95 code generator's, not the "
        "parser's) or a CE with no usable golden outside the pinned population; a golden log or "
        "a pin settles it (SV-CORPUS-GRAD.13e.13)"),
    "deferred:svpp_owned": (
        "NO VERDICT", "conformance owned by the preprocessor lane, by design"),
    "deferred:impl_varying": (
        "NO VERDICT", "LRM leaves the behaviour implementation-defined"),
    "deferred:verilog_ams_lane": (
        "NO VERDICT", "Verilog-AMS, a different language family"),
    "deferred:ni_unimplemented": (
        "NO VERDICT", "upstream marks the construct not-implemented"),
    "deferred:not_source_text": (
        "NO VERDICT", "the file is not SystemVerilog source text at all (a $readmemh memory "
        "image, a bare string literal, a tool command line) - it testifies to nothing about "
        "the parser; a CORPUS-COMPOSITION fact, stated per row (SV-CORPUS-GRAD.13c.1)"),
    "deferred:no_unit": (
        "NO VERDICT", "an OpenTitan file the suite's own compile declarations compile NOWHERE - a "
        "template copy under hw/ip_templates/ (only `.core.tpl` templates, rendered elsewhere) or an "
        "include file a core lists that nothing includes - so the design convention's must-accept has "
        "no subject; a CORPUS-COMPOSITION fact, derived per row from the tree (SV-CORPUS-GRAD.59)"),
    "deferred:vendor_extension_enabled": (
        "NO VERDICT", "the descriptor switches a NON-STANDARD vendor extension ON, so "
        "'it compiled' is testimony about a superset language (SV-CORPUS-GRAD.13e.3)"),
}
BUCKETS = ["ADJUDICATED", "ROUTED", "NO VERDICT"]

# SVPP-EXPANSION.6c.1 — the SAME rows read UNDER THE PIPELINE. `pipeline_readjudication.py`
# (`stimuli/sv/characterization/pipeline_readjudication.tsv`) re-reads every deferred / explained /
# must-reject row through the preprocess→parse pipeline and says what expansion changed; this map
# says how each of its verdicts contributes to the confidence claim. A row the pipeline did not
# re-read keeps its raw bucket. ⛔ Expected verdicts still come from the SPEC / the suite convention,
# never from the fix: a `chained:accepted_under_expansion` row testifies only when the KEY expects the
# file must-accept AS PART OF ITS UNIT (`expected = chained_must_accept`, SV-CORPUS-GRAD.57: a DESIGN
# corpus's file — the adjudicator's `DESIGN_SUITES` convention — or a Surelog multi-source unit whose
# golden log completes clean and whose driver names the file), and a `refuted:parser_question` row is a
# defect CANDIDATE that enters the bar until the corpus tree rules it (its raw class was
# `explained_svpp_*` — a must-accept row).
PIPELINE_CLASSES = {
    "corroborated": ("ADJUDICATED", "a must-accept row the raw lane explained away by preprocessing PASSES "
                                    "once preprocessed — the explanation held; the row testifies FOR the parser"),
    "refuted:parser_question": ("ADJUDICATED", "a must-accept row that still fails in its OWN text once preprocessed, "
                                               "legal by the clause the disposition names — a defect candidate on "
                                               "the axis-2 bar, routed to SV-CORPUS-GRAD"),
    "refuted:key_question": ("NO VERDICT", "the parser is right to refuse; the row's expectation awaits a pin "
                                           "(the .13y family) — no verdict until the key rules"),
    "refuted:preprocessor_item": ("NO VERDICT", "an expansion-stage rule not yet implemented (named per row)"),
    "refuted:cross_file_base_class": ("NO VERDICT", "the class header's base is declared in another file of the unit "
                                                    "(the library design, SVPP-EXPANSION.6d)"),
    "refuted:cross_file_define": ("NO VERDICT", "a predecessor file of the unit defines the macro (the unit design)"),
    "refuted:driver_define": ("NO VERDICT", "the suite's driver defines the macro on its command line (.4c)"),
    "refuted:vendor_identity_branch": ("NO VERDICT", "the file's portable branch is the vendor's own `` `error `` "
                                                     "stub behind its identity macro — a fact about the row's context"),
    "refuted": ("NO VERDICT", "refuted and not yet dispositioned"),
    "hollow": ("NO VERDICT", "the rejection lands in an included file; the header's own row carries it"),
    "refused_context": ("NO VERDICT", "the preprocessor could not supply an include the derived context lacks"),
    "refused_engine": ("NO VERDICT", "the preprocessor refused the row (E_SVPP_* / a malformed directive)"),
    "must_reject:match_under_expansion": ("ADJUDICATED", "a row the key expects rejected IS rejected in its own text "
                                                         "under the pipeline — the raw lane could not reach the construct"),
    "must_reject:accepts_invalid": ("ADJUDICATED", "invalid SV the parser accepts (the axis-2 bar; the raw class "
                                                   "already counts it)"),
    "must_reject:hollow": ("NO VERDICT", "the rejection lands in an included file"),
    "must_reject:refused_context": ("NO VERDICT", "refused for a context gap"),
    "must_reject:refused_engine": ("ADJUDICATED", "a must-reject row the PREPROCESSOR refuses by a rule of its own "
                                                  "(E_SVPP_* / an invalid directive, each naming its clause) — the "
                                                  "rejection the key expects, at the expansion stage (SVPP-EXPANSION.6c.3)"),
    "chained:accepted_under_expansion": ("ADJUDICATED", "a file ACCEPTED under its derived context that the key expects "
                                                        "must-accept AS PART OF ITS UNIT (`chained_must_accept`: a shipping "
                                                        "design's file, or a Surelog multi-source unit whose golden log "
                                                        "completes clean and whose driver names the file — .57); with no "
                                                        "per-file expectation the row keeps NO VERDICT"),
    "accepted_in_unit": ("ADJUDICATED", "a must-accept row the raw lane explained away is a MEMBER of a package unit "
                                        "the library pass ACCEPTED (SVPP-EXPANSION.6d.2: an included class file has no "
                                        "import of its own; its bytes were parsed inside its unit) — the row testifies FOR "
                                        "the parser"),
    "unreached_in_unit": ("NO VERDICT", "a member whose every including unit rejects elsewhere — its bytes were never "
                                        "judged; the unit's origin is named (.6d.2)"),
    "chained:accepted_in_unit": ("ADJUDICATED", "a file ACCEPTED as a MEMBER of its package unit under the library "
                                                "(.6d.2) that the key expects must-accept as part of its unit "
                                                "(`chained_must_accept`); with no per-file expectation the row keeps "
                                                "NO VERDICT"),
    "chained:unreached_in_unit": ("NO VERDICT", "DARK UNDER THE PIPELINE: a member whose every including unit rejects "
                                                "elsewhere (.6d.2) — the unit's own origin row carries the verdict"),
    "chained:rejects_own_file": ("NO VERDICT", "DARK UNDER THE PIPELINE: rejected in its own file under its derived "
                                               "context — the barrier behind the macro barrier (a cross-file "
                                               "declaration, mostly a class base), the library design's population"),
    "chained:explained_store_gate": ("ADJUDICATED", "the sound store gate's refusal UNDER THE PIPELINE (SV-CORPUS-GRAD.57b): a "
                                                    "must-accept-as-part-of-its-unit row, no alterable directive before the failure, "
                                                    "the demanded type name declared nowhere in its unit (nowhere in the suite, or — "
                                                    "`.57c` — only in files outside the row's own unit), proven to parse once "
                                                    "declared by the dark worklist's search, `.13w`'s soundness holding — "
                                                    "explained, never a pass"),
    "chained:hollow": ("NO VERDICT", "the rejection lands in an included file"),
    "chained:refused_context": ("NO VERDICT", "an include the derived context cannot supply"),
    "chained:refused_engine": ("NO VERDICT", "refused by the preprocessor"),
    "svpp_owned:accepted_under_expansion": ("NO VERDICT", "conformance owned by the preprocessor lane; accepted — and "
                                                          "no expectation derived yet for this suite (.6c.3 derives "
                                                          "sv-tests'; the rest keep NO VERDICT)"),
    "svpp_owned:corroborated": ("ADJUDICATED", "a svpp-owned row whose own header says must_accept (sv-tests "
                                               ":type: preprocessing, no :should_fail_because:) PASSES under the "
                                               "pipeline — the preprocessor lane answering the question the raw key "
                                               "deferred to it by design (SVPP-EXPANSION.6c.3)"),
    "svpp_owned:refuted": ("NO VERDICT", "a svpp-owned row whose own suite convention says must_accept still fails in "
                                         "its own text under the pipeline — a defect candidate awaiting its disposition "
                                         "(.6c.3 / .6c.4)"),
    "svpp_owned:refuted:key_question": ("NO VERDICT", "the stage or the parser is right to refuse; the svpp-owned row's "
                                                      "expectation (its suite's own convention) is what needs a ruling (.6c.4)"),
    "svpp_owned:refuted:parser_question": ("ADJUDICATED", "a svpp-owned must-accept row that still fails in its own text, "
                                                          "legal by the clause the disposition names — a defect candidate on "
                                                          "the bar (.6c.4)"),
    "svpp_owned:refuted:vendor_identity_branch": ("NO VERDICT", "the svpp-owned row's portable branch is the vendor's own "
                                                                "stub (.6c.4)"),
    "svpp_owned:preprocess_only_output": ("NO VERDICT", "a verilator -E test the preprocessor expanded as its driver asks; "
                                                        "the expanded text is compared, not parsed, so its parse rejection is "
                                                        "no claim either way (.6c.4)"),
    "svpp_owned:rejects_own_file": ("NO VERDICT", "conformance owned by the preprocessor lane; rejected"),
    "svpp_owned:hollow": ("NO VERDICT", "conformance owned by the preprocessor lane"),
    "svpp_owned:refused_context": ("NO VERDICT", "conformance owned by the preprocessor lane"),
    "svpp_owned:refused_engine": ("NO VERDICT", "conformance owned by the preprocessor lane"),
}
PIPELINE_BAR_CLASSES = {"pipeline:refuted:parser_question", "pipeline:svpp_owned:refuted:parser_question", "pipeline:must_reject:accepts_invalid"}
sys.path.insert(0, str(Path(__file__).resolve().parent))
# SV-CORPUS-GRAD.57: the design-suite convention is read from the KEY (`expected = chained_must_accept`,
# derived by the adjudicator from `pipeline_context.DESIGN_SUITES` — still ONE definition), never from a
# suite set here: a Surelog unit's golden gives the same expectation to a non-design row.
UNIT_MUST_ACCEPT = "chained_must_accept"


def read_readjudication(path: Path):
    """(suite, relpath) -> (verdict, pipeline_status) from the re-adjudication table; {} if absent."""
    if not path.is_file():
        return {}
    out = {}
    with path.open(encoding="utf-8") as fh:
        header = fh.readline().rstrip("\n").split("\t")
        iv, ist = header.index("verdict"), header.index("pipeline_status")
        isuite, irel = header.index("suite"), header.index("relpath")
        for line in fh:
            if not line.strip():
                continue
            c = line.rstrip("\n").split("\t")
            out[(c[isuite], c[irel])] = (c[iv], c[ist])
    return out


def pipeline_reading(rows, readj):
    """Every manifest row's bucket UNDER THE PIPELINE, plus the bar and the dark count there.
    Returns (counts_by_pipeline_class, by_bucket, bar, dark) — the class names of re-read rows are
    prefixed `pipeline:`; rows the pipeline did not re-read keep their raw class."""
    counts, by_bucket, bar, dark = Counter(), Counter(), 0, 0
    for r in rows:
        key = (r["suite"], r["relpath"])
        raw_cls = r["adjudication"]
        if key in readj:
            verdict, status = readj[key]
            if verdict not in PIPELINE_CLASSES:
                raise SystemExit(f"⛔ REFUSING: re-adjudication verdict this census has no bucket for: {verdict!r} "
                                 f"({r['suite']}/{r['relpath']}). Classify it explicitly.")
            bucket, _why = PIPELINE_CLASSES[verdict]
            if verdict in ("chained:accepted_under_expansion", "chained:accepted_in_unit", "chained:explained_store_gate") and r.get("expected") != UNIT_MUST_ACCEPT:
                bucket = "NO VERDICT"
            if verdict == "must_reject:accepts_invalid" and raw_cls.startswith("divergence:"):
                cls = raw_cls                      # already the raw bar's row; no double count
                bucket = CLASSES[raw_cls][0]
            else:
                # `.6c.3`: a must-reject row the raw lane DEFERRED (svpp_owned / chained_only) that the pipeline
                # ACCEPTS is an over-acceptance the raw bar never counted — it is the PIPELINE's bar row
                cls = "pipeline:" + verdict
            counts[cls] += 1
            by_bucket[bucket] += 1
            if cls in PIPELINE_BAR_CLASSES or cls in ("divergence:unexplained_rejects_valid",
                                                       "divergence:unexplained_accepts_invalid"):
                bar += 1
            if bucket == "NO VERDICT" and status != "pass" and raw_cls not in NOT_DARK:
                dark += 1
        else:
            bucket = CLASSES[raw_cls][0]
            counts[raw_cls] += 1
            by_bucket[bucket] += 1
            if raw_cls in ("divergence:unexplained_rejects_valid", "divergence:unexplained_accepts_invalid"):
                bar += 1
            if bucket == "NO VERDICT" and r["observed"] == "fail" and raw_cls not in NOT_DARK:
                dark += 1
    return counts, by_bucket, bar, dark

# SV-CORPUS-GRAD.13c.1: the one NO-VERDICT class whose rows are NOT DARK. DARK means "the parse of
# SOURCE TEXT failed and the deferral is why nobody looked"; a $readmemh image failing to parse
# as SystemVerilog is not a parse nobody looked at, it is a file that was never a parse input.
# The adjudicator decides membership from the BYTES (`not_source_text()`), never from a list.
NOT_SOURCE = "deferred:not_source_text"
# SV-CORPUS-GRAD.59: the second such class - a file no unit compiles was never a compile input either.
NO_UNIT = "deferred:no_unit"
NOT_DARK = {NOT_SOURCE, NO_UNIT}

# --- SV-CORPUS-GRAD.13f: the PERMANENT deferrals, stated as permanent and BOUNDED -----------------
# ⛔ A permanent deferral is a CLAIM — "no future work on the parser, the key or the corpus can give
# this row a verdict" — and until .13f it was indistinguishable from a to-do: the same `deferred:`
# prefix, the same NO VERDICT bucket, a one-line reason that read as a description. Each class below
# states WHY nothing can ever adjudicate it, and the set is BOUNDED: a tracked ceiling register names
# the row count each class may hold and the ARGUMENT every member joins; a run that finds a class above
# its ceiling REFUSES (the gate re-runs this instrument, so the refusal reaches the pre-commit hook),
# and the only way through is to raise the ceiling in the register while naming the argument the new
# rows join. Growing the set silently — one more `impl_varying` row nobody read — is what the bound
# forbids. `deferred:chained_only` and `deferred:no_sv_key` are NOT here: chaining and a future key
# could give those rows verdicts, so they are open work, not permanence.
PERMANENT = {
    "deferred:svpp_owned":
        "the row's conformance question is about the PREPROCESSOR (macro/conditional/include "
        "semantics the parser never sees) and is answered by the svpp lane, by design of the "
        "two-parser architecture — no grammar or key change on this lane can move it",
    "deferred:impl_varying":
        "the suite's own contract marks the verdict implementation-dependent (ispras TYPE VARYING) — "
        "the LRM leaves the behaviour to the implementation, so no answer key can exist",
    "deferred:verilog_ams_lane":
        "the descriptor runs the file under -gverilog-ams: a different LANGUAGE (Verilog-AMS), "
        "outside IEEE 1800's scope; owned by the parked VERILOG-AMS tree, never by this grammar",
    "deferred:ni_unimplemented":
        "the upstream runner marks the construct NI (Not Implemented) and skips the test, so the "
        "suite itself holds no expectation to compare against",
    "deferred:not_source_text":
        "the bytes are not source text in any language this grammar describes - a $readmemh "
        "memory image (IEEE 1800-2017 §21.4), a bare string literal, a tool command line - so no "
        "answer key, no include/define chaining and no grammar revision can produce a verdict "
        "ABOUT SYSTEMVERILOG from them; the row is a statement about the corpus's composition",
    "deferred:no_unit":
        "no unit of the suite compiles the file (OpenTitan: a template copy whose only compile "
        "declarations are `.core.tpl` templates rendered elsewhere, or an include file a core lists "
        "that nothing includes - derived from the tree by the adjudicator, never from a list), so the "
        "design convention 'valid as part of its unit' has no subject: no chaining, no context and no "
        "grammar revision can make a file valid in a unit that does not exist; the row is a statement "
        "about the corpus's composition (SV-CORPUS-GRAD.59)",
}
# DONE-BAR.3f (2026-10-08): the verilog_2005 LANE's permanent classes, stated beside the sv_2017 ones so
# permanence has ONE home. That lane has no coverage instrument of its own (its rows are the sv_2017
# lane's ROUTED rows), so this instrument never counts them; the leg-3 comparator
# (`corpus_conformance_compare.py`) reads this set and holds each class under its ceiling in the same
# register's `ceilings_verilog_2005` section. The two arguments are the sv_2017 lane's, restated for the
# 1364-2005 keys: ispras-1364 marks TYPE VARYING rows implementation-dependent by its own contract, and
# ivtest's NI descriptors are skipped by the upstream runner. `deferred:ni_unimplemented` is the same
# class NAME on both lanes with a different population, which is why the ceilings are per lane.
PERMANENT_V2005 = {
    "deferred:impl_varying_v2005":
        "the suite's own contract marks the verdict implementation-dependent (ispras-1364 TYPE VARYING) — "
        "the LRM leaves the behaviour to the implementation, so no answer key can exist",
    "deferred:ni_unimplemented":
        "the upstream runner marks the construct NI (Not Implemented) and skips the test, so the "
        "suite itself holds no expectation to compare against",
}
DEFAULT_CEILINGS = ("docs/tasks/artifacts/sv_corpus_grad/verdict_coverage/"
                    "permanent_deferral_ceilings.json")


def enforce_permanent_ceilings(counts, register_path):
    """Refuse (exit 1) when a PERMANENT class holds more rows than its recorded ceiling, or when the
    register does not cover every permanent class. Returns the register for rendering."""
    try:
        reg = json.loads(Path(register_path).read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        print(f"corpus_verdict_coverage: REFUSE — the permanent-deferral ceiling register "
              f"{register_path} is unreadable ({e}); a bound nobody can read is no bound",
              file=sys.stderr)
        sys.exit(1)
    ceilings = reg.get("ceilings", {})
    missing = [c for c in PERMANENT if c not in ceilings]
    if missing:
        print(f"corpus_verdict_coverage: REFUSE — {register_path} records no ceiling for "
              f"{', '.join(missing)}; every PERMANENT class must be bounded", file=sys.stderr)
        sys.exit(1)
    over = [(c, counts.get(c, 0), ceilings[c]["ceiling"]) for c in PERMANENT
            if counts.get(c, 0) > int(ceilings[c]["ceiling"])]
    if over:
        for c, n, ceil in over:
            print(f"corpus_verdict_coverage: REFUSE — PERMANENT class `{c}` holds {n} rows, above "
                  f"its recorded ceiling {ceil} (SV-CORPUS-GRAD.13f). A permanent deferral may not "
                  f"grow silently: name the argument the new rows join and raise the ceiling in "
                  f"{register_path} deliberately, in the same commit.", file=sys.stderr)
        sys.exit(1)
    return reg

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
    "deferred:negative_stage_triage":
        "information only — the label is an unresolved failure STAGE (the parser, or a later "
        "stage / the vlog95 code generator); a backtick-free file says the reject is not "
        "expansion's, and still not which stage refused — a golden log or a pin settles it "
        "(SV-CORPUS-GRAD.13e.13)",
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
    "deferred:not_source_text":
        "⛔ **NOT MEASURED, and saying so is the point** — the census asks whether expansion "
        "could alter a failing PARSE of source text; these rows are not source text, so there "
        "is no parse for a directive to reach. They are excluded from DARK by construction "
        "(`.13c.1`) and appear here only so the class table is total",
    "deferred:no_unit":
        "⛔ **NOT MEASURED, and saying so is the point** — expansion could alter these parses, "
        "but no unit of the suite compiles the file (SV-CORPUS-GRAD.59: a template copy rendered "
        "elsewhere, an include file nothing includes), so there is no unit for chaining to read it "
        "through and no expectation to refute. Excluded from DARK by construction and listed only so "
        "the class table is total",
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
               ("suite", "relpath", "observed", "expected", "adjudication", "basis")}
        # `expected` since SV-CORPUS-GRAD.57: the pipeline reading cashes a unit-level must-accept from the KEY
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
    ap.add_argument("--ceilings", default=ROOT / DEFAULT_CEILINGS, type=Path,
                    help="the PERMANENT-deferral ceiling register (SV-CORPUS-GRAD.13f)")
    ap.add_argument("--no-corpus-scan", action="store_true",
                    help="skip the DARK-half backtick census (no corpus file reads)")
    ap.add_argument("--pipeline-readjudication",
                    default=ROOT / "stimuli/sv/characterization/pipeline_readjudication.tsv",
                    type=Path,
                    help="SVPP-EXPANSION.6c.1: the deferred population re-read under the pipeline; "
                         "absent → the reading under the pipeline is NOT EVALUATED")
    args = ap.parse_args()

    rows = read_manifest(args.manifest)
    counts = Counter(r["adjudication"] for r in rows)
    reg = enforce_permanent_ceilings(counts, args.ceilings)
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
        if r["adjudication"] == NOT_SOURCE:
            # SV-CORPUS-GRAD.13c.1: not a parse of source text, so neither testimony nor DARK.
            s["not_source"] += 1
        elif r["adjudication"] == NO_UNIT:
            # SV-CORPUS-GRAD.59: never a compile input of the suite, so neither testimony nor DARK.
            s["no_unit"] += 1
        elif r["observed"] == "pass":
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
                    and r["observed"] == "fail"
                    and r["adjudication"] not in NOT_DARK):
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
    tot_ns = sum(s["not_source"] for s in strata.values())
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
    L.append(f"- **NOT SOURCE TEXT — {tot_ns:,} rows ({100*tot_ns/total:.1f} % of the "
             "corpus)** (SV-CORPUS-GRAD.13c.1): the file is a `$readmemh` memory image, a bare "
             "string literal or a tool command line, decided from its BYTES by the adjudicator. "
             "It never was a parse input, so it is neither testimony nor DARK — it stays in the "
             "denominator as a stated fact about the corpus's composition, and the class is "
             "PERMANENT and bounded below. ⛔ Before `.13c.1` these rows were counted as DARK "
             "`chained_only`, a defect worklist no chaining could ever burn down.")
    L.append("")
    L.append("| class | rows | one-sided (unit) | ⚠️ one-sided (fragment) | DARK | not source |")
    L.append("|---|---:|---:|---:|---:|---:|")
    for cls, n in sorted(counts.items(), key=lambda kv: -kv[1]):
        if CLASSES[cls][0] != "NO VERDICT":
            continue
        s = strata.get(cls, Counter())
        L.append(f"| `{cls}` | {n:,} | {s['one_sided_unit']:,} | "
                 f"{s['one_sided_fragment']:,} | {s['dark']:,} | {s['not_source']:,} |")
    L.append(f"| **TOTAL** | **{tot_unit + tot_frag + tot_dark + tot_ns:,}** | "
             f"**{tot_unit:,}** | **{tot_frag:,}** | **{tot_dark:,}** | **{tot_ns:,}** |")
    L.append("")

    # --- SV-CORPUS-GRAD.13f: the permanent set, stated and bounded ------------------
    L.append("## Permanent deferrals — by construction, and BOUNDED (SV-CORPUS-GRAD.13f)\n")
    L.append("A permanent deferral is a CLAIM: no future work on the parser, the answer key or the "
             "corpus can give these rows a verdict. Each class says why, and each is bounded by a "
             "recorded ceiling the instrument REFUSES to exceed — a new row may join a class only "
             "by naming the argument below and raising the ceiling deliberately "
             f"(`{DEFAULT_CEILINGS}`).\n")
    L.append("| class | rows | ceiling | why nothing can ever adjudicate it |")
    L.append("|---|---:|---:|---|")
    for cls, why in PERMANENT.items():
        L.append(f"| `{cls}` | {counts.get(cls, 0):,} | {int(reg['ceilings'][cls]['ceiling']):,} | {why} |")
    perm_total = sum(counts.get(c, 0) for c in PERMANENT)
    L.append(f"| **TOTAL** | **{perm_total:,}** | | {100*perm_total/total:.1f} % of the corpus, "
             "permanent by construction |")
    L.append("")
    L.append("⛔ `deferred:chained_only` and `deferred:no_sv_key` are NOT permanent: chaining and a "
             "future key could give those rows verdicts, so they stay open work. ⭐ The "
             "FRAGMENT-shaped one-sided positives above (SV-CORPUS-GRAD.13g) are ruled "
             "**unadjudicable as whole files**: a fragment parsed entry-relative would yield an "
             "accept with no key to compare against, so it cannot move the bar either way — the "
             "population stays visible in the fragment column and is never counted as testimony.\n")

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
            # a reason is prose quoted into a table cell: a `|` would add a cell and a backtick run cut
            # by the truncation would open a code span the row never closes (TABLE-ARITY-RATCHET,
            # measured on the .6c.2 pins) — pipes escaped, backticks shown as apostrophes
            shown = shown.replace("|", "\\|").replace("`", "'")
            L.append(f"| `{cls}` | {rn:,} | {shown} |")
    L.append("")

    readj = read_readjudication(args.pipeline_readjudication)
    pipe = None
    if readj:
        p_counts, p_bucket, p_bar, p_dark = pipeline_reading(rows, readj)
        pipe = (p_counts, p_bucket, p_bar, p_dark)
        L.append("## Under the PIPELINE — the same rows re-read through preprocess→parse "
                 "(SVPP-EXPANSION.6c.1)\n")
        L.append(f"> Input: `{args.pipeline_readjudication.relative_to(ROOT)}` "
                 f"(`{sha256(args.pipeline_readjudication)}`) — the re-adjudication table "
                 "(`stimuli/sv/pipeline_readjudication.py`, held current by PIPELINE-LANE-CURRENCY). "
                 "A row the pipeline re-read takes its verdict from there; every other row keeps its raw "
                 "bucket. ⛔ Expected verdicts still come from the spec and the suite convention: a "
                 "chained-only row testifies only in a DESIGN corpus, and a `parser_question` row is a "
                 "defect CANDIDATE on the bar until the corpus tree rules it.\n")
        L.append("**Live verdict-coverage tuple under the pipeline — `adjudicated/routed/no-verdict/dark/axis-2-bar` = "
                 f"`{p_bucket['ADJUDICATED']}/{p_bucket['ROUTED']}/{p_bucket['NO VERDICT']}/{p_dark}/{p_bar}`** "
                 f"(raw: `{by_bucket['ADJUDICATED']}/{by_bucket['ROUTED']}/{by_bucket['NO VERDICT']}/{tot_dark}/"
                 f"{counts.get('divergence:unexplained_rejects_valid', 0) + counts.get('divergence:unexplained_accepts_invalid', 0)}`).\n")
        for b in BUCKETS:
            L.append(f"- **{b}: {p_bucket[b]:,} rows ({100*p_bucket[b]/total:.1f} %)** "
                     f"(raw {by_bucket[b]:,}, {p_bucket[b] - by_bucket[b]:+,})")
        L.append("")
        L.append("| bucket | class under the pipeline | rows | % | why |")
        L.append("|---|---|---:|---:|---|")
        for b in BUCKETS:
            for cls, n in sorted(p_counts.items(), key=lambda kv: -kv[1]):
                if cls.startswith("pipeline:"):
                    v = cls[len("pipeline:"):]
                    bucket, why = PIPELINE_CLASSES[v]
                    if v in ("chained:accepted_under_expansion", "chained:accepted_in_unit"):
                        # split by the design-suite condition
                        continue
                else:
                    bucket, why = CLASSES[cls]
                if bucket != b:
                    continue
                L.append(f"| {b} | `{cls}` | {n:,} | {100*n/total:.1f} % | {why} |")
        for acc_v in ("chained:accepted_under_expansion", "chained:accepted_in_unit"):
            acc_design = sum(1 for r in rows if readj.get((r['suite'], r['relpath']), ('',''))[0] == acc_v and r.get('expected') == UNIT_MUST_ACCEPT)
            acc_other = p_counts.get("pipeline:" + acc_v, 0) - acc_design
            if acc_design:
                L.append(f"| ADJUDICATED | `pipeline:{acc_v}` (key: must-accept as part of its unit) | {acc_design:,} | "
                         f"{100*acc_design/total:.1f} % | {PIPELINE_CLASSES[acc_v][1]} |")
            if acc_other:
                L.append(f"| NO VERDICT | `pipeline:{acc_v}` (no per-file expectation) | {acc_other:,} | "
                         f"{100*acc_other/total:.1f} % | accepted under its context, but the suite gives no per-file expectation |")
        L.append(f"| | **TOTAL** | **{total:,}** | **100.0 %** | |")
        L.append("")
    else:
        L.append("## Under the PIPELINE — NOT EVALUATED\n")
        L.append(f"> `{args.pipeline_readjudication.relative_to(ROOT)}` is absent; run the pipeline lane and "
                 "`stimuli/sv/pipeline_readjudication.py` to re-derive the reading under the pipeline.\n")

    args.outdir.mkdir(parents=True, exist_ok=True)
    (args.outdir / "coverage.md").write_text("\n".join(L) + "\n", encoding="utf-8")
    if pipe is not None:
        p_counts, p_bucket, p_bar, p_dark = pipe
        with (args.outdir / "pipeline_coverage.tsv").open("w", encoding="utf-8") as fh:
            fh.write("bucket\tclass\trows\tpct\n")
            for cls, n in sorted(p_counts.items(), key=lambda kv: -kv[1]):
                if cls.startswith("pipeline:"):
                    v = cls[len("pipeline:"):]
                    if v == "chained:accepted_under_expansion":
                        fh.write(f"ADJUDICATED\t{cls}\t{acc_design}\t{100*acc_design/total:.2f}\n")
                        if acc_other:
                            fh.write(f"NO VERDICT\t{cls}\t{acc_other}\t{100*acc_other/total:.2f}\n")
                        continue
                    bucket = PIPELINE_CLASSES[v][0]
                else:
                    bucket = CLASSES[cls][0]
                fh.write(f"{bucket}\t{cls}\t{n}\t{100*n/total:.2f}\n")
            fh.write(f"TUPLE\tadjudicated/routed/no-verdict/dark/axis-2-bar\t"
                     f"{p_bucket['ADJUDICATED']}/{p_bucket['ROUTED']}/{p_bucket['NO VERDICT']}/{p_dark}/{p_bar}\t\n")
    with (args.outdir / "coverage.tsv").open("w", encoding="utf-8") as fh:
        fh.write("bucket\tclass\trows\tpct\n")
        for cls, n in sorted(counts.items(), key=lambda kv: -kv[1]):
            fh.write(f"{CLASSES[cls][0]}\t{cls}\t{n}\t{100*n/total:.2f}\n")
    with (args.outdir / "strata.tsv").open("w", encoding="utf-8") as fh:
        fh.write("class\trows\tone_sided_unit\tone_sided_fragment\tdark\tnot_source\t"
                 "dark_no_backtick\n")
        for cls, n in sorted(counts.items(), key=lambda kv: -kv[1]):
            if CLASSES[cls][0] != "NO VERDICT":
                continue
            s = strata.get(cls, Counter())
            fh.write(f"{cls}\t{n}\t{s['one_sided_unit']}\t"
                     f"{s['one_sided_fragment']}\t{s['dark']}\t{s['not_source']}\t"
                     f"{'' if args.no_corpus_scan else no_tick[cls]}\n")

    for b in BUCKETS:
        print(f"{b:>12}: {by_bucket[b]:6,}  ({100*by_bucket[b]/total:.1f} %)")
    if pipe is not None:
        p_counts, p_bucket, p_bar, p_dark = pipe
        print(f"{'PIPELINE':>12}: {p_bucket['ADJUDICATED']}/{p_bucket['ROUTED']}/{p_bucket['NO VERDICT']}/{p_dark}/{p_bar}"
              "  (adjudicated/routed/no-verdict/dark/axis-2-bar under the pipeline)")
    if undecidable:
        print(f"{'+undecidable':>12}: {undecidable:6,}  (inside ADJUDICATED)")
    print(f"{'one-sided':>12}: {tot_unit + tot_frag:6,}  (of which "
          f"{tot_frag:,} fragment-shaped — an accept there is NOT testimony)")
    print(f"{'DARK':>12}: {tot_dark:6,}  ({100*tot_dark/total:.1f} % — the burn-down "
          "population)")
    print(f"{'not-source':>12}: {tot_ns:6,}  (not SystemVerilog source text — a fact about the "
          "corpus, neither testimony nor DARK)")
    if not args.no_corpus_scan:
        print(f"{'no-backtick':>12}: {sum(no_tick.values()):6,}  (DARK rows no "
              "preprocessing can alter)")
    # Repo-root-relative when it can be (the project's path policy), absolute when the
    # caller pointed --outdir elsewhere — reporting where the file went must not raise.
    out_md = args.outdir / "coverage.md"
    print(f"wrote {out_md.relative_to(ROOT) if out_md.is_relative_to(ROOT) else out_md}")


if __name__ == "__main__":
    main()
