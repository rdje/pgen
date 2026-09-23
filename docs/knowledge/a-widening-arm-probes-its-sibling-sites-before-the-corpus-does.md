---
id: a-widening-arm-probes-its-sibling-sites-before-the-corpus-does
title: A widening arm probes its sibling sites before the corpus does — ten-line files under the released probe find the over-acceptance and the next gap for the price of a trace
answers:
  - "I am adding an escape / an accepting arm at one site — what do I probe before editing the grammar"
  - "how did SV-CORPUS-GRAD.30 find that the .28 escape admitted a declared typedef with a parameter list"
  - "why does a block item's type need no scope gate when class_type needed one"
  - "how do I decide whether ident #( can be an instantiation head at a site"
  - "what is the cheapest way to find the sibling over-acceptance of a widening before the lanes run"
tags: [sv-corpus, grammar-design, over-acceptance, wildcard-escape, toolbox-first, claim-verification]
date: 2026-09-23
status: current
evidence: "SV-CORPUS-GRAD.30 (PGEN-SV-CORPUS-GRAD-0411): fifteen ten-line probes under the released 1.0.241 probe, before any grammar edit, read (1) `import q::*; typedef int T; class c; T #(int) x; endclass` ACCEPTED through the .28 escape (`🏁 Rule 'class_type' selected branch 2/2`) while the same text without the import is refused at byte 34 — an over-acceptance fixed at the shared head in the same slice; (2) `typedef int T; module m; initial begin T #(int) x; end endmodule` ACCEPTED with no import by `provisional_unscoped_block_class_type` (`block_data_type` branch 13/17) — routed .32; (3) `uvm_config_db#(int)::set(...)` under an open import REFUSED at the `(` after `::set` (`rejected by post predicate 'fact_attribute_equals [type_name, uvm_config_db, declaration_family, class]'`) — routed .31, the dominant UVM idiom. The block site's reach was read off the grammar: block_data_type has one caller chain to block_item_declaration, whose references admit no module_instantiation."
reverify: "python3 stimuli/sv/run_adjudication_repros.py --verbose 2>&1 | grep -E 'invalid_wildcard_escape_parameterized_declared_typedef|accepts_invalid_block_typedef_parameterized|defect_wildcard_escape_class_scope_static_call_tf'   # the three findings, pinned two-sided"
---

**Before an arm widens what a name can be at one site, ask the released probe what the same
bytes already do at every sibling site — with ten-line files, not corpus rows.**

## The rule

A widening (an escape, a new accepting alternative) is designed against ONE site — the corpus row
that opened the leaf. Its sibling sites are where the same shape already has a reading, and the
three questions below are each a five-second `--parse` under the released probe:

1. **Over-acceptance at the site the widening copies from.** Write the shape with the name
   *provably* something else (a local `typedef`, a nettype, a covergroup) and parse it with and
   without the enabling condition (the open wildcard import). If the enabling condition alone flips
   REJECT to ACCEPT, the copied arm admits what the store can refute — fix it at the SHARED head,
   in the same slice, with the exclusion the known arm already spells (`SV-CORPUS-GRAD.30`:
   `lacks_fact_attribute_equals(type_name, $body, declaration_family, typedef)`).
2. **The pre-existing permissiveness at the site being widened.** Parse the same illegal shape with
   NO enabling condition: an ACCEPT there is an older decision (`provisional_unscoped_block_class_type`,
   kept "for class-internal scoped types" in 2026-05), not the widening's — pin it `accepts_invalid`
   and route it; do not fold it in.
3. **The competing shape at the same position.** List the constructs that can open with the same
   tokens at that position (a block item's type competes with a STATEMENT opening `C#(int)::`) and
   parse the most common corpus spelling of each: `SV-CORPUS-GRAD.30` found the UVM
   `uvm_config_db#(T)::set(...)` idiom REFUSED under an open import — a bigger gap than the row that
   opened the leaf, one release earlier than the residual table would have surfaced it.

## Whether a site needs a scope gate is read off the grammar, not argued

`.28` gated its `class_type` arm on a `class_body` scope because `data_type` is reached from module
scope, where `m #(…) u (…)` is an instantiation head and the escape's parameter list was parsed
twice. The block site took the escape UNGATED because `block_data_type` has exactly one caller
chain (`block_data_type_or_implicit_explicit` → `block_data_declaration` → `block_item_declaration`)
and `block_item_declaration`'s references are seq / par blocks and tf / constructor bodies — none
admits a `module_instantiation` (IEEE 1800-2017 A.6.3). `grep -n` on the callers is the proof; the
ratchet prices what is left.

## Then MEASURE the rendering

The escape's template copied the known arm's `{head, params, scope_chain}`; the claim that an escaped
block local and a declared one are the same node was read as `SUBTREE-IDENTITY: byte-identical`
from two `--interpret-parse-ast-json` dumps, never from the template text (`TOOLBOX.md` §1.5c).
