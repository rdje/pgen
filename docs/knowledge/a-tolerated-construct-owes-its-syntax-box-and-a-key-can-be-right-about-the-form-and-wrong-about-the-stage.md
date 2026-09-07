---
id: a-tolerated-construct-owes-its-syntax-box-and-a-key-can-be-right-about-the-form-and-wrong-about-the-stage
title: A construct the parser TOLERATES owes the clause its syntax box (tolerance without the box's syntax is over-acceptance, and "not in Annex A" is where such boxes hide), an answer key can be right about the FORM of a defect and wrong about its STAGE, and a corpus census's file list is a text selection whose misses the parser must name
answers:
  - "the parser tolerates `timescale / `pragma / `line lines as opaque text — is `timescale frump an over-acceptance"
  - "which IEEE 1800 syntax boxes are captioned 'not in Annex A' and does the grammar transcribe them"
  - "how are compiler directives typed in the SystemVerilog AST since 1.0.217 — what does body carry"
  - "is the magnitude of a `timescale argument syntax or a value rule — 1, 10, 100 — and why does timeunit 2ps still parse"
  - "sv2v line_char1 / t_pp_line_bad were keyed preprocessing-stage — why are they must_reject now"
  - "a suite tags a directive-argument error ':type: preprocessing' or 'simulation elaboration' — does the tag decide the key"
  - "`begin_keywords \"1800-2023\" — legal under sv_2017 or only under sv_2023"
  - "`begin_keywords \"VAMS-2.3\" — which lane owns a Verilog-AMS file, and does the parser refuse the directive"
  - "a parse stops ON `default_nettype inside a `define continuation line — is that svpp-explained"
  - "a parse stops ON `timescale whose arguments are macro uses — svpp-explained or unexplained"
  - "the census file list selected a file by grep but the parser read no directive node in it — what is no_node"
  - "how do I census every compiler-directive line the corpus writes and classify it against clause 22 (DIRECTIVE-CLASSIFY)"
  - "what is the residue the parser still tolerates as an opaque backtick line, and under which profile is `undefineall opaque"
tags: [systemverilog, grammar, directives, answer-key, adjudicator, census, corpus, lrm, release-ceremony]
date: 2026-09-06
status: current
evidence: "SV-CORPUS-GRAD.3.39 (release 1.0.217, ledger SV-0090, schema 28 -> 29). IEEE 1800-2017 clause 22 gives thirteen directives a syntax box, every one captioned 'not in Annex A' (docs/systemverilog/2017/md/section-22-compiler-directives.md 22.3 :72-86, 22.5.2 :395-407, 22.5.3 :408-412, 22.7 :615-633, 22.8 :681-686, 22.9 :693-716, 22.10 :717-726, 22.11 :734-763, 22.12 :804-822, 22.14 :851-870); through 1.0.216 compiler_directive := trivia /`[^\\r\\n]*/ carried each as one string. The census (AST-FIELD-CENSUS kind=compiler_directive field=body over the 699 lane files carrying a typed-name line; classify.py DIRECTIVE-CLASSIFY) read 1 615 typed lines on sv_2017: legal=1552 ILLEGAL=51 TRAILING=10 opaque=146 macro_body=2 no_node=1, the 51 illegal in 26 accepting files — every one a box refusal — and 0 on verilog_2005. Keys corrected on the FORM/STAGE distinction: sv2v line_char1/line_char2/line_eof/line_level (pinned .8b.2 'directive-argument validity, preprocessing stage') and default_nettype_invalid (SV2V_PREPROC), sv-tests 22.12--line-illegal-1..5 and 22.11--pragma-invalid (':type: preprocessing'), 22.9--unconnected_drive-invalid-1..3 and 22.7--timescale-basic-3 (':type: simulation elaboration'), verilator t_pp_defnettype_bad / t_pp_line_bad / t_pp_pragma_bad (the t_pp_* rule) -> must_reject with the clause; iverilog begin_keywords_1800_2023.v -> must_reject on the 2017 lane as an EDITION CONTRAST (must_accept on the 2023 lane); six Verilog-AMS files -> out_of_scope_with_cause:verilog_ams_lane by a suite-neutral rule (every `begin_keywords in the file names a VAMS version; the mixed keyword-region test t_preproc_kwd.v is not taken). Two adjudicator refinements, each with RED/GREEN arms: a stop on a `define CONTINUATION line is svpp-explained (sv2v test/core/default_nettype.sv), and a stop ON a pass-through directive is explained exactly when an ARGUMENT is a text-macro use (verilator t_time_vpi.v `timescale `time_scale_units / `time_scale_prec). The no_node row: Surelog tests/BeginKeywords/top.v, whose two directives sit inside a block comment."
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/directive_census/classify.py --self-test | tail -1   # misses=0; ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/invalid_directive_timescale_bad_unit.sv --profile sv_2017; echo rc=$?   # rc 1 since 1.0.217; ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_directive_begin_keywords_2023.sv --profile sv_2023; echo rc=$?   # rc 0 (and rc 1 under sv_2017); python3 -c \"import importlib.util as u; s=u.spec_from_file_location('a','stimuli/sv/adjudicate_external_corpus.py'); m=u.module_from_spec(s); s.loader.exec_module(m); m._self_check_arm_split(); print('arms ok')\""
---

**Tolerance is additive — in both directions.** `.3.14a`/`.3.14b` settled WHERE a compiler directive may
appear and carried it as a single regex terminal, so the arguments were never read and
`` `timescale frump `` parsed on every profile. The standing rule "over-acceptance is a defect, tolerance
is additive" says a tolerated name owes exactly the syntax its clause gives it, no more: `.3.39`
transcribes each of the thirteen boxes as a production, and the parser refuses what the box refuses.
The boxes were never in the grammar because every one is captioned *"not in Annex A"* and every
Annex-A-driven slice — and the LRM extractor — reads Annex A only; TOOLBOX §5.8's gap census counts
sixty such boxes in 1800-2017, and clause 22 is the first family closed from that list.

**Form versus stage.** Four suites had keyed these rows as *preprocessing-stage* or *elaboration-stage*
defects — sv2v's `.8b.2` pins ("directive-argument validity, preprocessing stage"), sv-tests'
`:type: preprocessing` and `:type: simulation elaboration` tags, verilator's `t_pp_*` rule. Every one was
RIGHT about the form of the defect (level 3, an unquoted filename, `pull2`) and wrong only about which
stage judges it: the box is the directive's syntax, and the stage that reads it is whichever stage
reads the arguments. Once the parser reads them, a stage tag is tool testimony and the spec outranks it
(`.3.14a`). The re-key is done WITH the clause, so the provenance census reads the rows as clause-cited.

**A value rule is judged as syntax only where the box's prose is its only definition.** `1 | 10 | 100`
for `` `timescale `` (§22.7) is syntax here because no Annex A production exists for the directive;
`timeunit 2ps;` stays derivable because Annex A's `time_literal` derives it. The precision ORDERING is a
value rule at both sites and is not judged — the v2005 lane's `timescale3` ruling, kept. State the
contrast in the grammar comment, or the next reader files one of the two as a defect.

**A census file list is a text selection; the parser's misses must be named.** `grep -lE` selected a
file whose only directives sit inside a block comment; the parser read no node there. The classifier
reports it as `no_node` — counted, never a mover — rather than folding it into `unclassified`, so the
gap between "what the text selected" and "what the parser read" is a printed number.

**When the parser starts reading a slot, the adjudicator's explanations move with it.** A pass-through
directive (`` `timescale ``, `` `default_nettype ``) used to be a token svpp hands back unchanged, so a
parse stopping ON one could not be blamed on the preprocessor. With the arguments read, two shapes
become svpp's after all — an argument that is a text-macro use, and a directive on a `` `define ``
continuation line (macro text) — and both are recognised STRUCTURALLY, each with a control that fails
when the rule is removed, never by a per-file exception.
