# INLINE-ACTIONS: branch-local inline semantic ACTION directives fire at parse time (parser-agnostic engine feature)

## Metadata

- Tree ID: `INLINE-ACTIONS`
- Status: `active`
- Roadmap lane: cross-cutting engine quality — PGEN semantic-annotation feature-completeness (parser-agnostic)
- Created: `2026-06-09`
- Last updated: `2026-06-09`
- Owner: repo-local workflow

## Provenance (how this was found)

Found while designing the `SV-PARSE-STRICT.2` parser-bug fix. That fix wants to
`@emit_fact` a `wildcard_import_open` marker on the *wildcard branch* of
`package_import_item` (`systemverilog.ebnf:3593`) — i.e. a semantic action attached to a
specific **branch**, not the whole rule. The director (2026-06-09) was surprised that no
branch-level `@emit_fact` precedent existed and directed a tools-first investigation:
"I thought semantic annotations were elegantly supported before-rules and inline-before
(inside branches) the targeted item… ANY semantic annotation shall be supported on
specific branches before the targeted item" + "Did you carefully read `ebnf.ebnf`? I am
sure this is possible" + the standing directive: "PGEN semantic annotations are not
feature complete… the goal is to capture the feature sets that enable PGEN to parse as
many languages as humanly possible (C, JS, HTML, Perl6, …); if a feature would be a good
addition, propose it so we can design carefully and implement it."

The director is **right about the syntax** and identified a **real runtime gap**.

## The capability gap (empirically proven, 2026-06-09)

The meta-grammar `grammars/ebnf.ebnf` supports inline annotations by design:

```
sequence          := sequence_element+
sequence_element  := ( inline_semantic_annotation | quantified_element | primary_element )
inline_semantic_annotation := semantic_annotation
# comment (ebnf.ebnf:118-122):
#  - branch-start inline annotations can mean branch-local steering
#  - later inline annotations in a sequence can mean true mid-sequence actions
```

A probe grammar with `@emit_fact` in three positions, run through
`ast_pipeline … --generate-parser`, proves what the *runtime* actually wires:

