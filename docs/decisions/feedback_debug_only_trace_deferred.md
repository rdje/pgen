<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_debug_only_trace_deferred.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_debug_only_trace_deferred
description: User-set timing constraint (2026-05-24) — trace + counter + dashboard features should be debug-only (excluded from release builds), but this retrofit is deferred until AFTER current parsers are released; not urgent now
metadata:
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**User-set timing (2026-05-24):** "The debug-only is really not urgent. We just need to do that at some point in the future, after all current parser are released, no worries."

**The principle (still true):** trace logger, --trace-rules, --trace-from, per-rule call-counter, live dashboard — none of it should ship in `cargo build --release` parsers. Currently all of it does (the runtime fast-path checks `logger_enabled` / `current_dump_rule_call_counts_top_n()` but the code paths exist).

**The retrofit (deferred):** wrap all instrumentation behind `#[cfg(debug_assertions)]` (cleanest: automatic on debug, gone on release) OR a Cargo feature `parser_instrumentation` for hybrid control. Estimated ~9000+ codegen sites + parser struct fields + accessor methods + parser_registry wiring. Tracked as **Task #59 (.b.6.2.23)** — pending.

**The trigger for unblocking:** "after all current parsers are released" — interpret as: once the SV corpus is 16/16 PASS and an official cut is published (and presumably VHDL + RTL parser families reach their respective close states). Until then, instrumentation stays in release builds; the cost is negligible (~1ns per rule entry for the counter; cached-bool for trace gates) and the diagnostic benefit during the correctness campaign outweighs the principled cleanliness.

**How to apply:**
- Continue landing trace/counter/dashboard work **without** the `#[cfg(debug_assertions)]` gate. Mention in each commit message that "debug-only gating deferred per [[feedback_debug_only_trace_deferred]] → Task .b.6.2.23".
- Don't propose the retrofit as a near-term slice; it's a post-correctness-campaign cleanup.
- Strengthen [[feedback_correctness_before_speed]]: correctness-enabling tooling is in-scope NOW even if it's not the architectural ideal; cleanup follows once correctness is done.
