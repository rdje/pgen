---
id: two-tables-answering-one-question-drift-and-the-one-that-prints-knows-less
title: When two tables answer one question, delete the one with fewer readers — filling it in is correct today and wrong at the next entry
answers:
  - "two dispatches answer the same question and disagree — which one do I fix"
  - "should I fill in the missing entries or delete the duplicate table"
  - "a lookup table has stale entries — how do I stop it going stale again"
  - "how do I decide between a cheap fix and a structural one for duplicated data"
  - "my registry field and my match statement disagree — how did that happen"
  - "how do I price deleting a struct field versus populating it"
  - "why did filling in the missing rows feel right and turn out to be the bigger change"
  - "how do I make a divergence structurally impossible instead of currently absent"
  - "what is the no-regression arm for a change that only alters a log or label string"
tags: [registries, duplication, refactoring, dispatch, diagnostics, parser-registry, evidence]
date: 2026-08-22
status: current
evidence: >
  GRAMMAR-WELLFORMED.H.18 (PGEN-GRAMMAR-WELLFORMED-0157). "Is there a detail-capable parser for this
  grammar?" had two implementations in rust/src/parser_registry.rs — a `parse_detail` field on
  GENERATED_PARSER_REGISTRY, and a `match` in parse_sample_detail_with_profile(). They disagreed for
  9 of 13 rows, so certificate-coverage printed "(no detail-capable parser registered)" for grammars
  whose detail parser was one call away. The field had exactly ONE reader (parse_error()), which has
  exactly ONE caller — so deleting the field (plus its type alias, 13 initializers and 1 orphaned
  adapter) was a smaller change than filling in nine entries, and it is the only one the next grammar
  cannot undo. Labels 5 -> 0 false / 0 -> 5 real; all ten certificate tuples byte-identical.
reverify: "python3 docs/tasks/artifacts/grammar_wellformed/cert_wiring_census/probe.py   # LABEL-BLIND-CENSUS: parse_detail_fields=0 — the deleted table has not re-appeared"
---

**Two lookup tables that answer the same question do not stay equal. Nothing holds them equal, so the
only question is which one goes stale and whether anyone notices.**

PGEN's parser registry answered *"is there a detail-capable parser for this grammar?"* twice in one
file. A `parse_detail` field on the registry table:

```rust
pub fn parse_error(grammar_name: &str, sample: &str, profile: Option<&str>)
    -> Option<Result<(), String>> {
    let detail = find_entry(grammar_name)?.parse_detail?;   // ← table A
    Some(detail(sample, profile))
}
```

…and, ~800 lines later, a `match` on the grammar name:

```rust
pub fn parse_sample_detail_with_profile(grammar_name: &str, …) -> Option<Result<(), String>> {
    match grammar_name {                                    // ← table B
        "return_annotation" => Some(parse_with_return_annotation_detail(sample)),
        "ebnf"              => Some(parse_with_ebnf_detail(sample)),
        "vhdl"              => Some(parse_with_vhdl_detail(sample)),
        …
    }
}
```

Table B grew arms as grammars were added. Table A kept its `None`s. They disagreed for **9 of 13
rows** — and table A was the one the certificate-coverage report consulted, so every sample-parse
failure on those nine printed *"(no detail-capable parser registered)"* while the parser it named as
absent sat in table B, working, complete with `furthest_position` augmentation.

## The pricing question that inverts the obvious fix

Faced with nine missing entries, the cheap-looking move is to fill them in. Nine `Some(...)`s, done in
minutes. The expensive-looking move is to delete the field and route the one reader to the surviving
dispatch — that touches a struct, a type alias, every initializer, and any adapter that existed only
to satisfy the field's signature.

Which is actually smaller depends on a fact neither option states. **Ask how many readers the
duplicate has.**

`parse_detail` had **one** — `parse_error()` — which itself had **one** caller. So the field was never
a source of truth anything depended on; it was a private cache of a wrong answer. Deleting it was the
smaller diff *and* the only one the next grammar cannot undo:

| | fill in the nine | delete the duplicate |
|---|---|---|
| correct today | yes | yes |
| correct at grammar #14 | **only if whoever adds it remembers both tables** | yes — there is no second table |
| readers whose meaning changes | 0 | 0 (measured: one reader, redirected) |
| failure mode if forgotten | silent, in the passing direction | impossible |

The decisive column is the last one. A divergence that is *currently absent* is a different property
from one that is *structurally impossible*, and only the second survives contact with the next person.

## Two things to check before you delete

**Prove behavioural equivalence for the rows that already worked, by name.** Four rows had a non-`None`
field; three named the identical function with identical arguments in the surviving dispatch, and the
fourth was a pure passthrough whose whole body called exactly what the survivor calls. That is a
five-minute read and it is what turns "should be equivalent" into "is equivalent".

**Separate your own orphans from pre-existing ones.** Deleting the field orphaned one adapter, and the
build helpfully reported `never used` — for *two* functions. The second was already dead at `HEAD`
(`git show HEAD:<file>` shows only its siblings referenced). Two warnings surfacing together after a
delete is exactly the shape that invites sweeping both into the commit; one `git show` keeps an
unrelated deletion out of a leaf that does not own it.

## And the no-regression arm is not "I only changed a string"

This looked like a label change. It is not: the label path *constructs and runs a parser*, on the
failure path of the very pass that computes the result. The arm that earns the claim is that the
**classification does not move** — the certificate tuple read `144/0/109/35 (spf=13)` before and
after, and all ten families reproduced their prior tuples exactly. That measurement is what makes the
diff a label change, rather than a claim that it is one.

⭐ **The payoff for fixing an instrument before using it arrived on the first run.** With real labels
restored, the report immediately showed the `ebnf` meta-parser rejecting `/**/` — a block comment in
the language `ebnf.ebnf` itself describes — while `block_comment` sat in that grammar's uncertified
list. That correlation had been in the output all along, addressed to nobody, because the field the
report consulted said the parser did not exist.

Related: [[a-refusal-message-that-names-a-cause-instead-of-its-condition-oversizes-the-gap]] — the
sibling defect, where one table's answer was not stale but its *wording* over-sized the gap.
