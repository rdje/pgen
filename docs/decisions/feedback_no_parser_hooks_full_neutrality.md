---
name: feedback_no_parser_hooks_full_neutrality
description: Director 2026-07-20 RULING — there shall be NO parser hooks; the AST pipeline stays fully parser-neutral/agnostic with no per-grammar side modules; sub-1µs must be met with zero hooks (it already is — no landed speed lever uses them)
metadata:
  node_type: memory
  type: feedback
---

**Standing director ruling (2026-07-20, session #175, verbatim intent):**
"There shall be no parser hooks. Everything should be parser neutral,
agnostic, no hooks on the side. … We should be able to meet the sub-1µs
mark with zero hooks. If we can't achieve that, fine — but I do not want to
resort to hooks or break or violate this AST pipeline being parser
neutral, agnostic rule."

**Why:** per-grammar side modules — even ones plugged in through a generic
registry with zero rule names inside — are the start of the unmanageable
trajectory the project's core intent forbids. Neutrality is not just "no
grammar names in the pipeline"; it is "no per-parser code paths at all."
The director additionally ruled that missing a performance target is
acceptable; violating neutrality to reach one is not (this subordinates
even the RGX-0078 <1µs call-off bar to neutrality — no conflict exists
today: every landed lever is parser-agnostic and the typed-hook emit is
dead code on the measured path, probes byte-identical under fat-LTO).

**Process lesson (owned):** the hook architecture
(`ast_pipeline/parser_hooks.rs` + `rust/src/parser_hooks/regex.rs` +
`--enable-parser-hooks`) was landed during the typed-AST campaign as an
architectural extension point without a prominent director-fork
discussion — a violation of [[feedback_surface_insights_prominently]] /
[[feedback_strategic_fork_pause_reflect]]. Any future "the pipeline needs
an extension point / special case" moment is a MUST-SURFACE fork, before
any code lands.

**Reinforcement (same conversation):** "We have principles and we need to
stick to them, I do not want to compromise, create exceptions." — so the
resolution is FULL REMOVAL, not a preserved-in-neutral-form variant.

**How to apply:** remove it ALL — the `ParserHooks` trait +
`ParserHookRegistry` (`ast_pipeline/parser_hooks.rs`), the
`rust/src/parser_hooks/` tree, the `--enable-parser-hooks` flag, the
per-rule `parse_<rule>_typed` hook surface, and the two consumers built on
it (`regex_typed_differential_gate`, `regex_typed_perf_probe`) — with the
regex integration contract amended (drop `parse_regex_typed()` from the
public API list, version + ledger note per the release policy; the typed
JSON carrier itself is unaffected — the main parse path has produced the
typed `Shaped` carrier since the REPRESENTATION era, byte-proven by the
remaining oracle battery). The only open item is SEQUENCING, not
substance: whether RGX currently calls `parse_regex_typed()` (if yes, the
contract amendment ships with a migration note — the equivalent output is
`parse_full_regex()` + `to_json_value()`/the serialized typed AST; the
removal still happens). The canonical regex artifact returns to the
default emit — identical to every other parser — which also retires the
typed-gate silent-restore trap class and the hooks-form custody special
case ([[project_regex_canonical_artifact_hooks_form]] becomes historical).
Owning tree: `docs/tasks/PARSER-NEUTRALITY.md`. Related assessment:
[[project_parser_hooks_neutrality_assessment]].
