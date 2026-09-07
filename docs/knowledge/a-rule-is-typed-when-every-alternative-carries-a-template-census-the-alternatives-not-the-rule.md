---
id: a-rule-is-typed-when-every-alternative-carries-a-template-census-the-alternatives-not-the-rule
title: A grammar rule is typed only when EVERY alternative carries a return template — every register that counts annotations (the book row, the inventory, the shape contract) counts a RULE, while the consumer meets an ALTERNATIVE; census the alternatives, select a discriminator-less slot by its key set, and pre-register the files that must move AND the files that must not
answers:
  - "why did function_body_declaration read as typed in the book while the ported function body was a positional array"
  - "how much of the SystemVerilog corpus's function bodies were raw envelopes before release 1.0.218"
  - "how do I find every rule whose return templates cover only some of its alternatives — the partial-typing census"
  - "what does PARTIAL-TYPING-CENSUS report and what is a passthrough versus a RAW_ENVELOPE alternative"
  - "how do I census a typed-AST slot whose node has no kind field — ast_field_census --select-keys"
  - "how does a consumer tell a ported function body from a non-ported one in the SV AST — ports and scope key presence"
  - "why is key presence rather than a kind discriminator used on function_body_declaration"
  - "what does a two-sided prediction for an AST identity sweep look like — movers and stayers"
  - "how were the 66 moved pin arms of the .3.44 pre-screen shown to be exactly the raw-body arms"
  - "which rules still carry raw-envelope alternatives after 1.0.218 — the .3.45 residue"
tags: [systemverilog, grammar, ast-shape, annotation-campaign, census, instrument, sv-corpus-grad]
date: 2026-09-07
status: current
evidence: "SV-CORPUS-GRAD.3.44 (release 1.0.218, ledger SV-0091, schema 29 -> 30). (1) The AST-field census with the new --select-keys lifetime,body selector over the 4 811 SV-lane files carrying the function / task token, on the 1.0.217 release probe: AST-FIELD-CENSUS: keys=body,lifetime field=body profile=sv_2017 files=4811 accepted=1958 rejected=2853 timeout=0 nodump=0 nodes=3411 distinct=2968 fallback_rows=0; TF-BODY-CLASSIFY: ... typed=1200 RAW=2211 unclassified=0 (function:alt2:RAW 1646, task:alt2:RAW 513, function:alt4:RAW 28, task:alt4:RAW 22, function:alt3:RAW 2) in 1 060 of 1 958 accepting files; sv_2023 3521/1300/2221 in 1 064 files; the verilog_2005 lane 470/379/91 in 40 files. (2) The class: PARTIAL-TYPING-CENSUS over the frontend's raw_ast of 1.0.217 — rules=1610 multi_alternative=558 fully_typed=490 untyped=28 PARTIAL=40 partial_untyped_alternatives=70 passthrough=45 RAW_ENVELOPE=25; after the six templates PARTIAL=38 ... RAW_ENVELOPE=19, the set difference exactly function_body_declaration and task_body_declaration. (3) Two-sided pre-registration: PRESCREEN-PINS: jobs=761 verdict_moved=0 ast_moved=66 and, classifying every accepting arm's HEAD AST, 66 arms carry a raw body and all 66 moved, 16 arms carry only typed bodies and none moved; over the corpus `ast_moved=1060 verdict_moved=0` on `sv_2017` (8696 of 9756 accepted trees byte-identical), `ast_moved=1064 verdict_moved=0` on `sv_2023`, `ast_moved=40 verdict_moved=0` on the `verilog_2005` lane — exactly the census's predicted files (`sv_2017`: predicted 1060, moved 1060, predicted-but-identical 0, moved-but-not-predicted 0; `sv_2023`: predicted 1064, moved 1064, predicted-but-identical 0, moved-but-not-predicted 0; `verilog_2005`: predicted 40, moved 40, predicted-but-identical 0, moved-but-not-predicted 0) against the census's 1 060 / 1 064 / 40 named files. (4) Alternative 1 byte-identical: the two in-block pins on all three profiles under the interpreter (HEAD vs candidate) and every all-typed corpus file in the sweep. Record: docs/tasks/SV-CORPUS-GRAD.md .3.44; docs/tasks/artifacts/sv_corpus_grad/tf_body_census/census.md."
reverify: "./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --emit-raw-ast-json rust/target/km_raw.json > /dev/null 2>&1 && python3 docs/tasks/artifacts/sv_corpus_grad/tf_body_census/partial_typing_census.py rust/target/km_raw.json | grep -o 'PARTIAL=[0-9]* [^ ]* passthrough=[0-9]* RAW_ENVELOPE=[0-9]*'   # PARTIAL=38 ... RAW_ENVELOPE=19 until .3.45 lands, then fewer; python3 docs/tasks/artifacts/sv_corpus_grad/tf_body_census/classify.py --self-test | head -c 40   # TF-BODY-CLASSIFY SELF-TEST: OK; grep -c 'ports: \\$' grammars/systemverilog.ebnf   # >= 4 (the four ported templates)"
---

