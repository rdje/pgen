# SEMREF-SHAPED: a semantic annotation on rule X must resolve refs against the structure X returns

## Metadata

- Tree ID: `SEMREF-SHAPED`
- Status: `active`
- Roadmap lane: AST-pipeline / semantic-runtime resolver correctness
  (shared engine; parser-agnostic)
- Created: `2026-05-18`
- Last updated: `2026-05-18`
- Owner: repo-local workflow
- Priority: user-directed — "Any gap in the expected behavior in the
  way [return & semantic annotations] interact shall be addressed head
  on"; no workaround. `RGX-0084.2` depends on this.
- Renamed from `CODEGEN-RAWCAP` (2026-05-18): the original
  "capture raw `Sequence`" framing was corrected after the user's
  clarifying question + design decision — the goal is **shaped-output
  resolution**, not raw capture. Renamed while brand-new + uncommitted
  (no commit-ID churn).

## Goal

A semantic annotation (`@predicate` / `@emit_fact`) on rule **X**
resolves each `$name` (and dotted `$a.b` path) argument **against the
structure rule X produces**:

- **X has a `->` return annotation** → `$name` is resolved **strictly
  against the shaped `->` output** of X: look up `name` as a
  field/key of the produced object (scalar field → its scalar value;
  nested object → continue the path). **Shaped-only — no raw
  fallback** (user decision, 2026-05-18: a `$name` that is not a field
  of X's produced output is a clean resolution failure, never a
  silent reach into internal parse plumbing). The rule's declared
  output IS its semantic surface.
- **X has no `->`** → unchanged existing behavior: `$name` resolves
  against X's raw parse content by sub-rule `rule_name`
  (`find_semantic_named_descendant` over `Sequence`/`Quantified`/
  `Alternative`).

This is a **general, parser-agnostic** semantic-runtime resolver
capability (binding `feedback_ast_pipeline_parser_agnostic`): any
grammar may put a `->` and a content-ref'ing directive on one rule and
have the directive read the rule's declared output. `RGX-0084` is the
first consumer, not the motivation for any regex-specific behavior.

## The gap (exact)

A semantic annotation on a rule X **cannot read any value out of the
structure X's `->` produces**, because the engine has no code path
that looks a `$name` up inside a shaped result.

1. `resolve_named_semantic_reference` →
   `find_semantic_named_descendant`
   (`rust/src/ast_pipeline/ast_based_generator.rs:3594 / 3662`) matches
   a descendant whose `rule_name == "<name>"` and only recurses into
   `ParseContent::Sequence | Quantified | Alternative`; its final arm
   is `_ => None`. **There is no "look up key `<name>` in the produced
   object" logic anywhere.**
2. For a `->` rule, `node.content` **is** `ParseContent::Json(<shaped
   object>)` (e.g. `numeric_backreference` →
   `{type:"backreference",kind:"numeric",index:"1"}`). The resolver
   returns `None` on `Json` → `"Semantic runtime could not resolve
   attribute reference '<name>'"` → directive errors → rule
   backtracks. (Exactly the observed RGX-0084.2 failure.)
3. Secondary / now mostly moot under shaped-only: the raw pre-shaping
   content is not a usable fallback anyway — `semantic_raw_content` is
   declared but **never assigned** for single-branch rule bodies
   (`grep -c 'semantic_raw_content = Some(' generated/regex_parser.rs
   generated/systemverilog_parser.rs` → `0`, `0`; only the
   multi-branch path assigns it via `= best_raw_content`). This is
   also why the interim no-`->` `gated_backreference_digits` wrapper
   failed and why the assumed-proven SV `declared_X` +
   `@predicate has_fact[…, $X]` content-ref idiom is itself
   non-functional for *positive* resolution (SC gates evidently do
   not exercise it). ⇒ There is no working content-ref idiom to fall
   back to; the resolver must be fixed. **No workaround.**

## Non-Goals

- Not changing the semantic predicate/effect vocabulary.
- Not raw-content *capture* codegen work (the original mis-framing):
  shaped output is already `node.content`; the fix is resolver-side.
- Not changing no-`->` raw resolution behavior (must stay identical;
  proven by differential regen).
- Not RGX-0084 PCRE2 semantics (RGX-0084 is the first *consumer*).
- No regex/parser-specific identifiers in the engine.

## Acceptance Criteria

