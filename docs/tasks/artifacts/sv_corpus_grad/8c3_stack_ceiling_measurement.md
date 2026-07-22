# `.8c.3` design-input: the 4096-frame recursion ceiling is DECORATIVE for SV in BOTH build modes (measured 2026-07-22, session #195, `PGEN-SV-CORPUS-GRAD-0014`)

Instrument: `parseability_probe --parse systemverilog <file> --profile verilog_2005`
under controlled main-thread stacks (`ulimit -s`, macOS default 8,192 KB).
Synthetic worst-case construct: `module m; assign x = (…(1)…); endmodule` with
N nested parentheses (each `(` re-enters the full expression cascade).

## Measured matrix

| input | build | stack | outcome |
|---|---|---|---|
| deep-parens N=2000 | debug | 8/16/32 MB | rc 134 stack overflow (ceiling never fired) |
| deep-parens N=2000 | release | 8 MB | **rc 134 stack overflow (ceiling never fired)** |
| deep-parens bisect | debug | 8 MB | N=40 accept / **N=50 CRASH** ⇒ ≈180 KB per paren level |
| deep-parens bisect | release | 8 MB | N=380 accept / **N=400 CRASH** ⇒ ≈21 KB per paren level |
| br_gh330.v (600 ternaries) | debug | 8 MB / 16 MB | CRASH / **ACCEPT rc 0** ⇒ needs 8–16 MB |
| br_gh330.v | release | 1 MB / 2 MB | CRASH / accept ⇒ needs 1–2 MB |

## Findings

1. ⭐⭐ **RELEASE is affected too** — the `.8c.1` framing ("debug-build
   robustness") UNDERSTATED the class: a ~400-deep parenthesized expression
   (≈4 KB of text) hard-aborts the RELEASE process at the default 8 MB main
   stack. The clean `CycleType::MutualRecursive` ceiling
   (`GENERATED_RECURSION_GUARD_MAX_DEPTH = 4096`,
   `rust/src/ast_pipeline/ast_based_generator.rs:31`) NEVER fires for SV:
   release accepted N=380 without firing (⇒ ≤ ~11 logical frames per paren
   level ⇒ ceiling-fire needs N≈410+) and the guard page kills at N≈390 —
   crash-before-ceiling, the exact
   [[feedback_recursion_ceiling_must_bound_real_stack]] scenario (PGEN-RGX-0085's
   law), now proven for the flagship family in the shipping build mode. For an
   embedding signoff parser this is a process-abort class at the integration
   boundary (uncatchable SIGABRT — no `catch_unwind`).
2. Per-frame stack cost (paren worst case, ~10–11 frames/paren): release
   ≈2 KB/frame, debug ≈17 KB/frame (the RGX-0085 debug magnitude).
   Ceiling-real stack need: release ≈ 4096 × 2 KB ≈ 8 MB; debug ≈ 4096 ×
   17 KB ≈ 70 MB.
3. br_gh330's crash row is CONVERTIBLE: debug ACCEPTS at ≥16 MB (rc 0,
   matching release) — with the fix landed, the v2005
   `divergence:unexplained_crash` row re-adjudicates to a match on
   measurement (150 → 149).

## Design (the `.8c.3` fix, per the law's prescription)

Generalize the PGEN-RGX-0085 `GeneratedRegexWorker` model to the SV/all-grammar
parse entries: run generated-parser parses at the INTEGRATION/INSTRUMENT
boundaries (`parseability_probe`, the `ast_pipeline` CLI drivers, the
embedding-API family entries) on a dedicated worker thread with a
**256 MiB stack** (virtual reservation, lazily committed) so the EXISTING 4096
ceiling is PROVEN to fire before the guard page in BOTH build modes with ≥2×
margin (release 8 MB × 2 ≪ 256 MiB; debug 70 MB × 2 ≤ 256 MiB ✓; a deliberate
worst-case deep-parens probe becomes the regression oracle: graceful
recursion diagnostic, NEVER rc 134). Zero hot-path cost (no code inside the
parse loop changes ⇒ zero risk to the regex perf floor law); parser-agnostic
locus (boundary, not grammar); the global ceiling constant stays untouched
(the law forbids lowering a shared bound for one family's stack math).
Rejected alternatives: an SV O(n) boundary nesting pre-check (no sound
bracket proxy — ternary chains nest bracket-free); an in-loop
remaining-stack check (hot-path cost on every family incl. regex —
guardrail risk); lowering the ceiling (rejects legitimate deep files:
br_gh330 is REAL corpus and needs ~1000+ frames).
