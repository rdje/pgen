# CORPUS-GRAD-ALL.1 — FROZEN graduation rosters v1 + per-family `Done`-claim audit (2026-07-22)

Synthesis of the four exhaustive discovery reports (`regex_corpus_discovery.md`,
`sv_svpp_corpus_discovery.md`, `vhdl_corpus_discovery.md`,
`json_corpus_discovery.md`) + the repo-side vendored-asset audit. Per
[[project_all_parsers_done_requires_corpus_graduation]]: these rosters are
FROZEN v1 — additions re-open this leaf explicitly, never silently. Vendoring
of the ADD tiers is per-family campaign work (`.2`-class leaves); expected
verdicts always derive from each suite's own answer key / the governing spec.

## Roster: regex (LIVE row `Done` — audit: LIKELY GRADUATED, formal statement pending)

| Tier | Suites |
|---|---|
| VENDORED (in force) | pcre2-10.47 full testdata (incl. Perl-compat tests + the auto-generated Unicode-property block) + php-8.4.19 ext/pcre; the 2,189-cell pcre2test oracle corpus with divergences tracked |
| ADD v1 | Perl `t/re/re_tests` + `reg_mesg.t` (compile-error-keyed, dialect-closest); Lingua Franca `uniq-regexes-8.json` (537,806 real-world patterns, MIT — expecteds via the pcre2test oracle ONLY); .NET RegexParserTests (typed parse-error kinds, shared-syntax subset + divergence filter); test262 RegExp early-error + property-escapes/generated (filtered slices); RE2 `parse_test.cc` (AST-keyed subset) |
| N/A-with-cause | UTS#18 (no corpus exists — spec's own statement); OSS-Fuzz corpora (access-restricted, key-free); POSIX/AT&T/Tcl (wrong dialect); Oniguruma/Boost/ICU (intersection already in testinput2) |
| Audit verdict | The canonical + consumer corpora are vendored and continuously gated — the `Done` row is corpus-backed TODAY; the graduation gate formalization + the ADD-tier adjudication are the remaining work, not a status risk. |

## Roster: systemverilog (LIVE row `Mostly Done` — campaign ACTIVE, `SV-CORPUS-GRAD`)

| Tier | Suites |
|---|---|
| VENDORED (in force) | sv-tests `25e4d275` (:should_fail_because: keys) / verible `a0a8d8eb` / slang `4106501b` / verilator `a534a1d1` + VeeR/scr1/friscv + uvm-core-2020.3.1 |
| ADD v1 | ⭐ ispras/sv-tests `ieee-1800-2012/` (~904 LRM-clause-keyed, POSITIVE/NEGATIVE/VARYING); ⭐ ivtest `regress-sv.list` (922, CE/EF/gold keys; vendor from steveicarus/iverilog); sv2v `test/` (paired golden .v + triaged error/); Surelog `tests/` (accept/error key extraction); OpenTitan + black-parrot (real-design breadth + macro stress) |
| Differential-only (never vendored) | gmlarumbe tree-sitter-systemverilog-test (unlicensed; golden trees usable read-only for divergence hunting) |
| Rejects | yosys-tests, basejump_stl standalone, asic-world lineage, aggregates (dedupe by true upstream) — full causes in the discovery extract |

## Roster: verilog_2005 (SV profile)

| Tier | Suites |
|---|---|
| ADD v1 | ⭐ ivtest `regress-vlg.list` (the natural keyed 1364-2005 suite, CE negatives incl.); ispras/sv-tests `ieee-1364-2005/` (362 clause-keyed; VARYING = profile-boundary probes); sv2v golden `.v` (287, free) |
| CONDITIONAL | OSS-CVC tests (license read first); ispras/hdl-benchmarks (bulk volume only) |

## Roster: systemverilog_preprocessor (LIVE row `Done` — audit: mapping now PINNED)

| Tier | Suites |
|---|---|
| VENDORED (in force) | verilator `t_preproc*` goldens (the Snyder lineage) + sv-tests preprocessor cases + the UVM macro surface (uvm-core, exercised via the 14/14 chained corpus) |
| ADD v1 | hdlConvertor `tests/sv_pp/` (LRM preprocessor-chapter examples, golden expansions, MIT); verilog-perl `t/30_preproc*`+`t/80_vppreproc*` (reference-implementation goldens, Artistic-2.0); Surelog PreProc*/Macro* dirs (with the Surelog add) |
| Audit verdict | `Done` row keeps corpus backing via the vendored verilator/sv-tests preprocessor cases; the ADD tier + a dedicated graduation slice makes it explicit. |

## Roster: vhdl (LIVE row `Done` — audit: AT RISK, campaign = `.2`)

