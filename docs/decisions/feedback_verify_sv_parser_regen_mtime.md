<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_verify_sv_parser_regen_mtime.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: verify-sv-parser-regen-actually-happened-mtime-behavioral-oracle-don-t-trust-make-exited-0
description: "`make focus_systemverilog` can silently skip the slow parser-generation step and exit 0 while generated/systemverilog_parser.rs stays STALE relative to grammars/systemverilog.ebnf. ALWAYS assert parser mtime > grammar mtime (or see a real behavioral change) before trusting any probe result after an .ebnf edit."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**SV-EXH-PROOF.3.3.3 lost two ~5-min rebuild/test cycles to this:** after editing `grammars/systemverilog.ebnf`, ran `make -C rust focus_systemverilog >/dev/null 2>&1 && echo "regen done"` then `cargo build … parseability_probe`. `make` exited 0 ("regen done" printed) but `generated/systemverilog_parser.rs` (`09:12:36`) was OLDER than the edited grammar (`09:47:52`) — the slow `ast_pipeline --generate-parser` step did NOT run, so the probe tested the PREVIOUS leaf's parser. Both `.3.3.3` candidates ("$type_identifier.body fails", "$type.body fails") were therefore UNTESTED/invalid conclusions.

**Why it happens:** `focus_systemverilog`'s parser-generation recipe is slow; the json (`generated/systemverilog.json`) regenerates fast and can end up NEWER than the grammar while `generated/systemverilog_parser.rs` stays stale. `make` exiting 0 means "make ran", NOT "the parser was regenerated". `echo "regen done"` after `&&` only proves make's exit code.

⛔ **STILL LIVE, AND HARDER TO SPOT SINCE 2026-08-14.** The original note added *"and emits huge `--debug --trace` output"*; `CI-PARITY-GATE-ROT.31` removed those flags from the shipping recipe (6.89 GB → 4 838 B per full regeneration), so **volume is no longer a tell**. The trap itself is unchanged and was re-observed the same day while measuring that very fix: a `touch` landing **27 ms** after the previous run wrote the parser left make judging the target up to date, and `focus_json` returned exit 0 having generated nothing. It was caught only because the byte count was implausibly small. ⇒ the mtime assertion below is now the *only* cheap tell; do not substitute "the log looked busy" for it.

**How to apply (mandatory before trusting any post-`.ebnf`-edit probe/gate result):**
- After regen, ASSERT freshness: `[ $(date -r generated/systemverilog_parser.rs +%s) -gt $(date -r grammars/systemverilog.ebnf +%s) ]` — abort/redo if not newer. Bake this assertion into the regen+build+test command so a stale parser hard-fails instead of silently passing.
- The DECISIVE proof a regen took is a **behavioral change** in the probe (a FAIL→PASS or changed `--trace` failure locus). If behavior is byte-identical to the previous leaf, suspect a stale parser FIRST — re-verify mtime — before concluding "the fix didn't work" (this is the [[feedback_prove_independence_with_decisive_baseline]] discipline applied to the regen step: don't assume the toolchain did the work, verify it).
- Conversely: `.3.3.1` (corpus 4→6) and `.3.3.2` (`package pp;` FAIL→PASS) are TRUSTWORTHY precisely because they showed real behavioral changes a stale parser could not produce.
- Prefer running the regen with output visible enough to see the `Generating systemverilog parser from JSON...` line actually execute; don't `>/dev/null` the one signal that tells you it ran.
