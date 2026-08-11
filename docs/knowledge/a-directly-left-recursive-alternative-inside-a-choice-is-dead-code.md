---
id: a-directly-left-recursive-alternative-inside-a-choice-is-dead-code
title: A directly left-recursive alternative inside a choice is DEAD CODE — LR elimination rewrites only the indirect wrapper shape, and the runtime cycle guard rejects rather than handles
answers:
  - "I transcribed a left-recursive production from the standard and the operator never parses"
  - "does PGEN handle left recursion"
  - "why does the linter say left-recursive is handled by PGEN when the alternative never fires"
  - "why is my binary operator continuation inert while the rule still parses its operands"
  - "when does PGEN's LR elimination actually fire"
  - "how do I write a left-recursive Annex A production in a PGEN grammar"
  - "what does 'Infinite recursion detected in rule X' at the seed position mean"
  - "how do I find every dead left-recursive alternative in a grammar"
tags: [grammar-authoring, left-recursion, codegen, linter, instrument-soundness, lrm-fidelity, systemverilog]
date: 2026-08-11
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .13c.2a.2 (+ .13c.2a.3 subsumed, .13c.2a.4 routed); docs/tasks/GRAMMAR-WELLFORMED.md leaf A2.5; rust/src/ast_pipeline/mod.rs `detect_left_recursive_chain_plan` / `extract_wrapper_suffix`; grammars/systemverilog.ebnf `select_expression` + `block_event_expression`; stimuli/sv/adjudication_repros/fixed_select_expression_{with,paren,or}.sv
reverify: "./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --generate-stimuli --count 1 --seed 0 --dump-gen-ast tmp/ga.json >/dev/null 2>&1 && python3 -c \"import json;d=json.load(open('tmp/ga.json'))['grammar_tree']\nk=lambda n: next(iter(n)) if isinstance(n,dict) and len(n)==1 else None\ndef rr(n):\n    if k(n)=='Atom':\n        t=n['Atom'].get('value',{}).get('Token')\n        return t[1].get('String') if isinstance(t,list) and len(t)==2 and t[0].get('String')=='rule_reference' else None\ndef first(n):\n    x=k(n)\n    if x=='Sequence': e=n['Sequence']['elements']; return first(e[0]) if e else []\n    if x=='Or': return [y for a in n['Or']['alternatives'] for y in first(a)]\n    if x=='Quantified': return first(n['Quantified']['element'])\n    if x=='Atom': r=rr(n); return [r] if r else []\n    return []\nfor nm,nd in d.items():\n    alts=nd['Or']['alternatives'] if k(nd)=='Or' else []\n    for i,a in enumerate(alts):\n        if nm in first(a): print('DEAD', nm, 'alt', i+1, 'of', len(alts))\""
---

PGEN **does** eliminate left recursion — for exactly one shape. Knowing which one is the difference
between a production that works and a production that is silently unreachable.

## The shape the eliminator rewrites

`detect_left_recursive_chain_plan` walks a rule's alternatives looking for one that is a **bare
reference to another rule**, where that other rule's body **begins with the base rule**:

```ebnf
expr      := expr_and     # a bare rule reference — this is what the planner matches
           | primary
expr_and  := expr "&&" expr
```

It rewrites this into `expr := expr_lr_base ( expr_lr_suffix )*` and rebuilds the left-nested tree
from `_pgen_lr_chain` annotations, so the per-alternative `-> {kind: …}` shapes survive.

## The shape it does NOT rewrite — and what happens instead

Write the same language the way a standard's Annex A writes it, with the recursion **inline in the
choice**, and nothing matches:

```ebnf
expr := primary
      | expr "&&" expr        # ⛔ dead: a self-reference in FIRST position inside the choice
```

The alternative reaches codegen intact, and at parse time the runtime cycle guard meets it at the
seed position:

```
🚪 Entering branch 3/8 for rule 'select_expression' at position 127
💥 Infinite recursion detected in rule 'select_expression' at position 127
…
🏁 Rule 'select_expression' selected branch 7/8 consuming 2 chars (branch_policy=longest_match)
✅ Rule 'select_expression' successfully parsed from 127 to 129 (consumed 2 bytes: ' x')
```

⭐ **The guard does not *handle* the alternative; it *rejects* it.** The seed wins, nothing extends
it, and the rule still parses its operands — so the construct looks supported right up until
something has to follow the first operand.

## ⛔ The linter says this is fine

```
[info] grammar info: rule 'select_expression' is left-recursive (cycle: select_expression -> select_expression)
       — handled by PGEN's LR elimination + runtime cycle-breaking (informational, not an error)
```

That message is true for the wrapper shape and **false for this one**, and it is the reason three
dead alternatives of IEEE 1800-2017's `select_expression` — `&&`, `||` and `with ( … )` — survived
into a shipped parser. It is the well-formedness contract's own item 3 (*no dead branches*) failing
in the passing direction. Tracked as `GRAMMAR-WELLFORMED.A2.5`; until it lands, the sweep in this
card's `reverify` is the only instrument that finds these.

## What to write instead

Flatten the recursion into **seed + continuation iteration**, which is what PGEN's SV grammar
already does for `expression` and `constant_expression`:

```ebnf
select_expression      := select_expression_primary ( select_expression_continuation )*
                          -> {kind: "select_chain", head: $1, continuations: $2}
select_expression_primary      := …the non-recursive alternatives…
select_expression_continuation := logical_and select_expression_primary -> {kind: "and", rhs: $2}
                                | …
```

Three things to get right when you do:

1. **Check the language is preserved, not just the strings.** A flat chain is only equivalent when
   the operators are associative and carry no relative precedence — true for `&&`/`||` over bin-set
   predicates, *not* true in general. Where grouping matters, the `( … )` seed arm must exist.
2. **It is an AST-shape change**, because the operands are no longer nested `lhs`/`rhs`. That is a
   schema bump and a consumer migration, so say so in the contract rather than discovering it
   downstream.
3. **Give the wrapper a unique `kind`** so a test can prove the continuation fired — see
   [[a-catch-all-alternative-makes-an-accept-meaningless]].

The alternative repair — teaching the eliminator the direct case, which is mechanically the wrapper
shape inlined — is the better long-term answer and is deliberately left open in `A2.5`: it changes
the AST of every affected rule, so it is a design decision, not a bug fix.

## Sizing it in your own grammar

Run the sweep in `reverify` above. It reads the **post-elimination** gen-AST, so anything it prints
is a genuine dead alternative rather than one the planner already rewrote. On PGEN's SystemVerilog
grammar it prints exactly two rules and four alternatives out of 1 483: `select_expression` (fixed
here) and `block_event_expression`'s `or` arm (`SV-CORPUS-GRAD.13c.2a.4`).

⚠️ Do not read a small count as reassurance. `--lint-grammar` prints only the first **10** of a
grammar's left-recursive findings with no way to show the rest, so 22 of SystemVerilog's 32 were
invisible from the CLI; the sweep had to go around the instrument to see them.

Related: [[a-catch-all-alternative-makes-an-accept-meaningless]] (how the three dead arms looked
supported), [[annex-a-footnotes-license-derivations-the-productions-cannot-derive]] (the standard's
text is not always the grammar you want), [[a-rising-pass-rate-is-not-evidence-of-correctness]].
