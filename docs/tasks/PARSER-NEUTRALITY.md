# PARSER-NEUTRALITY: remove the parser-hook mechanism — full AST-pipeline neutrality, no exceptions

## Metadata

- Tree ID: `PARSER-NEUTRALITY`
- Status: `active` (created 2026-07-20, session #175, by direct director ruling)
- Roadmap lane: pipeline governance / architecture integrity
- Created: `2026-07-20`
- Owner: repo-local workflow
- Priority: **DIRECTOR-RULED (2026-07-20): "There shall be no parser hooks. Everything
  should be parser neutral, agnostic, no hooks on the side. … I do not want to
  compromise, create exceptions."** Durable ruling:
  `docs/decisions/feedback_no_parser_hooks_full_neutrality.md`; tool-backed
  assessment: `docs/decisions/project_parser_hooks_neutrality_assessment.md`.

## Goal

Remove the parser-hook mechanism and everything built on it, restoring the
invariant that the ONLY per-parser inputs to generation are the grammar
(EBNF + annotations) and generic capability/census gates computed from
grammar structure — zero per-grammar code modules, zero per-grammar build
configuration:

1. `ast_pipeline/parser_hooks.rs` (`ParserHooks` trait + `ParserHookRegistry`) — REMOVE.
2. `rust/src/parser_hooks/` (the regex handler module tree) — REMOVE.
3. `--enable-parser-hooks` CLI flag + the binary-boundary registration in `main.rs` — REMOVE.
4. The emitted per-rule `parse_<rule>_typed` hook surface — REMOVE (the typed
   JSON carrier is unaffected: the main parse path has produced the typed
   `Shaped` carrier since the REPRESENTATION era, byte-proven by the
   remaining oracle battery).
5. `regex_typed_differential_gate` + `regex_typed_perf_probe` binaries, their
   Makefile targets, feature entries, and battery/step references — RETIRE
   (their oracle role is covered by the ALL-11 interpreter↔generated
   equivalence gate, the shape contract, duality, and the PCRE2 oracle).
6. Regex integration contract: drop `parse_regex_typed()` from the public
   API list with a version bump + released-parser ledger note per the
   release policy; include the migration note (equivalent output =
   `parse_full_regex()` + `to_json_value()` / the serialized typed AST).
   **External-surface step — director-owned sign-off on the published
   amendment text.**
7. Canonical regex artifact returns to the DEFAULT emit — identical build
   configuration to every other parser. This retires the typed-gate
   silent-restore trap class, the hooks-form custody special case, and the
   `-0200` regen-train step 5b interim rule.
8. Books/docs lockstep: top-level book (pipeline architecture chapters
   mentioning hooks), regex parser book/changelog, contracts, TOOLBOX/
   reference cards, and the `-0200` evidence cross-references (marked
   historical, not rewritten).

## Non-goals / constraints

- No behavior change to any parser's accepted language, AST, errors, or
  performance floor (the typed emit is dead code on the measured path;
  probes proven byte-identical under fat-LTO at #140 — re-prove at land
  time).
- The RGX-0078 speed campaign is UNAFFECTED and remains hook-free by
  construction; the <1µs bar is subordinate to neutrality by the ruling
  (no conflict exists).
- One open SEQUENCING input (not a design fork): whether RGX currently
  calls `parse_regex_typed()` — determines only whether the contract
  amendment ships with an active-migration note. Director to confirm.

## Leaves

### `PARSER-NEUTRALITY.1` — hook-mechanism + typed-surface removal (status: `done`, session #175 — director ordered same-session execution; owns items 1–5, 7)
- Executed same-session on the director's direct order ("simply remove
  them"), which supersedes the fresh-session default for this non-perf
  removal. Scope delivered exactly as specced; evidence
  `docs/tasks/artifacts/parser_neutrality/removal_verification.md`.
- `--emit-typed-entry-skeleton` deliberately NOT touched (pipeline-internal,
  grammar-agnostic, no per-parser anything) — queued as `.3` below instead
  of silently widening this slice.

##### Acceptance Checklist (enforced) — hook-mechanism + typed-surface removal (`PGEN-PARSER-NEUTRALITY-0002`, leaf PARSER-NEUTRALITY.1)
- [x] **REPRODUCE / ISSUE** — the director ruling of 2026-07-20 (recorded
  verbatim-intent in `docs/decisions/feedback_no_parser_hooks_full_neutrality.md`):
  the `--enable-parser-hooks` mechanism surfaced by the `-0200` custody
  finding constitutes a per-parser side surface (one registered handler,
  `rust/src/parser_hooks/regex.rs`; the hooks-form canonical regex
  artifact diverging from all other parsers' default emit) — forbidden
  regardless of the pipeline's own internal neutrality.
- [x] **ROOT CAUSE (WHY + WHERE)** — tool-backed assessment
  (`docs/decisions/project_parser_hooks_neutrality_assessment.md`): the
  pipeline itself names no grammar and the handler hardcodes zero rule
  names, but the mechanism's EXISTENCE creates per-grammar modules and a
  per-parser build-configuration fork — the exact unmanageable trajectory
  the project intent forbids. WHERE, tool-pinned: the artifact-form fork
  was exposed by the `-0200` regen train's expected-delta review (the
  hooks emit = 769 `parse_*_typed` occurrences vs 0 in the `focus_regex`
  emit) and re-proven here by the emission byte-compares; the behavioral
  ground truth is re-certified on the hook-free vintage via
  `--report-certificate-coverage` (seeds 0/7/42, byte-exact
  `CERTIFICATE-COVERAGE: 268/9/259/0 fully_certified=true`,
  `sample_parse_failures=0`) and the `regex_perf_probe` full-corpus sweep
  (0/2,189 verdict flips vs the banked `-0200` per-cell JSONL). Consumers audited: in-repo only the two
  typed verification binaries; the contract's `parse_regex_typed()`
  mentions are all historical release notes (zero live normative
  promise, verified by line-range grep).
- [x] **FIX / ADJUDICATION** — full removal, no compromise (the ruling's
  reinforcement): trait/registry/context, the handler tree, the CLI flag
  and binary-boundary registration, the generator threading
  (`parser_hook_registry` + `ebnf_grammar_name` + `extend_parser_impl` +
  splice + the `_with_hooks` variant), both typed binaries with their
  Cargo/Makefile surface (tombstone note left in the Makefile). The
  canonical regex artifact returns to the DEFAULT emit — identical build
  configuration for all 11 parsers.
- [x] **ADDRESSED (verified)** — `grep -r "parser_hooks|ParserHook|
  enable-parser-hooks|regex_typed_"` over `rust/src` + `Makefile` +
  `Cargo.toml` returns only the tombstone/doc-comment mentions; the regex
  artifact carries 0 `parse_*_typed` occurrences and its hook-free regen
  is byte-identical (`1a5f7018…`) to the hook-capable tool's no-flag emit;
  `json`/`ebnf` control regens byte-identical modulo own-path strings —
  the removal changed nothing but the extension point itself.
- [x] **NO REGRESSION** — per-file compiler-warning counts IDENTICAL
  (stash-based before/after; the only delta is the deleted handler's own
  warning). Performance neutrality proven empirically (the `#140`
  probe-byte-identity precedent does NOT transfer — the lib changed — so
  measured instead): bench floorval **+1.21%** vs the banked 1,818.3 ns
  and full-corpus sweep **+1.61%** vs the `-0200` candidate — both inside
  the ≈2.3% noise span — with verdict flips **0/2,189** and MAX 474,792 ≤
  settled 483,583 ns; the ACCEPTED floor (1,153.5 ns) unchanged. Battery
  green at the changed vintage minus the retired gate: dual-feature lib
  suite (incl. the ALL-11 interpreter↔generated oracle), cert ×3 seeds,
  shape, duality, PCRE2 compile oracle, clippy source-strict — see
  `docs/tasks/artifacts/parser_neutrality/battery_summary.txt`.
- [x] **LOCKSTEP** — this tree + evidence dir + decision records (already
  committed `fddecb41`/`06f08e36`) + MEMORY/CHANGES/DEVELOPMENT_NOTES/
  LIVE + the contract amendment + both books swept in the paired
  `PARSER-NEUTRALITY.2` commit (same session, minutes apart — one
  ruling's two halves). Post-removal probe preserved
  `preserved_probes/regex_perf_probe_neutrality_948cbd63` (the next
  campaign session's immediate-parent baseline).

### `PARSER-NEUTRALITY.2` — contract amendment + books (status: `done`, session #175; owns items 6, 8)
- Contract → `1.1.107` / release → `1.1.105` (schema stays `1`): a new
  2026-07-20 maintenance update declares the removal (surface-changing;
  wire-format unchanged; "action for RGX: none expected" — the typed entry
  was opt-in, never in the default build; equivalent output =
  `parse_full_regex()?.content.to_json_value()`, byte-identical). No live
  normative API line existed to delete; historical release notes stay
  verbatim (history is not rewritten).
- Books: top-level Parser Hooks chapter DELETED (+ SUMMARY +
  developer-architecture reference); regex book swept with explicit
  removal notes (public-api, build-recipe, quickstart, glossary ×2,
  migration, parse-content-variants, schema-versioning incl. the Tier-1
  table row and the CI snapshot example re-pointed to the equivalent
  call); HTML rebuilt; both book gates green.

### `PARSER-NEUTRALITY.3` — `--emit-typed-entry-skeleton` dead-scaffolding adjudication (status: `queued`)
- The flag is pipeline-internal and grammar-agnostic (no neutrality
  violation) but appears unused by any maintained target (Makefile
  references are comments only). Adjudicate remove-vs-keep with the
  director; if removed, it is a small mechanical slice (one emission
  branch + a contract test + fixture initializers).

## Verification log

- 2026-07-20 `.1`: emission identity (regex `1a5f7018` hook-free ==
  no-flag hook-capable emit; json/ebnf control regens byte-identical
  modulo own-path strings); warning-count parity; neutrality custody
  (bench +1.21%, corpus +1.61%, flips 0/2,189, MAX ≤ settled); battery
  green minus the retired gate. Evidence:
  `docs/tasks/artifacts/parser_neutrality/`.

## Commit log

- 2026-07-20 `PGEN-PARSER-NEUTRALITY-0001` (docs-only): tree opened; ruling +
  assessment decision records committed (`fddecb41`, `06f08e36`).
- 2026-07-20 `PGEN-PARSER-NEUTRALITY-0002` (leaf `.1`): the code removal +
  evidence + this tree's checklists.
- 2026-07-20 `PGEN-PARSER-NEUTRALITY-0003` (leaf `.2`): contract amendment +
  book sweep + continuity docs.
