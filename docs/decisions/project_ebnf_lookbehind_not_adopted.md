---
name: project-ebnf-lookbehind-not-adopted
description: PGEN's EBNF keeps its PEG lookahead assertions (`&`/`!` — already first-class and load-bearing) and does NOT add regex-style lookbehind (`(?<=)`/`(?<!)`). In a top-down grammar the left context is structural (the rule path already encodes it); no mainstream PEG tool ships lookbehind; the real "left-context" needs are served by regex-token `\b`, rule restructuring, or the semantic store; and every lookbehind use would widen the generator's lookahead-blindness duality debt. Status — PROPOSED 2026-07-08 (director brainstorming question); awaiting director ratification.
metadata:
  node_type: memory
  type: project
  director_directive: false
  status: proposed-awaiting-ratification
  created: 2026-07-08
  owning_tree: (none — language-design adjudication; would be owned by an EBNF-language tree if adopted)
---

**The question (director, 2026-07-08, brainstorming).** Would it help to add support in the EBNF
language for general positive/negative lookahead AND lookbehind assertions, like the ones
`grammars/regex.ebnf` *describes* for the regex language — or is there no concrete value / would it
be really rarely used?

**Context — what PGEN's EBNF already has.** General lookAHEAD is already first-class: the
meta-grammar defines `lookahead_assertion := ("&" | "!") primary_element` (`grammars/ebnf.ebnf:515`),
and it is load-bearing across the corpus — `!quantifier` (the `.3.13` non-quantifiable anchor piece),
`!( comma kw_type )` separators and the `!( identifier )` firewall in `systemverilog.ebnf`, the
`( !")" builtin_any_char )*` payload idiom, and the `.3.14` recognized-name exclusion guard. Nothing
to add on the lookahead side. The open question is only lookBEHIND (`(?<=…)`/`(?<!…)`).

**The adjudication (PROPOSED): do not add lookbehind.**

1. **Left context is structural in a grammar.** Regex needs lookbehind because a regex is a flat
   scan with no derivation context. A PEG/EBNF parse *arrives* at every position through an explicit
   rule path — the left context is already encoded by where you are in the grammar. The cases where
   a regex author reaches for lookbehind are, in a grammar, just "put the alternative on the right
   rule path."
2. **No ecosystem precedent.** Ford's PEG formalism defines only `&`/`!`; no mainstream PEG tool
   (peggy, pest, rust-peg, tree-sitter, ANTLR's syntactic predicates) ships parse-time lookbehind.
   That absence across the whole tool space is strong evidence of near-zero demand.
3. **Zero observed need in this repo.** Across systemverilog / vhdl / regex / svpp / ebnf / rtl_* /
   json + all annotation grammars there is no site that wanted lookbehind and worked around it. The
   two real "left-context" needs that ever appeared have better-fitting existing constructs:
   keyword word-boundaries → regex-token `\b` (the RTL-FE-CLOSURE.10 fix); "declared-before-use" →
   the semantic store (`@emit_fact`/`has_fact` — parser-state-aware, which raw-text lookbehind can
   never soundly be).
4. **It would widen the duality debt.** The stimuli generator is lookahead-BLIND (the tracked
   lookahead-gap class; two honest bounds in `.3.13`/`.3.14` alone, plus `STIMULI-SIGNOFF.14`).
   Lookbehind adds a second, harder blindness: "the text just emitted must (not) match X" — the
   generator would need retrospective re-checking of already-emitted text. Every use-site would be a
   potential generator⟷parser break of exactly the class STIMULI-SIGNOFF is driving to zero.
5. **Real engine cost.** Matching a sub-language *ending* at the current position needs candidate
   re-scans or reversed automata; regex engines cap it (PCRE2 requires bounded-length lookbehinds)
   for this reason. A packrat integration would carry the same restriction + memo interactions.

**Consequences.** If a concrete left-context need ever appears, the fix hierarchy applies before any
language extension: (1) restructure the rule path, (2) regex-token `\b`-style boundary, (3) semantic
store predicate. Only a proven case that defeats all three would reopen this — as its own designed,
parser-agnostic engine feature (the `.3.3.3` IIFE-model precedent for legitimate engine additions).

**Related.** [[project-ebnf-is-single-source-of-truth]] · the STIMULI-SIGNOFF lookahead-gap class ·
`feedback_prefer_grammar_leave_engine_alone` · `feedback_no_workarounds_fix_hierarchy`.
