# RAWCAP-TRANSFORM-PATH — emit the raw-content capture on the non-`Or` rule-body transform path (so Raw-view `@predicate`/directive refs resolve on `->`-shaped Sequence/Atom/Quantified rules)

- Tree ID: `RAWCAP-TRANSFORM-PATH`
- Status: `active` (created 2026-07-09, session #77, `PGEN-RAWCAP-TRANSFORM-PATH-000x`).
  Surfaced by **`REGEX-PCRE2-FIDELITY.4.3`** (`PGEN-REGEX-PCRE2-0029`) while landing the first
  `value_compare` consumer; director directive 2026-07-09: *task-tree own it right away + fix it
  whenever you get the chance*. Parser-AGNOSTIC codegen/engine gap (affects every grammar).

## 1. The finding (tool-established)

A **Raw-view positional `@predicate` (`args: [$1, le, $5]`) HARD-ERRORS at parse time on a rule
whose body is a non-`Or` node (Sequence / Atom / Quantified / Lookahead) that ALSO carries a `->`
return transform.** Reproduced in `.4.3`:

- `counted_quantifier_range = quant_bound_number brace_ws? "," brace_ws? quant_bound_number brace_ws? -> {min:$1, max:$5}`
  (a single-branch **Sequence** body + `->`) with `@predicate: {name: value_compare, args:[$1, le, $5], phase: post}`.
- `--trace-rules counted_quantifier_range` (`PGEN_TRACE_VERBOSITY=debug`) on `A{0,0}`:
  `❌ Exiting rule 'counted_quantifier_range' with error: "Semantic runtime could not resolve
  attribute reference '$1'"` → cert `A{0,0}` `parsed=false`, UNKNOWN=1.
- Worked around in `.4.3` with `view: shaped` + NAMED refs (`[$min, le, $max]`, the proven SV idiom).
  That workaround is fine for `.4.3` but the underlying codegen asymmetry is a general latent trap:
  **any raw-view positional `@predicate` on a non-`Or` `->` rule will similarly fail.**

## 2. Root cause (WHY + WHERE — pinned by direct source read at HEAD, `ast_based_generator.rs`)

The raw-content capture is emitted on the **`Or`** rule-body path but NOT on the **non-`Or`** path:

- **`Or` path (`generate_or_logic`)** — captures raw content BEFORE applying the per-branch
  transform, gated by `semantic_capture_raw_for_post`:
  - single-branch, transform == `result`: `:3349-3351` `if semantic_capture_raw_for_post { semantic_raw_content = Some(result.clone()); }`
  - single-branch, real transform: **`:3358-3360`** (capture) THEN `:3368` `result = { #transform };`
  - single-branch, no transform: `:3377-3379`
  - multi-branch: `:3731-3733` `if semantic_capture_raw_for_post { best_raw_content = Some(raw_content.clone()); }` then `best_content = Some(transformed);`
- **non-`Or` path (`generate_rule_method_with_recursion`, `rule_body_inner` `:2917-2928`)** — the gap:
  ```
  let semantic_capture_raw_for_post = …needs_raw_post_capture_for_rule(#rule_name);   // :2918
  let mut semantic_raw_content: Option<ParseContent<'input>> = None;                  // :2922 (stays None!)
  #parse_logic;                                                                        // :2924 result = RAW body content
  #post_parse_transform_tokens                                                         // :2928 → `let result = { #transform };` SHADOWS result with shaped Json
  ```
  `#post_parse_transform_tokens` is built at `:2787-2815`: for `ASTNode::Or` it is empty (Or handles
  transforms inline); for **every other root** it applies the rule-level (or synthetic `-> $1`)
  return transform via `let result = { #transform };` — **but nothing ever assigns
  `semantic_raw_content`**, so it stays `None`.
- **Consequence at resolution** (`semantic_runtime.rs:3228`): a `view: Raw` post-predicate resolves
  against `raw_content = semantic_raw_content.unwrap_or(&node.content)` = `None.unwrap_or(shaped Json)`
  = the shaped Json. Positional `$N` has no slot in a Json object → `None` → hard error
  (`ast_based_generator.rs` POST hard-resolve). Default view IS `Raw` (`semantic_runtime.rs:4085`
  `None => SemanticPredicateContentView::Raw`), so the trap is the DEFAULT for such rules.

Net: `generate_or_logic` and the non-`Or` `rule_body_inner` are ASYMMETRIC on raw-capture. The fix is
to make the non-`Or` path capture the raw `result` into `semantic_raw_content` (gated by
`semantic_capture_raw_for_post`) BEFORE `#post_parse_transform_tokens` shadows it — mirroring `:3358`.

## 3. ⚠️ Design tension (the `.1` blast-radius question — do NOT skip)

The naive fix (mirror `generate_or_logic`: insert the capture line between `:2924` and `:2928`) is
gated by `semantic_capture_raw_for_post`, so it is inert on rules with no Raw-view post-predicate.
But for a non-`Or` `->` rule that TODAY has a default-Raw **named**-ref post-predicate, behavior
changes: today `semantic_raw_content == None` ⇒ the ref resolves against the **shaped Json** (the
`SEMREF-SHAPED` Json-key path — this is the design `SEMREF-SHAPED` deliberately chose, renaming
itself away from `CODEGEN-RAWCAP` precisely to AVOID populating raw-capture here). After the fix
`semantic_raw_content == Some(raw)` ⇒ the ref resolves against the **raw tree**
(`find_semantic_named_descendant`), which — per `SV-EXH-PROOF.3.1` — does NOT descend into `Json`
and does NOT find flattened single-rule-ref chain names ⇒ could become unresolvable ⇒ **regress
SV `declared_*` and any similar consumer.**

So `.1` MUST audit: across all shipped grammars, is there any non-`Or` `->` rule with a default-Raw
(or explicit-Raw) post-predicate whose named refs currently resolve via the shaped fallback? If yes,
the naive mirror-fix regresses it and a view/ref-kind-aware variant is required; if no (all such
predicates are `view: shaped`, or there are none), the mirror-fix is safe. The
`parse_harness_equivalence_gate` (11 CERTIFIED byte-identical) + SV cert union gate are the
regression tripwires either way.

Candidate fix shapes (choose in `.1` on evidence):
- **(A) mirror-fix** — insert `if semantic_capture_raw_for_post { semantic_raw_content = Some(result.clone()); }`
  at `:2924`→`:2928`. Simplest, symmetric with `Or`. SAFE iff the `.1` audit finds no default-Raw
  named-ref post-predicate on a non-`Or` `->` rule in any shipped grammar.
- **(B) raw→shaped resolver fallback** — capture raw AND, when a Raw-view ref fails against the raw
  tree, fall back to the shaped Json. Preserves both positional (raw) and named (shaped) resolution.
  Larger semantic change; reconcile with `SEMREF-SHAPED`'s deliberate "shaped-only, no raw fallback".
- **(C) narrow the capture gate** — capture only when a raw-view post-predicate on the rule uses a
  positional ref. Most surgical behaviorally, but couples codegen to ref-kind (needs a new gate
  helper). 

## 4. Leaves

- `.1` — **EVIDENCE + DESIGN: confirm locus (done in tree preamble) + blast-radius audit + safe
  fix-shape decision.** `active`. NO code (Code-Change-Doctrine precursor). Tasks:
  - Enumerate every shipped-grammar rule that is (a) a non-`Or` root, (b) carries a `->`, (c) has a
    `@predicate`/directive post-ref; record each ref's view (explicit vs default-Raw) and kind
    (positional vs named/dotted). Tools: grep grammars + read frozen `RuleReference` literals in
    `generated/*.rs` + `--trace-rules` spot-checks. Decide A/B/C.
  - Add a minimal reproducing pin (a `parse_harness_semantic_suite` construct: a non-`Or` `->`
    Sequence rule with a raw-view positional predicate) that REJECTS today and will ACCEPT post-fix —
    the differential proof surface, byte-identical across interp/generated by construction.
- `.2` — **FIX + regen-all + no-regression + lockstep.** `not started` (pending `.1`). Implement the
  chosen fix at the pinned locus; regen ALL parsers (gitignored) + rebuild dual `ast_pipeline`;
  prove: `parse_harness_equivalence_gate` 11 byte-identical, `parse_harness_semantic_gate` (new pin
  REJECT→ACCEPT), SV cert union seeds 0/7/42 (`1343/10/1332/1`), regex cert 239, regex oracle
  `2189/1858/285/46`, duality unchanged, full dual lib, clippy. Lockstep: normative spec (raw-capture
  now symmetric on non-`Or` transform roots) + the relevant book chapter(s) + `SEMREF-SHAPED`
  cross-reference (the deliberate shaped-only decision is now reconciled) + CHANGES/DEV_NOTES/LIVE +
  this tree + TASK_TREE. No engine release-version bump unless a shipped parser's frozen literal
  changes (audit in `.1`).

## Acceptance Checklist (enforced — completed at `.2` close)
- [ ] **REPRODUCE / ISSUE** — the minimal raw-view-positional-on-non-`Or`-`->` pin REJECTs pre-fix on
  both implementations (`.4.3` trace `"could not resolve attribute reference '$1'"` is the seed).
- [x] **ROOT CAUSE (WHY + WHERE)** — non-`Or` `rule_body_inner` (`ast_based_generator.rs:2917-2928`)
  never assigns `semantic_raw_content`; the `Or` path does (`:3358` et al.). Pinned by direct read.
- [ ] **FIX** — chosen A/B/C at the `:2924`→`:2928` locus (fix-hierarchy: engine/codegen tier — no
  grammar/annotation tier can reach a codegen emit asymmetry; parser-agnostic per
  `feedback_ast_pipeline_parser_agnostic`).
- [ ] **ADDRESSED (verified)** — the pin flips REJECT→ACCEPT; `--trace-rules` shows the positional ref
  resolving against the captured raw content.
- [ ] **NO REGRESSION** — `parse_harness_equivalence_gate` 11 byte-identical; SV cert union +
  regex cert/oracle/duality all at their locked baselines; full dual lib; clippy.
- [ ] **LOCKSTEP** — normative spec + book + `SEMREF-SHAPED` reconciliation + live docs + trees.

## 5. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `.1` (evidence + blast-radius audit + fix-shape) | `active` | Locus PINNED (`:2917-2928`); needs the shipped-grammar audit + minimal pin + A/B/C decision. NO code. |
| 2 | `.2` (fix + regen + no-regression + lockstep) | `not started` | Pending `.1`. Parser-agnostic; regen all; equivalence + cert tripwires. |

## 6. Relationships

- Surfaced by [`REGEX-PCRE2-FIDELITY`](REGEX-PCRE2-FIDELITY.md) `.4.3` (`PGEN-REGEX-PCRE2-0029`) — the
  `value_compare` first-consumer slice; the `view: shaped` workaround there is the sanctioned
  short-term path.
- Sibling / must-reconcile: [`SEMREF-SHAPED`](SEMREF-SHAPED.md) (chose shaped-only for `->`-rule
  NAMED refs — this tree's `.1` must not regress that decision) and
  [`POSITIONAL-PAYLOAD-REFS`](POSITIONAL-PAYLOAD-REFS.md) (F4: positional payload refs resolve when
  raw content IS available — this tree makes raw content available on the non-`Or` transform path).
- Doctrine: [`feedback_ast_pipeline_parser_agnostic`], [`feedback_features_parser_agnostic_enable_all_parsers`]
  — the fix is a GENERAL primitive for all grammars, capability-gated (`semantic_capture_raw_for_post`),
  never grammar-name-gated.
