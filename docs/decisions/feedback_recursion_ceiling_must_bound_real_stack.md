<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_recursion_ceiling_must_bound_real_stack.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback-recursion-ceiling-must-bound-real-stack
description: A recursion-depth ceiling that returns a clean error is useless if its bound exceeds the stack the recursion actually runs on — the OS guard page faults (uncatchable SIGABRT) before the clean ceiling fires. Bound it to the real stack, or pre-check at the integration boundary before recursion starts.
metadata:
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**Lesson (PGEN-RGX-0085, 2026-05-18):** PGEN's generated parser had a
`RecursionGuard` that *was* mechanically correct — at
`parse_stack.len() >= MAX` it returned a clean
`Err(ParseError::RecursionDepthExceeded)`. Yet deeply nested input
still **hard-aborted the host process** (stack-overflow → SIGABRT,
uncatchable even under `catch_unwind`). Root cause: `MAX = 4096`
parse-stack frames ≈ 512 regex nesting levels, but a debug build
consumes ~128 KB of stack per nesting level, so even the dedicated
64 MiB worker thread held only ~512 levels — the OS stack guard page
faulted **before** `parse_stack.len()` ever reached 4096. The clean
ceiling existed but could never fire. A separate fast-path also ran
"shallow" nesting inline on an unknown (possibly 2 MiB) caller stack
behind a comment ("well under 1 MB in debug") that was empirically
false.

**Why:** a recursion ceiling is a *stack-safety* mechanism, not just
a config knob. Its only job is to fire before the OS guard page. If
`MAX × worst-case-per-frame-bytes > smallest-stack-it-runs-on`, the
ceiling is decorative — the process dies first. "There is a guard and
it returns a clean error" is necessary but not sufficient; the bound
must be *proven* against the real minimum stack and the real
debug-frame cost (debug frames are vastly larger than release).

**How to apply:**
- For DoS-grade / untrusted input, prefer a **cheap O(n) structural
  pre-check at the integration boundary, before any recursion
  starts** (count nesting; reject over-limit with a clean located
  diagnostic). This is PCRE2's model (`PCRE2_CONFIG_PARENSLIMIT`,
  build-time `--with-parens-nest-limit`) and makes a stack overflow
  *structurally impossible* (the recursive code never runs on bad
  input) — far more robust than relying on the in-recursion guard.
- If you do rely on an in-recursion ceiling, pick its value so
  `MAX × measured-debug-per-frame < smallest-supported-stack` with a
  ≥2× margin; verify it on a deliberately small (libtest ~2 MiB)
  thread so a regressed/guardless build *deterministically aborts*
  the test instead of falsely passing on a large main stack.
- Default such limits to the ecosystem standard when one exists
  (PCRE2 / Rust `regex` parenthesis-nest default = 250) — defensible,
  documented, and almost certainly what the downstream report
  references.
- Fix at the **parser-agnostic-correct locus**: a regex-specific
  embedding-boundary ceiling does not touch the *global* engine
  recursion guard ⇒ zero risk to other grammars (SV/VHDL). Don't
  lower a shared global bound to fix one family's stack math.
- A guardless recursive descent + recursive `serde_json`
  serialization of its deep output are *two* unbounded paths of the
  same call — bounding the input nesting bounds both at once
  (no over-deep AST is ever built to serialize).

Related: [[feedback_prove_independence_with_decisive_baseline]]
(root-cause from the artifact/code, not assumption — here the guard
"existed" so the lazy conclusion "it has a guard, must be elsewhere"
would have been wrong; reading the wired value `4096` vs the stack
math was decisive), [[feedback_ast_pipeline_parser_agnostic]]
(engine-vs-boundary locus choice), [[feedback_corpus_expected_from_spec_not_fix]]
(verification expecteds from the PCRE2/`regex` spec, not the fix).
