---
name: project-store-aware-generation
description: PLANNED capability (director 2026-06-08) — the stimuli generator shall become semantic-store-aware (context-valid): it maintains a generation-time semantic store, emits the same facts the parser would, and honours the grammar's `@predicate` annotations (starting with `fact_count_at_least`) so it emits only samples that satisfy the same semantic constraints the parser enforces. The generation-side dual of the parser's store-gated rules; the SAME `@predicate`/`@emit_fact` annotations steer both parse and generation (no new annotation vocabulary). Tracked by tree STORE-AWARE-GEN; implementation deliberate / "at some point".
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-06-08
  owning_tree: STORE-AWARE-GEN
---

**THE DIRECTIVE (director, 2026-06-08).** "Please plan this 'a new generator capability
(semantic-store-aware generation honoring `fact_count_at_least`)' for the stimuli generation at some
point. Task-tree track it, so that we do not forget." (In response to surfacing that the regex
`REGEX-PCRE2-FIDELITY.3.12` residual — the generator emitting `\98495`, a backreference to a
non-existent group — cannot be fixed cleanly without this capability.)

**LANDED SO FAR.** `.3` (`PGEN-STORE-AWARE-GEN-0003`) landed the `fact_count_at_least`-aware MVP
(generation-time store + count-prune + emit hook), closing the regex `.3.12` driver. `.4b.2`
(`PGEN-STORE-AWARE-GEN-0006`, 2026-06-22) generalised the witness-pass semantic-prelude from *count*
gates to NAME-matching store gates (`has_fact` / `fact_attribute_equals`) as a **declare-then-use name
coordination**: a producer declaration is hosted upstream, its emitted name resolved to the rendered
identifier, and the gated use-site forced to render that same store name — so a use-of-a-declared-name
rule witnesses through a synthesised declaration. SystemVerilog cert `UNKNOWN 56 → 46` (the directly
reachable `checked_*` / `known_unscoped_*` type/covergroup/nettype/let/parameter cohort), generator-only,
capability-gated (byte-identical for predicate-free grammars and regex), zero newly-UNKNOWN, the parser
re-check the sole witness judge. Self-emitting producers (the forward-declaration idiom) are excluded
from the consumer cohort. `.4b.3` (`PGEN-STORE-AWARE-GEN-0007`, 2026-06-22, PURE-DOCS DESIGN) decomposed
the remaining `UNKNOWN=46` into 3 implement sub-cohorts (3A block-scoped non-gated carrier, 3B
class-member/class-scope, 3C/misc reach-routing) — after subtracting 19 `no_path` non-defects and the ~7
SVA infix-operator PARSE bug (out of scope) — and A/B-proved the `block_type`/`data_type` asymmetry root
cause: `compute_name_prelude`'s gated-rule search inspects only the reach-path hops + the target, never
the target's MANDATORY SUB-RULES, so a non-gated carrier of the inner-gated `checked_type_identifier`
never receives a declare-then-use prelude. `.4b.4` (`PGEN-STORE-AWARE-GEN-0008`, 2026-06-22) landed that
fix — a bounded mandatory-first gated-rule descent so a carrier inherits its inner gate (the prelude
arms on the inner rule that actually renders) — closing sub-cohort 3A: SystemVerilog cert `UNKNOWN 46 → 43`
(+3 block-scoped carriers `known_unscoped_block_type_identifier`, `known_unscoped_block_covergroup_identifier`,
`provisional_unscoped_block_class_type` witnessed), generator-only, purely additive, zero newly-UNKNOWN,
the 6 fully-certified grammars unperturbed. `.4b.5` (`PGEN-STORE-AWARE-GEN-0010`, 2026-06-23, PURE-DOCS
DESIGN) root-caused the class-member/class-scope sub-cohort 3B into three sub-blockers (3B-ii
producer-selection, 3B-i gate-not-mandatory-first, 3B-iii target-own incompleteness). `.4b.6`
(`PGEN-STORE-AWARE-GEN-0011`, 2026-06-23) landed the 3B-ii fix — and REFINED its root cause tools-first:
the non-bootstrapping-ness is NOT in the producer rule body (both family=class producers share the
byte-identical body `type_identifier`) but in the producer's HOST BRANCH of `type_declaration` (the
typedef-alias's `class_type` mandatory SIBLING is `has_fact(type_name)`-gated, so its empty-store prelude
declaration cannot parse). The fix makes producer selection reach-path-aware (a two-pass scan preferring a
producer whose forced reach path renders no unsatisfiable same-store gate, reusing the existing
`mandatory_node_gated` store-gate walk), closing the class-scope cohort: SystemVerilog cert `UNKNOWN 43 → 41`
(+2 rules `known_unscoped_class_scope_class_identifier`, `known_unscoped_class_scoped_call_class_identifier`
witnessed), generator-only, zero newly-UNKNOWN, the 6 fully-certified grammars unperturbed. `.4b.7`
(`PGEN-STORE-AWARE-GEN-0012`, 2026-06-23) landed the 3B-i fix by widening *where* the discovery
(`name_gate_via_mandatory_prefix`) looks for the gate: a second leg scans past a leading run of
optional/keyword elements and descends an ordered choice, but only as far as the FIRST rendered position
that is unavoidably store-gated (`node_render_store_gated` = the same `mandatory_node_gated` walk) and only
into a choice with no ungated escape, guarded by `mandatory_reach_gate`. (A first cut that scanned every
element + every choice regressed 6 rules — reaching a dodgeable `type_name` gate behind a self-satisfying
producer + an ungated `data_type` escape — caught by the deterministic strict-subset gate and refined.)
This arms a prelude for the `extern_constraint_declaration*` family (gate behind a leading `constraint`
keyword + inside `class_scope_type`'s `Or`): SystemVerilog cert `UNKNOWN 41 → 38` (+3:
`extern_constraint_declaration_sv_2017`, `extern_constraint_declaration`, bonus `class_scoped_tf_call`),
generator-only, zero newly-UNKNOWN, the 6 fully-certified grammars unperturbed. `constraint_set` is a
DISTINCT off-path-mandatory-sibling discovery → folded into `.4b.9`; target-own incompleteness → `.4b.8`.

