#!/usr/bin/env python3
"""The verdict only EXPANSION can give — the deferred population of the SV answer key, re-read
under the preprocess→parse pipeline (SVPP-EXPANSION.6a).

WHY THIS EXISTS
---------------
The raw lane (`stimuli/run_external_corpus.sh sv`) parses each file in isolation and its answer
key (`adjudication_manifest.tsv`) is honest about what that cannot decide: a must-accept row whose
raw parse stops on a macro use, a conditional or an `include is `divergence:explained_svpp_*` — the
explanation is a POSITION test (SV-CORPUS-GRAD.12a), not a parse — and a multi-file design file is
`deferred:chained_only`, no verdict at all. `.12b` sized the rows position cannot settle (263) and
`.13c`/`.13d` sized the dark chained-only rows expansion really owns (the pre-registered 1 527);
both leaves say the same thing: THE ONLY INSTRUMENT THAT SETTLES THEM IS EXPANSION ITSELF.

The pipeline lane (`stimuli/run_pipeline_lane.sh`, `SVPP-EXPANSION.4`/`.4b`) is that instrument.
This script joins its dump with the answer key and the two per-row audits those leaves left behind,
and says, row by row, what expansion changed:

  corroborated   a must-accept row the raw lane explained away by preprocessing PASSES once
                 preprocessed — the explanation was right, and the row now testifies FOR the parser
  refuted        a must-accept row the raw lane explained away still FAILS after preprocessing, in
                 its OWN file — the explanation was wrong; the row is a defect candidate that needs
                 a disposition (a parser defect, or an answer-key error), one by one (`.6b`)
  hollow         the rejection lands in an INCLUDED file — the row's own text was never judged;
                 the verdict belongs to the header, and the header's own row carries it
  refused        the preprocessor refused the row — a context gap (an include the derived context
                 does not supply) or an engine refusal (`E_SVPP_*`, a malformed directive)
  accepted_in_unit / unreached_in_unit  (SVPP-EXPANSION.6d.2) a row that fails PER FILE but is a
                 MEMBER of a package unit the library pass parsed (`pipeline_library_members.tsv`, from
                 the preprocessor's source map; `pipeline_library.tsv`, the unit's verdict): an
                 `` `include ``d class file has no `import` of its own, so its verdict is its UNIT's at
                 that file's origin — `accepted_in_unit` when a unit that includes it PASSES (every
                 byte of the member was parsed there), the per-file verdict kept when the unit's
                 rejection lands IN this file, `unreached_in_unit` when every unit that includes it
                 rejects elsewhere (the member's bytes were never judged; the unit's origin is named).
                 Read for the explained_svpp and chained_only populations only.
  chained:*      the same four outcomes for a `deferred:chained_only` row — reported with that
                 prefix because the answer key carries NO expected verdict for the row: a pass under
                 its context is "accepted under expansion", not yet a `match`. ⛔ A row earns a
                 `match` only when its EXPECTED verdict is derived from the suite's own convention
                 (corpus-expected-from-SPEC, never from the fix); that derivation is `.6c`'s.
  chained:explained_store_gate  (SV-CORPUS-GRAD.57b) a `chained:rejects_own_file` row the key expects
                 must-accept as part of its unit, whose failure no alterable directive precedes (`.13c`:
                 the expanded text IS the file's own text) on a type name declared NOWHERE IN ITS UNIT —
                 `.13c`'s `UNDECLARED-ANYWHERE` (nowhere in the suite) or `.57c`'s `FACT-GATED-OUT-OF-UNIT`
                 (in the suite, but in no file of the row's own unit: Surelog, another test's directory) —
                 and which the dark worklist's fixed-point search PROVED parses bare once the names the
                 parser demanded and lacked are declared (`adjudicate_dark_worklist.py`, CROSS-FILE-FACTS,
                 the minimal prelude published in the `unit` column) — with `.13w`'s soundness conditions
                 holding at the refused use (no wildcard import before it, no package export, no LRM-
                 predefined name): the sound store gate's refusal (SV-PARSE-STRICT), ADJUDICATED-explained
                 under the pipeline exactly as `divergence:explained_store_gate` is on the raw lane;
                 never a pass
  must_reject:*  every row whose EXPECTED verdict is must_reject — from the manifest's `expected`
                 column, or from the answer key's own PIN TABLES when the raw lane demoted the pin
                 (`reject_arm_demotion`: the raw parse stops on a macro use above the construct, so
                 the raw class is `deferred:svpp_owned` and the manifest's `expected` reads
                 `out_of_scope_with_cause`; the pin, with its LRM cite, is the key's ruling and it
                 is read here from the adjudicator module itself, SVPP-EXPANSION.6b.1) —
                 `match_under_expansion` (rejected in its own file), `accepts_invalid` (the pipeline
                 ACCEPTS text the standard cannot derive: an over-acceptance defect candidate),
                 `hollow`, `refused_*`.

⛔ NOTHING here rewrites the answer key. The manifest is read, never written; the report is a
derivation of tracked inputs plus the lane dump, held current by `PIPELINE-LANE-CURRENCY` (`--check`).

INPUTS (repo-root-relative)
---------------------------
  stimuli/sv/characterization/adjudication_manifest.tsv           the answer key (raw lane)
  stimuli/sv/characterization/pipeline_results.tsv                the pipeline lane's dump (.gitignored)
  docs/tasks/artifacts/sv_corpus_grad/explained_svpp_audit/after/audit.tsv   `.12a`/`.12b`: the
      position verdict per explained_svpp row (STUCK-ON-SVPP-TOKEN / IN-WINDOW / PAST-LAST-TICK)
  docs/tasks/artifacts/sv_corpus_grad/dark_chained_only/audit.tsv            `.13c`: the dark
      chained-only rows' position verdict and cross-file CHANNEL

  stimuli/sv/characterization/refuted_dispositions.tsv          `.6b`'s DISPOSITION TABLE: one row per
      `refuted` row that has been looked at — `suite relpath disposition basis` — hand-written like the
      answer key's pins, each with its evidence. Validated here: a row that is not (or no longer) a
      `refuted` row is REFUSED, so the table cannot carry a disposition for a row that moved.

OUTPUTS
-------
  stimuli/sv/characterization/pipeline_readjudication.md    the tracked report
  stimuli/sv/characterization/pipeline_readjudication.tsv   one row per re-read manifest row:
      suite  relpath  raw_class  prior_stratum  pipeline_status  landing  construct  verdict  origin
      (a `refuted` row's verdict reads `refuted:<disposition>` once the table dispositions it)

USAGE
-----
  python3 stimuli/sv/pipeline_readjudication.py            # render both outputs + one summary line
  python3 stimuli/sv/pipeline_readjudication.py --check    # re-render to rust/target/ and diff; drift → 1,
                                                           # an absent dump → NOT EVALUATED (0), an
                                                           # empty dump → REFUSED (2)
"""
from __future__ import annotations

