<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_question_bypasses_manual_cleanup.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: rust-bypasses-manual-cleanup-is-a-classic-correctness-hazard-iife-try-block-emulation-contains-it
description: "When a function does `std::mem::take`/`replace` (or other mutation that requires manual restoration) and then performs `?`-fallible calls before the manual restore, every `?` early-return JUMPS OVER the restore. The fix is to contain the `?` inside an immediately-invoked closure (\"try-block\" emulation) so it returns into a local `result`, making the manual restore reachable on every non-commit exit. Pinned via full SEMTRACE instrumentation in SV-EXH-PROOF.3.3.3 (`PGEN-SV-EXH-PROOF-0024`) after 3 prior hypotheses (try_parse semantic-checkpoint, grammar-wrapper-alone, memoization-cache) were each disproven."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**Pattern to suspect/look for:**
```rust
fn f(&mut self) -> Result<...> {
    let saved = std::mem::take(&mut self.field);   // self.field now Default/EMPTY
    self.field = saved.clone();                     // working state
    let result = {
        ...
        let x = something(self)?;                   // `?` returns from THE FUNCTION
        ...
        let local = std::mem::take(&mut self.field); // self.field EMPTY again
        ...
        self.fallible(...)?;                        // `?` returns from THE FUNCTION ← JUMPS OVER restore
        ...
    };
    if result.is_err() { self.field = saved; }      // ← DEAD CODE on the `?` paths
    result
}
```
The `if result.is_err()` restore looks correct but is unreachable when any `?` between `take` and the if propagates Err — Rust's `?` returns from the enclosing function, not from the `{ }` block. Result: `self.field` left as the Default value, silently corrupting downstream state.

**Concrete instance (SV-EXH-PROOF.3.3.3, `PGEN-SV-EXH-PROOF-0024`, 2026-05-20):** the generator-emitted `with_semantic_runtime_rule_transaction` in `rust/src/ast_pipeline/ast_based_generator.rs` did exactly this with `self.semantic_runtime_state`. A `?` on `resolve_semantic_predicate_spec_against_content(...)` jumped over the restore, leaving `self.semantic_runtime_state` EMPTY and silently destroying every fact emitted by prior COMMITTED sibling rules. Decisive evidence (SEMTRACE, 1664 trace lines): `RESTORE` fired **0 times**; `took post-body state (self.state→EMPTY)` fired **9 times**. EXPLAINED EVERYTHING that 3 prior hypotheses had failed to fix — context-aware-memo was DISPROVEN before building (the user's "prove it solves the problem first" instruction saved weeks; memo poisoning was real but secondary).

**The fix (idiomatic, zero `unsafe`, parser-agnostic):**
```rust
let result: Result<...> = (|| -> Result<...> {
    ...                       // body unchanged, ?s preserved
})();                         // ? now returns into the closure (= into `result`)
if result.is_err() { self.field = saved; }  // ← NOW REACHABLE on every error path
result
```
This is the canonical "try-block emulation" in Rust. Zero behaviour change on success. Closes every `?`/early-return/explicit-`Err`. Not panic-safe by strict definition (Drop runs during unwind but a non-RAII manual restore does not), but if the take's leftover (`Default`) is a valid state and no `catch_unwind` is reachable mid-execution, the IIFE is panic-ROBUST in practice. (Verify `T::default()` is a valid state for your invariants.)

**RAII Drop-guard alternative (panic-safe by strict definition):** a Drop guard owning the saved value would also fire on unwind. In `with_semantic_runtime_rule_transaction` it was REJECTED because a safe-Rust guard holding `&mut self.field` would conflict with the body's `&self` method calls (whole-struct borrow; no partial borrows across method calls). A true Drop guard would need a contained `*mut` + ~3 lines of `unsafe`. Acceptable in some codebases; deferred here since the IIFE + `Default == valid_state` is robust enough and the parser never `catch_unwind`s mid-parse.

**How to apply:**
1. **Audit:** when reviewing any function that does `std::mem::take`/`std::mem::replace` (or `let x = self.field; self.field = ...;`) followed by `?`-fallible calls before a manual restore, FLAG IT. The restore on the error path is almost certainly dead code on every `?` path.
2. **Fix:** prefer the IIFE/try-block emulation (`let r = (|| -> Result<…> { … })()` ; restore on `r.is_err()`). Idiomatic, no `unsafe`, minimal change.
3. **Diagnose:** if a state-corruption bug looks like "fact A is emitted, persists for a while, then mysteriously vanishes before being read," instrument every mutator of the state (emit/restore/take/rollback) with env-gated stderr trace and look for an UNEXPECTED `take` not followed by the matching restore. Static reading repeatedly mis-predicted this in `.3.3.3`; SEMTRACE pinned it immediately.
4. **Build the proof discipline:** when a fix candidate is proposed, the user-imposed pattern that worked here was: PROVE the fix solves the problem on a decisive cheap experiment BEFORE investing in the elegant implementation. (For `.3.3.3`: disable memoization entirely as the upper-bound "context-aware cache" → my_t [3:0] x STILL failed → cache was disproven as the fix → saved weeks.) See [[feedback_prove_independence_with_decisive_baseline]].

Related memories: [[feedback_prove_independence_with_decisive_baseline]], [[feedback_verify_sv_parser_regen_mtime]], [[feedback_prefer_grammar_leave_engine_alone]] (engine changes ARE on the table when they are parser-AGNOSTIC features that make the EBNF cleanly + elegantly express what the language needs — `.3.3.3` is the model).
