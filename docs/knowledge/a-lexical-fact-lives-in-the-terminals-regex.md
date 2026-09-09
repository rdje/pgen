---
id: a-lexical-fact-lives-in-the-terminals-regex
title: A lexical fact lives in the terminal's regex — a directive line that ends at the first newline, or a literal that admits only spaces and tabs between its tokens, is a claim about the language's lexis, and clauses 5 and 22 are its source
answers:
  - "why did PGEN reject a multi-line `` `define `` with backslash continuation lines before release 1.0.227"
  - "how does IEEE 1800-2017 22.5.1 continue macro text over lines — the newline preceded by a backslash"
  - "may a based literal's size, base and digits be separated by newlines or comments — 5.7.1 and 5.3"
  - "is 5 ' b 01010 legal — no, the apostrophe and the base format character shall not be separated"
  - "what does opaque_directive_line match in the SystemVerilog grammar"
  - "why did a census of comments inside numbers find English apostrophes — blank the comments first"
tags: [systemverilog, grammar, lexical, directives, numbers, annex-a, sv-corpus-grad]
date: 2026-09-09
status: current
evidence: "SV-CORPUS-GRAD.3.56 (PGEN-SV-CORPUS-GRAD-0386, release 1.0.227, ledger SV-0100). On the 1.0.226 probe: a `` `define `` whose text continues over four backslash-newlines furthest_position=29 (the first continuation line) while the one-line macro parses; `5 'b 01010` parses, `5 <newline> 'b <newline> 01010` furthest_position=40, with block comments 67, with a line comment 60, and `5 ' b 01010` 40 (correctly). Corpus by token: a backslash-continued `` `define `` in 500 failing SV-lane files and 3 accepted; a based literal with a newline or comment between its tokens in 1 failing file and 0 accepted once comments were blanked — the unblanked count was fourteen English apostrophes."
reverify: "./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_define_continuation_lines.sv --profile sv_2017 2>&1 | grep -q 'parse_full passed' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/invalid_based_literal_space_after_apostrophe.sv --profile sv_2017 2>&1 | grep -q 'furthest_position='"
---

**The facts.** IEEE 1800-2017 §22.5.1: "If more than one line is necessary to specify the text, the newline
character shall be preceded by a backslash. The first newline character not preceded by a backslash shall end
the macro text" (`docs/systemverilog/2017/md/section-22-compiler-directives.md:171-174`). §5.7.1: a based
literal "shall be composed of up to three tokens — an optional size constant, an apostrophe character followed by
a base format character, and the digits"; "the unsigned number token shall immediately follow the base format,
optionally preceded by white space"; "the apostrophe character and the base format character shall not be
separated by any white space" (`docs/systemverilog/2017/md/section-1024-characters-if-an-identifier-exceeds-the-implementation-specific-length-limit-an-error-shall-be-reported.md:191-210`).
§5.3: "White space shall contain the characters for spaces, tabs, newlines, and formfeeds"
(`docs/systemverilog/2017/md/section-5-lexical-conventions.md:52`) — and a comment separates tokens here as it
does everywhere in the language. IEEE 1364-2005 says the same for the `verilog_2005` lane: §19.3.1
`docs/verilog/2005/md/section-19-compiler-directives.md:107-108`, §3.2 `docs/verilog/2005/md/section-3-lexical-conventions.md:43`,
§3.5.1 `docs/verilog/2005/md/section-3-lexical-conventions.md:155-157`.

**The defects.** Through release `1.0.226` the SystemVerilog grammar's opaque directive line was
`` opaque_directive_line := trivia /`[^\r\n]*/ `` — it ended at the first newline, so a multi-line `` `define ``
kept its first line and left its continuation lines to be parsed as module items, where they failed; 500
failing lane files carry that shape. And `integral_number` admitted `[ \t]*` between its tokens, so
`5 <newline> 'b <newline> 01010` — a newline is white space — rejected at the newline after the size.

**The fix shape.** Both are regex-only. `` opaque_directive_line := trivia /`(?:[^\\\r\n]|\\\r?\n|\\[^\r\n])*/ `` — a
backslash before a newline continues the line, a backslash before any other character is literal, and the
directive node's `text` now carries the whole macro. `integral_number`'s two inter-token gaps admit `\s`, block
comments and line comments (the grammar's `\/` escaping for the slashes); the apostrophe and the base stay
adjacent, so `5 ' b 01010` still rejects. No rule was added; every accepted tree is byte-identical except the
three accepted corpus files whose multi-line define bodies fold into the directive's opaque text.

**The reading rules.** A lexical fact lives in the terminal's regex: a line that ends at the first newline, or a
literal that admits two white-space characters, is a claim about the language's lexis, and the standard's
clauses 5 and 22 are the source. And census by token with comments BLANKED, or the count is English — the first
count of "a comment inside a number" was fourteen `typedef'd` and `ACK'd` in comments.

**A widening exposes a latent over-acceptance.** Admitting white space inside a based literal let the recognized-union certificate's witness generator compose a based-literal *time* literal — `timeprecision'D 9F? fs` — whose interior layout the glued time unit (`@glued`, SV-CORPUS-GRAD.3.55) must refuse, so those samples failed to parse (non-deterministically, since the generator only sometimes composes one). The tell was correct: IEEE 1800-2017 A.8.7 gives a time literal's magnitude as `unsigned_number` or a fixed-point `real_number`, never a based literal, and `.3.55`'s `time_literal := number time_unit_glued` had reached the wider `number` all along. `1.0.227` restricts it — `real_number time_unit_glued | unsigned_number time_unit_glued` — dropping the based-literal magnitude (0 corpus rows, the value node byte-identical for every real time literal). A grammar too loose for the language is invisible until a tool that reads the grammar — here a witness generator — walks the slack.
