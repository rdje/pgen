---
name: feedback_every_parser_meets_its_full_requirements_no_documented_limits
description: "STANDING DIRECTIVE (director 2026-10-18) — every PGEN parser meets 100 % of its requirements; a gap is FIXED, never re-stated as a documented limit or a narrowed promise ('accepting a premature defeat'). Concrete rulings on the systemverilog_preprocessor grammar: a one-line conditional is accepted, a malformed directive inside a skipped group is skipped (not rejected), and the grammar gets per-edition profiles."
metadata:
  node_type: memory
  type: feedback
---

# Every parser meets its full requirements — no documented limits, no narrowed promises (director, 2026-10-18)

**Status:** standing directive, verbatim, with three concrete rulings for the `systemverilog_preprocessor` grammar.

## Context

Release `1.0.8` of the `systemverilog_preprocessor` parser (`SV-CORPUS-GRAD.13c.2v.2`) took the generated parser's
SystemVerilog-corpus rejections from 4,477 to 64. Reporting the 64, I filed two of them as boundaries of the
line-oriented model and proposed stating one — a malformed directive inside a group the compiler would skip — as a
documented limit in the contract, "because that changes what the parser promises". The routed leaves were
`SV-CORPUS-GRAD.13c.2v.9` (IEEE 1800-2023 directive forms: no edition axis in the grammar) and `.13c.2v.10` (a
conditional written on one line; a malformed directive in an ignored group).

## Decision (the director's words, 2026-10-18)

> (1) for me this "- A whole `ifdef … `endif written on one line is rejected (5 verilator files)." shall be properly
> handled if that shall be accepted.
>
> (2) This "- A malformed directive inside a block the compiler would skip is rejected, even though the standard only
> asks that skipped text be lexically valid (2 files). My proposal is to state this as a documented limit in the
> contract rather than change the grammar. That changes what the parser promises, so it's your call." shall be skip and
> not rejected. for flexibilty gives the grammar per-edition profiles. Everything we do ought to be sota, signoff,
> production-grade, no short-cut, no compromise, no exception, ALl the parser we have shall fully, 100% all their
> requirement, no changing the parsers' promises, doing this would be accepting a premature defeat and that is
> unacceptable.
>
> (3) "- Edition question (leaf .13c.2v.9): the preprocessor grammar has no edition setting. 1800-2023 adds forms such
> as `ifdef ( expr ) and macros inside directive arguments. It needs a decision: give the grammar per-edition profiles,
> or read 2023 as its one edition.", give the grammar per-edition profils

## Consequences

- **Standing rule.** A parser gap is fixed. "State it as a documented limit", "narrow the contract", "a boundary of the
  model" are not dispositions any leaf may propose for a requirement the parser does not meet; a known gap is recorded
  as a GAP, owned by the leaf that fixes it, until it is fixed. This sharpens the existing every-finding-is-fixed and
  strict-LRM records ([[feedback_sv_strict_lrm_compliance_default]]); it does not relax strictness — over-acceptance is
  still a defect.
- **(1) A conditional on one line is accepted** — `` `ifdef TEST_VERBOSE `display_counts("x"); `endif `` (verilator
  `t_clk_2in.v`, `t_func_dotted.v`, `t_interface2.v`, `t_interface_modport.v`, `t_preproc_undefineall.v`). Owned by
  `SV-CORPUS-GRAD.13c.2v.10`, ORDERED for the next `systemverilog_preprocessor` release.
- **(2) A malformed directive inside a group the compiler skips is SKIPPED, not rejected** — IEEE 1800 §22.6: an ignored
  group need only *"follow the SystemVerilog lexical conventions"* (verilator `t_preproc.v:246`, sv2v
  `test/lex/string_macro.sv:42`). The same directive in a LIVE group stays refused end to end. Owned by `.13c.2v.10`,
  ORDERED.
- **(3) The `systemverilog_preprocessor` grammar gets per-edition PROFILES** (the SystemVerilog family's editions), so
  IEEE 1800-2023's directive forms — `` `ifdef ( ifdef_macro_expression ) `` (§22.6), a macro usage inside a directive's
  parameters (§22.2) — are derived under the 2023 profile, and 1800-2017's narrower forms (`` `elsif `` takes a
  `text_macro_identifier`) hold under the 2017 one. Owned by `SV-CORPUS-GRAD.13c.2v.9`, ORDERED.
- Release `1.0.8`'s contract, book and disposition table describe the three as **known gaps ordered fixed**, never as a
  stated limit of the parser's promise.
- ⛔ **Correction (2026-10-18, `PGEN-SV-CORPUS-GRAD-0460`) to an example in (3), which was mine, not the director's:** a
  macro usage inside a directive's parameters is NOT a 1800-2023-only form. Every edition licenses it — a defined macro
  *"can be used anywhere in a source description"* (1364-2005 §19.3.1) / *"anywhere in the compilation unit where it is
  defined"* (1800-2017 and 1800-2023 §22.5.1); 1800-2023 §22.2 only states the directive case. It derives under every
  profile. `` `ifdef ( ifdef_macro_expression ) `` (§22.6) stays the 1800-2023 form the ruling's example names. Recorded
  in `docs/tasks/SV-CORPUS-GRAD.md` leaf `.13c.2v.13`.
