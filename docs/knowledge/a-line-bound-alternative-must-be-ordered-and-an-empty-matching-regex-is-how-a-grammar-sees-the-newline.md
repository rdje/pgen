---
id: a-line-bound-alternative-must-be-ordered-and-an-empty-matching-regex-is-how-a-grammar-sees-the-newline
title: Two engine facts a line-oriented construct rests on — the SystemVerilog grammar's DEFAULT branch policy is a longest-match tournament (a line-bound alternative loses to a run-on one unless the rule is @branch_policy ordered), and a regex terminal that CAN match the empty string is offered only horizontal white space by the layout skip, which is the only way a rule can assert end-of-line
answers:
  - "why did `pragma once followed by foo u1(); parse foo as a pragma keyword — the bare-name alternative was first"
  - "what is the SystemVerilog grammar's default branch policy — ordered PEG choice or a tournament"
  - "when do I need @branch_policy: ordered on a rule"
  - "how can a rule in the SV grammar see a newline when the layout skip eats all white space before every terminal"
  - "what does can_match_empty change in consume_layout_for_regex — horizontal skip only"
  - "how is `line's 'only white space may appear on the same line, comments are not allowed' enforced"
  - "is a newline between compiler-directive arguments legal — `timescale 1ns / newline / 1ps"
  - "how do I bound a directive with an optional argument list to its line without a lexer"
  - "a regex ending in (?:\\r?\\n|$) — is it can_match_empty, and does the interpreter agree with codegen"
  - "a scratch grammar shows the end-of-line regex works but the same rule fails in the SV grammar — what differs"
  - "the certificate's probe sample reads `celldefineendmodule or `pragmawj — why does the generator fuse a backtick-led token to the next word"
  - "the certificate reports sample_parse_failures on `timescale 100 fs ///x — why is the / separator gone"
  - "does a @sample annotation change the generated parser's bytes"
tags: [systemverilog, grammar, engine, layout, branch-policy, interpreter, codegen, directives]
date: 2026-09-06
status: current
evidence: "SV-CORPUS-GRAD.3.39 (release 1.0.217, ledger SV-0090, schema 28 -> 29). (1) pragma_directive := kw_dir_pragma pragma_name pragma_line_end -> {...expressions: []} | kw_dir_pragma pragma_name pragma_expression_list -> {...} on 'module m; `pragma once / foo u1(); endmodule' REJECTED at u1 through the interpreter (accepted=false furthest_position=34, rust/target/s339/micro/g.sv) although alternative 1 matched: the grammar's default policy is longest_match (TOOLBOX 5.1 names it; the annotation vocabulary is @branch_policy: ordered | priority_first | longest_match) and alternative 2 consumed one token more (foo as a pragma_keyword). @branch_policy: ordered -> accepted=true furthest_position=46; the tracked pin control_directive_pragma_line_bound.sv holds it with arm compiler_directive>pragma,module_instantiation. (2) consume_layout_for_regex(can_match_empty, pattern) in the generated parser and the interpreter both skip only HORIZONTAL white space when the anchored regex matches the empty string (codegen regex_pattern_layout_facts, interpreter regex_can_match_empty: the same \\A(?:pattern) probe on ''), so line_end := /[ \\t]*(?:\\r?\\n|$)/ sees the newline every other terminal skips; a three-rule scratch grammar through the interpreter read 'a once\\nb' accepted with e=\"\\n\" and 'a once b' rejected at 6 before the SV rules were trusted. The same pattern spelled non-empty-matching (/[ \\t]*\\r?\\n/) is skipped past its own newline and can never match. (3) Every other directive lets its arguments cross a newline — IEEE 1800-2017 5.3 makes a newline white space and only 22.12 states a line rule — so control_directive_args_across_newline.sv flips fail -> pass on 1.0.217."
reverify: "./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --interpret-parse stimuli/sv/adjudication_repros/control_directive_pragma_line_bound.sv --grammar-profile sv_2017 2>&1 | grep -o 'accepted=[a-z]*'   # true; ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --interpret-parse stimuli/sv/adjudication_repros/invalid_directive_line_comment_on_line.sv --grammar-profile sv_2017 2>&1 | grep -o 'accepted=[a-z]* furthest_position=[0-9]*'   # false@13 — line_end refused the comment; grep -n -B1 '^pragma_directive :=' grammars/systemverilog.ebnf | grep -c 'branch_policy: ordered'   # 1"
---

