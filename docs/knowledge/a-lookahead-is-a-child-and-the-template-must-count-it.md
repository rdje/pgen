---
id: a-lookahead-is-a-child-and-the-template-must-count-it
title: A lookahead is a CHILD of the sequence it guards — a positional template must count it, and no gate that compares a release with its predecessor can see a binding that was ALWAYS wrong
answers:
  - "I put a !x guard in front of a rule and now $1 is empty — which slot is the identifier"
  - "my list template [$1, $2::2*] returns comma nodes instead of the items after the first"
  - "does a negative lookahead occupy a positional slot in a return annotation"
  - "how can an AST have been wrong for 150 releases with every gate green"
  - "the AST identity sweep read ast_moved=0 — does that prove the tree is right"
  - "how do I pin an AST shape so the pin FAILS on the old parser and passes on the new one"
  - "why is a typed rule's default type missing from the AST"
tags: [grammar, return-annotations, lookahead, ast-shape, gates, evidence, instruments, blind-spots]
date: 2026-09-10
status: current
evidence: SV-CORPUS-GRAD.3.61 (2026-09-10, release 1.0.230, ledger SV-0104 / SV-0105 / SV-0106): ten sites in grammars/systemverilog.ebnf bound a lookahead child and shipped that way across releases 1.0.79, 1.0.191 and 1.0.210; every gate stayed green; found by a scan the moment a NEW guard reproduced the shape on a control (the .3.60 parameter list)
reverify: "bash scripts/check_lookahead_slot_templates.sh   # OK — the auditor's self-test fires on the three finding kinds, then the scoped grammars read findings=0"
---

**In the engine a lookahead is a child.** `!x`, `&x` and `!( … )` derive nothing, but they stand in
the sequence as a positional child that binds `[]` — the interpreter walks it as the `/l` node path,
the generated parser keeps a successful lookahead's pushes — and the documented meaning of `$<int>` is
"the N'th child (1-indexed) of the current rule's parse content". So behind a leading guard the
identifier is `$2`, not `$1`; inside a repeated group `( !( … ) comma item )*` the guard is child 1,
the comma child 2 and the item child 3, and the per-iteration slice that reads the item is `$N::3*`.

PGEN shipped ten templates that did not count it, and every one stayed green under every gate the
tree has:

```text
type_assignment_sv_2017 := !kw_type ident ( assign data_type )? -> {name: $1, value: $2}
    every `parameter type T = int` from 1.0.79 to 1.0.229: name = [], value = T, the `int` GONE
list_of_type_assignments := ta ( !( comma … ) comma ta )* -> [$1, $2::2*]
    every `type T = int, U = real` from 1.0.210: [T, {"kind": "comma"}] — U dropped
property_list_of_arguments … ( comma !dot ( arg )? )* -> ordered_tail: [$2::2*]
    from 1.0.191: ordered_tail = [[]] — the guard's empty node in place of the argument
```

⭐ **Why no gate saw it, and it is a PROPERTY of those gates, not a lapse.** The accept set never
moved, so the lanes, the pins and the certificates were silent by construction — they read
reachability and verdicts, not bindings. The AST identity sweep compares each release's trees with the
PREVIOUS release's, and a consistently wrong tree is byte-identical to itself; `ast_moved=0` proves
that nothing CHANGED, never that anything is RIGHT. Only an oracle that reads the template against the
sequence's own slots can see this class, and none existed — the finding surfaced when a NEW guard
(`.3.60`'s `!( comma ident ident )` on the parameter list) reproduced the shape on a control and the
question "how does the `.3.35` precedent survive its lookahead?" was answered by dumping its tree.

**What to do with the class, not the instance.** (1) An audit that reads templates against slots —
textual, seconds, every commit (`LOOKAHEAD-SLOT-TEMPLATES`; its self-test proves the three finding
kinds fire before the scoped grammars are read). (2) A pin whose ARM is ABSENT on the old parser and
PRESENT on the new one (`control_type_parameter_default_types.sv`: `integer_atom` — the bound
default — and `non_integer` — the second assignment surviving the slice); a verdict-only pin is
green on both sides of a binding bug. (3) A ledger row per WINDOW, because "since when" is the
consumer's first question and three defects had three different first releases. (4) The bootstrap-chain
grammars are outside the textual audit's tokenizer by measurement; the engine-side lint that reads the
real child list is routed, not assumed.

⭐ The converse blind spot is banked separately: a counter counts EVENTS and cannot see a per-event
cost rise ([[a-deterministic-counter-cannot-see-a-per-entry-cost-rise]]); this card is its structural
twin — an identity sweep reads CHANGE and cannot see a constant error. Both are properties of the
metric, and both were given a number only after the blind spot had been paid for.
