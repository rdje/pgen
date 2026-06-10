# BRANCH-BROADCAST-FIX — group-trailing annotation broadcast regression + branch-level `$text` tournament span defect

- Tree ID: `BRANCH-BROADCAST-FIX`
- Status: `active`
- Roadmap lane: cross-cutting engine quality / released-parser bug remediation (parser-agnostic
  annotation-extraction + codegen fidelity)
- Created: 2026-06-10
- Last updated: 2026-06-10 (`.2` done — remap fixed; blast radius measured: BOTH annotation
  parsers carry newly-correct shapes; `.4` scope widened accordingly)

## Goal

Restore the documented parens-grouped-Or trailing-annotation **broadcast** semantics
(`RULE = ( A | B | C ) -> ann` ≡ per-branch `ann` — task #38's contract, currently regressed), and
make branch-level **`$text` (MatchedText)** produce the correct matched span inside multi-branch
(tournament) rules. Both defects are engine-level, parser-agnostic, and currently bite a SHIPPED
parser (see Evidence).

## Non-goals

- No change to the documented annotation-language surface (this restores documented behavior).
- The `GRAMMAR-WELLFORMED.H.10.2.1` regex atomicity consumer is owned by its own leaf (it
  re-applies once `.2`+`.3` land).
- No revert of the 2026-05-14 inner→outer remap's legitimate fixes (codegen-drop patterns (A)–(D),
  [[feedback_codegen_outer_branch_remap]]) — the fix must keep those green.

## Evidence (tools-first, 2026-06-10 — all pinned on a clean `-0060` baseline + minimal probes)

**Defect A — whole-body-group trailing-annotation broadcast collapses to branch 0.**
Minimal probe grammar (Rust frontend path, `--emit-raw-ast-json` → `--generate-parser`):

```ebnf
start = item
item = ( "D" | "S" ) -> $text
other = ( "X" | "Y" ) -> {kind: $1}
start2 = other
```

- The raw IR is correct: `group_open, "D", |, "S", group_close, return_scalar $text`.
- The declared-annotation inventory records **branch 0 only** for BOTH rules (`$text` AND the
  object form — the defect is shape-level, not `$text`-specific).
- The generated parser runs the rules as **2-branch tournaments** ("branch 1/2", "2/2"), with the
  annotation transform emitted in **branch 0's arm only**; branch 1 gets the implicit passthrough.
- Mechanism (`rust/src/ast_pipeline/mod.rs`, `extract_rule_annotations`): the group-close
  bookkeeping correctly records the inner range `(0,1)` and the trailing annotation IS broadcast
  to inner branches 0..=1 — but the **2026-05-14 inner→outer `branch_to_outer` remap** (added for
  codegen-drop patterns (A)–(D), where a group is a *sub-part* of a sequence and inner branches do
  NOT survive as rule-level branches) then collapses both inner branches to **outer branch 0**.
  For a **whole-body group**, `step2_group_by_or` unwraps the group's alternatives into the rule's
  own runtime branches, so the collapse is wrong there: the inner indices ARE the runtime indices.
