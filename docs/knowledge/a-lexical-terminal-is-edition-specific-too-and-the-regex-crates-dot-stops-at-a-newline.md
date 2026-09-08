---
id: a-lexical-terminal-is-edition-specific-too-and-the-regex-crates-dot-stops-at-a-newline
title: A lexical terminal is edition-specific too — IEEE 1800 §5.9, 1800-2023 §5.9 and IEEE 1364-2005 §3.6 spell the string literal three ways, the one regex that served all three was wrong for each in a different direction, and the regex crate's `.` stops at a newline so `\\.` is not "backslash plus any character"
answers:
  - "why did PGEN reject a SystemVerilog string literal continued with a backslash at the end of the line"
  - "is a backslash-newline continuation inside a string literal legal in IEEE 1800-2017 5.9 and in IEEE 1364-2005 3.6"
  - "is a raw newline inside a double-quoted string literal legal SystemVerilog or Verilog"
  - "what is a triple-quoted string in IEEE 1800-2023 5.9 and which PGEN profile accepts it"
  - "why is verilator t_display_qqq.v must_reject on the sv_2017 lane and must_accept on the sv_2023 lane"
  - "why is iverilog ivltests/string12.v must_reject on the verilog_2005 lane"
  - "does the Rust regex crate's dot match a newline — what to write for any character in a PGEN terminal"
  - "how is a lexical terminal split by profile in the SystemVerilog grammar — the SELECTION-dispatcher pattern for terminals"
  - "how does the parse-cost ratchet accept the frame a profile dispatcher adds — the leaf-twin arm of introduced_wrapper_frames"
  - "why does the string literal's body keep the backslash and the newline of a continuation"
  - "how was the population of string-literal continuations in the corpus censused before the fix"
tags: [systemverilog, lexical, grammar, profiles, regex, answer-key, parse-cost, sv-corpus-grad, census]
date: 2026-09-08
status: current
evidence: "SV-CORPUS-GRAD.3.47 (PGEN-SV-CORPUS-GRAD-0377, release 1.0.221, ledger SV-0094). Release probe 28948d5cd63c747d… (1.0.220): `$display(\"broken \\` + newline + `line\");` furthest_position=29 on every profile; `$display(\"raw` + newline + `newline\");` parse_full passed on every profile. Interpreter A/B HEAD vs candidate (rust/target/s347/ab_347.tsv): the continuation false -> true on sv_2017 / sv_2023 and unchanged false on verilog_2005, the raw newline true -> false on all three, the triple-quoted forms false -> true on sv_2023 alone, every control ast=SAME. Final bytes: AST-IDENTITY-SWEEP ast_moved=0 on every lane, verdict_moved 5 / 6 / 0 exactly the pre-registered rows; bar 92 -> 86; certificates 1493/10/1468/15 + 1483/0 and 1186/370/802/14. IEEE 1800-2017 §5.9 docs/systemverilog/2017/md/section-32-h-12ab-f001.md:101-102; 1800-2023 §5.9 docs/systemverilog/2023/md/section-5-lexical-conventions.md:436-464; 1364-2005 §3.6 docs/verilog/2005/md/section-3-lexical-conventions.md:299, A.8.8 docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:1218."
reverify: "./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_string_line_continuation.sv --profile sv_2017 2>&1 | grep -q 'parse_full passed' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/invalid_string_line_continuation_v2005.sv --profile verilog_2005 >/dev/null 2>&1; test $? -ne 0 && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/invalid_string_raw_newline.sv --profile sv_2017 >/dev/null 2>&1; test $? -ne 0 && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_string_triple_quoted_sv2023.sv --profile sv_2023 2>&1 | grep -q 'parse_full passed'"
---

**The fact.** `string_literal := trivia /"([^"\\]|\\.)*"/` served every profile of the SystemVerilog grammar since
`1.0.0`, and it was wrong for each standard in a different direction. IEEE 1800-2017 §5.9 and 1800-2023 §5.9 say a
string literal *"shall be contained in a single line unless the newline character is immediately preceded by a \
(backslash); in this case, the backslash and the newline character are ignored"*; 1800-2023's own syntax box adds
`quoted_string_item ::= any_ASCII_character except \ or newline or "` and the triple-quoted string
`""" { triple_quoted_string_item | string_escape_seq } """` whose item is any character except `\`; IEEE 1364-2005 §3.6
says *"contained on a single line"* and A.8.8 spells `string ::= " { Any_ASCII_Characters_except_new_line } "` — no
continuation at all. The regex crate's `.` does not match `\n`, so `\\.` could not consume a backslash-newline and the
continuation REJECTED at the backslash on every profile; `[^"\\]` does match `\n`, so a string spanning lines with NO
backslash PARSED on every profile. Three twins under `@profiles` now say what each standard says: the SV twins admit
`\` + any character (`\\(?:\r\n|[\s\S])`) and refuse a bare newline (`[^"\\\n]`); the `sv_2023` twin adds the
triple-quoted alternative, spelled without look-around (a `"` or a `""` is admitted only when a non-quote follows —
the regex crate has no look-around, and the pre-`-0289` `"""` attempt hung on exactly that); the 1364-2005 twin refuses
both. The body stays the source text verbatim, continuation included: §5.9's *ignored* is the consumer's reading, not a
rewrite of the source, and it is what keeps every previously accepted tree byte-identical.

**The census rule.** A line ending in `\` with an odd count of unescaped `"` before it named 36 continuation lines in 16
lane files — five bar rows and eleven deferred or macro-explained ones — and the pre-screen and the lanes then moved
exactly those. The reverse scan (an unclosed string on a line with no trailing backslash) flagged three accepted rows;
all three were escapes (`"\\"`, `"\""`) once the line was read, so the narrowing moved nothing. Run the token grep BEFORE
the fix, in BOTH directions, and read what it flags: that is the cheapest census in the repository and it is two-sided.

**The instrument rule.** The dispatcher adds one frame per string-literal attempt, and the parse-cost ratchet's
`introduced_wrapper_frames` refused it as *"references nothing — a wrapper wraps something"*: a terminal twin references
only `trivia`. "Could not be evaluated" is the honest reading, and it owes an ARM, not a row: a leaf twin is bounded by the
rules that DISPATCH it, a rule nothing references stays refused, with a HOLDS and a FAILS control
(`ENGINE-UNIVERSAL-SERVICES.99`). The family share moved `2.662 → 2.658 %` because the corpus gained exactly those
frames in its denominator, and the constant is adopted with its anchors, as `.3.42` adopted its own.