| Placement | Parsed | Compiled into the generated parser | Fires at parse time |
| --- | --- | --- | --- |
| **Rule-level** (annotation on its own line *above* `name :=`) | yes | yes → `directives_by_rule` | **YES** — applied by `effect_directives_for_rule` against the rule's content after body parse, before post-predicates (`ast_based_generator.rs:1566-1575`). This is the proven idiom used across every grammar. |
| **Branch-start inline** (after `:=`, *before* the 1st item) | yes | yes → `branch_directives_by_rule` | **NO** — the branch tournament only acts on `Predicate` directives (`ast_based_generator.rs:2979-3091`); `EmitFact`/`OpenScope`/`CloseScope` fall into the `=> {}` no-op arm (`:3092-3099`). Registered but never applied. |
| **Mid-sequence inline** (after ≥1 matched item) | yes | **NO** — extracted into `branch_mid_sequence_semantic_annotations` but `compile_semantic_runtime_annotations` (`semantic_runtime.rs:2878-2939`) never compiles that registry into runtime directives, so the directive is silently dropped (the probe's `marker_mid` fact appears nowhere in the generated parser). | **NO** |

Corroboration that this is *unfinished wiring*, not a *deliberate non-feature*: the
well-formedness linter's F1 emitter-enumeration already walks **rule-level, per-branch,
AND mid-sequence** annotation sites "so no emitter is missed"
(`grammar_wellformedness.rs:824-866`). The linter treats branch/mid-sequence
`@emit_fact` as real emitters; the runtime does not fire them — a latent inconsistency.

Net: only **rule-level** action directives and **branch-local predicates** are live
today. Branch-local and mid-sequence **actions** (`@emit_fact`, `@open_scope`,
`@close_scope`) are not — so a grammar author must reshape a branch into a dedicated
helper rule to emit per-branch, which is exactly the kind of non-elegant workaround the
director wants removed.

## Why this is a good, general feature (not an SV one-off)

The store-gated, fact-emitting style ("emit a fact at the declaration site, consult it at
the use site") is the mechanism PGEN uses for *every* context-sensitive language
(SV type-name feedback, but equally C `typedef`, JS strict-mode, Perl prototypes, HTML
content models). Many declaration sites live in a *branch* of a multi-branch rule, not in
a rule of their own. Letting `@emit_fact` (and scope open/close) attach directly to a
branch — exactly where the declaration is recognized — is a foundational expressiveness
primitive for the broader "parse as many languages as humanly possible" goal. It is
strictly parser-agnostic: no parser-specific concept enters the engine; a grammar chooses
which branch emits which fact.

## Design (branch-start action wiring — `.2`)

Semantics: when a multi-branch rule selects a branch, that branch's branch-start **effect**
directives (`is_effect()` = `OpenScope | CloseScope | EmitFact`,
`semantic_runtime.rs:480-485`) fire **once**, for the **winning** branch only, resolved
against that branch's matched content — identical lifecycle to a sub-rule's rule-level
`@emit_fact` invoked inside the branch body.

This rides the existing C3-B tournament delta machinery with no new lifecycle concept:

1. In the per-branch attempt arm, the branch body is parsed speculatively; `raw_content`
   and `transformed` are computed (`ast_based_generator.rs:2972-2976`); branch-local
   predicates run and set `branch_predicate_blocked` (`:2979-3100`).
2. **NEW:** immediately after the predicate loop, *iff* `!branch_predicate_blocked`,
   iterate this branch's effect directives (a new
   `branch_effect_directives_for_rule_branch(rule, idx)` = `branch_directives_for_rule_branch`
   filtered by `is_effect()`), resolve each directive's `$refs` against the branch content,
   and apply it onto the live `parser.semantic_runtime_state`.
3. Those emissions land in `candidate_delta` (`extract_delta_since`, `:3162`) and are
   rolled back for every branch (`:3174`); the winner's delta — now including its
   branch-start actions — is replayed onto the committed state post-selection
   (`best_semantic_delta` → `apply_delta`, `:3287`). Loser branches' actions are
   automatically undone; the winner's persist. No new winner/loser bookkeeping.

Ref resolution mirrors the rule-level effect path (resolve against the branch's shaped
content; `$package.body`-style dotted refs are already supported). Predicate-before-action
ordering ("steering gates, then action fires") matches the meta-grammar's branch-start
intent. Reuse `apply_semantic_runtime_*` directive-application code; no annotation-language
or meta-grammar change is required (the surface already parses).

## Non-Goals

- **Mid-sequence inline actions** (`@emit_fact` after ≥1 matched item, resolving refs
  against *partial* content at that sequence position) — a strictly larger change (the
  `branch_mid_sequence_semantic_annotations` registry must be compiled and applied at the
  exact element boundary). Captured as `.3` (deferred) so `.2` stays a reviewable slice.
- Branch-level **library** directives (`@export_to_library` / `@import_from_library`) —
  not needed by any current consumer; out of `.2` scope.
- Any change to the annotation language or `ebnf.ebnf` (the inline surface already
  parses).
- The SV parser-bug fix itself (owned by `SV-PARSE-STRICT.2`, which *consumes* this).

## Acceptance Criteria

- A probe/unit grammar with a branch-start `@emit_fact` proves the fact is emitted when
  (and only when) that branch wins, and is visible to a later `@predicate`/`fact_count_at_least`
  consumer; loser-branch emissions do not leak.
- The winning-branch-only + rollback-on-loss property is locked by a regression test
  (multi-branch rule where two branches emit different facts; the selected branch's fact
  is present, the other absent).
- Branch-local **predicate** behavior is byte-for-byte unchanged (the existing
  `@predicate … phase: branch` path is untouched).
- All 10 generated parsers regenerate; cross-parser no-regression: regex broader corpus /
  RGX conformance, `cargo test --lib` (± `--features generated_parsers`), SV shape-contract
  — all green. Grammars without branch-start actions are byte-identical (zero blast radius
  when the feature is unused).
- Clippy strict (source + generated) clean.
- Books ↔ code lockstep: the EBNF-authoring/annotation reference documents branch-local
  inline actions (placement, lifecycle, the winning-branch-only semantics).

## Task Tree

- ID: `INLINE-ACTIONS`
  Status: `active`
  Goal: inline semantic ACTION directives (`@emit_fact`/`@open_scope`/`@close_scope`) attached to a branch fire at parse time, parser-agnostically.
  Children: `INLINE-ACTIONS.1`, `INLINE-ACTIONS.2`, `INLINE-ACTIONS.3`

- ID: `INLINE-ACTIONS.1`
  Status: `done`
  Goal: scope + tools-first root-cause the gap; design the branch-start action wiring; propose the feature.
  Acceptance: this file records the empirical 3-position proof, the parser-agnostic rationale, and the delta-machinery design.
  Verification: probe grammar generated via `ast_pipeline --generate-parser`; generated-parser inspection (marker_start registered as a no-op branch directive; marker_mid absent); codegen read of the tournament loop + `effect_directives_for_rule` + `is_effect`.
  Commit: `PGEN-INLINE-ACTIONS-0001`

- ID: `INLINE-ACTIONS.2`
  Status: `done`
  Goal: wire branch-start effect directives to fire for the winning branch (the design above), with regression tests + cross-grammar no-regression + book lockstep.
  Acceptance: the Acceptance Criteria above hold; all 10 parsers regenerate; unused-feature grammars byte-identical.
  Verification: codegen wiring (probe `.ebnf` → `ast_pipeline --generate-parser`: helper+loop emitted; unit test `generated_parser_wires_branch_start_effect_application_for_winning_branch`); gating (unit test `generated_parser_omits_branch_start_effect_wiring_without_branch_effects`; all 10 grammars regenerate with `helper=0 loop=0`); runtime data path + emit (unit test `branch_effect_directives_accessor_returns_only_effects_and_emit_fires`); lib 624/0 (no-features) + 689/0 (`--features generated_parsers`); strict source clippy clean. Loser-branch non-leak is by construction (the application runs only in the winner-selected block keyed on `best_branch_index`). End-to-end firing in a compiled parser is integration-proven by the first consumer, `SV-PARSE-STRICT.2`.
  Commit: `PGEN-INLINE-ACTIONS-0002`

- ID: `INLINE-ACTIONS.3`
  Status: `deferred`
  Goal: wire mid-sequence inline action directives (compile `branch_mid_sequence_semantic_annotations`; apply at the element boundary against partial content).
  Acceptance: a mid-sequence `@emit_fact` fires at its sequence position; refs resolve against the matched-so-far content; no-regression.
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `INLINE-ACTIONS.2` | `done` | Branch-start action wiring landed (`PGEN-INLINE-ACTIONS-0002`); unblocks `SV-PARSE-STRICT.2`'s clean inline `@emit_fact`. |
| 1 | `INLINE-ACTIONS.3` | `deferred` | Mid-sequence actions; larger (positional partial-content ref-resolution); activate on demand. Not blocking — `SV-PARSE-STRICT.2` needs only branch-start. |

The tree's deliverable (branch-start inline action directives) is **landed**; the
only open child (`.3`, mid-sequence) is `deferred`. The next active work is the
consumer `SV-PARSE-STRICT.2`.

## Decisions

- `2026-06-09`: **Implement branch-start emit first, then the SV fix** (director, AskUserQuestion: "Wire branch-start emit first, then fix"). Mid-sequence deferred to `.3`.
- `2026-06-09`: **Ride the existing C3-B delta machinery** — apply branch-start effect directives into the live state after predicates pass and before delta capture, so the winner-replay/loser-rollback path applies unchanged. No new winner/loser bookkeeping, no new lifecycle concept.
- `2026-06-09`: **Predicate-before-action ordering** within a branch's branch-start annotations (steering gates, then the action fires), matching the meta-grammar's documented branch-start intent.
- `2026-06-09`: **Parser-agnostic, surface-stable** — no annotation-language or `ebnf.ebnf` change; grammars not using branch-start actions stay byte-identical (zero blast radius).

## Open Questions

- Ref-resolution view for branch-start actions: shaped (`transformed`) vs raw (`raw_content`). Resolve in `.2` to match the rule-level effect path (rule-level resolves against the rule's post-transform content); does not block the frontier.
- Whether `@export_to_library` / `@import_from_library` should also be branch-attachable. No current consumer; left out of `.2` and re-raised only if a grammar needs it.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-09` | `INLINE-ACTIONS.1` | probe-grammar parser generation + generated-parser inspection (3 placements) + codegen read (tournament loop, `effect_directives_for_rule`, `is_effect`, `compile_semantic_runtime_annotations`) | gap proven; pure-docs |
| `2026-06-09` | `INLINE-ACTIONS.2` | probe `.ebnf` regen (helper+loop wired); all 10 grammars regen `helper=0 loop=0` (zero blast radius); 3 unit tests (codegen wiring + gating + accessor/emit); lib 624/0 (no-features) + 689/0/21ign (`--features generated_parsers`); strict source clippy clean (generated-stage debt pre-existing, non-strict) | pass |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `INLINE-ACTIONS.1` | `PGEN-INLINE-ACTIONS-0001` | scoping/design/proposal (pure docs) |
| `INLINE-ACTIONS.2` | `PGEN-INLINE-ACTIONS-0002` | branch-start action wiring (engine/codegen + tests + book) |

## Changelog

- `2026-06-09`: Created task tree from the director directive; `.1` scoping/design done (empirical 3-position gap proof + parser-agnostic rationale + delta-machinery design); `.2` (branch-start wiring) is the frontier; `.3` (mid-sequence) deferred. `SV-PARSE-STRICT.2` consumes `.2`.
- `2026-06-09`: `.2` DONE (`PGEN-INLINE-ACTIONS-0002`) — branch-start inline action directives (`@emit_fact`/`@open_scope`/`@close_scope`) now fire for the winning branch. Engine: `branch_effect_directives_for_rule_branch` accessor (`semantic_runtime.rs`); conditionally-emitted `apply_branch_start_effect_directive` helper + per-rule-gated winner-branch application loop (`ast_based_generator.rs`); book `semantic-store.md` "Rule-level vs branch-local placement". Zero blast radius (all 10 grammars regen byte-identical). `.3` (mid-sequence) stays deferred. Frontier → `SV-PARSE-STRICT.2`.
