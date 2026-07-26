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

---

## ⭐⭐ REAFFIRMED (director, 2026-07-26, session #208)

Director, verbatim: *"the north idea goal or idea, is to make PGEN EBNF handling so
good that we can venture in parsing very complex languages like Javascript, Raku,
Ruby, …, that is any human design language shall be parsed with the right EBNF."*

Note the load-bearing clause: **"with the right EBNF."** The bar is not "PGEN has a
JS parser" — it is that **the EBNF language is expressive enough that a correct JS
grammar is WRITABLE**, with the engine staying parser-neutral and agnostic
([[project_ebnf_steers_the_engine_at_full_granularity]],
[[project_ebnf_is_single_source_of_truth]]). Every capability gap is therefore an
*expressiveness* gap in the EBNF surface first, and an engine gap only second.

### Two capability axes ADDED to the living list (found session #208)

Neither was on the axes list above; both are cross-language, and both were found
while fixing ONE SystemVerilog defect.

- **Lexical adjacency / no-layout boundaries** — *"these two elements admit no
  layout (or no newline) between them."* PGEN can do it on both engine halves but
  **no grammar can declare it** ([[project_no_layout_primitive_is_undeclarable]]);
  owned by [`LEX-ADJACENCY`](../tasks/LEX-ADJACENCY.md).
  ⭐ **This axis is squarely on the JavaScript path, and ECMA-262 writes it exactly
  the way the director asked for it.** JS's Automatic Semicolon Insertion is
  specified as `[no LineTerminator here]` markers placed **between two elements of a
  production** — `return [no LineTerminator here] Expression`, and likewise for
  `throw`, `break`, `continue`, postfix `++`/`--`, `async`/arrow `=>`. That is a
  **per-seam lexical constraint stated inline in the production**, i.e. precisely the
  inline-annotation placement the director directed. So the per-seam design is not a
  SystemVerilog convenience — **it is the shape the JS standard itself uses**, and a
  rule-level-only directive could not express ASI at all. Same axis, other targets:
  Ruby (`foo?`/`foo!` method names; `a +b` argument vs `a + b` binary op), Raku
  (whitespace-significant postfix/adverb syntax), C++ (`>>` vs `> >`).
- **Steering-surface GRANULARITY (a meta-capability)** — not a parsing capability
  itself, but the expressiveness of the annotation surface, which **gates every other
  axis**. A capability declarable only per-rule cannot express a per-seam constraint
  no matter how well the engine implements it. PGEN's meta-grammar already admits
  inline annotations (`grammars/ebnf.ebnf:117-130`); the runtime drops mid-sequence
  ones ([`INLINE-ACTIONS`](../tasks/INLINE-ACTIONS.md)). Completing that wiring is a
  prerequisite for the adjacency axis and probably for others.

### ⚠️ The meta-finding — the audit is PRESCRIBED but has never been RUN

§3 above says "pursue it via a capability-gap audit, not language-by-language."
In practice both axes above were found **reactively**: one from an SV corpus defect,
one because the director noticed a design was too coarse. Two years of capability
work has been instance-driven (regex/SV/VHDL trees hardening primitives as they
break), which is effective but **discovers gaps only where a tracked language
already hurts** — and JS/Ruby/Raku are not tracked languages, so their gaps are
structurally invisible today.

⇒ The concrete recommendation the director should rule on: **stand up the audit as a
real tracked tree** (capability × representative-language × PGEN-status), seeded with
the axes above plus the existing list, and drive primitive work from it — rather than
waiting for the next reactive discovery. The cheapest high-value first pass is to
take **one** untracked target language with a precise spec (JS/ECMA-262 is the
strongest candidate: precise, adversarial on lexical adjacency + ASI + regex-vs-division,
and widely understood) and enumerate which of its constructs are *expressible in
PGEN's EBNF today* — a paper exercise, no parser required, that converts invisible
gaps into a ranked list.

## ⭐⭐⭐ SHARPENED — the bar is EAGERNESS, not sufficiency (director, 2026-07-26, session #209)

**Verbatim:** *"PGEN EBNF support shall so powerful, flexible that we should be eager to handle
the creation of even more tricky languages"* — clarified moments later in the same session:
*"… more tricky languages parsers."*

This raises the bar of the goal above without changing its direction. Read it as three
distinguishable commitments, all stricter than what the record previously said:

1. **The test is EAGERNESS, not capability.** The prior framing ("shall be able to parse any
   language like it is a walk in the park") is satisfiable by a grammar that is *possible* to
   write. This one is not: if a notoriously-hard language would be answered with reluctance,
   caveats, or "yes, but you would have to encode it as …", the EBNF is **not yet good enough**,
   even though nothing is strictly missing. ⇒ **expressive AWKWARDNESS is now in scope as a
   defect class**, not just expressive absence. Concrete precedent already on the record:
   `LANG-CAPABILITY-AUDIT.3b` proved Raku-style user-chosen delimiters and here-documents are
   *expressible today* — but only via a four-rule store-guard idiom with a monotone-store
   caveat. Under the old bar that row closes as ✅; under this one it stays open until the
   idiom is something an author reaches for **eagerly**.

2. **"POWERFUL **and** FLEXIBLE"** — two properties, and the second is the one PGEN is thinner
   on. Power = the construct can be expressed at all. Flexibility = it can be expressed
   *naturally*, at the granularity the language actually varies at. The director has now made
   the same point from three directions in two sessions — per-SEAM not per-rule
   ([[project_ebnf_steers_the_engine_at_full_granularity]]), every user-controllable feature
   declared IN the EBNF ([[project_ebnf_is_single_source_of_truth]]), and now flexibility as a
   first-class goal. Treat a capability that exists but only at the wrong granularity as a
   **gap**, not a win.

3. **✅ "the creation of … tricky languages PARSERS" — the unit of work is the PARSER, and the
   ambiguity is CLOSED.** The clarification ("*more tricky languages parsers*") settles what
   could otherwise have been read two ways: this is **not** a new greenfield-language-DESIGN
   audience. PGEN's job stays what this record already says — building signoff-grade *parsers*
   for languages that already exist and are precisely described. The eagerness is ours, about
   **taking on the next hard parser**, and the measure of it is how the grammar-authoring
   experience feels when we do. ⇒ no scope widening, and the ranking input in (1)/(2) is
   specifically the **grammar author's** ergonomic distance, not a language designer's.

### How this re-prioritizes the work

- **`LANG-CAPABILITY-AUDIT.4`** (the prioritized primitive roadmap with a per-gap cost model)
  is now the highest-value open leaf on this axis: the directive is about **building**
  capability, and `.4` is where measured gaps become ranked, priced work items. It outranks
  `.6` (deleting a dead meta-grammar production) — that is hygiene, this is the goal.
- **The matrix must keep GROWING.** "even more tricky languages" is an instruction not to close
  the audit at the 16 rows `.3` enumerated. Each new notoriously-hard language is a column that
  may surface a row nothing else does.
- **Rank by AWKWARDNESS too, not only absence.** Per (1), `.4`'s ranking input is now
  (coverage × tractability × *ergonomic distance*) — a row that is technically ✅ but reached
  only through a multi-rule workaround competes for roadmap space with a row that is ❌.
- The **duality** invariant (§2) and the **zero-cost/neutrality** acceptance test
  ([[project_capability_growth_is_zero_cost_and_neutral]]) are unchanged and still bind every
  new primitive: eagerness never buys an exemption from "non-users pay zero".
