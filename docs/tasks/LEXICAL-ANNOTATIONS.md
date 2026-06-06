# Task Tree: LEXICAL-ANNOTATIONS (the 4th pillar)

> **Status:** `active` (2026-06-06). **Frontier:** `.2` DESIGN (begins once `.1` survey lands).
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
to be.* It is **enforced in generation**; the parser **already honours** it via maximal munch and now
**verifies** it (the certifying gate re-parses every generated sample). Prefer the name *lexical
annotations*; *layout annotations* is an accepted synonym.

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
- `.2` — **DESIGN (NEXT).** The parser-agnostic primitive: the *faithful-rendering invariant*
  (`render(t)` re-lexes to `t`), with separation **auto-derived from the token regexes** the generator
  already holds (does B extend A's match? does A's terminator require a newline?), plus an optional
  **declarative follow-restriction lexical annotation** (SDF-style) for what derivation can't infer.
  Define how `enforce_word_boundary_spacing` is subsumed. Output: a design doc + the annotation surface
  (if any) specified in `grammars/*.ebnf` terms. NO code yet.
- `.3` — **IMPLEMENT.** The generation-side enforcement in the engine (parser-agnostic): on emitting
  each token, insert the minimal separator (space → newline) that preserves faithfulness; honor regex
  anchors (`$`/`^`/`\b`) as position assertions rather than free empties. Regenerate parsers; commit
  codegen only (generated/ is local).
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
