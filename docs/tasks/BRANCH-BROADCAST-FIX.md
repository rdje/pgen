# BRANCH-BROADCAST-FIX — group-trailing annotation broadcast regression + branch-level `$text` tournament span defect

- Tree ID: `BRANCH-BROADCAST-FIX`
- Status: `active`
- Roadmap lane: cross-cutting engine quality / released-parser bug remediation (parser-agnostic
  annotation-extraction + codegen fidelity)
- Created: 2026-06-10
- Last updated: 2026-06-10

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
- `BRANCH-BROADCAST-FIX.2` — **`pending` (engine): fix the inner→outer remap for whole-body
  groups.** Design constraint: keep patterns (A)–(D) green (the remap exists for groups inside
  sequences/quantifiers); the whole-body-group case (the rule body is exactly one top-level group,
  so `step2_group_by_or` promotes its alternatives to rule-level branches) must keep inner
  indices. Decide the discriminator from the same token stream the bookkeeping already walks
  (e.g. no syntax elements outside the just-closed group at depth 0). Add focused unit tests for:
  whole-body group (`(A|B) -> ann`), mixed (`(A|B) | C -> ann` → ann on C), trailing-group
  (`A | (B|C) -> ann` → ann on B,C — the book's documented disambiguation), and patterns (A)–(D).
- `BRANCH-BROADCAST-FIX.3` — **`pending` (engine): fix branch-level MatchedText span in the
  tournament arm.** The transform emission needs the branch-local end (`candidate_end`) instead of
  `parser.position`. Audit the other transform forms for the same hazard (object/array/property
  transforms read captured `content`, not `parser.position` — expected unaffected; verify).
- `BRANCH-BROADCAST-FIX.4` — **`pending` (release/ledger): ship the corrected `string_literal`
  shape.** Regenerate the annotation parsers; assess AST-dump schema impact (the single-quoted
  branch changes raw-Sequence → typed object = a correction of a mis-shape, SVPP-0004 category);
  ledger row + contract + book lockstep.
- `BRANCH-BROADCAST-FIX.5` — **`pending` (verify consumers): unblock + re-apply
  `GRAMMAR-WELLFORMED.H.10.2.1`** (the regex atomicity edits, re-verified end-to-end: AST A/B
  byte-identical, fused rendering, cert-coverage `UNKNOWN 7→5`).

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `.1` | `done` | Root-cause + design (this slice). |
| 1 | `.2` | `pending` | The broadcast remap fix — prerequisite for `.3` verification breadth + `.4`/`.5`. |
| 2 | `.3` | `pending` | The `$text` span fix — independent, but verified together with `.2` consumers. |
| 3 | `.4` | `pending` | Shipped-parser correction + ledger/contract/book lockstep. |
| 4 | `.5` | `pending` | Re-apply the H.10.2.1 consumer. |

## Decisions

- 2026-06-10: Both defects are owned in ONE tree because they were discovered together, share the
  verification surface (branch-level annotations in tournament rules), and `.5`'s consumer needs
  both. The 2026-05-14 remap is NOT reverted — it fixed real codegen drops; `.2` refines its
  applicability condition instead.

## Open questions

- (`.2`) Exact discriminator for "group's branches survive as runtime branches" — derive from the
  token stream vs consult `step2_group_by_or`'s actual grouping; decide in-leaf with tests.
- (`.4`) Schema bump decision for `return_annotation` (mis-shape correction category) — assess
  against the release policy when the corrected shape is in hand.

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
