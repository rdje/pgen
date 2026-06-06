# Task Tree: LEXICAL-ANNOTATIONS (the 4th pillar)

> **Status:** `active` (2026-06-06). **Frontier:** `.3d` (on-by-default flag policy + operator-fusion)
> and/or `.3c` (declarative annotation, after the notation decision). `.3` Obligations A (`-0005`) + B
> (`-0006`) DONE. **Design:** [`LEXICAL-ANNOTATIONS-design.md`](LEXICAL-ANNOTATIONS-design.md).
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
  - **`.3d` (ii-CLI) on-by-default for the production CLI — DEFERRED (surface change).** Making
    `--generate-stimuli` faithful-by-default is NOT a simple flag flip: `args.enforce_word_boundary_spacing`
    is overloaded at `main.rs:770` as a "stimuli command present" sentinel, so inverting it misfires
    there. Needs a clean opt-out flag (e.g. `--no-word-boundary-spacing`) + the `:770` guard fix + book
    lockstep — its own deliberate slice.
  - **`.3d` (i) distinct-longer-token (operator) fusion — DEFERRED.** `<`+`<`→`<<` where the previous
    token is a fixed literal but a longer token spans the boundary; needs cross-terminal analysis (the
    leaf guard can't see it). Rare, not currently biting — a deliberate completeness pass when it bites.
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