- **SHIPPED impact (a live released-parser bug — the #38 exemplar itself re-broken):** the
  `return_annotation` parser's `string_literal := ('"' … '"' | "'" … "'") -> {type:"string",
  value:$2}`. Runtime proof (release `parseability_probe --parse-dump-ast return_annotation`):
  `"x"` → `{Json:{type:"string", value:"x"}}` (typed, correct) but `'x'` → **raw `Sequence`** of
  terminals — exactly the pre-#38 defect shape. The generated `parse_string_literal` contains ONE
  object-transform site for its 2 branches.

**Defect B — branch-level `$text` slices an empty span inside a tournament.**
- `ast_return_transform.rs:91-93` emits `ParseContent::Terminal(&parser.input[start_pos..parser.position])`
  for `MatchedText` — correct on the single-branch/rule-level path (position is still at the match
  end), WRONG inside the multi-branch tournament arm: codegen emits
  `let candidate_end = parser.position; parser.position = parse_start;` **before** evaluating the
  branch transform, so the slice is `start_pos..parse_start` = **empty**.
- Runtime proof (uncommitted probe state, since reverted): with `-> $text` bound to branch 0 of
  `ascii_restrict_modifier`, `(?^aD-aD)` parsed with `restrict: ""` (a matched `"D"` returning the
  empty string); `\pC` parsed with `name: ""` while `\pL` (branch 1, no transform) stayed `"L"`.
- Fix direction: the branch-arm MatchedText emission must use the branch's own captured end
  (`candidate_end`) — e.g. parameterize the transform emission with an end-position expression, or
  evaluate the transform before the tournament's position reset.

**Discovery chain:** `GRAMMAR-WELLFORMED.H.10.2.1` tried the documented declarative atomicity fix
(parens-group broadcast `-> $text` on `short_prop_letter`/`ascii_restrict_modifier`) and the A/B
verification caught the `restrict:""` mis-shape immediately → root-caused to this pair; the
grammar edits were REVERTED (baseline re-verified byte-identical) and `H.10.2.1` is now blocked on
`.2`+`.3`. Note the consumer dependency: `rule_is_lexically_atomic`
(`stimuli_generator.rs:7978`) requires MatchedText on **every** branch, so Defect A alone also
prevents the atomicity mechanism from engaging on Or-root rules.

## Acceptance criteria (tree-level)

1. `RULE = ( A | B | C ) -> ann` broadcasts to every group branch through BOTH frontend paths,
   with the codegen-drop patterns (A)–(D) regression-locked green.
2. Branch-level `$text` returns the exact matched span in multi-branch rules.
3. The shipped `return_annotation` `string_literal` single-quoted shape is typed again
   (`'x'` ≡ `"x"` modulo the value), with a released-parser ledger row + contract/book lockstep
   and the release/schema decision made per the release policy.
4. Cross-grammar: full regen byte-comparison — only rules genuinely exercising the fixed paths
   change; all suites/gates green.

## Task tree

- `BRANCH-BROADCAST-FIX.1` — **`done` (this slice, pure docs): root-cause + evidence + design
  direction** (everything in Evidence above; minimal probe grammar reproduces both defects in
  isolation; shipped impact pinned at runtime on `string_literal`).
- `BRANCH-BROADCAST-FIX.2` — **`done` (engine): the inner→outer remap fixed for whole-body
  groups.** Implementation: `extract_rule_annotations` (`rust/src/ast_pipeline/mod.rs`) now tracks
  a SECOND mapping `branch_to_body` (`|` at `group_depth <= 1`) alongside the 2026-05-14
  `branch_to_outer` (`|` at depth 0), and selects per rule via the new module-level discriminator
  `syntax_is_single_whole_body_group(&syntax_elements)`: true iff the first syntax element is a
  `group_open` whose MATCHING `group_close` is the last syntax element (any depth-0 token —
  quantifier on the group, leading/trailing atoms, top-level `|` — disqualifies). For whole-body
  groups the group's alternatives ARE the runtime branches (step2_group_by_or sees no top-level
  `|` and the group's Or is unwrapped to the rule root), so group-local indices are the correct
  runtime indices; every other shape keeps the outer remap (patterns (A)–(D) intact). The
  cross-extractor `extract_declared_annotations_from_json` (`rust/src/ast_shape_contract.rs`) now
  mirrors the pipeline's bookkeeping EXACTLY (inner branch counting + group-close broadcast range
  + selected remap, sharing the same discriminator fn) instead of the outer-only walk. Manifest:
  `return_annotation_v1.json` gains the `string_literal` branch-1 inventory row (19→20; the
  broadcast row the gate had been missing). Commit: `PGEN-BRANCH-BROADCAST-FIX-0002`.
- `BRANCH-BROADCAST-FIX.3` — **`done` (engine): branch-level MatchedText span fixed in the
  tournament arm.** Implementation (`rust/src/ast_pipeline/ast_based_generator.rs`, the
  branch-attempt arm): the branch transform is now evaluated BEFORE the arm's
  `parser.position = parse_start;` rollback (at which point `parser.position == candidate_end`,
  the branch's true end), instead of after it. This is the complete fix because the transform-form
  AUDIT confirmed `$text`/MatchedText is the ONLY form reading `parser.position` — every other
  arm of `AstReturnTransformer::generate_transform` (positional/literals/object/array/spread/
  property/array-access/quantified-extraction/passthrough) reads only the captured `content`, so
  the reorder is observable to MatchedText alone. Regression-locked by a codegen unit test
  asserting per-arm ordering (transform binding precedes the rollback in BOTH arms of a 2-branch
  `$text` tournament) + the MatchedText slice emission. Commit: `PGEN-BRANCH-BROADCAST-FIX-0003`.
- `BRANCH-BROADCAST-FIX.4` — **`done` (release/ledger): the corrected shapes shipped for BOTH
  annotation parsers.** Versioning decision: the annotation families carry NO numeric
  release/AST-dump-schema machinery (their contracts are date-versioned "Notable Recent Shape
  Changes"/"Recent Additions" surfaces — unlike regex/svpp), so the SVPP-0004-style schema bump
  maps to: dated contract entries documenting the regression window (2026-05-14 → 2026-06-10) +
  ledger rows + manifest regression locks. Shipped: (a) ledger rows `RETANN-0001` + `SEMANN-0001`
  (one per family, same engine root cause, cross-referenced); (b) contract entries in both
  family contracts (consumer guidance: drop any single-quoted-`Sequence` workaround adopted
  inside the window; branch-0 inputs byte-identical); (c) `return_annotation_v1.json` gains the
  DISCRIMINATING runtime sample `single_quoted_string_literal_typed` (`'x'` → `json_object`
  `{type:"string", value:"x"}` — pre-fix this input produced `sequence`); (d)
  `semantic_annotation_v1.json` gains the FULL 151-row declared-annotation inventory (artifact +
  raw-IR crosscheck comparisons both active for that grammar for the first time).
  Commit: `PGEN-BRANCH-BROADCAST-FIX-0004`.
- `BRANCH-BROADCAST-FIX.5` — **`pending` (verify consumers): unblock + re-apply
  `GRAMMAR-WELLFORMED.H.10.2.1`** (the regex atomicity edits, re-verified end-to-end: AST A/B
  byte-identical, fused rendering, cert-coverage `UNKNOWN 7→5`).

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `.1` | `done` | Root-cause + design. |
| — | `.2` | `done` | The broadcast remap fix (`PGEN-BRANCH-BROADCAST-FIX-0002`). |
| — | `.3` | `done` | The `$text` tournament-span fix (`PGEN-BRANCH-BROADCAST-FIX-0003`). |
| — | `.4` | `done` | Ship surface for BOTH annotation parsers (`PGEN-BRANCH-BROADCAST-FIX-0004`). |
| 1 | `.5` | `pending` | Re-apply the H.10.2.1 consumer (also the live-fire runtime proof for `.3`). |

## Decisions

- 2026-06-10: Both defects are owned in ONE tree because they were discovered together, share the
  verification surface (branch-level annotations in tournament rules), and `.5`'s consumer needs
  both. The 2026-05-14 remap is NOT reverted — it fixed real codegen drops; `.2` refines its
  applicability condition instead.
- 2026-06-10 (`.2`): discriminator DECIDED — derived from the same annotation-free
  `syntax_elements` token list the extraction already accumulates (`first element is `group_open`
  AND its matching `group_close` is the last element`), exposed as a `pub(crate)` module fn so
  the `ast_shape_contract` cross-extractor shares the SINGLE implementation instead of mirroring
  it by hand. Both mappings are tracked during the one existing walk (`branch_to_outer` depth==0,
  `branch_to_body` depth<=1) and selected after it — no second pass, conservative on unbalanced
  groups (falls back to the outer/status-quo mapping).
- 2026-06-10 (`.2`): the manifest inventory row moves WITH the engine fix (not deferred to `.4`)
  because `generated/` is untracked — any fresh-clone regen uses the fixed pipeline, so a stale
  manifest would fail the inventory gates immediately. The SHIP surface (ledger/contract/book/
  release versioning for the corrected runtime shapes) remains `.4`.

## Open questions

- none. (`.4`'s schema question RESOLVED: the annotation families are date-versioned contract
  surfaces with no numeric release/schema constants — the correction ships as dated contract
  entries + ledger rows + manifest locks, the family-appropriate equivalent of the SVPP-0004
  schema bump.)

## Blockers

- none.

## Verification log

- `.1` (2026-06-10): probe grammar IR verified correct; inventory branch-0-only for `$text` AND
  object forms; generated probe parser transform in branch 0 only (branch 1 implicit passthrough);
  shipped `string_literal` runtime A/B (`"x"` typed vs `'x'` raw Sequence); `$text` tournament
  empty-span runtime proof (`restrict:""`/`name:""` on matched single chars); baseline restored +
  re-verified after the probe edits were reverted (AST dumps byte-identical; regex cert-coverage
  `UNKNOWN=7, spf=0` at the `-0060` state).
  Commit: `PGEN-BRANCH-BROADCAST-FIX-0001`.
- `.2` (2026-06-10): (1) 6 focused unit tests green — whole-body trailing broadcast (`$text` AND
  object form, asserting the rule root IS the 2-branch Or), whole-body PER-branch annotations,
  the documented disambiguations (`(A|B) | C -> ann` → ann on C only; `A | (B|C) -> ann` → ann on
  the outer group arm), patterns (A)–(D) regression locks, and the discriminator's 6 token
  shapes (incl. nested `((A|B)|C)` and adjacent-groups `(A|B)(C|D)`); plus the cross-extractor
  broadcast test on a temp frontend JSON. NOTE: synthetic IR fixtures must use the REAL 2-element
  group tokens (`["group_open","("]`) — bare 1-element tokens are silently dropped by
  `parse_raw_element` (arity < 2) and distort the AST lane. (2) full suites green: default 682/0,
  `--features generated_parsers` workspace 767/0. (3) DECISIVE stash-baseline cross-grammar regen
  byte-comparison (pre-fix pipeline regen vs post-fix regen, all 9 grammar targets): semantic
  delta confined to `return_annotation_parser.rs` (+ its inventory artifact 19→20) and
  `semantic_annotation_parser.rs` (+ inventory 108→151, the 43 broadcast rows); `regex_parser.rs`
  and `systemverilog_parser.rs` diffs proven SET-EQUAL `directives_by_rule` entries (5/5 and
  43/43) + map-order churn in two serialized LR-chain templates — zero semantic delta (HashMap
  iteration-order across binaries; same-binary double-regen is byte-identical); json /
  rtl_const_expr / svpp / rtl_frontend / vhdl parsers byte-identical. The frontend `*.json`
  envelope diffs are `generated_at` timestamp noise only. (4) runtime A/B on the shipped defect:
  `'x'` → `{Json:{type:"string",value:"x"}}` ≡ `"x"`; generated `parse_string_literal` carries
  the object transform in BOTH branch arms (was 1). (5) regex cert-coverage baseline preserved
  IDENTICALLY: `total=198 witness=191 UNKNOWN=7 sample_parse_failures=0` at seeds 0/7/42
  (`--count 40`, the `-0060` numbers). (6) clippy strict-source clean; generated-stage non-strict
  failures are the pre-existing generated-code debt class (all 191 sites inside
  `generated/systemverilog{,_preprocessor}_parser.rs`).
  Commit: `PGEN-BRANCH-BROADCAST-FIX-0002`.
- `.3` (2026-06-10): (1) transform-form audit complete — `generate_transform`'s arms read only
  captured `content` EXCEPT MatchedText (`&parser.input[start_pos..parser.position]`), so moving
  the arm's position rollback after the transform binding is the complete fix. (2) new codegen
  unit test green (per-arm whitespace-insensitive ordering assertion: `candidate_end` binding →
  transform binding → rollback, for both arms of a 2-branch `$text` tournament; also asserts the
  MatchedText slice emission). (3) all 9 grammar targets regenerated with the reordered arm;
  default workspace 683/0; dual-feature workspace 768/0 (no shipped grammar currently binds
  branch-level `$text` in a tournament — H.10.2.1 reverted — so runtime corpora are unaffected,
  as expected). (4) regex cert-coverage baseline IDENTICAL again (`total=198 witness=191
  UNKNOWN=7 spf=0`, seeds 0/7/42, `--count 40`). (5) clippy strict-source clean; generated-stage
  non-strict debt class unchanged (same 191 pre-existing generated sites). The live-fire runtime
  proof of the corrected span lands with `.5` (the regex atomicity consumer re-applied:
  `restrict:"D"` instead of `restrict:""`, A/B byte-identical dumps).
  Commit: `PGEN-BRANCH-BROADCAST-FIX-0003`.
- `.4` (2026-06-10): (1) the new `single_quoted_string_literal_typed` manifest sample passes
  against the corrected compiled-in parser (with full structural assertions: keys + string
  values), and the `semantic_annotation` 151-row inventory passes BOTH comparisons (pipeline
  artifact + raw-IR crosscheck) — all 14 shape-contract gates green. (2) the aggregate
  annotation spine `annotation_contract_gate` exits 0 with the corrected parsers (validator
  coverage, built-in/shared suites, SC semantic slices, aggregate semantic/return gates,
  robustness/stimuli verification). (3) dual-feature workspace 768/0. (4) versioning decision
  recorded in the leaf (date-versioned contracts; no numeric release machinery exists for the
  annotation families to bump).
  Commit: `PGEN-BRANCH-BROADCAST-FIX-0004`.
