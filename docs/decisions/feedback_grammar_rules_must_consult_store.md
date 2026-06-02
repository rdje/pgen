<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_grammar_rules_must_consult_store.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_grammar_rules_must_consult_store
description: User-set 2026-05-25 (principle) — the semantic store exists specifically to answer identifier-categorisation questions (type vs not-type, class vs not-class, package vs not-package, etc.). Grammar rules that face such questions SHALL use the store. Ungated/provisional rules that bypass the store for these categorisations are defective by design.
metadata:
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**User-set principle (2026-05-25, verbatim):**

> "We have the semantic store, rule shall use it for things like type not type, the store is there for that specific purpose."

## What this binds

The semantic store (`.b.5.1`, multi-indexed, schema-agnostic, scope-aware) was built for exactly this purpose: tracking facts about identifiers (their declaration site, their category, their attributes) so later rules can answer "is `X` a type? is `Y` a class? is `Z` a package?" without re-deriving the answer from parse context.

For any grammar rule whose body is shaped as:
> "match a bare identifier, claim it represents category K"

…where K is a category the producer pass tracks (type, class, interface, package, type_parameter, covergroup, scope, …) — that rule **SHALL** consult the store via the appropriate predicate (`has_fact(K, $body)`, `fact_attribute_equals(K, $body, declaration_family, …)`, etc.). Bypassing the store and matching unconditionally is a design defect, not a "graceful degradation."

The rule has no other authoritative source for the category question. The store IS the authority.

## Why this is not "gate every rule"

Three classes of rule are EXEMPT (they have no categorisation question the store could answer):

1. **Lexical rules** — keywords, literals, operators, punctuation. Match on text pattern alone.
2. **Producer rules** — rules carrying `@emit_fact` for category K. Gating them on facts of category K creates a chicken-and-egg deadlock (the fact doesn't exist until the rule succeeds).
3. **Rules with no identifier-categorisation question** — `expression`, `lparen`, `comma`, statement-shape rules. The store has nothing to say about them.

Every other rule that classifies a bare identifier into a tracked category is in-scope for this principle.

## The defect this names (concrete instance: 2026-05-25)

`grammars/systemverilog.ebnf:1638`:
```
provisional_unscoped_block_class_type := class_identifier ( parameter_value_assignment )? ( scope_resolution class_identifier ( parameter_value_assignment )? )*
```

This rule is one of 15 alternatives in `data_type` (line 1697). It matches any bare identifier as a candidate class type, with **no predicate**. Its predicate-gated sibling `known_unscoped_block_class_type` (line 1635) DOES consult the store via `has_fact(type_name, $head.body)` + `lacks_fact_attribute_equals(... typedef ...)`. The provisional rule bypasses what the gated rule respects.

Trace evidence (2026-05-25, on input `function bit f(string a, b="");`):
- `has_fact(type_name, b) → false` (correctly absent)
- Every predicate-gated type-identifier alternative correctly rejected `b`
- Only `provisional_unscoped_block_class_type` accepted `b` — because it didn't ask

The defect under this principle: the provisional rule asked no question. The store was ready to answer.

## Implied audit (mechanical, non-optional, SYSTEMATIC — user 2026-05-25)

User mandate: **"we need to systematically identify those rules and have them consult the store and not make blind decisions."**

The audit is not a one-off task. It is the standing process: every time a new tracked category is added to the producer pass, the consumer side must be audited for rules in this class. Concretely:

**Pass 1: candidate rule identification.** For each grammar pgen compiles, identify every rule whose body is "match a bare identifier (optionally with a scope-resolution / parameter-value-assignment suffix) and claim it represents a tracked category." Heuristics:
- Rule name starting with `provisional_`, `unscoped_`, `unchecked_`, or similar negation/under-specification prefixes
- Rule body of shape `class_identifier ...` or `simple_identifier ...` or similar bare-identifier rule with no preceding predicate annotation
- Rule appearing in a top-level alternation (e.g., `data_type`, `expression`, `primary`) as a bare-identifier catch-all

**Pass 2: classification per candidate.** For each candidate, determine:
- Is there a paired predicate-gated sibling (`known_*`)? If yes, the candidate's existence is the defect — it must either gate consistently with the sibling or be removed.
- If no sibling, does a tracked category K plausibly cover what the rule is classifying? If yes, the gate is the new annotation to add.
- If the rule legitimately must match without consulting the store (e.g., for forward references the producer cannot cover), document the justification IN the grammar comment so the next audit doesn't re-flag it.

**Pass 3: action per candidate.** Either gate, remove, or document. Each action is its own slice owned by a leaf task. The audit produces the TASK LIST, not the fixes — fixes happen one at a time, each verified on a minimal repro plus the corpus.

**Pass 4: durable record.** Findings land in `docs/reference/SV_EXH_PROOF_DEFECT_TAXONOMY.md` (for SV) or the equivalent for other grammars, under a new defect class entry — so the audit's outputs persist and re-running the audit shows progress.

The audit applies to every grammar pgen compiles, not just SystemVerilog. Any rule, in any `.ebnf`, that classifies an identifier into a tracked category without consulting the store has the same defect shape.

## Why this principle was needed (the meta)

Without this principle stated, the engine kept getting blamed for problems the GRAMMAR was creating by bypassing the store. The Slice-67/68 / Architecture B detour spent significant effort on "fact persistence" engine mechanisms when the actual defect was a grammar rule that never consulted the store in the first place. The store works correctly; the rules that bypass it are the problem.

**Stated differently:** if a grammar rule is going to ignore the store's answer (or not ask), then "the store didn't work" complaints downstream are misattributed. Either use the store, or own the consequences of not asking.

## Cross-references

- [[feedback_universal_semantic_store]] — what the store IS (the substrate this principle binds rules to consult)
- [[project_vision_and_discipline]] — "use what we have well, don't drift"
- [[feedback_no_workarounds_fix_hierarchy]] — L1 (existing semantic annotations) is where this principle applies; L5 (engine) is what gets blamed when L1 isn't applied
- [[feedback_predicate_parse_order_sota]] (RETIRED) — the detour caused by violating this principle and then blaming the engine
- [[feedback_try_parse_must_snapshot_semantic_state]] — the engine-level companion (C3-A is real engine hygiene; C3-B's framing was downstream of this principle being violated)
- [[reference_sv_defect_taxonomy]] — the audit referenced above should produce taxonomy entries for any rules found in violation
