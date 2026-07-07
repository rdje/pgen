# BOUNDED-QUANT — bounded quantifiers `{n}` / `{n,m}` / `{n,}` / `{,m}`: close the codegen half-wire (full generator⟷parser duality)

- Status: `active` (created 2026-07-07, session #53). FRONTIER = `.1`.
- Roadmap lane: cross-cutting engine correctness — the EBNF input language's declared repetition
  surface must be honored END-TO-END (frontend → codegen → generated parser → interpreter →
  linter → stimuli), per the Layer-0 strategic line (director, 2026-05-21: *"After Layer 0, all
  the repetition in the EBNF shall have the right behavior"*) and the EBNF-single-source-of-truth
  doctrine ([[project_ebnf_is_single_source_of_truth]]).
- Origin: tool-established finding #1 of `PARSE-HARNESS.6.1` (session #46, recorded in
  `DEVELOPMENT_NOTES.md` and `docs/tasks/PARSE-HARNESS.md` §20): *"bounded quantifiers are
  expressible in `.ebnf` and supported at runtime, but cannot be compiled."* Parked there
  one-clean-slice; picked up as its own tree by PNT (session #53).

## 1. Problem statement (tool-backed, reproduced live 2026-07-07)

The EBNF meta-grammar **declares** the four bounded quantifier forms (`grammars/ebnf.ebnf:164-186`:
`exact_count` `{n}`, `min_count` `{n,}`, `max_count` `{,m}`, `range_count` `{n,m}`), and the
ebnf parser book **documents all seven quantifier forms as first-class codegen surface**
(`docs/ebnf_parser_book/src/quantifiers.md`, `codegen-model.md`, `welcome.md`). In reality the
surface is HALF-WIRED:

- **REPRODUCE (generation side — WORKS).** Minimal grammar
  `scratch := item{2,3}` / `item := "a"`:
  `ast_pipeline bounded_quant.ebnf --generate-stimuli --count 5 --seed 0` →
  `a a a` / `a a` / `a a a` / `a a` / `a a a`,
  `Stimuli coverage: rules 2/2 (100.00%) … sample_successes=5/5` — correct 2..3 repetition.
- **REPRODUCE (parser side — HARD ABORT).** Same grammar:
  `ast_pipeline bounded_quant.ebnf --generate-parser --output …` →
  `Error: Failed to generate parser using AST-based generator` / `Unknown quantifier: 2,3`.

So the stimuli generator emits strings for a grammar whose parser **cannot even be built** — a
live generator⟷parser duality break (the exact defect class `EBNF-SOURCE-OF-TRUTH` /
`GRAMMAR-WELLFORMED` protect against), plus a live book⟷codebase drift on the user-facing
surface (the book is the director's only window; it promises a capability the toolchain refuses).

## 2. Root cause (WHY + WHERE — pinned before any code change)

The quantifier *surface string* carried in `ASTNode::Quantified::quantifier` has TWO dialects and
the consumers disagree on which one they speak:

1. **Emission (brace-less).** The Rust EBNF frontend `parse_braced_quantifier`
   (`rust/src/ebnf_frontend.rs:1013-1032`) strips the braces: `item{2,3}` → raw-AST token
   `["quantifier","2,3"]` (verified via `--emit-raw-ast-json`; contract-locked by the frontend
   test `tokenizes_regex_and_bounded_quantifier`, `ebnf_frontend.rs:1169`). The token flows
   unchanged into `RawRuleElement::Quantifier` → `ASTNode::Quantified { quantifier: "2,3" }`
   (`rust/src/ast_pipeline/mod.rs:3065`, `:3491`).
2. **Canonical decode (braced-only).** The shared Layer-0 helper `parse_quantifier_bounds`
   (`rust/src/ast_pipeline/mod.rs:925`) accepts `?` / `*` / `+` and the **braced** forms
   `{N}` / `{N,M}` / `{N,}` / `{,M}` — and returns `None` for the brace-less strings the
   frontend actually emits.
3. **The abort.** Both parser codegens delegate to the canonical helper and hard-error on
   `None`: `ast_based_generator.rs:4013-4015` and `ast_code_generator.rs:437-439`
   (`Unknown quantifier: 2,3`).
4. **The asymmetry (why generation works).** The stimuli generator does NOT use the canonical
   helper — it has a **private, duplicated** `parse_quantifier_bounds`
   (`stimuli_generator.rs:10346-10391`) that accepts the brace-less forms (`"2"`, `"2,3"`,
   `"2,"`, `",3"`). Duplicated logic drifted exactly as
   [[feedback_duplicated_metadata_needs_derived_drift_gate]] predicts.
5. **Silent misreads (collateral).** Every other canonical-helper consumer decodes a bounded
   form as its `unwrap_or` fallback instead of its true bounds: the interpreter
   (`parse_harness_interpreter.rs:2073`, `unwrap_or((0, None))` → `{2,3}` silently behaves like
   `*`) and 7 `grammar_wellformedness.rs` sites (`:197`, `:531`, `:583`, `:664`, `:725`,
   `:2314`, `:2639` — min/max misclassification for nullability/liveness analyses), plus
   `stimuli_generator.rs::count_mandatory_yield_atoms` (`:7344`, `unwrap_or(true)`).

Fix-direction adjudication (PINPOINT, one clean fix, no menu): extend the **canonical decoder**
to also accept the brace-less normalized forms. The alternative (make the frontend emit braced
forms) is rejected: the brace-less token is a pinned cross-frontend raw-AST JSON contract
(`ebnf_frontend.rs:1169` + stored raw_ast artifacts), and the stimuli generator's acceptance is
already brace-less — decoder extension is strictly additive (braced forms remain accepted),
zero contract churn, and it heals codegen + interpreter + linter through the one shared helper
they already call. The stimuli generator's private parser is then DELEGATED to the canonical
helper (single source of truth; keeps its `max_repeat` clamping), so the two dialect speakers
can never drift again.

## 3. Tree

- `BOUNDED-QUANT.1` — **the decoder wire + stimuli delegation + per-combinator proof + lockstep**
  (one clean slice). Status: `done` (2026-07-07, session #53, `PGEN-BOUNDED-QUANT-0001`).

## 4. Tree-level acceptance criteria

1. A grammar using any of the four declared bounded forms compiles through
   `--generate-parser` (both codegens), parses with correct accept/reject windows, interprets
   byte-identically to the generated parser, stimuli-generates, and lints — no dialect split.
2. The combinator suite (`parse_harness_combinator_gate`) covers the bounded forms as
   first-class isolating cases (interp == compile-and-run oracle, byte-identical verdict +
   `furthest_position` + typed AST) — the `.6.1` honest bound is CLOSED, not re-documented.
3. Exactly ONE bounds decoder exists (`ast_pipeline::parse_quantifier_bounds`); the stimuli
   generator delegates to it.
4. Lockstep: TOOLBOX 1.7 honest-bound text, book *Parse Harness* chapter, ebnf parser book
   claims corrected to truth (including the false "used throughout `grammars/regex.ebnf`"
   claim — tool-scan shows ZERO bounded tokens in any frontend-parseable shipped grammar),
   PARSE-HARNESS §20 live-spec note, continuity docs.
5. No regression: full lib suite, the three parse-harness gates, emit-neutrality for every
   shipped generated parser (no shipped grammar uses bounded forms — regen byte-identical),
   clippy strict-source, mdbook gates.

## 5. BOUNDED-QUANT.1 — acceptance checklist (enforced)

## Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — `ast_pipeline bounded_quant.ebnf --generate-parser` →
  `Error: Failed to generate parser using AST-based generator` / `Unknown quantifier: 2,3`,
  while `--generate-stimuli --count 5 --seed 0` on the SAME grammar emits `a a a`/`a a`
  (`sample_successes=5/5`) — the duality break, reproduced live 2026-07-07 (§1).
- [x] **ROOT CAUSE (WHY + WHERE)** — dialect split on `Quantified::quantifier`: frontend emits
  brace-less `["quantifier","2,3"]` (`ebnf_frontend.rs:1027`, shown by `--emit-raw-ast-json`);
  canonical `parse_quantifier_bounds` (`mod.rs:925`) decodes braced-only → `None` →
  `ast_based_generator.rs:4015` / `ast_code_generator.rs:439` abort; stimuli generator speaks
  brace-less via its private duplicate decoder (`stimuli_generator.rs:10346`) — full chain in §2.
- [x] **FIX** — minimal shared-helper extension (fix tier: parser-agnostic engine helper — the
  lowest tier that can fix a codegen-internal abort; no annotation/store level applies):
  `parse_quantifier_bounds` (`rust/src/ast_pipeline/mod.rs`) now strips the braces when present
  and decodes BOTH spellings through one inner parser (braced acceptance unchanged; +6 unit
  assertions incl. both-spelling invalid-form rejection and the degenerate `{,}`≡`,`≡`(0,None)`
  consistency); the stimuli generator's private decoder is replaced by a delegation to the
  canonical helper (`stimuli_generator.rs`, unbounded maxima still clamp
  `config.max_repeat.max(min)`); the combinator suite gains 4 `Combinator` variants + 4
  `quant_bounded_*` isolating cases (16→20) and its stale honest-bound module doc is rewritten;
  the interpreter's stale bounded-unreachable NOTE updated.
- [x] **ADDRESSED (verified)** — before→after on the §1 reproducer: `--generate-parser` was
  `Error: Failed to generate parser using AST-based generator / Unknown quantifier: 2,3` →
  now `SOTA parser generated: …/bounded_quant_parser.rs` (129 231 bytes, rc=0); the same-grammar
  stimuli run is byte-identical pre/post delegation (`a a a`/`a a`/…, `sample_successes=5/5`,
  seed 0). Re-runnable oracle: `make -C rust SHELL=/bin/bash parse_harness_combinator_gate` →
  `every_structural_combinator_is_byte_identical` + `combinator_coverage_is_complete` **2 passed /
  0 failed**, per-case report **20/20 CLEAN** incl. `quant_bounded_exact` / `quant_bounded_range` /
  `quant_bounded_at_least` / `quant_bounded_at_most` (each `diverge=0 anchor_miss=0` — the exact
  accept windows `{2}`:only-`xx`, `{2,3}`:2..=3, `{2,}`:≥2, `{,2}`:0..=2 hold byte-identically on
  the interpreter AND the compile-and-run oracle, i.e. the REAL codegen + runtime, authoritative
  by construction — TOOLBOX 1.4).
- [x] **NO REGRESSION** — features-on lib suite `cargo test --lib --features "generated_parsers
  ebnf_dual_run"` → **811 passed / 0 failed** (29 ignored — the durable `--ignored` probes);
  `make -C rust parse_harness_equivalence_gate` → **4 passed / 0 failed** (11 CERTIFIED grammars
  byte-identical); `make -C rust parse_harness_semantic_gate` → **2 passed / 0 failed** (24/24
  CLEAN); emit-neutrality PROVEN: `make focus_json` + `make focus_regex` regen → `cmp` clean
  ("json BYTE-IDENTICAL" / "regex BYTE-IDENTICAL") and the §1 raw-AST scan shows ZERO bounded
  tokens in all 15 frontend-parseable shipped grammars (so no other generated parser can be
  affected); `clippy_on_rust_change` strict source stage `clippy_source_all_targets: ok`
  (generated stage = the KNOWN pre-existing `eq_op` debt, non-strict by design, unchanged);
  `mdbook_docs_gate` + `ebnf_parser_book_gate` both ✅.
- [x] **LOCKSTEP** — TOOLBOX §1.7 (20 cases; bounded forms covered; honest bound narrowed to
  direct-LR); top book `docs/book/src/parse-harness.md` (20-case table incl. the bounded row;
  the "two subtleties" section rewritten to record the closure; combinator count in the
  trust-story bullet); ebnf parser book `quantifiers.md` (the "all seven first-class" claim now
  TRUE + proof pointer; the false "used throughout `grammars/regex.ebnf`" claim corrected —
  regex-literal quantifiers, not EBNF ones) + rendered `docs/ebnf_parser_book-html/` regenerated;
  `docs/tasks/PARSE-HARNESS.md` §20 LIVE-SPEC note (finding #1 → RESOLVED here);
  `parse_harness_interpreter.rs` smoke-set NOTE updated; CHANGES.md + DEVELOPMENT_NOTES.md
  entries; MEMORY.md overwritten; docs/TASK_TREE.md row; LIVE_ACHIEVEMENT_STATUS tracker note
  (status rows UNCHANGED — engine-internal closure, no family status moves).

## 6. Verification log

- 2026-07-07 (session #53): §1 reproduction (frontend token dump, codegen abort, stimuli
  success), §2 chain pinned by direct source reads at the cited lines, all-grammars raw-AST
  bounded-token scan (15/15 frontend-parseable grammars: ZERO bounded tokens; the 3
  `*_lrm_extracted.ebnf` snapshots are frontend-unparseable raw extraction artifacts —
  pre-existing, unrelated).
- 2026-07-07 (session #53, post-fix): decoder unit tests 5/5 (`parse_quantifier_bounds_tests`,
  incl. the new `brace_stripped_bounded_quantifiers`); reproducer `--generate-parser`
  REJECT→PASS (129 231-byte parser emitted); stimuli byte-identical pre/post delegation (seed 0,
  5/5); `parse_harness_combinator_gate` 2/2 with **20/20 CLEAN** (4 new `quant_bounded_*` cases
  each `diverge=0 anchor_miss=0`); `parse_harness_equivalence_gate` 4/4;
  `parse_harness_semantic_gate` 2/2; features-on lib 811/0; emit-neutrality `cmp` clean for
  json + regex regen; clippy strict-source ok; `mdbook_docs_gate` + `ebnf_parser_book_gate` ✅.
  Commit `PGEN-BOUNDED-QUANT-0001`.

## 7. Decisions

- Decoder-extension over frontend-emission-change (rationale in §2 adjudication).
- The interpreter's `unwrap_or((0, None))` fallback for *invalid* quantifier strings is kept:
  an invalid quantifier cannot compile through the oracle side at all, so no differential case
  can exist; the fallback is unreachable for any grammar the harness can compare. Recorded, not
  changed (one-clean-slice).
- The private stimuli decoder's bespoke per-shape error messages collapse into one clear
  message on delegation; a repo-wide grep proved none of the old message strings are
  test-pinned.