import argparse
import csv
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import pipeline_vs_raw as pv  # noqa: E402  (the lane report: the join, the buckets, the identity)
import adjudicate_external_corpus as adj  # noqa: E402  (the answer key's pin tables — read, never written)
from store_gate_explanations import soundness_reason  # noqa: E402  (SV-CORPUS-GRAD.13w's conditions 2..4, .57b)

CHAR = ROOT / "stimuli" / "sv" / "characterization"
ART = ROOT / "docs" / "tasks" / "artifacts" / "sv_corpus_grad"
EXPLAINED_AUDIT = ART / "explained_svpp_audit" / "after" / "audit.tsv"
DARK_AUDIT = ART / "dark_chained_only" / "audit.tsv"
DARK_ADJUDICATION = ART / "dark_chained_only" / "adjudication.tsv"   # .13c.2: the fixed-point proof per dark row (.57b reads it)
STORE_GATE_STRATUM = "NO-ALTERABLE-TICK/UNDECLARED-ANYWHERE"         # .57b: raw text == expanded text, declared nowhere in the suite
STORE_GATE_STRATA = (STORE_GATE_STRATUM,
                     "NO-ALTERABLE-TICK/FACT-GATED-OUT-OF-UNIT")       # .57c: declared in the suite, in NO file of the row's own unit
STORE_GATE_VERDICT = "chained:explained_store_gate"
DISPOSITIONS = CHAR / "refuted_dispositions.tsv"
LIBRARY = CHAR / "pipeline_library.tsv"                 # SVPP-EXPANSION.6d.1: the library pass's per-unit verdicts (a dump)
LIBRARY_MEMBERS = CHAR / "pipeline_library_members.tsv"  # SVPP-EXPANSION.6d.2: (suite, package file, member file) (a dump)

# `.6b`'s disposition vocabulary — what a `refuted` row turned out to be, once looked at
DISPOSITION_KINDS = {
    "cross_file_base_class": "the class header's base class is declared in another file of the unit "
                             "(uvm_pkg for the sv-tests `*-uvm.sv` rows) — the library design's population "
                             "(`SVPP-EXPANSION.6d`); no expansion can supply a declaration",
    "vendor_identity_branch": "the file's portable branch is the vendor's own `` `error `` stub behind "
                              "`` `ifndef VERILATOR ``; the vendor branch needs the tool's identity predefine, "
                              "which the lane does not carry (measured `.6b.3`: defining it recognises 5 "
                              "files and rejects 24 on vendor extensions) — a fact about the row's context",
    "driver_define": "the origin line uses a macro the suite's DRIVER defines on its command line "
                     "(`+define+…`, `-D…`) that the derived context CANNOT carry: the driver computes the "
                     "value at run time, or several variant drivers configure the file differently "
                     "(`.4c.1` derives a literal value from the file's own driver and REPORTS both cases, "
                     "never guesses) — a fact about the row's context",
    "parser_question": "the expansion is legal text the parser refuses, or text the standard cannot derive: "
                       "a grammar / answer-key question routed to `SV-CORPUS-GRAD` with the construct named",
    "preprocessor_item": "an expansion-stage rule not yet implemented (named) — the expansion stage's own",
    "cross_file_define": "the origin line uses a macro a PREDECESSOR FILE of the compilation unit defines "
                          "(§22.5.1: a macro is unit-wide once defined) — a sibling the harness passes first. "
                          "Since `.4c.2` the lane carries the units the tree DECLARES (verilator flag lists, slang "
                          "`.f` files, sv2v `run.sh`); a row here names a pairing NO declaration in the tree states "
                          "— reported by the deriver as underivable, never guessed",
    "key_question": "the parser is RIGHT to refuse: the expansion is text the standard cannot derive, or the "
                    "fixture exercises a tool's error path — the row's expectation is the question, routed to "
                    "the answer key (a pin with the clause, in the `.13y` family)",
}

POPULATIONS = (
    ("explained_svpp", "must-accept rows the raw lane explained away by preprocessing "
                       "(`divergence:explained_svpp_*`; `.12a`'s position test, `.12b`'s 263)"),
    ("chained_only", "multi-file design rows the raw lane could not judge (`deferred:chained_only`; "
                     "`.13c`'s strata, `.13d`'s expansion-owned rows)"),
    ("svpp_owned", "rows whose conformance the raw lane routed to the preprocessor lane "
                   "(`deferred:svpp_owned`) and for which this lane derives no expectation yet"),
    ("svpp_owned_expected", "svpp_owned rows whose own suite convention this lane reads as must_accept — sv-tests "
                            "`:type: preprocessing` tests without `:should_fail_because:` (`.6c.3`), verilator "
                            "preprocessor tests whose driver expects success or whose golden is a lint verdict "
                            "(`.6c.4`; a `-E` driver's must_accept is \"must preprocess\") — the answer key's own "
                            "functions in their pipeline mode; a should-fail header, a preprocessor- or "
                            "parse-stage golden, or sv2v's `error/` contract joins must_reject"),
    ("must_reject", "rows the answer key expects REJECTED — the manifest's `expected`, or a pin the raw "
                    "lane demoted (`.6b.1`); a pipeline pass here is an over-acceptance candidate"),
)

TSV_COLUMNS = ["suite", "relpath", "raw_class", "prior_stratum", "pipeline_status", "landing",
               "unit",  # .6d.2: the package unit the reading came from (empty = none)
               "construct", "verdict", "origin"]


def rel(p: pathlib.Path) -> str:
    return str(p.relative_to(ROOT)) if p.is_absolute() and p.is_relative_to(ROOT) else str(p)