- Resolver resolves `$name` and dotted `$a.b.c` against the
  `ParseContent::Json` a `->` rule produced: scalar field → its
  scalar string; intermediate object → descend; **field absent → a
  deterministic resolution failure with the SAME downstream semantics
  the engine already defines for an unresolved ref** (decided + spec'd
  in `.1`; must not panic; must be consistent + documented).
- No-`->` rules: `$sub_rule` resolution **byte-identical** to today
  (differential regen proof across every generated parser).
- Parser-agnostic: zero grammar-specific identifiers added to the
  engine; the capability is usable by any grammar.
- Engine/codegen change owns its full gate set, same-slice: `cargo
  test` (incl. resolver unit tests: scalar field, nested path, absent
  field, no-`->` raw unchanged), SC-01..SC-07, every parser AST-shape
  contract, every per-parser book gate + `mdbook_docs_gate`,
  `ebnf_frontend_dual_run_diff_gate`, performance gate.
- Strictly-more-capable: every existing grammar's generated output and
  parse behavior unchanged except where a rule legitimately uses the
  now-working `->`+content-ref combination (differential proof).
- Lockstep same-commit: semantic-annotation book
  (`annotation-system.md`/`parser-hooks.md`) + annotation normative
  spec + relevant integration contract document the
  "semantic refs resolve against the rule's returned structure
  (shaped-only for `->` rules)" contract + a worked example;
  CHANGES/DEVELOPMENT_NOTES/LIVE/memory.
- Tree closed + Completed; `RGX-0084.2` unblocked to adopt the clean
  same-rule form (`@predicate fact_count_at_least(regex_capture_group,
  $index)` on `numeric_backreference`).

## Task Tree

- ID: `SEMREF-SHAPED`
  Status: `active`
  Goal: `Semantic-annotation $name/$a.b refs on rule X resolve against X's produced structure (shaped-only for ->; raw unchanged for no->); parser-agnostic; zero cross-parser regression; lockstep; unblock RGX-0084.2.`
  Children: `SEMREF-SHAPED.1`, `SEMREF-SHAPED.2`, `SEMREF-SHAPED.3`

- ID: `SEMREF-SHAPED.1`
  Status: `done`
  Goal: `Pinpoint + repro + DESIGN/semantics decision (no code).`
  Acceptance: `done — LOCUS PINNED: rust/src/ast_pipeline/ast_based_generator.rs::resolve_named_semantic_reference (line 3594). Minimal additive design: add a leading branch — if root_content is ParseContent::Json(value), walk the dotted reference as serde_json object-key path segments (each segment must be a semantic_identifier; cur = cur.get(segment)), then coerce the leaf to a scalar String (String→clone; Number/Bool→to_string; Null/Object/Array→None); else fall through to the EXISTING find_semantic_named_descendant raw path COMPLETELY UNCHANGED. Repro/route confirmed from prior high-traces: for a `-> {…}` rule, semantic_raw_content is None (never assigned for single/sequence bodies — grep -c 'semantic_raw_content = Some(' = 0/0), so the post-predicate's raw_content = semantic_raw_content.unwrap_or(&node.content) = node.content = the shaped ParseContent::Json; the resolver is already handed the Json — find_semantic_named_descendant just returns None on it (no Json arm). So NO codegen / view / raw-capture change is needed — purely a resolver addition. CONTENT-VARIANT-KEYED: Json ⟺ object-producing `->`; raw Sequence/Quantified/Alternative/Terminal ⟺ no-`->` (or passthrough). The new branch only fires on the Json variant ⇒ no-`->` path provably byte-identical (no `->`/passthrough rule ever yields a ParseContent::Json object root). SEMANTICS DECIDED (autonomous, principled, to be book-documented): (a) absent key / Null / non-scalar (Object|Array) leaf → resolver returns None → existing resolve_unified_semantic_value_against_content maps None → the SAME hard ContextualError "could not resolve attribute reference '<ref>'" used today ⇒ rule backtracks. Rationale: a directive referencing a field the rule's `->` does not produce (or a non-scalar) is an author/grammar bug — fail LOUD + consistent with current unresolved-ref behavior, never silently mis-parse, never panic. NOTE this is resolution failure, distinct from predicate EVALUATION returning false; RGX-0084's $index always resolves (numeric_backreference's `->` always emits index) so its gate is pure count true/false, never this error. (b) scalar coercion mirrors the existing semantic_content_scalar Json arm (String as-is; Number/Bool to_string; Null→None). (c) dotted $a.b.c descends object keys; intermediates must be objects; leaf must be scalar. (d) parser-agnostic: zero grammar-specific identifiers; any grammar's `->`-object rule benefits. Minimal-repro for .2 = a focused resolver UNIT test constructing a ParseContent::Json + asserting resolve_named_semantic_reference (scalar field / nested path / absent / Null / non-scalar) + a Sequence/Alternative case asserting the raw path is byte-unchanged — no new registered grammar needed.`
  Verification: `done — see Verification Log 2026-05-18 (.1)`
  Commit: `PGEN-SEMREF-SHAPED-0001`

