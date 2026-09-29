---
name: a-capability-a-grammar-cannot-express-climbs-the-neutrality-ladder-before-it-becomes-a-family-component
description: "DISCIPLINE (director, 2026-09-29, ENGINE-UNIVERSAL-SERVICES.113 / PGEN-ENGINE-UNIVERSAL-SERVICES-0170) — when a capability cannot be expressed in the EBNF, it is first tried as a grammar DIRECTIVE the engine serves for everyone, then as a NEUTRAL engine service, and only when both fail — with the second language that diverges named and measured — as a family component shaped to be lifted later. The ladder lives in the task template; the neutrality guard cannot judge tiers."
id: a-capability-a-grammar-cannot-express-climbs-the-neutrality-ladder-before-it-becomes-a-family-component
title: "A capability a grammar cannot express climbs the neutrality ladder — directive, engine service, family component — and stepping down needs a measured reason"
date: 2026-09-29
evidence: "docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .113 (d): four preprocessor semantics classed by the ladder — include resolution owed to the engine (.114, two witnesses: SystemVerilog and the C family), macro expansion / conditional compilation / keyword tables per family with the diverging second language named; -0170: `--define` renamed `--symbol` because the CONCEPT was universal and the WORD one family's, a flavour the guard could not see."
reverify: "grep -n 'NEUTRALITY TIER' TOOLBOX.md   # the template line; bash scripts/check_engine_neutrality.sh   # what the guard CAN measure (names), which is why the ladder is a review line"
answers:
  - "where does a new capability go when the EBNF cannot express it"
  - "when is a family component allowed instead of an engine service"
  - "what is the neutrality ladder"
  - "why is a one-language engine service a mistake"
  - "what must a leaf record when it adds a family capability"
metadata:
  node_type: memory
  type: feedback
  created: 2026-09-29
---

**The director's rule (2026-09-29, agreed): when work on one grammar needs a capability the EBNF could not
express, try very hard to define it in a way the engine can implement neutrally; only when that fails too
may it become a family capability.** Made operational as a ladder with one rule for stepping down:

1. **Extend the EBNF itself** — a grammar directive the engine serves for every family (`@profiles`,
   `@profile_alias`, `@entry`, the return and semantic annotations). A fact the grammar declares and the
   engine merely reads is the most neutral form there is.
2. **A neutral engine service** — code that is a property of parsing, not of the language parsed
   (left-recursion elimination, the dedicated parse stack, the preprocessing stage's request and outputs).
3. **A family component** — only when both above fail, and shaped so it can be lifted later: declared
   options, the engine's request and output types, reached by family name only
   ([[a-family-component-lives-beside-its-family-and-is-found-by-path-not-typed-as-a-dependency]]).

**Stepping down is allowed only with a measured reason written in the leaf**, never a preference. The
frame's test is "shared by every language that has the concept", so the reason must NAME the second
language and show where it diverges. `.113`(d) is the worked example: macro expansion diverges in syntax
and rules between SystemVerilog and the C family, so it stays per family; include resolution does not
diverge, so it is owed to the engine (`.114`).

**Two cautions keep the ladder honest.** *Speculative generality is the opposite failure*: a "neutral"
service witnessed by ONE language is a family capability wearing the engine's clothes, and it hardens the
first language's reading into everyone's — with one witness, stop at tier 3 but keep the shape liftable;
lifting is a leaf with two witnesses. *The guard cannot judge tiers*: `ENGINE-NEUTRALITY` reads names
([[the-engine-is-neutral-and-the-forbidden-vocabulary-is-derived-from-the-grammars]]) and will never say
that `--define` was flavoured or that a helper belongs one tier higher — which is why the ladder is a
line in the task-acceptance template (`TOOLBOX.md`, **NEUTRALITY TIER**) that every capability-adding leaf
answers: the tier reached, and the measured reason for not reaching the tier above.
