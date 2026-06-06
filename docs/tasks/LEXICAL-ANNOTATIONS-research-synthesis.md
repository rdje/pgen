# LEXICAL-ANNOTATIONS — SOTA research synthesis (leaf `.1`)

> Companion to [`LEXICAL-ANNOTATIONS.md`](LEXICAL-ANNOTATIONS.md). Per the research-grounded-SOTA
> discipline ([[feedback_research_grounded_sota_no_trial_and_revert]]): survey the published
> literature and produce a *worked mapping* to PGEN **before** designing the mechanism.

## 0. The problem, named precisely

The defect class is **lexical-syntactic faithfulness across the parse↔generate boundary**:

- A *parser* turns characters into a token/parse structure. The character→token step is governed by a
  **lexical-disambiguation** discipline (chiefly **maximal munch** / longest match).
- A *generator* (unparser) turns a structure back into characters. To be correct, the produced text
  must **re-lex/re-parse to the structure it came from** — the **round-trip property**
  `parse(render(t)) ≅ t`.

PGEN's three pillars (EBNF, return annotations, semantic annotations) specify the *structure* and its
*meaning*. None specifies the **lexical surface** — how tokens are separated into characters so the
round-trip holds. Parsing gets this for free (maximal munch); generation does not, which is exactly
where PGEN's generator produced invalid output.

## 1. The parsing side — lexical disambiguation (well-established)

