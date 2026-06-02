<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_sv_parser_pause.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: SV parser pause LIFTED 2026-05-04 — SV is the active lane
description: User explicitly redirected focus 100% to systemverilog parser on 2026-05-04. SV mdbook + integration contract + return-annotations campaign are required deliverables.
type: feedback
originSessionId: f74f4acc-7183-408a-ae5d-dcdce15a103c
---
The user lifted the prior SV-parser pause on 2026-05-04 with the directive: "Now let's focus 100% on the systemverilog parser." Active lane is now SV, not RGX-0073.

**Why:** The user expanded scope: every generated PGEN parser (systemverilog, vhdl, rtl_*, ...) needs the same deliverables the regex parser got — an mdbook (under `docs/<parser>_parser_book/`), an integration contract (`docs/contracts/PGEN_<PARSER>_PARSER_INTEGRATION_CONTRACT.md`), an RGX-style handoff/explainer for downstream consumers, and a systematic return-annotations campaign on the EBNF grammar. SV is first in line because Nexsim depends on it.

**How to apply:**
- SV parser work is now PERMITTED and EXPECTED. Run `parseability_probe --parse systemverilog`, `sv_syntax_closure_gate`, `sv_stimuli_quality_gate`, etc. as needed.
- Create `docs/systemverilog_parser_book/` mirroring `docs/regex_parser_book/`'s structure (build-recipe, ast-envelope, json-carrier, schema-versioning, changelog-index, glossary, plus per-rule and per-example chapters).
- Create `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` mirroring the regex contract (Contract Identity, Highlights sections per release, etc.).
- Apply return-annotations slice-by-slice to `grammars/systemverilog.ebnf` and bump contract identity per slice, same pattern as the regex campaign (slices 1-42+).
- Live-book policy carries over: every SV grammar/shape/AST/surface change must update the SV mdbook in the same commit.
- The same RGX-targeted handoff explainer pattern applies: when an SV downstream consumer (Nexsim or other) files an issue or asks about a number, the changelog-index entry should be self-contained enough that they can land on it and understand the rationale without external context.
- After SV is done, repeat for VHDL and RTL parsers.
