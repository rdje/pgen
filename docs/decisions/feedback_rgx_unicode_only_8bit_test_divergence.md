<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_rgx_unicode_only_8bit_test_divergence.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback-rgx-unicode-only-8bit-test-divergence
description: RGX is Unicode-only by design; PCRE2 8-bit-library-only negative tests (testinput9 #forbid_utf) that RGX "fails" are inherent harness divergences, NOT PGEN or RGX bugs — do not chase them as defects.
type: feedback
---
When triaging a regex/octal/charset conformance divergence where PCRE2 rejects but RGX accepts (or vice-versa), FIRST check the PCRE2 testfile's mode directive. `subs/pcre2/testdata/testinput9` runs **only with the 8-bit library (`#forbid_utf`)**: cases like `testinput9:287 /(?i:A{1,}\6666666666)/` are 8-bit-only negative tests (`\666`=0o666=438>255 errors **only** in 8-bit). RGX is **Unicode-only by design (no 8-bit mode)** so it accepts `\666`=U+01B6 — an **inherent RGX-Unicode-vs-PCRE2-8-bit-library divergence, NOT a PGEN bug and NOT an RGX defect**. The PGEN-correct behavior is mode-agnostic emission (the parser never sees width/UTF); range-by-mode is the mode-aware consumer's call (RGX-0088 prescribed + RGX-confirmed resolution).

**Why:** RGX explicitly adjudicated (2026-05-19) that PGEN-RGX-0088 is genuinely resolved and the testinput9:287 residual is exactly this inherent 8-bit-test-vs-Unicode-engine divergence — net conformance 12,806/4 unchanged (testinput10:218 UTF-octal ↔ testinput9:287 8-bit-octal swap), adoptable, "well done". Chasing such 8-bit-only-negative-test divergences as PGEN/RGX defects wastes effort and risks over-broad mode-blind parse-time rejects (the exact PGEN-RGX-0088 regression).

**How to apply:** for any PCRE2 conformance divergence in `testinput9` (or any `#forbid_utf` / 8-bit-only testfile), classify it as an inherent-mode divergence, NOT a bug, unless RGX explicitly files it. The parser stays mode-agnostic (`feedback_ast_pipeline_parser_agnostic`); the mode-aware consumer owns width/UTF range decisions. See [[project-all-task-trees-complete]] (RGX-0088).