**THE GAP (tool-backed, 2026-06-08).** PGEN's EBNF `@predicate` annotations are the single source of
truth for the accepted language, and the PARSER honours them (e.g. `numeric_backreference` is gated by
`@predicate: fact_count_at_least(regex_capture_group, $index)` — accepted only if that many capture
groups exist). But the stimuli generator has **zero** fact/predicate machinery (`fact_count_at_least`
is evaluated only in `semantic_runtime.rs` and the linter, never in `stimuli_generator.rs`). So the
generator over-generates semantically-invalid samples the parser correctly rejects — a generator⟷parser
duality break at the SEMANTIC level.

**THE DECISION.** Build a generation-time semantic store + predicate evaluator (reusing
`semantic_runtime.rs`, parser-agnostic) so the generator emits the parser's facts as it generates and
gates its branch/value choices by the grammar's `@predicate` constraints. Start with
`fact_count_at_least` (the regex `.3.12` driver: restrict a generated backreference index to ≤ the live
count of emitted `regex_capture_group` facts), then generalize to the composable primitives
(`has_fact`, `lacks_fact`, `fact_attribute_equals`, `resolve_path`). NO new annotation vocabulary — the
generator becomes a second consumer of the existing one; NO fixed-bound guesses (the valid bound is
context-dependent). Per the no-workarounds hierarchy this is a justified **Level-3+** general
parser-agnostic generator capability (the generator structurally cannot otherwise honour a
context-dependent predicate).

