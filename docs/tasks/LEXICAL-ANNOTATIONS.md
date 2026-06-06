# Task Tree: LEXICAL-ANNOTATIONS (the 4th pillar)

> **Status:** `active` (2026-06-06). **Frontier:** `.3` IMPLEMENT (Obligations A + B; derivation-only,
> no EBNF notation needed). **Design:** [`LEXICAL-ANNOTATIONS-design.md`](LEXICAL-ANNOTATIONS-design.md).
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
  - **Open design question — the NOTATION (director-raised 2026-06-06, deferred to `.2`).** Return
    annotations use `->`, semantic annotations use `@`; lexical annotations need their own EBNF syntax.
    The name/syntax will stick for years, so decide deliberately. Candidates: adopt/adapt **SDF's
    follow-restriction operator `-/-`** (`A -/- [chars]` — literature-grounded, ties straight to the
    survey) vs. mint a fresh sigil. Constraints: must read distinctly from `->` and `@`; attaches at
    the terminal/token level (lexical constraints are about token boundaries); and since most
    faithfulness is **derived** from the regexes, the notation only ever appears for the *rare explicit*
    declaration — so it can be lightweight. Decide with the director.
- `.3` — **IMPLEMENT (NEXT) — Obligations A + B (derivation only; no EBNF notation needed).** A: honor
  regex anchors in `generate_from_regex_hir` (anchor arm + alternation branch selection). B: track the
  last emitted terminal's regex; replace the char-class word-boundary check with the regex boundary test
  + minimal-separator ladder; subsume `enforce_word_boundary_spacing`. Regenerate parsers; commit codegen
  only (generated/ is local). **Unblocked by the notation decision** — A+B are pure derivation.
- `.3c` — **Obligation C (declarative follow-restriction annotation).** Deferred until the EBNF
  **notation** is agreed with the director (see `.2` note). Then: EBNF surface → annotation compiler →
  follow-restriction table consulted by Obligation B.
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