- ID: `SEMREF-SHAPED.2`
  Status: `pending`
  Goal: `Implement the resolver shaped-structure access per .1 (for -> rules, resolve $name/$a.b as field/key lookups into the produced ParseContent::Json; no-> rules unchanged). Resolver unit tests (scalar field, nested path, absent field semantics, no-> raw unchanged) + the minimal repro grammar resolves+gates. Regenerate ALL parsers.`
  Acceptance: `repro resolves+gates; resolver unit tests green; all parsers regenerated; cargo test green. Cross-parser no-regression proven in .3.`
  Verification: `pending`
  Commit: `pending`

- ID: `SEMREF-SHAPED.3`
  Status: `pending`
  Goal: `Cross-parser no-regression + full lockstep + closeout. Differential proof (no-> behavior byte-identical everywhere; only legitimately-using rules change). SC-01..SC-07; every AST-shape contract; every per-parser book gate + mdbook_docs_gate; ebnf_frontend_dual_run_diff_gate; performance gate. Semantic-annotation book + normative spec + integration contract document the contract + worked example. CHANGES/DEVELOPMENT_NOTES/LIVE/memory. Tree closed; RGX-0084.2 unblocked.`
  Acceptance: `all gates green; differential proof recorded; book/spec/contract lockstep same-commit; tree Completed; RGX-0084.2 adopts the clean same-rule $index form.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SEMREF-SHAPED.2` | `pending` | `.1` DONE: locus pinned (`resolve_named_semantic_reference:3594`), minimal additive Json-branch design + semantics locked. `.2` implements the branch + resolver unit tests + regen all parsers (Code-Change Doctrine: `.2` owns the engine edit). |
| 2 | `SEMREF-SHAPED.3` | `pending` | Cross-parser differential no-regression + book/spec/contract lockstep + closeout; unblocks RGX-0084.2. |

## Decisions

- `2026-05-18`: Discovered during `RGX-0084.2`. Initial framing
  ("capture raw `Sequence` before the `->` shadow", tree
  `CODEGEN-RAWCAP`) was **corrected** after the user's clarifying
  question: the goal is that a semantic annotation on X reads the
  structure X's `->` returns — *shaped-structure resolution*, not raw
  capture. Tree renamed `CODEGEN-RAWCAP` → `SEMREF-SHAPED`
  (brand-new, uncommitted; no commit-ID churn).
- `2026-05-18` (**user decision, binding**): for a `->` rule,
  `$name` resolves **shaped-only** — strictly against the produced
  structure's fields, with **no raw fallback**. A `$name` that is not
  a field of X's output is a clean resolution failure, never a silent
  reach into internal parse plumbing. (Selected over "shaped, then raw
  fallback".) Keeps the rule's declared output as its semantic
  contract; predicates do not couple to grammar-internal names.
- `2026-05-18` (**user directive, binding**): no workaround. The
  interim `gated_backreference_digits = backreference_digits` no-`->`
  wrapper is **rejected as RGX-0084's final form**; empirically it
  does not even work (single-branch raw capture is never emitted), so
  there is no working idiom to fall back to — the resolver gap is
  fixed head-on here.
- `2026-05-18`: Own tree (parser-agnostic engine capability,
  `feedback_ast_pipeline_parser_agnostic`), not an RGX-0084 leaf.
  `SEMREF-SHAPED` is the active frontier; `RGX-0084` is
  dependency-blocked on it; `SV-EXH-PROOF` stays paused.
- `2026-05-18`: Strictly-more-capable change — gated on "X has a
  `->`"; no-`->` resolution path provably untouched (differential
  regen). "Release bump, no schema bump" category.

## Open Questions

- ~~Absent-field semantics~~ **DECIDED (`.1`, binding):** absent
  key / `Null` / non-scalar (`Object`/`Array`) leaf → resolver `None`
  → the existing hard `ContextualError "could not resolve attribute
  reference '<ref>'"` (option (a), loud) → rule backtracks. Rationale:
  referencing a field the `->` does not produce, or a non-scalar, is
  an author/grammar bug — fail loud + consistent with today's
  unresolved-ref path; never silent, never panic. Distinct from
  predicate *evaluation* false. `.3` documents this in the book.
