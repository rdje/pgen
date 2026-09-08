---
id: a-lexical-adjacency-rule-needs-a-glued-terminal
title: A lexical adjacency rule needs a glued terminal — a whitespace-insensitive grammar cannot say "no layout between these two tokens" without the rule-level @glued directive, and the two workarounds each cost something a consumer sees
answers:
  - "why did PGEN reject #1 s = 0 and trans ##1 s before release 1.0.226 — a time unit after white space read as a time literal"
  - "is 1 ns with a space a SystemVerilog time literal — IEEE 1800-2017 Annex A footnote 44"
  - "what does the @glued directive do in a PGEN grammar and where is it read"
  - "why could the SystemVerilog grammar not spell the time-literal adjacency rule with regexes alone — layout skip, capture groups, lookaround"
  - "why does the `timescale directive still accept 1 ns / 1 ps with spaces"
  - "can a @glued rule contain a string literal terminal — no, generation refuses"
tags: [systemverilog, grammar, engine, layout, lexical, annotations, time-literal, sv-corpus-grad, engine-universal-services]
date: 2026-09-08
status: current
evidence: "SV-CORPUS-GRAD.3.55 (PGEN-SV-CORPUS-GRAD-0385, release 1.0.226, ledger SV-0099) and ENGINE-UNIVERSAL-SERVICES.103. On the 1.0.225 probe: `module m; logic s; initial #1 s = 0; endmodule` furthest_position=33; `trans ##1 s;` with `sequence s` furthest_position=133 while `trans ##1 t;`, `trans ##1 (s);`, `trans ##1 s_x;` parse; `trans ##1 ms;` @161, `trans ##1 ps ##1 ns;` @165. Removing the `trivia` prefix from the unit terminals on a candidate copy changed nothing in the interpreter (`#1 s = 0` @33 both sides). A string literal's AST body is `\"\\\"hi\\\"\"` — a regex atom's `$n` is its whole match. Corpus by token: 5 failing lane files carry `<number> <white space> <unit>` after `#` or `##`, 0 accepted; 240 accepted files carry glued time literals."
reverify: "./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/control_delay_then_variable_named_like_a_unit.sv --profile sv_2017 2>&1 | grep -q 'parse_full passed' && ./rust/target/release/parseability_probe --parse systemverilog stimuli/sv/adjudication_repros/invalid_spaced_time_literal_in_timeprecision.sv --profile sv_2017 2>&1 | grep -q 'furthest_position='"
---

**The fact.** IEEE 1800-2017 A.8.7 `time_literal ::= unsigned_number time_unit | fixed_point_number time_unit`,
and Annex A footnote 44: "The unsigned number or fixed-point number in time_literal shall not be followed by
white_space" (`docs/systemverilog/2017/md/section-41-data-read-api.md:3149-3153`, `:3567`; 1800-2023 footnote
49). So `1ns` is a time literal and `1 ns` is the number `1` followed by the identifier `ns` — which is why
`#1 s = 0;` assigns the variable `s` after a delay of one, and `trans ##1 s` is a cycle delay followed by the
sequence `s` (§16.8 and §16.9.11 write exactly that). The `` `timescale `` directive is different text under
different rules: §22.7 allows `` `timescale 1 ns / 1 ps ``.

**The defect.** Through release `1.0.225` the SystemVerilog grammar spelled `time_literal := number time_unit`
with each unit a terminal (`kw_s_a0f1490a := trivia /s\b/` …), and the engine skips layout before every terminal
it tries, so `1 s` matched the literal and won the `constant_primary` tournament over the bare number (three
bytes against one). Every identifier spelled like a unit — `s`, `ms`, `us`, `ns`, `ps`, `fs` — vanished after a
delay. Bisecting the NAME found it: `trans ##1 t` passed where `trans ##1 s` failed with identical declarations.

**Why no grammar spelling could fix it.** Three things were measured before an engine change was chosen.
Removing the `trivia` prefix from the unit terminals changed nothing: the skip is `consume_layout_for_regex`,
engine-level, in front of every regex atom. A return template's `$n` on a regex atom is the WHOLE match (a
string literal's body keeps its quotes), so folding number and unit into `/([0-9]+)ns\b/` cannot keep
`value.body = "10"` — and 240 accepted corpus files carry glued time literals whose typed node
`{value: {kind, body}, unit: {kind}}` would have moved. The `regex` crate has no lookaround, so the number
cannot assert a glued unit without consuming it.

**The fix shape.** The rule-level `@glued: true` directive (`ENGINE-UNIVERSAL-SERVICES.103`): the rule's regex
terminals accept no leading layout, neither white space nor a comment. It is read in the ONE shared layout
decision (`regex_atom_skips_leading_layout`) that every codegen path and the parse-harness interpreter query, so
the generated parser and the interpreter cannot drift; a glued rule that spells a literal terminal is refused at
generation, because `match_string` carries no per-site decision and the promise could not be kept. The grammar
then says `time_literal := number time_unit_glued` with `@glued: true` on `time_unit_glued := /s\b/ | /ms\b/ |
/us\b/ | /ns\b/ | /ps\b/ | /fs\b/`, each alternative carrying the same `{kind}` template — every time literal's
node is byte-identical, `1 ns` is no longer a literal, and `timeprecision 1 ps;` rejects as footnote 44 says.

**The reading rule.** A lexical adjacency rule is a third kind of constraint beside syntax and semantics. A
whitespace-insensitive grammar needs a terminal that owns its adjacency to state it; the two workarounds — an
opaque regex, or reading `1 s` as `1s` — each cost something a consumer sees.
