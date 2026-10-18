# Editions and Profiles

Release `1.0.9` reads compiler directives the way the IEEE edition you name reads them. The three editions in force
for SystemVerilog work — IEEE 1364-2005 (Verilog), IEEE 1800-2017 and IEEE 1800-2023 (SystemVerilog) — do not agree
on every directive, and the parser no longer pretends they do.

## The profiles

| profile | edition | spellings it also answers to |
|---|---|---|
| `sv_2017` (**the default**) | IEEE 1800-2017 clause 22 | `2017`, `ieee1800-2017`, `ieee_1800_2017` |
| `sv_2023` | IEEE 1800-2023 clause 22 | `2023`, `ieee1800-2023`, `ieee_1800_2023` |
| `sv_2023_tolerant` | as `sv_2023` (the SystemVerilog family's opt-in tolerant profile; no directive is tolerated beyond 1800-2023) | — |
| `verilog_2005` | IEEE 1364-2005 clause 19 | `1364-2005`, `ieee1364-2005`, `ieee_1364_2005` |

These are the SystemVerilog family's profile names, so the profile you hand the SystemVerilog parser is the one you
hand this one. An unspecified profile reads IEEE 1800-2017, as the expansion stage's unprofiled reading does.

```bash
# the default reading (IEEE 1800-2017)
rust/target/release/parseability_probe --parse systemverilog_preprocessor top.sv
# a named edition
rust/target/release/parseability_probe --parse systemverilog_preprocessor top.sv --profile sv_2023
```

## Where the editions differ

Every row below is a reproducer in the bank (`docs/tasks/artifacts/sv_corpus_grad/svpp_corpus_census/bank/`), read
under each profile it splits:

| the input | `verilog_2005` | `sv_2017` | `sv_2023` | why |
|---|---|---|---|---|
| `` `ifdef (A && !B) … `endif `` | refused | refused | accepted | only 1800-2023's Syntax 22-5 has `( ifdef_macro_expression )` |
| `` `elsif (A \|\| B) `` | refused | refused | accepted | the same production for `` `elsif `` |
| `string s = """a` / `b""";` | refused | refused | accepted | the triple-quoted string is new in 1800-2023 (§5.9) |
| `` `begin_keywords "1800-2023" `` | refused | refused | accepted | each edition's own `version_specifier` list |
| `` `begin_keywords "1800-2005" `` | refused | accepted | accepted | 1364-2005 knows only the four `1364-*` specifiers |
| `` `timescale 1ns `` / `/ 1ps` (two lines) | accepted | accepted | refused | 1800-2023 §22.2: *"compiler directives shall be all on one line"*; the earlier editions have no line rule for a directive's tokens |
| `` `define M(a, `` / ` b) a+b` (two lines) | accepted | accepted | refused | the same rule, applied to a formal list |
| `` `timescale 1ns /* a `` / `b */ / 1ps` (a comment crossing the line) | accepted | accepted | refused | the same rule: a block comment inside a directive carries it onto a second line — between its tokens, in a formal default, in a `` `pragma `` (release `1.0.9`, `SV-CORPUS-GRAD.13c.2v.16`); a comment BEFORE the directive or AFTER its last token (it may run on — iverilog's `br_gh782a.v`), and a comment in macro TEXT, are not inside it |
| `` `define M a /* one `` / `two */ b` | refused | refused | accepted | only 1800-2023 lets a block comment in macro text span lines without a backslash |
| `` `define \M@CRO3 (a=5) a `` | object-like, text `(a=5) a` | object-like | function-like, formal `a` | 1800-2023 puts an escaped name's formals after the single white space that ends the name |
| `` `include <a.v> `` | refused | accepted | accepted | 1364-2005's `` `include `` takes only `"filename"` |
| `` `define M(a=1) a `` | refused | accepted | accepted | a formal DEFAULT is new in 1800 |
| `` `define S(x) `"x`" `` | refused | accepted | accepted | `` `" `` / `` `\`" `` / ``` `` ``` are new in 1800 |
| `` `undefineall `` | a macro usage | the directive | the directive | not a 1364-2005 directive |
| `` `define undefineall 1 `` | accepted | refused | refused | a directive's name is not a macro name — each edition's own directive list |

What every edition shares is not tagged: the `` `pragma ``, `` `default_nettype ``, `` `line `` and `` `undef ``
productions, `` `include ``'s rule that only white space or a comment may share its line, and the licence for a defined
macro to be *"used anywhere"* — inside a directive's parameters included (`` `line `N "f.sv" 0 ``,
`` `timescale `U / `P ``).

## A worked contrast

```systemverilog
`timescale 1ns
         / 1ps
```

Under `sv_2017` this is one `` `timescale `` whose precision sits on the next line — 1800-2017 says nothing that
forbids it, and iverilog's own test `br_gh782b.v` writes a directive one token per line. Under `sv_2023` the same text
is refused at the line end, because 1800-2023 §22.2 added the sentence that puts a directive on one line. Pick the
profile of the edition your sources are written to; the parser reports the edition's answer, not a compromise
between two.
