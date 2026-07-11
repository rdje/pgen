# HORIZON GOAL — PGEN as a universal parser platform for any precisely-described real-world language

**Category:** project · **Established:** 2026-07-11 (director, revising/expanding the prior vision) · supersedes nothing but sits ABOVE [[project_vision_and_discipline]] as the long-term north star.

## The goal (director, verbatim intent)

Long term, PGEN shall have the **minimally required feature set** to accurately parse **any**
real-world machine-readable language that has a precise-enough description we can gather —
concrete named targets: **JavaScript, Perl5, Perl6 / Raku, all markup languages (HTML, XML, …),
any LRM, any programming language**. "Anything machine-readable with a precise enough description
shall be PGEN-parsable." For that to happen, **PGEN's feature set and capabilities must get
stronger** — this is the driver for capability/primitive work.

## Strategic read (how PGEN gets there — the operating frame)

1. **The architecture is already the right foundation, not a rewrite.** PGEN = one declarative
   `.ebnf` (+ semantic annotations/predicates) compiled by a parser-agnostic Rust AST pipeline into
   a signoff-grade parser AND a stimulus generator. The horizon is **completing the primitive set**,
   not changing the model. `EBNF = single source of truth` ([[project_ebnf_is_single_source_of_truth]])
   is the invariant that makes "any language" tractable: every acceptance rule lives in the grammar.

2. **The distinguishing bar is DUALITY, and it is HARDER than parsing alone.** PGEN doesn't just
   parse — it generates valid stimuli (parse⟷generate). Most parser toolkits only parse. For "any
   language," **every capability must be duality-complete**: a primitive that can parse a construct
   but cannot SOUNDLY GENERATE it is INCOMPLETE. First sharp demonstration:
   [[project_scs_numeric_migration_witnessing_wall]] (a forward `+N` reference is parseable but has
   no sound generation ⇒ the migration is blocked). **Design invariant going forward: no new
   primitive is "done" until it is duality-complete.** This is what keeps PGEN signoff-grade and is
   the real moat.

3. **Pursue it via a CAPABILITY-GAP AUDIT, not language-by-language.** Enumerate the cross-language
   PARSING CAPABILITIES real languages need, map each to a minimal general parser-agnostic primitive,
   and build by (coverage × tractability). Languages then fall out as COMBINATIONS of primitives.
   Candidate capability axes (living list — research-grounded per [[feedback_research_grounded_sota_no_trial_and_revert]]):
   - **Context-sensitivity via the semantic store** (already strong: `has_fact`/`fact_count_at_least`/
     scope-context/`value_compare`/`phase:final`). Covers C typedef "lexer hack", declared-before-use,
     cross-reference validation. This is PGEN's core strength.
   - **Forward / deferred obligations** — `phase:final` (FINAL-PHASE-PREDICATE) landed; the
     **forward/suffix-count** primitive (`.4.7.c`) is the next concrete gap.
   - **Layout / indentation sensitivity** — Python, Haskell, YAML, Raku heredocs (partial:
     `@whitespace_sensitive`; needs an INDENT/DEDENT/offside-rule primitive).
   - **Lexer/parser feedback & ambiguity** — JS ASI + regex-vs-division, C++ `>>` / template vs
     shift, most real grammars (needs richer lookahead/backtracking-control + tie policies; partial via
     the tournament/`ordered`/`longest_match` policies).
   - **Error recovery** — HTML/"tag soup" is DEFINED by the WHATWG recovery algorithm, not a clean
     grammar; robust real-world parsing needs a recovery/resync primitive (NOT yet in PGEN).
   - **Preprocessor / macro expansion** — C/C++, the SV preprocessor (partial: svpp grammar).
   - **Encoding / Unicode** — identifier classes, normalization, byte-vs-codepoint (partial).

4. **Honest hard cases (bounded by "precise enough description").** Perl5 is the canonical
   "cannot be parsed without executing it" (BEGIN blocks, prototypes) — the realistic target is the
   precise static subset. HTML "accurate" parsing = implementing the recovery algorithm. These set
   the true scope: not every pathological corner, but every construct with a precise spec, via a
   duality-complete primitive.

## How to apply

- **Reframes prioritization:** capability/primitive-strengthening is the through-line; the regex
  PCRE2-fidelity / SV / VHDL trees are INSTANCES that harden primitives. When choosing the next
  slice, prefer the general primitive that unlocks the most real-language capability.
- **Every new primitive** is designed duality-complete (parse + sound generation) from the start —
  the witnessing wall is the cautionary precedent.
- **Proposed structure (pending director confirmation):** a top-level living CAPABILITY-GAP AUDIT
  (a capability × representative-language × PGEN-status matrix) that drives the prioritized primitive
  roadmap. The `.4.7.c` forward/suffix-count and `.4.9` lookbehind-length primitives are the first
  concrete, already-scoped items on it.
