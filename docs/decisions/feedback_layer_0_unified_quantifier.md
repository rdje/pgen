<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_layer_0_unified_quantifier.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_layer_0_unified_quantifier
description: "Layer 0 unified quantifier engine landed 2026-05-21; codegen has one parameterised loop for ?/*/+/{N}/{N,M}/{N,}/{,M}; per-iter atomicity is uniform via try_parse; quantifier-level atomicity via quantifier_start_position save+restore on min-failure (elided for min==0)."
metadata: 
  node_type: memory
  type: project
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**SV-EXH-PROOF.3.3.4.b.3 (PGEN-SV-EXH-PROOF-0029, rel 1.0.126, 2026-05-21)** — Layer 0 is the parser-agnostic primitive that brings every EBNF repetition operator under one symmetric codegen path. Recorded for future quantifier-related work so the design rationale isn't re-derived from scratch.

**The defect Layer 0 cures**: `rust/src/ast_pipeline/ast_based_generator.rs::generate_quantified_logic` (and `ast_code_generator.rs::generate_quantified_content`) had three independently-written code paths for `*`, `+`, `?`, with the `+` arm emitting the FIRST iteration INLINE WITHOUT `try_parse` wrapping — so on first-iter failure the cursor was left wherever partial parse stopped, relying on the surrounding rule's `try_parse` to roll back. Bounded forms (`{N}` / `{N,M}` / `{N,}` / `{,M}`) declared in `grammars/ebnf.ebnf:149-189` were never implemented (`_ => Err("Unknown quantifier")` fallthrough).

**The two-layer atomicity model**:
- **Layer 1 (per-iteration)**: every iteration is wrapped in `try_parse` → on iter failure, cursor restores to iter start (= where the failed iter began). Loop breaks; iteration_count records committed iters.
- **Layer 2 (quantifier-level)**: when `min > 0`, codegen emits `let quantifier_start_position = parser.position;` BEFORE the loop; on min-failure restores `parser.position = quantifier_start_position;` before `Err(Backtrack)`. The quantifier is atomic at its own boundary — partial successes are undone if `min` not reached. For `min == 0` (`*` / `?`) the bind + check are ELIDED to avoid always-false comparisons (would emit 495 unused-variable warnings on the SV parser alone).

**Surface → (min, max) mapping** (canonical helper `parse_quantifier_bounds` in `rust/src/ast_pipeline/mod.rs`):
- `?` → `(0, Some(1))`
- `*` → `(0, None)`
- `+` → `(1, None)`
- `{N}` → `(N, Some(N))`
- `{N,M}` → `(N, Some(M))` (rejects `M < N`)
- `{N,}` → `(N, None)`
- `{,M}` → `(0, Some(M))`

**Verification reach**: lib 465/465 PASS (+4 helper tests); lib 521/521 with `--features generated_parsers`; RGX broader corpus / conformance 44/0 ✅; SV shape-contract GREEN; SV smoke 4/4 (module, if-no-else, else-if chain, for-loop); **SV external corpus 8/14 → 10/14** (friscv_rv32i_core ×{2017,2023} unblocked).

**Honest design-vs-result update**: At design time I hypothesised Layer 0 was pure infrastructure (`stays 8/14` expected). The triage gate proved that wrong by +2 cases. The friscv_rv32i blocker — categorized at `.3.3.3`/`.3.3.2` as the `.3.3.6` statement-level residual — was NOT a separate statement-level grammar defect; it was the prior codegen's asymmetric `+`-first-iter-not-wrapped-in-`try_parse` defect. Whatever `+`-quantifier was hit deep inside friscv_rv32i's statement parsing left the cursor in a non-rollback-able state on first-iter failure. Layer 0 wraps the first iter of `+` uniformly with every other iter, the cursor rolls back cleanly, and friscv_rv32i parses end-to-end. THIS is why .b.3 is a corpus-mover even though the design claimed "behavior-equivalent for `*`/`+`/`?` on success paths": the BEHAVIORAL DIFFERENCE on the prior `+` first-iter-FAILURE path was real. `.3.3.6` is now closed-by-evidence. **Lesson for future infrastructure-feeling slices**: don't pre-emptively label a slice "no corpus delta expected" without running the gate — the asymmetric defects you're refactoring away may BE the blocker some corpus case is hitting. Run the full gate post-implementation and let the gate speak.

**What Layer 0 does NOT fix**: cross-rule backtracking (e.g. `hierarchical_identifier`'s first-set-overlap case `(identifier constant_bit_select dot)* identifier` — PEG can't try the trailing `identifier` at each iter before committing the prefix). That's a separate engine class — `.b.2` was the surgical patch for one such case (reverted in favor of Layer 0 as infrastructure; the surgical patch CAN be re-applied later if it turns out a per-rule explicit stop-guard is still needed). Cross-rule backtracking would be a future Layer 1+ if needed.

**Bounded quantifiers are now infrastructure**: `{N}` / `{N,M}` / `{N,}` / `{,M}` produce well-formed parsers but no current grammar uses them. If a future grammar (e.g. SV `[N:M]` ranges, regex `{N,M}` exponent constraints) wants them, just write them in EBNF — the codegen is ready.

**User-described semantics, recorded verbatim** (the design constraint Layer 0 satisfies):
- "the current value of the cursor shall be saved at the start of an iteration, if the iteration fails we put the cursor back to the position before starting that iteration, and we check if the repetition is fulfilled or not. if it passes, we continue with the next iteration if necessary until the repetition operator is fulfilled" — per-iter cursor save+restore via `try_parse`; min check at end determines success/failure.
- "For `{N}`, we need the group to match exactly N times; if it doesn't, the cursor is left at the position it had before trying to match `<group>{N}`" — `quantifier_start_position` save+restore on min-failure.

Files touched: `rust/src/ast_pipeline/mod.rs` (`parse_quantifier_bounds` helper + 4 unit tests), `rust/src/ast_pipeline/ast_based_generator.rs` (live codegen path, unified), `rust/src/ast_pipeline/ast_code_generator.rs` (parallel legacy codegen, unified for symmetry). No grammar files touched (Layer 0 is pure engine).

Restore tag (pre-Layer-0 clean): `checkpoint/post-3-3-4-b-1-clean-pre-layer-0` @ `f758b878`. ⛔ NOT pushed (active no-push override per [[feedback_push_pacing]]).

Strengthens [[feedback_prefer_grammar_leave_engine_alone]] (engine changes ARE on the table when they're parser-AGNOSTIC features that make EBNF cleanly express what the language needs) and [[feedback_ast_pipeline_parser_agnostic]] (every pipeline change must be a general primitive usable by any parser). Layer 0 is the model: zero `unsafe`, zero grammar identifiers in production code, every parser benefits.