- ~~Scalar vs structured field~~ **DECIDED (`.1`):** only scalar
  leaves are valid as a directive arg — `String`→clone,
  `Number`/`Bool`→`to_string`, `Null`/`Object`/`Array`→`None`
  (→ ContextualError), mirroring the existing `semantic_content_scalar`
  Json arm; dotted `$a.b.c` descends object keys (intermediates must
  be objects).
- **Open for `.1`-grep follow-up (carried to `.3`):** does any
  existing grammar already put a content-ref directive on a `->`-object
  rule and silently error/no-op today (a latent bug this fix would
  correct → that grammar's contract/book joins `.3` lockstep)? Sweep
  `grammars/*.ebnf` in `.2`/`.3`; current known set: regex (the
  intended RGX-0084 consumer) — to be wired *after* this lands.

## Blockers

- None for scoping/implementation (repro + resolver locus
  self-contained). `RGX-0084.2`'s final clean grammar form is blocked
  on `SEMREF-SHAPED.3` closing.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-18` | `SEMREF-SHAPED` (setup) | Discovered in RGX-0084.2; root cause evidenced (`find_semantic_named_descendant` `_ => None` on `Json`; `->` rule content == that `Json`; `semantic_raw_content` never assigned single-branch — `grep -c` 0/0; SV `declared_*` idiom non-functional for positive content-ref). User clarified goal = shaped-output resolution; user decided shaped-only (no raw fallback); user directive no-workaround. Initial raw-capture framing corrected; tree renamed `CODEGEN-RAWCAP`→`SEMREF-SHAPED`. | `pass — gap precisely characterized + contract decided (shaped-only); 3 leaves (pinpoint+semantics → resolver impl → cross-parser proof+lockstep+closeout). No code (Code-Change-Doctrine precursor).` |
| `2026-05-18` | `SEMREF-SHAPED.1` | Read `resolve_named_semantic_reference` (3594) / `find_semantic_named_descendant` (3662) / `semantic_content_scalar` (3691) + the post-predicate resolution chain (`view` Raw → `semantic_raw_content.unwrap_or(&node.content)`); confirmed from prior high-traces that a `->`-object rule's resolver root_content IS the `ParseContent::Json` (raw is None). Pinned the minimal additive locus + content-variant-keyed design (Json branch in `resolve_named_semantic_reference`; raw path untouched; no codegen/view/raw-capture change). Decided absent-field + scalar-coercion + dotted-path + parser-agnostic + no-`->` invariance semantics. Reverted the rejected RGX-0084.2 workaround grammar (`git checkout grammars/regex.ebnf`); kept P1 (`fact_count_at_least`, inert without wiring). | `pass — locus + minimal design + semantics LOCKED; no code (Code-Change-Doctrine precursor for .2). Frontier → .2.` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SEMREF-SHAPED` (setup) | `PGEN-SEMREF-SHAPED-0000` | tree created (renamed from CODEGEN-RAWCAP) + activated; frontier `.1`; RGX-0084.2 dependency-blocked; SV-EXH-PROOF paused |
| `SEMREF-SHAPED.1` | `PGEN-SEMREF-SHAPED-0001` | pinpoint + locus (`resolve_named_semantic_reference:3594`) + minimal additive Json-branch design + semantics decisions (absent→ContextualError, scalar-only, dotted path, no-`->` invariance); rejected workaround grammar reverted, P1 kept. No code. Frontier → `.2` |

## Changelog

- `2026-05-18`: Created (as `CODEGEN-RAWCAP`), then **renamed +
  retargeted** to `SEMREF-SHAPED` after the user's design
  clarification. Real goal: a semantic annotation on rule X resolves
  `$name`/`$a.b` against the structure X's `->` returns
  (**shaped-only** for `->` rules, per user decision; raw unchanged
  for no-`->`). Gap = resolver has no shaped-`Json` field lookup
  (`find_semantic_named_descendant` `_ => None` on `Json`).
  Decomposed `.1` pinpoint+semantics-decision → `.2` resolver impl +
  tests → `.3` cross-parser no-regression + book/spec/contract
  lockstep + closeout. RGX-0084.2 depends on this; active frontier
  (RGX-0084 dependency-blocked, SV-EXH-PROOF paused).
  Code-Change-Doctrine precursor — no code yet.