def read_dispositions(path: pathlib.Path) -> dict[tuple[str, str], tuple[str, str]]:
    """The `.6b` disposition table: (suite, relpath) -> (disposition, basis); absent = empty."""
    if not path.exists():
        return {}
    out: dict[tuple[str, str], tuple[str, str]] = {}
    with open(path, newline="") as fh:
        for r in csv.DictReader(fh, delimiter="\t"):
            if r["disposition"] not in DISPOSITION_KINDS:
                raise SystemExit(f"PIPELINE-READJUDICATION: REFUSED — {rel(path)}: unknown disposition "
                                 f"{r['disposition']!r} for {r['suite']}/{r['relpath']} (known: "
                                 f"{', '.join(sorted(DISPOSITION_KINDS))})")
            out[(r["suite"], r["relpath"])] = (r["disposition"], r["basis"])
    return out


def read_audit(path: pathlib.Path) -> dict[tuple[str, str], dict[str, str]]:
    with open(path, newline="") as fh:
        return {(r["suite"], r["relpath"]): r for r in csv.DictReader(fh, delimiter="\t")}


def read_dark_adjudication(path: pathlib.Path) -> dict[tuple[str, str], dict[str, str]]:
    if not path.exists():
        return {}
    with open(path, newline="") as fh:
        return {(r["suite"], r["relpath"]): r for r in csv.DictReader(fh, delimiter="\t")}


def store_gate_explanation(verdict: str, expected: str, stratum: str, adjudicated: dict | None,
                           audit_row: dict | None, read_text) -> tuple[str, str] | None:
    """SV-CORPUS-GRAD.57b — the `.13w` store-gate explanation UNDER THE PIPELINE. A `chained:rejects_own_file` row
    earns `chained:explained_store_gate` only when every condition below is decided mechanically, in this order
    (the cheap ones first; the file is read only when they hold):
      1. the key expects the file must-accept AS PART OF ITS UNIT (`chained_must_accept`);
      2. `.13c`'s audit found NO alterable directive before the failure (the expanded text is the file's own text,
         so the raw-text proof is the pipeline's) and the demanded name declared NOWHERE IN THE ROW'S UNIT — either
         nowhere in the suite (`NO-ALTERABLE-TICK/UNDECLARED-ANYWHERE`, `.57b`) or, `.57c`, in the suite but in no
         file of the row's own unit (`NO-ALTERABLE-TICK/FACT-GATED-OUT-OF-UNIT`: Surelog, another test's directory
         — the unit's compile cannot see it); a name a file of the unit declares is the `.6d` barrier, kept dark;
      3. the dark worklist's fixed-point search (`adjudicate_dark_worklist.py`) PROVED the file parses bare once the
         names the parser demanded and lacked are declared (`CROSS-FILE-FACTS`, not overridden by a RESIDUAL_ROWS
         ruling), the minimal prelude published;
      4. `.13w`'s soundness conditions hold at the refused use — no wildcard import of a package the file does not
         declare BEFORE the use (IEEE 1800-2017 §26.3, positional), no package `export`, no LRM-predefined name —
         else the rejection may be the gate being unsound, a defect for SV-PARSE-STRICT, and the row stays dark.
    Returns (verdict, the prelude note for the `unit` column) or None (the row keeps `chained:rejects_own_file`)."""
    if verdict != "chained:rejects_own_file" or expected != "chained_must_accept" or stratum not in STORE_GATE_STRATA:
        return None
    if not adjudicated or adjudicated.get("verdict") != "CROSS-FILE-FACTS" or adjudicated.get("final_verdict") != "CROSS-FILE-FACTS":
        return None
    names = [n for n in (adjudicated.get("declarations") or "").split(",") if n]
    if not names:
        return None
    stuck = (audit_row or {}).get("stuck", "")
    anchor = int(stuck) if stuck.isdigit() else None
    if soundness_reason(adj.blank_comments_and_strings(read_text()), names, anchor):
        return None
    return STORE_GATE_VERDICT, f"parses once declared: {','.join(sorted(names))} ({adjudicated.get('form', '')})"


def self_test() -> int:
    """The `.57b` explanation on literal rows — every condition observed refusing and the GREEN arm passing."""
    ok_adj = {"verdict": "CROSS-FILE-FACTS", "final_verdict": "CROSS-FILE-FACTS", "declarations": "DD,id_t1", "form": "class"}
    plain = lambda: "module m; id_t1 x; DD y; endmodule\n"
    wild_before = lambda: "import pkg::*;\nmodule m; id_t1 x; endmodule\n"
    wild_after = lambda: "module m; id_t1 x; endmodule\nimport pkg::*;\n"
    V, E, S = "chained:rejects_own_file", "chained_must_accept", STORE_GATE_STRATUM
    checks = [
        ("GREEN: must-accept, no alterable tick, declared nowhere, CROSS-FILE-FACTS, sound → explained",
         store_gate_explanation(V, E, S, ok_adj, {"stuck": "20"}, plain) == (STORE_GATE_VERDICT, "parses once declared: DD,id_t1 (class)")),
        ("a row the key does not expect must-accept stays dark", store_gate_explanation(V, "chained_only", S, ok_adj, {"stuck": "20"}, plain) is None),
        ("a PRE-TICK stratum (expansion could alter the text) stays dark", store_gate_explanation(V, E, "PRE-TICK/UNDECLARED-ANYWHERE", ok_adj, {"stuck": "20"}, plain) is None),
        ("a FACT-GATED-CROSS-FILE stratum (declared in a file of the row's own unit — the .6d barrier) stays dark",
         store_gate_explanation(V, E, "NO-ALTERABLE-TICK/FACT-GATED-CROSS-FILE", ok_adj, {"stuck": "20"}, plain) is None),
        ("GREEN (.57c): a FACT-GATED-OUT-OF-UNIT stratum (declared in the suite, in no file of the row's unit) is explained",
         store_gate_explanation(V, E, "NO-ALTERABLE-TICK/FACT-GATED-OUT-OF-UNIT", ok_adj, {"stuck": "20"}, plain) == (STORE_GATE_VERDICT, "parses once declared: DD,id_t1 (class)")),
        ("a PRE-TICK out-of-unit stratum (expansion could alter the text) stays dark",
         store_gate_explanation(V, E, "PRE-TICK/FACT-GATED-OUT-OF-UNIT", ok_adj, {"stuck": "20"}, plain) is None),
        ("a RESIDUAL dark adjudication (no prelude makes it parse) stays dark", store_gate_explanation(V, E, S, dict(ok_adj, verdict="RESIDUAL", final_verdict="RESIDUAL"), {"stuck": "20"}, plain) is None),
        ("a proof a RESIDUAL_ROWS ruling overrode stays dark", store_gate_explanation(V, E, S, dict(ok_adj, final_verdict="INVALID-SV"), {"stuck": "20"}, plain) is None),
        ("a proof with no declarations is no proof", store_gate_explanation(V, E, S, dict(ok_adj, declarations=""), {"stuck": "20"}, plain) is None),
        ("a wildcard import BEFORE the refused use keeps the row dark (the gate may be unsound there)",
         store_gate_explanation(V, E, S, ok_adj, {"stuck": "30"}, wild_before) is None),
        ("a wildcard import AFTER the refused use does not (IEEE 1800-2017 §26.3 is positional)",
         store_gate_explanation(V, E, S, ok_adj, {"stuck": "20"}, wild_after) is not None),
        ("a verdict other than rejects_own_file is never re-read", store_gate_explanation("chained:hollow", E, S, ok_adj, {"stuck": "20"}, plain) is None),
    ]
    failed = 0
    for label, good in checks:
        print(f"  {'PASS' if good else 'FAIL'} {label}")
        failed += 0 if good else 1
    print(f"PIPELINE-READJUDICATION-SELF-TEST: checks={len(checks)} failed={failed}")
    return 7 if failed else 0


