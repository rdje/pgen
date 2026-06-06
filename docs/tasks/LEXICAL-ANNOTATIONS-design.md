# LEXICAL-ANNOTATIONS — design (leaf `.2`)

> Companion to [`LEXICAL-ANNOTATIONS.md`](LEXICAL-ANNOTATIONS.md) and the
> [SOTA synthesis](LEXICAL-ANNOTATIONS-research-synthesis.md). This is the design only — **no code**;
> implementation is `.3`, verification is `.4`. Decisions here are within the agreed principles
> (parser-agnostic, derive-don't-hardcode, no hacks); the one explicit director decision (the EBNF
> **notation**) is called out and deferred.

## 1. The contract — the faithful-rendering invariant

A generator produces an ordered token sequence `T = [t₁ … tₙ]` (the terminal leaves of the generated
tree). It renders them to text `R = surface(t₁) · sep₁ · surface(t₂) · … · sepₙ₋₁ · surface(tₙ)`.

> **Invariant (LEX-FAITHFUL):** re-lexing/re-parsing `R` recovers `T` (and the tree). The generator
> must choose each `sepᵢ` to be the **minimal** separation that preserves the boundary.

This is the unparsing/round-trip property (Oppen/Wadler) restricted to *faithfulness* (not aesthetics).
The certifying gate (`GRAMMAR-WELLFORMED.G.4`) is the end-to-end **verifier** — it already re-parses
every generated sample.

## 2. What breaks a boundary (taxonomy)

Both observed defects, and the whole class, reduce to **one mechanism: a greedy token regex matching
*past* its intended end.**

| # | Failure | Example | Why |
|---|---|---|---|
| F1 | **Fusion** — the previous token's regex extends into the next | `endprogram`·`module` → `endprogrammodule`; `1`·`2`→`12`; `<`·`<`→`<<` | the boundary chars continue a valid match of the *previous* token's regex |
| F2 | **Swallow** — the previous token lacks its terminator and consumes onward | `//x`·`;` → `//x;…` eaten to EOL | the comment regex `//[^\n]*(\n\|$)` greedily eats `;` because no `\n` is present |

F2 is F1 with the twist that the *previous token's own surface was incomplete* (its terminator was
dropped). So the design needs **two** obligations, applied in order.

## 3. Design — two derived obligations + one declarative escape hatch

### Obligation A — *intra-token faithfulness* (each surface is a complete, valid token)

When generating a single terminal's surface from its regex, the surface must be a token that is valid
**wherever it may appear**, not only at an input boundary. The current bug is the root cause of F2:
`generate_from_regex_hir` maps every anchor `HirKind::Look(_)` to the empty string and picks
alternation branches uniformly, so for `//[^\n]*(\n|$)` it can choose the zero-width `$` branch and
emit a comment with no newline — a token only valid at true EOF.

