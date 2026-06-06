# Task Tree: LEXICAL-ANNOTATIONS (the 4th pillar)

> **Status:** `active` (2026-06-06). **Frontier:** `.4` verify+generalize (cross-grammar certificate-gate
> re-run); `.3d` (i) distinct-longer-token operator fusion stays open but has NO tool-backed failing case
> (don't change code speculatively) and is now declaratively expressible via the landed `[>! …]` path.
> **`.3d` (ii-CLI) DONE (`-0015`)** — `--generate-stimuli` is now faithful-by-default with a
> `--no-word-boundary-spacing` opt-out; verified across the affected stimuli gates (parity gate is the
> decisive canary). **`.3c` DONE (`-0013`)** — re-landed the declarative follow-restriction **COMPLETE** in ONE
> verified slice on the **per-rule** (before-rule) design the generator consumes: tokenizer
> (`ebnf_frontend.rs`) + IR handler (`mod.rs`, `FollowRestriction` carried per-rule in `Annotations`) +
> generator consumption (`stimuli_generator.rs`, FORBID self-terminates the rule surface with the
> minimal separator). Additive + gated on `[>` presence (0/17 grammars use it → byte-identical on all
> existing grammars; lib 610/610 no-features + 632/632 `ebnf_dual_run`, source clippy clean). REQUIRE
> (`[> …]`) is carried for the parse direction (documented generation no-op); the **inline** (per-element)
> form stays DEFERRED — per the annotation consumption matrix it is generator-unconsumable, so adding it
> tokenizer-only would re-create the `-0011` half-feature. Derivation half DONE + measured: A `-0005`
> (anchors), B `-0006` (regex-derived trailing guard), `-0007` (faithful-by-default); gate
> `sample_parse_failures` 25→1→0. Design + notation LOCKED.
> ⚠️ **Read the AST-pipeline KM cards ([[ast-pipeline-architecture]], [[ebnf-frontend-architecture]]) and
> [[feedback_understand_subsystem_holistically_first]] BEFORE touching any pipeline code.**
> **Design:** [`LEXICAL-ANNOTATIONS-design.md`](LEXICAL-ANNOTATIONS-design.md) ·
> [`research synthesis`](LEXICAL-ANNOTATIONS-research-synthesis.md).
> **Family / slice-id prefix:** `PGEN-LEXICAL-ANNOTATIONS-<NNNN>`.
> **Decision record:** [`project_lexical_annotations_fourth_pillar`](../decisions/project_lexical_annotations_fourth_pillar.md).
> **Book chapter:** [`docs/book/src/lexical-annotations.md`](../book/src/lexical-annotations.md).
> **SOTA survey:** [`LEXICAL-ANNOTATIONS-research-synthesis.md`](LEXICAL-ANNOTATIONS-research-synthesis.md).

## The frame (why this tree exists)

PGEN expresses a language with **three** declarative pillars:

1. **EBNF grammar** — the context-free *structure* (which token sequences are valid). text → tree.
2. **Return annotations** — the structure → *AST/output* mapping. tree → data.
3. **Semantic annotations** — context-sensitive *meaning* (types, scopes, the store). tree ⟷ meaning.

These three fully pin down the **tree** and its meaning. They say **nothing** about how that tree is
rendered back into **characters** — token separation, mandatory newlines, layout. That character↔token
layer is governed, for *parsing*, by the lexer's **maximal munch** (longest match), which is implicit
in regex matching and needs no annotation. For **generation** (tree → text) there is no maximal munch
to lean on, and nothing in pillars 1–3 supplies the missing discipline — so the generator can emit
text that does **not** re-lex to the tokens it intended.

This is a genuine **fourth pillar — LEXICAL (a.k.a. LAYOUT) ANNOTATIONS** — governing **surface
faithfulness**: *the characters emitted must re-lex (and re-parse) to exactly the tokens they were meant
to be.* The pillar is **bidirectional** — lexical-surface constraints can **steer both parsing and
generation** (SDF uses the same follow restrictions for parse-time disambiguation). **In PGEN today
they are needed for generation far more often**, because the parser already resolves most token
boundaries via **maximal munch** while the generator has **no** lexical discipline at all. So the
`.2`–`.4` work is generation-side enforcement; the parser both **honours** the constraints (maximal
munch) and **verifies** faithfulness (the certifying gate re-parses every generated sample) — designed
so the *same* annotations can serve parsing if ever needed. Prefer the name *lexical annotations*;
*layout annotations* is an accepted synonym.

### Origin (the two real defects that proved the gap)

Both surfaced from the `GRAMMAR-WELLFORMED.G.4` certificate-coverage gate parsing generator output:

- **Token fusion** (`G.4.7` slice 1/2): `endprogram`+`module` → `endprogrammodule`. Partially patched
  by enabling the *existing* `enforce_word_boundary_spacing` flag (a one-case, flag-driven prototype of
  this pillar — the smell that it wants to be first-class).
- **Comment swallow** (`G.4.7` slice 3): a generated `//` line comment with no terminating newline eats
  the rest of a single-line sample to EOF. Root cause confirmed at source: `generate_from_regex_hir`
  treats every regex anchor (`HirKind::Look(_)`) as the empty string and picks alternation branches
  uniformly, so for `line_comment := /\/\/[^\n]*(\n|$)/` it selects the `$` (end-of-input) branch and
  emits no newline. The `$` is *load-bearing for parsing* (a file may end `//x` with no newline — proven
  to parse), so the EBNF is correct; the defect is the generator not honouring the anchor.

Both are **one missing capability**, not two bugs: *the generator does not guarantee its output
re-lexes faithfully.* This tree makes that capability first-class, general, and parser-agnostic — per
the no-workarounds hierarchy this is a Level-5 addition (a new general parser-agnostic primitive),
justified because pillars 1–3 structurally cannot express it (see the decision record).

## Acceptance (tree-level)

- A **declarative + derived** lexical-faithfulness mechanism, parser-AGNOSTIC, that guarantees the
  generator's output re-lexes/re-parses to the intended token stream — subsuming
  `enforce_word_boundary_spacing` as a special case.
- The certificate-coverage gate (the verifier) shows `sample_parse_failures → 0` from the *general*
  mechanism (not per-case patches), on SV and on every grammar with a registered parser.
- Lockstep docs: this tree, the decision record, the book chapter, the SOTA survey, CHANGES.

## Leaves

- `.1` — **SCOPING + SOTA SURVEY** (`-0001`, this commit). Name the pillar; survey the literature
  (SDF2 follow restrictions / reject productions, scannerless lexical syntax, maximal munch, unparsing
  / pretty-printing, grammar-based test generation) and map each to PGEN; write the decision record +
  book stub. Establishes the cited ground per the research-grounded-SOTA discipline. **DONE.**
- `.2` — **DESIGN. DONE (`-0004`)** — see [`LEXICAL-ANNOTATIONS-design.md`](LEXICAL-ANNOTATIONS-design.md).
  The faithful-rendering invariant `LEX-FAITHFUL` (`render(t)` re-lexes/re-parses to `t`), achieved by
  **two derived obligations + one declarative escape hatch**: **(A) intra-token faithfulness** (each
  surface is a complete valid token — honor regex anchors `$`/`^` as assertions, prefer concrete
  alternation branches; fixes the comment-`$` bug at source); **(B) inter-token faithfulness** (between
  adjacent surfaces, insert the *minimal* separator — `""`→`" "`→`"\n"` — such that the previous token's
  regex doesn't over-match; derived by a local greedy-match boundary test, sound for PEG); **(C)** an
  optional declarative follow-restriction annotation for what derivation can't infer (notation TBD).
  Subsumes `enforce_word_boundary_spacing` (a crude special case of B). On by default; opt-out only for
  negative-test generation.
  - **NOTATION — DECIDED (`-0008`, with the director 2026-06-06).** `[> LIST ]` ("must be followed by")
    and `[>! LIST ]` ("must NOT be followed by"), where `LIST` is one or more items — each a `/regex/`
    or a `"string"`, freely **mixed** — separated by whitespace and/or commas; the list is a **union**
    (`[>! /\w/, "endmodule"]` = "not followed by any of these"). The enclosing `[ … ]` bounds the list
    cleanly — the director's rationale for the bracket over a bare sigil + single spec. Directive
    position (binds to the following rule). Unambiguous despite reusing `[ ]` because no
    `rule_expression` can start with `>` (so `[>`/`[>!` is never an optional). `[< ]`/`[<! ]` reserved
    for lookbehind. Chosen over `-/-` (SDF) and `~>`/`@>` for being bounded, list-friendly, readable.
    Full record: [[project_lexical_annotations_fourth_pillar]].
- `.3` — **IMPLEMENT — Obligations A + B (derivation only; no EBNF notation needed).**
  - **A — intra-token anchor honoring. DONE (`-0005`).** `generate_from_regex_hir`'s alternation
    picker now prefers branches that are NOT pure zero-width assertions (`regex_branch_is_anchor_only`
    + `regex_hir_can_produce_nonempty`/`regex_hir_contains_look`), so a bare `$`/`^`/`\b` branch is
    never realized inline when a concrete sibling exists — e.g. `(\n|$)` always yields `\n`. This is a
    GENERATOR change (no parser regen). **VERIFIED:** gate `sample_parse_failures 1 → 0`, deterministic
    ×2 (the comment-`$` swallow fixed at source); `witness 199 → 191` (RNG-path shift — different
    samples; all 50 now parse vs 49). Pinned by
    `obligation_a_line_comment_regex_always_terminates_with_newline`; 130 generator tests green.
  - **B — inter-token boundary separation. DONE (`-0006`)** (regex-derived intra-terminal trailing
    guard). `apply_word_boundary_spacing` is generalized from a `\b`-string match + space to a
    regex-DERIVED rule: an open-ended terminal (trailing `\b`, OR a greedy unbounded class repetition
    like `\w*`/`[0-9]+`/`[^\n]*`) self-terminates with the minimal separator its tail class cannot
    absorb (space, else newline). New helpers `regex_terminal_trailing_separator` / `regex_hir_tail` /
    `regex_tail_greedy_blocker` / `regex_class_contains`. This subsumes the `\b` special case into the
    principled regex test, at the leaf (no pipeline refactor), and covers greedy terminals for ANY
    grammar — not just those whose regex happens to end in `\b`. Flag-gated (low blast radius). VERIFIED:
    gate `sample_parse_failures` stays 0; `witness` unchanged at 191 (SV terminals already use `\b`, so
    SV output is identical — the win is principle + coverage of non-`\b` grammars); pinned by
    `obligation_b_regex_derived_trailing_separator`; 132 generator tests green.
  - **`.3d` (ii) on-by-default — LIBRARY/CONFIG default flipped. DONE (`-0007`).** `StimuliConfig::default()`
    now has `enforce_word_boundary_spacing: true` — lexical faithfulness is the default for any
    programmatic generator; negative-test generation opts out explicitly. **MEASURED: full lib suite
    607/607 green** (zero blast radius — the explicit-`false` tests are unaffected; the gate sets it
    `true` explicitly so is unchanged). Faithful is now the correct default at the type level.
  - **`.3d` (ii-CLI) on-by-default for the production CLI — DONE (`-0015`, 2026-06-06).** `--generate-stimuli`
    / `--generate-stimuli-module` are now **faithful by default**. Added a `--no-word-boundary-spacing`
    opt-out (`Args`, clap), a single-source-of-truth helper `effective_word_boundary_spacing(enforce, no_spacing)
    = enforce || !no_spacing` applied at all three `StimuliConfig`-construction sites (`main.rs:1121/1249/1304`)
    so the in-memory and generated-module paths stay consistent, and the new flag added to the
    stimuli-only-flag validation guard (`:774`) + its error message. The legacy
    `--enforce-word-boundary-spacing` stays accepted (now redundant; explicit-on still wins). **VERIFIED**
    (the blast-radius scoping below drove the gate selection): guard smoke (rejects `--no-word-boundary-spacing`
    without a stimuli command); `stimuli_module_parity_gate` ✅ (the decisive canary — in-memory vs compiled-module
    parity holds under faithful-by-default), `ast_dump_contract_gate` ✅, `annotation_stimuli_quality_gate` ✅,
    `ebnf_stimuli_quality_gate` ✅, `annotation_robustness_gate` ✅, `annotation_nonbootstrap_e2e_gate` ✅, the
    ebnf/hdl frontend readiness reports ✅; lib 610/610; source clippy clean. `sv_syntax_closure_gate` fails
    on `reachable_rules 1406 < min 1407` — **PRE-EXISTING** static grammar/contract drift (decisively cleared:
    `grammars/systemverilog.ebnf` + the closure contract + the analyzer are byte-unchanged since pre-session,
    and this slice touches none of them; owned by GRAMMAR-WELLFORMED A2.1, not this slice). Book chapter
    documents the CLI default-on + opt-out.
  - **(historical) `.3d` (ii-CLI) DEFERRAL note.** Making
    `--generate-stimuli` faithful-by-default is NOT a simple flag flip: `args.enforce_word_boundary_spacing`
    is a clap bool flag defaulting `false` (`main.rs:243`), passed straight into `StimuliConfig` at three
    sites (`main.rs:1116/1241/1293`), so the **CLI** is still faithful-OFF by default even though the
    library `StimuliConfig::default()` flipped to `true` (`-0007`). It is also listed in the
    stimuli-only-flag validation guard at `main.rs:770`. Clean design: add a `--no-word-boundary-spacing`
    opt-out (`#[arg(long)]`, default false), set the effective value to `!args.no_word_boundary_spacing`
    at the 3 sites, add the new flag to the `:770` guard, keep `--enforce-word-boundary-spacing` accepted
    (now redundant) for back-compat, + CLI-reference/book lockstep.
    **⚠️ BLAST-RADIUS SCOPING (`-0013`, verified):** this CHANGES the default output of `--generate-stimuli`
    (separators now inserted) — NOT zero-blast-radius. **9 of 12** stimuli-invoking gate scripts under
    `rust/scripts/` call `--generate-stimuli` WITHOUT the flag (only `sv_stimuli_quality_gate.sh`,
    `sv_preprocessor_quality_gate.sh`, `vhdl_stimuli_quality_gate.sh` pass it). So this slice MUST run
    each affected gate (the heavy SV/VHDL/annotation/ebnf stimuli gates) to confirm the faithful-by-default
    flip is a no-op-or-improvement for them (and update any that pin exact byte output). A deliberate slice
    with real verification cost — do it with the heavy gates, not under a tight budget.
  - **`.3d` (i) distinct-longer-token (operator) fusion — DEFERRED.** `<`+`<`→`<<` where the previous
    token is a fixed literal but a longer token spans the boundary; needs cross-terminal analysis (the
    leaf guard can't see it). Rare, not currently biting — a deliberate completeness pass when it bites.
- `.3c` — **Obligation C (declarative follow-restriction annotation). DONE (`-0013`, 2026-06-06).**
  Re-landed COMPLETE on the **per-rule (before-rule)** design — the re-land RULE satisfied (tokenizer +
  IR handler + generator consumption in ONE verified slice, never tokenizer-only). What landed:
  1. **Frontend** (`src/ebnf_frontend.rs`): `scan_top_level_rules` collects column-0 `[>` / `[>!`
     directive lines (pending → bound to the next rule, exactly like `@` annotations);
     `parse_lexical_annotation_line` parses the `LIST` (mixed `/regex/` + `"string"`, ws/comma-separated,
     union) reusing `parse_regex_literal` / `parse_quoted_literal`; `convert_scanned_rule` emits a
     structured `["lexical_annotation", {polarity, items}]` token; the soft generated cross-check is
     gated off when lexical annotations are present (like `has_inline_semantic_annotations`). The
     body-level `[ … ]` optional tokenization is UNTOUCHED (directives are stripped at column 0 before
     the body is seen) → low blast radius.
  2. **IR** (`src/ast_pipeline/mod.rs`): `FollowRestriction { forbid, items: Vec<FollowItem> }` +
     `FollowItem::{Regex,Literal}`; `Annotations.lexical_follow_restrictions: HashMap<String,
     FollowRestriction>` (additive, `#[serde(default)]`); `extract_rule_annotations` has an explicit
     `"lexical_annotation"` arm (BEFORE the catch-all — closes the exact `-0011` IR-corruption hazard)
     that parses the payload (malformed ⇒ hard error, never a silent half-token); threaded through
     `ParsedRuleContent` → `transform_from_raw_ast` (incl. the annotations-present gate).
  3. **Generator** (`src/ast_pipeline/stimuli_generator.rs`): `generate_rule`'s output self-terminates per
     a FORBID restriction with the minimal separator the forbidden set can't absorb (space→newline),
     robust against every concatenation path; REQUIRE is a documented generation no-op (parse-direction).
     Gated on the lexical-faithfulness mode (`enforce_word_boundary_spacing`, default on) → negative-test
     generation opts out uniformly.
  - **Why per-rule only (inline DEFERRED):** the annotation consumption matrix
    ([[ast-pipeline-architecture]]) makes position-specific (inline) annotations codegen-only — the
    stimuli generator cannot consume them. A follow-restriction exists to steer the GENERATOR, so the
    inline form would be generator-unconsumable; adding it tokenizer-only would re-create the `-0011`
    half-feature. The before-rule (per-rule) form is the complete, generator-consumable unit.
  - **VERIFIED:** lib 610/610 (no-features, +3 tests) + 632/632 (`ebnf_dual_run`, +2 frontend tests);
    `generated_parsers` compiles; source clippy clean (generated-clippy `==` debt pre-existing,
    non-strict). Zero blast radius by construction (0/17 grammars declare `[>` → byte-identical
    generation; the helper is a pass-through when no restriction is declared). New tests:
    `parses_lexical_follow_restriction_directives`, `before_rule_lexical_annotation_binds_following_rule`
    (frontend); `transform_from_raw_ast_carries_lexical_follow_restriction` (IR);
    `forbid_follow_separator_picks_minimal_breaking_char`,
    `lexical_forbid_follow_restriction_prevents_distinct_longer_token_fusion` (generator, end-to-end).
  - The full cross-grammar certificate-coverage gate re-run (`sample_parse_failures` across every
    grammar with a registered parser) is the dedicated **`.4`** leaf; SV is unaffected here **by
    construction** (no grammar declares `[>`).
  -----
  **(historical) `.3c` step-1 (`-0011`) REVERTED after audit (`-0012`):** the `-0011` inline tokenizer
  emitted a `lexical_annotation_inline` token with **no downstream handler** (catch-all `_ =>` would
  corrupt the IR the moment any grammar used `[>`); dormant-safe only because 0/17 grammars used it.
  **Re-land RULE (now satisfied):** land COMPLETE (tokenizer + IR handler + generator consumption) in
  one verified slice, on the per-rule design the generator can actually consume — never tokenizer-only.
  ⚠️ **Corrected understanding (`-0010`, see KM [[ebnf-frontend-architecture]]):** PGEN's
  authoritative EBNF parser is the **hand-written `src/ebnf_frontend.rs`** — NOT the generated
  `generated/ebnf.rs` (that's a soft, non-fatal cross-check). So this is a **Rust-code change to the
  hand-written frontend + the IR converter, NOT a bootstrap regen** (my earlier "regen the bootstrap
  parser" plan was wrong). Steps:
  1. **Hand-written frontend** (`src/ebnf_frontend.rs`): teach the tokenizer the new syntax.
     - Inline (binds preceding): in `tokenize_rule_expression` the `'['` case currently always opens an
       optional (`[X]` → `"(" … ")" "?"`); branch it — peek the next non-space char, if `>` parse a
       `[> … ]` / `[>! … ]` lexical annotation and emit a `lexical_annotation_inline` token; else the
       existing optional. (Unambiguous: an optional body can't begin with `>`.)
     - Before-rule (binds the rule): in `scan_top_level_rules` the loop collects `@`-prefixed lines as
       `pending_annotations`; also collect `[>`/`[>!` lines; `convert_scanned_rule` emits a
       `lexical_annotation` token.
     - Gate the generated cross-check on lexical-annotation presence (like `has_inline_semantic_annotations`
       at ebnf_frontend.rs:53) so it doesn't spuriously warn.
  2. **IR** (`ast_pipeline/mod.rs::transform_from_raw_ast`, :1355): recognize the new token type → carry
     the follow-restriction (polarity + item list) onto the rule/element IR. Optionally keep
     `grammars/ebnf.ebnf` in sync as documentation/seed (not required for the live parser).
  3. **Generator** (Obligation B path): consult the declared follow-restrictions alongside the derived
     ones — `[>! …]` forbids the listed follows (insert a separator if the next token would match one);
     `[> …]` requires one of the listed follows. This also gives a DECLARATIVE answer to the deferred
     operator-fusion case (`.3d` (i)) without cross-terminal analysis.
  4. Tests + book examples; verify the gate stays at `sample_parse_failures` 0.
  Notation: `[> LIST ]` / `[>! LIST ]`, `LIST` = mixed `/regex/`+`"string"` items, ws/comma-separated.
- `.4` — **VERIFY + GENERALIZE.** Re-run the certificate-coverage gate (the verifier): SV
  `sample_parse_failures → 0` from the general mechanism; then every grammar with a registered parser
  (ties into `GRAMMAR-WELLFORMED` Phase H). Retire the comment-newline and word-fusion special cases as
  now-covered instances. Add round-trip / golden tests.

## Cross-links

- Subsumes / closes: `GRAMMAR-WELLFORMED.G.4.7` slice 2 (`enforce_word_boundary_spacing`) and slice 3
  (comment-newline) as instances of the general pillar.
- Feeds: [[project_stimuli_generator_signoff_vision]] — a concrete, named entry in the signoff
  capability-gap audit.
- Disciplines: [[feedback_no_workarounds_fix_hierarchy]] (Level-5, justified),
  [[feedback_prefer_grammar_leave_engine_alone]] (refined — a general parser-agnostic engine feature is
  allowed), [[feedback_research_grounded_sota_no_trial_and_revert]] (survey first),
  [[feedback_ast_pipeline_parser_agnostic]] (absolute).