def pinned_expected(suite: str, relpath: str) -> str | None:
    """The answer key's PIN for a row, read from the adjudicator's own tables (its source of truth),
    so a must_reject the raw lane demoted to `out_of_scope_with_cause` is still the key's ruling here."""
    tables = [adj.EXTRA_PINNED.get((suite, relpath))]
    if suite == "verilator":
        tables.append(adj.VERILATOR_PINNED.get(relpath))
    elif suite == "sv-tests":
        tables.append(adj.SVTESTS_PINNED.get(relpath))
    elif suite == "ispras-sv-tests":
        tables.append(adj.ISPRAS_NEGATIVE_PINNED.get(relpath))
    elif suite == "iverilog":
        tables.append(adj.IVTEST_CE_STAGE_PINNED.get(relpath.removeprefix("ivtest/")))
    for t in tables:
        if t:
            return t[0]
    return None


_VERILATOR_INDEX: list = []   # built once; the answer key's own index over the drivers and goldens


def svpp_owned_expected(suite: str, relpath: str) -> tuple[str, bool] | None:
    """SVPP-EXPANSION.6c.3 / .6c.4 — the expectation of a `deferred:svpp_owned` row UNDER THIS LANE, derived by
    the answer key's own functions in their pipeline mode — the raw key deferred these rows to the preprocessor
    lane BY DESIGN, and this lane is it: sv-tests by the file's own header (`.6c.3`), verilator by its driver
    and golden with the preprocessor-stage vocabulary beside the parse-stage one, sv2v's `error/` by the
    suite's contract (`.6c.4`). Returns (expected, preprocess_only): the flag marks a verilator `-E` driver,
    whose must_accept means "must preprocess" — the parse of the expanded text is not the test's claim.
    None: no derivation for this suite yet (the row keeps its expectation-free verdict)."""
    if suite == "sv-tests":
        path = ROOT / "stimuli" / "sv" / "subs" / "sv-tests" / relpath
        try:
            text = path.read_text(errors="replace")
        except OSError:
            return None
        expected, _basis = adj.expect_sv_tests(relpath, text, lane="pipeline")
        return (expected, False) if expected in ("must_accept", "must_reject") else None
    if suite == "verilator":
        if not _VERILATOR_INDEX:
            _VERILATOR_INDEX.append(adj.VerilatorIndex(ROOT / "stimuli" / "sv" / "subs" / "verilator" / "test_regress" / "t"))
        expected, basis = _VERILATOR_INDEX[0].expect(relpath, lane="pipeline")
        if expected not in ("must_accept", "must_reject"):
            return None
        return (expected, "(preprocess_only)" in basis)
    if suite == "sv2v":
        expected, _basis = adj.expect_sv2v(relpath, lane="pipeline")
        return (expected, False) if expected in ("must_accept", "must_reject") else None
    return None


def expected_of(m: list[str]) -> str:
    """The row's EXPECTED verdict: the manifest's column, unless a pin the raw lane demoted says
    otherwise (the demotion rewrites the column to `out_of_scope_with_cause` but keeps the cite), or —
    for a `deferred:svpp_owned` row — the derivation this lane owns (`svpp_owned_expected`)."""
    if m[3] == "must_reject":
        return "must_reject"
    # a demoted pin: macro/conditional-dependent rows demote to svpp_owned, `include-dependent
    # rows to chained_only (`reject_arm_demotion`, both arms) — the pin is the key's ruling either way
    if m[4] in ("deferred:svpp_owned", "deferred:chained_only") and pinned_expected(m[0], m[1]) == "must_reject":
        return "must_reject"
    if m[4] == "deferred:svpp_owned":
        derived = svpp_owned_expected(m[0], m[1])
        if derived is not None:
            return derived[0]
    return m[3]


def preprocess_only_of(m: list[str]) -> bool:
    """`.6c.4`: a verilator `-E` driver's row — its must_accept is "must preprocess"."""
    if m[4] != "deferred:svpp_owned":
        return False
    derived = svpp_owned_expected(m[0], m[1])
    return bool(derived and derived[1])


def population_of(raw_class: str, expected: str) -> str | None:
    if expected == "must_reject":
        return "must_reject"
    if raw_class.startswith("divergence:explained_svpp"):
        return "explained_svpp"
    if raw_class == "deferred:chained_only":
        return "chained_only"
    if raw_class == "deferred:svpp_owned":
        # `.6c.3`: a svpp_owned row whose expectation THIS lane derives as must_accept is judged like an
        # explained_svpp row (a must-accept row the raw lane could not reach): corroborated / refuted
        return "svpp_owned_expected" if expected == "must_accept" else "svpp_owned"
    return None


def landing_of(status: str, repo_rel: str, origin: str) -> str:
    if status == "pass":
        return "-"
    if status == "fail":
        m = re.match(r"(.+?):(\d+):(\d+)$", origin)
        origin_file = m.group(1) if m else origin
        return "own file" if origin_file == repo_rel else "included file"
    if status == "pp_fail":
        # `SVPP-EXPANSION.5`(q): the include bound's refusal ("include nesting deeper than N levels …", IEEE 1800
        # §22.4) is the ENGINE's bound, not a missing file; the spelling "cycle detected" it replaced stays for
        # the frozen dumps that still carry it.
        # `SVPP-EXPANSION.6f`: a backtick that survives expansion is refused at the stage — an UNDEFINED name
        # ("neither a defined macro nor a compiler directive") lands as CONTEXT (the derived context supplies
        # no definition); a function-like macro used bare or with an unclosed list, a macro-string token
        # outside macro text and an unmatched `end_keywords are the file's own errors — ENGINE refusals.
        return "engine" if re.search(r"E_SVPP_|invalid `|without matching|cycle detected|nesting deeper|unterminated|used without a complete argument list|outside macro text|closes no open", origin) else "context"
    return status


