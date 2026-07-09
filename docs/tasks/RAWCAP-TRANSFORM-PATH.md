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

### `.1` RESOLUTION (2026-07-09, `PGEN-RAWCAP-TRANSFORM-PATH-0002`) — audit done, **DECISION: (C)**

**Blast-radius audit (tool-backed).** Enumerated every shipped-grammar post-predicate NOT carrying
`view: shaped` (grep `grammars/*.ebnf`), read each owning rule (predicate binds to the FOLLOWING
rule), and classified body-root (`Or` vs non-`Or`), presence of `->`, and ref kind
(positional `$N` vs named `$name`). Source facts pinned by direct read:
- The capture gate `needs_raw_post_capture_for_rule` (`semantic_runtime.rs:1490`) fires for ANY
  post-predicate whose `predicate_view() == Some(Raw)`; the DEFAULT view is compiled to `Raw`
  (`semantic_runtime.rs:4085` `None => Raw`), so **default-view == Raw** for the gate — the trap is
  the default, exactly as stated.
- The `Or` path (`generate_or_logic`, `:3349`/`:3358`/`:3377`) and the non-`Or` path share the SAME
  `semantic_capture_raw_for_post` flag (computed once at `:2918`) ⇒ narrowing the SHARED helper would
  also change the `Or` path's capture (regressing `Or` rules whose named refs resolve against
  `best_raw_content`). So (C) MUST use a NEW positional-aware helper on the non-`Or` path only; the
  `Or` path stays byte-identical.

**The regression set for (A) is NON-EMPTY — (A) REJECTED.** Concrete shipped rules that are
(non-`Or` root) ∧ (`->` producing shaped Json) ∧ (default-Raw NAMED-ref post-predicate), which
resolve their named ref against the shaped Json via the `SEMREF-SHAPED` branch TODAY and would flip
to a failing raw-tree walk under (A):
- **SystemVerilog `declared_*` family (10 rules):** `declared_checker_identifier`,
  `declared_class_identifier`, `declared_interface_class_identifier`, `declared_covergroup_identifier`,
  `declared_let_identifier`, `declared_parameter_identifier`, `declared_property_identifier`,
  `declared_sequence_identifier`, `declared_type_parameter_identifier`, `declared_type_identifier` —
  each `declared_X := X_identifier -> { body: $1.body }` (single rule-ref body = non-`Or` Atom) with
  `@predicate has_fact/fact_attribute_equals [<kind>, $body], phase: post` (default-Raw named `$body`,
  resolved against the produced `{body:…}` Json today). (The `fact_attribute_equals [type_name, $body,
  …]` `known_*`/type-consumer rules in the same file are the same class.)
- **regex `numeric_backreference`:** `"\\" backreference_digits -> {…, index: $2}` (a Sequence =
  non-`Or`) with `@predicate fact_count_at_least [regex_capture_group, $index], phase: post`
  (default-Raw named `$index`); its own grammar comment states `$index` reads the rule's own `->`
  object field `index` — "the `SEMREF-SHAPED` engine capability."

**Zero positional-ref Raw-view post-predicates ship today.** `grep -rnE
'@predicate.*args:\s*\[[^]]*\$[0-9]' grammars/*.ebnf` → EMPTY (the only such consumer, `.4.3`'s
`counted_quantifier_range [$1,le,$5]`, was worked around to `view: shaped` + named `[$min,le,$max]`).
⇒ a positional-gated capture fires for **zero** shipped rules ⇒ **(C) is inert on every shipped
grammar ⇒ zero regression by construction**, and only becomes live when a future positional consumer
(e.g. a raw-view `value_compare [$1, le, $5]`) lands — which then correctly receives raw content.

**Why not (B):** (B) makes named-ref resolution "raw-first, shaped-fallback" globally — it partially
undoes `SEMREF-SHAPED`'s deliberate "shaped-only, no raw fallback" and changes the resolution PATH of
all ~11 named-ref rules above (same end value in the common case, but a real divergence if a `->` rule
ever has a raw sub-rule whose name collides with a Json key). Larger surface, needs reconciliation.
(C) touches none of that: named-ref resolution stays byte-identical everywhere; only positional-ref
non-`Or` rules change, of which there are none shipped. (C) wins decisively.

