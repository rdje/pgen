---
id: a-fallback-arm-goes-inside-the-shared-rule-as-an-ordered-last-alternative
title: A fallback arm goes INSIDE the shared rule as an ordered last alternative — a passthrough twin under longest_match pays the fallback at every site, including where the known arm already won
answers:
  - "where do I put an escape / fallback alternative for a rule that is referenced from many sites"
  - "why was the class_scope_escapable passthrough twin refused by the parse-cost ratchet"
  - "why is class_scope @branch_policy: ordered since 1.0.243 and why does class_scope_plain exist"
  - "does a passthrough dispatcher `a := b -> $1 | c -> $1` under longest_match derive both alternatives"
  - "when is an ordered choice provably the same choice longest_match would have made"
  - "why did the import-gate-first arm cost more than the ungated escape"
  - "how do I classify a shared rule's call sites into those that can afford an escape and those that cannot"
tags: [sv-corpus, grammar-design, parse-cost, branch-policy, wildcard-escape, costs-are-rejected-not-traded]
date: 2026-09-23
status: current
evidence: "SV-CORPUS-GRAD.31 (PGEN-SV-CORPUS-GRAD-0412, release 1.0.243): three spellings of ONE accept set measured by the parse-cost ratchet on the pinned 192-file sample — the passthrough twin `class_scope_escapable := class_scope -> $1 | wildcard_escape_class_scope -> $1` at nineteen sites under longest_match, entries +1.43 % (the dispatcher entered 3.6 M times, the escape's asks paid at every class_scope evaluation); the escape as class_scope's own LAST alternative under `@branch_policy: ordered`, entries +1.27 % (tried only where the known arm fails); the ordered escape opened with the zero-width `wildcard_import_open_gate`, entries +1.82 % (the gate and its `&( trivia )` lookahead entered 1.3 M / 0.9 M times, and a store-first arm re-asked the call prefix's known heads — the .27 memo-taint class). Every correctness lane (pins, three corpus lanes, AST identity sweeps, two-sided reading) read the three spellings identically; only the ratchet told them apart. docs/tasks/SV-CORPUS-GRAD.md `.31`, rust/target/s375/prereg.md ADDENDA 5/8/10 (session scratch), grammars/systemverilog.ebnf `class_scope` / `class_scope_plain` / `class_scoped_call_prefix`."
reverify: "grep -n -B2 -A6 '^class_scope := ' grammars/systemverilog.ebnf | grep -E 'branch_policy: ordered|wildcard_escape_class_scope_type scope_resolution'   # two lines: the ordered policy and the escape as the LAST alternative; then grep -c 'class_scope_plain' grammars/systemverilog.ebnf   # 6 lines: the definition, the four type-position references, one comment"
---

**When one rule serves many sites and a fallback (an escape, a permissive reading) is wanted at
most of them, the fallback is an ORDERED last alternative of the rule itself, and the sites that
cannot afford it take a plain twin — never a passthrough dispatcher under `longest_match`.**

## The mechanism

- A passthrough twin `x := known -> $1 | fallback -> $1` under `longest_match` is a tournament: the
  engine DERIVES every alternative and picks the longest, so the fallback's work (an identifier
  chain, a fact lookup, a memo-served parameter list) is paid at every evaluation — including the
  overwhelming majority where the known arm already won. Its memo replays are merely relabeled
  from the inner rule to the dispatcher, and the total rises by the fallback's asks everywhere.
- The same fallback as the rule's own last alternative under `@branch_policy: ordered` is tried
  only where the known arm FAILS. The choice is provably the one `longest_match` would have made
  when the two arms cannot both succeed on different spans: here a declared class reads the same
  span on both arms and the first wins by declaration order, so every previously accepted tree is
  byte-identical by construction (the AST identity sweeps then read `ast_moved=0` as the check,
  not the argument).
- Guarding the ordered fallback with a zero-width fact gate "to make the miss cheap" can cost MORE:
  the gate is itself one or two entries at every refused attempt, and a store-touching arm placed
  before the identifier ask taints the memo of its siblings (the `.27` lesson, restated by `.31`
  ADDENDUM 10). Measure the guard; do not assume it.

## Classifying the sites

The sites that cannot afford the fallback are found by a `grep` of the rule's callers plus the
language's item grammar, not by intuition: for `class_scope` the four TYPE-position prefixes
(`scoped_type_scope_prefix`, `nettype_scope_prefix`, the two `ps_type_identifier_scope` twins) can
open a module item, where `ident #( … )` is also an instantiation head and a consumed parameter
list is re-parsed on the instantiation route after the rollback
([[ENGINE-UNIVERSAL-SERVICES.111]]) — they take `class_scope_plain` (the first alternative alone).
The other nineteen sites take the rule with its escape.

## Why the ratchet is the instrument

Every correctness lane reads the three spellings as ONE accept set; only the parse-cost ratchet's
binding counters separate them, which is the case for measuring cost BEFORE the ceremony, not
after (`costs are REJECTED, not traded` — [[project_north_star]]). Related:
[[a-deterministic-counter-cannot-see-a-per-entry-cost-rise]],
[[a-widening-arm-probes-its-sibling-sites-before-the-corpus-does]].

## A second instance — the preprocessor grammar (2026-10-18, `SV-CORPUS-GRAD.13c.2v.13`)

The `systemverilog_preprocessor` grammar's next release added a fallback for a conditional group: a line that does not
parse as a directive is recorded as a `malformed_directive` item. Written as `pp_group_item := pp_item |
pp_malformed_directive` under the default `longest_match`, the fallback — a whole lexical line — would OUTLENGTH a
valid directive that a line's text follows (`` `timescale 1ns/1ps wire w; `` inside a group) and win. The same
release had a second case of the same misreading: an escaped-name macro usage (`` `\name ``) outlengthing the
`` `\`" `` escape it shares a prefix with. Both were written as if `|` were first-match; this card already said why
not. The fallback took `@branch_policy: ordered`; the escaped name may not begin with a backtick.
