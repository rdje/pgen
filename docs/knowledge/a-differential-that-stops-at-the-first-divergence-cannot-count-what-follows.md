---
id: a-differential-that-stops-at-the-first-divergence-cannot-count-what-follows
title: A token-positional differential files everything after a rule's first divergence as UNVERIFIED — pair it with a per-rule COUNT, built from the two readers you already have
answers:
  - "why did the frontend/meta-parser differential not catch the phantom alternatives"
  - "how do I check that the frontend reads the same number of alternatives the grammar declares"
  - "what does EBNF-ALTERNATIVE-PARITY prove and what does it not"
  - "can a new gate ship with a red control that is real rather than mutated"
  - "what does the meta-grammar say about a comment line between two alternatives"
tags: [ebnf-frontend, meta-grammar, differential, alternatives, doctrine, gates, red-green]
date: 2026-09-03
status: current
evidence: "PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0004: the LANG-CAPABILITY-AUDIT.10.6 differential saw parseable_source_item diverge at token 4 and filed 1 831 later tokens as unverified (98.76 % agreement) while the rule shipped with 4 alternatives for a declared 1. scripts/check_ebnf_alternative_parity.sh counts alternatives per rule in both readers (meta-parser typed AST vs frontend raw_ast): 14 grammars / 4 075 rules agree in 4 s; the pre-fix envelope reads source 1 vs IR 4; the .2 shape reads source 2 vs IR 1 today."
reverify: "bash docs/tasks/artifacts/ebnf_frontend_silent_truncation/alternative_parity/probe.sh   # arm 4 is the live RED (column-0 comment between alternatives); arm 1 the frozen pre-fix envelope"
---

**Agreement measured token by token stops counting at the first mismatch. A phantom alternative lives after it.**

## The rules

1. **A positional diff answers "where do the readers first disagree", not "how many of X does each read".**
   After one inserted or dropped token, every later position is unverified by construction — honestly reported,
   and invisible to a headline ratio. Ask the counting question separately.
2. **Count with the readers you have.** The parser generated from `ebnf.ebnf` is the source's reading; the
   frontend's `raw_ast` is the IR's. One count per rule each (`alternation.alternatives` vs top-level `|` + 1),
   pairing checked first. No third lexer.
3. **Name what you did not evaluate, and refuse when that is everything.** Grammars a reader cannot parse and
   grammars with include directives (the arms describe different rule sets) are listed by name in the summary;
   a population with nothing comparable is rc 2, never OK.
4. **A gate may arrive with a live RED.** The `.2` shape is a real disagreement today (the meta-grammar continues
   a rule past a comment line; the frontend ends it) — keep it as a bank arm that must flip when the fix lands,
   and let it decide the open design question: the meta-grammar already did.

## Related
[[a-comment-syntax-the-frontend-does-not-know-is-a-silent-language]] — the phantom alternatives this gate was
built to see. [[a-red-arm-that-returns-rc-1-from-a-crash-is-not-a-red-arm]] — every RED here asserts text.