**Positional detection primitive** (reused by the new helper, pinned): a `$`-ref is positional iff the
body after `$` is all-ASCII-digits — `dollar_reference_is_positional` at
`ast_based_generator.rs:5809-5814`.

## 4. Leaves

- `.1` — **EVIDENCE + DESIGN: confirm locus (done in tree preamble) + blast-radius audit + safe
  fix-shape decision.** `done` (2026-07-09, `PGEN-RAWCAP-TRANSFORM-PATH-0002`). NO code
  (Code-Change-Doctrine precursor). Outcome recorded in §3 "`.1` RESOLUTION":
  - Blast-radius audit DONE (grep `grammars/*.ebnf` + direct source read of the capture gate,
    default-view compile, and the `Or`/non-`Or` shared-flag). Regression set for (A) is NON-EMPTY
    (SV `declared_*` ×10 + regex `numeric_backreference`) ⇒ **(A) rejected**.
  - **DECISION = (C)** — a NEW positional-aware capture gate on the non-`Or` path only; shared
    `needs_raw_post_capture_for_rule` + the `Or` path untouched. Zero shipped positional-ref Raw-view
    predicates ⇒ **inert on all shipped grammars ⇒ zero regression by construction**.
  - Minimal reproducing pin DESIGNED (landed with `.2`, not now — a code precursor keeps `.1`
    no-code): a `parse_harness_semantic_suite` isolating grammar with a non-`Or` `->` Sequence rule
    carrying a POSITIONAL raw-view predicate, e.g.
    `pair := number brace_ws? "," brace_ws? number -> {min:$1, max:$5}` +
    `@predicate: {name: value_compare, args:[$1, le, $5], phase: post}` (default-Raw, positional).
    Pre-fix: `$1` hard-errors ("could not resolve attribute reference '$1'") ⇒ input `1,2` REJECTs.
    Post-fix: raw captured ⇒ `$1`≤`$5` ⇒ `1,2` ACCEPTs, `2,1` REJECTs — byte-identical across
    interpreter/generated by construction (differential proof surface). It is the un-worked-around
    twin of `.4.3`'s `counted_quantifier_range`, proving the raw-positional path directly.
- `.2` — **FIX (shape C) + regen-all + no-regression + lockstep.** `active` (pending was `.1`; now
  unblocked). Implementation, precise:
  1. Add `CompiledSemanticRuntime::needs_positional_raw_post_capture_for_rule(rule_name) -> bool`
     (`semantic_runtime.rs`, beside `needs_raw_post_capture_for_rule:1490`): true iff some
     `post_predicates_for_rule` directive has `predicate_view() == Some(Raw)` AND ≥1 arg is a
     positional `$N` ref (arg text starts `$`, remainder all-ASCII-digits — the
     `dollar_reference_is_positional` test at `ast_based_generator.rs:5809`). Do NOT touch
     `needs_raw_post_capture_for_rule`.
  2. In `rule_body_inner` (`ast_based_generator.rs:2917-2928`) emit a NEW capture token, Or/non-`Or`
     split MIRRORING the existing `semantic_span_transform_tokens` at `:2913-2916`
     (`ASTNode::Or => quote!{}`, else the capture): place
     `if <positional_gate> { semantic_raw_content = Some(result.clone()); }` BETWEEN `#parse_logic`
     (`:2924`) and `#post_parse_transform_tokens` (`:2928`). `generate_or_logic` UNCHANGED.
  3. Mirror the same positional-gated non-`Or` capture in the interpreter
     (`parse_harness_interpreter.rs:854`/`:2131`) so the differential-equivalence gate holds.
  4. Land the `.1` minimal pin in `parse_harness_semantic_suite` (22 → 23 isolating grammars) as the
     REJECT→ACCEPT differential proof.
  Then regen ALL parsers (gitignored) + rebuild dual `ast_pipeline` + release probe; prove:
  `parse_harness_equivalence_gate` 11 byte-identical, `parse_harness_semantic_gate` (new pin
  REJECT→ACCEPT + 24→25 constructs), SV cert union seeds 0/7/42 (`1343/10/1332/1`), regex cert 239,
  regex oracle `2189/1858/285/46`, duality unchanged, full dual lib, clippy. Because (C) is inert on
  shipped grammars, every shipped parser must regenerate byte-identical (the strongest no-regression
  proof) — any diff is a bug in the Or/non-`Or` split. Lockstep: normative spec (raw-capture now
  reaches the non-`Or` transform root for POSITIONAL raw-view predicates; named-ref shaped resolution
  unchanged) + the relevant book chapter(s) + `SEMREF-SHAPED` cross-reference (shaped-only for named
  refs preserved; positional refs were never in its scope) + CHANGES/DEV_NOTES/LIVE + this tree +
  TASK_TREE. **No engine release-version bump** — no shipped parser's frozen literal changes (proven
  by byte-identical regen).

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
| 1 | `.1` (evidence + blast-radius audit + fix-shape) | `done` | Audit done (§3 RESOLUTION): (A) rejected (regression set SV `declared_*` ×10 + regex `numeric_backreference`); **DECISION (C)** — positional-aware non-`Or` capture gate; zero shipped positional-ref Raw predicates ⇒ inert on shipped grammars. NO code. `PGEN-RAWCAP-TRANSFORM-PATH-0002`. |
| 2 | `.2` (fix shape C + regen + no-regression + lockstep) | `active` | Precise design in §4 `.2` (new `needs_positional_raw_post_capture_for_rule` helper + Or/non-`Or` split capture token + interpreter mirror + `parse_harness_semantic_suite` pin). Regen must be byte-identical for all shipped parsers (inert). |