def verdict_of(population: str, status: str, landing: str, preprocess_only: bool = False) -> str:
    prefix = {"explained_svpp": "", "chained_only": "chained:", "svpp_owned": "svpp_owned:",
              "svpp_owned_expected": "svpp_owned:", "must_reject": "must_reject:"}[population]
    judged = population in ("explained_svpp", "svpp_owned_expected")   # a must-accept expectation exists
    if status == "pass":
        if population == "must_reject":
            return prefix + "accepts_invalid"
        return prefix + ("corroborated" if judged else "accepted_under_expansion")
    if status == "fail":
        if landing == "own file":
            if population == "must_reject":
                return prefix + "match_under_expansion"
            if judged and preprocess_only:
                # `.6c.4`: a verilator -E test whose expanded text the parser rejects — the driver compares the
                # TEXT, the parse is not its claim; the preprocessor did what the test asks
                return prefix + "preprocess_only_output"
            return prefix + ("refuted" if judged else "rejects_own_file")
        return prefix + "hollow"
    if status == "pp_fail":
        return prefix + ("refused_engine" if landing == "engine" else "refused_context")
    return prefix + status  # timeout / crash, reported as themselves


def read_units(library: pathlib.Path, members: pathlib.Path) -> dict[tuple[str, str], list[tuple[str, str, str]]]:
    """(suite, member relpath) → [(package relpath, unit status, unit origin)] for every unit including it."""
    if not library.exists() or not members.exists():
        return {}
    verdict = {r[2]: (r[1], r[7]) for r in pv.read_tsv(library, 10)}     # package relpath → (status, origin)
    units: dict[tuple[str, str], list[tuple[str, str, str]]] = {}
    for r in pv.read_tsv(members, 3):
        suite, pkg, member = r[0], r[1], r[2]
        if pkg in verdict:
            st, origin = verdict[pkg]
            units.setdefault((suite, pv.suite_relpath(suite, member)), []).append((pkg, st, origin))
    return units


def unit_reading(row: dict[str, str], repo_rel: str, units: list[tuple[str, str, str]]) -> tuple[str, str] | None:
    """(verdict, unit) for a per-file failing member, or None when the per-file verdict stands."""
    if row["population"] not in ("explained_svpp", "chained_only") or row["pipeline_status"] not in ("fail", "pp_fail"):
        return None
    prefix = "chained:" if row["population"] == "chained_only" else ""
    passing = [u for u in units if u[1] == "pass"]
    if passing:
        return prefix + "accepted_in_unit", passing[0][0]
    here = [u for u in units if u[1] == "fail" and u[2].startswith(repo_rel + ":")]
    if here:
        return None                                    # the unit's rejection lands in this file: the per-file verdict IS the unit's
    elsewhere = [u for u in units if u[1] in ("fail", "pp_fail")]
    if elsewhere:
        return prefix + "unreached_in_unit", elsewhere[0][0]
    return None


def prior_stratum(population: str, key: tuple[str, str], explained: dict, dark: dict, raw_class: str = "") -> str:
    if population == "explained_svpp":
        a = explained.get(key)
        return a["verdict"] if a else "not-in-audit"
    if population == "chained_only":
        d = dark.get(key)
        if d is None:
            return "ONE-SIDED (raw pass)"
        return d["verdict"] + ("/" + d["channel"] if d["channel"] else "")
    if population == "must_reject":
        return "pinned, raw lane demoted" if raw_class == "deferred:svpp_owned" else raw_class
    return "-"