**Maximal munch / longest match.** The standard lexing rule: consume the longest prefix that matches a
token. It silently resolves most token-boundary questions — and it is precisely why
`endprogrammodule` lexes as *one* identifier rather than `endprogram` + `module`. Formalized in the
scannerless setting by **Salomon & Cormack, "Scannerless NSLR(1) parsing of programming languages,"
SIGPLAN Notices 24(7):170–178, 1989.** ([Maximal munch — Wikipedia](https://en.wikipedia.org/wiki/Maximal_munch))

**Declarative lexical constraints — SDF2 follow restrictions & reject productions.** Eelco Visser's
**SDF2** ("A Family of Syntax Definition Formalisms"; "Scannerless Generalized-LR Parsing," 1997)
integrates lexical and context-free syntax into one grammar and adds two *declarative* disambiguation
constructs:
- **follow restrictions** — `A -/- [c]`: a token of sort `A` may **not** be immediately followed by a
  character in class `[c]`. This is the declarative way to express longest-match (an identifier can't
  be followed by an identifier char → forces a boundary).
- **reject productions** — express keyword reservation ("prefer keywords").

These are filters over the parse forest, formalized in **van den Brand, Scheerder, Vinju & Visser,
"Disambiguation Filters for Scannerless Generalized LR Parsers," CC 2002**, and carried into **SDF3**
(Amorim & Visser, SEFM 2020). ([SGLR](https://researchr.org/publication/Visser97-SGLR) ·
[Disambiguation Filters (CC 2002)](https://homepages.cwi.nl/~jurgenv/papers/CC-2002.pdf) ·
[SDF3](https://eelcovisser.org/publications/2020/AmorimV20.pdf))

**Mapping to PGEN.** PGEN is effectively *scannerless* — token regexes live inline in the `.ebnf`, and
the runtime matches them with longest-match semantics. So the parsing side **already implements the
maximal-munch half**; it needs no new lexical mechanism to parse valid SV, and it is the natural
**verifier** of the round-trip property. The key reusable idea is the **follow restriction**: a
declarative, parser-agnostic statement of a token's boundary constraint. PGEN already has a *degenerate*
form of it — the `\b` word-boundary convention read by `enforce_word_boundary_spacing`.

## 2. The generation side — unparsing / pretty-printing (well-established)

**Pretty-printing** is the inverse: structure → text with layout (whitespace/newlines).
- **Oppen, "Prettyprinting," ACM TOPLAS 2(4):465–483, 1980** — the canonical linear-time, bounded-
  lookahead algorithm over a token stream with explicit layout primitives.
  ([Oppen 1980](https://dl.acm.org/doi/pdf/10.1145/357114.357115))
- **Wadler, "A prettier printer," 2003** — the functional reformulation; the basis of industrial
  formatters (Prettier). ([A prettier printer](https://www.semanticscholar.org/paper/A-prettier-printer-Wadler-Kilmer/23e0f52ebdd5dad33e61009afce1ecae3309eefb))

The load-bearing point for us: a correct unparser/formatter **must insert enough separation that the
output re-lexes to the same tokens.** Formatters get this implicitly by knowing the lexical rules; a
*grammar-driven* generator must derive or be told them.

**Grammar-based test generation.** Generators that feed real parsers must emit *valid* inputs; token
separation is a recognized obligation. **Havrikov & Zeller, "Systematically Covering Input Structure,"
ASE 2019** (the *k-path* algorithm) and the *Fuzzing Book* (Zeller et al.) treat the grammar as the
source of truth and rely on terminal definitions to render leaves — where, in a character-level
grammar, the separation discipline must be handled to avoid fusing adjacent terminals.
([Havrikov & Zeller, ASE 2019](https://havrikov.github.io/publications/ase19-preprint.pdf))

**Mapping to PGEN.** PGEN's generator has Oppen/Wadler's *job* (structure → text) but **none of the
discipline** — `generate_from_regex_hir` concatenates token strings with no separation reasoning, and
treats regex anchors (`HirKind::Look`) as empty. That is the gap.

## 3. Synthesis — the design the literature points to

1. **State the property, not a patch:** the generator must satisfy the **round-trip invariant** —
   `render(t)` re-lexes/re-parses to `t`'s token stream. (This is what pretty-printer correctness and
   parser round-tripping both demand.)
2. **Derive separation from the token regexes (automatic, parser-agnostic).** The generator owns every
   token's regex, so for adjacent emitted tokens A then B it can compute, from the regexes alone,
   whether the concatenation re-lexes faithfully (does B extend A's maximal munch? does A's terminator —
   e.g. a `(\n|$)` line-comment end — require a concrete newline rather than the zero-width `$`?). If
   not, insert the **minimal** separator (space, then newline) that restores it. This covers
   word-fusion, number-fusion, operator-fusion, and the comment-newline case **uniformly**.
3. **Declarative escape hatch — lexical (follow-restriction) annotations** (SDF2-style) for constraints
   derivation can't infer. This is the "lexical annotations" surface; most grammars should need few or
   none.
4. **Honor anchors as assertions, not empties.** `$`/`^`/`\b` are position assertions; the generator
   must not select a pure end-anchor alternation branch when more output follows — prefer the concrete
   branch. (Directly fixes the `(\n|$)` comment bug.)
5. **Verify with the parser.** The certificate-coverage gate already re-parses generated output, so it
   *is* the round-trip verifier — no new verifier needed.
6. **Subsume the prototype.** `enforce_word_boundary_spacing` becomes one derived case (the `\b` follow
   restriction); the flag and the comment-newline special case retire as instances.

**Direction — BOTH, generation-dominant today (per the director, refined 2026-06-06; confirmed by the
literature).** Lexical-surface constraints are direction-neutral: SDF uses the *same* follow
restrictions to disambiguate **parsing**, and an unparser uses the same boundary facts to render
**generation** faithfully. So lexical annotations can steer **both**. In PGEN today they are needed for
**generation far more often**, because the parser already supplies maximal munch (covering most of its
side) while the generator has no lexical discipline. The design (`.2`) is therefore generation-side
enforcement, deliberately built so the *same* declarative annotations can also serve parse-time
disambiguation if maximal munch ever proves insufficient.

## 4. Do-not-adopt / scope guards

- **Not** a full pretty-printer (no aesthetic layout, indentation, line-width optimization) — only
  *faithfulness* separation. Aesthetics are out of scope.
- **Not** scannerless GLR or a parser rewrite — the parser's maximal munch already works; we touch only
  the generator.
- Keep it **parser-agnostic** — derivation from regexes + a general annotation, never SV-specific.

## Sources

- [Maximal munch — Wikipedia](https://en.wikipedia.org/wiki/Maximal_munch)
- [Visser, Scannerless Generalized-LR Parsing (1997)](https://researchr.org/publication/Visser97-SGLR)
- [van den Brand, Scheerder, Vinju & Visser, Disambiguation Filters for SGLR (CC 2002)](https://homepages.cwi.nl/~jurgenv/papers/CC-2002.pdf)
- [Amorim & Visser, Multi-purpose Syntax Definition with SDF3 (SEFM 2020)](https://eelcovisser.org/publications/2020/AmorimV20.pdf)
- [Oppen, Prettyprinting (ACM TOPLAS 1980)](https://dl.acm.org/doi/pdf/10.1145/357114.357115)
- [Wadler, A prettier printer (2003)](https://www.semanticscholar.org/paper/A-prettier-printer-Wadler-Kilmer/23e0f52ebdd5dad33e61009afce1ecae3309eefb)
- [Havrikov & Zeller, Systematically Covering Input Structure (ASE 2019)](https://havrikov.github.io/publications/ase19-preprint.pdf)