## 7. Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-09` | `.1` (evidence + design) | Grep `@predicate` across `grammars/*.ebnf` (all Raw-view = non-`view:shaped` post-predicates enumerated); read owning rules (predicate binds FOLLOWING rule) for body-root/`->`/ref-kind. Source reads: `needs_raw_post_capture_for_rule` (`semantic_runtime.rs:1490`), default-view compile `None => Raw` (`:4085`), `predicate_view` (`:1088`), shared capture flag `:2918` vs `Or` capture `:3349/:3358/:3377`, positional detector `:5809`. `SEMREF-SHAPED` tree read (shaped-only named-ref contract). Regression-set grep for positional Raw predicates → EMPTY. | `pass — (A) rejected (non-empty regression set: SV declared_* ×10 + regex numeric_backreference resolve named refs via the SEMREF-SHAPED shaped-Json branch, which (A) would flip to a failing raw-tree walk). DECISION = (C): new positional-aware capture gate on the non-Or path only; Or path + shared helper untouched; inert on all shipped grammars (zero positional Raw predicates) ⇒ zero regression by construction. Minimal pin + full .2 implementation designed. No code (Code-Change-Doctrine precursor).` |

## 8. Decisions

- `2026-07-09` (`.1`, evidence-backed, autonomous within principles — the tree delegated A/B/C to
  `.1`): **fix-shape (C)** — a NEW positional-aware capture gate
  (`needs_positional_raw_post_capture_for_rule`) drives raw capture on the non-`Or` transform path
  ONLY; the shared `needs_raw_post_capture_for_rule` and the entire `Or` path are left byte-identical.
  Chosen over (A) [rejected — regresses the SV `declared_*` family + regex `numeric_backreference`,
  which resolve default-Raw NAMED refs against the shaped Json via `SEMREF-SHAPED`; (A) flips them to
  a failing raw-tree walk] and (B) [heavier — a global raw-first/shaped-fallback semantics that
  partially undoes `SEMREF-SHAPED`'s deliberate shaped-only decision]. (C) is zero-regression by
  construction: no shipped grammar has a positional-ref Raw-view post-predicate, so the new gate fires
  for zero rules and every shipped parser must regenerate byte-identical. Reconciles cleanly with
  [[SEMREF-SHAPED]] (named-ref shaped resolution untouched; positional refs were never in its scope)
  and unblocks [[POSITIONAL-PAYLOAD-REFS]] (F4) on the non-`Or` transform path.

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