**Fix (derive, parser-agnostic):** treat regex anchors as the **assertions** they are, not free empties.
- `$` / `^` (end/start of input), `\A` / `\z`: a zero-width branch that *asserts a boundary*. When
  generating a token that may be followed by more output (the common case), the generator must **not**
  realize such a branch as empty — it must prefer a sibling branch that produces a concrete realization.
  For `(\n|$)` that means choosing `\n`. (A trailing `\n` is harmless even when the token *is* last, so
  "prefer the concrete branch" is always safe; no need to know whether we're at EOF.)
- `\b` (word boundary): this is an *inter-token* constraint, not intra-token — it asserts the next
  character is a non-word char. It feeds Obligation B (it is exactly what `enforce_word_boundary_spacing`
  reads today).

This makes every emitted surface a complete token (e.g. the comment surface becomes `//x\n`). F2 is
fixed at the source.

### Obligation B — *inter-token faithfulness* (each boundary preserves separation)

Between two adjacent complete surfaces `sₐ` (regex `Rₐ`) and `s_b` (regex `R_b`), insert the **minimal**
separator so `Rₐ` does not over-match across the boundary.

**The boundary test (derived from the regexes — the heart of the design):**
> `sep` is safe iff the longest match of `Rₐ`, anchored at the start of `sₐ · sep · s_b`, equals `sₐ`.
> (If `Rₐ` matches longer, it has eaten into `sep`/`s_b` → fusion/swallow.)

**Minimal-separator ladder** — try in increasing strength, pick the first that passes the test:
`""`  →  `" "` (space)  →  `"\n"` (newline)  →  *(grammar-supplied stronger separators if any)*.
- Word/number/operator fusion (F1) is broken by a space.
- A line-comment boundary (if Obligation A didn't already terminate it) needs a newline — the ladder
  reaches it automatically, because a space still leaves the comment regex eating onward.

**State required:** the generator must remember the **last emitted terminal's regex + surface** (it
already tracks the output's trailing char in `append_generated_segment`; this generalizes that to the
trailing *token*).

**Soundness:** PGEN is scannerless and matches terminal regexes greedily with no intra-regex
backtracking, so a terminal's match at a position is exactly its greedy longest match — which is what
the local test computes. Grammar *context* selects *which* terminal is tried, never how far a tried
regex matches. So the local test is sound for PEG; the gate remains the end-to-end backstop.

### Obligation C — declarative escape hatch (*lexical annotations* proper)

For constraints the regexes don't capture (grammar-specific layout, or a deliberately stronger
follow rule), an optional declarative **follow restriction** (SDF2-style): "token `X` may not be
immediately followed by character-class `C`" (forcing a separator) / "must be followed by `C`-or-EOF."
Obligation B consults this table in addition to the derived test. Most grammars need **none** — the
derivation covers the common cases. **Notation deferred** (see §6).

## 4. Subsuming `enforce_word_boundary_spacing`

The flag is a crude special case of Obligation B: "insert a space when output ends in a word char and
the next starts with one." The regex boundary test generalizes it (word + number + operator + comment,
minimal separator each). Plan: implement A+B; route the word-char case through the general test; the
flag is removed (or kept only as a force-on alias during migration). The comment-newline special case
(`G.4.7` slice 3) likewise retires as a derived instance.

## 5. Always-on vs. opt-out

LEX-FAITHFUL should be **on by default** for valid-stimuli generation — invalid lexical surface is
never wanted there. The only legitimate opt-out is **negative-test generation** that *intentionally*
produces malformed lexical structure; that mode may disable it. So the `enforce_word_boundary_spacing`
boolean is replaced by a faithfulness mode that defaults on. (Decision taken here; reversible.)

## 6. The notation question (DEFERRED — director decision)

Return annotations use `->`, semantic annotations use `@`; Obligation C needs its own EBNF syntax. Only
the *rare explicit* follow restriction needs it (A+B are derivation, no syntax). Candidates: adopt/adapt
**SDF's `-/-`** (`A -/- [chars]` — literature-grounded) vs. a fresh sigil. Constraints: distinct from
`->`/`@`; attaches at the terminal/token level; lightweight. **Decide with the director before `.3`
touches the EBNF surface.** (Tracked in the tree's `.2` notes.)

## 7. Where it lands (guidance for `.3`)

- **Obligation A:** `stimuli_generator.rs::generate_from_regex_hir` — the `HirKind::Look` arm + the
  `HirKind::Alternation` branch selection (prefer concrete over zero-width-anchor branches).
- **Obligation B:** the terminal-emission path / `append_generated_segment` — track the last terminal's
  regex; replace the char-class check with the regex boundary test + minimal-separator ladder.
- **Obligation C:** EBNF surface (notation TBD) → annotation compiler → a follow-restriction table read
  by Obligation B.
- **Verification:** none new — the `--report-certificate-coverage` gate re-parses output.

All parser-AGNOSTIC (derivation from any grammar's regexes; every generated parser benefits). Bidirectional
bonus: the same follow-restriction table could later feed parse-time disambiguation.

## 8. Acceptance (for `.3`/`.4`)

- `.3` implements A + B (and C's table if the notation is decided), regenerates parsers (commit codegen
  only).
- `.4`: the certificate gate shows SV `sample_parse_failures → 0` from the **general** mechanism (not
  per-case patches); then every grammar with a registered parser (ties to `GRAMMAR-WELLFORMED` Phase H);
  add round-trip/golden tests; retire the word-boundary flag + comment special case.
