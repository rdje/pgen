<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_prefer_grammar_leave_engine_alone.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: prefer-ebnf-grammar-fixes-leave-the-stable-engine-alone
description: "STANDING PREFERENCE (user, 2026-05-19, stated emphatically twice): the codegen/runtime ENGINE is stable and considered off-limits; fix parsing defects in the .ebnf grammar files. Only escalate to an engine change as a true last resort, and only with explicit user authorization + strict scoping."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**User standing preference (2026-05-19, said twice with delight): "we won't have to touch the engine then, great!" / "The engine is pretty stable. We just need to play with the EBNF files and leave the engine alone!"**

**Why:** the shared codegen/runtime engine is mature and stable; engine changes carry cross-parser blast radius and regression risk. Grammar (`*.ebnf`) edits are localized, reviewable, and parser-scoped. The user actively prefers a grammar-only solution even when an engine fix looks "cleaner," and is visibly happier when a defect is resolved purely in the EBNF.

**How to apply:**
- For any parsing defect, find a `.ebnf`-only fix FIRST and exhaust grammar options before even proposing an engine change. Mirror proven in-grammar idioms (e.g. object-reconstruction `-> {body: $N.body}` instead of raw `-> $N` passthrough — the SV-EXH-PROOF.3.3.1 `non_keyword_identifier` fix that worked without any engine change).
- Treat an engine/codegen change as a genuine last resort: only when NO grammar form works (proven empirically, not assumed), and only with explicit user authorization, and strictly scoped (e.g. lookahead-only) with full cross-parser no-regression.
- This reinforces and strengthens [[feedback_ast_pipeline_parser_agnostic]]: not just "engine changes must be parser-agnostic" but "default to NOT changing the engine at all."
- Don't pause to ask which route when a grammar fix is available — just do the grammar fix. Surface the engine option only if grammar is empirically exhausted.

Evidence it works: SV-EXH-PROOF.3.3.1 — the entire `non_keyword_identifier`-routed declared-id directive family (class/typedef/parameter/localparam) was fixed by a single one-line grammar change (`non_keyword_identifier := !reserved_non_keyword_identifier identifier -> {body: $2.body}`), zero engine edits.
