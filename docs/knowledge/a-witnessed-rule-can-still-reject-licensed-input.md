---
id: a-witnessed-rule-can-still-reject-licensed-input
title: A WITNESSED rule can still reject inputs its grammar licenses — `UNKNOWN=0` answers "can this rule fire?", not "is the parser right about the language?"
answers:
  - "does UNKNOWN=0 mean the parser accepts everything the grammar licenses"
  - "a rule is witnessed but a hand-written input for it is rejected"
  - "what class of defect does certificate coverage not catch"
  - "two rules share a token at different depths and one swallows the other's input"
  - "why does --lint-grammar report ordered_choice_shadowing=0 when two readings compete"
  - "a map entry parses with a string key but not a numeric one"
  - "how do I find the divergence between an accepted and a rejected input"
  - "how do I diagnose a defect from two 4000-line parser traces"
tags: [certificate-coverage, grammar-wellformedness, ambiguity, peg, linter, instruments, tracing]
date: 2026-08-22
status: current
evidence: GRAMMAR-WELLFORMED.H.16.6 (`PGEN-GRAMMAR-WELLFORMED-0168`). `grammars/semantic_annotation.ebnf` spells `=>` twice — `map_entry := annotation_value /\s*/ "=>" /\s*/ annotation_value` (:290) and `implication_expr := logical_or_expr (/\s*/ "=>" /\s*/ logical_or_expr)?` (:332), reachable from `annotation_value → expression_value → logical_expression`. `map_entry` is WITNESSED by a string-keyed sample; `{1 => 2}` is rejected because the KEY consumes the arrow. `--lint-grammar` reads `ordered_choice_shadowing=0, exit 0`.
reverify: "printf '@type: {1 => 2}' > /tmp/a && printf '@type: {1 => \"b\"}' > /tmp/b && ./rust/target/debug/ast_pipeline grammars/semantic_annotation.ebnf --interpret-parse /tmp/a; ./rust/target/debug/ast_pipeline grammars/semantic_annotation.ebnf --interpret-parse /tmp/b   # rejected / accepted"
---

> ⛔⛔ **CORRECTION — `GRAMMAR-WELLFORMED.H.16.6a` (`PGEN-GRAMMAR-WELLFORMED-0171`, 2026-08-22).**
> The card's **headline claim stands** and is the reason it exists: a witnessed rule can still reject
> inputs its grammar licenses. **Three supporting details below are superseded**, and the body is left
> standing rather than rewritten so the correction is visible:
>
> 1. **`=>` is spelled FOUR times in THREE roles, not twice.** `map_entry:290`,
>    `implication_expr:332`, `lambda_expression:387`+`:388`, `function_type:426`.
> 2. **The biconditional is too narrow.** Not *"iff not a valid `implication_expr`"* but **iff not a
>    valid `annotation_value`**. Counterexample the tables below never tested: `{[a] => b}` is
>    rejected, and `[a] => b` is a valid `lambda_expression`, **not** an `implication_expr`. An
>    ARROW-CENSUS over all 113 rules × 9 probes proves the partition exactly —
>    `implication ∪ lambda ∪ function_type` = `annotation_value` = `{0,1,2,3,5,6,7,8}`, the exact
>    complement of `map_entry`'s `{4}`.
> 3. **"None is WIDEN-only" is REFUTED by measurement.** Six arms scored over 1 211 inputs: the
>    map-key arms read `widen=9 narrow=0` and `widen=25 narrow=0`. The reason is structural — PEG's
>    greedy key could never COMPLETE a `map_entry` whose key ate the arrow, so the key readings those
>    arms remove were already unreachable. ⭐ **"This edit narrows the grammar" and "this edit narrows
>    the accepted language" are different claims, and only the second is about behaviour.**
>
> ⛔ And the closing method note below — *"Kill your own findings with a control"*, illustrated with
> the `value: ""` example — is **half wrong in its example**: that control was sound (it is not a `=>`
> defect) but the symptom IS a defect, root-caused in `H.16.7` to `value: $6` pointing at a `/\s*/`
> separator. See [[a-control-that-clears-your-hypothesis-has-not-cleared-the-symptom]].

**Certificate coverage answers *"can this rule fire at all?"*. It does not answer *"does the parser
accept everything the grammar licenses?"*** Those are different questions, and a family sitting at
`UNKNOWN=0` can still be wrong about its own language.

`semantic_annotation`'s `map_entry` is **witnessed** — a string-keyed sample reaches it and commits, so
it has never been `UNKNOWN` and never will be. And this does not parse:

| inside a map | verdict | | standalone | verdict |
|---|---|---|---|---|
| `@type: {"a" => "b"}` | accepted | | `@type: 1 => 2` | **accepted** |
| `@type: {1 => "b"}` | accepted | | `@type: "a" => 1` | rejected |
| `@type: {"a" => 1}` | accepted | | `@type: 1 => "b"` | rejected |
| `@type: {1 => 2}` | **REJECTED** | | | |

The two tables are **complements**, which turns an oddity into a characterization: *a map entry parses
**iff** its `key => value` is not a valid `implication_expr`.*

## The mechanism: one token, two rules, different depths

```ebnf
map_entry        := annotation_value /\s*/ "=>" /\s*/ annotation_value     # the map arrow
implication_expr := logical_or_expr (/\s*/ "=>" /\s*/ logical_or_expr)?    # the implication operator
```

`annotation_value` reaches `implication_expr` through `expression_value → logical_expression`, so the
map **key** greedily consumes `1 => 2` as one expression and `map_entry` then has no arrow left:

```text
✅ Rule 'annotation_value' successfully parsed from 8 to 14 (consumed 6 bytes: '1 => 2')
🔤 Attempting to match terminal '=>' at position 14 (end: 16)
❌ Terminal '=>' failed at position 14
❌ Exiting rule 'map_entry' with error: Backtrack { position: 14 }
```

PEG ordered choice commits `annotation_value` to the expression reading, and `map_entry` has no way to
ask for a shorter key.

## Why both green instruments miss it

- **`--lint-grammar`** reads `ordered_choice_shadowing=0`, exit 0. The two readings do not shadow each
  other *at a single choice point* — one is `map_entry`'s second element, the other is nine levels
  down. **The ambiguity is over a token shared by two rules at different depths**, and no current lint
  class describes that shape.
- **Certificate coverage** reads `map_entry` as witnessed, because the witness generator found an
  input in the accepted half. No amount of witness work moves this.

⇒ when a grammar spells the same operator in two places, neither instrument is the one to ask. Parse a
paired reproducer and control.

## Two method notes that did the work

**Diff the traces; do not read them.** Both arms produced ~4000 lines. Strip the echoed input context,
then `diff`. The divergence was a single `integer_literal` that **fails** in the accepted arm and
**succeeds** in the rejected one — a counter-intuitive shape no one finds by reading downward. A single
trace cannot tell you which line is the anomaly; only the pair can.

**Kill your own findings with a control before publishing them.** `@type: 1 => 2` yields a typed AST
whose `value` is the empty string, which reads exactly like the implication silently discarding its
right operand — a striking thing to report. Plain `@type: 1` yields `value: ""` too. One command;
record the negative result so it is not re-found.

## And stop at the diagnosis when the fix is a language-design call

Three fixes exist — a dedicated map-key rule that routes around the implication level, a negative
lookahead on the implication's optional tail, or re-spelling one of the two `=>` uses — and **none is
WIDEN-only**. The "widen from the empty set, so nothing can regress" argument that makes an inert-rule
repair safe does **not** transfer to a disambiguation. Price all three against a two-arm control first.