def render(rows: list[dict[str, str]], probe: pathlib.Path, dump: pathlib.Path, manifest: pathlib.Path,
           missing_dump_rows: int, units: dict | None = None) -> str:
    units = units or {}

    def count(pred) -> int:
        return sum(1 for r in rows if pred(r))
    L: list[str] = []
    L.append("# The verdict only expansion can give — the deferred SV population re-read under the pipeline\n")
    L.append("> Generated by `stimuli/sv/pipeline_readjudication.py` (SVPP-EXPANSION.6a) from the answer key, the "
             "`.12a`/`.13c` per-row audits and the pipeline lane's dump; held current by `PIPELINE-LANE-CURRENCY`. "
             "⛔ The answer key is READ, never written: a row earns a verdict by being PARSED under honest "
             "conditions, and a `chained:` outcome is not a `match` until the row's EXPECTED verdict is derived "
             "from its suite's own convention (`.6c`).\n")
    L.append("## Instrument identity\n")
    L.append("| input | repo-root-relative path | sha256 |\n|---|---|---|")
    for label, path in (("answer key", manifest), ("explained_svpp audit", EXPLAINED_AUDIT),
                        ("dark chained_only audit", DARK_AUDIT), ("dark chained_only adjudication", DARK_ADJUDICATION),
                        ("pipeline results (input)", dump)):
        L.append(f"| {label} | `{rel(path)}` | `{pv.sha256(path)}` |")
    if DISPOSITIONS.exists():
        L.append(f"| refuted dispositions (`.6b`) | `{rel(DISPOSITIONS)}` | `{pv.sha256(DISPOSITIONS)}` |")
    L.append(f"| parse binary | `{rel(probe)}` | `{pv.sha256(probe) if probe.exists() else 'absent'}` |")
    build_mode = "release" if "/release/" in str(probe) else ("debug" if "/debug/" in str(probe) else "unknown")
    L.append(f"\nMeasured at `HEAD` = `{pv.head_commit()}`; the parse binary is a **{build_mode}** build "
             "(the probe row and this line are provenance, masked by `--check`).\n")

    L.append("## The headline\n")
    total = len(rows)
    L.append(f"- rows re-read: **{total:,}** (manifest rows in the three populations that the lane ran; "
             f"{missing_dump_rows} such rows have no lane row)")
    for pop, desc in POPULATIONS:
        sub = [r for r in rows if r["population"] == pop]
        if not sub:
            continue
        vc: dict[str, int] = {}
        for r in sub:
            vc[r["verdict"]] = vc.get(r["verdict"], 0) + 1
        parts = ", ".join(f"`{v}` {n:,}" for v, n in sorted(vc.items(), key=lambda kv: -kv[1]))
        L.append(f"- **{pop}** — {desc}: {len(sub):,} rows → {parts}")
    L.append("")

    L.append("## explained_svpp — the `.12a` position verdict against the parse that actually happened\n")
    L.append("A `STUCK-ON-SVPP-TOKEN` row was one the raw parse stopped ON the directive; `IN-WINDOW` / "
             "`PAST-LAST-TICK` are `.12b`'s 263, the rows position could not decide. Read down a column: "
             "`corroborated` means the explanation held, `refuted` means it did not — the file still fails "
             "in its own text after preprocessing.\n")
    strata = sorted({r["prior_stratum"] for r in rows if r["population"] == "explained_svpp"})
    verdicts = ["corroborated", "accepted_in_unit", "refuted", "hollow", "unreached_in_unit", "refused_context", "refused_engine"]
    def is_verdict(r, v):
        return r["verdict"] == v or (v == "refuted" and r["verdict"].startswith("refuted:"))
    L.append("| position verdict (`.12a`) | rows | " + " | ".join(f"`{v}`" for v in verdicts) + " |")
    L.append("|---|---:|" + "---:|" * len(verdicts))
    for s in strata:
        sub = [r for r in rows if r["population"] == "explained_svpp" and r["prior_stratum"] == s]
        L.append(f"| `{s}` | {len(sub)} | " + " | ".join(str(count(lambda r, s=s, v=v: r["population"] == "explained_svpp" and r["prior_stratum"] == s and is_verdict(r, v))) for v in verdicts) + " |")
    L.append("")
    L.append("**The refuted rows, by disposition** (`.6b`; `stimuli/sv/characterization/refuted_dispositions.tsv` — "
             "a row keeps `refuted` until someone has looked at it and written down what it is):\n")
    L.append("| disposition | rows | what it means |\n|---|---:|---|")
    undisp = count(lambda r: r["verdict"] == "refuted")
    L.append(f"| *(awaiting a disposition)* | {undisp} | not yet looked at |")
    for kind, meaning in DISPOSITION_KINDS.items():
        n = count(lambda r, kind=kind: r["verdict"] == "refuted:" + kind)
        if n:
            L.append(f"| `{kind}` | {n} | {meaning} |")
    L.append("")
    L.append("**The refuted rows, by the construct on the rejecting line** (the same buckets as the lane report):\n")
    L.append("| construct at the origin line | refuted rows | one example (origin) |\n|---|---:|---|")
    cons: dict[str, list[dict]] = {}
    for r in rows:
        if r["population"] == "explained_svpp" and r["verdict"] == "refuted":
            cons.setdefault(r["construct"], []).append(r)
    L.append("*(the rows still awaiting a disposition)*\n")
    for c, sub in sorted(cons.items(), key=lambda kv: -len(kv[1])):
        L.append(f"| {c} | {len(sub)} | {pv.code(sub[0]['origin'])} |")
    L.append("")
    L.append("**The refuted rows, by suite and raw class:**\n")
    L.append("| suite | raw class | refuted | corroborated |\n|---|---|---:|---:|")
    pairs = sorted({(r["suite"], r["raw_class"]) for r in rows if r["population"] == "explained_svpp"})
    for suite, cls in pairs:
        ref = count(lambda r: r["population"] == "explained_svpp" and r["suite"] == suite and r["raw_class"] == cls and r["verdict"].startswith("refuted"))
        cor = count(lambda r: r["population"] == "explained_svpp" and r["suite"] == suite and r["raw_class"] == cls and r["verdict"] == "corroborated")
        if ref or cor:
            L.append(f"| {suite} | `{cls}` | {ref} | {cor} |")
    L.append("")

    L.append("## chained_only — the `.13c` strata against the parse under the derived context\n")
    L.append("`.13c` split the dark rows by whether an alterable directive can even REACH the failure "
             "(`IN-WINDOW` / `PAST-LAST-TICK` = expansion-owned, `.13d`'s population) or not (`PRE-TICK` / "
             "`NO-ALTERABLE-TICK`, with the cross-file CHANNEL the failure sits on). `ONE-SIDED` rows passed "
             "raw; the pipeline says whether that pass was hollow.\n")
    cverdicts = ["chained:accepted_under_expansion", "chained:accepted_in_unit", "chained:rejects_own_file", "chained:explained_store_gate",
                 "chained:hollow", "chained:unreached_in_unit", "chained:refused_context", "chained:refused_engine"]
    cstrata = sorted({r["prior_stratum"] for r in rows if r["population"] == "chained_only"})
    L.append("| `.13c` stratum | rows | " + " | ".join(f"`{v.split(':')[1]}`" for v in cverdicts) + " |")
    L.append("|---|---:|" + "---:|" * len(cverdicts))
    for s in cstrata:
        sub = [r for r in rows if r["population"] == "chained_only" and r["prior_stratum"] == s]
        L.append(f"| `{s}` | {len(sub)} | " + " | ".join(str(count(lambda r, s=s, v=v: r["population"] == "chained_only" and r["prior_stratum"] == s and r["verdict"] == v)) for v in cverdicts) + " |")
    L.append("")
    owned = [r for r in rows if r["population"] == "chained_only" and r["prior_stratum"] in ("IN-WINDOW", "PAST-LAST-TICK")]
    L.append(f"**The expansion-owned stratum** (`IN-WINDOW` + `PAST-LAST-TICK` with no cross-file channel — "
             f"`.13d`'s pre-registered population): **{len(owned):,}** rows in the audit; under the pipeline "
             + ", ".join(f"`{v.split(':')[1]}` {count(lambda r, v=v: r in owned and r['verdict'] == v)}" for v in cverdicts) + ".\n")
    L.append("**The rows that still reject in their own file after expansion, by construct** (chained_only, "
             "all strata — the barrier BEHIND the macro barrier):\n")
    L.append("| construct at the origin line | rows | one example (origin) |\n|---|---:|---|")
    cons2: dict[str, list[dict]] = {}
    for r in rows:
        if r["population"] == "chained_only" and r["verdict"] == "chained:rejects_own_file":
            cons2.setdefault(r["construct"], []).append(r)
    for c, sub in sorted(cons2.items(), key=lambda kv: -len(kv[1])):
        L.append(f"| {c} | {len(sub)} | {pv.code(sub[0]['origin'])} |")
    L.append("")

    # SVPP-EXPANSION.6d.2 — the unit-membership reading
    L.append("## The unit-membership reading (`.6d.2`)\n")
    L.append("An `` `include ``d class file has no `import` of its own, so per file it rejects on its first cross-file "
             "name; the library pass parsed it INSIDE its package unit (`pipeline_library_members.tsv`, from the "
             "preprocessor's source map). Its verdict is its unit's at that file's origin: `accepted_in_unit` when a unit "
             "including it passes, the per-file verdict kept when the unit's rejection lands in this file, "
             "`unreached_in_unit` when every unit including it rejects elsewhere (the member's bytes were never judged).\n")
    L.append("| suite | members re-read | `accepted_in_unit` | rejection lands here (per-file verdict kept) | `unreached_in_unit` |\n|---|---:|---:|---:|---:|")
    for s in sorted({r["suite"] for r in rows if r["unit"] or (r["suite"], r["relpath"]) in units}):
        mem = [r for r in rows if r["suite"] == s and (r["suite"], r["relpath"]) in units]
        acc = sum(1 for r in mem if r["verdict"].endswith("accepted_in_unit"))
        unr = sum(1 for r in mem if r["verdict"].endswith("unreached_in_unit"))
        kept = sum(1 for r in mem if r["pipeline_status"] in ("fail", "pp_fail") and not r["unit"])
        L.append(f"| {s} | {len(mem)} | {acc} | {kept} | {unr} |")
    if not units:
        L.append("| (no library dump: the lane ran without the library pass) | 0 | 0 | 0 | 0 |")
    L.append("")
    L.append("## must_reject — the rows the answer key expects rejected, under the pipeline\n")
    L.append("Every row whose expected verdict is `must_reject` — the manifest's column, or a pin the raw lane "
             "demoted because its parse stops on a macro use above the pinned construct (`.6b.1`; the pin is read "
             "from the adjudicator's own tables). `match_under_expansion` is the parser refusing text the standard "
             "cannot derive; `accepts_invalid` would be an over-acceptance only the pipeline can see.\n")
    L.append("| prior stratum | rows | `match_under_expansion` | `accepts_invalid` | `hollow` | `refused_context` | `refused_engine` |\n|---|---:|---:|---:|---:|---:|---:|")
    mverdicts = ["must_reject:match_under_expansion", "must_reject:accepts_invalid", "must_reject:hollow",
                 "must_reject:refused_context", "must_reject:refused_engine"]
    for s in sorted({r["prior_stratum"] for r in rows if r["population"] == "must_reject"}):
        sub = [r for r in rows if r["population"] == "must_reject" and r["prior_stratum"] == s]
        L.append(f"| `{s}` | {len(sub)} | " + " | ".join(str(count(lambda r, s=s, v=v: r["population"] == "must_reject" and r["prior_stratum"] == s and r["verdict"] == v)) for v in mverdicts) + " |")
    L.append("")
    L.append("## svpp_owned — the rows routed to the preprocessor lane\n")
    L.append("| verdict | rows |\n|---|---:|")
    sv: dict[str, int] = {}
    for r in rows:
        if r["population"] == "svpp_owned":
            sv[r["verdict"]] = sv.get(r["verdict"], 0) + 1
    for v, n in sorted(sv.items(), key=lambda kv: -kv[1]):
        L.append(f"| `{v}` | {n} |")
    L.append("")
    L.append("## Reading this report\n")
    L.append("- A `refuted` row is a defect CANDIDATE, not a defect: its expansion may be legal SystemVerilog "
             "the parser refuses (a parser defect — the axis-2 bar's business), or the row's expectation may "
             "be wrong (a vendor extension, a fragment). `.6b` dispositions them one by one with the toolbox, "
             "and only then does a row move in the verdict-coverage tuple.")
    L.append("- A `chained:rejects_own_file` row with a `FACT-GATED-CROSS-FILE` stratum is what the `.13c` audit "
             "predicted: no preprocessor supplies a type a sibling file declares. That population is the "
             "library design's (`SVPP-EXPANSION.6`, `--lib-in` / `--lib-out`), not the expansion stage's.")
    L.append("- A `chained:explained_store_gate` row (`SV-CORPUS-GRAD.57b`) is the sound store gate's refusal under the "
             "pipeline: the key expects the file must-accept as part of its unit, no alterable directive precedes the "
             "failure (the expanded text is the file's own), the demanded name is declared nowhere in the row's unit (nowhere "
             "in the suite, or — `.57c` — only in files outside the unit: Surelog, another test's directory), and the "
             "dark worklist's search proved the file parses once that name is declared (the prelude in the `unit` column), "
             "with `.13w`'s soundness conditions holding at the use — ADJUDICATED-explained, never a pass.")
    L.append("- A `hollow` row is judged in the header its rejection lands in; the lane report's origin tables "
             "name those headers.")
    L.append("- The per-row table is `stimuli/sv/characterization/pipeline_readjudication.tsv`.")
    return "\n".join(L) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--pipeline", default=str(CHAR / "pipeline_results.tsv"))
    ap.add_argument("--manifest", default=str(CHAR / "adjudication_manifest.tsv"))
    ap.add_argument("--probe", default=str(ROOT / "rust" / "target" / "release" / "parseability_probe"))
    ap.add_argument("--out", default=str(CHAR / "pipeline_readjudication.md"))
    ap.add_argument("--out-tsv", default=str(CHAR / "pipeline_readjudication.tsv"))
    ap.add_argument("--self-test", action="store_true", help="the .57b store-gate explanation on literal rows")
    ap.add_argument("--check", action="store_true",
                    help="re-render into rust/target/ and diff against the tracked report and table; drift exits 1, "
                         "an absent dump is NOT EVALUATED (0), an empty dump is REFUSED (2)")
    args = ap.parse_args()
    if args.self_test:
        return self_test()

    dump = pathlib.Path(args.pipeline)
    manifest_path = pathlib.Path(args.manifest)
    for needed in (manifest_path, EXPLAINED_AUDIT, DARK_AUDIT, DARK_ADJUDICATION):
        if not needed.exists():
            print(f"PIPELINE-READJUDICATION: REFUSED — tracked input {rel(needed)} is absent", file=sys.stderr)
            return 2
    if args.check:
        if not dump.exists():
            print(f"PIPELINE-READJUDICATION: NOT EVALUATED — {rel(dump)} is absent here (the dump is .gitignored; "
                  "run the lane to re-derive)")
            return 0
        scratch_dir = ROOT / "rust" / "target" / "pipeline_readjudication_check"
        scratch_dir.mkdir(parents=True, exist_ok=True)
        tracked_md, tracked_tsv = pathlib.Path(args.out), pathlib.Path(args.out_tsv)
        args.out = str(scratch_dir / tracked_md.name)
        args.out_tsv = str(scratch_dir / tracked_tsv.name)
    if not dump.exists():
        print(f"PIPELINE-READJUDICATION: REFUSED — {rel(dump)} is absent; run the lane first", file=sys.stderr)
        return 2
    pipeline = pv.read_tsv(dump, 7)
    if not pipeline:
        print(f"PIPELINE-READJUDICATION: REFUSED — {rel(dump)} has no rows (a killed or unfinished lane run?); "
              "nothing rendered", file=sys.stderr)
        return 2

    manifest = {(r[0], r[1]): r for r in pv.read_tsv(manifest_path, 6)[1:]}   # suite relpath observed expected class basis
    explained = read_audit(EXPLAINED_AUDIT)
    dark = read_audit(DARK_AUDIT)
    lane = {(r[0], pv.suite_relpath(r[0], r[2])): r for r in pipeline}

    rows: list[dict[str, str]] = []
    missing = 0
    for key, m in sorted(manifest.items()):
        expected = expected_of(m)
        pop = population_of(m[4], expected)
        if pop is None:
            continue
        p = lane.get(key)
        if p is None:
            missing += 1
            continue
        status, repo_rel, origin = p[1], p[2], p[6]
        landing = landing_of(status, repo_rel, origin)
        line = pv.origin_line_text(origin) if status == "fail" else None
        construct = pv.origin_bucket(line) if line else "-"
        rows.append({
            "suite": key[0], "relpath": key[1], "raw_class": m[4], "population": pop,
            "prior_stratum": prior_stratum(pop, key, explained, dark, m[4]),
            "pipeline_status": status, "landing": landing, "construct": construct,
            "verdict": verdict_of(pop, status, landing, preprocess_only_of(m)), "origin": origin, "unit": "",
        })
    # SVPP-EXPANSION.6d.2 — the unit-membership reading (a member's verdict is its package unit's)
    units = read_units(LIBRARY, LIBRARY_MEMBERS)
    for r in rows:
        mine = units.get((r["suite"], r["relpath"]))
        if not mine:
            continue
        repo_rel = lane[(r["suite"], r["relpath"])][2]
        ur = unit_reading(r, repo_rel, mine)
        if ur:
            r["verdict"], r["unit"] = ur

    # SV-CORPUS-GRAD.57b — the store-gate explanation under the pipeline (the dark worklist's proof, re-read here)
    dark_adj = read_dark_adjudication(DARK_ADJUDICATION)
    for r in rows:
        key = (r["suite"], r["relpath"])
        repo_rel = lane[key][2]
        res = store_gate_explanation(r["verdict"], expected_of(manifest[key]), r["prior_stratum"], dark_adj.get(key), dark.get(key),
                                     lambda p=repo_rel: (ROOT / p).read_bytes().decode("latin-1"))
        if res:
            r["verdict"], r["unit"] = res

    dispositions = read_dispositions(DISPOSITIONS)
    # `.6c.4`: a svpp-owned row this lane judged must_accept and still fails in its own text is `svpp_owned:refuted`
    # — the same disposition table, the same closed vocabulary, the verdict `svpp_owned:refuted:<disposition>`
    refuted_keys = {(r["suite"], r["relpath"]) for r in rows if r["verdict"] in ("refuted", "svpp_owned:refuted")}
    stale = sorted(k for k in dispositions if k not in refuted_keys)
    if stale:
        print("PIPELINE-READJUDICATION: REFUSED — " + rel(DISPOSITIONS) + " dispositions rows that are not "
              "`refuted` under this lane (they moved, or never were): "
              + ", ".join(f"{s}/{r}" for s, r in stale[:8]) + (" …" if len(stale) > 8 else "")
              + " — a disposition for a row that moved is a stale claim; drop it", file=sys.stderr)
        return 2
    for r in rows:
        if r["verdict"] in ("refuted", "svpp_owned:refuted") and (r["suite"], r["relpath"]) in dispositions:
            r["verdict"] = r["verdict"] + ":" + dispositions[(r["suite"], r["relpath"])][0]

    out_md = pathlib.Path(args.out)
    out_tsv = pathlib.Path(args.out_tsv)
    out_md.parent.mkdir(parents=True, exist_ok=True)
    with open(out_tsv, "w", newline="") as fh:
        w = csv.writer(fh, delimiter="\t", lineterminator="\n")
        w.writerow(TSV_COLUMNS)
        for r in rows:
            w.writerow([r[c] for c in TSV_COLUMNS])
    out_md.write_text(render(rows, pathlib.Path(args.probe), dump, manifest_path, missing, units))

    counts: dict[str, int] = {}
    for r in rows:
        counts[r["verdict"]] = counts.get(r["verdict"], 0) + 1
    counts["refuted_awaiting_disposition"] = sum(1 for r in rows if r["verdict"] in ("refuted", "svpp_owned:refuted"))
    summary = " ".join(f"{k}={v}" for k, v in sorted(counts.items()))
    print(f"PIPELINE-READJUDICATION: rows={len(rows)} missing_lane_rows={missing} {summary} → {rel(out_md)}, {rel(out_tsv)}")

    if args.check:
        provenance = ("| parse binary |", "Measured at `HEAD` = ")

        def masked(text: str) -> str:
            return "\n".join(next((pfx for pfx in provenance if line.startswith(pfx)), line) for line in text.splitlines())
        drift = []
        for tracked, fresh in ((tracked_md, out_md), (tracked_tsv, out_tsv)):
            if not tracked.exists():
                drift.append(f"{rel(tracked)} does not exist")
            elif masked(tracked.read_text()) != masked(fresh.read_text()):
                drift.append(f"{rel(tracked)} differs from a fresh render (scratch: {rel(fresh)})")
        if drift:
            print("PIPELINE-READJUDICATION: DRIFT — " + "; ".join(drift) + "; run the lane and re-render", file=sys.stderr)
            return 1
        print("PIPELINE-READJUDICATION: OK — the tracked report and table equal a fresh render of the dumps "
              "(the probe digest row and the HEAD line masked as provenance)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
