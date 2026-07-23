# project — make the LRM-markdown→EBNF extractor SOTA + guardrails against silent construct-drops (the `|->`/`|=>` class)

- Date: 2026-07-23 (session #198)
- Category: project (director directive — grammar-fidelity infrastructure)
- Status: ACTIVE — tracked by tree `LRM-GRAMMAR-FIDELITY`; director-gated on timing ("at a later point in time")

## Context

Raised by the director mid-`SV-CORPUS-GRAD.3.3`, right after that leaf found
the two most fundamental SVA operators — overlapped `|->` and non-overlapped
`|=>` (IEEE 1800-2017 A.2.10) — had been **entirely absent from
`grammars/systemverilog.ebnf` since the beginning** (`git log -S` empty). Root
cause: the LRM-markdown→EBNF extractor split the `|`-prefixed operators on `|`,
which is the EBNF **alternation metacharacter**, leaving the mangled `implies`
(`->`) / `or_assign` (`|=`) remnants in `prop_primary`.

Director, verbatim: *"What do you [think] about task-tree tracking this 'the
LRM-markdown→EBNF extractor' and at a later point in time to make this extractor
sota, top-notch and put in place all the necessary guard rails for this problem
to never happen again. Does it make any sense, your take?"*

## Decision

**Yes — it makes strong sense; tracked as tree `LRM-GRAMMAR-FIDELITY`.** The
`|->`/`|=>` gap is a *signature* of a metachar-collision defect class, not a
one-off: every LRM operator whose literal collides with an EBNF metacharacter
(`| ( ) * ? [ ] { }`) is suspect the same way (`[*N]`, `[->N]`, `[=N]`,
`(* … *)`, `->>`, `##`). Fixing them via the corpus burn-down is **reactive**,
one family at a time (O(defects)); a proactive audit + a standing guardrail is
**O(1)** for the whole class. This is the project's own doctrine (fix the class
at the source + a standing gate, cf. `BIN-BUILD-INTEGRITY`).

## Engineer's take (recorded — the sharpening caveat)

One honest caveat that makes the idea SHARPER, not weaker: **the extractor is no
longer on the live regeneration path.** The shipped `systemverilog.ebnf` is a
hand-annotated flattened *synthesis* (`@profiles`, return annotations, store
predicates) that a re-extraction would destroy. So "make the extractor SOTA"
*alone* does not fix the shipped grammar. That splits the work by leverage:

1. **Highest-leverage, extractor-independent (recommended FIRST):** a standing
   **LRM-Annex-A ⟷ shipped-grammar coverage gate** — prove every LRM
   production/operator is reachable in the shipped grammar. Finds all remaining
   `|->`-siblings in one pass and prevents recurrence regardless of how the
   grammar was authored. It is the proactive twin of `SV-CORPUS-GRAD.7a`'s
   rule-coverage instrument (LRM→grammar vs corpus→grammar) and directly feeds
   the `.9` 100%-LRM-coverage mandate ([[project_sv_corpus_100pct_lrm_coverage_mandate]]).
2. **Systemic capstone (the director's "later"):** the SOTA extractor + an
   extraction-fidelity gate — valuable as a re-derivable *oracle* to diff the
   shipped grammar against, and as the faithful path for future LRM-based
   families (VHDL; the horizon goal, [[project_horizon_universal_parser]]). The
   generalizable guardrail: any markdown→EBNF extractor must escape/detect
   operator literals that collide with EBNF metachars, with a round-trip check
   that the emitted grammar's operator terminals ⊇ the LRM's operator table.

**Recommendation:** pull the `.1` coverage audit forward (bounded, high-yield —
it surfaces the rest of the class proactively); the `.3` extractor rewrite can
wait. The director decides when each leaf opens.

## Sequencing (director-gated)

Framed "at a later point in time" — after the current SV delivery push. Session
#198 CREATED the tree (the "task-tree track this" directive) and did NOT start
any leaf; the SV `SV-CORPUS-GRAD.3.x` burn-down remains the active frontier.
Related trees: `SV-CORPUS-GRAD` (reactive corpus burn-down + the `.9` mandate),
`CORPUS-GRAD-ALL` (cross-family graduation).