| Tier | Suites |
|---|---|
| VENDORED (in force) | ghdl testsuite (VESTS 4,317 — the de-facto maintained copy, 2 files AHEAD of the nickg mirror — + gna 3,130) / nvc / OsvvmLibraries / UVVM / VUnit + PoC / Compliance-Tests / Interfaces / neorv32 / Rudi-RV32I |
| ADD v1 | GRLIB (TUT-ASI mirror, 1,168 — idiom-dense industrial); slaclab/surf (1,155, modern 2008); vhdl-style-guide `tests/` (2,692 per-construct + paired oracle); ⭐ IEEE VASG Packages (65, Apache-2.0 — the normative 2019 must-parse floor); microwatt (112); CERN general-cores (269); open-logic (211); vhdl-linter `test/` (505 — the ONLY new negative-case corpus; needs a spec→manifest mapping); vhdl-ideas (44, 2019 stress); I99T (22, ITC'99) |
| N/A-with-cause | VESTS upstream (dead; ghdl's copy is canonical); IEEE LRM-example extraction (does not exist — IEEE copyright; VASG Packages is the legal analogue); Sigasi (no public corpus); Xilinx/Intel sim libs (redistribution-restricted) |
| ⚠️ Answer-key reality | Only VESTS compliant/non_compliant, gna/nvc regressions, and vhdl-linter carry expected-fail keys — every other suite is positive-only; VHDL reject-side coverage rests on those + mutations. |
| Audit verdict | **`Done` NOT corpus-backed**: 29.4% characterized; VESTS answer-key adjudication (~2,042 compliant-but-rejected) is the campaign baseline. Row re-adjudication belongs to the VHDL campaign's first slice — honest interim: the row carries the tracker-note flag (already placed). |

## Roster: json (no family row; gated on parked `JSON-RFC8259`)

| Tier | Suites |
|---|---|
| VENDORED (in force) | JSONTestSuite `test_parsing` (318, MIT) |
| ADD v1 | ⭐ JSONTestSuite `test_transform` (22 — the missing half, same upstream/license); nativejson-benchmark pack (roundtrip 27 + parse-double 66 + parse-string 9 + RFC-corrected JSON_checker curation, MIT); JSON5 suite AS REJECT ORACLE (88 must-reject + 26 must-accept, MIT); RFC 8259 §13 in-spec examples; jansson suites (esp. invalid-unicode 19) |
| License trap (recorded) | raw json.org JSON_checker = the non-free "Good, not Evil" JSON License + 2 RFC-wrong cases — adopt ONLY via the nativejson-benchmark curation |
| Rejects | JSON Schema Test Suite (schema semantics — wrong layer); fuzz corpora (key-free); the duplicate/aggregate carriers |

## N/A-with-cause adjudications (internal DSLs — director may override)

| Family | Adjudication |
|---|---|
| return_annotation / semantic_annotation | PGEN-defined DSLs — no external standards body or recognized corpus can exist. Compensating evidence: cert-coverage fully_certified + the annotation contract gates + closed-loop stimuli proofs. N/A-with-cause. |
| ebnf (meta-grammar) | PGEN's own dialect (not ISO-14977/W3C EBNF) — external suites would test a different language. Compensating: self-hosting duality gate + fully_certified + the every-grammar corpus (all tracked .ebnf files ARE its real-world corpus). N/A-with-cause. |
| rtl_const_expr / rtl_frontend | PGEN-internal SV-subset bootstrap grammars for RTLSyn — no external corpus targets these subsets. Partial mapping possible (SV-subset slices of the SV roster filtered to the supported subset) — adjudicate inside the Phase-S lane if pursued. N/A-with-cause v1. |
| scratch / builtin_* | Tooling/bootstrap surfaces, not deliverable parsers. N/A. |

## Sequencing (per the Nexsim delivery directive)

> Correction (same session, post-synthesis): the "known burn-down member
> `H.12.5.8`" referenced in the discovery extracts is ALREADY FIXED (releases
> 1.0.148/1.0.149, 2026-06-25; 12/12 matrix re-verified at HEAD — see
> `../sv_replay_debt/h1258_matrix_at_head.txt`); the SV burn-down baseline is
> the `.2` adjudication manifest, not that bug.

1. SV campaign (`SV-CORPUS-GRAD.1` re-characterization + adjudication) — ACTIVE next.
2. VHDL campaign opening (`CORPUS-GRAD-ALL.2`) — the material `Done`-at-risk row + Nexsim's second language.
3. regex formal graduation statement (small); svpp explicit slice (small).
4. JSON: waits on the director's `JSON-RFC8259` GO (roster ready).