**The fact.** `function_body_declaration` carried a return template on its first alternative from
SV-Slice-25 (release `1.0.25`), and from that day the SV parser book's json-carrier row showed the
template, the annotation inventory counted it, and the AST-shape contract listed the rule as typed —
every one of those registers counts a RULE. The consumer meets an ALTERNATIVE. IEEE 1800-2017 A.2.6 gives
a function body two forms, the non-ported `type f; …` (ports declared inside the body) and the PORTED
`type f(ports); …`, and the ported one is the most common function in any corpus: on the 1.0.217 probe
2 211 of the 3 411 function / task bodies the parser wrote over the SV corpus were the raw positional
`Sequence` envelope — 1 646 ported functions, 513 ported tasks, 52 out-of-block methods — in 1 060 of the
1 958 accepting files that declare a function or task at all. Nothing was wrong with any register; each
answered the question it was built for, and none of them asked "does EVERY alternative return a shape?"

**The instrument that asks it.** `partial_typing_census.py` reads the EBNF frontend's own `raw_ast`
envelope — the reading the code generator consumes — and, per rule with two or more top-level
alternatives, which of them end in a return template. A rule typed on some alternatives and not others
is `PARTIAL`; each of its untyped alternatives is bucketed by what it EMITS: `passthrough` when it is
exactly one rule reference (the child's node passes through unwrapped — the profile-split `_sv_only`
helper pattern, typed one level down by design), `RAW_ENVELOPE` when it holds two or more elements,
because that alternative emits a positional array. On 1.0.217 the grammar carried 40 partial rules with
25 raw-envelope alternatives; the two body rules were six of them, and the residue — 19 alternatives in 8
rules (`assignment_pattern`, `block_data_type`, `case_statement`'s `matches` / `inside` forms,
`clocking_declaration`, `clocking_event_sv_2017`, `list_of_path_delay_expressions`,
`net_type_declaration_sv_2017`, `property_case_item`) — is owned by `SV-CORPUS-GRAD.3.45`.

**Reading a slot that has no discriminator.** The census that measured the corpus could not have run
through `ast_field_census.py --kind`: the node `function_declaration_sv_2017` returns is `{lifetime,
body}` with no `kind`. `--select-keys lifetime,body` selects every object whose key set is exactly
that, and `--select-keys dynamic_override,lifetime,body` its `sv_2023` twin. Any slot the campaign typed
without a `kind` is censusable the same way.

**Two-sided pre-registration.** A sweep's `ast_moved=N` is a number. The census names each file that
must move (one raw body or more) and each that must not (all bodies typed), so the release is read
against both lists; and the pin pre-screen's `ast_moved=66` became a measurement only when the 66
arms carrying a raw body on HEAD were shown to be exactly the 66 movers and the 16 typed-only arms
exactly the stayers. A one-sided prediction cannot distinguish a template that moved the right files
from one that moved the right files and a few more.

**Why key presence and not a `kind`.** The two A.2.6 forms differ by where the port list sits, and
`items` differs with them (`tf_item_declaration` wrappers on the non-ported form, bare
`block_item_declaration`s on the ported one — the standard's own two productions). `ports` present
exactly on the ported alternatives and `scope` exactly on the out-of-block ones carries that
distinction, and alternative 1's shipped shape stays byte-identical; a `kind` on every alternative would
have moved every function tree in the corpus for a distinction the keys already make.