**Fact 1 — ordered choice is not the default here.** A PEG author reads `a | b` as "try `a`, and only
if it fails try `b`". This grammar's default `@branch_policy` is `longest_match`: every alternative is
tried and the one consuming the most input wins, ties to the earlier one. That is invisible while
alternatives are disjoint by their first token (almost every rule in the grammar) and decisive the moment
two alternatives can both match with one of them running on. `` `pragma once `` ⏎ `foo u1();` is the
measured case: the line-bound alternative matched `` `pragma once ``⏎ and the expression alternative
matched one token more. The fix is one annotation, `@branch_policy: ordered`, placed on the rule whose
FIRST alternative must win whenever it matches; the two shipped precedents are `conditional_else_branch`
(the `else if` chain) and `known_unscoped_block_type_identifier`.

**Fact 2 — the layout skip has one hole, and it is the right one.** Before every non-empty-matching
terminal the engine consumes white space INCLUDING newlines and comments (`consume_optional_whitespace`
in `consume_layout_for_regex` / `consume_layout_for_terminal`), so no ordinary terminal can be placed
"at end of line": by the time it runs, the newline is gone. A regex terminal that can match the empty
string is treated differently — only horizontal white space is consumed — because such a terminal is
normally a separator that must be allowed to match nothing. `/[ \t]*(?:\r?\n|$)/` is therefore an
end-of-line ASSERTION: it succeeds on a newline or at end of input and fails on any other byte, and it
is offered exactly the bytes it needs. `can_match_empty` is decided the same way in codegen and in the
interpreter (`\A(?:pattern)` against `""`), which is why the pre-screen's prediction and the release
probe agree.

**Where each was needed.** IEEE 1800-2017 §22.12 binds `` `line `` to its line ("only white space may
appear on the same line … Comments are not allowed"), so `line_directive` ends in `line_end`. §22.11's
`pragma` has an OPTIONAL expression list whose first element is a `simple_identifier` — keywords
included, `` `pragma protect begin `` has `begin` there — so a bare-name pragma followed by an
identifier-led item on the next line is ambiguous by the letter and every tool reads the line; the
bare-name alternative ends in `pragma_line_end` and the rule is ordered. No other directive carries a
line assertion, because no other box states one: `` `timescale 1ns ``⏎`` / 1ps `` is white space
between arguments (§5.3) and parses.

**How to test a layout claim.** Not on the 1 641-rule grammar first. A three-rule scratch grammar
through `--interpret-parse` (TOOLBOX §1.5b) settled fact 2 in one command each way, and the SV rule
that still failed after that pointed at fact 1 — a different mechanism, found by elimination.

**Fact 3 — the generator's boundary tracking has two holes, and a sample can carry its own separator.**
The stimuli generator decides whether two adjacent rendered tokens need a space from the tokens' own
characters, and a token that BEGINS with punctuation is treated as needing no boundary on its right:
`` `abc `` + `def` renders `` `abcdef `` where `abc` + `def` renders `abc def` (three-rule scratch grammar,
`--generate-stimuli`). The literal `/` is likewise not separated from a following `//x` comment sample,
and `///x` re-lexes as a comment. Both were latent in SystemVerilog through 1.0.216 because the opaque
directive token swallowed the fused text; the typed productions exposed them as five union-residual rules
and sample-parse failures at every seed. The declarative fix is `@sample: "`name "` with a trailing space
on each backtick-led token, `@sample: " / "` on the slash separator and `@sample: "`FOO\n"` on the opaque
line (its separator is its newline) (the `line_comment`
`@sample: "//x\n"` precedent); the engine class is `ENGINE-UNIVERSAL-SERVICES.98`. ⛔ A sample annotation
does move the generated parser's bytes — an annotated token is emitted with the annotated-rule
scaffolding — so it is a rebuild, not a stimuli-only edit; the language is unchanged, proven by a sweep
of the probe built without the samples against the one built with them.