**SOTA ANCHOR.** ISLa — "Input Invariants" (Steinhöfel & Zeller, ESEC/FSE 2022): declarative semantic
constraints layered on a CFG to generate inputs satisfying grammar AND constraints. PGEN's
`@predicate fact_count_at_least(...)` is exactly such a constraint; this is PGEN's native realization,
reusing the existing annotation vocabulary. Also grounded in data-dependent grammars (Jim et al. POPL
2010 — already PGEN's parse-side basis) and the Fuzzing Book's constraint-generation chapter.

**STATUS.** PLANNED + task-tree tracked (tree `STORE-AWARE-GEN`, `.1` SCOPING done 2026-06-08). The
implementation (`.2` design → `.3` `fact_count_at_least` → `.4` generalize → `.5` verify) is a
deliberate effort sequenced "at some point" per the director; tracking now ensures it is not forgotten.

Composes with [[project_ebnf_is_single_source_of_truth]] (extends the duality from structure to
semantics) and the certifying-linter trustworthiness model (a generated witness must round-trip). Does
NOT change the parser or grammar acceptance semantics (generation-only; the parser stays the oracle).

---

**AMENDMENT — SHARPENED DIRECTIVE (director, 2026-06-08, same session, emphatic).** Triggered by the
director observing that the svpp stimuli generator "is outputting garbage" (tool-backed: ~48% of every
generated svpp sample is block-comment chars, ~9.4 comments/sample, plus stray-punctuation
`condition_atom` soup) and concluding: **"it simply means the generator is not properly steered by the
EBNF … the generator needs FULL support for the semantic fact store … it needs to output text based on
CONTEXT == semantic fact store."**

This ELEVATES the tree's scope. Store-aware generation is not just "honor `fact_count_at_least` to stop
one bad backreference" — it is THE mechanism by which the generator becomes **context-aware** and stops
emitting garbage. A signoff generator emits each token IN CONTEXT, and **the semantic fact store IS the
context**: a `` `MACRO `` reference only after its `` `define ``; a type-position identifier only after
its `typedef`; a scoped name only where the scope holds — by EMITTING (`@emit_fact`) and CONSULTING
(`@predicate`/`has_fact`/`lacks_fact`/`fact_attribute_equals`/`resolve_path`) the SAME store the parser
uses. Target = **FULL** support across every grammar (all predicate primitives + complete `@emit_fact`
emission + scope-awareness during generation), not the single-predicate regex MVP.

HONEST SCOPE NOTE (do not over-claim): full store-aware generation closes the **semantic** incoherence
class (wrong identifiers / dangling references / predicate violations). It does NOT by itself fix two
ADJACENT "garbage" facets that share the "properly-steered" goal: (1) **trivia/comment density** — the
`*`-quantifier + `(space_or_tab | block_comment)*` sampling that yields the ~48% comment soup is a
generation-WEIGHTING knob, not a store fact; (2) the svpp `condition_text` `\n` residual
(`GRAMMAR-WELLFORMED.H.5.1.3`) is a LEXICAL-context issue (a flexible atom in a `+` list), not a
semantic-store fact — the successor-aware `\n` deferral (H.5.1.3.1) was implemented + tools-measured
INSUFFICIENT (over-fires on comment-led successors) + reverted; that residual routes to Phase C. "Context
== store" is the dominant and deepest facet; the density knob + lexical-context are the other two facets
of the same "make generation faithful/representative" program. [[project_stimuli_generator_signoff_vision]].

IMPLEMENTATION PROGRESS (overwrite this line as the arc advances — detail in `docs/tasks/STORE-AWARE-GEN.md`):
the `.3` `fact_count_at_least` MVP closed the regex driver; the `.4b` name-coordination arc drove the
SystemVerilog certificate-coverage residual `UNKNOWN 56 → 46 → 43 → 41 → 38 → 37 → 33 → 32` (`.4b.2` name-prelude →
`.4b.4` mandatory-first descent → `.4b.6` self-bootstrapping host-branch → `.4b.7` structural gate-discovery
→ `.4b.8` **collide-aware free-name diversity** — the generation-side dual of the parser's
type-vs-identifier disambiguation: a free declaring identifier whose canonical name collides with a
name-gate-consumed `type_name` fact renders a distinct name so a type-first ordered choice cannot steal it →
`.4b.10` **off-path-sibling prelude-arming** — a third declare-then-use discovery leg
(`name_gate_via_offpath_sibling`) that arms a prelude on a name gate carried by a mandatory OFF-PATH SIBLING
of the on-path element along a reach hop, reusing the `.4b.6` path-walk + the `.4b.7` unavoidably-store-gated
guard; closed `constraint_set` + `declared_class_alias_identifier` + the `named_checker_port_connection*`
pair, +4 witnessed →
`.4b.12` **carrier-diversification reach pass** — the reach-side dual of the same principle: a residual target
that reaches its context but routes through a parent-ordered-choice sibling is re-routed to reach a rule on its
default path through an ALTERNATIVE parent (keeping the tail), so a different trailing context defeats the
shadowing sibling [`known_unscoped_class_scope_type_parameter_identifier` via `class_new`'s `::new` suffix];
the two `class_scoped_call` cousins are tool-proven NOT carrier-divisible (expression-level call-form ambiguity)
⇒ a deferred grammar-gate item). Every step is generator-only and capability-gated on the grammar's own name gates
(`gen_name_gate`) or runs only over a non-empty cert residual, so it is byte-identical/inert for the fully-certified
roster. Frontier `.4b.13` = reach-ROUTING
forcing for the residual 9C cohort (`parsed=true witnessed=false`).
