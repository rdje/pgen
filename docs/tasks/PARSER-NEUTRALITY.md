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

### `PARSER-NEUTRALITY.1` — hook-mechanism + typed-surface removal (status: `queued`; owns items 1–5, 7)
- One slice in a FRESH session: lib + emitter + CLI removal, all-11 regen
  train (expected delta: the typed surface disappears from the regex
  artifact; all other artifacts byte-identical — review-proven), full
  battery minus the retired gate, floor-probe byte-identity re-proof
  (fat-LTO probe rebuilt against the default-emit artifact must reproduce
  the 1,153.5 ns floor within custody bounds), doctrine enforcer, books.
- Acceptance: `grep -r "parser_hooks\|enable-parser-hooks\|_typed(" `
  over `rust/src/` + `generated/` returns zero product hits; battery
  green; floor unchanged within noise; all-11 custody clean.

### `PARSER-NEUTRALITY.2` — contract amendment + ledger + books (status: `queued`; owns items 6, 8; director signs the published text)
- Contract version bump dropping `parse_regex_typed()`; released-parser
  bug-ledger/maintenance note; migration note; book/doc sweep.

## Verification log

- (pending)

## Commit log

- 2026-07-20 `PGEN-PARSER-NEUTRALITY-0001` (docs-only): tree opened; ruling +
  assessment decision records committed (`fddecb41` + this commit); no code
  touched — removal executes in a fresh session per the session discipline.
