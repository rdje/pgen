---
name: project_parser_hooks_neutrality_assessment
description: Director 2026-07-20 restated the neutrality doctrine over --enable-parser-hooks; tool-backed assessment — the pipeline holds the doctrine, the per-grammar typed-hook scoping + hooks-form canonical artifact are the residual special case; normalization proposal AWAITING DIRECTOR GO
metadata:
  node_type: memory
  type: project
---

**Trigger (director, 2026-07-20, session #175):** on learning of
`--enable-parser-hooks` (surfaced by the `-0200` custody finding), the
director restated the standing principle in emphatic form: the parser
generation flow shall be fully agnostic and neutral — no logic specific to
any given parser; "if you were forced to use specific logic it would mean
that we failed the project intent and goal."

**Tool-backed assessment (same session, all claims from source):**

1. **The pipeline holds the doctrine.** `rust/src/ast_pipeline/` names no
   grammar. The hook architecture (`ast_pipeline/parser_hooks.rs`) exists
   precisely to keep it that way: handlers live OUTSIDE the pipeline
   (`rust/src/parser_hooks/<grammar>.rs`), register at the binary boundary
   only under the opt-in `--enable-parser-hooks` flag, are looked up by
   grammar name through a registry, and with no handler registered the
   pipeline's default emit is byte-identical for every parser (contract
   stated in the module docs and pinned by registry tests).
2. **No parsing/language logic is parser-specific.** The regex language
   definition is 100% the EBNF; the generated parser is complete and
   correct with the flag off. The one existing handler
   (`rust/src/parser_hooks/regex.rs`, 917 lines) emits per-rule
   `parse_<rule>_typed` entry points that DELEGATE to the legacy methods
   plus `to_json_value()` — mechanical, rule-name-agnostic boilerplate
   (zero hardcoded rule names; the only grammar-specific fact in the file
   is the scoping string `"regex"`). Nothing was "forced": the hook is an
   API-surface add-on (typed JSON entry points for the RGX integration +
   the typed differential oracle), not compensating logic.
3. **The residual special case is real and is configuration, not logic:**
   (a) the typed-entry surface is generic boilerplate scoped to one
   grammar — against the 2026-06-08 doctrine
   [[feedback_features_parser_agnostic_enable_all_parsers]] (one-parser
   features → parser-agnostic, capability-gated, all parsers);
   (b) the canonical on-disk regex artifact is the hooks-form emit
   (`parse_regex_typed()` is promised public API in the regex integration
   contract) while all other parsers' canonicals are the default emit — a
   per-parser build-config fork that already cost the `-0200` session a
   custody detour and historically arose from the typed-differential
   gate's silently-failing restore rather than a deliberate decision
   ([[project_regex_canonical_artifact_hooks_form]] /
   `docs/tasks/artifacts/recursion_id_only/design_prereg.md` §5b);
   (c) the next parser wanting a typed surface would need its own
   ~900-line handler module — the exact unmanageability the director
   refuses.

**Proposal (ONE recommendation, AWAITING DIRECTOR GO — no code changed):**
promote the typed-entry surface to a **generic, capability-gated pipeline
emission for every parser** — emit `parse_<rule>_typed` uniformly from the
pipeline (it names no grammar), retire `rust/src/parser_hooks/regex.rs`,
the registry special case, and the `--enable-parser-hooks` flag, and make
the canonical artifact form uniform across all 11 parsers. This
simultaneously: satisfies the neutrality doctrine at the configuration
level, keeps the regex contract promise (`parse_regex_typed` still exists
— every parser gains its analogue), removes the typed-differential gate's
regen/restore dance (and with it the tracked silent-restore trap class),
and ends the hooks/no-hooks canonical fork. Costs to price before landing:
artifact-size growth per parser (regex gained 769 typed occurrences; SV
will be the large one), one re-baseline wave, and a fat-LTO probe
byte-identity re-verification (the #140 session recorded the typed emit as
dead code stripped by fat-LTO — probes byte-identical from both vintages —
re-prove at land time). Fallback direction if the director prefers
retirement instead: withdrawing `parse_regex_typed()` from the public
contract is an external-surface decision and stays director-owned.

**Status:** proposal recorded; owning leaf to be opened on director GO (a
fresh session per the one-fix directive; the change is emitter+regen-wave
scale). The `-0200` interim mitigation (regen trains end with the hooks
re-emit, step 5b) remains the operative recipe until then.
